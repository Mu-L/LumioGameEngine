use crate::abi_generated::{
    VoxelBlockReadCellResult, VoxelBlockReadResult, VoxelBlockWriteEntry, VoxelBoxRequest,
    VoxelColumnRequest, VoxelOverlapHit, VoxelOverlapRequest, VoxelOverlapResult, VoxelPresence,
    VoxelQueryResolution, VoxelRaycastRequest, VoxelRaycastResult, VoxelSectionKey,
    VoxelSectionRevisionResult, VoxelSectionSegment, VoxelSweepRequest, VoxelSweepResult,
    VoxelWorldCoordinate, VoxelWorldPoint, VoxelWriteReceipt,
};
use crate::LumioStatus;
use lumio_voxel_contracts::sha256;
use lumio_voxel_domain::block::{
    BlockId, BlockType, CellOffset, MaterialClass as CatalogMaterialClass, OfficialCatalog,
};
use lumio_voxel_domain::config_snapshot::{
    HostCapabilitySet, VoxelConfigInput, VoxelConfigSnapshot, CONFIG_TABLE_SCHEMA,
    HOST_CAPABILITY_SCHEMA,
};
use lumio_voxel_domain::key::SectionId;
use lumio_voxel_domain::publication::PublishedStateRoot;
use lumio_voxel_domain::revision::{RevisionStamp, REVISION_STAMP_SCHEMA};
use lumio_voxel_domain::section::{
    SectionDeltaBuilder, SectionPage, SectionPayload, SectionPayloadEnvelope, SectionSlot,
    SectionStorage,
};
use lumio_voxel_ops::async_support::{OriginEnvelope, OriginToken};
use lumio_voxel_ops::mutation::{MutationEntry, MutationRequest, PreparedMutation};
use lumio_voxel_ops::query::{BlockReadSection, BlockReadWorld, VoxelQueryRequest};
use lumio_voxel_project::physics_query as physics;
use lumio_voxel_world::port::VoxelWorldPortAdapter;
use lumio_voxel_world::world::{
    PinBudget, PinId, RegionPinManager, VoxelWorld, WorldCommand, WorldConfigAdapter,
    WorldDescriptor,
};
use std::collections::BTreeMap;
use std::ffi::c_void;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct SectionKey(i32, u8, i32);

impl SectionKey {
    fn from_abi(key: VoxelSectionKey) -> Result<Self, &'static str> {
        if key.y > 15 {
            return Err("section_y_out_of_range");
        }
        Ok(Self(key.x, key.y, key.z))
    }

    fn to_abi(self) -> VoxelSectionKey {
        VoxelSectionKey::new(self.0, self.1, self.2)
    }

    fn id(self) -> String {
        format!("s:{}:{}:{}", self.0, self.1, self.2)
    }
}

#[derive(Clone, Debug)]
struct SectionState {
    presence: VoxelPresence,
    revision: u64,
    storage: Option<SectionStorage>,
}

#[derive(Clone, Copy, Debug)]
struct WriteEntry {
    section: SectionKey,
    offset: u16,
    block_id: u32,
}

struct PreparedToken {
    transaction_id: u64,
    request: MutationRequest,
    entries: Vec<WriteEntry>,
    mutation: Option<PreparedMutation>,
    receipts: Vec<VoxelWriteReceipt>,
    terminal_error: Option<&'static str>,
}

struct PinToken {
    id: PinId,
}

static NEXT_PROVIDER_ID: AtomicU64 = AtomicU64::new(1);

/// Native-owned integration adapter over the paired VoxelEngine public APIs.
///
/// `VoxelWorld` owns lifecycle, publication, query routing, and mutation routing. The
/// immutable `BlockReadWorld` is rebuilt from the same Section state after publication so the
/// ABI's caller-buffer shape can use the paired block-read API without leaking ownership.
pub struct NativeVoxelProvider {
    world: VoxelWorld,
    block_world: BlockReadWorld,
    /// 材质类表：契约 `physicsQuery.materialClassTable.nativeEntry = world-creation-injection`
    /// ——只在创建世界时注入一次，既不是根表槽也不是查询参数。未注入的世界不得回答任何物理查询
    /// （`resolvedBeforeQueryable` / `noBuiltInFallback`）。
    materials: Option<CatalogMaterialClasses>,
    sections: BTreeMap<SectionKey, SectionState>,
    prepared: BTreeMap<usize, Box<PreparedToken>>,
    transactions: BTreeMap<u64, usize>,
    pin_tokens: BTreeMap<usize, Box<PinToken>>,
    next_query_id: u64,
}

impl Default for NativeVoxelProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeVoxelProvider {
    pub fn new() -> Self {
        let provider_id = NEXT_PROVIDER_ID.fetch_add(1, Ordering::Relaxed);
        let world_id = format!("native-voxel-world-{provider_id}");
        let context_id = format!("native-voxel-context-{provider_id}");
        let snapshot = approved_snapshot("lumio-engine-native");
        let mut world = VoxelWorld::create(
            WorldDescriptor {
                role: "Authority".to_string(),
                world_context_id: context_id,
                capabilities: vec!["Native".to_string(), "ReferenceVoxel".to_string()],
                config: WorldConfigAdapter { world_id },
            },
            snapshot,
        )
        .expect("approved paired VoxelEngine snapshot must create a world");
        world.set_region_pin_manager(RegionPinManager::from_budget(PinBudget::new(
            usize::MAX,
            usize::MAX,
        )));
        drive_lifecycle(&mut world);
        Self {
            world,
            block_world: BlockReadWorld::new(),
            materials: None,
            sections: BTreeMap::new(),
            prepared: BTreeMap::new(),
            transactions: BTreeMap::new(),
            pin_tokens: BTreeMap::new(),
            next_query_id: 1,
        }
    }

    /// 宿主在创建体素世界时注入材质类表（契约 `physicsQuery.materialClassTable`）。
    ///
    /// 唯一来源是官方方块目录 `blockCatalog` 的 `materialClass` 一列——Native 不持第二份表、
    /// 不做内置回落，也不把目录做成根表槽或逐次查询参数。
    pub fn with_material_classes(catalog: OfficialCatalog) -> Self {
        let mut provider = Self::new();
        provider.materials = Some(CatalogMaterialClasses { catalog });
        provider
    }

    pub fn world_state(&self) -> lumio_voxel_world::world::WorldStateView {
        self.world.state_view()
    }

    /// 已发布切面的 WorldRevision 与逐 Section 的 SectionRevision。
    ///
    /// 用于证明查询是只读的（契约 `physicsQuery.readOnly`）：查询前后这两者必须逐一不变。
    pub fn published_revisions(&self) -> (u64, BTreeMap<String, u64>) {
        let view = self.world.publication_authority().capture();
        (
            view.stamp().world_revision,
            view.stamp().section_revision_set.clone(),
        )
    }

    pub fn as_opaque_ptr(&mut self) -> *mut c_void {
        self as *mut Self as *mut c_void
    }

    pub fn seed_ready_section(&mut self, key: VoxelSectionKey, revision: u64, block_id: u32) {
        self.seed(
            key,
            SectionState {
                presence: VoxelPresence::Ready,
                revision,
                storage: Some(SectionStorage::uniform(BlockId::from_raw(block_id))),
            },
        );
    }

    pub fn seed_unchanged_section(&mut self, key: VoxelSectionKey, revision: u64, block_id: u32) {
        self.seed(
            key,
            SectionState {
                presence: VoxelPresence::Unchanged,
                revision,
                storage: Some(SectionStorage::uniform(BlockId::from_raw(block_id))),
            },
        );
    }

    pub fn seed_pending_section(&mut self, key: VoxelSectionKey, revision: u64) {
        self.seed_missing(key, revision, VoxelPresence::Pending);
    }

    pub fn seed_unavailable_section(&mut self, key: VoxelSectionKey, revision: u64) {
        self.seed_missing(key, revision, VoxelPresence::Unavailable);
    }

    fn seed_missing(&mut self, key: VoxelSectionKey, revision: u64, presence: VoxelPresence) {
        self.seed(
            key,
            SectionState {
                presence,
                revision,
                storage: None,
            },
        );
    }

    fn seed(&mut self, key: VoxelSectionKey, state: SectionState) {
        let key = SectionKey::from_abi(key).expect("seed key must be canonical");
        self.sections.insert(key, state);
        self.rebuild_block_world();
        // The seed helpers also model a residency update that may be rejected by the
        // world publication transition (for example Ready -> Pending after a pin is
        // ready). Keep the caller-buffer source at the attempted state so the ABI
        // guard proves that such a stale/missing result is rejected rather than leaked.
        if let Err(error) = self.publish_section(key) {
            let attempted_missing = self.sections.get(&key).is_some_and(|state| {
                matches!(
                    state.presence,
                    VoxelPresence::Pending | VoxelPresence::Unavailable
                )
            }) && self
                .world
                .region_pin_manager()
                .is_some_and(|manager| manager.validate_presence(&key.id(), "Pending").is_err());
            if !attempted_missing {
                panic!("seed publication through paired VoxelEngine must succeed: {error}");
            }
        }
    }

    fn rebuild_block_world(&mut self) {
        let entries = self.sections.iter().map(|(key, state)| {
            let section = BlockReadSection::from_parts(
                presence_name(state.presence),
                Some(state.revision),
                state.storage.clone(),
            )
            .expect("ABI presence and SectionStorage must satisfy BlockReadWorld");
            (key.id(), section)
        });
        self.block_world = BlockReadWorld::from_sections(entries)
            .expect("canonical adapter Section ids must build BlockReadWorld");
    }

