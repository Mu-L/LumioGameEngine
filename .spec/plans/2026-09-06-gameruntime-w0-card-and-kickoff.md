---
name: 2026-09-06-gameruntime-w0-card-and-kickoff
description: LumioGameRuntime 补单——W0 清理卡 R-00475 的三层卡面（含待建的 8 条验收项）、开工提示词，以及 RT-1 = R-00462 的解锁与派活口径；派 Runtime 清理或 Tick 统一活时查
metadata:
  type: doc
  status: 设计中
---

# LumioGameRuntime · W0 清理卡卡面、开工提示词与 RT-1 解锁口径

> 来源：[`reviews/2026-09-05-engine-repos-progress-assessment.md`](../reviews/2026-09-05-engine-repos-progress-assessment.md) §2.3（前一会话事实 + 第二会话独立复核）与 §6 D12（已裁决：三层全清）、D16 ~ D18（建议，待裁决）。卡面按 workflow-ops `card-spec`（背景 / 目标 / 验收 / 边界）。**R-00475 已建**（`01a0725b-198c-7cbb-a1f6-1f49bd26c3a5`，RM-00005，P1，2026-09-05 16:16Z），线上正文目前只有五条范围、**0 条验收项**，且已被流转到「实现中」；本文 §一 是按 D12 三层重写后的卡面，**线上正文与验收项待 Owner 授权后回写**。与 NativeCore / VoxelEngine 两站的 [`2026-09-05-nativecore-w0-card-and-kickoff.md`](2026-09-05-nativecore-w0-card-and-kickoff.md)、[`2026-09-05-voxelengine-w0-card-and-kickoff.md`](2026-09-05-voxelengine-w0-card-and-kickoff.md) 同一格式。

## 一、卡面（RM-00005 · 既有卡 R-00475 · 待按此重写）

- **标题**：`[程序·工程] 退出旧合同制残留：删除 docs/architecture 镜像与 CI 基线校验、下线 V1.4 GeneratedContracts、活代码改用本仓自有类型`
- **链接**：`https://lumiogamesengine.workflow.games/requirements/01a0725b-198c-7cbb-a1f6-1f49bd26c3a5`

### 背景

架构仓已按 ADR-059 转入 Living Architecture：唯一 ABI 真值是 `engine/abi/native-abi.json`，公共语义各落一份 `engine/wire/<name>-v1.json`，Baseline / 生成物 bundle / `tools/lumio_contract.py` 已删除。Runtime 仓（origin `89a7a6c`）仍绑在旧制度上，机器计数：`LGE-V1` 31 个已跟踪文件、`LumioGameEngineArchitecture` 29、`CoreEngine` 7、`Root ABI` 3；`docs/architecture/` 6 版镜像 + `.baseline.sha256`；CI `readme` job grep 基线字符串并 `sha256sum -c`，`supply-chain` job checkout 旧仓跑 `eng/verify-generated-contracts.sh`；`src/Lumio.GameRuntime.GeneratedContracts/Generated/` 1,920 行 V1.4 生成物（manifest 钉旧仓 `a206e2c`）；**53 个活源文件**用 `Lumio.Gen.*` 类型（replication 13 / command 10 / coordination 9 / simulation 9 / config 5 / observability 3 / ecs 2 / gas 2），其中 `EntityIdentity` 21 处、`ProcessorDescriptorPhase` 12 处——`modules/simulation/.../Phases/TickPhase.cs` 的 13 相枚举本身是它的投影，而 `tick.md` 说相名以这个枚举为准；`README.md`、`modules/README.md`、11 个模块 README、`.spec/AGENTS.md`、`.spec/knowledge/standards/repository-architecture.md` 全写「架构基线 LGE-V1.4-2026-08-27 / 唯一架构源 LumioGameEngineArchitecture / lumio_contract.py validate」；`modules/gas/README.md` 另有「候选接口 activate / apply_effect / tick_effects」「PredictionKey / PredictionFrame」9 处旧词。`dotnet format --verify-no-changes` 在 coordination（`SessionRevisionVectorView.cs` imports 顺序）与 replication（`EntityBindingQuery.cs:130` 空白）各 1 处不过。

### 目标

同 NativeCore（D1，R-00473）与 VoxelEngine（D8，R-00474）口径：三层全清、不留兼容。做完后 Runtime 只认架构仓 `engine/wire` / `engine/abi` 与 `.spec/knowledge/features/*.md` 为公共语义来源，仓内没有第二份「契约真值」，`TickPhase` 等公共枚举是本仓自己的类型。

### 验收（8 条，全部机器可判；待建为原生验收项）

