use std::ffi::c_void;

use lumio_engine_native::{
    lumio_engine_get_api_v1, BlockCatalogRowInput, LumioEngineRootApiV1, LumioStatus,
    NativeVoxelProvider, OfficialCatalog, StateLayout, VoxelBlockReadCellResult,
    VoxelBlockReadResult, VoxelBoxRequest, VoxelOverlapHit, VoxelOverlapRequest,
    VoxelOverlapResult, VoxelPresence, VoxelQueryResolution, VoxelRaycastRequest,
    VoxelRaycastResult, VoxelSectionKey, VoxelSectionRevisionResult, VoxelSweepRequest,
    VoxelSweepResult, VoxelWorldCoordinate, VoxelWorldPoint, VoxelWriteReceipt,
};

#[test]
fn root_entry_returns_a_callable_table_with_build_identity() {
    let mut table = std::ptr::null();
    let status = unsafe { lumio_engine_get_api_v1(1, &mut table) };

    assert_eq!(status, LumioStatus::Success as i32);
    assert!(!table.is_null());

    let table = unsafe { &*table };
    assert_eq!(table.abi_version, 1);
    assert!(table.struct_size as usize >= std::mem::size_of::<LumioEngineRootApiV1>());
    assert_ne!(table.abi_hash, [0; 32]);
    assert_ne!(table.build_id, [0; 16]);
    assert!(table.ping.is_some());

    let ping = table.ping.unwrap();
    let mut marker = 0_u32;
    let marker_ptr = (&mut marker as *mut u32).cast::<c_void>();
    assert_eq!(unsafe { ping(marker_ptr) }, LumioStatus::Success as i32);
    assert_eq!(marker, 1);
}

#[test]
fn root_entry_rejects_an_unknown_requested_version() {
    let mut table = std::ptr::null();
    let status = unsafe { lumio_engine_get_api_v1(99, &mut table) };

    assert_ne!(status, LumioStatus::Success as i32);
    assert!(table.is_null());
}

#[test]
fn root_table_wires_the_clr_host_slots_to_real_implementations() {
    let mut table = std::ptr::null();
    let status = unsafe { lumio_engine_get_api_v1(1, &mut table) };
    assert_eq!(status, LumioStatus::Success as i32);

    let table = unsafe { &*table };
    assert!(
        table.create_clr_host.is_some(),
        "create_clr_host 槽位必须接线"
    );
    assert!(table.clr_host_call.is_some(), "clr_host_call 槽位必须接线");
    assert!(
        table.destroy_clr_host.is_some(),
        "destroy_clr_host 槽位必须接线"
    );

    // struct_size 随根表扩展自动更新（size_of），不得落后于实际布局。
    assert_eq!(
        table.struct_size as usize,
        std::mem::size_of::<LumioEngineRootApiV1>()
    );

    // 槽位必须转发到真实实现（clr-host 的参数校验路径），而不是空壳。
    let create = table.create_clr_host.unwrap();
    assert_eq!(
        unsafe {
            create(
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null_mut(),
            )
        },
        LumioStatus::InvalidArgument as i32
    );
    let call = table.clr_host_call.unwrap();
    let mut written = 0_u32;
    assert_eq!(
        unsafe {
            call(
                std::ptr::null_mut(),
                std::ptr::null(),
                0,
                std::ptr::null_mut(),
                0,
                &mut written,
            )
        },
        LumioStatus::InvalidArgument as i32
    );
    let destroy = table.destroy_clr_host.unwrap();
    assert_eq!(
        unsafe { destroy(std::ptr::null_mut()) },
        LumioStatus::InvalidArgument as i32
    );
}

#[test]
fn status_codes_match_the_abi_definition() {
    assert_eq!(LumioStatus::Success as i32, 0);
    assert_eq!(LumioStatus::InvalidArgument as i32, 1);
    assert_eq!(LumioStatus::UnsupportedVersion as i32, 2);
    assert_eq!(LumioStatus::ClrInitFailed as i32, 3);
    assert_eq!(LumioStatus::ClrEntryFailed as i32, 4);
    assert_eq!(LumioStatus::BufferTooSmall as i32, 5);
    assert_eq!(LumioStatus::TimerStaleHandle as i32, 6);
    assert_eq!(LumioStatus::TimerScopeInvalid as i32, 7);
    assert_eq!(LumioStatus::TimerScopeGenerationMismatch as i32, 8);
    assert_eq!(LumioStatus::TimerInvalidDueTick as i32, 9);
    assert_eq!(LumioStatus::TimerInvalidInterval as i32, 10);
    assert_eq!(LumioStatus::TimerScheduleBudgetExceeded as i32, 11);
    assert_eq!(LumioStatus::TimerSlotClosed as i32, 12);
    assert_eq!(LumioStatus::TimerSlotUnbound as i32, 13);
    assert_eq!(LumioStatus::TimerSlotDispatchMismatch as i32, 14);
    assert_eq!(LumioStatus::TimerSlotQueueFull as i32, 15);
    assert_eq!(LumioStatus::TimerLateCompletion as i32, 16);
    assert_eq!(LumioStatus::TimerManagerShutdown as i32, 17);
}

