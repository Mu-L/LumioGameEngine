---
name: 2026-09-06-client-w0-card-and-kickoff
description: LumioClient 补单——W0 清理卡 R-00481 的卡面（12 条验收项）、开工提示词与 R-00408 / R-00415 / 工作项处置口径；派 Client 清理活时查
metadata:
  type: doc
  status: 设计中
---

# LumioClient · W0 清理卡卡面、开工提示词与在途卡处置口径

> 来源：[`reviews/2026-09-05-engine-repos-progress-assessment.md`](../reviews/2026-09-05-engine-repos-progress-assessment.md) §2.5 与 §6 **D25 / D26 / D29（另一会话已执行，Owner 收签）**、**D30（已裁决：清干净、可接受大重构；GrantClaim / codec 归 R-00408）**、**D31（已裁决：Unity / HybridCLR 空壳全删）**、**D34（已裁决：R-00408 / 392 / 393 补差异评论不改状态）**、**D35（已裁决：CI 只留 spec-lint + dotnet test，跟 Runtime main 不钉号；开发期不堆检测工具）**、D36（已授权：三张工作项取消、R-00415 流转）。卡面按 workflow-ops `card-spec`。**R-00481 已建**（`01a07411-dbdc-7a6f-bf24-cc21f7c24a30`，RM-00007，P1 / medium，2026-09-06 00:15Z，另一会话建卡时正文三层、0 条验收项）；**线上正文与 12 条验收项已于 2026-09-06 00:5x UTC 按本文 §一 回写并读回 12 / 12**（Owner 授权；类型「质量验收」、初始「未开始」，与 R-00474 / R-00478 同一套 id）。与前四站的 [`2026-09-05-nativecore-w0-card-and-kickoff.md`](2026-09-05-nativecore-w0-card-and-kickoff.md)、[`2026-09-05-voxelengine-w0-card-and-kickoff.md`](2026-09-05-voxelengine-w0-card-and-kickoff.md)、[`2026-09-06-gameruntime-w0-card-and-kickoff.md`](2026-09-06-gameruntime-w0-card-and-kickoff.md)、[`2026-09-06-server-w0-card-and-kickoff.md`](2026-09-06-server-w0-card-and-kickoff.md) 同一格式。

## 一、卡面（RM-00007 · 既有卡 R-00481 · 已按此重写）

- **标题**：`[程序·工程] LumioClient 退出旧合同制残留：删镜像 / 五文件 / 旧 fixture 适配 / Unity 空壳，CI 只留 spec-lint + dotnet test 并跟 Runtime main`
- **链接**：`https://lumiogamesengine.workflow.games/requirements/01a07411-dbdc-7a6f-bf24-cc21f7c24a30`
- **优先级 / 风险**：P1 / medium（沿用）；Owner = Lumio 账号（`ownerId` 已设）。
- **与在途卡的边界**：R-00408 Client 部分拥有 `modules/replica/**`（`GrantClaim` 出清、删 `GameplayCodec.cs` / `LiteJsonParser.cs`）、`session/**`、`bot/**`；R-00467 拥有 prediction / session 重写；R-00470 拥有表现与浏览器宿主。**本卡与三者文件集互斥，可并行**。

### 背景 / 目标 / 边界

线上正文即真值（PATCH 已读回，5,362 字）。要点：四层清理——① 文档与镜像（`docs/architecture`、五文件包、`docs/specs`、`contract-mirror` 整目录与全部镜像 / 五文件 / sdk-pin / toolchain 脚本、`tests/Fixtures`；`docs/spikes` 保留）；② 旧 fixture 消费（四模块 `Generated*Adapter` / `Generated*FixtureTests` 删，契约测试改读架构仓 `engine/wire`）；③ CI 与测试（`repository-policy.yml` 只留 spec-lint 一步，`dotnet-test.yml` 跟 Runtime `main`，ArchitectureTests 改 `git ls-files` 口径，allowlist 删 unity / hybridclr 两项）；④ 空壳与口径（删 `unity-adapter` / `hybridclr-adapter` / `packages/com.lumio.client`，README / `.spec/AGENTS.md` / `repository-architecture.md` / 11 个模块 README 改 Living Architecture 口径，WSS 两个 public 类型出 `Internal/`）。

### 验收（12 条，已建为原生验收项，全部机器可判）

