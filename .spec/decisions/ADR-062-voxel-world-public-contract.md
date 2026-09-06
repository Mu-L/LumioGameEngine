# ADR-062：体素公共语义改从 `lumio.voxel-world.v1` 取——16³ 数据单元改名 Section，旧制度体素契约作废

状态：Draft（2026-09-06，契约当前为 55 错误码 / 61 规则 / 118 场景用例，见文末 R-00479 / R-00486 / R-00494 三段修订记录；R-00434 的解析域与目录校验增量见 [ADR-066](ADR-066-voxel-owner-rulings.md)，随 M1/M2 实现验证后转 Accepted）
取代：作废 [ADR-024](ADR-024-voxel-p0-contract-set.md)、[ADR-035](ADR-035-voxel-snapshot-payload.md)、[ADR-036](ADR-036-voxel-streaming-durability-ack.md) 中「Chunk 是 16³ 数据单元」的分层语义与其 `schemas/` 依赖；三者的事务、Pin 栅栏、耐久回执与 canonical 排序等**技术结论**仍可被引用，但其 Schema/Fixture 链已随旧制度删除，不再是可校验真值
Owner：`LumioGameEngine`（契约与裁决真值）、`LumioVoxelEngine`（唯一实现）、`LumioGameRuntime` / `LumioGame` / `LumioClient`（消费方）

## 治理原则

- 沿用 ADR-056：**第一性原理——如无必要，勿增实体。** 世界里只有 Section 一个数据单元；每格 dense 数据只有方块一路。
- 沿用 ADR-060：**彻底清理，不留兼容。** 旧的 `page` 概念与 `BlockType` 草案魔数整体删除，不留别名、不留过渡段。

## 背景

体素是最后一块没有公共契约的核心系统。它此前的真值分散在三处，且三处都已失效：

1. **ADR-024 / 035 / 036 依赖的 `schemas/` / `fixtures/` / `ids/` 校验链，已随旧「架构源 + Baseline 门禁 + 八仓镜像」制度一并删除**（见本目录 README 开头）。生成源仓不存在，那批 schema 永远不会再生成。
2. **`LumioVoxelEngine` 把 16×16×16 = 4096 格的数据单元叫 `chunk`**，命名来自上述已死镜像。
3. **`mvp-placevoxel-content-spec.md` §6.2 声称「`MaterialId` 是消费方拥有的目录，Voxel 侧按不透明 uint16 存取」**——与「引擎要解释方块以做调色板、材质类、网格与透光」直接冲突。

后果是消费方按三份互不相容的真值排设计。`LumioGame` 炸弹人 Stage 0a 的 `ITerrainStore`（其仓 ADR 0016）即按第 2、3 条写下：方法名 `ChunkRevision(chunkId)`、`MaterialId` 为不透明 uint16、竖直轴取 `z` 且允许 `z = -1`、快照编码对齐 ADR-035。四条今天全部不成立。

## 决策