#[test]
fn live_root_table_covers_c4_timer_slots() {
    let mut table = std::ptr::null();
    let status = unsafe { lumio_engine_get_api_v1(1, &mut table) };
    assert_eq!(status, LumioStatus::Success as i32);
    assert!(!table.is_null());

    let table = unsafe { &*table };
    let size = std::mem::size_of::<LumioEngineRootApiV1>();
    assert_eq!(
        table.struct_size as usize, size,
        "struct_size must be size_of the live root table"
    );
    assert!(
        size >= 200,
        "CLR-only root is 88 bytes; C-4 timer_* slots require size_of >= 200, got {size}"
    );
    assert_eq!(size, 304, "A-1 voxel slots extend the C-4 root table");
    assert_ne!(
        size, 88,
        "loaded struct_size 88 is the CLR-only layout that blocked R-00374/R-00376"
    );

    assert!(
        table.timer_create_manager.is_some(),
        "timer_create_manager must be a live function pointer"
    );
    assert!(
        table.timer_destroy_manager.is_some(),
        "timer_destroy_manager must be a live function pointer"
    );
    assert!(
        table.timer_register_dispatch.is_some(),
        "timer_register_dispatch must be a live function pointer"
    );
    assert!(
        table.timer_register_scope.is_some(),
        "timer_register_scope must be a live function pointer"
    );
    assert!(
        table.timer_teardown_scope.is_some(),
        "timer_teardown_scope must be a live function pointer"
    );
    assert!(
        table.timer_create_slot.is_some(),
        "timer_create_slot must be a live function pointer"
    );
    assert!(
        table.timer_bind_slot.is_some(),
        "timer_bind_slot must be wired"
    );
    assert!(
        table.timer_close_slot.is_some(),
        "timer_close_slot must be a live function pointer"
    );
    assert!(
        table.timer_schedule_one_shot.is_some(),
        "timer_schedule_one_shot must be a live function pointer"
    );
    assert!(
        table.timer_schedule_repeating.is_some(),
        "timer_schedule_repeating must be a live function pointer"
    );
    assert!(table.timer_cancel.is_some(), "timer_cancel must be wired");
    assert!(table.timer_advance.is_some(), "timer_advance must be wired");
    assert!(table.timer_pump.is_some(), "timer_pump must be wired");
    assert!(table.timer_drain.is_some(), "timer_drain must be wired");

    let create = table.timer_create_manager.unwrap();
    assert_eq!(
        unsafe { create(0, std::ptr::null_mut()) },
        LumioStatus::InvalidArgument as i32
    );

    let destroy = table.timer_destroy_manager.unwrap();
    assert_eq!(
        unsafe { destroy(std::ptr::null_mut()) },
        LumioStatus::InvalidArgument as i32
    );

    let mut manager = std::ptr::null_mut::<c_void>();
    assert_eq!(
        unsafe { create(0, &mut manager) },
        LumioStatus::Success as i32
    );
    assert!(!manager.is_null());
    assert_eq!(unsafe { destroy(manager) }, LumioStatus::Success as i32);
    assert_eq!(
        unsafe { destroy(manager) },
        LumioStatus::TimerManagerShutdown as i32,
        "shutdown-tombstone must return TimerManagerShutdown (status 17)"
    );
}

/// 反例探针（R-00496）：`timer_register_dispatch` 转发到内核未 gate 的
/// `try_register_dispatch`，其超预算失败必须映射为既有状态码
/// `TimerScheduleBudgetExceeded`(11)，不得静默成功。预算值是内核侧的实现细节，
/// 这里只钉「会拒绝」与「拒绝码」，不钉具体阈值。
#[test]
fn timer_register_dispatch_over_budget_returns_schedule_budget_exceeded() {
    let mut table = std::ptr::null();
    assert_eq!(
        unsafe { lumio_engine_get_api_v1(1, &mut table) },
        LumioStatus::Success as i32
    );
    let table = unsafe { &*table };
    let create = table.timer_create_manager.unwrap();
    let register = table.timer_register_dispatch.unwrap();

    let mut manager = std::ptr::null_mut::<c_void>();
    assert_eq!(
        unsafe { create(0, &mut manager) },
        LumioStatus::Success as i32
    );

    // 唯一 id 逐个注册，直到内核拒绝；上限远高于内核默认预算，兜底防死循环。
    const PROBE_CEILING: u32 = 1 << 20;
    let mut rejected_at = None;
    for dispatch_id in 1..=PROBE_CEILING {
        let status = unsafe { register(manager, dispatch_id) };
        if status != LumioStatus::Success as i32 {
            rejected_at = Some((dispatch_id, status));
            break;
        }
    }

    let (dispatch_id, status) = rejected_at.expect("注册必须在预算耗尽时被拒绝，而不是无限成功");
    assert_eq!(
        status,
        LumioStatus::TimerScheduleBudgetExceeded as i32,
        "第 {dispatch_id} 个 dispatch 超预算必须返回 TimerScheduleBudgetExceeded(11)，实际 {status}"
    );

    let destroy = table.timer_destroy_manager.unwrap();
    assert_eq!(unsafe { destroy(manager) }, LumioStatus::Success as i32);
}

#[test]
fn live_root_table_wires_a1_voxel_slots_including_the_three_physics_slots() {
    let mut table = std::ptr::null();
    assert_eq!(
        unsafe { lumio_engine_get_api_v1(1, &mut table) },
        LumioStatus::Success as i32
    );
    let table = unsafe { &*table };
    assert_eq!(table.struct_size as usize, 304);
    assert!(table.block_read_cell.is_some());
    assert!(table.block_read_box.is_some());
    assert!(table.block_read_column.is_some());
    assert!(table.block_write_prepare.is_some());
    assert!(table.block_write_commit.is_some());
    assert!(table.block_write_abort.is_some());
    assert!(table.section_revision_query.is_some());
    assert!(table.residency_pin_declare.is_some());
    assert!(table.residency_pin_release.is_some());
    assert!(table.residency_pin_status.is_some());
    // R-00477：三个物理槽必须是活函数指针，不再是空槽。
    assert!(table.raycast.is_some(), "raycast 槽位必须接线");
    assert!(table.sweep.is_some(), "sweep 槽位必须接线");
    assert!(table.overlap.is_some(), "overlap 槽位必须接线");

    // 槽位必须转发到真实实现（参数校验路径），而不是空壳。
    assert_eq!(
        unsafe {
            (table.raycast.unwrap())(std::ptr::null_mut(), std::ptr::null(), std::ptr::null_mut())
        },
        LumioStatus::InvalidArgument as i32
    );
    assert_eq!(
        unsafe {
            (table.sweep.unwrap())(std::ptr::null_mut(), std::ptr::null(), std::ptr::null_mut())
        },
        LumioStatus::InvalidArgument as i32
    );
    assert_eq!(
        unsafe {
            (table.overlap.unwrap())(
                std::ptr::null_mut(),
                std::ptr::null(),
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
            )
        },
        LumioStatus::InvalidArgument as i32
    );
}

#[test]
fn voxel_root_round_trip_preserves_unsigned_block_id_and_revision() {
    let mut provider = NativeVoxelProvider::new();
    provider.seed_ready_section(VoxelSectionKey::new(0, 0, 0), 12, 0x8000_0102);
    let mut table = std::ptr::null();
    assert_eq!(
        unsafe { lumio_engine_get_api_v1(1, &mut table) },
        LumioStatus::Success as i32
    );
    let table = unsafe { &*table };
    let mut result = VoxelBlockReadCellResult::default();
    let coordinate = VoxelWorldCoordinate::new(0, 1, 0);
    assert_eq!(
        unsafe {
            (table.block_read_cell.unwrap())(provider.as_opaque_ptr(), &coordinate, &mut result)
        },
        LumioStatus::Success as i32
    );
    assert_eq!(result.presence, VoxelPresence::Ready);
    assert_eq!(result.has_block_id, 1);
    assert_eq!(result.block_id, 0x8000_0102);
    assert_eq!(result.section_revision, 12);
}