    fn publish_section(&mut self, changed: SectionKey) -> Result<(), &'static str> {
        let view = self.world.publication_authority().capture();
        let mut directory = lumio_voxel_domain::section::SectionDirectoryBuilder::new();
        let mut revisions = BTreeMap::new();
        for (key, state) in &self.sections {
            directory
                .insert(&key.id(), section_slot(*key, state)?)
                .map_err(|e| e.error_id())?;
            revisions.insert(key.id(), state.revision);
        }
        let stamp = RevisionStamp {
            schema_id: REVISION_STAMP_SCHEMA,
            world_id: view.stamp().world_id.clone(),
            context_id: view.stamp().context_id.clone(),
            generation: view.stamp().generation,
            world_revision: view.stamp().world_revision,
            section_revision_set: revisions,
        };
        let root =
            PublishedStateRoot::new(stamp, directory.freeze(), view.dirty_frontier().clone());
        let state = self.sections.get(&changed).ok_or("unknown_section_key")?;
        let mut delta = SectionDeltaBuilder::new(view.directory());
        delta
            .stage((changed.id(), section_slot(changed, state)?))
            .map_err(|e| e.error_id())?;
        let replacement = delta.freeze().map_err(|e| e.error_id())?;
        let mut publication = self
            .world
            .publication_authority()
            .prepare(
                world_revision(view.stamp().world_revision)?,
                root,
                replacement,
            )
            .map_err(|e| e.error_id())?;
        let token = publication.seal().map_err(|e| e.error_id())?;
        self.world
            .publication_authority()
            .publish_once(token)
            .map_err(|e| e.error_id())?;
        Ok(())
    }

    fn read_cell(
        &mut self,
        coordinate: VoxelWorldCoordinate,
    ) -> Result<VoxelBlockReadResult, &'static str> {
        let section = coordinate_section(coordinate)?;
        self.ensure_world_query(&[section])?;
        self.read_cell_cached(coordinate)
    }

    fn read_cell_cached(
        &mut self,
        coordinate: VoxelWorldCoordinate,
    ) -> Result<VoxelBlockReadResult, &'static str> {
        let mut block_id = None;
        let result = if let Some(manager) = self.world.region_pin_manager() {
            self.block_world.read_cell_into_with_presence_guard(
                coordinate.x,
                i64::from(coordinate.y),
                coordinate.z,
                &mut block_id,
                manager,
            )
        } else {
            self.block_world.read_cell_into(
                coordinate.x,
                i64::from(coordinate.y),
                coordinate.z,
                &mut block_id,
            )
        }
        .map_err(|error| error.error_id())?;
        let presence = parse_presence(result.presence())?;
        Ok(VoxelBlockReadResult {
            presence,
            has_block_id: u8::from(block_id.is_some()),
            _reserved: [0; 3],
            block_id: block_id.map_or(0, BlockId::raw),
            section_revision: result.section_revision(),
        })
    }

    fn ensure_world_query(&mut self, sections: &[SectionKey]) -> Result<(), &'static str> {
        let state = self.world.state_view();
        let mut ids = sections.iter().map(|key| key.id()).collect::<Vec<_>>();
        ids.sort();
        ids.dedup();
        for chunk in ids.chunks(16) {
            let query_id = format!("native-query-{}", self.next_query_id);
            self.next_query_id = self.next_query_id.saturating_add(1);
            let origin = OriginToken::try_new(
                state.world_context_id().to_string(),
                state.instance_generation(),
                query_id.clone(),
                0,
                BTreeMap::new(),
                "VoxelCommit",
            )
            .map_err(|error| canonical_error_id(error.error_id()))?;
            let config_hash = self.world.config_hash().to_string();
            VoxelWorldPortAdapter::new(&mut self.world)
                .query(OriginEnvelope {
                    origin,
                    config_hash,
                    payload: VoxelQueryRequest {
                        query_id,
                        world_id: state.world_id().to_string(),
                        context: state.world_context_id().to_string(),
                        section_ids: chunk.to_vec(),
                        cancel: false,
                    },
                })
                .map(|_| ())
                .map_err(|error| canonical_error_id(error.error_id()))?;
        }
        Ok(())
    }

    fn ensure_world_query_range(
        &mut self,
        min: VoxelWorldCoordinate,
        max: VoxelWorldCoordinate,
    ) -> Result<(), &'static str> {
        let mut batch = [SectionKey(0, 0, 0); 16];
        let mut used = 0_usize;
        for y in (min.y / 16)..=(max.y / 16) {
            for z in min.z.div_euclid(16)..=max.z.div_euclid(16) {
                for x in min.x.div_euclid(16)..=max.x.div_euclid(16) {
                    batch[used] = SectionKey(x, y, z);
                    used += 1;
                    if used == batch.len() {
                        self.ensure_world_query(&batch)?;
                        used = 0;
                    }
                }
            }
        }
        if used > 0 {
            self.ensure_world_query(&batch[..used])?;
        }
        Ok(())
    }

    /// 物理查询的只读输入：一份已发布的不可变切面 + 创建世界时注入的材质类表。
    ///
    /// 走 `publication_authority().capture()` 而不是 `ensure_world_query`——查询不得改变
    /// 世界的任何一格（契约 `physicsQuery.readOnly`），所以这里不触发任何世界侧命令。
    fn physics_inputs(
        &self,
    ) -> Result<(physics::PhysicsWorld, &CatalogMaterialClasses), &'static str> {
        let materials = self
            .materials
            .as_ref()
            .ok_or("collision_behavior_not_from_material_table")?;
        let view = self.world.publication_authority().capture();
        // `Unchanged` 是零字节短票据，按不可变原图基线解析，不在查询里另建一份存储。
        let baseline = |id: &SectionId| {
            self.sections
                .get(&SectionKey(id.x(), id.y(), id.z()))
                .and_then(|state| state.storage.clone())
        };
        let world = physics::PhysicsWorld::from_published_view_with_baseline(&view, &baseline)
            .map_err(|error| error.error_id())?;
        Ok((world, materials))
    }

    /// 与逐格读一致地带上驻留守卫：同一个世界不得在读路径拒绝、却在物理路径放行。
    fn physics_query<'a>(
        &'a self,
        world: &'a physics::PhysicsWorld,
        materials: &'a CatalogMaterialClasses,
    ) -> physics::PhysicsQuery<'a, 'a> {
        match self.world.region_pin_manager() {
            Some(guard) => physics::PhysicsQuery::with_presence_guard(world, materials, guard),
            None => physics::PhysicsQuery::new(world, materials),
        }
    }
}

/// 材质类表的 Native 适配：只读官方方块目录的 `materialClass` 一列。
///
/// 契约 `physicsQuery.filter.source` 要求「阻挡与否只能来自材质类表」，所以这里没有、
/// 也不得有任何按 BlockId 分支的碰撞行为；解析不出材质类即 `unknown_material_class`
/// （由 `physics_query` 侧在 `class_for` 返回 `None` 时抛出），不得当作不阻挡。
struct CatalogMaterialClasses {
    catalog: OfficialCatalog,
}

impl physics::MaterialClassLookup for CatalogMaterialClasses {
    fn class_for(&self, block_type: BlockType) -> Option<physics::MaterialClass> {
        match self.catalog.get(block_type) {
            Ok(Some(definition)) => Some(match definition.material_class() {
                CatalogMaterialClass::Solid => physics::MaterialClass::Solid,
                CatalogMaterialClass::Liquid => physics::MaterialClass::Liquid,
            }),
            Ok(None) | Err(_) => None,
        }
    }
}

/// 契约 `voxel.enums.material_mask`：位序按 `materialClasses.v1Scope.classes` 声明顺序，
/// 从最低位起一类占一位。第 2 位及以上未分配，必须为 0。
const MATERIAL_MASK_SOLID: u32 = 1;
const MATERIAL_MASK_LIQUID: u32 = 2;
const MATERIAL_MASK_ASSIGNED_BITS: u32 = MATERIAL_MASK_SOLID | MATERIAL_MASK_LIQUID;

/// 掩码 0 不是「全选」也不是错误码，而是不匹配任何材质类（`zeroMatchesNothing`）——
/// 空掩码交给查询侧自然落到 Miss，而 Unresolved 仍然优先于 Miss。
fn material_mask(raw: u32) -> Result<physics::MaterialMask, &'static str> {
    if raw & !MATERIAL_MASK_ASSIGNED_BITS != 0 {
        return Err("unknown_material_class");
    }
    let mut mask = physics::MaterialMask::empty();
    if raw & MATERIAL_MASK_SOLID != 0 {
        mask |= physics::MaterialMask::solid();
    }
    if raw & MATERIAL_MASK_LIQUID != 0 {
        mask |= physics::MaterialMask::liquid();
    }
    Ok(mask)
}

fn physics_point(point: VoxelWorldPoint) -> physics::Vec3 {
    physics::Vec3::new(point.x, point.y, point.z)
}

fn abi_point(point: physics::Vec3) -> VoxelWorldPoint {
    VoxelWorldPoint {
        x: point.x(),
        y: point.y(),
        z: point.z(),
    }
}

fn abi_section_key(section: SectionId) -> VoxelSectionKey {
    VoxelSectionKey::new(section.x(), section.y(), section.z())
}

fn abi_cell(cell: physics::CellCoord) -> Result<VoxelWorldCoordinate, &'static str> {
    let y = u8::try_from(cell.y()).map_err(|_| "world_y_out_of_range")?;
    Ok(VoxelWorldCoordinate::new(cell.x(), y, cell.z()))
}

fn approved_snapshot(label: &str) -> Arc<VoxelConfigSnapshot> {
    let config = VoxelConfigInput {
        schema_id: CONFIG_TABLE_SCHEMA,
        host_capability_schema_id: HOST_CAPABILITY_SCHEMA,
        config_hash: hex32(&sha256(label.as_bytes())),
        host_capability: HostCapabilitySet::from_names(["Native", "ReferenceVoxel"]),
        start_capabilities: vec!["Native".to_string(), "ReferenceVoxel".to_string()],
        key_material: None,
    };
    VoxelConfigSnapshot::load(&config).expect("paired VoxelEngine config snapshot must be accepted")
}

fn drive_lifecycle(world: &mut VoxelWorld) {
    for (event, to) in [
        ("Initialize", "Initialized"),
        ("Prime", "Ready"),
        ("Start", "Running"),
    ] {
        let state = world.state_view();
        let origin = OriginToken::try_new(
            state.world_context_id().to_string(),
            state.instance_generation(),
            format!("native-lifecycle-{event}"),
            0,
            BTreeMap::new(),
            "VoxelCommit",
        )
        .expect("lifecycle origin");
        world
            .endpoint()
            .admit(WorldCommand::Lifecycle { event, to, origin })
            .expect("paired VoxelEngine lifecycle must admit");
    }
}

fn world_revision(n: u64) -> Result<lumio_voxel_domain::revision::WorldRevision, &'static str> {
    Ok(lumio_voxel_domain::revision::WorldRevision::from_raw(n))
}

fn section_slot(key: SectionKey, state: &SectionState) -> Result<SectionSlot, &'static str> {
    match state.presence {
        VoxelPresence::Ready => {
            let storage = state.storage.as_ref().ok_or("section_unavailable")?;
            let id = lumio_voxel_domain::key::SectionId::parse(&key.id())
                .map_err(|_| "unknown_section_key")?;
            let envelope = SectionPayloadEnvelope::encode_full(id, state.revision, storage);
            let payload = SectionPayload::from_pages_with_storage(
                [SectionPage::new(
                    "Dense",
                    "None",
                    envelope.payload().to_vec(),
                    sha256(envelope.payload()),
                )],
                Some(storage.clone()),
            )
            .map_err(|error| error.error_id())?;
            Ok(SectionSlot::ready(payload))
        }
        VoxelPresence::Unchanged => Ok(SectionSlot::unchanged()),
        VoxelPresence::Pending => Ok(SectionSlot::pending()),
        VoxelPresence::Unavailable => Ok(SectionSlot::unavailable()),
    }
}

