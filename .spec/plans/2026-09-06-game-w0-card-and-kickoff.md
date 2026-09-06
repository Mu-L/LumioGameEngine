---
name: 2026-09-06-game-w0-card-and-kickoff
description: LumioGame 补单——W0 清理卡 R-00482 按 D33 重写的卡面、炸弹人 G-0 v2 新卡、G-1 ~ G-7 分批口径、评论与开工提示词；派 Game 清理与炸弹人契约活时查
metadata:
  type: doc
  status: 设计中
---

# LumioGame · W0 清理卡卡面、G-0 v2 卡面、G-1 ~ G-7 分批口径与开工提示词

> 来源：[`reviews/2026-09-05-engine-repos-progress-assessment.md`](../reviews/2026-09-05-engine-repos-progress-assessment.md) §2.6 与 §6 **D32（Owner 2026-09-06 追认：RM-00008 四张 GAS 旧卡作废、R-00482 已建）**、**D33（Owner 2026-09-06 裁决 A：先重冻 G-0 v2，再按引擎接缝分两批派；R-00482 补验收项并扩到三层）**。卡面按 workflow-ops `card-spec`（背景 / 目标 / 验收 / 边界）；验收项类型「质量验收」、初始「未开始」，与 R-00474 / R-00478 / R-00481 同一套 id。与前五站的 [`2026-09-05-nativecore-w0-card-and-kickoff.md`](2026-09-05-nativecore-w0-card-and-kickoff.md)、[`2026-09-05-voxelengine-w0-card-and-kickoff.md`](2026-09-05-voxelengine-w0-card-and-kickoff.md)、[`2026-09-06-gameruntime-w0-card-and-kickoff.md`](2026-09-06-gameruntime-w0-card-and-kickoff.md)、[`2026-09-06-server-w0-card-and-kickoff.md`](2026-09-06-server-w0-card-and-kickoff.md) 同一格式。**本文各节正文即 Workflow 写入的唯一来源**（HTML 注释标记供脚本取用）；写入执行记录见文末 §六。

## 一、卡面（RM-00008 · 既有卡 R-00482 · 按 D33 重写）

- **链接**：`https://lumiogamesengine.workflow.games/requirements/01a0741e-6651-7e2e-8e1c-577bbf5fbd42`
- **优先级 / 风险**：P0 / medium（沿用）。
- **标题**：<!-- card:R-00482 title -->[程序·工程] 退出旧合同制残留：合入 R-00408 的 Game 修复恢复编译并给 CI 加构建测试作业、删 docs/architecture 镜像与基线校验、README 与 .spec 改 Living Architecture 口径、清远端分支<!-- /title -->

<!-- card:R-00482 body -->
## 背景

架构仓已按 ADR-059 转入 Living Architecture：唯一 ABI 真值是 `engine/abi/native-abi.json`，公共语义各落一份 `engine/wire/<name>-v1.json`，Baseline / `tools/lumio_contract.py` / 生成源仓 `LumioGameEngineArchitecture` 全部不存在；炸弹人切片的引擎验收真值在架构仓 `.spec/knowledge/features/bomber-slice.md`。LumioGame（origin `5bc5afc`，2026-09-06 实测）现状：

1. **main 编不过、CI 看不见**：同级 `LumioGameRuntime` 在 `c2d42b3`（2026-09-04）删了 `WorldManager.TryGetSession / BindSelf / UnbindSession / GrantClaim / HasClaim`，`modules/server-gameplay/src/Lumio.Game.ServerGameplay/Chat/ChatSetMessageSystem.cs:62` 仍调用，`dotnet test LumioGame.sln` 报 `CS1061` 两处（net10.0 / netstandard2.1）；`.github/workflows/repository-policy.yml` 只有一个查 README 的作业，没有任何 build / test 作业，所以 GitHub Actions 四次全绿。修复已在 origin：`fix/r5-entity-chat-scenarios`（`e243a39`，删 `InputCommandEnvelope.cs` 与其测试、新增 `RuntimeDrainConsumer` 与两组边界测试），对 Runtime `89a7a6c` 快照实跑 `dotnet test` 25 / 25、node 21 / 21、spec-lint OK；它是 R-00408（RM-00011）的 Game 半边，R-00408 最终评论点名的 Game 提交 `d3f6fe0` 尚未推送。
2. **旧合同制残留**（六站里最小）：`git grep -l LGE-V1` 18 个已跟踪文件；`docs/architecture/` 8 份正文 + `ADR_INDEX.md` + `5.5Max-ReviewV3/` + `.baseline.sha256` 共 331 KB；CI 断言 v1.4 正文、`LGE-V1.4-2026-08-27` 字样与 `sha256sum -c`；`README.md`「架构基线 / 唯一架构源 `LumioGameEngineArchitecture` / `python3 tools/lumio_contract.py validate` / Architecture Gate / Generated Contract Dependencies / ReleaseCatalog」、`.spec/AGENTS.md`（LGE-V1.4、设计落 `docs/specs/`、计划落 `docs/plans/`、收口门槛含 `lumio_contract.py`）、`.spec/knowledge/standards/repository-architecture.md`、`docs/specs/engineering/module-scaffolding-design.md`（10 个子模块蓝图，实际建了 3 个）、`docs/specs/engineering/mvp-placevoxel-content-spec.md`（V1.4 时代挖 / 放方块切片，已被炸弹人切片取代，ADR-063 第 13 条）、`modules/server-gameplay/README.md`（架构基线 LGE-V1.4、契约消费 `lumio.gameplay-envelope.v1`——信封已随 R-00408 归 Runtime `WireCodec`）；仓根两份已跟踪的 `.wf-report-R-00354.md` / `.wf-report-live11.md`。
3. **卫生**：远端 6 条已合入分支（`docs/bomber-v04-stage0-adr`、`feat/bomber-stage0-kernel-contract`、`feat/r-00348-chat-component`、`feat/r-00373-chatcomponent-runtime`、`feat/r-00376-eleven-scenarios`、`claude/confident-dhawan-567439`〔PR #1 squash 合入前的 tip，内容已在 `39f88c9`〕）与同名本地分支 1 条；PR #12（`Go1c-patch-1`，只改 `design.md` §1 一句措辞，`mergeable: CONFLICTING`）。

RM-00008 的四张 V1.4 时代 GAS 卡（R-00310 ~ R-00313）已按 D32 作废，这些文件已没有任何卡在引用。

## 目标

同 NativeCore（D1，R-00473）、VoxelEngine（D8，R-00474）、Runtime（D12，R-00475）、Server（D21，R-00478）、Client（D25，R-00481）口径：**全清、不留兼容**。做完后仓里只剩「消费 Runtime 公开 API 的玩法工程」这一种形状：`modules/server-gameplay`（含炸弹人契约壳，本卡不动）+ `config` / `scenario` 骨架 + `integration/`，main 编译绿且 CI 真跑构建与测试，公共语义只从架构仓 `engine/` 与 `knowledge/features` 取，没有第二份契约真值、没有指向已退役仓的引用。**本卡不碰 `Bomber/Contracts/**`、不写一行炸弹人规则代码**——契约对齐归 G-0 v2（D33），规则归 G-1 ~ G-7。

