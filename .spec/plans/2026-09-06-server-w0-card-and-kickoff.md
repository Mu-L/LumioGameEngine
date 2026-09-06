---
name: 2026-09-06-server-w0-card-and-kickoff
description: LumioServer 补单——W0 清理卡 R-00478 按 D21 重写的卡面（10 条验收项）、开工提示词，以及 R-00408 / R-00388 / R-00391 的处置口径；派 Server 清理活时查
metadata:
  type: doc
  status: 设计中
---

# LumioServer · W0 清理卡卡面、开工提示词与在途卡处置口径

> 来源：[`reviews/2026-09-05-engine-repos-progress-assessment.md`](../reviews/2026-09-05-engine-repos-progress-assessment.md) §2.4 与 §6 **D15（已裁决并执行：52 张旧卡作废、R-00260 / R-00276 转 done、R-00478 已建）**、**D21（已裁决：R-00478 只做清理，macOS / Linux 编译归 R-00408）**、**D22（已裁决：mvp-host 随本卡删除；account-server 归 R-00408 删）**、D23 ~ D24（建议，待裁）。卡面按 workflow-ops `card-spec`（背景 / 目标 / 验收 / 边界）。**R-00478 已建**（`01a07267-4621-7951-a79d-caecd65f6415`，RM-00006，P0 / medium，2026-09-05 16:29Z），建卡时正文只有一行「按 D15…」、0 条验收项；本文 §一 是按 D21 / D22 重写后的卡面，**线上正文与 11 条验收项已于 2026-09-05 23:50Z 回写并读回 11 / 11**（Owner 授权；类型「质量验收」、初始「未开始」，与 R-00474 同一套 id）。与前三站的 [`2026-09-05-nativecore-w0-card-and-kickoff.md`](2026-09-05-nativecore-w0-card-and-kickoff.md)、[`2026-09-05-voxelengine-w0-card-and-kickoff.md`](2026-09-05-voxelengine-w0-card-and-kickoff.md)、[`2026-09-06-gameruntime-w0-card-and-kickoff.md`](2026-09-06-gameruntime-w0-card-and-kickoff.md) 同一格式。

## 一、卡面（RM-00006 · 既有卡 R-00478 · 待按此重写）

- **标题**：`[程序·工程] 退出旧合同制残留：删 docs 镜像与蓝图、CI 基线校验、generated / contracts / xtask 守卫、13 个空模块骨架与 C# mvp-host 整目录；README 与 .spec 改 Living Architecture 口径`
- **链接**：`https://lumiogamesengine.workflow.games/requirements/01a07267-4621-7951-a79d-caecd65f6415`
- **优先级 / 风险**：P0 / medium（沿用）。

### 背景

架构仓已按 ADR-059 转入 Living Architecture：唯一 ABI 真值是 `engine/abi/native-abi.json`，公共语义各落一份 `engine/wire/<name>-v1.json`，Baseline / `tools/lumio_contract.py` / 生成源仓 `LumioGameEngineArchitecture` 全部不存在；Server 的设计现状在架构仓 `.spec/knowledge/features/ds-server.md`。LumioServer（origin `4c7688b`）仍整仓包在 V1.4 合同制里，2026-09-06 实测：`LGE-V1` 命中 60 个已跟踪文件；`docs/architecture/` 6 版正文 + `.baseline.sha256`；`docs/LumioServer_Framework_Implementation_Design_2026-08-27/`（568 KB）是 RM-00006 已作废 47 张卡的蓝图；`docs/specs/`（456 KB）是已冻结 C# mvp-host 的设计与 14 张卡；`generated/` 3 个 crate 里 1 个从已退役仓 git rev `3d5e29d` 拉 6 个 `lumio-gen-*` 依赖、2 个是 `compile_error!` 拒绝壳；`contracts/*.lock.toml` 3 份写死 `C:/Work/...`；`tools/xtask` 6,844 行只做 V1.4 contracts verify 与 15 模块 DAG / 队列守卫（`.spec/guards/*.toml`、`tests/policy/`）；15 个模块目录里 13 个只剩 README，是同一份蓝图的骨架；CI `repository-policy.yml` 断言 v1.4 正文、`sha256sum -c`、README 含 `LGE-V1.4-2026-08-27`；`README.md` / `modules/README.md` / `.spec/AGENTS.md`（写的还是 V1.2）/ `.spec/knowledge/standards/repository-architecture.md` 全是合同制口径，`.spec/AGENTS.md` 还把设计落 `docs/specs/`；仓根残留两份 `.wf-report-*.md`。RM-00006 的 52 张旧卡已于 D15 作废，这些文件已没有任何卡在引用。