1. `git ls-files | grep -E '^(docs/architecture|docs/LumioClient_five_requested_files|docs/specs|contract-mirror|tests/Fixtures)/'` 为空；`eng/` 下无 `sync-/verify-contract-mirror.*`、`verify-five-file-package.*`、`verify-sdk-pin.*`、`verify-toolchain.*`、`upstream-api-map.md`、`upstream-contract-smoke/`。
2. `git grep -l -i -e 'LGE-V1' -e 'LumioGameEngineArchitecture' -e 'lumio_contract.py' -e 'Architecture Gate'` 只命中 `docs/spikes/**` 与 `.spec/decisions/0001 ~ 0006` 历史正文。
3. `git ls-files | grep -E 'Generated(Envelope|Handshake|Replica|Prediction).*\.cs$'` 为空；四模块 `dotnet build` 0 warning 0 error（replica 若因 R-00408 未合入报 `GrantClaim` CS1061，记 BLOCKED-by-R-00408 附原文，不算本卡失败）。
4. `git grep -l 'contract-mirror\|upstreamCorpusPin\|LGE-V1.4-2026-08-27' -- modules tests eng` 为空；保留的契约测试从 `../LumioGameEngine/engine/wire` 或 `LUMIO_ENGINE_ROOT` 读取。
5. `repository-policy.yml` 只剩 spec-lint 一步；`dotnet-test.yml` 检出 Runtime 无 `ref:` 钉号；两份 yml `grep -c 'sha256sum\|LGE-V1\|verify-'` 为 0。
6. ArchitectureTests 无 `EnumerateFiles(RepoRoot`，改 `git ls-files` 口径；对照组：建嵌套 worktree 后全绿、删后仍全绿（输出留档）。
7. `git ls-files | grep -E '^(modules/unity-adapter|modules/hybridclr-adapter|packages/)'` 为空；slnx / allowlist / README 子模块表 / CI 模块名单无三者；T-00009 已取消。
8. `WebSocketClientConnectionFactory` / `WebSocketTransportOptions` 在 `Public/`；`Internal/` 下无 public 类型；`PublicApiSupplierLeakTests` 与 ArchitectureTests 全绿；成因注释与 README 已知缺口条目已删。
9. README 无「架构基线 / Architecture Gate / Generated Contract Dependencies / Unity 与 HybridCLR」段；子模块表 11 个；新增「开工先读」指向架构仓 `architecture.md`、`ecs.md` M10、`gas.md` M7、`movement.md`、`engine/wire/*.json`。
10. `.spec/AGENTS.md` 改 Living Architecture（设计落 `.spec/knowledge/features/`、计划 `.spec/plans/`），收口门槛 = `spec-lint && spec-lint.test && dotnet test LumioClient.slnx`；`repository-architecture.md` 同步；spec-lint OK。
11. 11 个模块 README `grep -c 'LGE-V1.2\|docs/architecture/'` 全 0；「阶段」与实况一致（session / prediction 写「Foundation 骨架，待 R-00467 重写」；observability 写 `EventDispatcherWorker` 线程模型待浏览器路线重看）。
12. `dotnet restore LumioClient.slnx --force-evaluate && dotnet test LumioClient.slnx --no-restore`（对 Runtime `main`）全绿，或唯一红项是 R-00408 未合入的 replica `GrantClaim`（附原文与 Runtime SHA）；交回物附删除清单、每条命令输出与退出码。

## 二、开工提示词（另开窗口，工作目录 `~/LumioGames/LumioClient`；整段粘贴）

