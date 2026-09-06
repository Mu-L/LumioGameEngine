---
name: 2026-09-06-nativecore-source-audit-reassessment
description: 外部 NativeCore 源码审计（F01–F14）对照 PR #9 / #10 与 lumio-hfsm 的逐条复核、与设计冲突点、Owner 三条裁决——排 NativeCore 下一批卡前查
metadata:
  type: doc
  status: 已交付
---

# 2026-09-06 NativeCore 源码审计复核

> 对象：外部审计 `LumioNativeCore_SourceAudit_2026-09-06.md`（基准 `7b3ef0d`，14 条 F01–F14）。审计之后 NativeCore 进了三个提交：`9d7dd40` PR #9（按审计修 F01–F13，+7.8k / -2.3k 行，自述「开发期破坏式改动」）、`c3bf777`（modules/ 文档并入 crates/，**新增 `lumio-hfsm`**）、`c5a8905` PR #10（修 1.88 clippy / fmt 红灯）。本文以 origin/main `c5a8905` 为准。
> 真值分层同 [`2026-09-05-engine-repos-progress-assessment.md`](2026-09-05-engine-repos-progress-assessment.md)：代码以 origin 提交为准，测试以本机实跑输出为准。裁决落 [ADR-069](../decisions/ADR-069-nativecore-audit-rulings.md)。

## 1. 一句话结论

审计的 14 条事实全部成立、没有误报；PR #9 把前 9 条真修了（本机 181 测试全绿）。问题出在两头：审计的**建议**有三处与我们 09-06 定的规则冲突（不堆检测工具、不钉号、第一性原理），PR #9 照单执行了；PR #9 / c3bf777 又顺手带进五类**新实体**（Python 检查器、三系统 CI、三仓三个编译器号、无消费者的 job / spatial 扩建、3,900 行零消费者的 `lumio-hfsm`）。另外 PR #9 的 API 改动把架构仓 SDK 编红了。

## 2. 证据（本机 macOS，2026-09-06 实跑）

| 项 | 命令 | 结果 |
| --- | --- | --- |
| NativeCore 状态 | `git rev-list --left-right --count main...origin/main` | `c5a8905`，0 / 0；GitHub CI `success`（`c3bf777` 与 `9d7dd40` 两次为 `failure`：1.88 的 clippy `uninlined_format_args` + fmt，PR #10 已修） |
| NativeCore 测试 | `cargo test --workspace --all-features --no-fail-fast`（rust 1.88.0，仓内钉号） | **181 passed / 0 failed / 0 ignored**；`cargo fmt --all --check` exit 0 |
| SDK 消费者 | 架构仓 `engine/native`：`cargo build -p lumio-engine-native` | **红，7 错**：`sdk-native/src/timer.rs:197` 调 `register_dispatch`，PR #9 把它藏进 `test-support` feature（默认替代是 `try_register_dispatch`）；其余 6 错是 `voxel.rs` 引用 VoxelEngine 已删的 `Generated*` / `BASELINE_ID` / `SCHEMA_EPOCH` / `from_generated`——**已有卡 R-00476**，且另一会话已在 `fix/sdk-native-timer-api-nativecore-9` 分支修 timer 那一处 |
| hfsm 消费者 | `grep -rli hfsm` 八仓 `*.md/*.rs/*.cs/*.json/*.toml` | 除 NativeCore 自身外 **0 命中**；架构仓设计文档零提及；hfsm README 声称的「架构仓 C ABI 插头与 `.hfsm.json` 工具」不存在 |
| 真实消费 | `sdk-native/Cargo.toml` | 只路径依赖 `lumio-kernel`（仅 `TypeId::of::<HandleKey>()` 编译期标记）与 `lumio-timer`（14 个 `timer_*` 槽，调 18 个方法）；job / spatial / hfsm / codec / diagnostics 零消费者 |
| 工具链 | 三仓 `rust-toolchain.toml` | NativeCore `1.88.0`、架构仓 `engine/native` `1.89.0`、VoxelEngine `1.98.0` |

## 3. 审计 14 条逐条复核