- **体素公共语义的唯一真值是 [`engine/wire/voxel-world-v1.json`](../../engine/wire/voxel-world-v1.json)（`lumio.voxel-world.v1`）**，校验入口 `node eng/verify-wire.mjs`。设计说明在 [`knowledge/features/voxel.md`](../knowledge/features/voxel.md)，冲突时以契约为准。消费方不得在实现仓另写一份真值。
- **三层分层与命名冻结。** `Block` → `Section`（16×16×16 = 4096 格，**数据载体、最小同步单位、驻留单位、版本锚点**）→ `Chunk`（竖摞 16 个 Section = 16×256×16，**不携带数据、不持有独立 revision**，是存档打包与按列计算的容器）→ `World`。**任何消费方不得用 Chunk 指代 16³ 数据单元。**
- **规范键以元数防呆。** Section `s:<x>:<y>:<z>`（y 限 0~15），Chunk `c:<x>:<z>`。三坐标的 `c:` 键在语法上即非法，必须显式拒绝，且不得被解读为 `c:x:z` 或 `s:x:y:z`。
- **`BlockId` 是 32 位且引擎解释它**：`BlockType << 8 | BlockState`，高 24 位种类、低 8 位摆法（动态位段）。**一律按无符号处理**——房间局部段的作用域位落在 bit 23，即 `BlockId` 的最高位，用有符号 `int32` 承载会变成负数。**这条取代 `mvp-placevoxel-content-spec.md` §6.2 的「不透明 uint16」。**
- **段表只靠一个位加一个 256 分界**：作用域位（bit 23）= 0 是全局官方段（`0` 空气 / `1` 错误块 / `2` 被 ECS 实体占用 / `3` 结构占位 / `4–255` 系统预留 / `256+` 官方素材库连号稠密）；= 1 是房间局部段（玩家素材库，局部号 `BlockType & 0x7FFFFF`，映射表随存档走）。旧草案的 `9999 / 10000 / 2000000` 三个魔数作废。
- **BlockType 解析域按 [ADR-066](ADR-066-voxel-owner-rulings.md) 固定**：`0..3` 是 typed built-in sentinel（其中 `2` 为 ECS occupancy、`3` 为结构占位），`4..255` 预留且不可解析；普通材质 / 行为模板解析只对已登记官方目录行或已映射房间局部行开放，其他 admitted type 返回 `unregistered_block_type`。
- **材质类是 BlockType 的配表属性，归引擎。** v1 只有 `Solid` / `Liquid` 两类，由**同一张表**声明网格 / 渲染通道 / 碰撞 / 透光四轴；不得在网格器、渲染器、物理、光照里各写一份分支，不得编码进 ID 分段，不得成为第三路逐格数据。新增材质类的唯一判据：这个差异能否只靠贴图表达——能，就不是新类。
- **载荷四编码，一个信封一个分发点。** `Uniform` / `Palette`（8 位索引，≤256 项）/ `Raw` 是全量编码，`Delta`（每条 6 字节 = 格内偏移 + 新 BlockId）是增量编码。**Delta 必须携带 `baseSectionRevision`**，对不上即 `delta_base_revision_mismatch`，拒收并请求一次全量重发，不许静默打补丁；首次送达与重同步禁用 Delta。
- **物理查询移进本契约**（原在 `scope.excludes`）：射线 / 重叠 / 扫掠三种，命中最小单位是 Block，结果三态 `Hit / Miss / Unresolved`。**`Unresolved` 既不等于 Miss 也不等于 Hit，且是正常结局不是错误码。** 阻挡与否只能查材质类表。语义在本契约，**C 函数签名属 [`engine/abi/native-abi.json`](../../engine/abi/native-abi.json)**。
- **光照是派生数据，永不入载荷**、不落盘、不上网（不可变的原始地图预烘焙除外）。
- **方块与实体的绑定只留一条稀疏引用**（`格内偏移 → NetEntityId`），业务数据挂 ECS 实体；体素侧不得自带第二套稀疏业务存储，业务字段不得随体素派发。
- **y 是竖直轴，世界 y 无符号 0~255**（`sectionY = worldY >> 4`、`cellY = worldY & 15`）；x 与 z 是水平轴，各自 signed 32 位，负坐标一等公民。**不接受 y-up / z-up 可配置项**——可配置意味着两套理解并存，而按「z 竖直」写下的代码能照过全部正则，错误要漂到渲染和物理才暴露。
- **官方方块目录是全局段的唯一分配来源**：每行六字段（`blockType` / `name` / `materialClass` / `behaviorTemplate` / `assetRef` / `stateLayout`），从 256 起**连号稠密**分配不留空洞（配表按编号直接下标，空洞会逼出哈希表）；编号与 `name` **永不回收、永不改写、永不重排**；实现仓不得自行铸号。目录校验结构优先：任一必填字段缺失 / null / 空值先报 `block_catalog_row_incomplete`，完整行的非空未知 `materialClass` 才报 `unknown_material_class`。
- **玩法侧批量读是独立于派发的一条路**：三种请求（单格 / 矩形 / 列），**结果必带 `sectionRevision`**，缺块四态与派发面共用一套且**不得把 Pending/Unavailable 填成空气**，预算是声明出来的数字（单次 262144 格 = 64 个 Section）且**超限整条拒绝、不静默截断**，结果写进调用方缓冲。它不是订阅——持续观察一片区域走改动层派发。
- **写入条目是结构化字段，不是字符串 map**：`sectionKey` + 格内偏移（0~4095）+ 新 `BlockId` + **`expectedSectionRevision`**；一批要么全生效要么全不生效（上限 65536 条），按事务 ID 幂等。字符串键值 map 让字段名、类型与边界全都不可机器校验，即 `unstructured_mutation_entry`。
- **尺寸与坐标语义一经冻结即不可变更**，改动等于全量转档，没有例外。