fn presence_name(presence: VoxelPresence) -> &'static str {
    match presence {
        VoxelPresence::Ready => "Ready",
        VoxelPresence::Unchanged => "Unchanged",
        VoxelPresence::Pending => "Pending",
        VoxelPresence::Unavailable => "Unavailable",
    }
}

fn parse_presence(name: &str) -> Result<VoxelPresence, &'static str> {
    match name {
        "Ready" => Ok(VoxelPresence::Ready),
        "Unchanged" => Ok(VoxelPresence::Unchanged),
        "Pending" => Ok(VoxelPresence::Pending),
        "Unavailable" => Ok(VoxelPresence::Unavailable),
        _ => Err("cell_read_missing_presence"),
    }
}

fn coordinate_section(coordinate: VoxelWorldCoordinate) -> Result<SectionKey, &'static str> {
    Ok(SectionKey(
        coordinate.x.div_euclid(16),
        coordinate.y / 16,
        coordinate.z.div_euclid(16),
    ))
}

fn provider<'a>(world: *mut c_void) -> Result<&'a mut NativeVoxelProvider, i32> {
    if world.is_null() {
        return Err(LumioStatus::InvalidArgument as i32);
    }
    // SAFETY: the opaque pointer is created by NativeVoxelProvider::as_opaque_ptr.
    Ok(unsafe { &mut *world.cast::<NativeVoxelProvider>() })
}

fn canonical_error_id(error: &str) -> &'static str {
    match error {
        "unknown_section_key" => "unknown_section_key",
        "unknown_chunk_key" => "unknown_chunk_key",
        "section_y_out_of_range" => "section_y_out_of_range",
        "coordinate_out_of_bounds" => "coordinate_out_of_bounds",
        "section_unavailable" => "section_unavailable",
        "stale_section_revision" => "stale_section_revision",
        "read_budget_exceeded" => "read_budget_exceeded",
        "read_result_missing_revision" => "read_result_missing_revision",
        "write_batch_too_large" => "write_batch_too_large",
        "unstructured_mutation_entry" => "unstructured_mutation_entry",
        "cell_offset_out_of_range" => "cell_offset_out_of_range",
        "residency_pin_exceeds_budget" => "residency_pin_exceeds_budget",
        "pin_region_not_ready" => "pin_region_not_ready",
        "pinned_section_evicted" => "pinned_section_evicted",
        "pinned_read_returned_pending" => "pinned_read_returned_pending",
        "world_y_out_of_range" => "world_y_out_of_range",
        "cell_read_missing_presence" => "cell_read_missing_presence",
        _ => "InvalidHandle",
    }
}

fn status_for_error(error: &str) -> i32 {
    match error {
        "unknown_section_key" => crate::abi_generated::VOXEL_ERROR_UNKNOWN_SECTION_KEY,
        "unknown_chunk_key" => crate::abi_generated::VOXEL_ERROR_UNKNOWN_CHUNK_KEY,
        "section_y_out_of_range" => crate::abi_generated::VOXEL_ERROR_SECTION_Y_OUT_OF_RANGE,
        "coordinate_out_of_bounds" => crate::abi_generated::VOXEL_ERROR_COORDINATE_OUT_OF_BOUNDS,
        "section_unavailable" => crate::abi_generated::VOXEL_ERROR_SECTION_UNAVAILABLE,
        "stale_section_revision" => crate::abi_generated::VOXEL_ERROR_STALE_SECTION_REVISION,
        "palette_overflow" => crate::abi_generated::VOXEL_ERROR_PALETTE_OVERFLOW,
        "section_encoding_mismatch" => crate::abi_generated::VOXEL_ERROR_SECTION_ENCODING_MISMATCH,
        "section_digest_mismatch" => crate::abi_generated::VOXEL_ERROR_SECTION_DIGEST_MISMATCH,
        "dirty_section_not_durable" => crate::abi_generated::VOXEL_ERROR_DIRTY_SECTION_NOT_DURABLE,
        "lighting_in_payload" => crate::abi_generated::VOXEL_ERROR_LIGHTING_IN_PAYLOAD,
        "chunk_carries_data" => crate::abi_generated::VOXEL_ERROR_CHUNK_CARRIES_DATA,
        "unknown_material_class" => crate::abi_generated::VOXEL_ERROR_UNKNOWN_MATERIAL_CLASS,
        "material_class_not_a_cell_lane" => {
            crate::abi_generated::VOXEL_ERROR_MATERIAL_CLASS_NOT_A_CELL_LANE
        }
        "liquid_auto_propagation_unsupported" => {
            crate::abi_generated::VOXEL_ERROR_LIQUID_AUTO_PROPAGATION_UNSUPPORTED
        }
        "cross_material_face_merge" => crate::abi_generated::VOXEL_ERROR_CROSS_MATERIAL_FACE_MERGE,
        "entity_binding_missing" => crate::abi_generated::VOXEL_ERROR_ENTITY_BINDING_MISSING,
        "entity_binding_orphan" => crate::abi_generated::VOXEL_ERROR_ENTITY_BINDING_ORPHAN,
        "entity_binding_type_mismatch" => {
            crate::abi_generated::VOXEL_ERROR_ENTITY_BINDING_TYPE_MISMATCH
        }
        "entity_binding_not_sparse" => crate::abi_generated::VOXEL_ERROR_ENTITY_BINDING_NOT_SPARSE,
        "business_data_in_payload" => crate::abi_generated::VOXEL_ERROR_BUSINESS_DATA_IN_PAYLOAD,
        "binding_commit_split" => crate::abi_generated::VOXEL_ERROR_BINDING_COMMIT_SPLIT,
        "block_type_scope_violation" => {
            crate::abi_generated::VOXEL_ERROR_BLOCK_TYPE_SCOPE_VIOLATION
        }
        "system_reserved_type_misuse" => {
            crate::abi_generated::VOXEL_ERROR_SYSTEM_RESERVED_TYPE_MISUSE
        }
        "room_local_type_without_mapping" => {
            crate::abi_generated::VOXEL_ERROR_ROOM_LOCAL_TYPE_WITHOUT_MAPPING
        }
        "player_type_declares_behavior" => {
            crate::abi_generated::VOXEL_ERROR_PLAYER_TYPE_DECLARES_BEHAVIOR
        }
        "palette_reclaim_before_escalation" => {
            crate::abi_generated::VOXEL_ERROR_PALETTE_RECLAIM_BEFORE_ESCALATION
        }
        "dead_palette_entry_in_payload" => {
            crate::abi_generated::VOXEL_ERROR_DEAD_PALETTE_ENTRY_IN_PAYLOAD
        }
        "delta_base_revision_mismatch" => {
            crate::abi_generated::VOXEL_ERROR_DELTA_BASE_REVISION_MISMATCH
        }
        "delta_used_for_first_delivery" => {
            crate::abi_generated::VOXEL_ERROR_DELTA_USED_FOR_FIRST_DELIVERY
        }
        "unresolved_hit_treated_as_air" => {
            crate::abi_generated::VOXEL_ERROR_UNRESOLVED_HIT_TREATED_AS_AIR
        }
        "unresolved_hit_treated_as_solid" => {
            crate::abi_generated::VOXEL_ERROR_UNRESOLVED_HIT_TREATED_AS_SOLID
        }
        "query_buffer_overflow" => crate::abi_generated::VOXEL_ERROR_QUERY_BUFFER_OVERFLOW,
        "query_result_divergence" => crate::abi_generated::VOXEL_ERROR_QUERY_RESULT_DIVERGENCE,
        "collision_behavior_not_from_material_table" => {
            crate::abi_generated::VOXEL_ERROR_COLLISION_BEHAVIOR_NOT_FROM_MATERIAL_TABLE
        }
        "query_mutates_world" => crate::abi_generated::VOXEL_ERROR_QUERY_MUTATES_WORLD,
        "world_y_out_of_range" => crate::abi_generated::VOXEL_ERROR_WORLD_Y_OUT_OF_RANGE,
        "block_catalog_not_dense" => crate::abi_generated::VOXEL_ERROR_BLOCK_CATALOG_NOT_DENSE,
        "block_catalog_name_reused" => crate::abi_generated::VOXEL_ERROR_BLOCK_CATALOG_NAME_REUSED,
        "block_catalog_row_incomplete" => {
            crate::abi_generated::VOXEL_ERROR_BLOCK_CATALOG_ROW_INCOMPLETE
        }
        "read_budget_exceeded" => crate::abi_generated::VOXEL_ERROR_READ_BUDGET_EXCEEDED,
        "read_result_missing_revision" => {
            crate::abi_generated::VOXEL_ERROR_READ_RESULT_MISSING_REVISION
        }
        "write_batch_too_large" => crate::abi_generated::VOXEL_ERROR_WRITE_BATCH_TOO_LARGE,
        "unstructured_mutation_entry" => {
            crate::abi_generated::VOXEL_ERROR_UNSTRUCTURED_MUTATION_ENTRY
        }
        "cell_offset_out_of_range" => crate::abi_generated::VOXEL_ERROR_CELL_OFFSET_OUT_OF_RANGE,
        "residency_pin_exceeds_budget" => {
            crate::abi_generated::VOXEL_ERROR_RESIDENCY_PIN_EXCEEDS_BUDGET
        }
        "pin_region_not_ready" => crate::abi_generated::VOXEL_ERROR_PIN_REGION_NOT_READY,
        "pinned_section_evicted" => crate::abi_generated::VOXEL_ERROR_PINNED_SECTION_EVICTED,
        "pinned_read_returned_pending" => {
            crate::abi_generated::VOXEL_ERROR_PINNED_READ_RETURNED_PENDING
        }
        "unknown_behavior_template" => crate::abi_generated::VOXEL_ERROR_UNKNOWN_BEHAVIOR_TEMPLATE,
        "cell_read_missing_presence" => {
            crate::abi_generated::VOXEL_ERROR_CELL_READ_MISSING_PRESENCE
        }
        "unregistered_block_type" => {
            crate::abi_generated::VOXEL_ERROR_UNREGISTERED_BLOCK_TYPE
        }
        "write_batch_partially_applied" => {
            crate::abi_generated::VOXEL_ERROR_WRITE_BATCH_PARTIALLY_APPLIED
        }
        "base_revision_on_full_encoding" => {
            crate::abi_generated::VOXEL_ERROR_BASE_REVISION_ON_FULL_ENCODING
        }
        "degenerate_query_shape" => {
            crate::abi_generated::VOXEL_ERROR_DEGENERATE_QUERY_SHAPE
        }
        _ => LumioStatus::InvalidArgument as i32,
    }
}