| # | 审计事实 | 现状（`c5a8905`） | 与设计 / 原则 |
| --- | --- | --- | --- |
| F01 Job 不执行计算 | 属实 | 已修：`TypedKernel::execute` 默认返回 `CapabilityUnavailable`、worker 真调它并 `catch_unwind`；输入 / 输出走 Context 字节预算；`worker_count` 生效（0 = 手动泵）；`JobSystem` 注册为 `ContextResource`，`quiesce` 报 Pending、`destroy` join 线程；`take_result` 回收。`CompletionBatch` 仍独立未接入 | 无消费者扩建 +360 行；`CompletionBatch` / `BoundedJobQueue` / `CancellationSource` / `JobStateMachine` 四件导出但 src 内无人用，`TimedOut` 变体不可达（D3：登记不删） |
| F02 Pending 当 Quiesced | 属实 | 已修：`close` 改为可续推进，Pending 不 destroy 并返回 `Quiescing`；超时 `TimedOut` 保留资源可重试；`destroy` 只对已 Quiesced 的资源。回归 `audit_close.rs` | 符合 |
| F03 准入 / 登记 TOCTOU | 属实 | 已修：`admit_work` 持 `admission` 锁跨准入与登记；`close` 先拿同一把锁再翻 gate、拍快照。回归 `resource_losing_close_cannot_register_after_snapshot` | 符合 |
| F04 Timer retire 漏世代 | 属实 | 已修：内部 `retire` 走 `get_timer` 全身份校验；`retire_index` 每次 bump 世代。组合回归 `stale_queued_error_cannot_retire_reused_timer_slot` | 符合。残留：被拒后退休的重复定时器，已入队的早期触发仍会作为「幽灵投递」被 drain 出来（对新定时器无害，无测试钉住） |
| F05 JobHandle 无命名空间 | 属实 | 已修：`JobHandle { id, system, context }`，先查 context 再查 system；ID 进程全局递增不复用，不加世代 | 符合第一性原理（比加世代少一个概念） |
| F06 Cancel 假终态 | 属实 | 已修：`cas_cancel` 循环重试；`JobSystem::cancel` 在调度锁下判定，Running 取消返回 `Requested` | 符合 |
| F07 Timer 补发无界 | 属实 | 已修：`advance` 先按除法算总触发数，超 `max_firings_per_advance`（默认 16384）整体拒绝且刻度不动；`min_interval` 为 0 挡在配置层；逐触发扫描改 O(1) 计数。回归 `excessive_catchup_is_rejected_before_commit_or_queue_changes` | **新静态反例**（未实测）：单个 tick 到期总数超预算（17 个 scope × 每 scope 1024 个）时，`advance(tick+1)` 永远失败，窗口已不可再拆 → 宿主时钟卡死。SDK `timer_advance` 只把它映射成状态码，Server 主循环没处理。落 N-W2 |
| F08 历史表无界 | 属实 | 已修：`take_result` 移除记录；`jobs.len() >= max_live` 背压；`release` 真删并保留有界窗口。回归 `completion_history_remains_bounded`（1000 轮 ≤ 2） | 符合 |
| F09 Spatial 绕过校验 | 属实 | 已修：字段仍 `pub`，但 `upsert` / `query_aabb` / 批量入口全部 `validate()`；NaN 回归 `literal_nan_and_inverted_boxes_are_rejected_by_insert_and_query` | 符合（边界校验代替私有字段，更简） |
| F10 假 rstar | 属实 | 已接真 `rstar =0.12.2`（默认 feature）、`SpatialContext::with_backend` 可注入、oracle 与 R-tree 以不同插入序对拍、批量查询先算容量再 staging | 与 [`ds-server.md`](../knowledge/features/ds-server.md) M5「空间网格 + 双半径 + 候选进 / 出对**有序清单**」形状不同，接第一个消费者时要改形；无消费者先引外部依赖 = 提前投资（D3：留着不动） |
| F11 业务策略入内核 | 属实 | 已修：Bot / Server / 重连常量、`test.slot` 映射、`force_*_generation` 全部移到 `test-support` feature；`SliceTraceEvent` 只剩通用 `Dispatched` | 修法把 SDK 打红（`register_dispatch` 也被门住）；`DispatchTarget` 枚举只为测试签名存在，SDK 永远传 `Registered` |
| F12 文档漂移 | 属实（`git show 7b3ef0d:.spec/AGENTS.md` 第 12 / 13 行自相矛盾） | 四行已改；codec / diagnostics 真有 `prototype` feature | **新漂移**：hfsm 全 crate 引「ADR 0010」实为 0011；`hfsm-semantics.md` 链到不存在的 `0010-hfsm-…`；ADR 0011 提到已不存在的 `xtask allowed_deps` / `modules/hfsm`；`lumio-timer/README.md` 16 / 35 行仍写 Bot / Server 职责；根目录 `.wf-report-R-00352*.md` 两个残留 |
| F13 CI 门禁窄 | 事实属实 | PR #9 照单加：`tools/check_repository.py` + 9 个 unittest（xtask 缩成 34 行启动器）、三系统矩阵、`rust-toolchain` 钉 1.88.0、供应商精确锁 | **与 [ADR-068](../decisions/ADR-068-development-verification-follow-main.md) 冲突**（不堆检测工具、不钉号、只留一套验证规范）；三仓三个编译器号 |
| F14 免审按行数 | 属实 | NativeCore `.spec/AGENTS.md` 已加「Native 高风险改动」一段：所有权 / 线性化 / 世代 / 取消 / 回收 / 预算 / 验证器改动不按行数免审 | 够用；架构仓 AGENTS.md 不另抄一份 |