## 本 ADR 明确不冻结的（消费方按现状排期，不要当作已定）

| 面 | 现状 | 归属 |
|---|---|---|
| ABI 面的体素 slot | `engine/abi/native-abi.json` 体素 slot 数为 0；聚合根 `lumio_engine_get_api_v1` 现有 ping / CLR host / timer | 本仓，待开卡 |
| Native 聚合与托管入口 | `engine/native/modules/` 未组入 VoxelEngine；`engine/managed/Lumio.Engine.SDK` 托管入口未开 | 本仓，待开卡 |
| 实现面 | `LumioVoxelEngine` 的段表 / 稠密配表 / 三态存储 / Delta 编解码 / 批量读 / 逐格写全部为零 | 实现仓，待开卡 |

> 原表中的**写入请求形状**、**官方方块目录与铸号规程**、**竖直轴语义**、**玩法侧矩形批量读**四项已于同日补进契约并冻结，见上方「决策」。

## 替代方案

- **继续用 ADR-024/035/036 作真值**：被否。它们的校验链已删除，`chunkOrder` / `voxel-chunk-page` 等 id 指向不可再生成的产物；继续引用会让消费方以为有机器校验兜底。
- **保留 `chunk` 指代 16³、另给列一个新名字**：被否。存量代码与文档里 `chunk` 的两种含义无法用命名区分，而键的元数（三坐标 vs 两坐标）可以——防呆放在语法层比放在文档层可靠。
- **`MaterialId` 维持不透明 uint16、目录归消费方**：被否。调色板、材质类解析、网格合面、透光衰减都要求引擎理解方块；不透明就意味着这四件事全部下沉到消费方，等于每个产品各造一套体素。
- **为玩家方块铸全局唯一编号**：被否。集中铸号且永不回收，百万玩家各造二十块即两千万个号，24 位装不下。

## 失败语义

契约现有 55 个稳定错误码（包括 `unregistered_block_type`、R-00479 追加的 `write_batch_partially_applied` / `base_revision_on_full_encoding` 与 R-00494 追加的 `degenerate_query_shape`；`unknown_section_key` / `unknown_chunk_key` / `delta_base_revision_mismatch` / `delta_used_for_first_delivery` / `lighting_in_payload` / `dirty_section_not_durable` / `block_type_scope_violation` / `room_local_type_without_mapping` / `player_type_declares_behavior` / `palette_reclaim_before_escalation` / `dead_palette_entry_in_payload` / `business_data_in_payload` / `binding_commit_split` 等）。三条红线级失败：**把 `Pending`/`Unavailable`/`Unresolved` 物化成空气**、**携带光照或业务字段入载荷**、**未经回执覆盖卸载脏 Section**。

## 兼容影响与迁移

- 开发态契约，无部署中的体素消费方，不需要迁移窗口。
- `LumioVoxelEngine` 的 `refactor/section-chunk-rename` 分支**已停止推进**（Owner 2026-09-04 裁决）；体素公共语义与其落地由本仓全权负责，不再产出跨仓交接说明。两个 golden 断代（snapshot manifest 摘要、差分 trace）是「尺寸与坐标语义变更等于全量转档」的直接落地，不是测试维护。
- `LumioGame` 炸弹人 `ITerrainStore` 的四条判断需修订，逐条落点见 [`reviews/2026-09-04-bomber-voxel-asks-reply.md`](../reviews/2026-09-04-bomber-voxel-asks-reply.md)。
- 后续改本契约的唯一顺序：本仓改 `engine/wire/voxel-world-v1.json` → 实现仓复制到其 `wire/` → 更新常量与摘要 → 一致性测试变绿。

## 验证

`node eng/verify-wire.mjs` 覆盖 7 份契约；`voxel-world-v1.json` 现有 118 条顶层声明场景（53 testCases + 65 invalidCases），并额外执行 ADR-066 的 resolver / row-validation vectors、R-00479 的 rule↔errorCode↔invalidCase 接线断言与 R-00494 的物理查询声明断言。`node .spec/tools/spec-lint.mjs` 校验本 ADR 的登记与链接可达。实现侧的一致性由 `LumioVoxelEngine` 解析同一份 JSON 逐条断言常量，并校验其 SHA-256 与仓内 `CONTRACT_SHA256` 相符。