#[test]
fn voxel_pending_batch_read_remains_explicit_and_prepare_commit_is_atomic() {
    let mut provider = NativeVoxelProvider::new();
    provider.seed_ready_section(VoxelSectionKey::new(0, 0, 0), 12, 0x8000_0102);
    provider.seed_pending_section(VoxelSectionKey::new(1, 0, 0), 99);
    let mut table = std::ptr::null();
    assert_eq!(
        unsafe { lumio_engine_get_api_v1(1, &mut table) },
        LumioStatus::Success as i32
    );
    let table = unsafe { &*table };

    let request = VoxelBoxRequest::new(
        VoxelWorldCoordinate::new(0, 1, 0),
        VoxelWorldCoordinate::new(16, 1, 0),
    );
    let mut cells = [VoxelBlockReadResult::default(); 17];
    let mut cell_count = 0;
    let mut segments = [Default::default(); 2];
    let mut segment_count = 0;
    let mut truncated = 0;
    assert_eq!(
        unsafe {
            (table.block_read_box.unwrap())(
                provider.as_opaque_ptr(),
                (&request as *const VoxelBoxRequest).cast(),
                cells.as_mut_ptr(),
                cells.len() as u32,
                &mut cell_count,
                segments.as_mut_ptr(),
                segments.len() as u32,
                &mut segment_count,
                &mut truncated,
            )
        },
        LumioStatus::Success as i32
    );
    assert_eq!(cell_count, 17);
    assert_eq!(cells[0].presence, VoxelPresence::Ready);
    assert_eq!(cells[16].presence, VoxelPresence::Pending);
    assert_eq!(cells[16].has_block_id, 0);
    assert_eq!(segment_count, 2);
    assert_eq!(segments[0].section_key.x, 0);
    assert_eq!(segments[0].section_key.y, 0);
    assert_eq!(segments[0].section_key.z, 0);
    assert_eq!(segments[0].presence, VoxelPresence::Ready);
    assert_eq!(segments[0].first_result, 0);
    assert_eq!(segments[0].result_count, 16);
    assert_eq!(segments[1].section_key.x, 1);
    assert_eq!(segments[1].section_key.y, 0);
    assert_eq!(segments[1].section_key.z, 0);
    assert_eq!(segments[1].presence, VoxelPresence::Pending);
    assert_eq!(segments[1].first_result, 16);
    assert_eq!(segments[1].result_count, 1);
    assert_eq!(truncated, 0);

    let entry = lumio_engine_native::VoxelBlockWriteEntry::new(
        VoxelSectionKey::new(0, 0, 0),
        256,
        0x8000_0103,
        12,
    );
    let mut token = std::ptr::null_mut();
    assert_eq!(
        unsafe {
            (table.block_write_prepare.unwrap())(provider.as_opaque_ptr(), 7, &entry, 1, &mut token)
        },
        LumioStatus::Success as i32
    );
    let mut receipts = [VoxelWriteReceipt::default(); 1];
    let mut receipt_count = 0;
    assert_eq!(
        unsafe {
            (table.block_write_commit.unwrap())(
                provider.as_opaque_ptr(),
                token,
                receipts.as_mut_ptr(),
                1,
                &mut receipt_count,
            )
        },
        LumioStatus::Success as i32
    );
    assert_eq!(receipt_count, 1);
    assert_eq!(receipts[0].up_to_section_revision, 13);

    let mut after = VoxelBlockReadCellResult::default();
    assert_eq!(
        unsafe {
            (table.block_read_cell.unwrap())(
                provider.as_opaque_ptr(),
                &VoxelWorldCoordinate::new(0, 1, 0),
                &mut after,
            )
        },
        LumioStatus::Success as i32
    );
    assert_eq!(after.presence, VoxelPresence::Ready);
    assert_eq!(after.block_id, 0x8000_0103);
    assert_eq!(after.section_revision, 13);

    let mut untouched = VoxelBlockReadCellResult::default();
    assert_eq!(
        unsafe {
            (table.block_read_cell.unwrap())(
                provider.as_opaque_ptr(),
                &VoxelWorldCoordinate::new(1, 1, 0),
                &mut untouched,
            )
        },
        LumioStatus::Success as i32
    );
    assert_eq!(untouched.presence, VoxelPresence::Ready);
    assert_eq!(untouched.block_id, 0x8000_0102);
    assert_eq!(untouched.section_revision, 13);

    let mut revision = VoxelSectionRevisionResult::default();
    assert_eq!(
        unsafe {
            (table.section_revision_query.unwrap())(
                provider.as_opaque_ptr(),
                &VoxelSectionKey::new(0, 0, 0),
                &mut revision,
            )
        },
        LumioStatus::Success as i32
    );
    assert_eq!(revision.presence, VoxelPresence::Ready);
    assert_eq!(revision.section_revision, 13);
}

#[test]
fn voxel_revision_and_null_arguments_return_stable_statuses_without_panicking() {
    let mut provider = NativeVoxelProvider::new();
    provider.seed_ready_section(VoxelSectionKey::new(0, 0, 0), 12, 1);
    let mut table = std::ptr::null();
    assert_eq!(
        unsafe { lumio_engine_get_api_v1(1, &mut table) },
        LumioStatus::Success as i32
    );
    let table = unsafe { &*table };
    let key = VoxelSectionKey::new(0, 0, 0);
    let mut revision = VoxelSectionRevisionResult::default();
    assert_eq!(
        unsafe {
            (table.section_revision_query.unwrap())(provider.as_opaque_ptr(), &key, &mut revision)
        },
        LumioStatus::Success as i32
    );
    assert_eq!(revision.presence, VoxelPresence::Ready);
    assert_eq!(revision.section_revision, 12);
    assert_eq!(
        unsafe {
            (table.block_read_cell.unwrap())(
                provider.as_opaque_ptr(),
                std::ptr::null(),
                &mut Default::default(),
            )
        },
        LumioStatus::InvalidArgument as i32
    );
}

#[test]
fn native_provider_is_backed_by_a_running_voxel_world() {
    let provider = NativeVoxelProvider::new();
    assert_eq!(provider.world_state().lifecycle(), "Running");
    assert_eq!(provider.world_state().role(), "Authority");
}