## 验收（8 条，全部机器可判；已建为原生验收项）

1. **编译恢复**：R-00408 实现方推送的 Game 提交（或其同源 `origin/fix/r5-entity-chat-scenarios` `e243a39`，二者以推送后 `git merge-base` 核对为准）已合入 main；对同级 Runtime `origin/main`（≥ `89a7a6c`）`dotnet build LumioGame.sln` 0 错误、`dotnet test LumioGame.sln` 全过且测试数 ≥ 25（含 `Bomber.RuntimeIntegrationProbeTests` 3 条）；`git grep -n -e TryGetSession -e GrantClaim -e InputCommandEnvelope -- modules` 0 命中。
2. **旧镜像与旧切片文档**：`docs/architecture/` 整目录不存在（含 8 份正文、`ADR_INDEX.md`、`5.5Max-ReviewV3/`、`.baseline.sha256`）；仓根 `.wf-report-R-00354.md`、`.wf-report-live11.md` 不存在；`docs/specs/engineering/mvp-placevoxel-content-spec.md` 删除并从 `docs/specs/README.md` 导航移除。
3. **CI**：`.github/workflows/repository-policy.yml` 删 `test -s docs/architecture/...`、两处 `grep 'LGE-V1.4-2026-08-27'`、`grep '^# LumioGameEngine V3 (v1.4)'`、`test -s docs/architecture/.baseline.sha256`、`sha256sum -c`；README 小节断言只断言重写后仍存在的小节；**新增一个 build-test 作业**：`actions/setup-dotnet`（按 `global.json` 10.0.100）+ checkout 同级 `LumioGames/LumioGameRuntime`（`ref: main`）到 `../LumioGameRuntime` 或经 `LUMIO_RUNTIME_ROOT` 指向 + `dotnet build LumioGame.sln` + `dotnet test LumioGame.sln` + `cd integration/entity-chat && node --test verify-evidence.mjs bot-credential.mjs web/chat-window.test.mjs`；PR 与 main 上该作业必过。
4. **旧字样归零**：`git grep -l -i -e 'LGE-V1' -e 'LumioGameEngineArchitecture' -e 'lumio_contract.py' -e 'Architecture Gate' -e 'BaselineId' -e 'ReleaseCatalog'` 在已跟踪文件中只命中 `.spec/decisions/0001 ~ 0019` 历史 ADR 正文（不改写，只在被取代的 ADR 追加「被 00xx 取代」段）。
5. **README 重写**：删「架构基线 / Architecture Gate / Replication Mapping / Generated Contract Dependencies / Runtime Loading Relationships / Release Composition / Room Modes / Version / Manifest」等 V1.4 段与「唯一架构源 `LumioGameEngineArchitecture`」句；契约来源改为架构仓 `LumioGameEngine` 的 `engine/wire/*.json`、`engine/abi/native-abi.json` 与 `.spec/knowledge/features/{architecture,bomber-slice,tick,ecs,gas,movement,voxel}.md`；「子模块」表只列实际存在的 `server-gameplay / config / scenario` 与 `integration/`，未建的 7 个子模块删行；不复述任何公共契约字段；「当前阶段」改为炸弹人 Stage 0（引用 `docs/specs/bomber/design.md` §16）。
6. **`.spec` 与工程文档改口**：`.spec/AGENTS.md`「项目是什么」删 `LGE-V1.4` 基线句与「只读镜像」句；设计落点改 `.spec/knowledge/features/`（策划案与美术规范仍落 `docs/specs/`，ADR 0002 不变）、计划落 `.spec/plans/`；「收口门槛」改为 `node .spec/tools/spec-lint.mjs && node --test .spec/tools/spec-lint.test.mjs && dotnet build LumioGame.sln && dotnet test LumioGame.sln && (cd integration/entity-chat && node --test verify-evidence.mjs bot-credential.mjs web/chat-window.test.mjs)`，删 `python3 tools/lumio_contract.py validate` 与「复现 repository-policy.yml」句。`.spec/knowledge/standards/repository-architecture.md` 按 Living Architecture 重写（唯一事实源 = 架构仓 `engine/` + `knowledge/features`；「跨 World 只经 Runtime Coordinator」改为「地形经引擎体素批量读写、玩法系统经 13 相第 3 / 4 相注册、移动 / 放弹为 GAS Ability」）；`docs/specs/engineering/module-scaffolding-design.md` 与 `modules/server-gameplay/README.md` 删 `LGE-V1.4` 基线行与「契约消费 `lumio.gameplay-envelope.v1`」行；`.spec/knowledge/README.md` 导航同步；`node .spec/tools/spec-lint.mjs && node --test .spec/tools/spec-lint.test.mjs` exit 0。
7. **决策记录**：本仓 `.spec/decisions/0020-exit-legacy-contract-regime.md` 新增（编号落笔时现查最高号），记录本次退出、来源 D32 / D33 与被删清单；`decisions/README.md` 索引同步；`0001 ~ 0019` 不改写。
8. **卫生与不夹带**：远端 `docs/bomber-v04-stage0-adr`、`feat/bomber-stage0-kernel-contract`、`feat/r-00348-chat-component`、`feat/r-00373-chatcomponent-runtime`、`feat/r-00376-eleven-scenarios`、`claude/confident-dhawan-567439` 六条（逐条 `git rev-list --count origin/main..<b>` = 0 或 squash 内容已在 main，删前列清单）与本地 `claude/confident-dhawan-567439` 删除；`fix/r5-entity-chat-scenarios` 在验收 1 合入后删除；PR #12 关闭（那一句措辞由 Owner 决定是否手工重提）；做完 `git branch -r` 只剩 `origin/main`（与 `origin/HEAD`）；`git diff --stat origin/main -- modules/server-gameplay/src/Lumio.Game.ServerGameplay/Bomber` 为空（契约壳一行不动，归 G-0 v2）；不新增任何兼容别名、`#if` 开关或「先保留」注释。

## 边界

只动本仓；不碰 `Bomber/Contracts/**`（G-0 v2）；不写炸弹人规则代码（G-1 ~ G-7）；不改上游 Runtime / Server / 架构仓；不改 `engine/wire`；不删 `docs/specs/bomber/**`（G-0 v2 会修订）；不建新卡。**两条 known gaps 记在本卡交回物、不在本卡做**：① `EntityChat.Protocol/AccountServerProcess.cs` 拉起同级 LumioServer 的 `lumio-account-server.dll`，D22 删该目录时随 R-00408 改为自签测试凭据或 Platform 账号服；② Game 生产程序集引用 Runtime **样例**工程 `Samples.Username.Server`（ChatComponent 真源），RT-2 样板落地后改指正式组件。
<!-- /body -->