## 修订记录（2026-09-06，R-00479 契约三处缺陷修订）

本段为附录，不改写上方决策原文。依据 [`reviews/2026-09-04-voxel-card-contract-drift.md`](../reviews/2026-09-04-voxel-card-contract-drift.md) §六 第 1–4 条与 [`reviews/2026-09-05-engine-repos-progress-assessment.md`](../reviews/2026-09-05-engine-repos-progress-assessment.md) §6 **D11**（Owner 2026-09-06「现在修」）。真值仍是 `engine/wire/voxel-world-v1.json`，`errorCodes` **只在末尾追加**，既有 id、顺序与数值映射（base 1000，`unknown_section_key` = 1000 … `unregistered_block_type` = 1051）一个未动。

- **① 批写入的原子性与尺寸上限拆成两个错误码。** `write.batch-is-all-or-nothing` 的 `onViolation` 由 `write_batch_too_large`（尺寸）改挂新增的 **`write_batch_partially_applied`**（原子性）；`blockWrite.batch.onExceeded` 补上对应 rule **`write.batch-size-cap`**，`onViolation` = `write_batch_too_large`。两个码不得互相顶替。
- **② 全量编码携带 `baseSectionRevision` 有码可引。** 新增 rule **`payload.full-encoding-carries-no-base-revision`** 与错误码 **`base_revision_on_full_encoding`**：`Uniform` / `Palette` / `Raw` 是整体替换，没有基线可对，携带即整条载荷拒收——不得按 Delta 解释，也不得忽略该字段照常接收。原先只有 `sectionPayload.envelope.conditional` 的散文，实现方只能自拟拒绝码。
- **③ pin 驻留预算有了声明字段。** `residency.pinnedRegions.budget` 新增 **`residentSectionBudget`**：量纲 = **Section 数**，由**宿主角色显式声明**，**不得缺省**（未声明按 0 处理，任何 pin 都超预算），**不得被平台在运行时悄悄下调**，专用服务器与浏览器客户端各自声明、永不共用一个数字。配套 rule `residency.pin-budget-is-declared`（复用 `residency_pin_exceeds_budget`，不新增码）。ABI 面由 `residency_pin_declare` 的调用方参数携带该数字，`native-abi.json` 不自造预算。
- **④ 错误码用例覆盖补齐。** `write_batch_too_large`（`entryCount = 65537` > 65536）、`pin_region_not_ready`、`residency_pin_exceeds_budget`（预算 256 Section / 请求 400 Section）与两个新增码各补 invalidCase；`block_catalog_row_incomplete` 原已由 `block_type_without_material_class` 覆盖，本次未重复添加。`eng/verify-wire.mjs` 新增 voxel 专属断言，机器校验 rule ↔ errorCode ↔ invalidCase 的接线、`blockWrite.batch.maxEntriesPerBatch` 与 `limits.maxEntriesPerWriteBatch` 一致、以及预算字段的量纲 / 无缺省 / 不得悄悄下调三条。

**计数变化**：错误码 52 → **54**（追加 `write_batch_partially_applied` = 1052、`base_revision_on_full_encoding` = 1053）；rules 57 → **60**（追加 `write.batch-size-cap`、`payload.full-encoding-carries-no-base-revision`、`residency.pin-budget-is-declared`）；场景用例 110 → **115**（testCases 53 不变，invalidCases 57 → 62）。ABI 生成物随源经 `node eng/generate-abi.mjs` 重生成，三种绑定只在错误常量表末尾各追加两行。

**未在本次修订内**：`LumioVoxelEngine` 的 `crates/lumio-voxel-contracts/wire/` 副本、`CONTRACT_SHA256` 与 `voxel_world.rs` 常量同步走单独 PR（本卡验收 7），顺序仍是「架构仓改 JSON → 递增本 ADR → 实现仓复制副本 → 更新常量与摘要 → 一致性测试变绿」。本契约当前 SHA-256 = `d05dbc52896c535529937ec41d90539f41359a45b42cf05b6a589ef57609939d`。