#[test]
fn residency_slots_route_to_the_paired_pin_manager() {
    let mut provider = NativeVoxelProvider::new();
    provider.seed_ready_section(VoxelSectionKey::new(0, 0, 0), 12, 1);
    let mut table = std::ptr::null();
    assert_eq!(
        unsafe { lumio_engine_get_api_v1(1, &mut table) },
        LumioStatus::Success as i32
    );
    let table = unsafe { &*table };
    let key = VoxelSectionKey::new(0, 0, 0);
    let mut pin = std::ptr::null_mut();
    assert_eq!(
        unsafe {
            (table.residency_pin_declare.unwrap())(provider.as_opaque_ptr(), &key, 1, 1, &mut pin)
        },
        LumioStatus::Success as i32
    );
    let mut status = lumio_engine_native::VoxelPinStatus {
        ready: 0,
        _reserved: [0; 7],
        section_count: 0,
        ready_section_count: 0,
    };
    assert_eq!(
        unsafe {
            (table.residency_pin_status.unwrap())(provider.as_opaque_ptr(), pin, &mut status)
        },
        LumioStatus::Success as i32
    );
    assert_eq!(status.ready, 1);
    assert_eq!(status.section_count, 1);
    assert_eq!(status.ready_section_count, 1);
    assert_eq!(
        unsafe { (table.residency_pin_release.unwrap())(provider.as_opaque_ptr(), pin) },
        LumioStatus::Success as i32
    );
}

#[test]
fn ready_pin_rejects_later_pending_and_unavailable_abi_reads() {
    let mut provider = NativeVoxelProvider::new();
    provider.seed_ready_section(VoxelSectionKey::new(0, 0, 0), 12, 1);
    let mut table = std::ptr::null();
    assert_eq!(
        unsafe { lumio_engine_get_api_v1(1, &mut table) },
        LumioStatus::Success as i32
    );
    let table = unsafe { &*table };
    let key = VoxelSectionKey::new(0, 0, 0);
    let mut pin = std::ptr::null_mut();
    assert_eq!(
        unsafe {
            (table.residency_pin_declare.unwrap())(provider.as_opaque_ptr(), &key, 1, 1, &mut pin)
        },
        LumioStatus::Success as i32
    );

    let coordinate = VoxelWorldCoordinate::new(0, 1, 0);
    let mut cell = VoxelBlockReadCellResult::default();
    provider.seed_pending_section(key, 13);
    assert_eq!(
        unsafe {
            (table.block_read_cell.unwrap())(provider.as_opaque_ptr(), &coordinate, &mut cell)
        },
        1048
    );

    let request = VoxelBoxRequest::new(coordinate, coordinate);
    let mut cells = [VoxelBlockReadResult::default(); 1];
    let mut cell_count = 0;
    let mut segments = [Default::default(); 1];
    let mut segment_count = 0;
    let mut truncated = 0;
    assert_eq!(
        unsafe {
            (table.block_read_box.unwrap())(
                provider.as_opaque_ptr(),
                (&request as *const VoxelBoxRequest).cast(),
                cells.as_mut_ptr(),
                1,
                &mut cell_count,
                segments.as_mut_ptr(),
                1,
                &mut segment_count,
                &mut truncated,
            )
        },
        1048
    );

    provider.seed_unavailable_section(key, 14);
    assert_eq!(
        unsafe {
            (table.block_read_cell.unwrap())(provider.as_opaque_ptr(), &coordinate, &mut cell)
        },
        1048
    );
    assert_eq!(
        unsafe { (table.residency_pin_release.unwrap())(provider.as_opaque_ptr(), pin) },
        LumioStatus::Success as i32
    );
}

#[test]
fn pending_pin_remains_explicit_before_ready() {
    let mut provider = NativeVoxelProvider::new();
    let key = VoxelSectionKey::new(0, 0, 0);
    provider.seed_pending_section(key, 12);
    let mut table = std::ptr::null();
    assert_eq!(
        unsafe { lumio_engine_get_api_v1(1, &mut table) },
        LumioStatus::Success as i32
    );
    let table = unsafe { &*table };
    let mut pin = std::ptr::null_mut();
    assert_eq!(
        unsafe {
            (table.residency_pin_declare.unwrap())(provider.as_opaque_ptr(), &key, 1, 1, &mut pin)
        },
        LumioStatus::Success as i32
    );
    let mut pin_status = lumio_engine_native::VoxelPinStatus {
        ready: 1,
        _reserved: [0; 7],
        section_count: 0,
        ready_section_count: 1,
    };
    assert_eq!(
        unsafe {
            (table.residency_pin_status.unwrap())(provider.as_opaque_ptr(), pin, &mut pin_status)
        },
        LumioStatus::Success as i32
    );
    assert_eq!(pin_status.ready, 0);

    let coordinate = VoxelWorldCoordinate::new(0, 1, 0);
    let mut result = VoxelBlockReadCellResult::default();
    assert_eq!(
        unsafe {
            (table.block_read_cell.unwrap())(provider.as_opaque_ptr(), &coordinate, &mut result)
        },
        LumioStatus::Success as i32
    );
    assert_eq!(result.presence, VoxelPresence::Pending);
    assert_eq!(result.has_block_id, 0);
    assert_eq!(
        unsafe { (table.residency_pin_release.unwrap())(provider.as_opaque_ptr(), pin) },
        LumioStatus::Success as i32
    );
}

#[test]
fn block_write_abort_releases_the_paired_mutation_reservation() {
    let mut provider = NativeVoxelProvider::new();
    provider.seed_ready_section(VoxelSectionKey::new(0, 0, 0), 12, 1);
    let mut table = std::ptr::null();
    assert_eq!(
        unsafe { lumio_engine_get_api_v1(1, &mut table) },
        LumioStatus::Success as i32
    );
    let table = unsafe { &*table };
    let entry =
        lumio_engine_native::VoxelBlockWriteEntry::new(VoxelSectionKey::new(0, 0, 0), 256, 2, 12);
    let mut token = std::ptr::null_mut();
    assert_eq!(
        unsafe {
            (table.block_write_prepare.unwrap())(provider.as_opaque_ptr(), 8, &entry, 1, &mut token)
        },
        LumioStatus::Success as i32
    );
    assert_eq!(
        unsafe { (table.block_write_abort.unwrap())(provider.as_opaque_ptr(), token) },
        LumioStatus::Success as i32
    );
    let mut replacement = std::ptr::null_mut();
    assert_eq!(
        unsafe {
            (table.block_write_prepare.unwrap())(
                provider.as_opaque_ptr(),
                8,
                &entry,
                1,
                &mut replacement,
            )
        },
        LumioStatus::Success as i32
    );
}

// ---------------------------------------------------------------------------
// R-00477 物理查询三槽（raycast / sweep / overlap）
//
// 场景统一用一个「水上石下」的世界：Section s:0:0:0 满格石头（世界 y 0..15），
// Section s:0:1:0 满格水（世界 y 16..31）。材质类只由创建世界时注入的官方方块目录解析。
// ---------------------------------------------------------------------------