## 4. 对审计文档本身的评价

- **事实层可信**：14 条逐条核对没有误报；它自己也标清了「源码确认 / 静态反例 / 建议」三种证据等级。
- **建议层三处与我们的规则冲突**（审计不知道 ADR-068 与第一性原理）：F13 的多平台矩阵 / 钉工具链 / 「真实消费方固定提交组合」；§8「最低放行条件」里同一句；F10「先接真后端再对拍」。
- **F01 的「二选一」被实现者选了贵的那条**：审计允许「标成原型」或「做完最小真实链」，PR #9 选了后者，在零消费者的 job 上扩了 worker 线程与结果账本。按第一性原理便宜的选项是标原型等 M5；Owner 本次裁决（D3）改为「开放让上游用」，不回退。

## 5. Owner 三条裁决（一次一问）

| # | 问题 | 裁决 | 后果 |
| --- | --- | --- | --- |
| D1 | `lumio-hfsm` 去留 | **必备基础框架；今后所有涉及状态机的框架务必用这一套**（现在未接入，后边会接） | 进预测世界的状态机（GAS 八态机）也必须走它 → `lumio-hfsm` + `lumio-kernel` **必须能编成 wasm32**（与 D3 / ADR-067 一致，作为 kill criterion）；架构仓落 ADR-069、`native-core.md`、`rules/system.md`；接入契约卡待 Owner 挑第一个消费者后开 |
| D2 | PR #9 带进的门与钉号 | **按建议清**：NativeCore 未到 Release（可用面约 15%，公开 API 仍随第一个消费者改）——Python 并回 Rust xtask、CI 单系统、依赖锁交给 Cargo.lock、三仓编译器统一一个号由架构仓定 | 编译器号 = **1.98.0**；落 N-W1 |
| D3 | job / spatial 死导出与无消费者扩建 | **不删；改为「开放」**：写能力目录让 Voxel / Runtime / Game 等上游知道 NativeCore 有什么、优先用它 | 死导出登记为 known gap；能力目录每项能力只给一个推荐入口 |

## 6. 下一步

- 架构仓（本批）：ADR-069、[`native-core.md`](../knowledge/features/native-core.md)、`rules/system.md`「底层库」两条、`gas.md` / `development-verification.md` 各一句、[W1 卡面草稿](../plans/2026-09-06-nativecore-w1-cards-and-kickoff.md)。
- NativeCore（待 Owner 授权建卡）：N-W1 门禁回收 + 工具链 1.98.0；N-W2 F07 活性 + 文档漂移 + 残留文件。
- 架构仓 A-W1（`register_dispatch → try_register_dispatch` + `engine/native` 工具链 1.98.0）：另一会话的 `fix/sdk-native-timer-api-nativecore-9` 分支已改前者，合入时顺带后者。
- 不开：hfsm 接入契约卡（等第一个消费者：候选 Runtime 连接生命周期 / GAS 八态机）；job / spatial 清理卡（D3）；F14 条款同步（NativeCore 自己那段够用）。