## 修订记录（2026-09-06，R-00486 物理查询三处声明缺口）

本段为附录，不改写上方决策原文与 R-00479 修订记录。依据 [`reviews/2026-09-05-engine-repos-progress-assessment.md`](../reviews/2026-09-05-engine-repos-progress-assessment.md) 中「三槽的请求结构体把形状与位姿留成裸指针、材质掩码没有位分配、材质类表怎么进 Native 无人声明」三条。真值仍是 `engine/wire/voxel-world-v1.json`；**本次不新增任何错误码**——`unknown_material_class` 早已在表内（index 12 = 1012），`errorCodes` 仍是 54 条，既有 id、顺序与数值映射一个未动；根表槽位 `raycast` = 280 / `sweep` = 288 / `overlap` = 296 与 `physicsSlotsStartOffset` = 280 逐字未动，本次只改请求结构体的**内部**布局。

- **① 形状与位姿合并成一个按值内联的 AABB。** 新增 `physicsQuery.shape`：v1 的 sweep 与 overlap **只接受世界空间轴对齐盒**（`v1Shape: "Aabb"`），由 `center` 与 `halfExtents` 两个 `world_point` 完整描述，**`center` 就是位姿**（`poseIsCenter`），没有旋转与缩放（`noRotation`——要旋转包围盒就自取一个能包住它的 AABB）。形状**必须按值内联进请求结构**（`inlineByValue`）：指针的目标布局若不在契约里，四方无法一致地读同一份数据，生命周期与对齐还得各实现自己猜。形状集合到此为止（`shapeSetIsClosed`，新增形状须开 ADR 并同步 ABI）；任一半长为 0 / 负 / 非有限即非法请求（`degenerateShape`），不得静默当成一个点或 Miss。`queries.raycast/overlap/sweep` 的 `input` 同步改写为具名字段。
- **② 材质掩码有了位分配。** 新增 `physicsQuery.filter.materialMask`：`u32`；位序按 `materialClasses.v1Scope.classes` 的声明顺序从最低位起（`bitOrder`），`bits` = {Solid: 0, Liquid: 1}、`values` = {Solid: 1, Liquid: 2}、`assignedBitCount` = 2；多类按位或（`combining`，两类都要 = 3）。**掩码 0 = 不匹配任何类 = 必定 Miss**（`zeroMatchesNothing`；Unresolved 优先级仍高于 Miss），它不是「全选」也不是错误码。**第 2 位及以上必须为 0**（`reservedBitsMustBeZero`），置位即 `unknown_material_class`；已分配的位永不重排（`appendOnly`）；掩码位与 BlockType 的 ID 分段无关（`notAnIdSegment`，见 `materialClasses.noIdSegmentEncoding`）。`filter.examples` 补上具体掩码值。
- **③ 材质类表的 Native 入口有了落点。** 新增 `physicsQuery.materialClassTable`，`nativeEntry` = **`world-creation-injection`**：由宿主在**创建体素世界时注入**，来源是官方方块目录 `blockCatalog` 的 `materialClass` 一列（LumioConfig 编译出的同一份配表），不是查询实现里的第二份表。**不加根表目录槽**（`whyNotARootSlot`：目录是世界的构造状态不是逐次查询的输入，世界句柄已是每个查询的第一个参数，多开一条入口就多一处能对不上的地方）；**也不做查询参数**（`whyNotAQueryParameter`：每次带一遍会让同一帧两次查询看到两份表，直接违背 `determinism`）。没注入目录的世界不得回答任何查询（`resolvedBeforeQueryable`），不得回落到内置默认表、不得把解析不出材质类的方块当作不阻挡（`noBuiltInFallback`），同一 Tick 内表不得变更（`immutableWithinTick`），实现仓测试助手直接构造材质表不构成生产注入路径（`testHelpersAreNotAPath`）。
- **④ ABI 跟上契约。** `native-abi.json` 的 `sweep_request` 由 `{shape: pointer, pose: pointer, displacement, material_mask}` 改为 `{center: world_point, half_extents: world_point, displacement: world_point, material_mask: u32}`；`overlap_request` 由 `{shape: pointer, pose: pointer, material_mask, _reserved: bytes4}` 改为 `{center, half_extents, material_mask}`——四字节对齐下无尾部填充，**`_reserved` 随之删除，不留兜底字段**。新增 `voxel.enums.material_mask`，位分配与契约同源。
- **⑤ 新增声明有可执行断言。** `eng/verify-wire.mjs` 新增 `checkVoxelPhysicsDeclarations()`：形状块的必备字段与条款、掩码位分配自洽（`bits` 键序等于 `materialClasses.v1Scope.classes`、`bits[c] == c 的序号`、`values[c] == 1 << bits[c]`、`assignedBitCount` 等于类数、`examples` 的掩码值不为 0 且不触碰保留位）、`materialClassTable.nativeEntry` 非空且**不得是根表槽名**、以及 wire ↔ `native-abi.json` 的对齐（`material_mask` 的 bits/values 一致、两个请求结构体逐字段按值内联且无 pointer 字段、`layout.types` 的 size/offsets 与按 C 规则算出的布局一致）。voxel 契约因此开始像 timer 契约一样携带 `native-abi.json` 进校验。