<!-- card:R-00482 acc -->
- 编译恢复：R-00408 的 Game 提交（或同源 `origin/fix/r5-entity-chat-scenarios` `e243a39`）已合入 main；对 Runtime `origin/main`（≥ `89a7a6c`）`dotnet build LumioGame.sln` 0 错误、`dotnet test LumioGame.sln` 全过且测试数 ≥ 25（含 Bomber 探针 3 条）；`git grep -n -e TryGetSession -e GrantClaim -e InputCommandEnvelope -- modules` 0 命中。
- `docs/architecture/` 整目录（8 份正文、`ADR_INDEX.md`、`5.5Max-ReviewV3/`、`.baseline.sha256`）不存在；仓根 `.wf-report-R-00354.md`、`.wf-report-live11.md` 不存在；`docs/specs/engineering/mvp-placevoxel-content-spec.md` 删除并从 `docs/specs/README.md` 导航移除。
- CI `repository-policy.yml` 删 v1.4 正文 / `LGE-V1.4-2026-08-27` / `sha256sum -c` 断言；新增 build-test 作业（setup-dotnet 10.0.100 + checkout 同级 LumioGameRuntime main + `dotnet build` + `dotnet test LumioGame.sln` + entity-chat 三个 node 测试文件），PR 与 main 上必过。
- `git grep -l -i -e 'LGE-V1' -e 'LumioGameEngineArchitecture' -e 'lumio_contract.py' -e 'Architecture Gate' -e 'BaselineId' -e 'ReleaseCatalog'` 在已跟踪文件中只命中 `.spec/decisions/0001 ~ 0019` 历史 ADR 正文（不改写）。
- README 重写：删全部 V1.4 段与「唯一架构源 LumioGameEngineArchitecture」句；契约来源改架构仓 `engine/wire` + `engine/abi` + `knowledge/features`；子模块表只列实际存在的 3 个模块与 `integration/`；不复述公共契约字段；当前阶段改为炸弹人 Stage 0。
- `.spec/AGENTS.md` 删基线与镜像句、设计落点改 `.spec/knowledge/features/`、收口门槛改为 spec-lint + dotnet build/test + entity-chat node 测试；`repository-architecture.md` 按 Living Architecture 重写；`module-scaffolding-design.md` 与 `server-gameplay/README.md` 删 LGE-V1.4 与 gameplay-envelope 行；`knowledge/README.md` 同步；spec-lint 与其测试 exit 0。
- 新增 `.spec/decisions/0020-exit-legacy-contract-regime.md`（编号现查），记录退出、来源 D32 / D33 与被删清单；`decisions/README.md` 同步；0001 ~ 0019 不改写。
- 远端 6 条已合入分支与本地 `claude/confident-dhawan-567439` 删除（删前逐条列 `rev-list --count` 证据）、`fix/r5-entity-chat-scenarios` 合入后删除、PR #12 关闭；`git branch -r` 只剩 `origin/main`；`git diff --stat origin/main -- …/Bomber` 为空；不新增兼容别名 / `#if` 开关。
<!-- /acc -->

## 二、卡面（RM-00013 · 新建 · G-0 v2 = **R-00483**，`01a07435-2dd6-784c-8df6-ce8d2c1a15d3`）

- **优先级 / 风险**：P0 / high；category `game / server-gameplay`，module `bomber Stage 0 slice`（与 R-00423 ~ R-00431 同惯例）。
- **前置**：R-00482 验收 1（main 编译恢复）；文档部分可先行。引用：R-00423（v1，被取代）、R-00461（已交付）、R-00462 / R-00468 / R-00469 / R-00480（接口按其卡面 Produces）。
- **标题**：<!-- card:G0V2 title -->[程序·协议/公共][S0-G0v2] 炸弹人 Stage 0 内核契约 v2：对齐架构第二样板——位置归 LogicTransform、属性走 AttributeComponent 两本账、移动 / 放弹为 Ability、爆炸 / 死亡为 13 相系统、地形走引擎体素读写<!-- /title -->

<!-- card:G0V2 body -->
## 背景

G-0（R-00423）于 2026-09-04 冻结的契约 v1.3.0（`docs/specs/bomber/stage0-kernel-contract.md`，`Bomber/Contracts/**` sha256 `d16de07a…`）是照着当天 Runtime 的三个缺口反向设计的（ADR 0015 §0 核验 ②③⑤ 不可行）：没有系统注册面 → 规则内核做成普通函数由 Scenario 宿主在 `WorldManager.Tick()` 前后手调；没有体素端口 → 地形走 Game 自有 `ITerrainStore` + `InMemoryChunkStore`（ADR 0016 / 0019）；没有 GAS → 位置是 `BomberPlayerState` 自带的 `CellX/Y/Z` + `PosMilliX/Y/Z`，血量 / 火力 / 移速 / 炸弹数是单账 `Sync<int>`，移动 / 放弹是 `MoveIntent` / `PlaceBombIntent` DTO。

2026-09-05 架构仓把这三个缺口全部立成了 Runtime 卡，并写下了以后所有战斗类 ECS / GAS 代码的标准（`.spec/knowledge/features/bomber-slice.md` §2 世界模型套用、§4 第二样板；ADR-063 第 13 / 14 条；ADR-064；`rules/system.md` 世界模型红线「静态必须是体素、GAS 只能是实体上的组件、预测一律经 GAS」；`movement.md` §9.1 已写好 Game 位置迁移五步；`tick.md` §4 系统注册）：

| 环节 | v1.3.0 | 架构仓定的 | 引擎依赖 |
| --- | --- | --- | --- |
| 位置 | `CellX/Y/Z` + `PosMilliX/Y/Z` 六个 `Sync<int>` | 唯一真值 `LogicTransform`，格子由逻辑位置推导 | R-00461（已在 Runtime main `11d34e1`） |
| 属性 | `HealthPoints / BombPower / BombCapacity / SpeedTier` 单账 | `AttributeComponent` 一处声明、生成基础账 + 当前账 | R-00468（RT-4） |
| 移动 / 放弹 | DTO + Game 函数 | `AbilityType`，共享文件两端跑，准入五步，档位「逻辑预测」 | R-00468（RT-4） |
| 爆炸 / 死亡 / 帽子 / 掉落 / 拾取 | Scenario 宿主在 `Tick()` 前后手调 | `[System(Phase.ProcessorPlan)]` 注册进第 4 相，`WorldManager.Tick()` 唯一路径 | R-00462（RT-1，评审中） |
| 伤害 | `DamageApplied` 事件 + 临时命中集合 | 瞬时 Effect 单改基础账、击杀 = 跨零、`OnFx` | R-00480（RT-5） |
| 地形 | `ITerrainStore` / `InMemoryChunkStore` | 体素（官方全局段 8 Solid + 水），帧初批量读、帧末一批写、pin | R-00469（E7） |

炸弹实体持火焰（四臂长度）、帽堆实体、重生三处两边一致，不动。按 v1.3.0 派 G-1 ~ G-7，就是在游戏仓造第二条 Tick 路径、第二份地形真值、第二份位置真值，RT-1 / E7 落地即整套推倒——Owner 2026-09-06 裁决 D33：先重冻 v2，再分批派。