fn ffi(call: impl FnOnce() -> i32) -> i32 {
    catch_unwind(AssertUnwindSafe(call)).unwrap_or(LumioStatus::InvalidArgument as i32)
}

pub unsafe extern "C" fn block_read_cell(
    world: *mut c_void,
    coordinate: *const VoxelWorldCoordinate,
    out: *mut VoxelBlockReadCellResult,
) -> i32 {
    ffi(|| {
        if coordinate.is_null() || out.is_null() {
            return LumioStatus::InvalidArgument as i32;
        }
        let provider = match provider(world) {
            Ok(provider) => provider,
            Err(status) => return status,
        };
        let result = match provider.read_cell(unsafe { *coordinate }) {
            Ok(result) => result,
            Err(error) => return status_for_error(error),
        };
        unsafe {
            (*out).presence = result.presence;
            (*out).has_block_id = result.has_block_id;
            (*out).section_revision = result.section_revision;
            if result.has_block_id != 0 {
                (*out).block_id = result.block_id;
            }
        }
        LumioStatus::Success as i32
    })
}

pub unsafe extern "C" fn block_read_box(
    world: *mut c_void,
    request: *const c_void,
    out_cells: *mut VoxelBlockReadResult,
    cell_capacity: u32,
    out_cell_count: *mut u32,
    out_segments: *mut VoxelSectionSegment,
    segment_capacity: u32,
    out_segment_count: *mut u32,
    out_truncated: *mut u8,
) -> i32 {
    batch_read(
        world,
        request,
        BatchKind::Box,
        out_cells,
        cell_capacity,
        out_cell_count,
        out_segments,
        segment_capacity,
        out_segment_count,
        out_truncated,
    )
}

pub unsafe extern "C" fn block_read_column(
    world: *mut c_void,
    request: *const c_void,
    out_cells: *mut VoxelBlockReadResult,
    cell_capacity: u32,
    out_cell_count: *mut u32,
    out_segments: *mut VoxelSectionSegment,
    segment_capacity: u32,
    out_segment_count: *mut u32,
    out_truncated: *mut u8,
) -> i32 {
    batch_read(
        world,
        request,
        BatchKind::Column,
        out_cells,
        cell_capacity,
        out_cell_count,
        out_segments,
        segment_capacity,
        out_segment_count,
        out_truncated,
    )
}

#[derive(Clone, Copy)]
enum BatchKind {
    Box,
    Column,
}

#[allow(clippy::too_many_arguments)]
unsafe fn batch_read(
    world: *mut c_void,
    request: *const c_void,
    kind: BatchKind,
    out_cells: *mut VoxelBlockReadResult,
    cell_capacity: u32,
    out_cell_count: *mut u32,
    out_segments: *mut VoxelSectionSegment,
    segment_capacity: u32,
    out_segment_count: *mut u32,
    out_truncated: *mut u8,
) -> i32 {
    ffi(|| {
        if request.is_null()
            || out_cell_count.is_null()
            || out_segment_count.is_null()
            || out_truncated.is_null()
        {
            return LumioStatus::InvalidArgument as i32;
        }
        if (cell_capacity > 0 && out_cells.is_null())
            || (segment_capacity > 0 && out_segments.is_null())
        {
            return LumioStatus::InvalidArgument as i32;
        }
        let provider = match provider(world) {
            Ok(provider) => provider,
            Err(status) => return status,
        };
        let (min, max) = match kind {
            BatchKind::Box => {
                let req = unsafe { &*request.cast::<VoxelBoxRequest>() };
                if req.min.x > req.max.x || req.min.y > req.max.y || req.min.z > req.max.z {
                    return LumioStatus::InvalidArgument as i32;
                }
                let count = (i64::from(req.max.x) - i64::from(req.min.x) + 1)
                    .checked_mul(i64::from(req.max.y) - i64::from(req.min.y) + 1)
                    .and_then(|n| n.checked_mul(i64::from(req.max.z) - i64::from(req.min.z) + 1));
                if !matches!(
                    count,
                    Some(n)
                        if n >= 0
                            && n <= i64::from(
                                crate::abi_generated::VOXEL_MAX_CELLS_PER_READ_REQUEST
                            )
                ) {
                    return status_for_error("read_budget_exceeded");
                }
                (req.min, req.max)
            }
            BatchKind::Column => {
                let req = unsafe { &*request.cast::<VoxelColumnRequest>() };
                if req.min_y > req.max_y {
                    return LumioStatus::InvalidArgument as i32;
                }
                let count = usize::from(req.max_y - req.min_y) + 1;
                if count > crate::abi_generated::VOXEL_MAX_CELLS_PER_READ_REQUEST as usize {
                    return status_for_error("read_budget_exceeded");
                }
                (
                    VoxelWorldCoordinate::new(req.x, req.min_y, req.z),
                    VoxelWorldCoordinate::new(req.x, req.max_y, req.z),
                )
            }
        };
        if let Err(error) = provider.ensure_world_query_range(min, max) {
            return status_for_error(error);
        }
        // Validate the complete immutable source before invoking callbacks so a rejected
        // ready-pin read cannot expose a partial caller-buffer result.
        if let Some(manager) = provider.world.region_pin_manager() {
            let guarded = provider.block_world.read_box_with_presence_guard(
                (min.x, i64::from(min.y), min.z),
                (max.x, i64::from(max.y), max.z),
                manager,
            );
            if let Err(error) = guarded {
                return status_for_error(error.error_id());
            }
        }

        let cell_limit = cell_capacity as usize;
        let segment_limit = segment_capacity as usize;
        let visit = match kind {
            BatchKind::Box => provider.block_world.visit_box(
                (min.x, i64::from(min.y), min.z),
                (max.x, i64::from(max.y), max.z),
                |index, cell| {
                    if index < cell_limit {
                        let presence = parse_presence(cell.presence())
                            .expect("BlockReadWorld only stores contract presences");
                        unsafe {
                            out_cells.add(index).write(VoxelBlockReadResult {
                                presence,
                                has_block_id: u8::from(cell.block_id().is_some()),
                                _reserved: [0; 3],
                                block_id: cell.block_id().map_or(0, BlockId::raw),
                                section_revision: cell.section_revision(),
                            });
                        }
                    }
                },
                |index, segment| {
                    if index < segment_limit {
                        let id = segment.section_id();
                        let presence = parse_presence(segment.presence())
                            .expect("BlockReadWorld only stores contract presences");
                        unsafe {
                            out_segments.add(index).write(VoxelSectionSegment {
                                section_key: VoxelSectionKey::new(id.x(), id.y(), id.z()),
                                presence,
                                section_revision: segment.section_revision(),
                                first_result: segment.first_cell() as u32,
                                result_count: segment.cell_count() as u32,
                            });
                        }
                    }
                },
            ),
            BatchKind::Column => provider.block_world.visit_column(
                min.x,
                min.z,
                i64::from(min.y)..=i64::from(max.y),
                |index, cell| {
                    if index < cell_limit {
                        let presence = parse_presence(cell.presence())
                            .expect("BlockReadWorld only stores contract presences");
                        unsafe {
                            out_cells.add(index).write(VoxelBlockReadResult {
                                presence,
                                has_block_id: u8::from(cell.block_id().is_some()),
                                _reserved: [0; 3],
                                block_id: cell.block_id().map_or(0, BlockId::raw),
                                section_revision: cell.section_revision(),
                            });
                        }
                    }
                },
                |index, segment| {
                    if index < segment_limit {
                        let id = segment.section_id();
                        let presence = parse_presence(segment.presence())
                            .expect("BlockReadWorld only stores contract presences");
                        unsafe {
                            out_segments.add(index).write(VoxelSectionSegment {
                                section_key: VoxelSectionKey::new(id.x(), id.y(), id.z()),
                                presence,
                                section_revision: segment.section_revision(),
                                first_result: segment.first_cell() as u32,
                                result_count: segment.cell_count() as u32,
                            });
                        }
                    }
                },
            ),
        };
        let summary = match visit {
            Ok(summary) => summary,
            Err(error) => return status_for_error(error.error_id()),
        };
        unsafe {
            out_cell_count.write(summary.cell_count() as u32);
            out_segment_count.write(summary.segment_count() as u32);
            out_truncated.write(u8::from(
                summary.cell_count() > cell_limit || summary.segment_count() > segment_limit,
            ));
        }
        LumioStatus::Success as i32
    })
}

pub unsafe extern "C" fn block_write_prepare(
    world: *mut c_void,
    transaction_id: u64,
    entries: *const VoxelBlockWriteEntry,
    entry_count: u32,
    out_token: *mut *mut c_void,
) -> i32 {
    ffi(|| {
        if out_token.is_null() || (entry_count > 0 && entries.is_null()) {
            return LumioStatus::InvalidArgument as i32;
        }
        let provider = match provider(world) {
            Ok(provider) => provider,
            Err(status) => return status,
        };
        if entry_count > crate::abi_generated::VOXEL_MAX_ENTRIES_PER_WRITE_BATCH {
            return status_for_error("write_batch_too_large");
        }
        let state = provider.world.state_view();
        let input = if entry_count == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(entries, entry_count as usize) }
        };
        let mut mutation_entries = Vec::with_capacity(input.len());
        let mut owned_entries = Vec::with_capacity(input.len());
        for entry in input {
            let section = match SectionKey::from_abi(entry.section_key) {
                Ok(section) => section,
                Err(error) => return status_for_error(error),
            };
            let offset = match CellOffset::new(entry.cell_offset) {
                Ok(offset) => offset,
                Err(_) => return status_for_error("cell_offset_out_of_range"),
            };
            mutation_entries.push(MutationEntry::new(
                section.id(),
                offset,
                BlockId::from_raw(entry.block_id),
                entry.expected_section_revision,
            ));
            owned_entries.push(WriteEntry {
                section,
                offset: entry.cell_offset,
                block_id: entry.block_id,
            });
        }
        let request = MutationRequest::new(
            transaction_id.to_string(),
            state.world_id(),
            state.instance_generation(),
            mutation_entries,
        );
        if let Some(address) = provider.transactions.get(&transaction_id).copied() {
            if provider
                .prepared
                .get(&address)
                .is_some_and(|token| token.request == request)
            {
                unsafe { out_token.write(address as *mut c_void) };
                return LumioStatus::Success as i32;
            }
        }
        let origin = match OriginToken::try_new(
            state.world_context_id(),
            state.instance_generation(),
            transaction_id.to_string(),
            0,
            BTreeMap::new(),
            "VoxelCommit",
        ) {
            Ok(origin) => origin,
            Err(error) => return status_for_error(error.error_id()),
        };
        let config_hash = provider.world.config_hash().to_string();
        let mutation =
            match VoxelWorldPortAdapter::new(&mut provider.world).prepare_mutation(OriginEnvelope {
                origin,
                config_hash,
                payload: request.clone(),
            }) {
                Ok(envelope) => envelope.payload,
                Err(error) => return status_for_error(error.error_id()),
            };
        let mut token = Box::new(PreparedToken {
            transaction_id,
            request,
            entries: owned_entries,
            mutation: Some(mutation),
            receipts: Vec::new(),
            terminal_error: None,
        });
        let address = (&mut *token) as *mut PreparedToken as usize;
        provider.transactions.insert(transaction_id, address);
        provider.prepared.insert(address, token);
        unsafe { out_token.write(address as *mut c_void) };
        LumioStatus::Success as i32
    })
}