```
你是 LumioClient 仓的清理工程师，承接 Workflow lumiogamesengine 的 R-00481《LumioClient 退出旧合同制残留》
（https://lumiogamesengine.workflow.games/requirements/01a07411-dbdc-7a6f-bf24-cc21f7c24a30）。
目标只有一句：做完之后，这个仓里找不到任何合同制时代的文档、镜像、守卫脚本和 Unity 空壳，CI 只回答两个问题——
.spec 结构对不对、对着 Runtime main 的 dotnet test 过不过。Owner 原则：能清干净就清干净，可以接受大重构，不留兼容；
开发期不堆检测工具。

【第一步：守门检查，任一不满足就停下回报，不得绕过】
1. `git status -sb` 工作区干净；`git pull --ff-only` 后 HEAD == origin/main，把 SHA 写进开工评论。
2. `git -C ../LumioGameRuntime pull --ff-only` 后 0 ahead / 0 behind；本卡所有 dotnet 命令都对这份 Runtime 跑。
3. `git ls-remote --heads origin`：若 `fix/r5-client-runtime-frames` 已合入 main，验收 3 / 12 的「BLOCKED-by-R-00408」豁免失效，必须全绿。
4. 用 Workflow API 四路读全本卡（正文、12 条验收项、全部评论、附件），再读架构仓
   ~/LumioGames/LumioGameEngine/.spec/reviews/2026-09-05-engine-repos-progress-assessment.md §2.5 与 §6 D30 / D31 / D35，
   以及 .spec/knowledge/features/architecture.md（Living Architecture：真值 = engine/abi/native-abi.json + engine/wire/*.json；
   LumioGameEngineArchitecture 仓已退役）。
5. 领卡：把本卡流转到「实现中」，reason 写 HEAD SHA 与 Runtime SHA。

【第二步：基线快照】
跑一次 `dotnet build LumioClient.slnx --no-restore 2>&1 | grep -E 'error|warning' | sort -u` 并留档。
预期唯一红项：modules/replica/src/Public/ReplicaWorld.cs 两处 `GrantClaim` CS1061（归 R-00408，不许你修）。
再跑 `git grep -l -i -e 'LGE-V1' -e 'LumioGameEngineArchitecture'` 留档，作删除清单的起点。

【第三步：四层清理，每层一个提交，层间跑 spec-lint】
① 文档与镜像：删 docs/architecture/、docs/LumioClient_five_requested_files/、docs/specs/、contract-mirror/ 整目录、tests/Fixtures/、
   eng/sync-contract-mirror.*、eng/verify-contract-mirror.*、eng/verify-five-file-package.*、eng/verify-sdk-pin.*、eng/verify-toolchain.*、
   eng/upstream-api-map.md、eng/upstream-contract-smoke/；LumioClient.slnx 去掉 smoke 工程。docs/spikes/ 与 spikes/runtime-wasm/ 不动。
② 旧 fixture 消费：删 connection / handshake / replica / prediction 四模块的 Internal/Generated*Adapter*.cs、Internal/GeneratedHandshakeMessageGate.cs、
   tests/Contract/Generated*FixtureTests.cs；它们引用的类型若仍有生产用途，改成不依赖旧语料的实现；契约测试如需保留，改读
   ../LumioGameEngine/engine/wire/*.json（或 LUMIO_ENGINE_ROOT），不内嵌副本。
③ CI 与测试：.github/workflows/repository-policy.yml 只留 spec-lint + spec-lint.test 一步；dotnet-test.yml 检出 LumioGames/LumioGameRuntime
   去掉 ref: 钉号（跟 main）；tests/Lumio.Client.ArchitectureTests 两处 Directory.EnumerateFiles(RepoRoot…) 改成 `git ls-files` 口径
   （不许用「排除 .claude/worktrees」打补丁、不许放宽断言），并做对照实验：仓内 `git worktree add .claude/worktrees/probe HEAD` 后架构测试全绿、
   remove 后仍全绿，输出留档；eng/project-reference-allowlist.json 删 unity / hybridclr 两项；global.json 与 eng/BannedSymbols.txt 保留。
④ 空壳与口径：删 modules/unity-adapter/、modules/hybridclr-adapter/、packages/com.lumio.client/，同步 slnx / allowlist / README 子模块表 / CI 名单；
   modules/connection 的 WebSocketClientConnectionFactory 与 WebSocketTransportOptions 迁到 Public/，删成因注释与 README 已知缺口条目；
   README.md 删「架构基线 / Architecture Gate / Generated Contract Dependencies / Unity 与 HybridCLR」段，改成 Living Architecture 口径并加
   「开工先读」（架构仓 architecture.md、ecs.md M10、gas.md M7、movement.md、engine/wire/*.json）；.spec/AGENTS.md「项目是什么」改 Living
   Architecture、设计落 .spec/knowledge/features/、计划落 .spec/plans/、收口门槛改为
   `node .spec/tools/spec-lint.mjs && node --test .spec/tools/spec-lint.test.mjs && dotnet test LumioClient.slnx`；
   .spec/knowledge/standards/repository-architecture.md 同步；11 个保留模块 README 删 v1.2 章节引用与 LGE-V1.2 行，「阶段」按代码实况重写
   （session / prediction 写「Foundation 骨架，待 R-00467 重写」；observability 写 EventDispatcherWorker 线程模型待浏览器路线重看）。

【第四步：收口】
`dotnet restore LumioClient.slnx --force-evaluate && dotnet test LumioClient.slnx --no-restore`，全绿或唯一红项是 replica 的 GrantClaim（附原文）；
`node .spec/tools/spec-lint.mjs && node --test .spec/tools/spec-lint.test.mjs` 过；12 条验收项逐条对着命令输出自检。
推分支 `feat/r-00481-exit-legacy-contract` 到 origin，开 PR（不合入）。

【立规】
- 证据只引已推送 origin 的提交号，先 push 再回写；测试证据必须是命令真实输出，「已通过」四个字不算。
- 删除即删除：不留 #if、不留「先保留」目录、不留兼容别名；生成物（packages.lock.json）随源重生，不手改。
- 不新增任何独立检测脚本；需要的断言写进 dotnet test 里的测试。
- 只动本仓文件；公共契约缺口 → 停，卡上标 BLOCKED 上报。
- 做完流转「验收中」，「已完成」由总调度核验后流转。

【禁区】
- 不碰 modules/replica/**、modules/session/**、modules/prediction/**、modules/bot/** 的生产代码（GrantClaim 与 GameplayCodec / LiteJsonParser 归 R-00408；
  预测重写归 R-00467；表现与 WASM 归 R-00470）。
- 不删 docs/spikes/**、spikes/runtime-wasm/**。
- 不改 .spec/rules/system.md 红线正文。
- 不 push main、不合 PR、不删别人的分支。

【交回格式（评论回写到 R-00481）】
一、改动清单（含逐目录删除清单与 PR 链接）；二、12 条验收项逐条证据（命令 + 关键输出 + 退出码）；三、known gaps（至少写明 GrantClaim 是否仍红及 Runtime SHA）；
四、沉淀落点（本仓 .spec/knowledge/lessons.md 若有复发教训，否则写「无需沉淀」）。
```