## 目标

契约 v2.0.0：`Bomber/Contracts/**` 里**今天就能编译**的部分按第二样板重写（实体挂 `LogicTransform`、删位置字段与单账属性字段、删 DTO 与 `ITerrainStore`），依赖引擎接缝的部分（技能 / 系统 / 属性两本账 / 体素读写）在文档里**定形到签名级**（TypeId、输入结构、准入判定、相归属、读写集、顺序），代码随批 B 的 G-1 / G-3 / G-6 在 R-00462 / R-00468 / R-00469 / R-00480 合入后落。做完后 Game 契约与架构仓第二样板逐环节一致，没有任何「Stage 2 再换实现」的替身。

## 验收（7 条；已建为原生验收项）

1. `docs/specs/bomber/stage0-kernel-contract.md` 升 v2.0.0：§0 增「架构第二样板对照」六行（位置 / 属性 / 技能 / 系统 / 伤害 / 地形），每行写「v1.3.0 怎么写、v2 怎么写、引擎依赖卡号」；文档头版本与 sha256 更新；全文无「待定」「TBD」。
2. 位置：`BomberPlayerState` 删 `CellX/Y/Z`、`PosMilliX/Y/Z`；`BomberBombState` / `BomberHatPile` / `BomberPickupItem` 删 `CellX/Y/Z`；四个 EntityType 各 `[Has(typeof(LogicTransform))]`；所在格由逻辑位置按 `movement.md` §9.1 第 3 条推导（floor，函数归 Game，本卡只定签名）；`gen-declarations` 重新生成 8 个生成文件；`git grep -n -e PosMilli -e CellX -e CellY -e CellZ -- modules` 0 命中。
3. 属性：删 `HealthPoints / BombPower / BombCapacity / SpeedTier` 四个 `Sync<int>`（`HatCount / RespawnAtTick / ProtectedUntilTick` 非属性、保留）；契约文档 §1 写明 `玩家属性 : AttributeComponent` 的四条声明（`血量 6 / 火力 2 / 移速 3500 / 手上炸弹数 1`，单位与 `design.md` 一致）与两本账 Scope（基础账 `Scope.Owner` + `[Persist]`，当前账 `Scope.Aoi`），照 `bomber-slice.md` §4 ③；代码随批 B 在 R-00468 合入后落，本卡不建占位文件、不加 `#if`。
4. 技能：删 `Bomber/Contracts/Commands/` 整目录（`MoveIntent` / `PlaceBombIntent`）；契约文档 §2 改为 `移动技能` / `放弹技能` 两个 `AbilityType` 的定形（`TypeId`、`输入` 结构、`可以激活吗` = 准入第 ⑤ 步判定、`执行` 语义、档位「逻辑预测」、放弹消耗 = `手上炸弹数`），照 `bomber-slice.md` §4 ①②；`design.md` §6.1 手感规则（转角缓冲）落为技能的普通字段；`git grep -n -e MoveIntent -e PlaceBombIntent -- modules docs` 0 命中（`docs/specs/bomber/**` 同步改口）。
5. 系统与伤害：契约文档 §2 新增「系统清单」——`爆炸系统` / `死亡系统` / `帽子系统` / `掉落系统` / `拾取系统` 各标相（全部 `Phase.ProcessorPlan`；`ApplyInputs` 相只放技能）、`[Reads]` / `[Writes]` 组件集、`[After]` 顺序（爆炸 → 掉落 / 帽子 → 拾取；死亡系统读到基础账 ≤ 0 才下结构单，晚一帧）；删 §0「规则内核由 Scenario 宿主在 `Tick()` 前后手调」与 §6 的宿主驱动循环描述，改为 `WorldManager.Tick()` 唯一路径 + 生成注册表（R-00462 Produces）；伤害改为瞬时 `EffectType` 定形（`点数`、改基础账、击杀 = 跨零由引擎判），`DamageApplied` / `PlayerDied` 保留为遥测 DTO 但产生点改为 Effect 单结算的 `OnFx` 记录（R-00480 Produces）；同弹命中记忆改为炸弹实体的普通字段（不上网、不存档）。
6. 地形：删 `ITerrainStore` / `InMemoryChunkStore` / `GetCell` / `GetColumn` / `GetBox` 整节与 §6 StateHash 的「确定性 box 读」半边；改为引用架构仓 `voxel.md` M6 ①c / M7a / M8 ③a（帧初批量读整图、帧末一批写、pin 常驻）与 R-00469 Produces；保留 ADR 0019 的坐标映射（游戏 (X,Y,Z) → 引擎 (x=X, z=Y, y=Z+1)）与九种方块目录行；`scenario.json` 地形数据改为按 Section 的 BlockId 数组（可直接喂 `blockWrite`）；StateHash = Runtime 快照哈希 + 各 Section revision。
7. 探针、矩阵与决策：`RuntimeIntegrationProbeTests` 更新为四种实体各挂 `LogicTransform` 入快照、两世界逐字节同哈希；`dotnet test LumioGame.sln` 全过（前置 R-00482 验收 1）；`stage0-test-matrix.md` 的传播 / 连锁 / 血量 / 回放行改引用新口径；本仓 ADR `0021-bomber-contract-v2-align-engine-second-exemplar.md`（编号现查）记录取代 ADR 0015（编排与地形两条）、0016（`ITerrainStore` 条）、0019（接口形状条，坐标映射保留），三份旧 ADR 只追加「被 0021 取代」段；G-1 ~ G-7 的 Workflow 卡面由主 loop 按 D33 回写，本卡不动 Workflow。

## 边界

不写规则代码（G-1 ~ G-7）；不动 Runtime；不改 `engine/wire`；不动 `design.md` 的玩法数值与帽子经济；不删 `docs/specs/bomber/**`；不加兼容开关、不留「Stage 2 再换」的替身实现；不碰 R-00482 的文件集（README / `.spec` / CI / `docs/architecture`）。
<!-- /body -->