pub unsafe extern "C" fn block_write_commit(
    world: *mut c_void,
    token: *mut c_void,
    out_receipts: *mut VoxelWriteReceipt,
    receipt_capacity: u32,
    out_receipt_count: *mut u32,
) -> i32 {
    ffi(|| {
        if token.is_null()
            || out_receipt_count.is_null()
            || (receipt_capacity > 0 && out_receipts.is_null())
        {
            return LumioStatus::InvalidArgument as i32;
        }
        let provider = match provider(world) {
            Ok(provider) => provider,
            Err(status) => return status,
        };
        let address = token as usize;
        let required = match provider.prepared.get(&address) {
            Some(token) => token
                .entries
                .iter()
                .map(|entry| entry.section)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            None => return LumioStatus::InvalidArgument as i32,
        };
        if required > receipt_capacity as usize {
            unsafe { out_receipt_count.write(required as u32) };
            return LumioStatus::BufferTooSmall as i32;
        }
        let prepared = match provider.prepared.get_mut(&address) {
            Some(token) => token.mutation.take(),
            None => return LumioStatus::InvalidArgument as i32,
        };
        let Some(prepared) = prepared else {
            if let Some(error) = provider.prepared[&address].terminal_error {
                return status_for_error(error);
            }
            let receipts = &provider.prepared[&address].receipts;
            return write_receipts(receipts, out_receipts, receipt_capacity, out_receipt_count);
        };
        let transaction_id = provider.prepared[&address].transaction_id;
        let state = provider.world.state_view();
        let origin = match OriginToken::try_new(
            state.world_context_id(),
            state.instance_generation(),
            format!("commit-{transaction_id}"),
            0,
            BTreeMap::new(),
            "VoxelCommit",
        ) {
            Ok(origin) => origin,
            Err(error) => return status_for_error(error.error_id()),
        };
        let config_hash = provider.world.config_hash().to_string();
        let receipt = match VoxelWorldPortAdapter::new(&mut provider.world).commit(OriginEnvelope {
            origin,
            config_hash,
            payload: prepared,
        }) {
            Ok(receipt) => receipt.payload,
            Err(error) => {
                let error_id = canonical_error_id(error.error_id());
                provider
                    .prepared
                    .get_mut(&address)
                    .expect("token remains owned")
                    .terminal_error = Some(error_id);
                return status_for_error(error_id);
            }
        };
        let _receipt_bytes = receipt.receipt;
        let view = provider.world.publication_authority().capture();
        let entries = provider.prepared[&address].entries.clone();
        let mut storages = BTreeMap::<SectionKey, SectionStorage>::new();
        for entry in &entries {
            if let std::collections::btree_map::Entry::Vacant(slot) = storages.entry(entry.section)
            {
                let storage = provider
                    .sections
                    .get(&entry.section)
                    .and_then(|state| state.storage.clone())
                    .unwrap_or_else(|| SectionStorage::uniform(BlockId::from_raw(0)));
                slot.insert(storage);
            }
            storages
                .get_mut(&entry.section)
                .expect("storage inserted above")
                .write(
                    CellOffset::new(entry.offset).expect("validated offset"),
                    BlockId::from_raw(entry.block_id),
                );
        }
        for (section, storage) in storages {
            let revision = view
                .stamp()
                .section_revision_set
                .get(&section.id())
                .copied()
                .unwrap_or(view.stamp().world_revision);
            provider.sections.insert(
                section,
                SectionState {
                    presence: VoxelPresence::Ready,
                    revision,
                    storage: Some(storage),
                },
            );
        }
        provider.rebuild_block_world();
        let receipts = entries
            .iter()
            .map(|entry| entry.section)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .map(|section| VoxelWriteReceipt {
                section_key: section.to_abi(),
                up_to_section_revision: view
                    .stamp()
                    .section_revision_set
                    .get(&section.id())
                    .copied()
                    .unwrap_or(view.stamp().world_revision),
                world_revision: view.stamp().world_revision,
            })
            .collect::<Vec<_>>();
        provider
            .prepared
            .get_mut(&address)
            .expect("token remains owned")
            .receipts = receipts;
        write_receipts(
            &provider.prepared[&address].receipts,
            out_receipts,
            receipt_capacity,
            out_receipt_count,
        )
    })
}

fn write_receipts(
    receipts: &[VoxelWriteReceipt],
    out: *mut VoxelWriteReceipt,
    capacity: u32,
    count: *mut u32,
) -> i32 {
    if receipts.len() > capacity as usize {
        unsafe { count.write(receipts.len() as u32) };
        return LumioStatus::BufferTooSmall as i32;
    }
    unsafe {
        count.write(receipts.len() as u32);
        for (index, receipt) in receipts.iter().enumerate() {
            out.add(index).write(*receipt);
        }
    }
    LumioStatus::Success as i32
}

pub unsafe extern "C" fn block_write_abort(world: *mut c_void, token: *mut c_void) -> i32 {
    ffi(|| {
        if token.is_null() {
            return LumioStatus::InvalidArgument as i32;
        }
        let provider = match provider(world) {
            Ok(provider) => provider,
            Err(status) => return status,
        };
        let address = token as usize;
        let request = match provider.prepared.get(&address) {
            Some(token) => token.request.clone(),
            None => return LumioStatus::InvalidArgument as i32,
        };
        let state = provider.world.state_view();
        let origin = match OriginToken::try_new(
            state.world_context_id(),
            state.instance_generation(),
            format!("abort-{}", request.txn_id),
            0,
            BTreeMap::new(),
            "VoxelCommit",
        ) {
            Ok(origin) => origin,
            Err(error) => return status_for_error(error.error_id()),
        };
        let config_hash = provider.world.config_hash().to_string();
        match VoxelWorldPortAdapter::new(&mut provider.world).abort(OriginEnvelope {
            origin,
            config_hash,
            payload: request.clone(),
        }) {
            Ok(_) => {
                provider.prepared.remove(&address);
                provider
                    .transactions
                    .remove(&request.txn_id.parse::<u64>().unwrap_or_default());
                LumioStatus::Success as i32
            }
            Err(error) => status_for_error(error.error_id()),
        }
    })
}

pub unsafe extern "C" fn section_revision_query(
    world: *mut c_void,
    key: *const VoxelSectionKey,
    out: *mut VoxelSectionRevisionResult,
) -> i32 {
    ffi(|| {
        if key.is_null() || out.is_null() {
            return LumioStatus::InvalidArgument as i32;
        }
        let provider = match provider(world) {
            Ok(provider) => provider,
            Err(status) => return status,
        };
        let key = match SectionKey::from_abi(unsafe { *key }) {
            Ok(key) => key,
            Err(error) => return status_for_error(error),
        };
        if let Err(error) = provider.ensure_world_query(&[key]) {
            return status_for_error(error);
        }
        let id = key.id();
        let view = provider.world.publication_authority().capture();
        let slot = match view.directory().lookup(&id) {
            Ok(Some(slot)) => slot,
            Ok(None) => return status_for_error("unknown_section_key"),
            Err(error) => return status_for_error(error.error_id()),
        };
        let presence = match parse_presence(slot.presence()) {
            Ok(presence) => presence,
            Err(error) => return status_for_error(error),
        };
        let revision = view
            .stamp()
            .section_revision_set
            .get(&id)
            .copied()
            .unwrap_or(view.stamp().world_revision);
        unsafe {
            out.write(VoxelSectionRevisionResult {
                presence,
                _reserved: [0; 4],
                section_revision: revision,
            })
        };
        LumioStatus::Success as i32
    })
}

pub unsafe extern "C" fn residency_pin_declare(
    world: *mut c_void,
    keys: *const VoxelSectionKey,
    section_count: u32,
    budget: u32,
    out_pin: *mut *mut c_void,
) -> i32 {
    ffi(|| {
        if out_pin.is_null() || (section_count > 0 && keys.is_null()) {
            return LumioStatus::InvalidArgument as i32;
        }
        let provider = match provider(world) {
            Ok(provider) => provider,
            Err(status) => return status,
        };
        let input = if section_count == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(keys, section_count as usize) }
        };
        let ids = input
            .iter()
            .map(|key| SectionKey::from_abi(*key).map(|key| key.id()))
            .collect::<Result<Vec<_>, _>>();
        let ids = match ids {
            Ok(ids) => ids,
            Err(error) => return status_for_error(error),
        };
        let ready = {
            let view = provider.world.publication_authority().capture();
            ids.iter().all(|section_id| {
                view.directory()
                    .lookup(section_id)
                    .ok()
                    .flatten()
                    .is_some_and(|slot| slot.presence() == "Ready")
            })
        };
        let manager = match provider.world.region_pin_manager_mut() {
            Some(manager) => manager,
            None => return LumioStatus::InvalidArgument as i32,
        };
        let id = match manager
            .declare_pin_with_budget(ids, PinBudget::new(budget as usize, budget as usize))
        {
            Ok(id) => id,
            Err(error) => return status_for_error(error.error_id()),
        };
        if ready {
            let _ = manager.mark_ready(id);
        }
        let mut token = Box::new(PinToken { id });
        let address = (&mut *token) as *mut PinToken as usize;
        provider.pin_tokens.insert(address, token);
        unsafe { out_pin.write(address as *mut c_void) };
        LumioStatus::Success as i32
    })
}