1. `docs/architecture/` 目录与 `.baseline.sha256` 不存在；`git grep -l "LGE-V1\|LumioGameEngineArchitecture\|Root ABI\|lumio_contract.py"` 在已跟踪文件中零命中（`.spec/decisions/000x` 历史 ADR 允许保留原文，但要追加「被本次退出取代」段）。
2. `.github/workflows/repository-policy.yml` 不再 grep 基线字符串、不再 `sha256sum -c`、不再 checkout 旧架构仓、不再调用 `eng/verify-generated-contracts.*`；`eng/verify-generated-contracts.*` 与 `eng/generate-contracts.*` 删除；`.spec/decisions/0002-*.md` 追加「被取代」段。
3. `src/Lumio.GameRuntime.GeneratedContracts/` 与 `tests/Lumio.GameRuntime.GeneratedContracts.Tests/` 整树删除；所有 csproj 不再 ProjectReference 它。
4. `git grep -l "Lumio\.Gen\."` 在 `modules/**/src` 零命中：`TickPhase` 改为 `modules/simulation` 自有枚举（13 个相名与顺序逐字不变，`PhaseContractTable` 测试不倒退）；`EntityIdentity` / `EntityIdentityLifecycle` / `EntityIdentityNamespace` 改用 ecs 自有 `NetEntityId` 与生命周期类型；`TxnJournalRecord*` 枚举改为 coordination 自有类型；`Catalog` 引用删除。类型改名不得改变任何 wire 字节或快照字节（用既有 golden / 两轮哈希测试证明）。
5. `README.md`、`modules/README.md`、11 个模块 README、`.spec/AGENTS.md`「项目是什么」段与「收口门槛」、`.spec/knowledge/standards/repository-architecture.md` 改指 Living Architecture（架构仓 `.spec/knowledge/features/architecture.md` + `engine/wire` / `engine/abi`）；`modules/gas/README.md` 按 `gas.md` 现行口径重写（删「候选接口」「Baseline」「PredictionKey / PredictionFrame」）；`modules/simulation/README.md` 提交点改为 `GasAndEventFinalize`。
6. `docs/workflow/game-runtime-requirements.md`（8-28 的 Workflow 快照）删除——真值在 Workflow，不留副本。
7. `dotnet format --verify-no-changes` 对全部 12 个生产项目 exit 0；本仓 `.spec/knowledge/standards/testing.md` 或 AGENTS.md 收口门槛加上这一条。
8. `dotnet test -c Release` 全部测试项目通过且总数 ≥ 662 减去随 GeneratedContracts 删除的 4 项；`node .spec/tools/spec-lint.mjs && node --test .spec/tools/spec-lint.test.mjs` exit 0；CI Repository Policy 在 PR 上绿。

### 边界

只动本仓；不改 `engine/wire` / `engine/abi`；不做 RT-1 ~ RT-5（Tick 统一、GAS、预测世界）；不改任何 wire / 快照字节；不加「兼容旧名字」的别名；不新建卡。

## 二、开工提示词（另开窗口，工作目录 `~/LumioGames/LumioGameRuntime`）

```text
你是 LumioGameRuntime 的工程清理工程师。任务：Workflow lumiogamesengine 的 R-00475（RM-00005）。

【守门】
1. LumioGameRuntime origin/main = 89a7a6c 或其后继，本地 main 与 origin 同步、工作区干净；不满足就停。
2. 架构仓 ~/LumioGames/LumioGameEngine origin/main 上不存在 packages/、tools/lumio_contract.py、engine/native/generated/architecture/
   以外的 Baseline 目录（那是本次要退出的制度）；存在就停下回报。
3. 用 workflow-execute 读全 R-00475：正文 + 验收项 + 评论；验收项必须是 8 条且与本文 §一 一致；若线上仍是 0 条或正文只有五条范围，
   说明卡面还没按 D12 三层重写，停下回报，不得按窄口径开工。

【指路】
- 卡面正文就是任务书：三层按验收 1 → 8 的顺序做。第 1 层删镜像 / CI 断言 / 旧仓名与 README 口径；第 2 层删 GeneratedContracts 整树与
  两个 eng 脚本；第 3 层把 53 个活源文件里的 Lumio.Gen.* 改成本仓自有类型——先做 TickPhase（simulation），再 EntityIdentity（replication /
  ecs），最后 TxnJournalRecord（coordination）；每层一个提交。
- 改名不改字节：动到 codec / 快照的地方先跑既有 golden 与两轮哈希测试，拿到红再改。
- 本仓 .spec/decisions/ 新增一条 ADR 记录这次退出（编号现查最高号），0002 只加「被 NNNN 取代」，不改写。
- 公共语义拿不准（例如某个生成枚举在架构仓 engine/wire 里有没有对应物）→ 停，卡上标 BLOCKED 上报，不本地造第二份。

【立规】
- 领卡先经 Workflow 流转「实现中」并写 reason（卡现在已在实现中，补一条开工评论钉 origin SHA 即可）；改动在 feat/r-00475-exit-legacy-contract
  分支，先 push 再回写证据。
- 每次提交前：dotnet build、dotnet test -c Release（全部测试项目）、dotnet format --verify-no-changes、node .spec/tools/spec-lint.mjs、
  node --test .spec/tools/spec-lint.test.mjs 全部 exit 0；测试证据必须是本机实跑的命令与输出；被删测试逐条列清单。
- 交付 = 改动清单 + 验证证据（命令 + 关键输出）+ known gaps + 沉淀落点（本仓新 ADR），写成 PR 描述并同步为 R-00475 的证据评论，
  评论只引用已推送 origin 的提交号；做完流转「验收中」，「已完成」由总调度核验后流转。走 PR，不直接推 main。
- 遇到 bug 或测试失败先找根因再改；同一问题修三次不成，停下上报。

【禁区】
- 不改 engine/wire、engine/abi、任何 wire / 快照字节；不做 Tick 统一、GAS、预测世界、体素端口公开（那是 R-00462 / 468 / 466 / 469）；
  不加别名 / 兼容层 / 「先保留」开关；不动其他仓；不建卡；密钥不入库不进日志。
```