<!-- card:G0V2 acc -->
- `stage0-kernel-contract.md` 升 v2.0.0：§0 增六行「架构第二样板对照」（位置 / 属性 / 技能 / 系统 / 伤害 / 地形），各写 v1.3.0 口径、v2 口径、引擎依赖卡号；文档头版本与 sha256 更新；全文无「待定」。
- 位置：四个组件删 `CellX/Y/Z` 与 `PosMilliX/Y/Z`；四个 EntityType 各 `[Has(typeof(LogicTransform))]`；所在格推导签名按 `movement.md` §9.1；生成文件重生成；`git grep -e PosMilli -e CellX -e CellY -e CellZ -- modules` 0 命中。
- 属性：删四个单账 `Sync<int>`；契约 §1 写明 `AttributeComponent` 四条声明与两本账 Scope（照 bomber-slice §4 ③）；不建占位文件、不加 `#if`。
- 技能：删 `Commands/` 目录；契约 §2 定形 `移动技能` / `放弹技能` 两个 AbilityType（TypeId、输入、准入 ⑤ 判定、执行、档位、消耗）；`git grep -e MoveIntent -e PlaceBombIntent -- modules docs` 0 命中。
- 系统与伤害：契约 §2 系统清单（五个系统的相、读写集、`[After]` 顺序），删宿主手调口径，改 `WorldManager.Tick()` 唯一路径；伤害改瞬时 EffectType 定形，事件 DTO 产生点改 `OnFx`；同弹命中记忆改炸弹实体普通字段。
- 地形：删 `ITerrainStore` / `InMemoryChunkStore` 整节与 box 读 StateHash；改引用 `voxel.md` M6/M7/M8 与 R-00469；保留 ADR 0019 坐标映射与九种方块；`scenario.json` 地形改按 Section 的 BlockId 数组；StateHash = 快照哈希 + Section revision。
- 探针更新（四实体挂 LogicTransform、两世界同哈希）；`dotnet test LumioGame.sln` 全过；`stage0-test-matrix.md` 同步；新增 ADR 0021（编号现查）取代 0015 / 0016 / 0019 对应条款，旧 ADR 只追加取代段。
<!-- /acc -->

## 三、G-1 ~ G-7 / C-1 分批回写口径（评论回写，不新建重复单；卡面正文由批次派活时按 v2 更新）

D33 的分批规则：**接缝无关的先派**（交付 = 纯函数 + 单元测试，不建 Scenario 宿主、不建 Tick 循环、不碰地形存储；系统壳随 R-00462 合入后一行接上），**碰引擎接缝的等引擎卡合入**。

| 卡 | 批 | 前置 | 回写要点 |
| --- | --- | --- | --- |
| R-00424 G-1 规则内核 | **B** | G-0 v2 + R-00462 + R-00468 + R-00480 | 移动 / 放弹改为两个 Ability、爆炸 / 死亡改为第 4 相系统、伤害改 Effect 单、地形改引擎批量读写；验收项 4「向 Voxel Port 提交 ≤ 24 格一笔事务」改为「帧末一批 `blockWrite`、同帧同格只一条」 |
| R-00425 G-2 帽子经济 | **A** | G-0 v2 | `HatCount` 与 `HatPile` 实体不变；铸帽 / 散落 / 拾取竞争 / 超时 / 守恒做成纯函数 + 单测；帽子系统壳随 R-00462 |
| R-00426 G-3 掉落与糖果 | **B** | G-0 v2 + R-00468 | 三糖果改基础账（`火力 / 手上炸弹数 / 移速`）的瞬时 Effect，等两本账 |
| R-00427 G-4 地图生成器 | **A** | G-0 v2 | 删「实现 ITerrainStore」半边，标题改「实现 19×19 灰盒地图生成器」；只做生成器 + §5.3 断言 + 双密度指标 + 固定 Seed 样本 + `PickSpawn`；输出为按 Section 的 BlockId 数组（可直接喂 `blockWrite`） |
| R-00428 G-5 Config 表 | **A** | G-0 v2 | 键按 v2 §5，不受影响的直接做 |
| R-00429 G-6 Scenario 宿主 | **B** | G-0 v2 + R-00462 + R-00469 | 宿主 = `WorldManager.Tick()` 唯一路径 + 生成注册表；命令流 = `Activate<T>` 输入序列；StateHash = 快照哈希 + Section revision |
| R-00430 G-7 遥测 | **A** | G-0 v2 | Schema / Sink / 报告工具照做；`DamageApplied` / `PlayerDied` 产生点改 `OnFx`，Schema 不变 |
| R-00431 C-1 网络面契约 | 等 v2 | G-0 v2 | Component Schema 随 v2 变；Living Architecture 下「ADR → Schema → Fixture → Baseline → 镜像」链已不存在，落点改架构仓 `engine/wire/bomber-*.json`（ADR-059 口径） |

<!-- comment:G-COMMON -->
## 2026-09-06 引擎总监盘点 · D33（Owner 裁决 A）分批回写

本卡**暂不按现卡面派工**。前置改为 **G-0 v2**（R-00483）冻结：G-0 v1.3.0 是照 Runtime 09-04 三个缺口反向设计的（位置字段 / 单账属性 / DTO 命令 / 宿主手调 / `ITerrainStore`），与架构仓 09-05 定的第二样板（`bomber-slice.md` §4：`LogicTransform` / `AttributeComponent` 两本账 / Ability / `[System(Phase.ProcessorPlan)]` / 体素批量读写）五处结构性不同，按现卡派会在游戏仓造第二套引擎。裁决与依据见架构仓 `.spec/reviews/2026-09-05-engine-repos-progress-assessment.md` §2.6 / §6 D33；分批口径见 `.spec/plans/2026-09-06-game-w0-card-and-kickoff.md` §三。

<!-- /comment -->

<!-- comment:R-00424 -->
**本卡 = 批 B**。前置：G-0 v2 + R-00462（RT-1 系统注册）+ R-00468（RT-4 Ability / 准入 / 属性展开）+ R-00480（RT-5 Effect 单 / 两本账）合入。派工时正文按 v2 改口：移动 / 放弹改为两个 `AbilityType`（共享文件、两端跑、准入五步）、爆炸 / 死亡改为第 4 相 `[System(Phase.ProcessorPlan)]`、伤害改瞬时 Effect 单（击杀 = 跨零由引擎判）、地形改引擎帧初批量读 / 帧末一批写；验收项 4「向 Voxel Port 提交的摧毁格数 ≤ 24 且为一笔事务」改为「帧末一批 `blockWrite`、同帧同格只一条」；其余四条验收项不变。
<!-- /comment -->

<!-- comment:R-00425 -->
**本卡 = 批 A**：G-0 v2 冻结即可派。`HatCount` 留在 `BomberPlayerState`、`HatPile` 实体与字段不变；铸帽 / 散落 / 拾取竞争 / 超时回收 / 退出回流 / 帽王判定 / 守恒断言做成纯函数 + 单元测试交付；**不建 Scenario 宿主、不建 Tick 循环**，帽子系统壳（`[System(Phase.ProcessorPlan)]` 注册进第 4 相）在 R-00462 合入后一行接上。四条验收项不变。
<!-- /comment -->

<!-- comment:R-00426 -->
**本卡 = 批 B**。前置：G-0 v2 + R-00468。三糖果（火力 / 炸弹 / 速度）改为对基础账（`火力 / 手上炸弹数 / 移速`）的瞬时 Effect 单，上限判定在准入里；软砖 30% 掉落表本身是纯函数，可随批 A 的 G-4 先出，但拾取生效等两本账。
<!-- /comment -->

<!-- comment:R-00427 -->
**本卡 = 批 A**：G-0 v2 冻结即可派，但范围收窄——删「实现 `ITerrainStore`」半边（v2 已删该抽象，地形走引擎体素读写，R-00469），标题改为「实现 19×19 灰盒地图生成器：分层地形、硬砖阵列、软砖 65%、双密度指标、连通与掩体断言、固定 Seed 样本与出生点查询」；只做生成器 + `design.md` §5.3 断言 + 双密度指标 + 5 个固定 Seed 快照 + `PickSpawn`；输出为按 Section 的 BlockId 数组（坐标映射照 ADR 0019，可直接喂 `blockWrite`）。四条验收项不变。
<!-- /comment -->