pub unsafe extern "C" fn residency_pin_release(world: *mut c_void, pin: *mut c_void) -> i32 {
    ffi(|| {
        if pin.is_null() {
            return LumioStatus::InvalidArgument as i32;
        }
        let provider = match provider(world) {
            Ok(provider) => provider,
            Err(status) => return status,
        };
        let address = pin as usize;
        let id = match provider.pin_tokens.get(&address) {
            Some(token) => token.id,
            None => return LumioStatus::InvalidArgument as i32,
        };
        let manager = match provider.world.region_pin_manager_mut() {
            Some(manager) => manager,
            None => return LumioStatus::InvalidArgument as i32,
        };
        match manager.release_pin(id) {
            Ok(()) => {
                provider.pin_tokens.remove(&address);
                LumioStatus::Success as i32
            }
            Err(error) => status_for_error(error.error_id()),
        }
    })
}

pub unsafe extern "C" fn residency_pin_status(
    world: *mut c_void,
    pin: *mut c_void,
    out: *mut crate::abi_generated::VoxelPinStatus,
) -> i32 {
    ffi(|| {
        if pin.is_null() || out.is_null() {
            return LumioStatus::InvalidArgument as i32;
        }
        let provider = match provider(world) {
            Ok(provider) => provider,
            Err(status) => return status,
        };
        let id = match provider.pin_tokens.get(&(pin as usize)) {
            Some(token) => token.id,
            None => return LumioStatus::InvalidArgument as i32,
        };
        let manager = match provider.world.region_pin_manager() {
            Some(manager) => manager,
            None => return LumioStatus::InvalidArgument as i32,
        };
        let status = match manager.status(id) {
            Ok(status) => status,
            Err(error) => return status_for_error(error.error_id()),
        };
        let ready_count = manager
            .sections(id)
            .map(|sections| {
                sections
                    .iter()
                    .filter(|section_id| {
                        provider.sections.iter().any(|(key, state)| {
                            key.id() == **section_id && state.presence == VoxelPresence::Ready
                        })
                    })
                    .count()
            })
            .unwrap_or(0);
        unsafe {
            out.write(crate::abi_generated::VoxelPinStatus {
                ready: u8::from(status.is_ready()),
                _reserved: [0; 7],
                section_count: status.section_count() as u32,
                ready_section_count: ready_count as u32,
            })
        };
        LumioStatus::Success as i32
    })
}

/// 根表 `raycast` 槽：转发到 VoxelEngine 的 DDA 实现（`lumio_voxel_project::physics_query`）。
///
/// 三态用 `resolution` 显式表达：Unresolved 携带挡路的 SectionKey，既不塌缩成 Miss
/// （契约 `query.unresolved-is-not-air`），也不塌缩成 Hit（`query.unresolved-is-not-solid`），
/// 且它是正常结局、不是错误码。
///
/// # Safety
///
/// 调用方必须保证 `request` 指向可读的 `VoxelRaycastRequest`、`out` 指向可写的
/// `VoxelRaycastResult`（或传 null 走拒绝路径）。
pub unsafe extern "C" fn raycast(
    world: *mut c_void,
    request: *const VoxelRaycastRequest,
    out: *mut VoxelRaycastResult,
) -> i32 {
    ffi(|| {
        if request.is_null() || out.is_null() {
            return LumioStatus::InvalidArgument as i32;
        }
        let provider = match provider(world) {
            Ok(provider) => provider,
            Err(status) => return status,
        };
        // SAFETY: null 已在上面拒绝，调用方拥有该请求结构。
        let request = unsafe { *request };
        let mask = match material_mask(request.material_mask) {
            Ok(mask) => mask,
            Err(error) => return status_for_error(error),
        };
        let (physics_world, materials) = match provider.physics_inputs() {
            Ok(inputs) => inputs,
            Err(error) => return status_for_error(error),
        };
        let resolution = match provider.physics_query(&physics_world, materials).raycast(
            physics_point(request.origin),
            physics_point(request.direction),
            request.max_distance,
            mask,
        ) {
            Ok(resolution) => resolution,
            Err(error) => return status_for_error(error.error_id()),
        };

        // 每条分支先整体清零再逐字段落笔：填充字节与未走到的字段都是确定的 0，
        // 同一份世界同一组输入两次调用才可能逐字节相同（契约 `physicsQuery.determinism`）。
        match resolution {
            physics::QueryResolution::Hit(hit) => {
                let cell = match abi_cell(hit.cell()) {
                    Ok(cell) => cell,
                    Err(error) => return status_for_error(error),
                };
                let block_id = hit.block_id().map_or(0, BlockId::raw);
                let point = abi_point(hit.point());
                let normal = abi_point(hit.normal());
                let distance = hit.distance();
                // SAFETY: `out` 非空且指向调用方拥有的一个完整结果结构。
                unsafe {
                    std::ptr::write_bytes(out, 0, 1);
                    (*out).resolution = VoxelQueryResolution::Hit;
                    // world_coordinate 的 y 后面有 3 字节结构填充：整体赋值会把局部量里
                    // 未初始化的填充字节一起搬过来，逐字节确定性就没了。只逐标量落笔。
                    (*out).hit_cell.x = cell.x;
                    (*out).hit_cell.y = cell.y;
                    (*out).hit_cell.z = cell.z;
                    (*out).block_id = block_id;
                    (*out).hit_point = point;
                    (*out).hit_normal = normal;
                    (*out).travel_distance = distance;
                }
            }
            physics::QueryResolution::Miss => {
                // SAFETY: 同上。
                unsafe {
                    std::ptr::write_bytes(out, 0, 1);
                    (*out).resolution = VoxelQueryResolution::Miss;
                }
            }
            physics::QueryResolution::Unresolved { section } => {
                let key = abi_section_key(section);
                // SAFETY: 同上。
                unsafe {
                    std::ptr::write_bytes(out, 0, 1);
                    (*out).resolution = VoxelQueryResolution::Unresolved;
                    (*out).unresolved_section = key;
                }
            }
        }
        LumioStatus::Success as i32
    })
}

/// 根表 `sweep` 槽：把本帧位移的 AABB 扫掠转发到同一份 VoxelEngine 实现。
///
/// 形状按值内联（契约 `physicsQuery.shape.inlineByValue`）：`center` 即位姿，v1 无旋转无缩放。
/// Miss 表示整段位移都走得通，因此 `travel_fraction` 是 1.0；Unresolved 不给可行进比例，
/// 由调用方自己决定挂起还是保守处理。
///
/// # Safety
///
/// 调用方必须保证 `request` 指向可读的 `VoxelSweepRequest`、`out` 指向可写的
/// `VoxelSweepResult`（或传 null 走拒绝路径）。
pub unsafe extern "C" fn sweep(
    world: *mut c_void,
    request: *const VoxelSweepRequest,
    out: *mut VoxelSweepResult,
) -> i32 {
    ffi(|| {
        if request.is_null() || out.is_null() {
            return LumioStatus::InvalidArgument as i32;
        }
        let provider = match provider(world) {
            Ok(provider) => provider,
            Err(status) => return status,
        };
        // SAFETY: null 已在上面拒绝，调用方拥有该请求结构。
        let request = unsafe { *request };
        let mask = match material_mask(request.material_mask) {
            Ok(mask) => mask,
            Err(error) => return status_for_error(error),
        };
        let (physics_world, materials) = match provider.physics_inputs() {
            Ok(inputs) => inputs,
            Err(error) => return status_for_error(error),
        };
        let shape = physics::Aabb::new(
            physics_point(request.center),
            physics_point(request.half_extents),
        );
        let resolution = match provider.physics_query(&physics_world, materials).sweep(
            shape,
            physics_point(request.displacement),
            mask,
        ) {
            Ok(resolution) => resolution,
            Err(error) => return status_for_error(error.error_id()),
        };

        match resolution {
            physics::QueryResolution::Hit(hit) => {
                let cell = match abi_cell(hit.cell()) {
                    Ok(cell) => cell,
                    Err(error) => return status_for_error(error),
                };
                let block_id = hit.block_id().map_or(0, BlockId::raw);
                let point = abi_point(hit.point());
                let normal = abi_point(hit.normal());
                let fraction = hit.fraction();
                // SAFETY: `out` 非空且指向调用方拥有的一个完整结果结构。
                unsafe {
                    std::ptr::write_bytes(out, 0, 1);
                    (*out).resolution = VoxelQueryResolution::Hit;
                    (*out).collided = 1;
                    (*out).travel_fraction = fraction;
                    // 同 raycast：world_coordinate 的填充字节不得整体搬运。
                    (*out).hit_cell.x = cell.x;
                    (*out).hit_cell.y = cell.y;
                    (*out).hit_cell.z = cell.z;
                    (*out).block_id = block_id;
                    (*out).hit_point = point;
                    (*out).hit_normal = normal;
                }
            }
            physics::QueryResolution::Miss => {
                // SAFETY: 同上。
                unsafe {
                    std::ptr::write_bytes(out, 0, 1);
                    (*out).resolution = VoxelQueryResolution::Miss;
                    (*out).travel_fraction = 1.0;
                }
            }
            physics::QueryResolution::Unresolved { section } => {
                let key = abi_section_key(section);
                // SAFETY: 同上。
                unsafe {
                    std::ptr::write_bytes(out, 0, 1);
                    (*out).resolution = VoxelQueryResolution::Unresolved;
                    (*out).unresolved_section = key;
                }
            }
        }
        LumioStatus::Success as i32
    })
}

