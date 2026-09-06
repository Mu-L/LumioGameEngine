---
name: 2026-09-06-nativecore-w1-cards-and-kickoff
description: NativeCore 补单草稿——W1 两张卡（门禁回收 + 工具链 1.98.0；F07 活性 + 文档漂移）卡面与开工提示词；派 NativeCore 活时查
metadata:
  type: doc
  status: 设计中
---

# NativeCore · W1 两张卡卡面与开工提示词

> 来源：[`reviews/2026-09-06-nativecore-source-audit-reassessment.md`](../reviews/2026-09-06-nativecore-source-audit-reassessment.md) 与 [ADR-069](../decisions/ADR-069-nativecore-audit-rulings.md) 第 3 / 4 条。卡面按 workflow-ops `card-spec`（背景 / 目标 / 验收 / 边界）。**已落单 Workflow RM-00002**：[R-00504](https://lumiogamesengine.workflow.games/requirements/01a07725-5953-7f7c-b745-5e231c991a79) 与 [R-00505](https://lumiogamesengine.workflow.games/requirements/01a07725-5e38-7d9c-a3fe-b23c7b593599)。两卡文件集不重叠（N-W1 只碰 CI / xtask / tools / toolchain；N-W2 只碰 timer 源码与测试、hfsm 注释与文档、根目录残留），并行派两个 worktree。

## 一、N-W1 门禁回收 + 工具链统一（[R-00504](https://lumiogamesengine.workflow.games/requirements/01a07725-5953-7f7c-b745-5e231c991a79)）

**标题**：`[程序·协议/公共][NativeCore·W1] 回收 PR #9 的 Python 检查器与三系统 CI，xtask 收回检查，工具链统一 1.98.0`

### 背景

外部审计 F13 建议加多平台矩阵、钉工具链、消费方固定提交组合；PR #9 照单做了：`tools/check_repository.py`（101 行）+ `tools/test_check_repository.py`（9 个 unittest），`xtask` 缩成 34 行的 Python 启动器，CI 矩阵 ubuntu / windows / macos，`rust-toolchain.toml` 钉 1.88.0，供应商精确锁表。Owner 2026-09-06 裁决（ADR-069 第 3 / 4 条）：NativeCore 未到 Release，不堆检测工具；Rust 仓不引第二种语言；工具链号只有一个来源（架构仓 `engine/native/rust-toolchain.toml` = 1.98.0）。

### 目标

仓内检查只剩 Rust 一种语言、CI 只跑一个系统、编译器号与架构仓一致；`cargo xtask check-dep-dag` 与 `assert-no-native-artifacts` 仍然存在且仍读真实 workspace（不回到手拆 TOML 的老路）。

### 验收

1. `tools/check_repository.py`、`tools/test_check_repository.py` 删除；`tools/` 目录若空则删。
2. `xtask/src/main.rs` 自己解析 `cargo metadata --format-version 1 --no-deps --all-features` 的 JSON 实现两个子命令：`check-dep-dag`（从真实 `workspace_members` 枚举、内部依赖白名单、未登记成员报错、环检测，含 optional / target / build 依赖，跳过 dev）与 `assert-no-native-artifacts`（任一 target 的 `crate_types` 含 `cdylib` / `staticlib` 即失败）。允许 `xtask` 使用工具级依赖（如 `serde_json`），它不进任何产物；不再维护供应商精确锁表（外部依赖版本以 `Cargo.lock` 为准）。
3. F13 的两个反例作为 Rust 测试保留在 `xtask`：新增未登记成员被报出；`crate-type = ['cdylib']`（单引号）被报出（用真实临时 workspace 跑 `cargo metadata`，不写第二个 TOML 解析器）。
4. `.github/workflows/repository-policy.yml`：`native` job 只跑 `ubuntu-latest`，删 `setup-python` 与 `python -m unittest` 步骤；保留 build / test（default 与 all-features）/ clippy / fmt / xtask 两项 / `cargo test -p lumio-spatial --no-default-features`。
5. `rust-toolchain.toml` 改为 `1.98.0`（`profile = "minimal"`，`components = ["rustfmt", "clippy"]`）；CI 的 `dtolnay/rust-toolchain` 同号。在 1.98.0 下 `cargo fmt --all --check`、`cargo clippy --workspace --all-targets --all-features -- -D warnings`、`cargo test --workspace` 与 `--all-features` 全绿并附输出。
6. 根 `README.md`「收口门槛」与 `.spec/AGENTS.md` 的命令清单同步（去 Python、去矩阵）；`docs/reviews/2026-09-06-native-remediation.md` 不改写（历史记录），在 NativeCore `.spec/decisions/` 新增一条 ADR 记录本次回收并引架构仓 ADR-069。
7. 本仓 `node .spec/tools/spec-lint.mjs` OK；CI 绿。

### 边界

- 不动任何 crate 的源码与公开 API；不动 `lumio-timer`（N-W2 的文件集）。
- 不恢复任何已删的 Baseline / 镜像 / 合同守卫。
- 不改架构仓文件；架构仓 `engine/native/rust-toolchain.toml` 抬到 1.98.0 归架构仓卡 A-W1。

## 二、N-W2 F07 活性缺口 + 文档漂移 + 残留文件（[R-00505](https://lumiogamesengine.workflow.games/requirements/01a07725-5e38-7d9c-a3fe-b23c7b593599)）

**标题**：`[程序·协议/公共][NativeCore·W1] 修 timer 单 tick 超预算永久拒绝；hfsm / timer 文档改引正确 ADR；删根目录残留报告`

### 背景

PR #9 修 F07 的办法是：`advance(to_tick)` 先按除法算出本次会产生多少次触发，超过 `TimerBudget::max_firings_per_advance`（默认 16384）就整体返回 `ScheduleBudgetExceeded`、刻度不动、让调用方拆小窗口重试。但窗口最小是 1 个 tick，拆不下去：若某一 tick 到期的定时器总数超预算（每 scope 上限 1024 个，17 个 scope 即可），`advance(tick + 1)` 永远失败，宿主时钟卡死。架构仓 SDK 的 `timer_advance` 只把错误映射成状态码，Server 主循环没有处理。另外 c3bf777 引入的 hfsm 全 crate 引「ADR 0010」（实为 0011），`hfsm-semantics.md` 链到不存在的 `0010-hfsm-…`，ADR 0011 提到已不存在的 `xtask allowed_deps` 与 `modules/hfsm`；`crates/lumio-timer/README.md` 第 16 / 35 行仍写 Bot / Server 职责（代码已移到 `test-support`）；根目录 `.wf-report-R-00352.md` / `.wf-report-R-00352-fix.md` 是 8 月分支的残留状态报告。

### 目标

`advance` 在任何合法配置下都能推进至少一个 tick；文档不再指向错误或不存在的决策；根目录只剩仓的正式文件。

### 验收

1. 先写失败测试：一个 manager、`max_firings_per_advance` 设小（如 4），建 5 个同一 tick 到期的一次性定时器，`advance(due)` **必须成功**推进到 `due`，所有 5 条触发要么入队要么按既有 `SlotQueueFull` 规则逐条拒绝并记入 `AdvanceReport::rejections`，`committed_tick() == due`；测试在修复前失败、修复后通过，输出附交回物。
2. 实现二选一并在 `docs/specs` 的 timer 契约里写明：(a) 预算改为「每次 `advance` 至少推进一个 tick，预算只限制跨 tick 的补发枚举」；或 (b) `TimerBudget::validate` 强制 `max_firings_per_advance ≥ max_scopes × max_active_timers_per_scope`（同 tick 最坏情况），使超预算在配置期就被拒绝。既有 `excessive_catchup_is_rejected_before_commit_or_queue_changes` 保持通过。
3. `grep -rn "ADR 0010" crates/lumio-hfsm docs/specs/hfsm-semantics.md` 零命中，全部改为 ADR 0011；`hfsm-semantics.md` 链接指向 `0011-hfsm-stateless-evaluator.md`；ADR 0011 里的 `xtask allowed_deps` 改为当前检查位置、`modules/hfsm` 改为 `crates/lumio-hfsm`（ADR 正文允许修正指针，不改决策）；hfsm README「C ABI 插头与 `.hfsm.json` 工具在架构仓」改为「待架构仓开接入契约卡（ADR-069）」。
4. `crates/lumio-timer/README.md` 第 16 / 35 行改为：Bot / Server / 重连策略属消费者，仓内只在 `test-support` feature 下保留夹具。
5. 删除根目录 `.wf-report-R-00352.md`、`.wf-report-R-00352-fix.md`。
6. 门全绿并附输出：`cargo fmt --all --check`、`cargo clippy --workspace --all-targets --all-features -- -D warnings`、`cargo test --workspace --all-features`、`node .spec/tools/spec-lint.mjs`；架构仓 `cargo build -p lumio-engine-native`（在 A-W1 与 R-00476 合入后）不因本卡新增错误。

### 边界

- 只改 `lumio-timer` 的 `advance` 预算路径与对应测试 / 契约文档；不动 F04 / F11 的修法，不动公开类型签名（`TimerBudget` 字段可加校验不可改名）。
- 不动 CI / xtask / toolchain（N-W1 的文件集）。
- 不接 hfsm ABI，不改 hfsm 逻辑；hfsm 只改 `.rs` 文档注释与 README / spec / ADR 0011 里的 ADR 指针。

## 三、开工提示词（已填卡号与链接）

### N-W1（R-00504）
```text
你是 LumioNativeCore 仓的 Native 内核工程师。这一轮只做一张卡：Workflow（lumiogamesengine）R-00504
「[程序·协议/公共][NativeCore·W1] 回收 PR #9 的 Python 检查器与三系统 CI，xtask 收回检查，工具链统一 1.98.0」
https://lumiogamesengine.workflow.games/requirements/01a07725-5953-7f7c-b745-5e231c991a79

【守门（第一步，任一不符立即停下回报 BLOCKED，不得继续）】
1. 本仓 origin/main 是 c5a8905 或其后继；用 git worktree 在独立目录开分支，主工作区不动。
2. 用 workflow-execute 读全这张卡：正文 + 全部验收项 + 评论；读不到就停。
3. 架构仓 ~/LumioGames/LumioGameEngine 的 .spec/decisions/ADR-069-nativecore-audit-rulings.md 与
   .spec/knowledge/features/native-core.md 存在；本卡的裁决依据在那里，读一遍再动手。

【指路】
- 卡面正文就是任务书，按验收项顺序做；每条验收项都要有命令 + 输出作证据，不接受「已通过」三个字。
- 先加载再动手：before-you-code；测试先行：test-driven-development。
- 不夹带：只做卡面里的事。N-W1 不碰 crate 源码。两卡并行，文件集不得越界。
- 改到本仓公开 Rust API 时必须在架构仓 engine/native 复跑 cargo build/test -p lumio-engine-native；
  编不过且原因不在本卡 → 标 BLOCKED 上报，不得改架构仓文件。

【交回物（全仓单一权威）】
① 改动清单；② 验证证据（命令与关键输出）；③ known gaps；④ 知识沉淀落点（本仓 .spec 与 ADR，或声明无需沉淀）。
提交命令与 PR 由用户在终端敲，你只准备好分支与交回物。
```

### N-W2（R-00505）
```text
你是 LumioNativeCore 仓的 Native 内核工程师。这一轮只做一张卡：Workflow（lumiogamesengine）R-00505
「[程序·协议/公共][NativeCore·W1] 修 timer 单 tick 超预算永久拒绝；hfsm / timer 文档改引正确 ADR；删根目录残留报告」
https://lumiogamesengine.workflow.games/requirements/01a07725-5e38-7d9c-a3fe-b23c7b593599

【守门（第一步，任一不符立即停下回报 BLOCKED，不得继续）】
1. 本仓 origin/main 是 c5a8905 或其后继；用 git worktree 在独立目录开分支，主工作区不动。
2. 用 workflow-execute 读全这张卡：正文 + 全部验收项 + 评论；读不到就停。
3. 架构仓 ~/LumioGames/LumioGameEngine 的 .spec/decisions/ADR-069-nativecore-audit-rulings.md 与
   .spec/knowledge/features/native-core.md 存在；本卡的裁决依据在那里，读一遍再动手。

【指路】
- 卡面正文就是任务书，按验收项顺序做；每条验收项都要有命令 + 输出作证据，不接受「已通过」三个字。
- 先加载再动手：before-you-code；测试先行：test-driven-development（N-W2 的验收 1 必须先红后绿）。
- 不夹带：只做卡面里的事。N-W2 不碰 CI / xtask / toolchain。两卡并行，文件集不得越界。
- 改到本仓公开 Rust API 时必须在架构仓 engine/native 复跑 cargo build/test -p lumio-engine-native；
  编不过且原因不在本卡 → 标 BLOCKED 上报，不得改架构仓文件。

【交回物（全仓单一权威）】
① 改动清单；② 验证证据（命令与关键输出）；③ known gaps；④ 知识沉淀落点（本仓 .spec 与 ADR，或声明无需沉淀）。
提交命令与 PR 由用户在终端敲，你只准备好分支与交回物。
```
