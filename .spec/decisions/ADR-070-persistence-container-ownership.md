# ADR-070：持久化容器层契约与所有权分工——容器层归 Server、内容层归 Runtime、Journal 重做顺序与 ADR-063 对齐

状态：Draft（2026-09-06；随首个消费方接入验证后转 Accepted）
关联：[`save-load.md`](../knowledge/features/save-load.md)（§4 功能模块与 §5 阶段 0 表）、[`ADR-063`](ADR-063-architecture-review-owner-rulings-identity-persist-prediction.md)（第 3 条发号预留、第 6 条 Sync 脏账）、[`ADR-068`](ADR-068-development-verification-follow-main.md)（依赖仓跟 main 不钉号）、[`wire/README.md`](../../engine/wire/README.md)（契约登记）
Owner：`LumioGameEngine`（规则与契约真值）；`LumioServer`（容器层实现）、`LumioGameRuntime`（内容层实现）

## 背景（大白话）

此前 LumioServer 在 PR #38 中单方面实现了 `CheckpointStore` 与一段 `Journal` 帧格式（`LW01` + 裸哈希链），但架构仓尚未定义公共持久化容器契约，导致：
1. Server 独自决定了 Journal 物理帧格式，而上层 Runtime 尚未定义流水账的 opaque payload 与切点，造成零调用的死代码；
2. save-load 设计中规定的「成组原子换档」、「三档耐久 profile 声明」、「恢复固定顺序（注册表→世界→实体→落位）」缺少跨仓可引用的公共 Schema；
3. Server 与 Runtime 职责边界模糊，容易出现两仓重复实现或越权解析 payload 的情况。

为了解决上述问题，架构仓通过 [`persistence-container-v1.json`](../../engine/wire/persistence-container-v1.json) 正式冻结持久化容器契约。

## 决策

1. **分层分工原则：容器归 Server，内容归 Runtime。严禁一项双归属。**
   - **Server（容器层）**：拥有世界存档物理目录（一个世界一个目录）、单写锁（`writer.lock`）、草稿区写入与原子交接（`checkpoint-<generation>.draft` -> `checkpoint-<generation>`）、旧档垫底保留与修剪、物理 fsync 同步策略、流水记录追加与哈希链完整性校验、向 Runtime 发送耐久回执（`durableThroughSeq`）、以及按固定阶段编排恢复流水线。Server **严禁**解析或修改 Runtime/Voxel 的 opaque payload 字节。
   - **Runtime（内容层）**：拥有提交点切片、ECS 实体列表序列化、Voxel 改动层切片、流水变更集（opaque body）、各 payload 的 SHA-256 摘要与字节计算、接收耐久回执并清除 `Sync<T>` 脏账、以及启动时的构造期修剪与纯 `OnHydrate` 重建。Runtime **严禁**直接执行宿主文件系统锁定或跨进程目录重命名。
2. **Server `persistence.rs` 现状收敛顺序**：
   - 现有的 `CheckpointStore` 保留，并按 `persistence-container-v1` 的 manifest 字段与原子语义收敛（补充耐久档位声明、baseMap 户口占位与已占号段引用）；
   - 现有的 `Journal`（Server 自定的 `LW01` 帧）删除；后续流水账必须按本契约的 `journalEnvelope`（`seq`, `tick`, `prevHash`, `checksum`, `providerId`, `body`）标准实现，不再维护私有协议。
3. **贯彻 ADR-063 第 3 条与第 6 条**：
   - **第 3 条（发号器快照存「已占到哪」）**：Manifest 中 `reservedThroughNetEntityId` 记录已向磁盘预留的最高号，崩溃重启后直接从其后继续发号，未使用的号自然作废；
   - **第 6 条（存档记账只有一本）**：内容层流水只从 `Sync<T>` 变更集取样，持久化回执到达后方可清除对应 sequence 的脏标记。
4. **硬性恢复顺序与挂起语义**：
   - 恢复过程必须严格按 ① 注册表（registry）→ ② 世界（world）→ ③ 实体（entities）→ ④ 玩家落位（placement）四步执行；
   - 实体引用的方块或 Section 尚未就位时，必须触发挂起等待状态（`load_suspended_missing_voxel`），严禁将缺块当空气或丢弃实体。

## 职责对照表（对照检查）

| 契约要素 / 字段 | save-load 对应模块 | 归属仓 | 职责说明 |
|---|---|---|---|
| **检查点组 Manifest** | M1（世界清单） / M3（检查点） | **LumioServer** | 维护生成代次、参与者清单、物理落盘与草稿原子重命名 |
| **运行时实体载荷 (`runtime.bin`)** | M4（实体快照） / M7（登录加载） | **LumioGameRuntime** | 序列化实体列表、构造期修剪、纯 `OnHydrate`、计算哈希 |
| **体素改动层载荷 (`voxel.bin`)** | M5（体素改动层） | **LumioVoxelEngine** / **Runtime** | 生成 Section 增量快照、校验 Section 数与哈希 |
| **已占号段 (`reservedThroughNetEntityId`)** | M4 / ADR-063 #3 | **LumioGameRuntime** (产生) / **Server** (落盘) | 发号器预留的高水位界限，快照记录 |
| **流水信封 (`journalEnvelope`)** | M2（流水账 WAL） | **LumioServer** (容器信封) / **Runtime** (内容 body) | Server 维护物理文件与哈希链校验；Runtime 填入 body 业务增量 |
| **耐久回执 (`durabilityReceipt`)** | M6（耐久档位） | **LumioServer** (派发) / **Runtime** (消费) | Server 刷盘完成后下发；Runtime 精确清脏 `Sync<T>` 账本 |
| **三档耐久 Profile** | M6（耐久档位） | **LumioServer** | 开档前必声明；控制文件与目录 fsync 频率与攒批窗口 |
| **恢复顺序编排** | M7（加载管线） / M9（恢复） | **LumioServer** (编排调度) / **Runtime** (节点恢复) | Server 严格按四阶段驱动；Runtime 在缺块时上报挂起状态 |

## 兼容影响

改：[`save-load.md`](../knowledge/features/save-load.md)（阶段 0 表 0-2 / 0-3 行回指 ADR-070 与契约文件）、[`engine/wire/README.md`](../../engine/wire/README.md)（新增 `persistence-container-v1.json` 登记行并标注 ADR-070）。