/// 根表 `overlap` 槽：命中条目写进调用方缓冲，Native 不返回任何需要释放的句柄。
///
/// 装不下时按契约 `query.overflow-must-be-reported` 显式回报 `truncated` 与 `actual_count`
/// （实际总条数），不静默截断。
///
/// # Safety
///
/// 调用方必须保证 `request` 指向可读的 `VoxelOverlapRequest`、`out_hits` 指向至少
/// `hit_capacity` 个可写的 `VoxelOverlapHit`、`out` 指向可写的 `VoxelOverlapResult`。
pub unsafe extern "C" fn overlap(
    world: *mut c_void,
    request: *const VoxelOverlapRequest,
    out_hits: *mut VoxelOverlapHit,
    hit_capacity: u32,
    out: *mut VoxelOverlapResult,
) -> i32 {
    ffi(|| {
        if request.is_null() || out.is_null() || (hit_capacity > 0 && out_hits.is_null()) {
            return LumioStatus::InvalidArgument as i32;
        }
        let provider = match provider(world) {
            Ok(provider) => provider,
            Err(status) => return status,
        };
        // SAFETY: null 已在上面拒绝，调用方拥有该请求结构。
        let request = unsafe { *request };
        let mask = match material_mask(request.material_mask) {
            Ok(mask) => mask,
            Err(error) => return status_for_error(error),
        };
        let (physics_world, materials) = match provider.physics_inputs() {
            Ok(inputs) => inputs,
            Err(error) => return status_for_error(error),
        };
        let shape = physics::Aabb::new(
            physics_point(request.center),
            physics_point(request.half_extents),
        );
        // 中转缓冲的长度取「调用方容量」与「契约单次查询格数上限」的较小值：调用方声明再大的
        // 容量也不会让 Native 按容量吃内存，而结果本身仍然只落在调用方缓冲里。
        let staging_len = (hit_capacity as usize)
            .min(crate::abi_generated::VOXEL_MAX_CELLS_PER_READ_REQUEST as usize);
        let mut staging = vec![physics::OverlapHit::default(); staging_len];
        let result = match provider.physics_query(&physics_world, materials).overlap(
            shape,
            mask,
            &mut staging,
        ) {
            Ok(result) => result,
            Err(error) => return status_for_error(error.error_id()),
        };
        let written = result.written_count().min(staging.len());
        // 先把全部条目转换完再落笔：任何一条转换失败都不得在调用方缓冲里留下半份结果。
        let mut hits = Vec::with_capacity(written);
        for hit in staging.iter().take(written) {
            match abi_cell(hit.cell()) {
                Ok(cell) => hits.push((cell, hit.block_id().map_or(0, BlockId::raw))),
                Err(error) => return status_for_error(error),
            }
        }
        let actual_count = match u32::try_from(result.actual_count()) {
            Ok(count) => count,
            Err(_) => return status_for_error("query_buffer_overflow"),
        };
        let unresolved = result
            .resolution()
            .unresolved_section()
            .map(abi_section_key);
        let resolution = match result.resolution() {
            physics::QueryResolution::Hit(()) => VoxelQueryResolution::Hit,
            physics::QueryResolution::Miss => VoxelQueryResolution::Miss,
            physics::QueryResolution::Unresolved { .. } => VoxelQueryResolution::Unresolved,
        };

        // SAFETY: `out_hits` 至少有 `hit_capacity` 个条目，而 `written <= staging_len
        // <= hit_capacity`；`out` 非空且指向一个完整结果结构。
        unsafe {
            for (index, (cell, block_id)) in hits.into_iter().enumerate() {
                let slot = out_hits.add(index);
                std::ptr::write_bytes(slot, 0, 1);
                // 同 raycast：world_coordinate 的填充字节不得整体搬运。
                (*slot).cell.x = cell.x;
                (*slot).cell.y = cell.y;
                (*slot).cell.z = cell.z;
                (*slot).block_id = block_id;
            }
            std::ptr::write_bytes(out, 0, 1);
            (*out).resolution = resolution;
            if let Some(key) = unresolved {
                (*out).unresolved_section = key;
            }
            (*out).actual_count = actual_count;
            (*out).truncated = u8::from(result.truncated());
        }
        LumioStatus::Success as i32
    })
}

fn hex32(bytes: &[u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(64);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 15) as usize] as char);
    }
    out
}