/// 官方方块目录：块类型 256 = 石头（Solid），257 = 水（Liquid）。
/// 这是查询解析材质类的唯一来源（契约 physicsQuery.materialClassTable.source）。
const STONE_BLOCK_TYPE: u32 = 256;
const WATER_BLOCK_TYPE: u32 = 257;
const STONE_BLOCK_ID: u32 = STONE_BLOCK_TYPE << 8;
const WATER_BLOCK_ID: u32 = WATER_BLOCK_TYPE << 8;

/// 契约 voxel.enums.material_mask：位 0 = Solid、位 1 = Liquid。
const MASK_SOLID: u32 = 1;
const MASK_LIQUID: u32 = 2;

fn catalog_row(block_type: u32, name: &str, material_class: &str) -> BlockCatalogRowInput {
    BlockCatalogRowInput {
        block_type: Some(block_type),
        name: Some(name.to_owned()),
        material_class: Some(material_class.to_owned()),
        behavior_template: Some(if material_class == "Liquid" {
            "Liquid".to_owned()
        } else {
            "FullCube".to_owned()
        }),
        asset_ref: Some(format!("asset://blocks/{name}")),
        state_layout: Some(StateLayout::empty()),
    }
}

fn stone_and_water_catalog() -> OfficialCatalog {
    OfficialCatalog::load(
        vec![
            catalog_row(STONE_BLOCK_TYPE, "lumio.stone", "Solid"),
            catalog_row(WATER_BLOCK_TYPE, "lumio.water", "Liquid"),
        ],
        &[],
    )
    .expect("官方方块目录必须是稠密且合法的")
}

/// 石头在下、水在上的可查询世界（材质类表已在创建世界时注入）。
fn water_over_stone_world() -> NativeVoxelProvider {
    let mut provider = NativeVoxelProvider::with_material_classes(stone_and_water_catalog());
    provider.seed_ready_section(VoxelSectionKey::new(0, 0, 0), 12, STONE_BLOCK_ID);
    provider.seed_ready_section(VoxelSectionKey::new(0, 1, 0), 12, WATER_BLOCK_ID);
    provider
}

fn root_table() -> &'static LumioEngineRootApiV1 {
    let mut table = std::ptr::null();
    assert_eq!(
        unsafe { lumio_engine_get_api_v1(1, &mut table) },
        LumioStatus::Success as i32
    );
    unsafe { &*table }
}

fn raw_bytes<T>(value: &T) -> &[u8] {
    // SAFETY: 只按 size_of 读同一个已完全写过的 repr(C) 结构的字节。
    unsafe {
        std::slice::from_raw_parts((value as *const T).cast::<u8>(), std::mem::size_of::<T>())
    }
}

/// 一条竖直向下的射线，起点落在水层内部。
fn downward_ray(material_mask: u32) -> VoxelRaycastRequest {
    VoxelRaycastRequest {
        origin: VoxelWorldPoint::new(0.5, 31.5, 0.5),
        direction: VoxelWorldPoint::new(0.0, -1.0, 0.0),
        max_distance: 64.0,
        material_mask,
    }
}

fn call_raycast(
    provider: &mut NativeVoxelProvider,
    request: &VoxelRaycastRequest,
) -> (i32, VoxelRaycastResult) {
    let table = root_table();
    let mut result = VoxelRaycastResult::default();
    let status =
        unsafe { (table.raycast.unwrap())(provider.as_opaque_ptr(), request, &mut result) };
    (status, result)
}

/// 验收 1：路径上有一个 Pending Section 时，结果必须是 Unresolved 且携带那个 Section 键——
/// 不得塌缩成 Miss（穿过去，玩家掉进地里），也不得塌缩成 Hit（卡在空气墙上）。
#[test]
fn physics_raycast_reports_unresolved_with_the_blocking_section_key() {
    let mut provider = NativeVoxelProvider::with_material_classes(stone_and_water_catalog());
    provider.seed_ready_section(VoxelSectionKey::new(0, 0, 0), 12, STONE_BLOCK_ID);
    // 射线先经过的那一层还没就位。
    provider.seed_pending_section(VoxelSectionKey::new(0, 1, 0), 12);

    let (status, result) = call_raycast(&mut provider, &downward_ray(MASK_SOLID));

    assert_eq!(
        status,
        LumioStatus::Success as i32,
        "Unresolved 是正常结局，不是错误码"
    );
    assert_eq!(
        result.resolution,
        VoxelQueryResolution::Unresolved,
        "Pending Section 挡路时不得回报 Hit 或 Miss"
    );
    assert_ne!(result.resolution, VoxelQueryResolution::Miss);
    assert_ne!(result.resolution, VoxelQueryResolution::Hit);
    assert_eq!(result.unresolved_section.x, 0);
    assert_eq!(result.unresolved_section.y, 1);
    assert_eq!(result.unresolved_section.z, 0);
    // 三态之间不得串味：Unresolved 不携带命中数据。
    assert_eq!(result.block_id, 0);
    assert_eq!(result.travel_distance, 0.0);

    // 同一个世界把那层补成 Ready 后必须给出确定结果，证明上面的 Unresolved 是残缺导致的，
    // 而不是这条射线本来就答不出来。
    provider.seed_ready_section(VoxelSectionKey::new(0, 1, 0), 13, WATER_BLOCK_ID);
    let (status, resolved) = call_raycast(&mut provider, &downward_ray(MASK_SOLID));
    assert_eq!(status, LumioStatus::Success as i32);
    assert_eq!(resolved.resolution, VoxelQueryResolution::Hit);
    assert_eq!(resolved.block_id, STONE_BLOCK_ID);
}

/// 验收 3：材质类过滤只查材质类表。Solid 掩码下射线穿过水命中水底石头；
/// Liquid 掩码下命中水面。
#[test]
fn physics_raycast_material_mask_selects_stone_under_water_or_the_water_surface() {
    let mut provider = water_over_stone_world();

    let (status, solid) = call_raycast(&mut provider, &downward_ray(MASK_SOLID));
    assert_eq!(status, LumioStatus::Success as i32);
    assert_eq!(solid.resolution, VoxelQueryResolution::Hit);
    assert_eq!(solid.block_id, STONE_BLOCK_ID, "Solid 掩码必须穿过水");
    assert_eq!(solid.hit_cell.y, 15, "水底石头的最高一格是 y=15");

    let (status, liquid) = call_raycast(&mut provider, &downward_ray(MASK_LIQUID));
    assert_eq!(status, LumioStatus::Success as i32);
    assert_eq!(liquid.resolution, VoxelQueryResolution::Hit);
    assert_eq!(liquid.block_id, WATER_BLOCK_ID, "Liquid 掩码必须命中水");
    assert_eq!(liquid.hit_cell.y, 31, "水面是水层的最高一格 y=31");

    // 掩码 0 不匹配任何材质类，必定 Miss（zeroMatchesNothing）——它既不是「全选」也不是错误码。
    let (status, empty) = call_raycast(&mut provider, &downward_ray(0));
    assert_eq!(status, LumioStatus::Success as i32);
    assert_eq!(empty.resolution, VoxelQueryResolution::Miss);
    assert_eq!(empty.block_id, 0);
}