### 目标

同 NativeCore（D1，R-00473）、VoxelEngine（D8，R-00474）、Runtime（D12，R-00475）口径：**全清、不留兼容**。做完后仓里只剩「消费 SDK 的 Rust 宿主」这一种形状：`modules/process` + `modules/host-runtime` + `crates/lumio-host-testkit` + `entity-chat-host`（`account-server/` 到 R-00408 把套件改为自签凭据后随它删，D22），公共语义只从架构仓 `engine/` 与 `knowledge/features` 取，没有第二份契约真值、没有蓝图骨架、没有指向已退役仓的依赖或路径。**本卡不碰宿主代码、不修 macOS / Linux 编译**（D21：归 R-00408）。

### 验收（11 条，全部机器可判；待建为原生验收项）

1. `git grep -l -i -e 'LGE-V1' -e 'LumioGameEngineArchitecture' -e 'Root ABI' -e 'lumio_contract.py' -e 'CoreEngine' -e 'C:/Work'` 在已跟踪文件中只命中 `.spec/decisions/0001 ~ 0008` 历史 ADR 正文（只追加「被 0009 取代」段，不改写原文）。
2. `docs/` 整目录不存在（含 `docs/architecture/` 6 份正文与 `.baseline.sha256`、`docs/LumioServer_Framework_Implementation_Design_2026-08-27/`、`docs/specs/`）；仓根 `.wf-report-R-00359.md`、`.wf-report-live11.md` 不存在。
3. `.github/workflows/repository-policy.yml`：`readme` job 不再 `test -s docs/architecture/...`、不再 grep `LGE-V1.4-2026-08-27`、不再 `sha256sum -c`，对 README 的小节断言只断言重写后仍存在的小节；`cargo-entity-chat` 与 `cargo-entity-chat-eleven` 两个 job 的命令与平台矩阵**逐字不变**（归 R-00408）；`mvp-host` job（含 `.architecture-source` checkout 与 `setup-dotnet`）整段删除。
4. `generated/`（3 crate）、`contracts/`（3 lock）、`tools/xtask/`（整 crate）、`.spec/guards/`、`tests/policy/` 不存在；`Cargo.toml` `members` 只剩 `crates/lumio-host-testkit`、`modules/host-runtime`、`modules/process`；`.cargo/config.toml` 无 `xtask` alias；`Cargo.lock` 随之更新且 CI `--locked` 通过。
5. 13 个 README-only 模块目录不存在（auth、control-plane-adapter、coreclr-host、host-profiles、maintenance-agent、observability、pacing、persistence-host、protocol-dispatch、release-agent、session、transport、world-slot）；`modules/README.md` 重写为只描述现有两个 crate 的职责、线程与有界队列现状，不再有 15 模块地图、三张依赖图、SRV-D 决策门、Baseline 行。
6. `README.md` 重写：删「架构基线 / 唯一架构源 / 本地镜像 / Architecture Gate / Generated Contract Dependencies」诸段，改为指向架构仓 `.spec/knowledge/features/architecture.md` 与 `ds-server.md`、`engine/wire/*.json`、`engine/abi/native-abi.json`；「子模块」表只列实际存在的目录；不复述任何公共契约字段。
7. `.spec/AGENTS.md`「项目是什么」删 `LGE-V1.2` 基线句与「只读镜像」句；「默认流程」设计落点改 `.spec/knowledge/features/`、计划 `.spec/plans/`；「收口门槛」改为 `node .spec/tools/spec-lint.mjs && node --test .spec/tools/spec-lint.test.mjs && cargo fmt --all -- --check && cargo clippy --workspace --all-targets --locked -- -D warnings && cargo test --workspace --locked`，并注明「Rust 三条在 main 上今天只能在 Windows 跑，macOS / Linux 归 R-00408」，删 `python3 tools/lumio_contract.py validate` 与「复现 repository-policy.yml」句。`.spec/knowledge/standards/repository-architecture.md` 按 Living Architecture 重写（唯一事实源 = 架构仓 `engine/` + `knowledge/features`；仓库表按 `architecture.md` §2 现状），`.spec/knowledge/README.md` 导航行同步；`rust-entity-chat-host.md`「待解决」的「`mvp-host/` 仍冻结，归 N-13」一句删除。
8. 本仓 `.spec/decisions/0009-exit-legacy-contract-regime.md` 新增（编号落笔时现查最高号），记录本次退出、来源 D15 / D21 与被删清单；0001、0002 与 0008（C# mvp-host 冻结链 0004 → 0008 的现行末端）追加「被 0009 取代」段，其余不改写；`decisions/README.md` 索引同步。
9. `node .spec/tools/spec-lint.mjs && node --test .spec/tools/spec-lint.test.mjs` exit 0；PR 上 `README policy` 与 `Cargo entity-chat acceptance`（windows-latest）两个 job 绿；`cargo test --workspace --locked` 在 Windows 机实跑通过，测试数 = 改动前的数减 42（`tools/xtask` 自带测试），被删测试逐条列清单；`cargo-entity-chat-eleven` 维持 `continue-on-error` 现状，不得为绿改尺子。
10. 不新增任何 `cfg` 门、`#[allow(dead_code)]`、兼容别名或「先保留」开关；`git diff --stat origin/main -- modules/process modules/host-runtime entity-chat-host account-server` 为空——前三处是 R-00408 的文件集，`account-server/` 随 R-00408 删（D22）。
11. `mvp-host/` 整目录不存在（含 `contract-mirror/`、`GeneratedContracts/`、`eng/*.sh|*.ps1`）；`.spec/knowledge/features/room-admission.md`（只描述 mvp-host `Admission` 实现）删除并同步 `knowledge/README.md` 导航；`.gitignore` 去掉 mvp-host 行；`git grep -l -i -e 'mvp-host' -e 'MvpHost' -e 'lumio-mvp-host'` 在已跟踪文件中只命中 `.spec/decisions/0002 ~ 0009`、`account-server/**`（随 R-00408 删）与 `modules/process/tests/entity_chat_acceptance.rs`（R-00408 文件集，该处是「不得假冒 lumio-mvp-host」断言）。