<!-- comment:R-00428 -->
**本卡 = 批 A**：G-0 v2 冻结即可派。Config 键按 v2 §5（`ITerrainStore` 相关键若有则删），其余 Schema / 默认值 / A/B 变体 / IntegerOnly 校验照做。四条验收项不变。
<!-- /comment -->

<!-- comment:R-00429 -->
**本卡 = 批 B**。前置：G-0 v2 + R-00462（`WorldManager.Tick()` 唯一路径 + 生成注册表）+ R-00469（体素批量读写）。宿主不再在 `Tick()` 前后手调规则函数；命令流 = `Activate<T>` 输入序列；StateHash = Runtime 快照哈希 + 各 Section revision（v2 §6）；Bot 行为与回放 oracle 的四条验收项不变。
<!-- /comment -->

<!-- comment:R-00430 -->
**本卡 = 批 A**：G-0 v2 冻结即可派。事件 Schema / Sink / 帽子经济与性能指标聚合 / 报告工具照做；`DamageApplied` / `PlayerDied` 的产生点改为 Effect 单结算的 `OnFx` 记录（R-00480），Schema 字段不变。四条验收项不变。
<!-- /comment -->

<!-- comment:R-00431 -->
**本卡等 G-0 v2**：Component Schema（`attribute-declarations.json`）随 v2 变（位置归 `LogicTransform`、属性归两本账、命令改 Ability），v2 冻结前不登记。另：本卡正文的「ADR → Schema → Fixture → Baseline → 镜像」链是 V1.4 合同制口径，Living Architecture 下不存在——落点改为架构仓 `engine/wire/bomber-*.json`（ADR-059），消费仓不再持镜像；派工时正文按此改。
<!-- /comment -->

## 四、R-00423 / R-00408 / R-00482 评论

<!-- comment:R-00423 -->
## 2026-09-06 引擎总监盘点 · 仓库领先纠偏（D33）

G-0 交付早已在 LumioGame `origin/main`：`5348635`（v1.0.0 冻结 + 探针）→ `10782b6`（v1.1.0，ADR 0016 / 0017）→ `b6f0c38`（v1.2.0，ADR 0018）→ `cdcca47` → `51c7829`（v1.3.0，ADR 0019），2026-09-04 全部合入；`Bomber/Contracts/**` 14 文件 sha256 `d16de07a9d5d7f6d05d1fadfd54b9bb8b0709a4925f4bbb779549a40ff98fc08` 2026-09-06 本机复算一致；09-04 交付时 `dotnet test` 26 / 26（含探针 3 条）。卡却停在 backlog、0 评论、5 条验收项未动——本次按纪律流转到「验收中」。

**QA 注意**：今天 main 因上游 Runtime `c2d42b3` 删 `TryGetSession` 而编译红（与本卡无关），验收项 4「`dotnet build LumioGame.sln` 与收口门槛命令通过」要等 R-00482 验收 1 合入修复后再跑；其余 4 条可直接对 `51c7829` 核。

**取代关系**：v1.3.0 与架构仓 09-05 第二样板五处结构性不同（位置 / 属性 / 技能 / 系统 / 地形），已由 Owner 裁决 D33 新建 **G-0 v2**（R-00483）重冻；本卡验收通过后即「已完成」，不再修订。
<!-- /comment -->

<!-- comment:R-00408 -->
## 2026-09-06 引擎总监盘点 · 差异记录（不改状态）

2026-09-06 00:11Z 的最终评论把本卡转「验收中」并把 4 条验收项判 passed，点名的四个交付提交 **2026-09-06 08:2x 在各仓 origin 任何 ref 上都找不到**（`git fetch --all --prune` 后 `git cat-file -e`）：Runtime `18b5feb`、Server `f031afb`、Client `5745d11`、Game `d3f6fe0`；架构仓 `9a22527` 只在 `origin/r5-closeout-20260906` 分支、不在 main。评论自述「no push, PR, deployment」。按 td-progress-audit 步骤 3：Workflow 领先、origin 无交付，**在推送并重核前不得向「已完成」流转**。

Game 侧 origin 现有 `fix/r5-entity-chat-scenarios`（`e243a39`，2026-09-05）：删 `InputCommandEnvelope.cs`、新增 `RuntimeDrainConsumer`，对 Runtime `origin/main` `89a7a6c` 快照实跑 `dotnet test` 25 / 25、node 21 / 21、spec-lint OK；它与 `d3f6fe0` 是否同源待推送后 `git merge-base` 核对。LumioGame main 今天因该修复未合入而编译红（`CS1061 TryGetSession`）。

**解铃条件**：四仓推送 → 逐仓 `git ls-remote` + `git show --stat` 覆盖各自改动清单 → Game 合入 main 且 CI 新增的 build-test 作业绿（R-00482 验收 1 / 3）→ 再判「已完成」。
<!-- /comment -->

<!-- comment:R-00482 -->
## 2026-09-06 引擎总监盘点 · Owner 追认 D32、卡面按 D33 重写

- **D32 追认**：本卡与 R-00310 ~ R-00313 的作废（2026-09-06 00:28:55 – 00:29:04Z，另一会话执行）Owner 已追认。
- **D33**：卡面按架构仓 `.spec/plans/2026-09-06-game-w0-card-and-kickoff.md` §一重写——范围从「删 `docs/architecture/` + 改 CI 字样」扩到三层（编译恢复 + CI 新增 build-test 作业、旧镜像与旧切片文档、README / `.spec` / 工程文档改口、ADR、远端分支与 PR #12），并建 8 条原生验收项（原正文只有 checkbox、0 条原生验收项）。
- **前置**：验收 1 依赖 R-00408 实现方推送 Game 提交（或以同源 `e243a39` 兜底）；本卡不替 R-00408 做活、不碰 `Bomber/Contracts/**`（归 G-0 v2 R-00483）。
- 开工提示词：同文件 §五。
<!-- /comment -->

## 五、开工提示词

### 5.1 R-00482（另开窗口，工作目录 `~/LumioGames/LumioGame`）