> **执行记录（2026-09-06，Owner 授权后逐笔读回）**：R-00475 卡面已按本节 §一 重写并建 8 条验收项；同时发现其交付提交 `dc37644`（分支 `Go1c/r5-a2-runtime-query-expiry`）**不在 origin**，卡上已留评论写明解铃条件（push + 开 PR + 补 origin 提交号）。派工前先确认这一条已解除，否则会与已有的本地交付冲突。

## 三、RT-1 = R-00462 的解锁与派活口径（Owner 2026-09-06 裁决 D16：B，已执行）

- **现状**：R-00462 正文是 8-30 口径的 E2 卡，「执行前置 = R-00460 A0 契约冻结」；A0 backlog、0 评论。RT-1 的正文（Tick 统一 + `[System(Phase)]` 注册 + `TickRate` + 文档）在 [`2026-09-05-bomber-engine-runtime-cards.md`](2026-09-05-bomber-engine-runtime-cards.md) `card:RT-1`，与 R-00462 的对齐关系写在同文件「落单口径」段。
- **解锁动作（已执行）**：① R-00462 已回写 RT-1 完整正文（现状证据 / 文件集 / 6 条要求 / 验证计划 / 接口 / 4 条验收 / 串行口径）；② R-00460 已补评论，A0 只约束碰 `engine/wire` 的四张卡（R-00464 / 465 / 466 / 467），E1 / E2 / E3 / E6 / E7 不以 A0 为前置；③ R-00462 已流转「评审中」。
- **GAS 拆卡（D19，已执行）**：R-00468 收窄为 RT-4（GAS M2）；新建 **R-00480** = RT-5（Effect 单 / 整数求值 / 两本账相尾重算 / 击杀跨零 / `OnFx`），前置 R-00468，与 R-00466（RT-3）并行。R-00466 已回写 RT-3 正文并更正前置。
- **派活**：提示词直接用 bomber 计划 `card:RT-1` 正文，守门第一步改为「origin/main = `50da4bb` 或其后继」（R-00475 已于 2026-09-06 03:36Z 合入，PR #34）。两卡文件集重叠、必须串行的约束已解除。
  - **派 RT-1 前必看**：R-00475 把 13 相枚举从 `modules/simulation/src/Lumio.GameRuntime.Simulation/Phases/TickPhase.cs` 移到了 **`src/Lumio.GameRuntime.Primitives/TickPhase.cs`**（`command` 硬依赖它、`simulation → command` 会成环；理由见 Runtime 仓 ADR 0003）。原文件更名 `Phases/PhaseContract.cs`，只留相契约类型。`tick.md` 已同步。**RT-1 的文件集按新路径写，否则实现方按旧路径找不到。**
  - 另两处新事实：耐久记录信封（`TxnJournalRecord` / `CommandLogRecord` / `WalRecordEnvelope`）现在在 `modules/observability/Records/`；类型归属的单一权威表是 Runtime 仓 `modules/README.md` §6.2。
- **顺带改卡**：RT-2 正文「坐标必须整数、建议毫格」已被 ADR-065 D04（Float32）与 `11d34e1` 实现取代；派 R-00463 前把该段改为「Float32 米制，按 ADR-065 D04 / movement.md M1」。
