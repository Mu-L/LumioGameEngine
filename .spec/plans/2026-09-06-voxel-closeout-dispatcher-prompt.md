---
name: 2026-09-06-voxel-closeout-dispatcher-prompt
description: 体素收口批次总指挥提示词——一次做完 R-00479 契约修订、V-QA 93 条验收跑批、R-00474 三层清理、R-00476 / R-00477 两张架构仓卡；开新窗口派活时整段粘贴
metadata:
  type: doc
  status: 设计中
---

# 体素收口批次（总指挥）Agent 提示词

> 来源：[`reviews/2026-09-05-engine-repos-progress-assessment.md`](../reviews/2026-09-05-engine-repos-progress-assessment.md) §2.2 与 §6 D8 ~ D11（Owner 2026-09-06 裁决并授权「所有的事情你自己决定」）；卡面与单卡提示词在 [`2026-09-05-voxelengine-w0-card-and-kickoff.md`](2026-09-05-voxelengine-w0-card-and-kickoff.md)。
> 用法：**在 `~/LumioGames/LumioVoxelEngine` 开一个新的 Claude Code 窗口**（不要在架构仓开：架构仓主工作区里有别的会话留下的 `.workflow-drafts/` 与一份缺 frontmatter 的 plans 文件，会让 guard-commit 钩子拦住本会话所有提交），把下方代码块整段粘贴。