/// 验收 2：overlap 结果写进调用方缓冲；容量不足时回报 truncated 与实际总数，
/// Native 侧不返回任何需要释放的句柄。
#[test]
fn physics_overlap_fills_the_caller_buffer_and_reports_truncation_with_the_actual_count() {
    let mut provider = water_over_stone_world();
    let table = root_table();
    // x∈[0,10) 一行共 10 格石头，y/z 各只罩一格。
    let request = VoxelOverlapRequest {
        center: VoxelWorldPoint::new(5.0, 5.5, 5.5),
        half_extents: VoxelWorldPoint::new(5.0, 0.5, 0.5),
        material_mask: MASK_SOLID,
    };

    let mut hits = [VoxelOverlapHit::default(); 4];
    let mut result = VoxelOverlapResult::default();
    let status = unsafe {
        (table.overlap.unwrap())(
            provider.as_opaque_ptr(),
            &request,
            hits.as_mut_ptr(),
            hits.len() as u32,
            &mut result,
        )
    };

    assert_eq!(status, LumioStatus::Success as i32);
    assert_eq!(result.resolution, VoxelQueryResolution::Hit);
    assert_eq!(result.actual_count, 10, "实际总数必须是完整的 10 格");
    assert_eq!(result.truncated, 1, "容量 4 装不下 10 格必须显式回报截断");
    for (index, hit) in hits.iter().enumerate() {
        assert_eq!(hit.block_id, STONE_BLOCK_ID);
        assert_eq!(hit.cell.y, 5);
        assert_eq!(hit.cell.z, 5);
        assert_eq!(hit.cell.x, index as i32, "按规范 y/z/x 顺序写入调用方缓冲");
    }

    // 容量够时不得报截断，且实际总数不变。
    let mut roomy = [VoxelOverlapHit::default(); 16];
    let mut complete = VoxelOverlapResult::default();
    let status = unsafe {
        (table.overlap.unwrap())(
            provider.as_opaque_ptr(),
            &request,
            roomy.as_mut_ptr(),
            roomy.len() as u32,
            &mut complete,
        )
    };
    assert_eq!(status, LumioStatus::Success as i32);
    assert_eq!(complete.actual_count, 10);
    assert_eq!(complete.truncated, 0);
    assert_eq!(roomy[9].cell.x, 9);
    assert_eq!(roomy[10].block_id, 0, "容量之外的条目不得被 Native 写脏");
}

/// 验收 1（sweep 面）+ 契约 sweep 输出：Hit 给 collided 与 0..1 的可行进比例，
/// Miss 表示整段位移都走得通，Unresolved 仍然单独成一态并携带 Section 键。
#[test]
fn physics_sweep_reports_collision_fraction_and_keeps_unresolved_separate() {
    let mut provider = water_over_stone_world();
    let table = root_table();
    let mut hit = VoxelSweepResult::default();
    // 半高 0.5 的盒子从 y=20 往下扫 8 格：石头顶面在 y=16，途中会撞上。
    let request = VoxelSweepRequest {
        center: VoxelWorldPoint::new(0.5, 20.0, 0.5),
        half_extents: VoxelWorldPoint::new(0.4, 0.5, 0.4),
        displacement: VoxelWorldPoint::new(0.0, -8.0, 0.0),
        material_mask: MASK_SOLID,
    };
    let status = unsafe { (table.sweep.unwrap())(provider.as_opaque_ptr(), &request, &mut hit) };
    assert_eq!(status, LumioStatus::Success as i32);
    assert_eq!(hit.resolution, VoxelQueryResolution::Hit);
    assert_eq!(hit.collided, 1);
    assert!(
        (0.0..=1.0).contains(&hit.travel_fraction),
        "可行进比例必须落在 0..1，实得 {}",
        hit.travel_fraction
    );
    assert!(hit.travel_fraction > 0.0, "起点不在石头里，不应原地卡死");
    assert_eq!(hit.block_id, STONE_BLOCK_ID);
    assert_eq!(hit.hit_cell.y, 15);

    // 同一段位移换成 Liquid 掩码：石头不在掩码里，水也不挡这段路径的终点 —— 撞到的是水。
    let mut liquid = VoxelSweepResult::default();
    let liquid_request = VoxelSweepRequest {
        material_mask: MASK_LIQUID,
        ..request
    };
    let status =
        unsafe { (table.sweep.unwrap())(provider.as_opaque_ptr(), &liquid_request, &mut liquid) };
    assert_eq!(status, LumioStatus::Success as i32);
    assert_eq!(liquid.resolution, VoxelQueryResolution::Hit);
    assert_eq!(liquid.block_id, WATER_BLOCK_ID);

    // 掩码 0 谁也不匹配：整段位移都走得通。
    let mut miss = VoxelSweepResult::default();
    let miss_request = VoxelSweepRequest {
        material_mask: 0,
        ..request
    };
    let status =
        unsafe { (table.sweep.unwrap())(provider.as_opaque_ptr(), &miss_request, &mut miss) };
    assert_eq!(status, LumioStatus::Success as i32);
    assert_eq!(miss.resolution, VoxelQueryResolution::Miss);
    assert_eq!(miss.collided, 0);
    assert_eq!(miss.travel_fraction, 1.0, "Miss 表示整段位移都走得通");

    // Pending 层挡路时 sweep 同样是第三态，不塌缩。
    let mut pending_provider =
        NativeVoxelProvider::with_material_classes(stone_and_water_catalog());
    pending_provider.seed_ready_section(VoxelSectionKey::new(0, 0, 0), 12, STONE_BLOCK_ID);
    pending_provider.seed_pending_section(VoxelSectionKey::new(0, 1, 0), 12);
    let mut unresolved = VoxelSweepResult::default();
    let status = unsafe {
        (table.sweep.unwrap())(pending_provider.as_opaque_ptr(), &request, &mut unresolved)
    };
    assert_eq!(status, LumioStatus::Success as i32);
    assert_eq!(unresolved.resolution, VoxelQueryResolution::Unresolved);
    assert_ne!(unresolved.resolution, VoxelQueryResolution::Miss);
    assert_ne!(unresolved.resolution, VoxelQueryResolution::Hit);
    assert_eq!(unresolved.collided, 0);
    assert_eq!(unresolved.unresolved_section.y, 1);
}

