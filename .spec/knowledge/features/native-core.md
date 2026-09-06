---
name: native-core
description: NativeCore 能力目录与接入指南——每个 crate 能干什么、唯一推荐入口、谁在用、怎么到达；上游做通用底层能力前必查
metadata:
  type: doc
  status: 设计中
---

# NativeCore 能力目录与接入指南

简介：`LumioNativeCore` 是引擎最底层、领域无关的 Rust 内核仓（资源身份、有限调度、定时、空间、状态机）。本文是给上游七个仓（Voxel / Runtime / Server / Client / Game / Config / Platform）看的**目录**：有什么、从哪个入口用、今天谁在用、怎么到达。规矩只有一条（[`rules/system.md`](../../rules/system.md)「底层库」）：**通用、领域无关的底层能力先查这里，有就用、没有先向架构仓主会话提需求（由主会话给 NativeCore 开契约卡），不在上游另造。**

## 背景 / 目标

- 2026-09-05 / 06 两次盘点的同一个结论：约 6,000 行内核里只有 timer 有真实消费链，其余模块「有货、没人知道」。Owner 2026-09-06 裁决（[ADR-069](../../decisions/ADR-069-nativecore-audit-rulings.md) 第 5 条）：不删没人用的模块，改为开放优先——让上游知道并优先使用。
- 本文只描述现状与入口，不复制 NativeCore 仓内的模块 README（那是细节真值，本文逐项链过去）。

## 设计

### 怎么到达（三条路，只有这三条）

| 路 | 谁走 | 形态 |
| --- | --- | --- |
| Rust 进程内 | 架构仓 `sdk-native`、VoxelEngine、Server 的 Rust 宿主 | Cargo 路径依赖直接 `use`（今天 `sdk-native/Cargo.toml` 依赖 `lumio-kernel` 与 `lumio-timer`） |
| C ABI 槽 | Runtime / Server / Client 的 C# 侧 | NativeCore 不导出 C 符号；架构仓 `engine/abi/native-abi.json` 加槽 → `sdk-native` 写插头 → C# facade（timer 的 14 个 `timer_*` 槽就是样板） |
| 浏览器 | 客户端预测世界（[ADR-067](../../decisions/ADR-067-browser-client-prediction-dotnet-wasm.md)） | 同一份 Rust 编 wasm32；**今天没有任何 crate 证明过能编**，`lumio-hfsm` + `lumio-kernel` 是第一个必须证明的 |

### 接入步骤（上游视角）

1. 先在下表找：这件事 NativeCore 有没有做。有 → 走对应那条路；表里「谁在用」为空不代表不能用，代表你是第一个。
2. 没有但属于「通用、领域无关」→ 向架构仓主会话提需求（写清输入 / 输出 / 谁调用 / 在哪一相收回），由主会话给 NativeCore 开契约卡；上游不得自己先写一份「临时的」。
3. 属于领域逻辑（体素规则、ECS、玩法）→ 不进 NativeCore，回自己仓。判据：换一个游戏它还成立吗，不成立就不是底层。
4. 第一个消费者接入时，公开 Rust API 允许按消费者形状改（开发期不承诺二进制 Rust ABI）；改了要在架构仓复跑 `cargo build/test -p lumio-engine-native`。

### 目录（按 crate；「唯一推荐入口」= 上游只从这一个类型进，其余导出不当入口）