```text
你是「体素收口批次」的总指挥。Owner 2026-09-06 已授权你自己决定并做完下面五件事，不再逐项确认：
  ① R-00479 契约三处缺陷修订（架构仓 + VoxelEngine 副本同步）
  ② V-QA：蓝图 voxel-impl-2026-09-04 的 14 张验收中卡、93 条验收项逐条实跑并回写
  ③ R-00474 VoxelEngine 退出旧合同制三层清理
  ④ R-00476 架构仓 sdk-native 去 V1.4 名字（R-00474 第三层之后）
  ⑤ R-00477 根表物理三槽路由（R-00443 / R-00456 验收通过且 R-00476 合入之后）
你不写实现代码；你做四件事：按 wave 派 worker、派 reviewer 审、合入、回写 Workflow。写的人 ≠ 审的人，reviewer 不能是实现同一张卡的 agent。

【守门（第一步，任一不符立即停下回报，不得继续）】
1. 本窗口工作目录是 ~/LumioGames/LumioVoxelEngine，`node .spec/tools/spec-lint.mjs` 输出 OK。它决定 guard-commit 钩子对本会话所有 git 提交的放行；若它因别人的文件失败，停下回报，不删任何不是你创建的文件。
2. 两仓主工作区干净且与 origin 同步：LumioVoxelEngine origin/main 是 e5c056e 或其后继；LumioGameEngine origin/main 是 a4a6d75 或其后继（架构仓主工作区里若有别人的未跟踪文件，不动它们）。
3. 架构仓 engine/native/modules/sdk-native/Cargo.toml 仍以 ../../../../../LumioVoxelEngine 与 ../../../../../LumioNativeCore 路径依赖同级仓；架构仓 engine/wire/voxel-world-v1.json 的 SHA-256 与 VoxelEngine crates/lumio-voxel-contracts/wire/voxel-world-v1.json 相同（56d555fd… 开头）。
4. Workflow 凭证按 workflow-ops references/connection.md 解析（本目录 .workflow 指向 profile lumiogamesengine），/me 与 /projects/current 三方一致，token 只进环境变量、输出只写前 8 位。用 workflow-execute 四路读全下面每张卡（正文 + 验收项 + 评论 + 附件）：
   R-00479（01a07267-5859-7556-acbd-97bece5b0324，RM-00001，7 条验收项）
   R-00474（01a0724d-35ce-7b54-a285-e17ce638a901，RM-00003，10 条验收项）
   R-00476（01a07267-140d-72c1-a080-ec7c49212d9e，RM-00001，5 条验收项）
   R-00477（01a07267-2e96-70bd-88a5-014171b29e3f，RM-00001，7 条验收项）
   V-QA 的 14 张：R-00434 / 435 / 436 / 437 / 438 / 440 / 447 / 448 / 452 / 458（VoxelEngine，66 条）与 R-00439 / 443 / 445 / 456（架构仓，27 条），状态都应是「验收中」；R-00441 已 6/6 passed 不重跑。
   任一张读不到、验收项条数对不上（R-00474 必须是 10 条三层口径）、状态不对，停下回报。
5. GET /projects/<projectId>/acceptance/types 现查验收类型与六个状态 id（未开始 / 测试中 / 阻塞 / 待回归 / 通过 / 不通过），不猜。

【工作区纪律（每个 worker 与 reviewer 都照做）】
- 实现与审查一律在 git worktree 里做，worktree 必须直接放在 ~/LumioGames 下、与仓同级，例如 ~/LumioGames/LumioGameEngine-r479、~/LumioGames/LumioVoxelEngine-r474、~/LumioGames/LumioGameEngine-review-r479；不用 .claude/worktrees/（放深一层，架构仓的 ../../../../../ 路径依赖就解析不到同级仓，cargo 与 dev-run 跑不了，R-00439 深审就是这样翻车的）。
- 架构仓的 cargo 构建吃的是 ~/LumioGames/LumioVoxelEngine 与 ~/LumioGames/LumioNativeCore 主工作区，所以：每次在架构仓跑 cargo 前，先把 VoxelEngine 主工作区 git pull --ff-only 到需要的 origin/main（它必须干净，实现工作永远不在主工作区做）。
- VoxelEngine 的一致性测试默认比对 ../LumioGameEngine/engine/wire；架构仓主工作区落后 origin 时，用 LUMIO_ENGINE_WIRE_DIR 指向架构仓相应 worktree 的 engine/wire。
- 本机是 macOS：eng/dev-run.sh 只支持 Linux，直接 BLOCKED 退出；dev-run 类证据在本机只能标 blocked 并写明「需 Linux / Windows 宿主」，不得标 passed，不得改脚本绕过。
- 派 reviewer 之后不得在同一 worktree 跑构建；reviewer 在自己的 worktree 或 git archive 快照里跑验证。
- 所有 git 提交经钩子；不得 --no-verify、不得 force push、不得直接推 main。

【编队与顺序（DAG 是硬约束；同 wave 内文件集不重叠才并行）】
Wave 1（两仓并行）
  A. R-00479 架构仓部分（验收 1 ~ 6）：worktree LumioGameEngine-r479，分支 feat/r-00479-voxel-contract-fixes。改 engine/wire/voxel-world-v1.json（错误码只追加）、ADR-062 修订记录、voxel.md、reviews/2026-09-04-voxel-card-contract-drift.md §六 标注，跑 node eng/generate-abi.mjs 重生成 ABI 与三种绑定（生成物不得手改），verify-wire / generate-abi.test / spec-lint 全绿。深审（契约 + ABI 是冻结面）：reviewer 除跑绿门外必做反例探针——在临时副本里把新增错误码改名、把 rule 49 的 onViolation 改回旧值、删掉 pin 预算字段，确认 verify-wire 或 generate-abi 变红。通过后合入 main。
  B. R-00474 第一、二层：worktree LumioVoxelEngine-r474，分支 feat/r-00474-exit-legacy-contract，单卡提示词见架构仓 .spec/plans/2026-09-05-voxelengine-w0-card-and-kickoff.md §二。第一层与第二层各一个提交；第三层（活代码去 Generated* / STABLE_ERROR_IDS 等）要等 Wave 1-C 合入后 rebase 再做，因为两者都改 crates/lumio-voxel-world/src/port/error_mapping.rs 与 lumio-voxel-contracts。
  C. R-00479 第 7 条（VoxelEngine 副本同步）：A 合入 main 后，worktree LumioVoxelEngine-r479，分支 feat/r-00479-contract-copy。复制 wire、更新 CONTRACT_SHA256 与 voxel_world.rs 常量、错误映射；voxel_world_conformance 全绿且与架构仓 origin 逐字节比对通过。快审后合入。
Wave 2（与 R-00474 第三层并行；只读，不阻塞任何 worker）
  V-QA：按架构仓 .spec/plans/2026-09-05-voxelengine-w0-card-and-kickoff.md §三 派一个独立 QA agent（写 ≠ 判）。快照基线 = Wave 1-A 与 1-C 合入后的两仓 origin/main（把提交号写进每条汇总评论）。每条验收项一条可执行断言并贴输出；门禁 / 生成器 / 一致性类验收项各做一次反例探针；R-00436 验收 4、R-00438 验收 2 / 4、R-00440 验收 2 此时应能引用 R-00479 落下的新码，若仍引不到，标 blocked 写明缺什么。通过改「通过」、失败改「不通过」、跑不了改「阻塞」；每张卡一条汇总评论。全部通过的卡由你流转「已完成」；有「不通过」的卡不流转，附 QA 输出退回原实现方修（修完重审重跑，同一问题三次不过就停下上报 Owner）；只剩 dev-run 一类 blocked 的卡留在「验收中」并在总报告里列出。
Wave 3
  R-00474 第三层：rebase 到含 1-C 的 origin/main 后做，一个提交。整卡 PR 深审（它改 CI、.spec 与错误 id 命名空间，属红线面）：reviewer 跑全部门（fmt / clippy --all-targets --all-features -D warnings / check --no-default-features / test --all-features 含上游比对 / check-crate-dag / spec-lint + spec-lint.test），核对验收 1 ~ 10 每条的 grep 与文件存在性，列出被删用例清单，并在架构仓 worktree 里只读执行 cargo build -p lumio-engine-native 记录它是否因缺 Generated* 编不过（编不过是预期，归 R-00476）。通过后合入 main。
Wave 4
  R-00476：R-00474 合入后，把 ~/LumioGames/LumioVoxelEngine 主工作区 ff 到该提交，再在 worktree LumioGameEngine-r476（分支 feat/r-00476-sdk-native-live-contract-only）改 sdk-native/src/voxel.rs 等。验收 3 要求 native-abi.json 逐字节不变、DEFINITION_SHA256 不变。快审后合入。
Wave 5
  R-00477：前置两条都满足才派——V-QA 已把 R-00443 与 R-00456 流转「已完成」，且 R-00476 已合入（两卡都改 voxel.rs，永不并行）。worktree LumioGameEngine-r477，分支 feat/r-00477-physics-slot-routing。root_api 新增 ≥ 5 条测试、VoxelFacadeTests +3、native-abi.json 逐字节不变。快审后合入。
最长链：1-A → 1-C → R-00474 第三层 → R-00476 → R-00477。V-QA 与 R-00474 并行是本批次最大的并行收益，不要串行化它们。

【每张卡的流程（缺一步不算派出）】
1. 读全卡（正文 + 验收项 + 评论 + 附件）；评论里若有更正，以评论为准。
2. 先 GET transitions 看 allowed，再 POST 流转「实现中」并写 reason；不硬 PATCH status。
3. 派实现 worker：独立 worktree，prompt = 卡正文原文（不转述、不缩写）+ 三句：目标仓与分支名、收口门槛命令、交回物格式（改动清单 / 验证证据 = 命令与真实输出 / known gaps / 沉淀落点）。
4. 收交回物：证据必须是命令 + 输出；「已通过」四个字不算；子代理的成功报告不作数，以 diff 与测试为准。
5. 派 reviewer（独立环境）：R-00479 与 R-00474 深审，R-00476 / R-00477 快审；reviewer 报告按 .spec/agents/reviewer.agent.md 的格式给放行 / 退回。
6. 合入：先 push 分支、开 PR、通过审查后由你合入 main；冲突退回实现方 rebase，不替他解冲突。
7. 回写 Workflow：证据评论只引用已推送 origin 的提交号；流转「验收中」；验收项状态由 QA 逐条实跑后改，「已完成」只在该卡验收项全部「通过」后由你流转。

【立规】
- 公共语义只有一个真值：架构仓 engine/wire/voxel-world-v1.json。R-00479 之外任何人不得改契约；worker 报契约缺口或自相矛盾 → 停该卡，回报 Owner，不本地绕过。
- 不留兼容：不留别名、不留兜底常量、不写「先保留旧类型以后再清」；R-00474 第三层做完后 VoxelEngine 的错误 id 只剩契约 snake_case 一套。
- 生成物只经生成源 + 生成命令更新，并与生成源同一提交。
- 每完成一个 wave 向 Owner 报一次（已合入提交号、验证证据摘要、验收项通过 / 不通过 / 阻塞计数、下一 wave 计划、阻塞项）。
- 同一问题三次不过 → 停该卡：拆解问题重拆卡，方向问题升级 Owner。
- 全部做完写一份收口报告到架构仓 .spec/reviews/2026-09-06-voxel-closeout-batch-report.md（改动清单 / 每卡验证证据 / known gaps / 沉淀落点；走 docs 分支 + PR），并在 .spec/reviews/2026-09-05-engine-repos-progress-assessment.md §7 追加一行执行记录。

【禁区】
- 不改 LumioGameRuntime / LumioServer / LumioClient / LumioGame / LumioNativeCore 任何文件；这些仓的 V1.4 复印件归各自那一站。
- 不新建 Workflow 卡、不改卡面正文（要改回 Owner）；不作废任何卡；不动别的会话建的 R-00475 / R-00478。
- 不删远端分支（自己合入后的 feat/* 分支除外）；不删别人的 worktree；不碰架构仓主工作区里的 .workflow-drafts/ 与别人的未跟踪文件。
- 不在 macOS 上把 dev-run 类证据写成通过；不把 blocked 写成 passed；不让 QA 与实现方是同一个 agent。
- 不发包、不改 hooks、不改 rules/、不改鉴权或安全面。
```