/// 验收 4：三种查询都是只读的——查询前后 WorldRevision 与每一个 SectionRevision 逐一不变。
#[test]
fn physics_queries_leave_world_and_every_section_revision_untouched() {
    let mut provider = water_over_stone_world();
    let table = root_table();
    let before = provider.published_revisions();
    assert!(!before.1.is_empty(), "基线必须真的包含 Section 修订号");

    let mut ray = VoxelRaycastResult::default();
    assert_eq!(
        unsafe {
            (table.raycast.unwrap())(
                provider.as_opaque_ptr(),
                &downward_ray(MASK_SOLID),
                &mut ray,
            )
        },
        LumioStatus::Success as i32
    );
    let mut swept = VoxelSweepResult::default();
    let sweep_request = VoxelSweepRequest {
        center: VoxelWorldPoint::new(0.5, 20.0, 0.5),
        half_extents: VoxelWorldPoint::new(0.4, 0.5, 0.4),
        displacement: VoxelWorldPoint::new(0.0, -8.0, 0.0),
        material_mask: MASK_SOLID,
    };
    assert_eq!(
        unsafe { (table.sweep.unwrap())(provider.as_opaque_ptr(), &sweep_request, &mut swept) },
        LumioStatus::Success as i32
    );
    let mut hits = [VoxelOverlapHit::default(); 16];
    let mut overlap_result = VoxelOverlapResult::default();
    assert_eq!(
        unsafe {
            (table.overlap.unwrap())(
                provider.as_opaque_ptr(),
                &VoxelOverlapRequest {
                    center: VoxelWorldPoint::new(5.0, 5.5, 5.5),
                    half_extents: VoxelWorldPoint::new(5.0, 0.5, 0.5),
                    material_mask: MASK_SOLID,
                },
                hits.as_mut_ptr(),
                hits.len() as u32,
                &mut overlap_result,
            )
        },
        LumioStatus::Success as i32
    );

    let after = provider.published_revisions();
    assert_eq!(before.0, after.0, "WorldRevision 不得被查询改动");
    assert_eq!(
        before.1.len(),
        after.1.len(),
        "查询不得增删 Section 修订号条目"
    );
    for (section, revision) in &before.1 {
        assert_eq!(
            after.1.get(section),
            Some(revision),
            "Section {section} 的修订号被查询改动了"
        );
    }
}

/// 结果结构的字节槽：对齐 8、够装最大的 `VoxelSweepResult`（64 字节）。
///
/// 两次调用各自从**不同的毒化字节**出发，只有当 Native 把结构里的每一个字节（含结构填充）
/// 都确定性写满时，两份字节才可能相等——这才是「逐字节相同」，而不是「恰好没读到脏字节」。
#[repr(C, align(8))]
struct ResultBytes([u8; 64]);

impl ResultBytes {
    fn poisoned(byte: u8) -> Self {
        Self([byte; 64])
    }

    fn as_mut<T>(&mut self) -> *mut T {
        assert!(std::mem::size_of::<T>() <= 64);
        assert!(std::mem::align_of::<T>() <= 8);
        self.0.as_mut_ptr().cast::<T>()
    }

    fn bytes<T>(&self) -> &[u8] {
        &self.0[..std::mem::size_of::<T>()]
    }
}

/// 验收 5：同一份世界、同一组输入，两次调用的结果逐字节相同（含结构填充字节）。
#[test]
fn physics_query_results_are_byte_identical_on_repeated_calls() {
    let mut provider = water_over_stone_world();
    let table = root_table();

    let request = downward_ray(MASK_SOLID);
    let mut first = ResultBytes::poisoned(0x00);
    let mut second = ResultBytes::poisoned(0xff);
    assert_eq!(
        unsafe {
            (table.raycast.unwrap())(
                provider.as_opaque_ptr(),
                &request,
                first.as_mut::<VoxelRaycastResult>(),
            )
        },
        LumioStatus::Success as i32
    );
    assert_eq!(
        unsafe {
            (table.raycast.unwrap())(
                provider.as_opaque_ptr(),
                &request,
                second.as_mut::<VoxelRaycastResult>(),
            )
        },
        LumioStatus::Success as i32
    );
    let ray = unsafe { *first.as_mut::<VoxelRaycastResult>() };
    assert_eq!(ray.resolution, VoxelQueryResolution::Hit);
    assert_eq!(ray.block_id, STONE_BLOCK_ID);
    assert_eq!(
        first.bytes::<VoxelRaycastResult>(),
        second.bytes::<VoxelRaycastResult>(),
        "raycast 结果在同一世界同一输入下必须逐字节相同"
    );

    let sweep_request = VoxelSweepRequest {
        center: VoxelWorldPoint::new(0.5, 20.0, 0.5),
        half_extents: VoxelWorldPoint::new(0.4, 0.5, 0.4),
        displacement: VoxelWorldPoint::new(0.0, -8.0, 0.0),
        material_mask: MASK_SOLID,
    };
    let mut sweep_first = ResultBytes::poisoned(0x00);
    let mut sweep_second = ResultBytes::poisoned(0xff);
    assert_eq!(
        unsafe {
            (table.sweep.unwrap())(
                provider.as_opaque_ptr(),
                &sweep_request,
                sweep_first.as_mut::<VoxelSweepResult>(),
            )
        },
        LumioStatus::Success as i32
    );
    assert_eq!(
        unsafe {
            (table.sweep.unwrap())(
                provider.as_opaque_ptr(),
                &sweep_request,
                sweep_second.as_mut::<VoxelSweepResult>(),
            )
        },
        LumioStatus::Success as i32
    );
    assert_eq!(
        unsafe { *sweep_first.as_mut::<VoxelSweepResult>() }.resolution,
        VoxelQueryResolution::Hit
    );
    assert_eq!(
        sweep_first.bytes::<VoxelSweepResult>(),
        sweep_second.bytes::<VoxelSweepResult>(),
        "sweep 结果必须逐字节相同"
    );

    let overlap_request = VoxelOverlapRequest {
        center: VoxelWorldPoint::new(5.0, 5.5, 5.5),
        half_extents: VoxelWorldPoint::new(5.0, 0.5, 0.5),
        material_mask: MASK_SOLID,
    };
    // 命中条目缓冲同样从不同的毒化字节出发。VoxelOverlapHit 的字段全是整数，任意位型都合法。
    let mut hits_first = [VoxelOverlapHit::default(); 10];
    let mut hits_second = [VoxelOverlapHit::default(); 10];
    unsafe {
        std::ptr::write_bytes(hits_first.as_mut_ptr(), 0x00, hits_first.len());
        std::ptr::write_bytes(hits_second.as_mut_ptr(), 0xff, hits_second.len());
    }
    let mut overlap_first = ResultBytes::poisoned(0x00);
    let mut overlap_second = ResultBytes::poisoned(0xff);
    assert_eq!(
        unsafe {
            (table.overlap.unwrap())(
                provider.as_opaque_ptr(),
                &overlap_request,
                hits_first.as_mut_ptr(),
                hits_first.len() as u32,
                overlap_first.as_mut::<VoxelOverlapResult>(),
            )
        },
        LumioStatus::Success as i32
    );
    assert_eq!(
        unsafe {
            (table.overlap.unwrap())(
                provider.as_opaque_ptr(),
                &overlap_request,
                hits_second.as_mut_ptr(),
                hits_second.len() as u32,
                overlap_second.as_mut::<VoxelOverlapResult>(),
            )
        },
        LumioStatus::Success as i32
    );
    assert_eq!(
        unsafe { *overlap_first.as_mut::<VoxelOverlapResult>() }.actual_count,
        10
    );
    assert_eq!(
        overlap_first.bytes::<VoxelOverlapResult>(),
        overlap_second.bytes::<VoxelOverlapResult>(),
        "overlap 结果必须逐字节相同"
    );
    assert_eq!(
        raw_bytes(&hits_first),
        raw_bytes(&hits_second),
        "写进调用方缓冲的命中条目必须逐字节相同（含结构填充）"
    );
}