**计数变化**：错误码 54 → **54**（不变）；rules 60 → **60**（不变）；场景用例 115 → **115**（不变）。新增的是三处**声明**，不是新错误面。ABI 侧新增结构字段 4 个（两个请求各 `center` / `half_extents`）、删除 4 个（两个请求各 `shape` / `pose`）、删除 1 个 `_reserved`，新增枚举 1 个（`material_mask`）；`sweep_request` 尺寸 32 → **40**（offsets `center` 0 / `half_extents` 12 / `displacement` 24 / `material_mask` 36），`overlap_request` 尺寸 24 → **28**（offsets `center` 0 / `half_extents` 12 / `material_mask` 24），两者均经 `cc` 编译 `offsetof` 实测复核。三种绑定随源经 `node eng/generate-abi.mjs` 重生成，`DEFINITION_SHA256` = `fd76885af15bb7e3bd3b957ae3ea3f267a56965086d1b2f360861f3034038fec`。

**未在本次修订内**：`LumioVoxelEngine` 的 `crates/lumio-voxel-contracts/wire/` 副本、`CONTRACT_SHA256` 与常量同步仍走单独 PR，顺序不变。本契约当前 SHA-256 = `523aec6e590d8c78f8a15fc4a67e98a80b19ea1ba5a9ced28fdf482d44e48fc9`。

## 修订记录（2026-09-06，R-00494 physicsQuery 契约收尾）

本段为附录，不改写上方决策原文与 R-00479 / R-00486 两段修订记录。依据 [`reviews/2026-09-06-voxel-closeout-batch-report.md`](../reviews/2026-09-06-voxel-closeout-batch-report.md) §三「R-00477 深审把契约缺口查得比实现方更糟」与 §五 R-00494 一行。真值仍是 `engine/wire/voxel-world-v1.json`；`errorCodes` **只在末尾追加**，既有 54 条的 id、顺序与数值映射（base 1000，`unknown_section_key` = 1000 … `base_revision_on_full_encoding` = 1053）一个未动；**根表槽位与顺序逐字未动**（`rootStructSize` = 304，`raycast` = 280 / `sweep` = 288 / `overlap` = 296），本次只追加一条错误码与若干声明。

- **① 退化盒定案：走方案 (a)，被调方必须拒绝。** 原条款「由调用方保证」与「体素侧不得静默当成一个点、也不得静默当成 Miss」互相拉扯，而 54 条错误码里没有一条可用于拒绝。R-00477 深审实测证明**两种被禁止的结局都真实可达**：`halfExtents = 0` 时盒心落在整数格边界返回 `Miss`、落在格心返回 `Hit`——同一个非法请求只因几何位置就被分成两种结局。选 (a) 而不是把它降级为纯前置条件（方案 b）的理由有三：**其一**，被禁止的两种结局正是本契约对 `Unresolved` 已经立为红线的那两种失败（`unresolved_hit_treated_as_air` / `unresolved_hit_treated_as_solid`，后果分别是踩空掉进地里与卡在空气墙上），同一后果不能一处立红线、另一处判未定义；**其二**，本契约有四方消费者（Rust / 托管 / 前端 / 验收），未定义行为意味着各实现可以各自选一种结局，**直接违背 `determinism`「两端必须同结果」**；**其三**，检测成本是三个 f32 的有限性与正负判断，相对 DDA 逐格步进可忽略，用「省一次判断」换一条无人负责的语义不划算。
  新增错误码 **`degenerate_query_shape`**（index 54 = **1054**）与 rule **`query.degenerate-shape-is-rejected`**。`physicsQuery.shape.degenerateShape` 重写为「必须当场拒绝并返回 `degenerate_query_shape`」，并新增三条：`degenerateShapeIsRejectedNotUndefined`（「由调用方保证」不豁免被调方的检测义务，附两种结局的实测对照）、`degenerateShapeCheckedBeforeTraversal`（**遍历之前**逐分量校验有限且严格大于 0；退化盒**不是第四种结局**，不返回 Hit / Miss / Unresolved 中的任何一个，Unresolved 也不行）、`degenerateShapeAppliesTo`（只约束带形状的 sweep 与 overlap，raycast 不带形状不受约束）。