```text
你是 LumioGame 的工程清理工程师。任务：Workflow lumiogamesengine 的 R-00482（RM-00008）。

【守门】
1. LumioGame origin/main = 5bc5afc 或其后继，本地 main 与 origin 同步、工作区干净；同级 ~/LumioGames/LumioGameRuntime 在 origin/main ≥ 89a7a6c。不满足就停。
2. 用 workflow-execute 读全 R-00482：正文 + 8 条验收项 + 评论；验收项必须是 8 条且与架构仓
   .spec/plans/2026-09-06-game-w0-card-and-kickoff.md §一 一致；不是就停下回报。
3. 验收 1 的前置：看 R-00408（RM-00011）最新评论里 Game 提交 d3f6fe0 是否已推送。已推送 → 合它；未推送 → 合 origin/fix/r5-entity-chat-scenarios（e243a39）
   并在 R-00408 上留一条评论说明「Game 侧先以 e243a39 恢复编译，d3f6fe0 推送后请 rebase」。两者都没有 → 停。

【指路】
- 卡面正文就是任务书，按验收 1 → 8 顺序做，四个提交：① 合入修复 + CI 新增 build-test 作业（先让红变绿，再删别的）；② 删 docs/architecture、.wf-report、mvp-placevoxel-content-spec；
  CI 去 v1.4 断言；③ README / .spec/AGENTS.md / repository-architecture.md / module-scaffolding-design.md / server-gameplay/README.md 改口 + ADR 0020；④ 分支与 PR #12 卫生。
- 收口门槛（改口后的）：node .spec/tools/spec-lint.mjs && node --test .spec/tools/spec-lint.test.mjs && dotnet build LumioGame.sln && dotnet test LumioGame.sln
  && (cd integration/entity-chat && node --test verify-evidence.mjs bot-credential.mjs web/chat-window.test.mjs)。
- 环境：macOS 上 dotnet 10.0.100（global.json）；Runtime 经同级目录自动发现，或 LUMIO_RUNTIME_ROOT。

【立规】
① 领卡先流转「实现中」（reason 写明）；② 证据评论只引用已推送 origin 的提交号，先 push 再回写；③ 测试证据必须是命令的真实输出，不写「已通过」；
④ 交付 = 改动清单 + 验证证据 + known gaps（至少两条：AccountServerProcess 拉起 Server account-server；Game 引用 Runtime 样例工程）+ 沉淀落点；
⑤ 公共契约缺口 → 停，卡上标 BLOCKED 上报；⑥ 只动本仓；⑦ 做完流转「验收中」，「已完成」由总调度核验后流转。

【禁区】
不碰 modules/server-gameplay/src/Lumio.Game.ServerGameplay/Bomber/**（G-0 v2 的文件集）；不写炸弹人规则代码；不改 Runtime / Server / 架构仓；不改 engine/wire；
不删 docs/specs/bomber/**；不加兼容别名 / #if 开关 / 「先保留」注释；删分支前逐条列 rev-list --count 证据；PR #12 只关闭，那一句措辞由 Owner 决定。
```

### 5.2 G-0 v2（另开窗口，工作目录 `~/LumioGames/LumioGame`；卡号 R-00483）

```text
你是 LumioGame 的炸弹人契约维护者（Server Gameplay）。任务：Workflow lumiogamesengine 的 R-00483（RM-00013）。

【守门】
1. 先读架构仓 ~/LumioGames/LumioGameEngine 的 .spec/knowledge/features/bomber-slice.md（§2 世界模型套用、§4 第二样板）、movement.md §9.1、tick.md §4、gas.md M2–M5、voxel.md M6–M8，
   以及 .spec/plans/2026-09-05-bomber-engine-runtime-cards.md 里 RT-1 / RT-4 / RT-5 的「接口 · Produces」——v2 里依赖引擎接缝的签名必须与这些卡面一致，不得自拟。
2. 用 workflow-execute 读全 R-00483：正文 + 7 条验收项；与架构仓 .spec/plans/2026-09-06-game-w0-card-and-kickoff.md §二 一致；不是就停。
3. 文档部分（验收 1 / 3 / 4 / 5 / 6 的定形）可以立刻做；代码部分（验收 2 / 7）要 main 编译绿——看 R-00482 验收 1 是否已合入，没有就先交文档提交、代码等它。

【指路】
- 三个提交：① 契约文档 v2.0.0（§0 对照表、§1 属性声明、§2 技能与系统清单、§6 地形与 StateHash、删 ITerrainStore 节）+ stage0-test-matrix 同步 + ADR 0021；
  ② Bomber/Contracts/** 代码：四组件删位置字段、PlayerState 删四个属性字段、删 Commands/、四个 EntityType 挂 LogicTransform、gen-declarations 重生成、探针测试更新；③ sha256 与文档头收口。
- Runtime 的 LogicTransform 已在 origin/main（11d34e1）：modules/ecs/src/Lumio.GameRuntime.Ecs/LogicTransform.cs；gen-declarations 在 LumioGameRuntime/tools/gen-declarations。
- 收口门槛：R-00482 改口后的那条（spec-lint + dotnet build/test + entity-chat node 测试）。

【立规】同 §5.1 ①–⑦。交付的 known gaps 至少写明：技能 / 系统 / 属性两本账 / 体素读写四处的代码随 G-1 / G-3 / G-6 在 R-00462 / R-00468 / R-00469 / R-00480 合入后落。

【禁区】
不写规则代码；不动 Runtime；不改 engine/wire；不动 design.md 玩法数值；不删 docs/specs/bomber/**；不加兼容开关、不留「Stage 2 再换实现」的替身；
不碰 R-00482 的文件集（README / .spec / CI / docs/architecture）；不动 Workflow 上 G-1 ~ G-7 的卡面（主 loop 回写）。
```

### 5.3 一窗串行版（R-00482 → R-00483，Owner 2026-09-06 要求「一个提示词做完」；与 5.1 / 5.2 同口径，冲突以本节为准）