impl VoxelSectionKey {
    pub const fn new(x: i32, y: u8, z: i32) -> Self {
        Self {
            x,
            y,
            _reserved: [0; 3],
            z,
        }
    }
}
impl VoxelWorldCoordinate {
    pub const fn new(x: i32, y: u8, z: i32) -> Self {
        Self { x, y, z }
    }
}
impl VoxelWorldPoint {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}
impl VoxelBoxRequest {
    pub const fn new(min: VoxelWorldCoordinate, max: VoxelWorldCoordinate) -> Self {
        Self { min, max }
    }
}
impl VoxelBlockWriteEntry {
    pub const fn new(
        section_key: VoxelSectionKey,
        cell_offset: u16,
        block_id: u32,
        expected_section_revision: u64,
    ) -> Self {
        Self {
            section_key,
            cell_offset,
            _reserved: [0; 2],
            block_id,
            expected_section_revision,
        }
    }
}
impl Default for VoxelBlockReadCellResult {
    fn default() -> Self {
        Self {
            presence: VoxelPresence::Unavailable,
            has_block_id: 0,
            _reserved: [0; 3],
            block_id: 0,
            section_revision: 0,
        }
    }
}
impl Default for VoxelBlockReadResult {
    fn default() -> Self {
        Self {
            presence: VoxelPresence::Unavailable,
            has_block_id: 0,
            _reserved: [0; 3],
            block_id: 0,
            section_revision: 0,
        }
    }
}
impl Default for VoxelSectionRevisionResult {
    fn default() -> Self {
        Self {
            presence: VoxelPresence::Unavailable,
            _reserved: [0; 4],
            section_revision: 0,
        }
    }
}
impl Default for VoxelSectionSegment {
    fn default() -> Self {
        Self {
            section_key: VoxelSectionKey::new(0, 0, 0),
            presence: VoxelPresence::Unavailable,
            section_revision: 0,
            first_result: 0,
            result_count: 0,
        }
    }
}
impl Default for VoxelRaycastResult {
    fn default() -> Self {
        Self {
            resolution: VoxelQueryResolution::Miss,
            unresolved_section: VoxelSectionKey::new(0, 0, 0),
            hit_cell: VoxelWorldCoordinate::new(0, 0, 0),
            block_id: 0,
            hit_point: VoxelWorldPoint::new(0.0, 0.0, 0.0),
            hit_normal: VoxelWorldPoint::new(0.0, 0.0, 0.0),
            travel_distance: 0.0,
        }
    }
}
impl Default for VoxelSweepResult {
    fn default() -> Self {
        Self {
            resolution: VoxelQueryResolution::Miss,
            collided: 0,
            _reserved: [0; 3],
            travel_fraction: 0.0,
            unresolved_section: VoxelSectionKey::new(0, 0, 0),
            hit_cell: VoxelWorldCoordinate::new(0, 0, 0),
            block_id: 0,
            hit_point: VoxelWorldPoint::new(0.0, 0.0, 0.0),
            hit_normal: VoxelWorldPoint::new(0.0, 0.0, 0.0),
        }
    }
}
impl Default for VoxelOverlapHit {
    fn default() -> Self {
        Self {
            cell: VoxelWorldCoordinate::new(0, 0, 0),
            block_id: 0,
        }
    }
}
impl Default for VoxelOverlapResult {
    fn default() -> Self {
        Self {
            resolution: VoxelQueryResolution::Miss,
            unresolved_section: VoxelSectionKey::new(0, 0, 0),
            actual_count: 0,
            truncated: 0,
            _reserved: [0; 3],
        }
    }
}
impl Default for VoxelWriteReceipt {
    fn default() -> Self {
        Self {
            section_key: VoxelSectionKey::new(0, 0, 0),
            up_to_section_revision: 0,
            world_revision: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unregistered_block_type_maps_to_stable_contract_status() {
        assert_eq!(
            status_for_error("unregistered_block_type"),
            crate::abi_generated::VOXEL_ERROR_UNREGISTERED_BLOCK_TYPE
        );
        assert_eq!(
            crate::abi_generated::VOXEL_ERROR_UNREGISTERED_BLOCK_TYPE,
            1051
        );
    }

    /// 公共契约定义(唯一真值),用于对手写映射做穷尽性校验。
    const ABI_DEFINITION: &str = include_str!("../../../../abi/native-abi.json");
    /// 生成物,用于确认契约条目数与生成的状态常量条目数一致。
    const ABI_GENERATED: &str = include_str!("abi_generated.rs");
    /// 手写镜像自身的源码。反方向穷尽性校验要枚举 `status_for_error` 的 match 臂,
    /// Rust 没有反射,只能读自己的源码。
    const SELF_SOURCE: &str = include_str!("voxel.rs");

    /// 从 `text[start]` 处的开引号出发,返回闭合引号**之后**的下标(识别 `\"` 转义)。
    /// JSON 与 Rust 源码的字符串字面量转义规则在这一点上一致。
    fn end_of_quoted_literal(text: &str, start: usize) -> usize {
        let bytes = text.as_bytes();
        assert_eq!(bytes[start], b'"', "end_of_quoted_literal 必须从开引号出发");
        let mut index = start + 1;
        while index < bytes.len() {
            match bytes[index] {
                b'\\' => index += 2,
                b'"' => return index + 1,
                _ => index += 1,
            }
        }
        panic!("未闭合的字符串字面量");
    }

    /// 契约根对象里 `voxel` 段的切片(含首尾大括号)。
    ///
    /// 所有对手写镜像做穷尽性校验的解析**只允许在这一段里检索**。对整份 `native-abi.json`
    /// 做全局 `find()` 会在将来任一子系统(runtime / timer / …)在 `voxel` **之前**引入同名键
    /// (`errorStatusBase` / `errorCodes` / `material_mask`)时**静默读到别人的值**,穷尽性断言
    /// 随之失效且不报错。托管侧 `VoxelFacadeTests` 用 `RootElement.GetProperty("voxel")` 定位,
    /// 这里做同口径的限定;不引 JSON 依赖,只做一次带字符串与转义感知的括号配对扫描。
    fn voxel_section() -> &'static str {
        let bytes = ABI_DEFINITION.as_bytes();
        let needle = "\"voxel\"";
        let mut index = 0usize;
        // depth 0 = 文档外;根对象的直接子键在 depth 1 上出现。
        let mut depth = 0usize;
        let mut found: Option<&'static str> = None;
        while index < bytes.len() {
            match bytes[index] {
                b'"' => {
                    let literal_end = end_of_quoted_literal(ABI_DEFINITION, index);
                    if depth == 1 && &ABI_DEFINITION[index..literal_end] == needle {
                        let rest = ABI_DEFINITION[literal_end..].trim_start();
                        let rest = rest
                            .strip_prefix(':')
                            .expect("native-abi.json voxel 键后必须跟冒号")
                            .trim_start();
                        let value_start = ABI_DEFINITION.len() - rest.len();
                        assert!(
                            ABI_DEFINITION.as_bytes()[value_start] == b'{',
                            "native-abi.json 的 voxel 必须是一个对象"
                        );
                        let value_end = end_of_object(ABI_DEFINITION, value_start);
                        assert!(
                            found.is_none(),
                            "native-abi.json 根对象里出现了多个 voxel 键"
                        );
                        found = Some(&ABI_DEFINITION[value_start..value_end]);
                    }
                    index = literal_end;
                }
                b'{' | b'[' => {
                    depth += 1;
                    index += 1;
                }
                b'}' | b']' => {
                    depth -= 1;
                    index += 1;
                }
                _ => index += 1,
            }
        }
        found.expect("native-abi.json 根对象必须声明 voxel 段")
    }

    /// 从 `text[start]` 处的 `{` 出发,返回配对 `}` **之后**的下标;跳过字符串字面量。
    fn end_of_object(text: &str, start: usize) -> usize {
        let bytes = text.as_bytes();
        assert_eq!(bytes[start], b'{', "end_of_object 必须从左大括号出发");
        let mut depth = 0usize;
        let mut index = start;
        while index < bytes.len() {
            match bytes[index] {
                b'"' => {
                    index = end_of_quoted_literal(text, index);
                    continue;
                }
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        return index + 1;
                    }
                }
                _ => {}
            }
            index += 1;
        }
        panic!("未闭合的对象");
    }

    /// 在 `body` 里定位**恰好出现一次**的键 `key`,返回其值的起始切片。
    /// 段内再出现同名键(例如 voxel 段自己长出嵌套的 `errorCodes`)时直接变红,
    /// 而不是取第一个了事。
    fn unique_key_value(body: &'static str, key: &str) -> &'static str {
        let needle = format!("\"{key}\"");
        let occurrences = body.matches(needle.as_str()).count();
        assert_eq!(
            occurrences, 1,
            "voxel 段里的键 `{key}` 必须恰好出现一次,实际 {occurrences} 次;\
             定位假设失效时本断言必须变红,不得静默取第一个"
        );
        let start = body.find(needle.as_str()).expect("已断言存在") + needle.len();
        body[start..]
            .trim_start()
            .strip_prefix(':')
            .unwrap_or_else(|| panic!("voxel.{key} 后必须跟冒号"))
            .trim_start()
    }

    /// 从 `native-abi.json` 的 **voxel 段**里取出 `errorStatusBase` 与 `errorCodes`。
    fn contract_error_codes() -> (i32, Vec<&'static str>) {
        let voxel = voxel_section();

        let base_rest = unique_key_value(voxel, "errorStatusBase");
        let base_end = base_rest
            .find(',')
            .expect("voxel.errorStatusBase must be followed by more fields");
        let base: i32 = base_rest[..base_end]
            .trim()
            .parse()
            .expect("voxel.errorStatusBase must be an integer");

        let codes_rest = unique_key_value(voxel, "errorCodes");
        assert!(
            codes_rest.starts_with('['),
            "voxel.errorCodes must be an array"
        );
        let close = codes_rest
            .find(']')
            .expect("voxel.errorCodes must be a closed array");
        let body = &codes_rest[1..close];

        let mut codes = Vec::new();
        let mut cursor = body;
        while let Some(open_quote) = cursor.find('"') {
            let after = &cursor[open_quote + 1..];
            let close_quote = after
                .find('"')
                .expect("voxel.errorCodes entries must be quoted strings");
            codes.push(&after[..close_quote]);
            cursor = &after[close_quote + 1..];
        }

        assert!(
            !codes.is_empty(),
            "voxel.errorCodes must not be empty in native-abi.json"
        );
        (base, codes)
    }

    /// 从 `native-abi.json` 的 `voxel.enums.material_mask` 里取一个整数字段。
    fn contract_u32_after(body: &str, key: &str) -> u32 {
        let start = body
            .find(key)
            .unwrap_or_else(|| panic!("native-abi.json material_mask must declare {key}"))
            + key.len();
        let rest = &body[start..];
        let end = rest
            .find([',', '}'])
            .expect("a material_mask integer field must be terminated");
        rest[..end]
            .trim()
            .parse()
            .unwrap_or_else(|_| panic!("material_mask field {key} must be an integer"))
    }

    /// 掩码位分配的唯一真值是契约的 `voxel.enums.material_mask`。手写常量与它对不上时
    /// (例如契约追加了一个材质类而这里没跟)本用例必须变红,而不是让一条错误的掩码悄悄放行。
    #[test]
    fn material_mask_bits_match_the_contract_declaration() {
        let voxel = voxel_section();
        let section_start = voxel
            .find("\"material_mask\"")
            .expect("native-abi.json must declare voxel.enums.material_mask");
        let section = &voxel[section_start..];
        let values_start = section
            .find("\"values\"")
            .expect("material_mask must declare values");
        let values = &section[values_start..];

        let solid = contract_u32_after(values, "\"Solid\":");
        let liquid = contract_u32_after(values, "\"Liquid\":");
        let assigned_bit_count = contract_u32_after(section, "\"assignedBitCount\":");

        assert_eq!(MATERIAL_MASK_SOLID, solid, "Solid 掩码位与契约不一致");
        assert_eq!(MATERIAL_MASK_LIQUID, liquid, "Liquid 掩码位与契约不一致");
        assert_eq!(
            MATERIAL_MASK_ASSIGNED_BITS,
            solid | liquid,
            "已分配位集合必须正好是契约声明的那些类"
        );
        assert_eq!(
            MATERIAL_MASK_ASSIGNED_BITS.count_ones(),
            assigned_bit_count,
            "已分配位数必须等于契约的 assignedBitCount"
        );

        // 第 2 位及以上未分配:置位即 unknown_material_class,不得当作「未来的类」忽略。
        let first_reserved = 1_u32 << assigned_bit_count;
        assert_eq!(
            material_mask(first_reserved),
            Err("unknown_material_class"),
            "第一位保留位置位必须被拒"
        );
        assert_eq!(
            material_mask(u32::MAX),
            Err("unknown_material_class"),
            "全 1 掩码必须被拒"
        );
        // 掩码 0 是合法输入(不匹配任何类),不是错误码。
        assert!(material_mask(0).is_ok(), "掩码 0 不得当成错误");
        assert!(material_mask(MATERIAL_MASK_ASSIGNED_BITS).is_ok());
    }

    /// 穷尽性断言:手写的 `status_for_error` 必须覆盖公共契约里的**每一条**错误码,
    /// 且数值与生成物一致。契约追加错误码而这里漏改时,本用例必须变红。
    #[test]
    fn status_for_error_covers_every_contract_error_code() {
        let (base, codes) = contract_error_codes();
        let generated = ABI_GENERATED
            .lines()
            .filter(|line| line.starts_with("pub const VOXEL_ERROR_"))
            .count();
        assert_eq!(
            codes.len(),
            generated,
            "native-abi.json voxel.errorCodes ({}) and abi_generated.rs VOXEL_ERROR_* ({}) diverged; \
             regenerate with `node eng/generate-abi.mjs`",
            codes.len(),
            generated
        );

        let fallback = LumioStatus::InvalidArgument as i32;
        for (index, code) in codes.iter().enumerate() {
            let expected = base + index as i32;
            let actual = status_for_error(code);
            assert_ne!(
                actual, fallback,
                "contract error code `{code}` collapses into the generic InvalidArgument fallback; \
                 add a named arm to status_for_error"
            );
            assert_eq!(
                actual, expected,
                "contract error code `{code}` must map to the stable status {expected}"
            );
        }

        assert_eq!(
            status_for_error("not_a_contract_error_code"),
            fallback,
            "non-contract errors must still land on the generic fallback"
        );
    }

    /// 取 `fn status_for_error` 的函数体(不含外层大括号)。
    /// 按行首签名定位——本文件里这条签名还会作为字符串字面量出现在下面的解析器里,
    /// 而那一处是缩进的,所以「行首 + 恰好一次」既能锁定真正的定义,又能在文件结构变化时变红。
    fn status_for_error_body() -> &'static str {
        let signature = "\nfn status_for_error(error: &str) -> i32 {";
        let occurrences = SELF_SOURCE.matches(signature).count();
        assert_eq!(
            occurrences, 1,
            "voxel.rs 必须在行首恰好定义一次 status_for_error,实际 {occurrences} 次"
        );
        let start = SELF_SOURCE.find(signature).expect("已断言存在") + signature.len();
        let bytes = SELF_SOURCE.as_bytes();
        let mut depth = 1usize;
        let mut index = start;
        while index < bytes.len() {
            match bytes[index] {
                b'"' => {
                    index = end_of_quoted_literal(SELF_SOURCE, index);
                    continue;
                }
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        return &SELF_SOURCE[start..index];
                    }
                }
                _ => {}
            }
            index += 1;
        }
        panic!("status_for_error 的函数体必须闭合");
    }

    /// `status_for_error` 里所有具名臂的模式字符串。函数体里的字符串字面量只有 match 臂的
    /// 模式(臂的值都是 `crate::abi_generated::VOXEL_ERROR_*` 路径),所以「字符串后面跟
    /// `=>` 或 `|`」就是一条具名臂。
    fn status_for_error_named_arms() -> Vec<&'static str> {
        let body = status_for_error_body();
        let bytes = body.as_bytes();
        let mut arms = Vec::new();
        let mut index = 0usize;
        while index < bytes.len() {
            if bytes[index] == b'"' {
                let end = end_of_quoted_literal(body, index);
                let tail = body[end..].trim_start();
                if tail.starts_with("=>") || tail.starts_with('|') {
                    arms.push(&body[index + 1..end - 1]);
                }
                index = end;
            } else {
                index += 1;
            }
        }
        arms
    }

    /// 反方向穷尽性断言:`status_for_error` **不得**含公共契约之外的具名臂。
    ///
    /// 正向断言(`status_for_error_covers_every_contract_error_code`)只沿「契约 → 手写映射」
    /// 遍历,查不出这一类漂移:契约删掉一条错误码后残留的臂、或有人为了给某个内部错误
    /// 「凑个码」私自加的臂,都能在正向断言全绿的情况下留下。托管侧对应的检查是
    /// `VoxelFacadeTests.EveryContractStatusHasAStableManagedErrorCode` 末尾的成员数相等断言,
    /// 这里补齐 Rust 侧的同一口径。
    #[test]
    fn status_for_error_has_no_named_arm_outside_the_contract() {
        let (_, codes) = contract_error_codes();
        let contract: std::collections::BTreeSet<&str> = codes.iter().copied().collect();
        assert_eq!(
            contract.len(),
            codes.len(),
            "native-abi.json voxel.errorCodes 不得有重复条目"
        );

        let mut seen = std::collections::BTreeSet::new();
        for arm in status_for_error_named_arms() {
            assert!(
                seen.insert(arm),
                "status_for_error 出现重复的具名臂 `{arm}`"
            );
            assert!(
                contract.contains(arm),
                "status_for_error 的具名臂 `{arm}` 不在 native-abi.json voxel.errorCodes 里;\
                 手写镜像不得自造契约之外的错误码"
            );
        }

        assert_eq!(
            seen.len(),
            contract.len(),
            "status_for_error 的具名臂数({})必须与契约错误码数({})相等",
            seen.len(),
            contract.len()
        );
    }
}