- **② 两半各有一条带真实数字的 invalidCase。** `degenerate_box_on_cell_boundary_silently_missed`（`center` = (5.0, 5.0, 5.0)、`halfExtents` = (0, 0, 0)，未检测的实现观测为 `Miss`）、`degenerate_box_at_cell_center_silently_hits`（同一个盒子只挪到 `center` = (5.5, 5.5, 5.5)，观测为 `Hit`、`actual_count` = 1）、`degenerate_box_with_negative_half_extent_swept`（`halfExtents` = (-0.5, 0.5, 0.5) 的 sweep，观测为 `Miss` 且携带 `travel_fraction` = 1.0）。**只堵一半会漏掉另一半**，所以两半共用同一条在遍历之前生效的拒绝，且由机器断言分别校验。
- **③ KG-2：sweep 的 `travel_fraction` 在三种结局下各有取值声明。** 新增 `physicsQuery.queries.sweep.travelFraction`：Hit 是 0~1 的可行进比例；**Miss 必须是 `1.0`**——没有任何阻挡，整段位移都走得通；**不得写 `0.0`**，那是「一步也走不动」，会把角色钉死在完全空旷的地方；**Unresolved 不给可行进比例**，调用方不得读该字段。附 `whyNotZeroDefault`：结构清零后的默认值恰好是 `0.0`，与「贴着墙动不了」同形，所以 Miss 的 `1.0` 必须显式写出来，不能靠默认值。
- **④ KG-3：物理查询的取数面写进契约。** 新增 `physicsQuery.dataSource`：查询**只从已发布的不可变切面取数**（`statement`）、**永不请求加载**、切面外的 Section 一律 Unresolved（`neverRequestsLoad`）、之所以取发布切面是因为客户端只可能拿到已发布数据、服务端若看见未发布的写入两端就会分叉（`whyPublishedCut`）。**与 `block_read_*` 是两个取数面**（`differsFromBlockRead`）：读接口按坐标问世界当前状态，物理只看发布切面——后果是实测过的，**同一帧、同一坐标，`block_read_cell` 可以是 `Ready` 而 `raycast` / `sweep` / `overlap` 是 `Unresolved`**。物理的 Unresolved 只表示「不在本次切面里」，**不区分**「从未加载」与「已加载但尚未发布」，v1 不提供区分手段，调用方也不得拿 `block_read_cell` 的 presence 去反推或推翻它（`unresolvedMeansNotInTheCut`）；一次查询自始至终只用一份切面（`sameCutWithinOneQuery`）。
- **⑤ KG-4：没注入材质类表时用哪个码，点名了。** `materialClassTable.resolvedBeforeQueryable` 明确为 **`collision_behavior_not_from_material_table`**。**不是 `unknown_material_class`**：那个码已经绑死在「掩码触碰保留位」（`filter.materialMask.reservedBitsMustBeZero`）与「单块解析不出材质类」（`noBuiltInFallback`）两个语义上，两者都能在**有表**的世界里单独到达；再拿它兼表示「整张表都没注入」，调用方就无法分辨是哪一件事。
- **⑥ KG-6：`nonVoxelBodies` 收窄为 v1 不支持。** 原条款要求非体素物体一起参与检测且「命中结果需能区分命中的是方块还是注册物体」，但 ABI 的 `raycast_result` / `sweep_result` / `overlap_result` 三个结构里**没有任何 target 判别字段**、Native 也没有任何注册入口——条款无人能满足。**不选「在 ABI 补 target 字段」**：那属 ABI 变更需另开 ADR，且按世界模型，角色 / NPC / 掉落物是**实体**，位姿与碰撞体归 ECS；把它们注册进体素世界等于在体素侧再开一份实体存储，正是 `boundary.noVoxelSideBusinessStore` 禁止的第二份真相。改为声明块：`v1`（不支持）、`whyNotInV1`（无法满足 + 归属错了两条独立理由）、`callerComposes`（调用方自己把地形结果与 ECS 实体碰撞结果合起来取先发生的那个；这不违反 `boundary.noSecondPhysicsSurface`——那条禁的是再造一份**地形**检测）、`reopeningCondition`（要在引擎面统一回答须**开 ADR**，至少给三个 result 各加 target 判别字段并同步 ABI；v1 不留占位字段、不留半截入口）。`scope.excludes` 同步追加一条。
- **⑦ 新增声明全部有可执行断言。** `eng/verify-wire.mjs` 扩写 `checkVoxelRuleErrorWiring()` 与 `checkVoxelPhysicsDeclarations()`：`degenerate_query_shape` 进 rule↔errorCode↔invalidCase 接线表；`degenerateShape` 必须点名错误码**且两半（Miss / 一个点）都堵**，`degenerateShapeCheckedBeforeTraversal` 必须排除 Unresolved；两半必须**各有**一条带真实 `center` 与真正退化的 `halfExtents` 数字、观测结局分别为 `Miss` 与 `Hit` 的 invalidCase；`travelFraction` 四条款齐备且 `miss` 同时点名 `1.0` 与被排除的 `0.0`、`unresolved` 必须声明「不给」；`dataSource` 六条款齐备且 `differsFromBlockRead` 必须同时点名 `block_read_cell` / `Ready` / `Unresolved`；`resolvedBeforeQueryable` 必须点名 `collision_behavior_not_from_material_table`；`nonVoxelBodies` 必须是声明块、`v1` 必须表述为不支持、`reopeningCondition` 必须要求开 ADR，**并双向锚到 ABI**——`native-abi.json` 的三个 result 结构不得出现 target 判别字段、根表不得出现 `register_body` 之类的注册槽，改口称支持而不动 ABI（或反过来）都过不了。