## 三、在途卡处置口径（D34 / D36 已裁决；Workflow 写入已于 2026-09-06 00:5x UTC 执行并读回）

| 卡 | 已执行 | 口径 |
| --- | --- | --- |
| R-00408 / R-00392 / R-00393 | 各 1 条差异记录评论（201；读回 7 / 2 / 2 条），状态不改 | 评论所引 Runtime `18b5feb` 不在 origin；Client / Server / Game 提交号与 origin 新分支 tip（`5652b95` / `deebeb4` / `e243a39`）不一致；四仓零 PR；解铃 = Runtime 推送 + 四仓 PR + CI 绿 + 合入后重核；此前不得 done。R-00408 另加 Client 侧口径：删 `GameplayCodec.cs` / `LiteJsonParser.cs`，只经 `WireCodec` |
| T-00004 / T-00005 / T-00009 | 流转「已取消」（200 × 3，读回 canceled） | reason 分别指 D22 + R-00408 / R-00389 / D31 |
| R-00415 | approved → in_progress → acceptance（200 × 2）+ 证据评论（201，`06db121` 在 origin/main） | 4 条验收项由 LumioPlatform MS-1 P5-1 联调实跑后判定 |
| R-00466 | 评论 + 1 条验收项 `[RT3-AC-WASM]`（201；6 → 7 条） | CL-1 上游缺口 (a)：Runtime 客户端程序集 `PublishTrimmed` 零警告、浏览器重建哈希逐位一致 |
| 架构仓 | 新建 [`ADR-067`](../decisions/ADR-067-browser-client-prediction-dotnet-wasm.md)（Draft，D3 / D27 落文档） | 浏览器预测走 .NET WASM；不另开「WASM 壳」卡，落 R-00470 AC06 + 三个预算 |

不开的卡：「WASM 客户端壳」（= R-00470 AC06）；「Client Chunk 三态 / RTT 校正」（R-00296 / 298，体素不进预测世界、Stage 0 不下发）；「IL2075 修复」单独卡（已并入 R-00466 验收项）。