### 边界

只动本仓；不碰 `modules/process/**`、`modules/host-runtime/**`、`entity-chat-host/**` 与 Cargo CI 作业；不修 macOS / Linux 编译、不动两份 Win32 加载器（R-00408，D21 / D24）；不改 `engine/wire` / `engine/abi`；不建新卡。**D22 已裁决**：mvp-host 随本卡删除（验收 11）；`account-server/` **本卡不删**——今天只有 11 场景套件经 `discover.rs:24` 拉起它，三个必过测试文件都不依赖，套件改为自签测试凭据（`suite.rs` 已 `generate_keys()`、`issue_admission_credential` 已在）与目录删除一起归 R-00408（§三 ③），避免两卡抢 `modules/process/**`。

## 二、开工提示词（另开窗口，工作目录 `~/LumioGames/LumioServer`；Rust 侧验证在 Windows 机或 PR 的 windows-latest 作业）

```text
你是 LumioServer 的工程清理工程师。任务：Workflow lumiogamesengine 的 R-00478（RM-00006）。

【守门】
1. LumioServer origin/main = 4c7688b 或其后继，本地 main 与 origin 同步、工作区干净；不满足就停。
2. 用 workflow-execute 读全 R-00478：正文 + 验收项 + 评论；验收项必须是 11 条且与架构仓
   .spec/plans/2026-09-06-server-w0-card-and-kickoff.md §一 一致；若线上仍是一行正文 / 0 条验收项，说明卡面还没按 D21 回写，
   停下回报，不得按窄口径开工。
3. 看一眼 R-00408 是否已合入 origin/main：合入了也不影响本卡（本卡不碰 modules/** 与 entity-chat-host/**），但 Cargo.lock
   要在它之后重生成，避免锁文件冲突。

【指路】
- 卡面正文就是任务书，按验收 1 → 11 的顺序做，四个提交：① 删 docs/、.wf-report 与 mvp-host/ 整目录（含 room-admission.md 与 .gitignore 行），改 CI（readme job 去基线断言、mvp-host job 整段删）；② 删 generated / contracts /
  xtask / guards / tests/policy、改 Cargo.toml 与 .cargo/config.toml、重生成 Cargo.lock；③ 删 13 个空模块目录、重写 modules/README.md
  与 README.md；④ 重写 .spec/AGENTS.md、repository-architecture.md、knowledge/README.md 导航行、rust-entity-chat-host.md 一句，
  新增 ADR 0009 并在 0001 / 0002 / 0008 追加「被取代」段。
- 公共语义拿不准（README 某段是不是仍是现行口径）→ 以架构仓 .spec/knowledge/features/architecture.md 与 ds-server.md 为准；
  两边都没有的东西一律删，不本地造第二份。
- macOS 上 cargo test 链接不过（kernel32）是已知现状，归 R-00408；本卡不得加 cfg 门绕过。Rust 验证在 Windows 机跑
  cargo fmt / clippy / test --workspace --locked，或以 PR 的 windows-latest 作业为证据。

【立规】
- 领卡先经 Workflow 流转「实现中」并写 reason（钉 origin SHA）；改动在 feat/r-00478-exit-legacy-contract 分支，先 push 再回写证据。
- 每次提交前：node .spec/tools/spec-lint.mjs、node --test .spec/tools/spec-lint.test.mjs exit 0；Rust 三条按上条在 Windows 或 CI 取证；
  测试证据必须是实跑的命令与输出；被删测试（xtask 42 条）逐条列清单。
- 交付 = 改动清单 + 验证证据（命令 + 关键输出）+ known gaps + 沉淀落点（本仓 ADR 0009），写成 PR 描述并同步为 R-00478 的证据评论，
  评论只引用已推送 origin 的提交号；做完流转「验收中」，「已完成」由总调度核验后流转。走 PR，不直接推 main。
- 遇到问题先找根因再改；同一问题修三次不成，停下上报。

【禁区】
- 不碰 modules/process、modules/host-runtime、entity-chat-host、两个 cargo-entity-chat CI 作业；不修编译、不加 cfg / allow / 别名；
  account-server/ 不动（随 R-00408 删）；不改 engine/wire、engine/abi；不动其他仓；不建卡；密钥不入库不进日志。
```