**计数变化**：错误码 54 → **55**（追加 `degenerate_query_shape` = 1054）；rules 60 → **61**（追加 `query.degenerate-shape-is-rejected`）；场景用例 115 → **118**（testCases 53 不变，invalidCases 62 → 65）。ABI 侧**不新增结构字段、不改任何字段类型与顺序**，只改两处 `doc`（`root.sweep` 与 `voxel.types.sweep_result` 补 Miss = 1.0 / Unresolved 不给）并在 `voxel.semantics` 追加 `degenerateQueryShape` / `sweepTravelFraction` / `publishedCutOnly` / `noNonVoxelBodies` 四条。**根表槽位偏移经真实编译器复核未变**：`cc -O0` + `offsetof` 在改前改后各跑一次，`sizeof(root)` = 304、`raycast` = 280 / `sweep` = 288 / `overlap` = 296、`sizeof(sweep_result)` = 64 全部相同。三种绑定随源经 `node eng/generate-abi.mjs` 重生成，只在错误常量表末尾各追加一行，`DEFINITION_SHA256` = `32f99c21329b064d18431e1d8ef1161f2e71c8fe5bf5eba9e933f99fdedb28de`。Native 与托管两侧以 `native-abi.json` 为真值的穷尽性镜像（`voxel.rs::status_for_error`、`VoxelFacade.cs` 的 `VoxelError` / `ContractStatuses`）同批补齐。

**未在本次修订内**：`LumioVoxelEngine` 的 `crates/lumio-voxel-contracts/wire/` 副本、`CONTRACT_SHA256` 与常量同步走单独 PR，顺序不变。**`lumio-voxel-project` 的 `cell_range` 实现未动**——本次只定契约，退化盒的实际拒绝、sweep Miss 的 `travel_fraction` = 1.0 落地由实现卡承接。本契约当前 SHA-256 = `23030c870cc87b800ab7fca2c0faa8bbc8eb9c6dedfce8987a898b77ccd169a1`。