| crate | 能干什么（大白话） | 唯一推荐入口 | 谁在用 | 现状 |
| --- | --- | --- | --- | --- |
| `lumio-kernel` | 一个「上下文」当所有 Native 资源的根：句柄（带世代，用旧句柄会被拒）、字节预算、资源登记、可续推进的关闭 | `KernelContext::create(config)` + 实现 `ContextResource` 注册进去 | SDK 仅作编译期标记 | 内核成型；无宿主真正创建过 Context |
| `lumio-timer` | 两种钟（墙钟毫秒 / 逻辑 tick）的定时器：一次性、重复、取消、按 Slot 排队投递、确定性顺序 | `TimerManager::with_mode` → `create_slot` / `schedule_*` / `advance` 或 `pump` / `drain_records` | **SDK 14 个 `timer_*` 槽**；Server / Client 经 SDK | 唯一有真实消费链的模块；只管宿主节拍，不进预测世界（[`gas.md`](gas.md)） |
| `lumio-job` | 把一段纯计算交给 Native 执行：提交输入、（可选）worker 线程或手动泵、协作取消、取回结果并回收 | `JobSystem::create(context, config, registry, clock)` → `submit_input` / `pump_one` / `take_result`；计算实现 `TypedKernel::execute` | 无 | 执行链已闭合；预期第一个消费者是 [`tick.md`](tick.md) 第 6 相 `NativeJobBarrier` 收回空间粗筛结果 |
| `lumio-spatial` | AABB 包围盒的插入 / 删除 / 批量交叠查询（R-tree 后端 + 独立暴力对拍实现） | `SpatialContext` → `upsert` / `remove` / `query_aabb_batch` | 无 | 是通用交叠查询，**不是** [`ds-server.md`](ds-server.md) M5 要的「网格 + 双半径 + 候选进 / 出对有序清单」；M5 开卡时按那个形状改 |
| `lumio-hfsm` | 层级状态机的迁移计算器：你给状态图定义 + 当前快照 + 事件 + Guard 结果，它算出「退出什么、进入什么、请求什么动作」，不持状态、不回调、不读钟 | `compile(spec, limits)` → `evaluate_batch(items, limits, out, scratch)`；定义共享用 `HfsmDefinitionRegistry`（`ContextResource`） | 无 | **引擎内一切状态机的唯一迁移语义实现**（ADR-069 第 1 条）；ABI 槽 / 插头 / C# facade 未建，wasm32 未证明；第一个消费者由 Owner 指定 |
| `lumio-platform` | 可注入的单调时钟与 Deadline | 内核内部用 | 内核内部 | 不单独接入 |
| `lumio-codec` / `lumio-diagnostics` | 压缩接缝 / 有界记录器 | 默认关（`prototype` feature） | 无 | 原型，不可用；不进目录 |
| `lumio-test-support` | 测试时钟、交错辅助 | dev-only | 测试 | 不进目录 |

### 一个游戏例子

1. Server 想让每个 Bot「每 2 秒说一句话」，Client 想「断线 30 秒内保留角色」。
2. 两边都不写自己的计时循环：Server 用 SDK 的 `timer_*` 槽建一个 tick 钟、Client 建一个墙钟，各自 `schedule_repeating` / `schedule_one_shot`，每帧 `drain_records` 取到期记录。
3. 「说什么话」「保留谁」是业务，留在各自仓；NativeCore 只管「到点了」。
4. 明天 Runtime 要做连接状态机（连接中 → 已认证 → 断线保留 → 关闭）：不写 `switch`，把状态图交给 `lumio-hfsm`，自己只持快照、答 Guard、执行动作。

### Known gaps（登记，不开卡）

- job 导出但内部不用的 `CompletionBatch` / `BoundedJobQueue` / `CancellationSource` / `JobStateMachine`，`TimedOut` 不可达；timer 的 `DispatchTarget` 只为测试签名存在。上游不要从这些进（ADR-069 第 5 条）。
- timer `advance` 在单 tick 到期总数超预算时会永久拒绝（静态反例，N-W2 修）。

## 待解决

- `lumio-hfsm` + `lumio-kernel` 的 wasm32 编译证明（ADR-069 kill criterion）。
- hfsm 第一个消费者（Owner 指定）与 `native-abi.json` 槽位形状。
- spatial 改形为 M5 粗筛内核的契约（等 DS 视野排期）。
- job 第一个消费者（`NativeJobBarrier`）的收回形状。

## 相关

- [ADR-069](../../decisions/ADR-069-nativecore-audit-rulings.md)、[`2026-09-06-nativecore-source-audit-reassessment.md`](../../reviews/2026-09-06-nativecore-source-audit-reassessment.md)
- [`architecture.md`](architecture.md) §仓库边界、[`repository-architecture.md`](../standards/repository-architecture.md)
- NativeCore 仓：根 `README.md`「crate 一览」、各 `crates/*/README.md`、`docs/specs/`（`kernel-context-lifecycle.md` / `job-state-machine.md` / `spatial-backend.md` / `hfsm-semantics.md`）