/// 契约 materialClassTable：没有在创建世界时注入材质类表的世界不得回答任何查询，
/// 也不得回落到实现内置的默认表。
#[test]
fn physics_refuses_a_world_without_an_injected_material_class_table() {
    let mut provider = NativeVoxelProvider::new();
    provider.seed_ready_section(VoxelSectionKey::new(0, 0, 0), 12, STONE_BLOCK_ID);
    provider.seed_ready_section(VoxelSectionKey::new(0, 1, 0), 12, WATER_BLOCK_ID);

    let (status, _) = call_raycast(&mut provider, &downward_ray(MASK_SOLID));
    assert_eq!(
        status,
        contract_status("collision_behavior_not_from_material_table"),
        "未注入材质类表的世界必须拒绝查询，而不是当作不阻挡放行"
    );

    // 同一个世界注入目录后立刻可查询，证明上面的拒绝来自「没有表」而不是别的原因。
    let mut injected = water_over_stone_world();
    let (status, _) = call_raycast(&mut injected, &downward_ray(MASK_SOLID));
    assert_eq!(status, LumioStatus::Success as i32);
}

/// 契约 material_mask.reservedBitsMustBeZero：第 2 位及以上置位即 unknown_material_class，
/// 不得当作「未来的类」静默忽略。
#[test]
fn physics_rejects_reserved_material_mask_bits_on_every_slot() {
    let mut provider = water_over_stone_world();
    let table = root_table();
    let reserved = MASK_SOLID | MASK_LIQUID | 0b100;
    let expected = contract_status("unknown_material_class");

    let (status, _) = call_raycast(&mut provider, &downward_ray(reserved));
    assert_eq!(status, expected, "raycast 必须拒绝保留位");

    let mut swept = VoxelSweepResult::default();
    let status = unsafe {
        (table.sweep.unwrap())(
            provider.as_opaque_ptr(),
            &VoxelSweepRequest {
                center: VoxelWorldPoint::new(0.5, 20.0, 0.5),
                half_extents: VoxelWorldPoint::new(0.4, 0.5, 0.4),
                displacement: VoxelWorldPoint::new(0.0, -8.0, 0.0),
                material_mask: reserved,
            },
            &mut swept,
        )
    };
    assert_eq!(status, expected, "sweep 必须拒绝保留位");

    let mut hits = [VoxelOverlapHit::default(); 4];
    let mut overlap_result = VoxelOverlapResult::default();
    let status = unsafe {
        (table.overlap.unwrap())(
            provider.as_opaque_ptr(),
            &VoxelOverlapRequest {
                center: VoxelWorldPoint::new(5.0, 5.5, 5.5),
                half_extents: VoxelWorldPoint::new(5.0, 0.5, 0.5),
                material_mask: reserved,
            },
            hits.as_mut_ptr(),
            hits.len() as u32,
            &mut overlap_result,
        )
    };
    assert_eq!(status, expected, "overlap 必须拒绝保留位");

    // 反例：把保留位去掉后同一组调用必须成功，证明上面拒的是保留位本身。
    let (status, _) = call_raycast(&mut provider, &downward_ray(MASK_SOLID | MASK_LIQUID));
    assert_eq!(status, LumioStatus::Success as i32);
}

/// 契约错误码的稳定状态码：`errorStatusBase + errorCodes 里的下标`，
/// 与 `native-abi.json`（唯一真值）逐条对齐。
fn contract_status(error_code: &str) -> i32 {
    const DEFINITION: &str = include_str!("../../../../abi/native-abi.json");
    let base_key = "\"errorStatusBase\":";
    let base_start = DEFINITION
        .find(base_key)
        .expect("native-abi.json must declare voxel.errorStatusBase")
        + base_key.len();
    let base_rest = &DEFINITION[base_start..];
    let base: i32 = base_rest[..base_rest.find(',').expect("errorStatusBase 后必须还有字段")]
        .trim()
        .parse()
        .expect("errorStatusBase 必须是整数");

    let codes_key = "\"errorCodes\":";
    let codes_start = DEFINITION
        .find(codes_key)
        .expect("native-abi.json must declare voxel.errorCodes")
        + codes_key.len();
    let codes_rest = &DEFINITION[codes_start..];
    let open = codes_rest.find('[').expect("errorCodes 必须是数组");
    let close = codes_rest.find(']').expect("errorCodes 数组必须闭合");
    let mut cursor = &codes_rest[open + 1..close];
    let mut index = 0_i32;
    while let Some(quote) = cursor.find('"') {
        let after = &cursor[quote + 1..];
        let end = after
            .find('"')
            .expect("errorCodes 条目必须是带引号的字符串");
        if &after[..end] == error_code {
            return base + index;
        }
        index += 1;
        cursor = &after[end + 1..];
    }
    panic!("`{error_code}` 不在 native-abi.json 的 voxel.errorCodes 里");
}