```text
你是 LumioGame 仓的实现工程师，一个窗口串行完成两张 Workflow（lumiogamesengine）卡：先 R-00482（RM-00008，退出旧合同制 + 恢复编译 + CI 加构建测试作业），再 R-00483（RM-00013，炸弹人 Stage 0 内核契约 v2）。
工作目录 ~/LumioGames/LumioGame；同级 ~/LumioGames/LumioGameRuntime、~/LumioGames/LumioGameEngine（架构仓，只读）。卡面正文与验收项的唯一来源是架构仓
.spec/plans/2026-09-06-game-w0-card-and-kickoff.md（§一 = R-00482，§二 = R-00483，§三 = G-1~G-7 分批口径，只读参考）。

【守门（不满足就停，回报，不绕）】
1. git fetch --all --prune；本地 main == origin/main（5bc5afc 或其后继）、工作区干净；LumioGameRuntime 的 origin/main >= 89a7a6c 且本地同步。
2. 用 workflow-execute 读全 R-00482 与 R-00483：正文 + 验收项（8 条 / 7 条）+ 评论，与上述 plans 文件 §一 / §二 一致；不一致就停。
3. 看 R-00408（RM-00011）最新评论：Game 提交 d3f6fe0 已推送 → 合它；未推送 → 合 origin/fix/r5-entity-chat-scenarios（e243a39），并在 R-00408 留一条评论
   「Game 侧先以 e243a39 恢复编译，d3f6fe0 推送后请 rebase」。两者都没有 → 停。

【第一段：R-00482】
- 领卡先流转「实现中」（reason 写明）。分支 feat/r-00482-exit-legacy-contract-regime。
- 按验收 1 → 8 顺序，四个提交：① 合入修复 + CI 新增 build-test 作业（先让红变绿）；② 删 docs/architecture、两份 .wf-report、mvp-placevoxel-content-spec，CI 去 v1.4 断言；
  ③ README / .spec/AGENTS.md / repository-architecture.md / module-scaffolding-design.md / server-gameplay/README.md 改口 + ADR 0020（编号现查）；④ 分支与 PR #12 卫生。
- 收口门槛（改口后的）：node .spec/tools/spec-lint.mjs && node --test .spec/tools/spec-lint.test.mjs && dotnet build LumioGame.sln && dotnet test LumioGame.sln
  && (cd integration/entity-chat && node --test verify-evidence.mjs bot-credential.mjs web/chat-window.test.mjs)。每条验收项贴命令与真实输出。
- push 分支、开 PR、等 CI 新作业绿；证据评论只引用已推送的提交号；流转「验收中」。**不自己合 main**（Owner 合）。删远端分支与关 PR #12 前逐条列
  rev-list --count 证据，在评论里先列清单再执行。known gaps 至少两条：AccountServerProcess 拉起 Server account-server（D22 时改自签凭据 / Platform）；Game 引用 Runtime 样例工程 Samples.Username.Server（RT-2 后改指正式组件）。

【第二段：R-00483（R-00482 的 PR 开出后即可开始文档部分；代码部分等 R-00482 合入 main 或直接基于其分支）】
- 先读架构仓 .spec/knowledge/features/bomber-slice.md §2 / §4、movement.md §9.1、tick.md §4、gas.md M2–M5、voxel.md M6–M8，
  以及 .spec/plans/2026-09-05-bomber-engine-runtime-cards.md 里 RT-1 / RT-4 / RT-5 的「接口 · Produces」——v2 里依赖引擎接缝的签名必须与这些一致，不得自拟。
- 领卡流转「实现中」。分支 feat/r-00483-bomber-contract-v2（基于 R-00482 分支或合入后的 main）。三个提交：① 契约文档 v2.0.0（§0 六行对照表、§1 属性声明、§2 技能与系统清单、
  §6 地形与 StateHash、删 ITerrainStore 节）+ stage0-test-matrix 同步 + ADR 0021（编号现查，取代 0015 / 0016 / 0019 对应条款，旧 ADR 只追加取代段）；
  ② Bomber/Contracts/** 代码：四组件删 CellX/Y/Z 与 PosMilliX/Y/Z、PlayerState 删四个属性字段、删 Commands/ 目录、四个 EntityType 挂 LogicTransform
  （Runtime origin/main 11d34e1，modules/ecs/src/Lumio.GameRuntime.Ecs/LogicTransform.cs）、用 LumioGameRuntime/tools/gen-declarations 重生成 8 个生成文件、探针测试更新；③ sha256 与文档头收口。
- 收口门槛同第一段；验收 7 条逐条贴证据；push、开 PR、流转「验收中」，不自己合 main。known gaps 写明：技能 / 系统 / 属性两本账 / 体素读写四处代码随 G-1 / G-3 / G-6 在
  R-00462 / R-00468 / R-00469 / R-00480 合入后落。

【公共纪律】
① 领卡先流转「实现中」；② 证据评论只引用已推送 origin 的提交号，先 push 再回写；③ 测试证据必须是命令真实输出；④ 交付 = 改动清单 + 验证证据 + known gaps + 沉淀落点；
⑤ 公共契约缺口 → 停，卡上标 BLOCKED 上报；⑥ 只动本仓；⑦ 做完流转「验收中」，「已完成」由总调度核验后流转；⑧ 两张卡各自独立分支、独立 PR、独立交回物。

【禁区】
不写炸弹人规则代码（G-1~G-7）；不改 Runtime / Server / 架构仓、不改 engine/wire；不动 design.md 玩法数值；不删 docs/specs/bomber/**；不加兼容别名、#if 开关、「先保留」注释、
「Stage 2 再换实现」的替身；R-00482 不碰 Bomber/Contracts/**，R-00483 不碰 README / .spec / CI / docs/architecture；不动 Workflow 上 G-1~G-7 的卡面；不合 main、不发布。
交回格式：每张卡一份——改动清单 / 验证证据（命令 + 关键输出）/ known gaps / 沉淀落点（本仓 ADR 0020 / 0021 与 knowledge 导航同步即沉淀）。
```

## 六、Workflow 写入执行记录（2026-09-06，Owner 一次性授权「写入不要每笔问我」后逐笔读回）

<!-- exec-log -->
- 2026-09-06 08:5x（北京时间），操作人 Lumio 账号（`usr_e6bba…`），每笔 GET 读回：
  - **R-00482**：`PATCH` 标题 + 四节正文（200，读回 6,894 字）；`POST` 8 条原生验收项（质量验收 / 未开始，读回 8 / 8 not_started）；追认评论 `01a07436-410b`（D32 追认 + D33 重写）；状态仍 backlog（待派）。
  - **R-00483（G-0 v2，新建）**：`POST /requirements` 200 → `01a07435-2dd6-784c-8df6-ce8d2c1a15d3`，读回 RM-00013 / P0 / high / `game / server-gameplay` / `bomber Stage 0 slice` / backlog；`POST` 7 条原生验收项（读回 7 / 7 not_started）；引用 `PUT` 7 条均 201：R-00423（v1，被取代）、R-00461、R-00462、R-00468、R-00469、R-00480、R-00482。
  - **G-1 ~ G-7 + C-1**（R-00424 ~ R-00431）：各 1 条分批评论（201 × 8，`01a07435-b219` … `01a07435-ee6c`，读回各 1 条）；状态不动（backlog），派工时按 §三 改正文。
  - **R-00423（G-0 v1）**：评论 `01a07436-0365`（仓库领先证据 + QA 注意 + 取代关系）；流转 backlog → in_review → in_progress → acceptance（200 × 3，reason 引 D33），读回 `acceptance`；5 条验收项仍 not_started，待 QA 实跑。
  - **R-00408（RM-00011）**：差异记录评论 `01a07436-39f4`（四仓提交不在 origin、解铃条件），读回评论 6 条；**状态不动**（acceptance）。
  - Room 读回：RM-00013 11 张 {'acceptance': 1, 'backlog': 10}；RM-00008 7 张 {'acceptance': 1, 'backlog': 1, 'done': 1, 'rejected': 4}；RM-00011 42 张 {'acceptance': 1, 'done': 37, 'in_review': 2, 'rejected': 2}。
- **未做**：R-00482 / R-00483 的实施（另开窗口按 §五 提示词）；远端分支删除与 PR #12 关闭（归 R-00482 验收 8，由实现方做）；架构仓本文件与报告的提交（待 Owner）。
- 首轮脚本在第 3 步因评论块闭合标记解析错误中断，第 1–2 步已成功且未重复执行；续跑脚本从第 3 步开始；最后一次 Room 全量读回改用 curl 落文件后解析（zsh `echo` 会吃掉 JSON 里的反斜杠转义，不能经管道）。
<!-- /exec-log -->