## 三、在途卡处置口径（D22 / D23 / D24 已裁决；Workflow 写入已于 2026-09-05 23:59Z 执行并读回）

- **R-00408（R5-03，in_progress）**：**已评论**（`01a07403-4634`）钉三件事——① 跨平台口径（D24）：删两份 Win32 手写加载器（`sdk_loader.rs` 840 行、`native_timer.rs` 418 行），只留一份跨平台 `dlopen`（`libloading`）加载，或消费架构仓 SDK 提供的 Rust loader；验收加「Linux `cargo test --workspace --locked` 与 `clippy -D warnings` 绿」，CI Rust 作业加 ubuntu；② 卡面里「xtask policy 绿」一句作废——xtask 随 R-00478 删除，`suite.rs` / `wire.rs` 的 sleep / spawn 约束若仍要，用测试实现；③ D22 已裁决：验收套件改为自签测试凭据（`suite.rs` 已持有签票私钥，`account.rs` / `discover.rs` 的拉起路径删除）并删除 `account-server/` 整目录、README 子模块表去掉该行；LumioGame `integration/entity-chat/scenarios.mjs` 里 11 处 `lumio-mvp-host / connectMvpHost` 路径与 blocked 占位一并清（原 R-00391）。
- **R-00388（R4-02）**：**已流转 rejected + 评论 `01a07403-3b14`**；PR #33 **已关闭**（分支保留）；PR #33 不直接合，由 R-00408 实现方 cherry-pick 自驱 / 背压 / 日志 / cfg-gate 提交后关闭（D23 已裁决）。
- **R-00391（R4-08）**：**已流转 rejected + 评论 `01a07403-423f`**，并入 R-00408，评论引 `plans/2026-09-04-rm-00011-r5-cards.md` 第 41 行（D23 已裁决）。
- **远端分支**：13 个已合入 `feat/*` + `feat/r-00346-admission` 删除（需授权）；`feat/r-00388-r4-02-self-drive` 待 R-00408 cherry-pick 完成后删。
