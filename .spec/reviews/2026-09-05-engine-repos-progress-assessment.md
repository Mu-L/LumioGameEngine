---
name: 2026-09-05-engine-repos-progress-assessment
description: 八个实现仓逐仓进度盘点（自最底层 NativeCore 起）——仓库事实、Workflow 对账、阶段判定与下一步；排下一阶段派活前查
metadata:
  type: doc
  status: 实施中
---

# 2026-09-05 引擎实现仓逐仓进度盘点

> 盘点流程按 `skills/td-progress-audit`。真值分层：需求状态以 Workflow（lumiogamesengine）为准；代码以 **已推送 origin 的提交** 为准；测试以 **本机实跑输出** 为准，`cargo check` 与「已通过」声称不算。
> 本文按仓逐节推进；尚未盘到的仓标「待盘」。全部盘完后补执行摘要与 wave 编排。

## 1. 执行摘要（待全部仓盘完后补）

| 仓 | origin/main | 阶段判定（一句话） | 状态 |
| --- | --- | --- | --- |
| LumioNativeCore | `70b9834`（2026-09-03） | 地基打完、门全绿、只有定时器被真用上；其余模块无消费者，整仓仍绑着已退役的 Baseline 契约；唯一未完成是 3 张与 ADR-064 脱节的 GAS 卡 | 已盘（§2.1，已合并 Codex 会话报告） |
| LumioVoxelEngine | `e5c056e`（2026-09-05） | 服务端半边（编码 / 存储 / 事务 / 派发 / 读 / 物理 / 绑定 / pin）已合入 origin 并经 SDK 14 槽可达，但没有一个上层消费者；15 张蓝图卡的 Workflow 回写落后于代码；整仓仍绑着 V1.4 合同制，活代码与 SDK 都在用它的死名字；27 张 V1.4 旧卡待作废 | 已盘（§2.2） |
| LumioGameRuntime | `89a7a6c`（2026-09-05） | 门全绿（12 测试工程 662 项）；ECS 单世界 / 复制 / A2 控制 / 双 Transform 组件已入主干；但 13 相 Tick 没接上 `WorldManager`（两条 Tick 路径并存）、ADR-064 GAS 切片面 0%、体素端口仍 internal；旧合同制重灾区（53 个活源文件绑 V1.4 生成物）；RM-00014 被 A0 契约冻结前置锁死、一张能派的功能卡都没有；20 张旧蓝图卡已作废，R-00461 直跳 done 待纠偏 | 已盘（§2.3，两会话独立盘后合并） |
| LumioServer | `4c7688b`（2026-09-04） | 聊天切片形态的 Rust 宿主只在 Windows 能链接（两份手写 Win32 加载器），自驱主循环只在未合入的 PR #33；三张卡重叠抢「修 macOS」；C# mvp-host 冻结但 verify-all 在 macOS 红；整仓被 V1.4 合同制包裹（13 个空模块骨架、6.8k 行守卫、已退役仓 git 依赖）；RM-00006 52 张旧卡已作废，R-00478 卡面已按 D21 / D22 重写（11 条验收项）待派工 | 已盘（§2.4）；D15 / D21–D24 已裁并执行 |
| LumioClient | `c3825a4`（2026-09-05） | Foundation 骨架 + 聊天切片客户端在 origin 成立（连接 / ReplicaWorld / Bot 宿主 / 静态页）；但 main 对真 Runtime 编不过（2 处已删的 `GrantClaim`，CI 钉旧 Runtime pin 假绿）、仍带 1,079 行手写 codec、预测只有 Foundation 空架子、表现采样为零；CL-1 证实浏览器 .NET WASM 路线可行（D3 / D27），落地点 = R-00470；Unity 只剩空壳；RM-00007 9 张旧卡已出清，新卡 R-00481 与在途 R-00408 撞 `modules/replica/**` 且 0 验收项；R-00408「验收中」但 Runtime 零推送、四仓零 PR | 已盘（§2.5，与另一会话草稿对账后合并）；D25–D29 已执行（另一会话），D30 待裁 |
| LumioGame | `5bc5afc`（2026-09-04） | 炸弹人只有契约壳（5 组件 / 11 事件 / 探针 3 条），规则代码为零；契约是照 Runtime 09-04 三个缺口反向设计的「游戏仓自备引擎」，与架构第二样板五处结构性不同；main 因上游 `TryGetSession` 漂移编不过且 CI 无构建作业（修复在 origin 分支未合入）；旧合同制残留面六站最小；RM-00008 4 张 GAS 旧卡已作废、W0 卡 R-00482 已建但 0 验收项 | 已盘（§2.6，两会话合并）；D32 已追认，D33 已裁并执行（R-00483 新建） |
| LumioConfig | — | — | 待盘 |
| LumioPlatform | — | — | 待盘 |

## 2. 各仓详情

### 2.1 LumioNativeCore（依赖图最底层 · 领域无关 Rust 内核）

> 本节合并了同日另一会话（Codex）的 `2026-09-05-nativecore-progress-audit.md` 与 `2026-09-05-nativecore-kickoff-dispatch-prompts.md`（两文件已删除，内容并入此处）。两份材料对仓内现状判断一致；对方多跑了 build / 本仓 spec-lint / spec-lint.test 并先发现 timer 文档漂移，本节采纳；对方「等架构仓发布 provider composition 契约」的下一步前提与 Living Architecture 冲突（见漂移 ①），本节改写。

#### 仓库侧事实（本机 macOS，2026-09-05，两会话各自实跑、结果一致）

| 项 | 实测 | 出处 |
| --- | --- | --- |
| origin/main = 本地 main | `70b9834`（2026-09-03，PR #7 squash），0 ahead / 0 behind；工作区只有两个未跟踪 `.DS_Store` | `git status -sb`、`git rev-list --left-right --count` |
| PR / Issue / CI | 7 个 PR 全 MERGED，0 open；0 issue；main 最近 8 次 CI 全 success | `gh pr list`、`gh issue list`、`gh run list` |
| 规模 | 10 crate + xtask；`.rs` 合计 12,202 行 = 源码 6,051 + 测试 6,101 + bench 50；`#[test]` 129 个；测试目标 150；`todo!/unimplemented!/TODO` 0 处 | `find`、`cargo test -- --list` |
| 质量门 | `cargo fmt --check`、`cargo clippy --all-targets -D warnings`、`cargo build --workspace`、`cargo test --workspace`（150 passed / 0 failed）、`cargo xtask check-dep-dag`（11 crate 合规）、`cargo xtask dump-symbols`（0 未批准符号）、本仓 `spec-lint` OK、`spec-lint.test` 13/13：**全部 exit 0** | 本机实跑 |
| 被 SDK 消费 | 架构仓 `engine/native/modules/sdk-native/Cargo.toml` 路径依赖 `lumio-kernel`、`lumio-timer`；`cargo build/test -p lumio-engine-native` 通过（6 passed） | 本机实跑，rustc 1.89 |
| 跨仓依赖 | VoxelEngine / Server / Runtime / Client / Game / Config / Platform 对 NativeCore 任一 crate **零引用** | `grep` 各仓 `Cargo.toml` / `*.csproj` |
| 分支 | 本地 5 条 `claude/*` 与远端 3 条 `feat/*` 全部已合入 main，可清理（删前确认） | `git merge-base --is-ancestor` |

#### 模块状态（规划 × 代码 × 消费 × 缺口）

| 模块 | 规划 | 代码 / 测试 | 有没有人用 | 到下一阶段缺什么 |
| --- | --- | --- | --- | --- |
| contract-types | approved / I0 | 631 行 / 14 | 只被本仓其它 crate 用 | **锚在已退役的 Root ABI bundle**（`LGE-V1.4`、ADR-040 `lumio_core.h` golden 与 digest 漂移门），见漂移 ① |
| error / capability / handle / memory / kernel-context（`lumio-kernel`） | approved / I0 | 1,410 行 / 28 | SDK 只以 `TypeId::of::<HandleKey>()` 做编译期标记，**根表没有槽位通向它们** | 没有宿主消费；跨进程装载证明为零 |
| job | approved / I0 | 585 行 / 8 | 无 | 消费者（tick 第 6 相）未开卡；缺负载曲线、Sanitizer / Miri |
| spatial | approved / I1 | 413 行 / 6 | 无 | 现为通用 AABB 查询（grid 参考 + rstar），**不是 ds-server.md M5 要的「双半径候选进 / 出对有序清单」**；缺跨平台 Benchmark |
| timer | 新增（ADR 0008） | 1,281 行 / 40 | **唯一被真用上的模块**：SDK 根表 14 个 `timer_*` 槽转发到它；Server / Client 经 SDK 消费（R-00374 / 375 / 389 done） | 仓内三处文档口径不一（漂移 ⑥） |
| native-core-ffi | approved / I0 | 824 行 / 16 | 无。按设计不导出 C 符号；`lumio_core_api` provider 表对应已退役的 `lumio_core.h`，`lumio_core_init` 槽为 `None`（源码注释「Still blocked, R-00179」） | Living Architecture 下**没有装载路径**；不是「等契约」，是「契约已退役」 |
| codec / diagnostics | pending / I1，feature-gated 默认关 | 277 / 302 行，5 / 5 | 无 | 等架构源批准公共语义；ADR 0005 前不进 ABI |

#### Workflow 对账（RM-00002 · 只读；已完成的卡只做一次机器复核，不再逐张看）

| 项 | 实测 |
| --- | --- |
| 总数 | 71 张：**68 done、3 backlog**；phase `ready`；0 工作项、0 缺陷 |
| 68 张 done 的机器复核 | 全部有含 origin 可核提交号的证据评论（68 / 68）；验收项引用的 69 处文件路径与 72 处标识符在 `70b9834` 全部存在（1 处 `modules/*/README.md` 为 glob 假阳性）；**66 / 68 验收项全 passed**，R-00007（蓝图源卡）与 R-00083 的 9 条验收项停在 `not_started`（交付有 PR #4 `e2a801e` 证据，只是验收项没跑）。**结论：done 卡可信，不再复核。** |
| 3 张未完成 | R-00302 GAS-A3 帧调度器契约、R-00308 GAS-N01 无状态求值 / Tag / 堆叠内核、R-00309 GAS-N02 帧调度器实现。均 P0、2026-08-30 建、0 评论、无负责人、正文有编码损坏（`??????`） |
| 3 张卡的前置 | 全部 backlog：架构室 R-00299 / 300 / 301（GAS-A0 / A1 / A2）、Runtime 室 R-00303 ~ 307（GAS-R1 ~ R5）。也就是说 **2026-08-30 那套 GAS 卡族（架构 4 + Runtime 5 + NativeCore 3）整体没动**，而 GAS 方向已由 ADR-064（炸弹人切片，C# 一份实现，Rust 下沉推到阶段 2）与 RM-00014 R-00468（最小 GAS 可执行纵链）接管 |
| 其它 Room | RM-00011 的 R-00352 / 372 / 386 done 且有 PR 证据；**RM-00013（炸弹人）与 RM-00014（九项必备能力）0 张 NativeCore 卡** |

#### 漂移与问题（按严重度）

① **整仓仍绑在已退役的 Baseline 制度上（结构性）。** 架构仓已按 ADR-059 / `architecture.md` §6–§7 转入 Living Architecture：Baseline、`packages/`、`tools/lumio_contract.py`、contract mirror 全部删除（本机实测不存在），唯一 ABI 真值是 `engine/abi/native-abi.json`。NativeCore 这边：README「架构基线 `LGE-V1.4-2026-08-27`」；`docs/architecture/` 保留 5 版架构正文 + `abi/` bundle 镜像（276 KB）；`lumio-contract-types` 的 golden / 漂移门锚在退役 bundle 的 digest 上；CI `readme` job 断言基线字符串与镜像 sha256；`lumio-native-ffi` 的 provider 表对应退役的 `lumio_core.h`。`architecture.md` §7 第 4 条「活动源码和 CI 不再依赖 LumioCoreEngine、Baselines 或 contract mirror」是迁移完成条件，**NativeCore 目前不满足**。另：R4 整体审查（2026-09-04）称「旧仓名 NativeCore 零命中」不准确，`git grep -l LumioGameEngineArchitecture` 命中 20 个已跟踪文件，`CoreEngine` 命中 16 个。

② **库存与消费严重不对称。** 约 6,000 行内核里只有 timer（约 1,300 行）有真实消费链；handle / memory / job / kernel-context / spatial 五块地基没有任何宿主能摸到。这不是代码质量问题（门全绿），是**没有需求方**：两个在途 Room 都没给 NativeCore 排卡；设计里唯一的预期消费者是 ds-server.md M5 空间粗筛内核经 tick 第 6 相 `NativeJobBarrier` 收回，但无卡。

③ **3 张 GAS 卡与现行设计脱节。** 卡按 2026-08-30 裁决（板 11a：帧调度器落 NativeCore）写成，要新建 `lumio-gas-scheduler` / `lumio-gas-eval` crate；两天后 `lumio-timer` tickFrame 模式已提供「按 Tick 推进、每帧一次 drain 到期清单」，与「帧调度器批量取件」职责重叠；ADR-064 把 Rust 下沉定为阶段 2，gas.md 明说 0-6 帧调度器不在炸弹人切片。按「如无必要勿增实体」，重启前必须先回答：M9 帧调度器是不是 lumio-timer tickFrame 的扩展？

④ **Workflow 状态欠账（轻）。** R-00007、R-00083 共 9 条验收项 `not_started`。

⑤ **卫生项（轻）。** 5 本地 + 3 远端已合入分支可清理；3 张 GAS 卡正文编码损坏，重派前要重写。

⑥ **timer 归属口径三处不一致（轻，纯文档；Codex 会话先发现）。** 事实：内核 `lumio-timer` 在 NativeCore，C ABI 插头 `timer_*` 在架构仓 `engine/native/modules/sdk-native/src/timer.rs`，经 `native-abi.json` 到达托管侧（ADR 0008 修订记录 / ADR-057 第 9 条）。仓内：`modules/timer/README.md` 与 `docs/specs/native-core-module-map.md` 仍写 ADR 0007 时代的「不进 native-abi.json、不导出」；`.spec/knowledge/standards/repository-architecture.md` 写成「本仓……经 `native-abi.json` 的 `timer_*` 槽导出」。三处应统一成同一句话，否则下游会再造一份 timer FFI 副本。

⑦ **发布硬化不足（P2，Codex 会话提出）。** CI Native job 只跑 Ubuntu，无 Windows / macOS 产物与 ABI 装载证据；无 Sanitizer / Miri / SBOM / 可复现构建。按 `architecture.md` §6 这些属正式硬化阶段，预上线不开卡，只登记。

#### 近期架构演进对 NativeCore 的影响（逐项核对）

| 演进项 | 对 NativeCore 的含义 | 现状 | 需要的动作 |
| --- | --- | --- | --- |
| ADR-059 CoreEngine 退役 + Living Architecture（ABI 唯一真值 `native-abi.json`） | Root ABI bundle / Baseline / mirror 整套制度作废 | 仓内残留见漂移 ① | **W0 卫生卡**：删镜像与 CI 基线断言；`contract-types` 改锚 `native-abi.json` 或删掉 golden 门；`lumio-native-ffi` 退役 provider 表删除；README 按 Living Architecture 重写 |
| ADR-056 §7 / ADR-057 第 9 条：单一定时内核，插头归架构仓 | timer 只留内核 | 已落地（R-00372 / R-00386） | 三处文档统一（并入 W0） |
| ADR-064 炸弹人 GAS：C# 一份实现，Rust 下沉 = 阶段 2 | 3 张 GAS 卡失去当前需求方 | 3 张 backlog，前置全 backlog | **挂起**；重启时先裁「帧调度器 = lumio-timer 扩展 or 新 crate」，再重写卡面 |
| ds-server.md M5 空间粗筛内核 + tick.md 第 6 相 `NativeJobBarrier` | NativeCore 下一个真实消费者：候选进 / 出对有序清单、逐字节确定性、双端复用 | 现 `lumio-spatial` 是通用 AABB 查询，形状不对；`lumio-job` 是它的运载体，无卡 | 待 DS 视野排期时开 **契约卡 + 实现卡**；不在炸弹人 Stage 0（19×19、8 Bot 不需要 AOI） |
| ADR-063「爆炸传播下沉 Rust」被否；ADR-065 F05：spatial 只证明数值类型 | 炸弹人不向 NativeCore 要新东西 | — | 无 |
| ADR-062 体素物理查询 C 签名进 `native-abi.json` | 同一根表的 VoxelEngine 槽位，不是 NativeCore 的 | 槽数 0，待 VoxelEngine 开卡 | 无（只提醒：根表扩展只追加不插入） |
| CL-1 WASM 调研（LumioClient PR #18）：浏览器客户端用 .NET browser-wasm 跑 Runtime，**不装 Native 库** | 「单一定时内核」「空间粗筛双端复用」在浏览器端不成立；帧驱动靠浏览器定时器 | 调研未触及 NativeCore | **需 Owner 裁决 D3**：浏览器端允许 C# / JS 兜底（承认「单一内核」只在有 Native 的宿主成立），还是 NativeCore 出 wasm32 目标（`lumio-job` 依赖 crossbeam 线程，wasm 默认无线程，成本不低） |
| LumioServer Rust 宿主 timer ABI loader Windows-only（`native_timer.rs` `cfg(not(windows))` BLOCKED） | 影响 NativeCore 在 mac / Linux 的端到端证明，但归 LumioServer | NativeCore 自身 mac 构建测试全绿 | 记入 Server 仓盘点，不在本仓开卡 |
| `architecture.md` §6 预上线质量边界 | Production Hardening 推到正式硬化 | 未开始 | 不开卡，只登记（漂移 ⑦） |

#### 阶段判定

按仓自己的路线图：**Architecture Gate ✔ → Foundation ✔（9 模块全落地）→ NativeHeadless 半程**（spatial 有了但形状待改；codec / diagnostics 仍是默认关闭原型；「CoreEngine 包加载」路线随 CoreEngine 退役作废，改为 SDK 路径依赖）**→ Production Hardening 未开始**。

完成度口径（沿用 Codex 会话的两分法，数值为判断非测量）：**底层算法 / 生命周期约 70–80%**；**可被上层直接用上的部分约 10–15%**（10 个模块里只有 timer 通向宿主；对方给的 35–45% 把「等契约发布就能装载」算了进去，而该契约已退役，故下调）。

放到全项目里看：NativeCore 是「**一座打好地基、只亮了一间房的仓库**」，对当前切片不构成阻塞，也没有在途需求；下一阶段完全取决于上层何时提出真实消费（DS M5 空间粗筛、GAS 阶段 2），不取决于它自己再堆代码。

#### 剩下的单子怎么补（RM-00002 · 待 Owner 裁决后落卡，新建卡须逐次授权）

| # | 卡 | 性质 | 内容要点 | 前置 |
| --- | --- | --- | --- | --- |
| N-W0 | 退出旧合同制清理（**已建：R-00473**，8 条验收项） | 小卡，纯本仓 | 删 `docs/architecture/` 镜像与 `.baseline.sha256`；删 CI `readme` job 基线断言；`lumio-contract-types` 去掉 Root ABI bundle golden / 漂移门，改锚 `native-abi.json` 或只保留本仓内部类型；删 `lumio-native-ffi` 退役 provider 表（保留 panic 边界 / 句柄校验若仍有用，否则整 crate 删）；README、`.spec` 与模块 README 去掉 `LGE-V1.4` / `LumioGameEngineArchitecture` / CoreEngine 口径；timer 三处文档统一 | 无；D1 裁决 |
| N-GAS | R-00302 / R-00308 / R-00309 | 既有 3 张 | **已作废（已否决）并各补一条评论**：前提失效、重开条件 = GAS 阶段 2 且 benchmark 证明需要下沉 | D2 已裁 |
| N-M5 | 空间粗筛内核契约卡 + 实现卡（**新建，暂不派**） | 契约先行 | 契约：`(viewer, target, enter|leave)` 有序清单、排序键、双半径、确定性义务、在第 6 相收回、根表槽位形状；实现：`lumio-spatial` 改形 + `lumio-job` 运载 | DS 视野排期；ECS 视野表真值就位 |
| N-ACC | R-00007 / R-00083 验收项补记 | 写操作 | 9 条 `not_started` → 按 PR #3 / #4 证据补跑或补记 | 写授权 |
| — | 分支清理 | 卫生 | **已完成**：远端 3 条 `feat/*` 删除（`git push origin --delete`）；本地 `claude/*` 与 `.claude/worktrees` 复核时已不存在，读回只剩 `main` / `origin/main` 与单一 worktree | 已确认执行 |

不开的卡及原因：Production Hardening（§6 推到正式硬化）；wasm32 目标（等 D3）；codec / diagnostics 转正（等架构源批准公共语义）。

### 2.2 LumioVoxelEngine（体素世界 · 唯一 Rust 实现）

> 盘点当天另一会话（体素落地总指挥）刚把蓝图 `voxel-impl-2026-09-04` 的 15 张卡合入两仓 origin/main（VoxelEngine `e5c056e`、架构仓 `4d6d2c3`），Workflow 回写还没跟上。本节以两仓 **origin HEAD** 为准：VoxelEngine 本地已 `git pull --ff-only` 到 `e5c056e`；架构仓本地 main 领先 origin 2 个未推送提交（NativeCore 盘点 `4a5f596` / `94a2540`）、落后 59 个，不能快进，故架构仓 origin 用只读快照（`git archive origin/main`）核验，本地 main 未动。

#### 仓库侧事实（本机 macOS，2026-09-05 实跑）

| 项 | 实测 | 出处 |
| --- | --- | --- |
| origin/main = 本地 main | `e5c056e`（2026-09-05，「merge: integrate R-00434 voxel public layer」），0 ahead / 0 behind，工作区干净 | `git status -sb`、`git rev-list --left-right --count` |
| 最近 5 个提交 | R-00441 键契约（`58746f1` → `d76b9ab`）；「integrate reviewed voxel implementation batch」`10bf536`（+6,297 行实现、13 个新测试文件，一次性覆盖 I-2 ~ I-11 全部实现卡）；「ratify block resolution and catalog precedence」`8c88efd`（ADR-066 落地）；`e5c056e` 合入。**后两批直接推到 main，没有 PR**（PR 列表停在 #15，2026-09-04） | `git log`、`gh pr list` |
| PR / Issue / CI | 15 PR 全 MERGED、0 open；0 issue；main 最近 8 次 CI success（含 `e5c056e`） | `gh` |
| 规模 | 7 crate；`.rs` 源码 26,900 行 + 测试 15,570 行；`#[test]` 392 个；`todo!/unimplemented!/TODO` 0 | `find`、`grep` |
| 质量门 | `cargo fmt --check`、`clippy --all-targets --all-features -D warnings`、`check --no-default-features`、`build`、`check-crate-dag`（7 crate）、`check-generated-clean`、本仓 spec-lint：**全部 exit 0**；`cargo test --workspace --all-features --no-fail-fast`：**388 passed / 1 failed**。唯一失败 `vendored_copy_matches_upstream_when_available`——它把仓内契约副本与**同级架构仓工作区**逐字节比对，而本机架构仓 main 落后 origin；用 `LUMIO_ENGINE_WIRE_DIR` 指向架构仓 origin 快照后 12/12 通过。**不是仓的问题，是本机架构仓没拉** | 本机实跑，rustc 1.98.0（`rust-toolchain.toml` 与 CI 同钉） |
| 契约副本 | `crates/lumio-voxel-contracts/wire/voxel-world-v1.json` SHA-256 `56d555fd…` **= 架构仓 origin 同名文件**（52 错误码 / 57 规则 / 53 + 57 用例）；仓内 `CONTRACT_SHA256` 常量与副本一致，一致性测试逐字段断言。**是消费活契约，不是又复印一份**。架构仓本地 main 那份仍是 51 / 56（差 ADR-066 加的 `unregistered_block_type` 与规则 `blockType.resolution-domain`），拉 origin 即齐 | `shasum`、`node` 对比 |
| 被 SDK 消费 | 架构仓 origin `sdk-native/Cargo.toml` 路径依赖 4 个 crate（world / domain / ops / contracts）；`sdk-native/src/voxel.rs` 1,486 行把根表体素槽转发到这些 crate；`native-abi.json` `root.fields` 35 个，其中体素 14 个（`block_read_cell / box / column`、`block_write_prepare / commit / abort`、`section_revision_query`、`residency_pin_declare / release / status`、`raycast / sweep / overlap`）；托管侧 `VoxelFacade.cs` 779 行 + 测试 10 个。origin 快照上 `cargo build / test -p lumio-engine-native` 通过（`root_api` 14 passed）、`verify-wire` 30/30、`generate-abi.test` 19/19、spec-lint OK | 本机实跑（快照） |
| 跨仓依赖 | Runtime / Server / Client / Game / Config / Platform / NativeCore 对 VoxelEngine crate **零引用**。Runtime / Server / Client 各自仍持有 V1.4 生成物复印件，体素代码里 `VoxelChunkResidency` 72 处、`ChunkRevisionSet` 45 处、`VoxelChunkPage` 18 处，`Section` 0 处——**Section 改名一处都没到消费方**。LumioGame 体素相关代码 0 个文件（炸弹人地形只在 ADR 0019 与本地草稿卡里） | `grep` 各仓 |
| 分支 / worktree | 本地 16 条分支（14 `claude/*` + `fix/vox-d-001-004-post-sha256-retest` + `test/r-00290-sha256-kat`）全部已合入 main；远端只有 main；`.claude/worktrees/` 下 3 个 detached worktree 指向的提交都已在 main | `git merge-base --is-ancestor` |

#### 模块状态（`voxel.md` 模块图 × 代码 × 卡 × 消费）

| 模块 | 代码（origin `e5c056e`） | 卡 | 有没有人用 | 到下一阶段缺什么 |
| --- | --- | --- | --- | --- |
| M1 分层与方块编码 + M1a 目录 | `domain/block.rs` 877 行（BlockId 位段 / 段表 / 材质类表 / 行为模板 / 目录校验 / `cellOffset` 唯一算式）、`key.rs` 326 行；测试 42 + 12 + 9 + 22 | R-00434（实现中）、R-00441（已完成） | SDK `voxel.rs` | 无；ADR-066 三条裁决已进契约与代码 |
| M2 三态存储 | `section/block_storage.rs` 353 + `block_payload.rs` 329；测试 10 + 16 | R-00435（评审中） | SDK | 无 |
| M3 光照 | **0** | 无（R-00433 列为非目标） | — | 客户端要画地形时立项 |
| M4 网格生成与零拷贝 | **0**（`lumio-voxel-project` 里只有 physics_query） | 无（非目标） | — | 同上 |
| M5 改动层与派发 | `section/dispatch.rs` 120 + `modification_layer.rs` 115 + `delta.rs` 92；测试 11 | R-00436、R-00458（评审中） | SDK 未暴露派发面 | 派发到 DS 的接线在 Server 仓 |
| M6 权威写入与事务 | `ops/mutation/*`（结构化条目、prepare / commit、幂等回执、原子发布）；测试 6 + 既有 | R-00438（评审中）；旧 R-00096 / R-00104（评审中） | SDK `block_write_*` | 无 |
| M6a 方块与实体绑定 | `domain/binding.rs` 878；测试 7 | R-00447（评审中） | 无（`NetEntityId` 接线归 Runtime） | 跨域同提交点要 Runtime 把 `IVoxelWorldPort` 公开（R-00469） |
| M7 物理检测 | `project/physics_query.rs` 1,209；测试 16 | R-00448（评审中） | **无**——根表 `raycast / sweep / overlap` 三槽只有声明，`voxel.rs` 没有路由（R-00443 按卡边界 declaration-only） | 一张「路由物理槽」卡（架构仓） |
| M7a 批量读 | `ops/query/block_read.rs` 1,133；测试 6 | R-00437（评审中） | SDK `block_read_*` + C# `VoxelFacade` | Runtime / Game 零消费（R-00469 未派） |
| M8 驻留 / pin | `world/residency.rs` 865；测试 9 + 7 | R-00440、R-00452（评审中） | SDK `residency_pin_*` | 真·按玩家位置的流式加载无卡；炸弹人整图 pin 不需要 |
| M9 存档与恢复 | 只有 V1.4 时代的 snapshot / restore 脊柱（capture、shadow root、restore） | 旧 R-00134 / R-00136（评审中，QA 不通过） | 无 | 非目标；新口径（体素与实体成组原子激活）未开卡 |
| M10 离线检查器 | 0 | 无（非目标） | — | — |

#### Workflow 对账（RM-00003 + 蓝图 `voxel-impl-2026-09-04` · 只读；done 卡只做一次机器复核）

| 项 | 实测 |
| --- | --- |
| RM-00003 总数 | 55 张：28 done、**13「评审中」（`in_review`）**、14 backlog；0 实现中 / 验收中。`in_review` 在本项目状态机里是**「评审中」——需求池之后、已评审 / 实现中之前的早期状态**（transitions 实查：需求池 → 评审中 → 已评审 → 实现中 → 验收中 → 已完成，另有已否决），不是「已开工」 |
| 28 张 done 的机器复核 | 28 / 28 评论里都有 origin 可核提交号；验收项与评论引用的 91 处路径 76 处在 origin HEAD 存在，15 处缺失全部是 ADR 0013 改名前的旧路径（`src/chunk/` → `src/section/`、`crates/lumio-voxel-persistence`、`tools/lumio_contract.py` 等）；**13 张 done 卡的验收项没跑**（12 张 × 4 条 `not_started`，R-00203 1 条 `failed`），R-00264 / R-00290 0 验收项。**结论：done 卡交付可信；验收项欠账是 V1.4 旧制度遗留，不再复核** |
| 13 张「评审中」 | 全是 2026-08-27 建的 **V1.4 框架卡**：R-00002 原始需求、R-00066 配置快照、R-00068 OriginToken、R-00070 Revision 分配器、R-00076 Staged Delta、R-00078 PublishedState Root、R-00080 查询计划器、R-00096 Prepare、R-00104 Commit、R-00116 World 生命周期、R-00134 快照、R-00136 恢复、R-00142 Port 适配。每张都有 8 月 28 日的交付评论（提交号在 origin）和 **8 月 29 日独立 QA「不通过」评论**（验收项无载体 / 自证循环），此后无人再动；52 条验收项全 `not_started`。它们的代码今天仍是仓的脊柱（revision / publication / mutation / query / snapshot / restore / world / port，约 2 万行），9 月 5 日的新批次直接叠在上面 |
| 14 张 backlog | 同一批 V1.4 卡，全部「未开工」：Streaming ×3（R-00151 / 153 / 155）、Spatial ×2（R-00163 / 166）、Migration ×2（R-00169 / 170）、Project ×1（R-00182）、Mesh / Collision Source ×2（R-00193 / 194）、测试 ×2（R-00196 / 198）、QA 发布门 ×2（R-00204 / 208）。卡面按 V1.4 的 Demand / Ticket 流式、Revision-scoped Source、生成 Manifest 迁移、LocalEmbedded 双树写；新模块图的对应物（M8 流式、M4 网格、M9 转档）形状都不一样；Spatial 两张与「不为地形建空间划分树」直接冲突 |
| 蓝图 15 张 + 来源卡 | **27 张卡没有 Room**（`roomId` 为空）：R-00432（已否决）、R-00433（backlog，来源卡）、15 张活卡、10 张 9 月 4 日重复建出来的已否决卡（R-00442 / 444 / 446 / 449 / 450 / 451 / 453 / 454 / 455 / 457）。活卡状态：R-00441 已完成（6 / 6 passed）；R-00434 实现中；R-00439 验收中；其余 12 张（435 / 436 / 437 / 438 / 440 / 443 / 445 / 447 / 448 / 452 / 456 / 458）**停在「评审中」**，但每张都有 9 月 5 日 12:31–12:36 的「Batch final independent review PASS」评论，引用的是**本地** main（VoxelEngine `d76b9ab` + 补丁 SHA；架构仓 `766e1ae`）并注明「未创建新 commit / push」——这些补丁现已在 origin（`10bf536` / `8c88efd` / `e5c056e`；架构仓 `4d6d2c3`）。**93 条验收项全 `not_started`**（Voxel 66、架构 27）。R-00434 只有 9 月 5 日 01:33 的深审 RETURN 评论（三条契约冲突），之后 ADR-066 裁决落地、代码合入，卡上没有后续评论 |
| 其它 Room | RM-00013 R-00427（LumioGame `ITerrainStore` 内存版，backlog，P0）；RM-00014 R-00469（Runtime 消费体素批量读写 + 跨域提交接线，backlog）——这两张是「体素真后端接到游戏」的全部剩余路径 |

#### 漂移与问题（按严重度）

① **仓库领先 Workflow：15 张蓝图卡的代码已在两仓 origin，卡还停在「评审中」。** 事实链：9 月 5 日 12:3x 独立复审 PASS（引用本地 main）→ 14:55 前后推到 VoxelEngine origin（`10bf536` / `8c88efd` / `e5c056e`）与架构仓 origin（`4d6d2c3`）→ 卡未流转、评论没有 origin 提交号、93 条验收项没跑。解铃条件：归 Room → 评审中 → 实现中 → 验收中 → 评论补 origin 提交号 → QA 逐条跑验收项 → 已完成。在此之前不得向已完成流转。

② **整仓仍绑在已退役的 V1.4 合同制上（结构性，与 NativeCore 漂移 ① 同款，且深一层）。** 复印件与门：README「架构基线 `LGE-V1.4-2026-08-27` / 唯一架构源 `LumioGameEngineArchitecture` / `python3 tools/lumio_contract.py validate`」；CI `readme` job grep 基线字符串并 `sha256sum -c docs/architecture/.baseline.sha256`；`docs/architecture/` 6 版正文（152 KB）+ `docs/LumioVoxelEngine_Framework_Design_LGE-V1.3/`（264 KB）+ `docs/plans/lve-v1.4-implementation-blueprint.md` + `docs/evidence/decision-gates/VOX-D-001~008`；`crates/lumio-voxel-contracts/generated/`（420 KB：`lumio_core.h`、`RootAbi.cs`、6 个 C# 生成目录、descriptors）由 `tools/architecture/generated-lock.json` + `check-generated-clean` 锁住并进 CI；`legacy_baseline.rs` 保留 `voxel-chunk-page` / `VoxelChunkResidency` 两个旧 id；`.spec/AGENTS.md` 收口门槛仍写「公共契约变更必须在 `LumioGameEngineArchitecture` 通过 `lumio_contract.py`」；`modules/README.md` 10 模块图是旧图（mesh-collision / migration / spatial / streaming …）；`lumio-voxel-migration` crate 5 行空壳。机器计数：`LGE-V1` 118 个已跟踪文件、`LumioGameEngineArchitecture` 66、`CoreEngine` 21、`Root ABI` 11。**比 NativeCore 深的一层在活代码里**：33 个源文件用 `Generated*` 类型（`GeneratedVoxelConfig` / `GeneratedRevisionStamp` / `GeneratedVoxelWorldPortAdapter` / `GeneratedVoxelQueryRequest` …）、21 个用 `STABLE_ERROR_IDS`（第二套错误 id 命名空间）、8 个用 `BASELINE_ID` / `SCHEMA_EPOCH`、6 个用 VOX-D 决策门常量；test-support 9,657 行里 b0 / b2 / mvp / reference / fixture_runner 全是 V1.4 fixture 骨架；**架构仓 `sdk-native/voxel.rs` 也直接引用这些**（`GeneratedVoxelWorldPortAdapter` 5 处、`P0_DECISION_GATES` 3 处、`BASELINE_ID` / `SCHEMA_EPOCH` 各 2 处）——SDK 根表正在把 V1.4 的死名字往托管侧递。`architecture.md` §7 第 4 条的迁移完成条件本仓同样不满足。

③ **27 张 V1.4 旧卡无人认领。** 13 张 QA 不通过后停在评审中 8 天；14 张 backlog 卡面与新模块图形状不同。继续挂着会让「RM-00003 还有 27 张没做」这个数字一直是假的。

④ **消费方一个都没接上，且拿着死复印件。** Runtime `IVoxelWorldPort` 仍 `internal`（`TxnPrepareCoordinator.cs:73`，只有 Prepare / Commit / Abort / Query / ReadRevision 五个跨域事务方法，没有读写方块）；Runtime / Server / Client 三仓生成物复印件里 `VoxelChunk*` 旧名 135 处。今天体素唯一的消费者是架构仓 SDK 聚合层（14 槽 + C# `VoxelFacade`），托管侧再往上零消费。消费方清理归各仓那一站。

⑤ **物理三槽只声明不路由。** R-00443 按卡边界 declaration-only，R-00456 的 A-2 只路由了读 / 写 / revision / pin，`raycast / sweep / overlap` 没接到 `physics_query`；炸弹人 Stage 0 用不到物理查询，但 ABI 面「有槽无实现」会误导消费方。

⑥ **契约三处已知缺陷未修**（`reviews/2026-09-04-voxel-card-contract-drift.md` §6 第 1–3 条：rule 49 `onViolation` 错位、全量编码携带 `baseSectionRevision` 无错误码、pin 预算无常量）；R-00440 验收项 2「超出驻留预算的 pin 当场失败」因此没有机器可判的界。

⑦ **Workflow 卫生（轻）。** 27 张卡无 Room；10 张重复建出来的已否决卡；R-00433 正文停在「44 / 49 / 98」与旧轨道（缺 I-6 ~ I-11、A-2 ~ A-4）；R-00441 收口评论中文全部是 `??????`（Windows 侧编码损坏）。

⑧ **仓库卫生（轻）。** 16 本地已合入分支 + 3 个 stale worktree；最近两批合入没走 PR（审查轨迹只在 Workflow 评论与架构仓 `reviews/2026-09-05-r-00439-*.md`）。

⑨ **架构仓侧顺带发现（不归本站开卡，只登记）。** origin main 根目录被 RM-00011 会话提交了工作文件：`R-00406.json`、`baseline-binding.json`、`baseline-gameplay.json`、`.wf-report-R-0035x*.md` × 8、`.wf-evidence-r00357.txt`、`.wf-selfcheck-r00357.mjs`、`r5-01-fix-request.md`、`r5-01-review-findings.md`、`.sdd-scratch/`；`lumio-clr-host` 在 `cargo clippy -D warnings` 下有一个 dead-code 常量（`HDT_LOAD_ASSEMBLY_AND_GET_FUNCTION_POINTER`，本地 main 与 origin 同，不在本仓收口门槛内）；本机架构仓 main 领先 origin 2 个未推送提交、落后 59。

#### 近期架构演进对 VoxelEngine 的影响（逐项核对）

| 演进项 | 对 VoxelEngine 的含义 | 现状 | 需要的动作 |
| --- | --- | --- | --- |
| ADR-062 文末「明确不冻结的」三行（9 月 4 日口径） | ABI 体素 slot = 0 / SDK 只有编译期标记 / 实现面全零 | **三行全部过时**：14 槽已进 `native-abi.json`（origin）；SDK 4 crate 路径依赖 + `voxel.rs` 1,486 行 + `VoxelFacade.cs` 779 行；实现面 6,297 行 | 改写缺口表；Draft → Accepted 的条件（M1 / M2 实现验证）已满足，等 93 条验收项跑完再转 |
| ADR-066（R-00434 三条 Owner 裁决） | 哨兵 0..3、4..255 不可解析、目录校验结构优先 | 契约、ADR-062、`voxel.md`、VoxelEngine `block.rs`、SDK 已同步（origin） | 无 |
| ADR-063 世界模型 / `tick.md` 第 5、8 相 | 体素是参与者不是协调者；帧初读、帧末一批写、同帧多批合一 | 体素侧 prepare / commit 两段 + `expectedSectionRevision` + 幂等回执在；协调者（Runtime）侧 `IVoxelWorldPort` internal、没有 Tick 接线 | R-00469（RM-00014）负责，未派 |
| 炸弹人地形接 Voxel 真后端 | LumioGame `ITerrainStore` → 换 Voxel 实现 | LumioGame ADR 0019 已把口径对齐（y 竖直、BlockId u32、blockRead / blockWrite 形状、九种方块进官方段）；代码 0 行；R-00427 内存版 backlog；R-00469 backlog | 顺序：R-00427（内存版，Game 可先跑）→ R-00469（Runtime 经 SDK `VoxelFacade` 消费）→ Game 换实现；本仓无新卡 |
| Section / Chunk 改名 | 本仓 ✓（ADR 0013）；架构仓 ✓；消费方 ✗（死复印件） | — | 各仓那一站的 W0 卡 |
| 体素与 NativeCore spatial 边界（D4） | 体素派生碰撞归本仓（`physics_query` ✓），实体间粗筛归 NativeCore M5 | 一致；本仓对 NativeCore 零引用 | 无；RM-00003 两张 Spatial 旧卡与此冲突 → 作废 |
| D3（浏览器端没有 Native） | `voxel.md` 写「客户端体素 Rust 编成 WASM」；炸弹人 Stage 0 体素不进预测世界、客户端只按 Delta 重画 | 7 个 crate `#![forbid(unsafe_code)]`、活代码无线程 / 文件 / 网络依赖（test-support 除外）——wasm32 目标技术上便宜 | 不开卡，登记；客户端要画地形（M3 / M4）时一起立项 |
| 已裁原则 ①③（唯一最干净版本、如无必要勿增实体） | 本仓 26,900 行里约 2 万行是 V1.4 制度的脊柱与 fixture 骨架，新实现叠在上面；错误 id 有两套命名空间 | — | D8 |

#### 阶段判定

按 `voxel.md` §5：阶段 0「先立规矩」8 张里 7 张有代码（0-5 存档自描述是非目标）；阶段 1 垂直切片 6 步里**服务端半边**（挖一格 → prepare / commit → Delta；批量读；DDA 三种检测；箱子绑定；改动层派发）有代码，**光照、网格、客户端画面三步为零**，所以「老王挖一格土，另一个玩家看见」这条主线今天在任何宿主里都跑不通；阶段 2 / 3 未开始。

完成度口径（判断非测量）：**服务端算法与存储约 60–70%**；**可被上层用上的部分：经 SDK 根表 14 槽可达，但 Runtime / Game 零消费，端到端 0%**。放到全项目里看：VoxelEngine 是「**货已经上架、柜台（SDK）也开了、还没有一个顾客走进来**」的仓库；下一步都在别的仓（R-00469 Runtime、R-00427 Game），本仓自己剩的是把旧制度的架子拆掉。

#### 剩下的单子怎么补（RM-00003 / RM-00001 · Owner 2026-09-06 一次性授权，已落卡并逐笔读回）

| # | 卡 | 性质 | 内容要点 | 前置 |
| --- | --- | --- | --- | --- |
| V-SYNC | 15 张蓝图卡回写（既有卡） | 写操作 | 归 Room（Voxel 卡 → RM-00003，架构卡 R-00439 / 443 / 445 / 456 → RM-00001）；评审中 → 实现中 → 验收中；每张一条评论补 origin 提交号；R-00434 → 验收中 | **已执行**：13 张验收中 + 收口评论（9 月 5 日）；归 Room 16 张（9 月 6 日，PATCH 200 × 16 读回一致） |
| V-QA | 93 条验收项跑批（既有卡） | QA 提示词 | 独立环境按验收项逐条实跑，通过才 → 已完成；提示词见 `plans/2026-09-05-voxelengine-w0-card-and-kickoff.md` §三 | V-SYNC 已完成，可派（提示词 §三，含反例探针） |
| V-OLD | 27 张 V1.4 旧卡（既有卡） | 流转 | 作废（已否决）+ 每张一条取代评论（指向新卡 / 非目标 / 保留代码的测试落点） | **已执行**：27 张已否决（评论为通用理由） |
| V-W0 = **R-00474** | 退出旧合同制清理（RM-00003；2026-09-06 按三层重写正文，10 条验收项读回 10 / 10） | 一张卡三层 | ① 文档 / CI / 镜像 / 旧仓名；② `generated/` 树 + `generated-lock.json` + `check-generated-clean` + `legacy_baseline.rs` + VOX-D 门与 V1.4 fixture 骨架；③ 活代码里的 `Generated*` / `BASELINE_ID` / `SCHEMA_EPOCH` / `STABLE_ERROR_IDS` / `from_generated`，错误 id 只留契约 snake_case 一套；卡面见 `plans/2026-09-05-voxelengine-w0-card-and-kickoff.md` §一 | D8 已裁决；与 R-00476 串行 |
| A-W0 = **R-00476** | 架构仓 `sdk-native/voxel.rs` 去掉对 V1.4 名字的引用（RM-00001，5 条验收项，与 R-00474 互引） | 配套小卡 | `GeneratedVoxelWorldPortAdapter` / `P0_DECISION_GATES` / `BASELINE_ID` / `SCHEMA_EPOCH` 8 处引用改走活契约类型 | D8 已裁决；R-00474 第三层合入之后 |
| A-5 = **R-00477** | 物理三槽路由到 `physics_query`（RM-00001，7 条验收项，与 R-00443 / R-00456 互引） | 小卡 | `raycast / sweep / overlap` → `lumio-voxel-project`；`root_api` 补 3 条测试；C# facade 补三个入口 | D10 已裁决；R-00443 / R-00456 验收通过之后 |
| C-FIX = **R-00479** | 契约三处缺陷（RM-00001，7 条验收项，与 R-00440 / 436 / 438 互引） | 契约小卡 | rule 49 改挂「批被部分应用」的错误码、全量编码携带 `baseSectionRevision` 加错误码、`limits` 加 pin 预算常量；ADR-062 修订记录；VoxelEngine 复制副本 | D11 已裁决；可立即派，与其他卡无文件重叠 |
| — | R-00433 正文更正 | 评论 | 52 / 57 / 110、决策账本补 ADR-066、轨道补 I-6 ~ I-11 与 A-2 ~ A-4、后续四卡号 | **已执行**：评论 1 条（201，读回 2 条） |
| — | 分支 / worktree 清理 | 卫生 | 16 本地分支 + 3 worktree | **已执行**（2026-09-06）：先核全部已在 main 且 worktree 干净，再删 16 条本地分支、移除 3 个 stale worktree；读回只剩 `main` 与主工作区 |

不开的卡及原因：M3 光照 / M4 网格（客户端画地形时再开，届时连 wasm32 一起）；M8 真·流式加载（炸弹人整图 pin，用不上）；M9 新口径存档、M10 检查器（R-00433 非目标）；Runtime 侧 `IVoxelWorldPort` 公开与 Tick 接线（R-00469 已有）。

### 2.3 LumioGameRuntime（第三站）

#### 事实汇总（2026-09-06）

- **Git 与状态**：
  - HEAD：`89a7a6c0b885248ded32a851d15b84dce269ab1f`（`Merge branch 'Go1c/r5-a2-runtime-fix-review'`），已 `git pull --ff-only`（前移 14 个提交）。
  - 分支与 Worktree：本地领先/落后 `0 0`，主工作区 clean（0 脏文件、0 未跟踪）；保留 3 个过期 `.claude/worktrees`（`distracted-moser-154e2f`、`exciting-chaplygin-77e103`、`priceless-meitner-bb6110`）；本地分支 12 个、远端分支 15 个。
  - 规模：425 个 `.cs` 文件，源码 32,135 行（`modules/*/src`），测试 15,410 行（`modules/*/tests`），工具 1,270 行（`tools/`）；共 30 个 `.csproj`。
  - 项目依赖关系：本仓全部 30 个项目为纯内部模块间引用（`ecs`、`simulation`、`command`、`coordination`、`gas`、`replication`、`config`、`observability`、`hello`、`GeneratedContracts`），**零外部跨仓项目引用**（不直接引用 NativeCore / VoxelEngine / GameEngine，无 C# 到 Rust 的不可回退链接）。
- **门禁与测试**：
  - 规范 linter：`node .spec/tools/spec-lint.mjs && node --test .spec/tools/spec-lint.test.mjs` → **全部通过（13/13 passed, 0 failed）**。
  - 单元测试：`dotnet test` 全覆盖（12 个测试工程）→ **662 个测试全部通过（0 失败、0 跳过；原文「626」为加总笔误，第二会话逐项复跑相加为 662）**：
    - `gen-declarations.Tests`: 3 passed
    - `GeneratedContracts.Tests`: 4 passed
    - `Gas.Tests`: 57 passed
    - `Coordination.Tests`: 89 passed
    - `Config.Tests`: 41 passed
    - `Simulation.Tests`: 146 passed
    - `Ecs.Tests`: 27 passed
    - `Username.Tests`: 6 passed
    - `Observability.Tests`: 39 passed
    - `Replication.Tests`: 205 passed
    - `Hello.Tests`: 17 passed
    - `Command.Tests`: 28 passed
- **旧合同制残留（重灾区）**：
  - 机器排查：`LGE-V1` 残留 31 处，`LumioGameEngineArchitecture` 29 处，`Root ABI` 3 处，`CoreEngine` 7 处。
  - 复印件镜像：`docs/architecture/` 目录保留 `v0.3`、`v1.0`、`v1.1`、`v1.2`、`v1.3`、`v1.4` 副本（其中 `v1.4.md` 高达 47KB）与 `.baseline.sha256`。
  - CI 强绑定：`.github/workflows/repository-policy.yml` 显式 `test -s docs/architecture/LumioGameEngine_Architecture_v1.4.md`、`sha256sum -c docs/architecture/.baseline.sha256`，并在供应链闸门中 `git clone LumioGames/LumioGameEngineArchitecture`。
  - 生成物残留：`src/Lumio.GameRuntime.GeneratedContracts/` 包含基于 `LGE-V1.4-2026-08-27` 生成的 `RootAbi.cs`、`ContractRuntime.cs`、`ContractTypes.cs` 等。
- **架构演进现状**：
  - **双 Transform（ADR-065）已先于 Workflow 入库**：最新提交 `11d34e1` 已落地 `LogicTransform.cs`（577 行，单写者 Controller 门禁、米制、Fixed Scale 1.0）、`ModelTransform.cs`（189 行，客户端表现位姿采样与 Slerp/Lerp 插值、32 样本环形缓冲、Teleport 清空跳变）、`TransformTypes.cs`（139 行，`Pose`、`TransformController`、`TransformTeleportId`），测试 217 行全绿通过。
  - **ECS 单一世界（ADR-058 / ADR-056）已彻底清除旧三世界**：`IWorldStorageAdapter`、`EntitySlotTable`、`EcsWorld` 均已移除，测试显式断言 `Assert.DoesNotContain(exported, type => type.Name == "EcsWorld")`。
  - **GAS 调度机制（ADR-064）**：Runtime GAS 模块拥有 57 条测试，为纯 C# 内存状态机，**完全不依赖 Native 定时器**；完全能在浏览器 WebAssembly 环境独立运行。
  - **体素对接端口（ADR-062 / ADR-063）**：`IVoxelWorldPort` 仍为 `internal`（在 `Lumio.GameRuntime.Coordination` 内），仅有旧生成的 `GeneratedVoxelWorldPortAdapter`，尚未公开给 Game 玩法层，尚未接入新体素门面（VoxelFacade / SDK）。

#### 漂移与脱节分析

1. **仓库大幅领先 Workflow（代码已入库，Workflow 仍为 backlog）**：
   - 双 Transform 基础：RM-00014 中的 **R-00461**（`[必备-E1] 双 Transform 的 Runtime 基础：LogicTransform、单写者与 Teleport`）在仓库中已经由 commit `11d34e1` 完成并合入 `main`，但 Workflow 上状态仍为 **backlog**。
   - ECS 单一世界：RM-00011 中的 **R-00407** 停在 `acceptance`（待验收），实际上仓库早已在 `main` 删除了旧三世界并完全通过回归测试。
2. **RM-00005（Runtime 基础室）大量旧时代单据悬空**：
   - 共 40 张需求卡中，**20 张处于 backlog**，全基于已作废的 V1.4 架构蓝图（如 T13、T15、T16、T17、T22、T23、T25、T26、T28、T29、T30，以及 GAS-R1~R5 R-00303~307）。这些卡与 NativeCore 的 R-00302/308/309 属于同一批历史陈旧单据，很多已被后续的 ADR-058、ADR-064、ADR-065 和实际代码覆盖或取代。
3. **旧合同制与镜像校验强绑定，阻碍 CI 现代化**：
   - 同 NativeCore（D1）与 VoxelEngine（D8），Runtime 的 CI policy 仍然在断言 `docs/architecture/.baseline.sha256` 和旧架构仓克隆，存在「第二份真值」与多源漂移隐患。

#### 待写清单（攒齐待授权）

| 卡号 | 动作 | 类别 | 正文/范围摘要 | 依据 |
| --- | --- | --- | --- | --- |
| **R-NEW-RT** | 新建退出旧合同制清理卡（RM-00005） | 架构契约清理 | ① 删 `docs/architecture/` 全部 6 份旧 md 与 `.baseline.sha256`；② CI `repository-policy.yml` 删旧校验与克隆；③ `GeneratedContracts` 退役并收敛为现行活契约 | 同 D1、D8；Owner 原则 ① |
| **R-00461** | 流转：backlog → done | 状态对账 | 补 commit `11d34e1` 交付证据与 `TransformComponentTests` 测试通过记录 | 仓库已交付事实 |
| **R-00407** | 流转：acceptance → done | 状态对账 | 补单一世界、模板内联与删三世界测试通过记录 | 仓库已交付事实 |
| **RM-00005 旧卡** | 批量作废（~15 张旧 backlog 卡） | 清理出清 | 作废 R-00141、R-00162、R-00167、R-00174、R-00176、R-00181、R-00184、R-00187、R-00191、R-00192、R-00195、R-00197、R-00199 及 GAS-R1~R5 | 同 D2、D7 |
| — | 分支 / worktree 清理 | 卫生 | 清理本地 3 个 `.claude/worktrees` 与已合入的本地分支 | 授权确认 |

#### 本会话独立复核（2026-09-06 深夜，第二会话；先独立盘完再与上文前一会话的结论对照）

> 两个会话各自实跑，仓库侧事实一致（HEAD `89a7a6c`、0 ahead / 0 behind、spec-lint 13 / 13）。差异先说：上文「626 个测试」是加总笔误——12 个测试工程逐项相加为 **662**（3 + 4 + 57 + 89 + 41 + 146 + 27 + 6 + 39 + 205 + 17 + 28），R-00475 卡面「626+」与 R-00461 流转 reason 里的「626」同源；`.cs` 计数 425 含 `obj/` 下生成的 AssemblyInfo，排除 bin / obj / worktree 后 332 个；`dotnet format --verify-no-changes` 抽 5 个生产项目：ecs / gas / simulation 通过，**coordination（`SessionRevisionVectorView.cs` imports 顺序）与 replication（`EntityBindingQuery.cs:130` 空白）各 1 处不过**——format 不在本仓声明的门槛里、CI 也不跑。

##### 模块状态（规划 × 代码 × 消费 × 缺口）

| 模块 | 代码（origin `89a7a6c`） | 有没有人用 | 到炸弹人切片缺什么 |
| --- | --- | --- | --- |
| ecs | 7,311 行：单世界 `WorldManager`（服务器 / 客户端两条私有路径）、`Sync<T>` / `SyncList` / `SyncDict`、生成三件、128 位发号、`CreateFromSnapshot`、A2 控制消息、**双 Transform**（9-05 新入） | LumioClient replica / bot、LumioGame server-gameplay 以 ProjectReference 直引；LumioServer 反射装载 | 系统注册进第 3 / 4 相（RT-1）、`Sync<NetEntityId>` / `Scope.None` / `TickRate`（0 命中）、预测世界（`PredictedWorld` 0 命中） |
| simulation | 7,246 行：`TickPhase` 13 相枚举、`PhaseContractTable`（断言唯一提交点 = `GasAndEventFinalize`）、`TickRunner`、Ingress / Native barrier；146 项测试 | **只有自己的测试在用**：`modules/ecs` 对 `TickRunner` / `SimulationSession` 零引用 | 接上 `WorldManager.Tick()`（RT-1） |
| coordination | 9,378 行：CrossWorld 两段提交（`VoxelCommit → EcsCommandBufferCommit`）、Reservation、Revision Vector、Journal、Recovery；89 项测试 | 无真实体素参与者；`IVoxelWorldPort` internal、只有 fail-closed 替身与 V1.4 形状的 `GeneratedVoxelWorldPortAdapter` | 公开端口 + 接 SDK `VoxelFacade`（R-00469） |
| replication | 13,978 行：C-1″ codec、按观察者打包、绑定 / 查询、聊天映射；205 项测试 | Server / Client 消费 | `sequence` / `appliedInputSequence` 消费（0 命中；形状随 R-00406 已定） |
| gas | 1,585 行：TypeId / Handle / 世代号、框架六态（Unloaded…Faulted）、`IGasEcsProjectionPort`；57 项测试 | 无 | **ADR-064 全部**：八态 / 准入五步 / `Activate<T>` / Effect 单 / 两本账 / 整数求值 / `OnFx`（RT-4 / RT-5） |
| command / config / observability | 3,064 / 3,527 / 2,136 行 | 内部 | 无切片缺口 |
| hot-reload / persistence / testing | **0 行**（只有 README） | — | 不在切片 |
| GeneratedContracts | 1,920 行 V1.4 生成物（manifest 钉旧仓 `a206e2c`） | **53 个活源文件**用 `Lumio.Gen.*`：replication 13 / command 10 / coordination 9 / simulation 9 / config 5 / observability 3 / ecs 2 / gas 2；最重的是 `EntityIdentity`（21 处）与 `ProcessorDescriptorPhase`（12 处，`TickPhase` 枚举本身是它的投影） | R-00475 第三层 |

##### 五项架构核对（逐项附证据）

| # | 核对项 | 现状（代码事实） | 结论 |
| --- | --- | --- | --- |
| ① ADR-064 GAS 帧调度与到期 | `modules/gas` 只有句柄索引 + 框架状态机；README 仍写「候选接口 activate / apply_effect / tick_effects」、Baseline `LGE-V1.4`、PredictionKey / PredictionFrame（旧词 9 处）。全仓零处引用 Native 定时器；唯一时间源是 `TickExecutionContext` 的 `Stopwatch`，只做超预算判定。M9「按帧计数到期」也没有实现（切片是瞬时效果，本来不需要） | **不等 NativeCore 定时器（✓），但 ADR-064 的切片面 0% 落地**。上文「GAS 调度机制完全能在浏览器独立运行」说的是框架六态，不是 ADR-064 的 GAS |
| ② ADR-062 / 063 体素消费端 | `IVoxelWorldPort` internal（`TxnPrepareCoordinator.cs:73`），五个方法只有 Prepare / Commit / Abort / Query / ReadRevision，没有读写方块；`VoxelAdapters` 适配的是一个私有「替身接缝」`IGeneratedVoxelWorldPort`（注释明说等旧生成器发布 binding）；对 `sdk-native` / `VoxelFacade` / `native-abi.json` 零引用；coordination 里 `VoxelChunk*` 旧名 39 处 / 19 文件、`Section` 0 处 | 两段提交的骨架在，**没有一个真实体素参与者接上**；R-00469 未派 |
| ③ 双 Transform | `11d34e1` +1,209 行：Float32 `Vector3` / `Quaternion`（按 ADR-065 D04）、父子（`SetParent` 保世界位姿、结构结算时 `ResolvePendingTransformParents`、父销毁子解绑）、`TransformController` 单写者（二次注册抛、`BeginWrite` 作用域）、`Teleport` 标记、`ModelTransform` 32 样本环 / 0.1 s 外推；9 个测试。**偏差**：位置 / 旋转以**字符串**序列化走 `TransformSyncField`（`Scope.Room`、`Authority.Server`，不是类型化 `Sync<T>`）；`ModelTransform` 与 `LogicTransform` 同在 Ecs 程序集（没有 `.Client.cs` 端别拆分）；无双写者启动期校验（`[Writes]` 机制归 RT-1）；样板 `username` 没接、LumioGame 零消费 | **组件在、没接线**。另：`plans/2026-09-05-bomber-engine-runtime-cards.md` RT-2 卡面「必须整数、建议毫格」已被 ADR-065 D04（Float）取代，派活前要改卡 |
| ④ 13 相 Tick | `TickPhase` 13 相齐、`PhaseContractTable` 断言唯一提交点、`TickRunner` 146 项测试全绿；**但 `WorldManager.Tick()` 服务器路径仍是私有五步 `ApplyInputs → CommitCreates → Project → ConsumeSave → Tick++`**，`modules/ecs` 对 runner 零引用；第 5 相 Prepare / 第 8 相 Commit 只在 coordination 的事务器里、没被 Manager 调；第 10 相 Finalize 只在 runner 里；`[System(Phase)]` / `TickRate` / `Ticks.FromMilliseconds` 0 命中；`simulation/README.md:29` 仍写提交点在「`EcsCommandBufferCommit` 后、`GasAndEventFinalize` 前」，与 `PhaseContractTable` 矛盾 | **两条 Tick 路径并存**——13 相是一台没接传动轴的发动机；这正是「如无必要勿增实体」要消灭的第二份真值（RT-1 / R-00462） |
| ⑤ 浏览器 WASM | 生产代码零 `DllImport` / `NativeLibrary` / `unsafe` / `new Thread` / `Task.Run` / 文件 / Socket；`lock` 387 处、`ManagedThreadId` 21 处（owner-thread 守卫）、`Channel` 7 处，单线程 wasm 下都无害；12 个生产项目全部 `net10.0;netstandard2.1`。CL-1 实测（LumioClient `docs/spikes/2026-09-05-spike-runtime-wasm.md`）：Ecs + Replication + 样板零改动进 browser-wasm，起世界、解真实包、上行被接受、哈希与桌面逐位一致；债务 = Ecs 4 处 IL2075 反射（`World.TryReadAccountId`、`WorldManager.FindSyncField` ×2、`TryDispatchSendMessage`）让 `PublishTrimmed` 默认失败 + 无 InputCommand 信封公开编码 API | **没有对 Native 的不可回退依赖**；Runtime 是今天唯一能进浏览器的预测世界载体，D3 方案 A 成立 |

用炸弹人说这五条：玩家按放弹 → 今天 Runtime 没有 `Activate<放弹>`（①）；引信到点 → 玩法系统没有第 3 / 4 相可以注册进去跑扫描（④）；火烧到木箱 → 没有公开的体素批量读写、第 5 / 8 相没接（②）；人被炸退一格 → `LogicTransform` 能写、能插帧，但没有系统在写它（③）；这一切搬进浏览器 → 代码本身进得去（⑤）。

##### 漂移与脱节（补上文 1–3 未覆盖的）

④ **R-00461 直跳「已完成」缺证据与验收记录**：16:17Z 四次连续流转（backlog → 评审中 → 实现中 → 验收中 → 已完成，reason 引用 `11d34e1`），但卡上 **0 条评论、5 条验收项全部 `not_started`**。Owner 2026-09-06 已收签这批写入，状态不改；欠的证据评论与验收项补记见 D17。
⑤ **R-00475 流转到「实现中」但仓内没人开工**：16:27Z 两次流转（reason 编码损坏 `????`），**0 条验收项**（plans 文件写了 6 条，线上没建），仓内无分支无改动；卡面第 3 条「收敛 GeneratedContracts」没写第三层（53 个活源文件、`TickPhase` 投影、`EntityIdentity`）。
⑥ **RM-00014 整个 Room 被一张契约卡锁死**：13 张 E / C / S 卡正文统一写「执行前置 = R-00460（A0）尚待冻结，禁止把登记状态解释为可立即开工」，而 R-00460 backlog、0 评论、没人认领；`plans/2026-09-05-bomber-engine-runtime-cards.md` 的 RT-1 ~ RT-5 → R-00462 / 461+463 / 466 / 468 对齐表也没有回写进 E 卡正文。结果：Runtime 关键路径上一张能派的卡都没有。
⑦ **作废 T22 / T23 / T25 / T26 没有接手卡**：T26「串联 13 相、唯一 Commit Point」正是 ④ 要做的事；名义接手卡 R-00462 卡在 ⑥。**R-00295「复制发送调度」**（优先级 / 饥饿上限 / 截断回流 / 慢客户端阶梯，ds-server.md M6 / ADR-063 第 11 条）被作废后 RM-00014 没有任何卡承接——真孤儿。
⑧ **12 张 T-00010 ~ T-00021 工作项没动**：8-30 建的 ECS 拆解（父卡 R-00149 / 150 / 152 已 done），全部 `todo`，Room 总账显示 12 active；T-00021「LogicTransform、父子结构」与 R-00461 重复。
⑨ **A2 控制消息无卡**：`ExpireEntity / ResolveBinding / AttributeQuery`（`048d70a` … `5c94bad`，9-05）只挂在 R-00408 评论里，Runtime 室没有对应需求。
⑩ **卫生**：本会话 00:07 实测仍有 3 个 `.claude/worktrees` 与 10 条本地分支，01:00 复测前一会话 D14 已执行——worktree 只剩主工作区，本地剩 4 条非 main 分支（`ci/supply-chain-gates-admission-path` 已合入；`claude/kind-burnell-414cb8` 4 ahead、`docs/spec-baseline-v14` 1 ahead、`feat/dependency-policy-forbidden-packages` 3 ahead 都是 8-29 PR 合并前的旧 tip / PROBE 提交，远端同名 3 条同样）；远端 19 条已合入 `feat/*` 未删；PR #33 之后 16 个提交直推 main 没走 PR。

##### 阶段判定

Foundation ✔（模块骨架 + 662 项测试全绿）→ 聊天切片 ✔（R5-02 单世界 + 打包、A2 控制在 origin，三仓接入 R-00408 进行中）→ **炸弹人切片 0 / 5**（RT-1 Tick 统一未做、RT-2 组件在但没接线、RT-3 / 4 / 5 为零）。完成度口径（判断非测量）：**底层模块约 70%**；**跑一局炸弹人所需的 Runtime 能力约 15%**（有 ECS 世界 + 复制 + 双 Transform 组件，没有 Tick 接线 / GAS / 预测世界 / 体素消费）。放到全项目里看：Runtime 是关键路径上的**唯一堵点**——NativeCore 与 VoxelEngine 都在等它来消费，Client 与 Game 都在等它给系统注册和 GAS；今天它一张能派的功能卡都没有。

##### 两份审阅互相吸取（前一会话 vs 本会话）

| 项 | 前一会话（上文 §2.3 / §6 D12–D14 / §7） | 本会话 | 取谁 |
| --- | --- | --- | --- |
| 仓库事实与门禁 | HEAD / 分支 / spec-lint / 逐项目测试数一致；「626」是加总笔误 | 同；另跑 `dotnet format`（2 处不过）与依赖 / 消费方扫描 | 数字取 662；format 结果记入 R-00475 |
| 旧合同制 | 三层口径与 D1 / D8 对齐，落了 R-00475 | 量化了第三层（53 文件、`TickPhase` 投影） | R-00475 补第三层 + 验收项 |
| 架构五项 | 只核 GAS「不依赖 Native」与体素端口 internal；未核 13 相接线、双 Transform 偏差、WASM 证据 | 五项逐条附证据；发现两条 Tick 路径并存 | 本会话表为准；上文「GAS 完全能在浏览器独立运行」改读为「框架六态能，ADR-064 切片面还没写」 |
| Workflow 对账 | 核了 R-00461 / R-00407 / 20 张旧卡；执行了作废与流转（reason 写在流转上） | 补核 12 张 T 工作项、RM-00014 A0 死锁、R-00295 孤儿、A2 无卡；发现 R-00461 直跳 done 与 R-00475 空验收项 | 两边合并：作废与 R-00407 保留；R-00461 状态不动、补证据评论与验收项 |
| 速度 vs 纪律 | 快：一晚三站 + 三仓写入 | 慢：只读、逐笔核 | 下一站沿用前者的节奏，但流转前先建验收项、只到「验收中」 |

##### 剩下的单子怎么补（RM-00005 / RM-00014 · 本会话建议，全部待授权）

| # | 卡 | 性质 | 内容要点 | 前置 |
| --- | --- | --- | --- | --- |
| RT-W0 = **R-00475** | 卡面补第三层 + 建 8 条验收项（既有卡） | PATCH + 验收项 | 三层：① 镜像 / `.baseline.sha256` / CI readme job 基线断言与 supply-chain job 旧仓 checkout / README + 13 处 README「架构基线」+ `.spec/AGENTS.md` + `repository-architecture.md`；② 删 `src/Lumio.GameRuntime.GeneratedContracts` 整树 + `eng/verify-generated-contracts.*` + `generate-contracts.*` + `.spec/decisions/0002`（加「被取代」）；③ 53 个活源文件去 `Lumio.Gen.*`：`TickPhase` 改为本仓枚举、`EntityIdentity*` 改用 ecs 自有 `NetEntityId` / 生命周期类型、`TxnJournalRecord*` 改 coordination 自有类型，`modules/gas/README.md` 重写；顺手修 format 2 处；验收 = 上述 grep 零命中 + 662 项不倒退 + spec-lint + CI 绿。卡面与开工提示词见 `plans/2026-09-06-gameruntime-w0-card-and-kickoff.md` | D12 已裁；卡面待改 |
| RT-1 = **R-00462** | Tick 统一（既有卡，正文按 bomber 计划 RT-1 回写） | 评论回写 + 流转 | `WorldManager.Tick()` 只走 `TickRunner` 13 相、删私有五步；`[System(Phase)]` / `[After]` / `[Reads]` / `[Writes]` 生成注册；`WorldEntity.TickRate` + `Ticks.FromMilliseconds`；simulation README 改口 | D16（解 A0 死锁） |
| RT-2′ = **R-00461** | 状态不动（Owner 已收签 done）；补证据评论 + 5 条验收项补记（既有卡） | 评论 + 验收项 | 证据 = `11d34e1` + 本机 `Ecs.Tests` 27 / 27；接线小项（样板 `PlayerEntity` 加 `LogicTransform`、`[Writes]` 负例、类型化编码）随 RT-1 后并入 R-00463 | D17 |
| RT-4 = **R-00468** | GAS M2（既有卡，正文按 RT-4 回写） | 评论回写 | 见 bomber 计划 `card:RT-4` | RT-1 |
| RT-5 = **新建**（RM-00014） | GAS M3 / M4 / M5（Effect 单、整数求值、两本账重算、击杀跨零、`OnFx`） | 新建卡 | 正文 = bomber 计划 `card:RT-5`，前置 R-00468 | D19 已裁 |
| RT-3 = **R-00466** | 预测世界重建（既有卡） | 评论回写 | 见 bomber 计划 | RT-1、RT-4 |
| — | 12 张 T-00010 ~ T-00021 工作项 | 作废 | 父卡已 done、内容被 R5-02 覆盖、T-00021 与 R-00461 重复 | D18 |
| — | R-00295 复制发送调度 | 记账 | 内容归 Server 站视野下发排期时开卡（与 D4 M5 同批） | 无 |
| — | A2 控制消息 | 记账 | 留在 R-00408 证据链，不补卡 | 无 |
| — | 20 张 done 卡 47 条 `not_started` 验收项 | 不复核 | V1.4 遗留，同 VoxelEngine 口径 | 无 |
| — | 远端 19 条已合入 `feat/*` + 3 条旧 tip、本地 4 条非 main 分支 | 卫生 | 全删（D20 已裁）；执行时逐条 `merge-base --is-ancestor` / PR 号核对后再删 | D20 |

不开的卡：R-00475 之外不新建任何卡——RM-00014 的 E 卡已经是 RT-1 ~ RT-5 的落点，只缺回写与解锁。

### 2.4 LumioServer（第四站 · Dedicated Server Host 与网络基础设施）

> 本节由本会话按五步实测重写。同日另一会话的草稿（含 D15 与已执行的 Workflow 写入）在末尾「与另一会话草稿的对照」小节逐条对账：采纳其方向与比喻，更正其数字与根因。

#### 仓库侧事实（本机 macOS，2026-09-06 实跑）

| 项 | 实测 | 出处 |
| --- | --- | --- |
| origin/main = 本地 main | `4c7688b`（2026-09-04，`Merge branch 'docs/ecs-knowledge-sync-r2'`），`git pull --ff-only` → Already up to date，0 ahead / 0 behind；工作区 clean；本地只剩 `main`；`.claude/worktrees/` 已空（目录 mtime 2026-09-06 00:27，与另一会话的 Workflow 写入同一分钟——分支 / worktree 清理已由该会话执行并记入其 §7 条目） | `git status -sb`、`git branch -vv --all`、`git worktree list`、`ls -la .claude/worktrees` |
| 远端分支 | 盘点时 15 个 `feat/*`：13 个已合入 main；2 个未合入——`feat/r-00388-r4-02-self-drive`（`f8aef77`，PR #33 OPEN，+1,322 / −374，含 09-04 三个 `cfg-gate non-windows` 提交；CI 五项：README ✓、MVP C# ✗、Cargo acceptance windows ✓ / ubuntu ✓、11-scenario ✗）与 `feat/r-00346-admission`（只多两个 lock 文件，已被 `r-00346-lockfiles` 取代）。**收口时已删 14 条（见 §7），远端只剩 `main` 与 `feat/r-00388-r4-02-self-drive`；PR #33 已关** | `git branch -r --merged / --no-merged origin/main`、`gh pr list / close --repo LumioGames/LumioServer`、`git ls-remote --heads` |
| 语言与规模 | Rust 52 文件 22,134 行：`modules/process` 29 文件（其中 5 个测试文件 2,423 行）、`modules/host-runtime` 7 文件 1,242 行、`crates/lumio-host-testkit` 950 行、`tools/xtask` 6,844 行（V1.4 合同守卫）、`generated/` 24 行；C# 193 文件 43,870 行：`mvp-host` 40,198 行（159 文件、28 csproj）、`account-server` 2,975 行（3 csproj）、`entity-chat-host` 697 行（1 csproj） | `find … -name '*.rs'` / `'*.cs'` + `wc -l` |
| 15 个模块目录 | 只有 `process` 与 `host-runtime` 有代码；其余 13 个（auth / transport / session / world-slot / pacing / coreclr-host / release-agent / persistence-host / maintenance-agent / control-plane-adapter / observability / host-profiles / protocol-dispatch）各只剩一份 README——是已作废 47 卡蓝图的骨架 | `find modules -type f` |
| Rust 门禁 | `cargo test --workspace --locked` **exit 101**。第一道：`lumio-host-runtime` 的 `ENTRY_SYMBOL` / `GetApiV1` 在非 Windows 成 dead code，撞 `.cargo/config.toml` 的 `-D warnings`；把 dead_code 放行后第二道：**链接失败 `ld: library 'kernel32' not found`**——`sdk_loader.rs:116` 与 `native_timer.rs:90` 无条件 `#[link(name = "kernel32")]` + `LoadLibraryW` / `GetProcAddress`。结论：**main 上的 Rust 宿主只能在 Windows 编译链接**；207 个 `#[test]`（process 144 / xtask 42 / host-runtime 11 / testkit 10）在 macOS 上 0 个能跑。PR #33 分支已 cfg-gate，CI `ubuntu-latest` 作业 SUCCESS，但非 Windows 上加载 Native 仍返回 BLOCKED（能编不能跑）。CI 里 main 的 Rust 作业只跑 `windows-latest` | 两次实跑日志；`.github/workflows/repository-policy.yml`；PR #33 checks |
| C# 门禁 | `node .spec/tools/spec-lint.mjs && node --test …` → OK，13/13。`cd mvp-host && bash eng/verify-all.sh` → **`MVP_HOST_VERIFY_FAIL test …App.Tests`**（4 分 54 秒）：isolation / `SDK_OK sdk=10.0.400` / portability / mirror 29 文件 / generated 11 文件 / restore / format / build 全过，Admission.Tests 26/26，**App.Tests 32 过 2 败**（`LiveElevenPathTests.GameplayAssemblyDiscoveryFindsSiblingLumioGame`：找不到兄弟仓 `Lumio.Game.ServerGameplay.dll`；`…SecondRoomRunOnTestControl`：`{"ok":false,"kind":"Rejected","error":"invalid_request"}`），脚本在此中止，其余 11 个测试工程未跑。ADR 0008 早已记录 CI `MVP C# host policy` 是 FAILURE（CS0234）。`account-server` restore / build / test → **32/32 通过** | 实跑日志 |
| 旧合同制残留 | `LGE-V1` 命中 60 文件；`docs/architecture/` 6 份正文（v0.3 / 1.0 / 1.1 是 428 B 空壳，v1.2 / 1.3 / 1.4 各 40–48 KB）+ `.baseline.sha256`（共 152 KB）；`docs/LumioServer_Framework_Implementation_Design_2026-08-27/`（568 KB，就是那 47 张已作废卡的蓝图）；`docs/specs/`（456 KB，mvp C# 设计 + 14 张卡）；`generated/` 3 个 crate（2 个 `compile_error!` 拒绝壳；1 个从**已退役仓 `LumioGameEngineArchitecture` 的 git rev `3d5e29d` 拉 6 个 `lumio-gen-*` 依赖**，靠 GitHub 重定向活着）；`contracts/*.lock.toml` 3 份（写死 `C:/Work/...`）；`tools/xtask` 6,844 行 V1.4 守卫（contracts verify / dag / queues / policy / source_scan）+ `.spec/guards/*.toml` + `tests/policy/`；CI `repository-policy.yml` 断言 v1.4 正文、`sha256sum -c`、README 含 `LGE-V1.4-2026-08-27`；README / `.spec/AGENTS.md`（写的还是 **V1.2**）/ `repository-architecture.md` / `modules/README.md` 全是合同制口径，`.spec/AGENTS.md` 仍把设计落 `docs/specs/`；仓根两份 `.wf-report-*.md`；mvp-host `contract-mirror/`（29 锁文件）+ `GeneratedContracts`（11 文件）vendored V1.4 | `grep -rIl LGE-V1`、`du -sh`、各文件实读 |
| 账号服残留 | `account-server/`（2,975 行）按 ADR-061 已迁 `LumioPlatform`（那边 `src/Lumio.Platform.Account` + `contract/account-port-v1.json` 已在）；但本仓 Rust 验收套件 `discover.rs:24` 仍从本仓 `account-server/.../lumio-account-server.dll` 拉起账号服——第二份账号权威还活着，且是验收套件的依赖 | `discover.rs`、`account.rs`、`ls ~/LumioGames/LumioPlatform` |

#### 模块状态（`ds-server.md` M1–M11 × 代码 × 卡）

| M | 模块 | 代码现状（main） | 卡 |
| --- | --- | --- | --- |
| M1 | 准入五步与连接层 | `entity_chat/admission.rs` Ed25519 凭据验签 + `wire.rs` loopback WebSocket + 会话表；无未验证限额、无关闭原因码词表 | R-00346 / 350 / 374 done；R-00388 / 408 in_progress |
| M2 | Account Server | 本仓 `account-server/` 应删（归 Platform，ADR-061） | R-00344 done（历史） |
| M3 | 传输适配 | 只有 WS 文本帧；尺寸上限是代码常量（`INGRESS_QUEUE_PER_CONNECTION = 64`），违反「上限走配置」 | — |
| M4 | 每帧字节配额 | 无 | 契约卡 0-4 未开 |
| M5 | 空间粗筛 | 无（D4：炸弹人 Stage 0 用不上，不开） | D4 |
| M6 | 复制内核 | 宿主只搬 Runtime `BuildDelta` 字节（R-00374）；R5-03 要改成 `Enqueue / Tick / DrainOutbox` 五 op | R-00408 in_progress |
| M7 | 发送调度 / 慢客户端阶梯 | 无；只有 `pending > 64 不 RunTick` 的粗背压 | 契约卡 0-6 未开 |
| M8 | 断线重连接管 | 5 分钟窗走 NativeCore `wallClock` one-shot（Windows 才能真跑）；顶号先发 `ConnectionSuperseded` | R-00350 / 374 done |
| M9 | 进程、房间、维护 | 无维护 / 滚动更新；hello 世界循环 `world.rs`（1,285 行）与 entity-chat 宿主 `host.rs`（845 行）**两套宿主循环并存于同一 crate** | 自驱主循环在 PR #33（R-00388） |
| M10 | 时间同步 | 无 | 契约卡 0-9 未开 |
| M11 | 观测 | NDJSON audit（`audit.rs`）；PR #33 加 `log.rs` 写 `server.ndjson` | — |

#### Workflow 对账（只读；RM-00006 + 四个纵切 Room）

RM-00006 overview：68 张 = 15 done / 52 rejected / 1 backlog（R-00478）；phase `needs_definition`；验收项未通过 319 条（全挂在已作废旧卡上）；缺验收项的需求 2 张（R-00260、R-00478）。

**这些写入是另一会话在 2026-09-05 16:28:39–16:29:25 UTC（北京 09-06 00:28）执行的**（Owner 2026-09-06 已追认，D15），操作人 `usr_e6bba…`（Lumio 账号）：52 张 backlog 旧卡 → rejected（47 张 08-27 蓝图卡 + R-00183 原始需求 + 4 张 mvp-host 卡 R-00278 / 280 / 281 / 282）；R-00260 与 R-00276 acceptance → done（**未补证据评论**）；新建 R-00478《[程序·工程] 退出旧合同制残留、修复 Rust 宿主 macOS 编译并归档 C# mvp-host》，P0，**正文只有一行「按 D15…」、0 验收项、0 评论**——不符合建单规范「裸标题不落库 / 至少背景 · 目标 · 验收 · 边界四节」。

纵切 Room 里真正属于 LumioServer 的卡（注意 `[程序·服务端]` 前缀在 RM-00011 / RM-00013 里也标 Runtime / NativeCore / Game / 玩法卡，不能按前缀认）：

| 卡 | Room | 状态 | 与仓库的关系 |
| --- | --- | --- | --- |
| R-00340 | RM-00010 | done | hello 世界 Rust Server，`world.rs` / `server.rs` 就是它 |
| R-00346 / 350 / 359 / 374 / 387 | RM-00011 | done | 都在 main |
| **R-00388**（R4-02） | RM-00011 | **in_progress** | PR #33 未合入；r4 整体复核已退回，r5 蓝图明文「并入 R5-03，不再单独续做」（`plans/2026-09-04-rm-00011-r5-cards.md` 第 41 行），Workflow 未同步关闭 |
| **R-00408**（R5-03） | RM-00011 | **in_progress** | 09-04 先 BLOCKED（Runtime `Enqueue` 无 admit 消息），后一条乱码评论称 Server 分支 `Go1c/r5-03-server a6e7024`——**该分支不在 origin**，仓内无交付 |
| R-00391（R4-08） | RM-00011 | backlog | 旧仓名 / 死引用清理；r5 已说并入 R5-03 |
| R-00392（R4-09） | RM-00011 | backlog | 11 场景 Rust 宿主重跑，等 R-00408 |
| R-00465（必备-S4） | RM-00014 | backlog | 服务器接入通用有序输入 + 已提交复制输出；conditional，等 R-00460 冻结 |
| RM-00013 S0-G1 ~ G6 | RM-00013 | backlog | 标 `[程序·服务端]` 但全是 LumioGame 玩法卡，不归本仓 |

#### 架构演进与前线现状（炸弹人通俗透视）

老王开网页打炸弹人，服务器进程一局里干三件事：

1. **门卫**：老王先去平台（LumioPlatform）登录拿一张签过名的门票；服务器只验票，不认用户名口令（M1 / M2）。验过票才给他建会话、绑到房间里的一个玩家实体。
2. **钟表匠**：服务器按固定节拍（比如每 50 ms）敲一下 Runtime 的 `Tick()`，13 相走完，第 10 相唯一提交点之后才有「这帧世界变了什么」。节拍来自 NativeCore 定时内核，不是 Rust 自己 `sleep`。
3. **邮局**：把 Runtime 打好的包（谁在哪、几号炸弹引爆、哪块木箱没了）按每个人的视野和字节配额发出去；老王的按键上行也只是塞进有界队列等下一拍，网络线程一个字节都不碰世界。

今天代码里成立的：门卫（Ed25519 验票、会话表）、钟表匠（`timer_*` ABI，只在 Windows）、邮局的最简版（Runtime `BuildDelta` 字节原样广播）——都是聊天切片（101 实体）的形态。炸弹人 Stage 0 还缺：通用有序输入 / 已提交输出直通（R-00465）、自驱主循环合入 main（PR #33 → R-00408）、每帧字节配额（M4）、发送调度（M7）；粗筛（M5）19×19、8 Bot 用不上（D4）。

**更底的坑：生产 DS 跑 Linux，Rust 宿主今天只能在 Windows 链接。** 架构仓 `eng/dev-build.ps1` 在非 Windows 已经产 `liblumio_engine_native.so`；Server 却手写了 840 行 Win32 `LoadLibraryW` 加载器（`sdk_loader.rs`）+ 418 行 timer 加载器（`native_timer.rs`），两份都无条件链 `kernel32`。PR #33 的 cfg-gate 只是让它在 Linux 编过并 BLOCKED，不是能跑。Living Architecture 说共享 Loader 归 SDK（`architecture.md` §5），托管侧已有 `Lumio.Engine.NativeLoader`；Rust 宿主的加载器要么换成一份跨平台 `dlopen`（`libloading`），要么消费 SDK 提供的 Rust loader——两份 Win32 手写加载器都不该留。

#### 漂移与问题（按严重度）

① **P0 · 三张卡同抢「修 macOS 编译 + 旧仓名清理」**：R-00478 ③、R-00408 标题尾、R-00391 全卡，PR #33 里还有半成品提交。同一件事四处写，正是「第二份真值」。
② **P0 · Rust 宿主 Windows-only 是链接级**（kernel32），不是另一草稿说的 dead-code lint；只修 lint 解决不了。生产 Linux 目标今天零覆盖。
③ **P1 · R-00388 状态漂移**：r5 已宣布并入 R5-03，Workflow 仍 in_progress，PR #33 悬着；R-00408 的 Server 分支不在 origin，「Workflow 领先、origin 无交付」。
④ **P1 · R-00478 正文违规**（一行、0 验收项）；R-00260 / R-00276 转 done 无证据评论。
⑤ **P1 · 三套宿主**：hello `world.rs` 与 entity-chat `host.rs` 同 crate 并存，C# mvp-host 是第三套；ADR 0008「整目录删除等 51 张 Rust 主线」的前提已随 52 张作废而失效，mvp-host 去留要重裁。
⑥ **P1 · account-server 第二份账号权威**仍被验收套件依赖（ADR-061 已把账号归 Platform）。
⑦ **P2 · verify-all 在 macOS 红**（2 个 LiveEleven 测试依赖兄弟仓 Game 产物）；CI `MVP C# host policy` 长期红。
⑧ **P2 · 旧合同制残留面比前三站都大**：13 个空模块骨架、6,844 行 xtask 守卫、从已退役仓拉 git 依赖的 `generated/` crate、写死 `C:/Work` 的 lock 文件、`.spec/AGENTS.md` 还写 V1.2。

#### 与另一会话草稿的对照（互相吸取）

| 项 | 另一会话草稿 | 本会话实测 | 处置 |
| --- | --- | --- | --- |
| 宿主路线转向 Rust、C# 冻结 | ✓ | ✓（ADR 0004 → 0008 链） | 采纳 |
| 钟表匠 / 邮局 / 包租婆比喻 | ✓ | ✓ | 采纳，「包租婆」改「门卫」与 ds-server.md M1 口径一致 |
| RM-00006 52 张旧卡作废、2 张 acceptance → done | ✓ 且已执行 | Workflow 读回一致 | 方向采纳；执行未记 §7、无证据评论——本节与 §7 补记 |
| macOS 编译阻断根因 | `native_timer.rs` dead-code 警告 | dead-code 只是第一道；第二道是 kernel32 链接失败，两个文件 | **更正** |
| mvp-host verify-all | §2.4 草稿写「全绿 `MVP_HOST_VERIFY_OK`」，其 §7 又承认 App.Tests 2 处失败 | **FAIL**（App.Tests 32 过 2 败，脚本中止） | **更正**：结论与证据要一致，附命令输出 |
| 代码量 | rs 7,370 + 2,860；cs 15,200 + 11,800 | rs 22,134；cs 43,870 | 更正 |
| csproj / 远端分支 / 旧设计目录 | 28 / 13 / 2.8 MB | 32 / 15（2 个未合入，含 PR #33）/ 568 KB | 更正 |
| 纵切 Room 对账（R-00388 / 408 / 391 / 392 / 465） | 未做 | 已做，发现三卡重叠与 R-00388 漂移 | 本会话补 |
| account-server vs ADR-061、Linux 生产目标 | 未提 | 已提 | 本会话补 |
| R-00478 | 已建，一行正文 | 需按 card-spec 重写 + 验收项，范围去掉 ③ | 待 Owner 裁决（D21） |

对方值得学的：先把 Workflow 出清动作做掉，Room 干净了再谈新卡；比喻直白。本会话值得对方学的：数字用命令跑出来、根因跑到失败的那一层、纵切 Room 必须扫、写入必须记 §7 并补证据评论。

#### 阶段判定

一句话：**聊天切片形态的 Rust 宿主在 Windows 上成立，其它平台链接不过；主循环自驱只在未合入的 PR #33；三处把「修 macOS」写成自己的活；整仓仍被 V1.4 合同制包裹（13 个空模块骨架、6.8k 行守卫、已退役仓 git 依赖）；RM-00006 旧卡已出清，新卡 R-00478 只有一行正文。**

#### 剩下的单子怎么补（待 Owner 裁决；写 Workflow 须逐次授权）

| 卡 | 动作 | 范围 | 依据 |
| --- | --- | --- | --- |
| R-00478 | **已回写（Owner 授权，2026-09-05 23:50Z）**：标题 + 四节正文 PATCH、11 条验收项 POST，读回 11 / 11 not_started；卡面见 [`plans/2026-09-06-server-w0-card-and-kickoff.md`](../plans/2026-09-06-server-w0-card-and-kickoff.md) §一 | **只做清理**（D21 已裁决）：① 删 `docs/architecture/` 6 md + `.baseline.sha256`、`docs/LumioServer_Framework_Implementation_Design_2026-08-27/`、`docs/specs/`、两份 `.wf-report-*.md`；② CI 删 v1.4 正文 / sha256 / `LGE-V1.4` 断言；③ 删 `generated/` 3 crate、`contracts/*.lock.toml`、`tools/xtask` 合同守卫（dag / queues / policy 是否保留另议）、`.spec/guards/`、`tests/policy/`、13 个 README-only 模块目录；④ README / `.spec/AGENTS.md` / `repository-architecture.md` / `modules/README.md` 改 Living Architecture 口径，设计落点改 `.spec/knowledge/features/`；⑤ **去掉「修 macOS 编译」**——归 R-00408 | D1 / D8 / D12 同路；漂移 ① |
| R-00408 | **已执行（23:59Z）**：评论 `01a07403-4634` 钉五条——D24 加载器口径（`libloading` 一份、装一次、Linux + Windows 双绿、CI 加 ubuntu 必过）、D23 cherry-pick 清单、D22 account-server 自签凭据后删、xtask 作废、与 R-00478 的文件集边界 | 该卡已拥有 `modules/process/**` + `host-runtime/**` + CI | 漂移 ②；D24 |
| R-00388 | **已执行（23:59Z）**：流转 `rejected`（reason 引 D23）+ 评论 `01a07403-3b14`；PR #33 已用 `gh` 关闭（23:59:32Z，分支 `feat/r-00388-r4-02-self-drive` 保留待 cherry-pick） | — | r5 cards 第 41 行；D23 |
| R-00391 | **已执行（23:59Z）**：流转 `rejected` + 评论 `01a07403-423f`，并入 R-00408 | — | 同上 |
| mvp-host | 已裁决删（D22）：并入 R-00478 验收 11 | 40k 行、CI 一个作业、verify-all 红 | ADR 0008 前提失效 |
| account-server | 已裁决删（D22）：归 R-00408——套件改自签测试凭据后删目录 | — | ADR-061 |
| 远端 14 条分支 | **已删（Owner 授权，2026-09-06）**：13 条逐条实测零差异 + `feat/r-00346-admission`（82 行 mvp-host 锁文件，随 D22 作废）；远端只剩 `main` 与待 cherry-pick 的 `feat/r-00388-r4-02-self-drive` | — | 卫生 |

不开的卡及原因：M4 配额 / M7 调度 / M10 时间同步 契约卡（炸弹人 Stage 0 8 Bot 单房间跑通前不开，先让 R-00465 把输入 / 输出直通做实）；M5 粗筛（D4）。

### 2.5 LumioClient（第五站 · 客户端基础运行时、浏览器 / Bot 宿主与表现层）

> 本节由本会话按五步只读实测写成。同日另一会话（工作区在架构仓 worktree `LumioGameEngine-r479`，其草稿未合入本文件）已先盘过本仓并**执行了 Workflow 写入与分支清理**（D25–D29，Owner 已收签）；其草稿在末尾「与另一会话草稿的对照」小节逐条对账：采纳其方向，更正其数字与两处口径。与第四站相同：数字全部用命令跑出，别人的结论先复核再引用。

#### 仓库侧事实（本机 macOS，2026-09-06 08:00–08:30 北京时间实跑）

| 项 | 实测 | 出处 |
| --- | --- | --- |
| origin/main = 本地 main | `c3825a4`（2026-09-05，`Merge pull request #18 … spike/2026-09-05-cl-1-runtime-wasm`），`git pull --ff-only` → Already up to date，0 ahead / 0 behind；工作区只有 `.spec/lumio-agent.lock` 一处未提交改动（第三站的跨仓锁文件修复，见 §7 GameRuntime 站）；74 个提交、18 个 PR 全部 MERGED、0 个 OPEN；main 最近 4 次 CI（Repository Policy + .NET Test ubuntu / macos）全绿 | `git status -sb`、`git rev-list --count`、`gh pr list --state all`、`gh run list --branch main` |
| 分支 / worktree（两次快照） | **08:06**：3 个游离 worktree（`charming-dijkstra-fb6d5a`@`45d804b`、`elated-neumann-1ca4c2`@`219e1a4`、`wonderful-stonebraker-f29af4`@`219e1a4`，均 0 脏文件、8 月 29 日建）+ 8 条本地分支（全部 `merge-base --is-ancestor` = 已合入）+ 远端 3 条已合入分支（`codex/r-00415-p3-3`、`feat/r-00349-replica-chat`、`feat/r-00375-replica-snapshot`）。**08:23 复查**：worktree 目录已空（mtime 08:14）、本地只剩 `main`、远端 3 条已删——是另一会话按 D29 执行的；远端同时**新出现** `fix/r5-client-runtime-frames`@`5652b95`（11 个提交，2026-09-05 11:05 → 09-06 07:08，作者 Go1c，无 PR、无 CI 运行），见 Workflow 对账 R-00408 | `git worktree list`、`git branch -vv`、`git ls-remote --heads`、`ls -la .claude/worktrees`、`gh pr list`、`gh run list --branch fix/r5-client-runtime-frames` |
| 语言与规模 | C#：`modules/` 261 文件 22,202 行（src 165 文件 12,411 / tests 90 文件 8,841 / host 950）、`tests/` 35 文件 1,892、`eng/upstream-contract-smoke` 104、`spikes/` 11 文件 1,047；另 `contract-mirror/` vendored 上游 C# 1,772 行——合计 319 文件 27,017 行。手写 JS / HTML / CSS（`git ls-files`，不含未入库的 `publish/` 生成包）2,206 行：`modules/web` 753、`spikes/runtime-wasm/tools` 696、`eng/*.mjs` 757。Rust 1,735 行全部是 `contract-mirror` 里的 vendored 生成物。Unity：`packages/com.lumio.client` UPM 骨架 = `package.json` + 9 个 `.asmdef`，无场景 / prefab；`unity-adapter` / `hybridclr-adapter` 两个生产 csproj **0 个 .cs 源文件**（各只有 csproj + lock），测试工程只有 `GlobalUsings.cs` | `find … -name '*.cs' -not -path '*/obj/*'` + `wc -l`；`git ls-files '*.js' '*.mjs' '*.html' '*.css'`；`find packages -type f` |
| 工程与测试 | 40 个 csproj（slnx 29 + spikes 5 + contract-mirror 6，不计 worktree 副本）；14 个 xunit 测试工程共 252 个 `[Fact]` / `[Theory]`（replica 44 / connection 57 / session 35 / hello 24 / prediction 14 / bot 12 / observability 10 / input 8 / handshake 5 / persistence 2 / unity-adapter 0 / hybridclr 0 + ArchitectureTests 38 + IntegrationTests），`modules/web/chat` 3 个 node test | `grep -rho '\[Fact\]\|\[Theory\]'`、`grep -c '<Project Path' LumioClient.slnx` |
| 门禁 1（规范 / 钉版 / 镜像） | `node .spec/tools/spec-lint.mjs` → OK；`node --test spec-lint.test.mjs` 13 / 13；`node eng/verify-sdk-pin.mjs` → OK（4 处 10.0.400 一致，CI 两个 yml 未固定 dotnet-version）；`node eng/verify-five-file-package.mjs` → OK（5 文件哈希一致）；`bash eng/verify-contract-mirror.sh` → OK（326 个文件逐字节匹配 `contract-mirror.sha256`）；`repository-policy.yml` 的 README / `.baseline.sha256` 断言本地复现 → 全过 | 实跑日志 `client-gate.log` |
| 门禁 2（restore / 编译 / 测试） | `dotnet restore LumioClient.slnx` → All projects are up-to-date（**未出现另一草稿所说的 NU1004**）。`dotnet test LumioClient.slnx --no-restore` 对同级 `../LumioGameRuntime`（本机 = origin/main `89a7a6c`）编译：**预期编译失败**——`modules/replica/src/Public/ReplicaWorld.cs:109` 与 `:451` 各调用一次 `_manager.GrantClaim("self", "EntityIdentity.claimedMark")`，而 Runtime origin/main 的生产代码已 0 处 `GrantClaim`（`c2d42b3`，2026-09-04「complete r5 world manager projection」删除；只剩测试里的禁用词表）。**根因链**：Client CI `dotnet-test.yml` 把 Runtime 钉在 `7f198e5`（2026-09-03，仍含 `GrantClaim`，落后 origin/main 16 个提交）→ CI 绿是「假绿」：main 对着一个已过期的 Runtime 通过，对着真 Runtime 编不过。实跑结果见本表末行 | `grep -rn GrantClaim modules`、`git -C ../LumioGameRuntime log -S GrantClaim`、`git rev-list --count 7f198e5..origin/main`、`.github/workflows/dotnet-test.yml` |
| 旧合同制残留 | `LGE-V1` 命中 **125** 文件（其中 93 在 `contract-mirror/`，其余 32：README、`.spec/AGENTS.md`（写的还是 **V1.2**，设计落 `docs/specs/`）、`repository-architecture.md`、13 个模块 README、CI、`docs/` 全部、`eng/upstream-api-map.md`、`tests/Fixtures/index.json`）；已退役仓名 `LumioGameEngineArchitecture` 81 处；`docs/architecture/` 6 份（v0.3 / 1.0 / 1.1 各 428 B 空壳，v1.2 / 1.3 / 1.4 共 134 KB）+ `.baseline.sha256`（合计 152 KB）；`contract-mirror/`（**326 个文件 1.7 MB**，逐字节镜像已退役架构仓 `a206e2c` 的 schemas / fixtures / ids / packages 四目录，1 个锁文件 `contract-mirror.sha256`）+ `eng/sync-contract-mirror.*` / `verify-contract-mirror.*` + `eng/upstream-contract-smoke` 工程 + `tests/Fixtures/index.json` 语料 pin + 4 个模块的 `Generated*Adapter.cs` / `Generated*FixtureTests.cs`（connection / handshake / replica / prediction 消费旧 fixture 语料）；`docs/LumioClient_five_requested_files/`（272 KB，8 月 27 日五文件包）+ `eng/verify-five-file-package.*`（CI 一步专门校它的哈希）；`docs/specs/` 60 KB；CI `repository-policy.yml` 断言 v1.4 正文 / `sha256sum -c` / README 含 `LGE-V1.4-2026-08-27` / 每个模块 README 含 `LGE-V1.2-2026-08-27` / 模块目录数恰为 13；README「架构基线 LGE-V1.4 · 唯一架构源 LumioGameEngineArchitecture · Architecture Gate 用 `python3 tools/lumio_contract.py validate`」整节合同制口径 | `grep -rIl LGE-V1 --exclude-dir=contract-mirror`、`du -sh`、各文件实读 |
| 第二份 codec | `modules/replica/src/Internal/GameplayCodec.cs`（568 行）+ `LiteJsonParser.cs`（511 行）= **1,079 行手写 C-1 JSON 编解码**（自己的 `bad_envelope` / `bad_payload_hash` 词表、mapping 白名单、SHA-256 校验）；生产代码 0 处引用 Runtime `WireCodec`。R-00408（R5-03）正文明写 Client 侧「删自编 FullSnapshot / 手写信封 / 第二份 codec」——main 未做；`fix/r5-client-runtime-frames` 分支加了 4 处 `WireCodec.` 调用，但两文件仍在（544 + 511 行） | `wc -l`、`git grep -c 'WireCodec\.' origin/main`、同命令对分支 |
| 模块 README 口径 | 13 个模块 README 的「阶段」：9 个写「未实现」（session 1,967 行 / prediction 1,172 行 / observability 963 行 / bot / handshake / input / persistence 都有代码却仍标未实现；unity-adapter / hybridclr-adapter 是真未实现），只有 connection / replica / hello / web 写了交付状态；全部引用 `docs/architecture/…v1.2.md` 章节作「公共契约来源」 | `awk '/^## 状态/…' modules/*/README.md` |
| CL-1 调研（PR #18 已合入） | `docs/spikes/2026-09-05-spike-runtime-wasm.md`（562 行）+ `spikes/runtime-wasm/`（入库 71 文件 748 KB：探针 app / bench / host / tools + `results/*.jsonl`；磁盘 358 MB 是未入库的 publish / bin / node_modules）。结论：Runtime Ecs + Replication + 样板零改动进 .NET 10 browser-wasm，解真实 FullSnapshot / Delta、上行被接受，三档实体世界哈希与桌面逐位一致；启动 0.6–5.6 s、AOT 后重建中位 0.5–1.7 ms；手机全部未执行；推荐路线 A（桌面浏览器 .NET WASM）。**三条上游缺口**：Runtime Ecs 4 处 IL2075 反射让 `PublishTrimmed` 默认失败；Runtime 无公开 InputCommand 信封编码 API；Server Rust 宿主 Windows-only 且与 Runtime HEAD API 漂移。落点核对见「漂移与问题」⑤ | 报告 §0 / §5 / §6 / 附 |
| `dotnet test` 实跑结尾 | 全解决方案 `dotnet test LumioClient.slnx --no-restore` 跑 25 分钟无输出后由本会话终止（exit 144），改为定向实跑：`dotnet build modules/replica/src/Lumio.Client.Replica.csproj --no-restore` → **Build FAILED**，`ReplicaWorld.cs(109,26)` 与 `(451,26)` 各一条 `error CS1061: 'WorldManager' does not contain a definition for 'GrantClaim'`（7.3 s）；对照 `dotnet test modules/connection/tests/Lumio.Client.Connection.Tests.csproj --no-restore` → **Passed! 70 / 70**（1 s；不依赖 Replica 的工程正常）。结论：**Replica 及所有引用它的工程（session / bot / Bot.Host / ArchitectureTests / IntegrationTests）在本机对 Runtime `89a7a6c` 都编不过**，另一草稿「dotnet test 全面红灯」成立，根因同 | `client-gate.log`；定向 build 输出 |

#### 模块状态（README 13 模块 × 代码 × 卡 × 与 Living Architecture 的关系）

| 模块 | 代码现状（main） | 卡 | 判定 |
| --- | --- | --- | --- |
| connection（P0） | LocalEmbedded 环回 + WSS `ClientWebSocket` 传输（535 行）+ 有界发送队列 + 状态机；57 测试 | T-00002 / T-00003 done | 成立，是今天最扎实的模块 |
| handshake（P0） | 412 行：状态机 + `Generated*` 适配（消费旧 fixture 语料）；5 测试 | — | 按 V1.2「Release / Manifest / Schema / ABI / Capability 五校」设计，现行 C-1″ 只有 `Welcome`——**形状待随 A0 重定** |
| replica（P0） | 2,875 行：`ReplicaWorld`（薄门面套 Runtime `WorldManager.Create`）、C-1 手写 codec 1,079 行、Gap / Stage 账本；44 测试 | R-00349 / 375 / 389 done（证据提交都在 origin/main） | 成立但**带第二份 codec + 2 处对已删 Runtime API 的调用**（编译红） |
| prediction（P0） | 1,172 行 Foundation 骨架：`PredictionKey` / 窗口 / 账本 / `RuntimePredictionPlanAdapter`；`AuthorityUpdateOrchestrator.cs:52` 固定 `AuthorityPredictionUpdate(update, 0)`，`IClientRuntimePort` 常量成功 | R-00467（C5）backlog | **不是 ADR-064 第 8 条的「确认世界 + 预测世界整体克隆 + 重放」**，C5 要整段重写；README 标「未实现」反而是对的 |
| session（P0） | 1,967 行：`ClientSession` + 9 个 Orchestrator（握手 / 首连 / 权威更新 / 本地预测 / Resync / 重连 / 关闭）；35 测试 | R-00389 done（顶号回登录）；R-00408 Client 部分 | 骨架成立；权威更新事务走 `replicaPlan` 单腿、`reconcile` 丢弃——C5 改 |
| input（P1） | 380 行：有界 Sample 队列 + 序号分配；8 测试 | — | 成立（最小切片） |
| persistence（P1） | 464 行：内存 checkpoint / verified artifact；2 测试 | — | 最小切片，无消费者 |
| observability（P1） | 963 行：有界事件管线 + 内存 Sink + `EventDispatcherWorker`（`Channel<T>` 消费线程）；10 测试 | R-00031 done | 成立；**注意** wasm 单线程下不能开线程（CL-1 A1-5），浏览器路线要重看 |
| unity-adapter / hybridclr-adapter（P1） | **0 行源码**，空 csproj + 空测试工程 + UPM 9 asmdef | R-00253 / 254 / 255 三张 SPIKE 已 rejected（D26）；T-00009 todo | 与 LumioGame ADR 0013「首发不接任何游戏引擎、Unity 降为后续候选」一致 → **空壳去留待裁（D31）** |
| bot（P1） | 452 行：`HeadlessBotHost` + `ClientTimerManager`（消费 NativeCore `tickFrame`，经 SDK NativeLoader）；host 950 行；12 测试 | R-00342 / 375 / 389 done | 成立；是今天唯一能跑「同进程双端」验收的 C# 客户端宿主（bomber-slice §3 末段） |
| hello / web（P1） | hello 1,787 行 + 24 测试（MS-00002 hello-wire-v1 Bot）；web 753 行纯静态页（hello 441 JS + chat 49 JS，`fetch('./contract.json')` 读契约） | RM-00010 R-00341 / 342 done；R-00415（RM-00012）approved | 成立；web/chat 是 R5「聊天窗口归 UI」的落点 |

#### Workflow 对账（只读；RM-00007 + RM-00010 / 11 / 13 / 14 + RM-00012 顺带）

**先说时间线（第四站教训 ①）**：本会话 08:00 开工；**另一会话在 08:14:42–08:15:22（UTC 00:14–00:15）写了 RM-00007**：9 张 backlog 卡 → `rejected`、新建 R-00481；更早 **08:11–08:12（UTC 00:11–00:12）又一会话写了 RM-00011**：R-00408 → `acceptance`（4 条验收项全 passed）、R-00392 / R-00393 → `in_review`。三处写入操作人都是 `usr_e6bba…`（Lumio 账号）。本会话**零写入**。

RM-00007 overview（08:16 读回）：19 张 = 9 done / 9 rejected / 1 backlog；phase `needs_definition`；工作项 5 = 3 done / 2 todo；缺验收项的需求 7 张（含 R-00481）；验收项未通过 56 条（全挂在 done / rejected 卡上）。

| 卡 | Room | 状态 | 与仓库的关系 |
| --- | --- | --- | --- |
| R-00001 / 019 / 031 / 065 / 067（五文件包与 Foundation） | RM-00007 | done | 8 月 27–29 日交付，是 `docs/LumioClient_five_requested_files/` 与 `verify-five-file-package` 的来源；Foundation 骨架就是今天的 session / prediction / handshake |
| R-00287 / 288 / 291 / 256 | RM-00007 | done | CI dotnet test job（在）、供应商泄漏测试（在）、**contract-mirror（在，就是 1.7 MB 镜像）**、HybridCLR 官方版本 SPIKE（`docs/spikes/2026-08-28-…`，Unity 路线已降级） |
| R-00055 + R-00253 / 254 / 255（Wave 7 垂直切片 + 三张 Unity / IL2CPP SPIKE） | RM-00007 | **rejected（08:14）** | 与 ADR 0013 一致，方向对；**流转无 reason、无作废评论** |
| R-00289 | RM-00007 | rejected | 早在 8 月 29 日就被核实「落错仓，对象在 LumioGame」，作废对 |
| **R-00292**（架构测试递归枚举整个仓根 → 有嵌套 worktree 就假红，P0） | RM-00007 | rejected | **代码里的缺陷还在**：`ProjectGraphTests.cs:27`、`ToolchainPolicyTests.cs:29` 仍 `Directory.EnumerateFiles(RepoRoot.Path, …, AllDirectories)`；CL-1 报告 §7.2 又撞了一次。作废卡不等于修好——建议并入 R-00481 验收项 |
| R-00294（WSS 工厂两个 public 类型落在 `Internal/` 目录） | RM-00007 | rejected | 缺陷仍在（`Internal/Transport/WebSocket/WebSocketClientConnectionFactory.cs`）；P2 卫生项，作废可接受，但应在 R-00408 / R-00481 顺手改 |
| R-00296 / R-00298（Client Chunk 三态 / RTT/2 离群剔除，8 月 30 日 DS-MVP 边界卡） | RM-00007 | rejected | 炸弹人 Stage 0 体素不进预测世界、不下发客户端（voxel.md / D3），现在不需要；以后体素流送到客户端时按新契约重开 |
| **R-00481**《[程序·工程] 退出旧合同制残留：修复 Replica 上游破坏性漂移、删除 docs/architecture 镜像并对齐 WASM 预测》 | RM-00007 | backlog（08:15 建） | 正文三层（① GrantClaim 对齐 + lock；② 删 docs/architecture / contract-mirror / CI 基线；③ README 改口）；**0 条验收项、0 评论、未指派**；「拥有范围 `modules/replica/**`」与 R-00408 Client 部分（同样拥有 `modules/replica/**`、`session/**`、`bot/**`、allowlist、`web/**`）**重叠**——见漂移 ① |
| T-00004 / T-00005（A1 MVP C# 宿主的握手帧分类器 / bot host 远程 CLI） | RM-00007 工作项 | todo | 来源 R-00260 MVP C# 宿主已随 D22 判删、T-00005 的 Bot.Host 远程接入已被 R-00389 用另一条路做掉 → 两张都已无效，待作废 |
| T-00009（UPM `package.json` 缺 unity / unityRelease / dependencies） | 挂 R-00255（已 rejected） | todo | 随 D31 处置 |
| R-00341 / R-00342 | RM-00010 | done | 证据 `7a1c37c` 在 origin/main ✓（hello Bot + web/hello 页） |
| R-00349 / R-00375 / R-00389 | RM-00011 | done | 证据 `cf218a8` / `780bc5c` / `1473cc9`（PR #14 / #15 / #16）都在 origin/main ✓ |
| **R-00408**（R5-03，三仓一卡，Client 部分） | RM-00011 | **acceptance（08:11）** | 评论称五仓「本地 main」提交 Engine `9a22527` / Runtime `18b5feb` / Server `f031afb` / Client `5745d11` / Game `d3f6fe0`，4 条验收项 passed，证据目录 `.tmp-acceptance-r5-pack5/`。**本机核验**：五个提交只有 `9a22527` 在 origin（架构仓分支 `r5-closeout-20260906`）；Client / Server / Game 各有一条**今天新推的分支**但 tip 是另外的号（`fix/r5-client-runtime-frames`@`5652b95`、`fix/r5-server-wire-forwarding`@`deebeb4`、`fix/r5-entity-chat-scenarios`@`e243a39`）；**Runtime 什么都没推**（`18b5feb` 不存在）；四仓都**没有 PR**、分支没有 CI 运行；证据目录与 `.acceptance-*` 检出都在 Windows `C:/Work/LumioGames`。结论：「Workflow 领先、origin 无完整交付」——同 R-00475 `dc37644` 的情形；解铃条件 = Runtime 推送 + 四仓开 PR 合入后重核，此前不得 → done |
| R-00392 / R-00393 | RM-00011 | in_review（08:11–08:12） | 同一批证据；R-00393 的独立复核报告在 `r5-closeout-20260906` 分支（17 条 fixture 15 过 2 partial）。同上，等推送 |
| **R-00467**（必备-C5 输入确认 / 预测历史 / 共同校正） | RM-00014 | backlog | 6 条验收项 not_started、0 评论；正文点名今天的缺陷（`ConfirmedThroughSeq=0`、`reconcile` 丢弃、`committed=true` 常量）；D16 把 A0 前置收窄后**它仍在四张被约束的卡里**（碰 wire 的 `sequence` / `appliedInputSequence`），要等 R-00460 那一组冻结 |
| **R-00470**（必备-C8 ModelTransform 表现采样 + 浏览器最小消费宿主） | RM-00014 | backlog | 6 条验收项 not_started、0 评论；AC06 要「浏览器通过正式样本端口真实显示移动 / 纠偏 / 销毁」——**这就是 CL-1 路线 A 的立项落点**，不需要另开「WASM 客户端壳」卡；不在 A0 约束名单里，前置是 R-00461（done）+ R-00466（RT-3） |
| R-00415（浏览器游戏页经平台 launch 取地址） | RM-00012 | **approved** | PR #17 已合入 `06db121`（2026-09-04），评论已附交付证据——**仓库领先 Workflow**，待流转（Platform 站一并处理） |
| RM-00013 S0-* | RM-00013 | backlog | 无 Client 卡（Stage 0a headless 先行，0b 浏览器客户端按 LumioGame ADR 0014 后置）——正确，无缺口 |

#### 架构演进与前线现状（炸弹人通俗透视）

老王开网页玩炸弹人，客户端在整套架构里只干三件事：

1. **传声筒**：把老王的按键编成一条带序号的输入包发给服务器，把服务器每帧发回的权威包解开——**编解码这件事只许有一份代码**，就是 Runtime 的 `WireCodec`，客户端不许自己再写一份（ADR-060 / R5-03）。
2. **抢跑的沙盘**：客户端手里有两个世界——「确认世界」是服务器说了算的，「预测世界」是老王按了键之后自己先算一步的。老王按右键，预测世界里的人立刻动、按空格炸弹立刻出现（1–3 ms，不等网络）；每收到一包权威状态，预测世界**整个从确认世界重新克隆一遍**，再把服务器还没确认的那几条输入按序重放一遍——猜对了画面不变，猜错了错的那件事自己消失（ADR-064 第 8 条，`gas.md` M7 / `ecs.md` M10）。放弹被拒时，那颗预测炸弹随重建消失，不用改号、不用认领。
3. **画画的人**：表现层只读预测世界，按「表现键」（实体类型 + `fx_key` + 稳定参数）做差：同键继续、键消失结束、新键开始，所以确认包到了画面不闪断；位置用 `ModelTransform` 插值 / 纠偏平滑，`LogicTransform` 永远是服务器的（`movement.md` M8）。

**今天代码里成立的**：传声筒的下半段（WSS 连接、解 FullSnapshot / Delta、`ReplicaWorld` 套 Runtime `WorldManager`）、聊天切片的画面（web/chat 静态页）、Bot 宿主按 NativeCore 节拍发言。**不成立的**：传声筒还是自己手写的 1,079 行 codec，且对着 Runtime 主干编不过；「沙盘」是 Foundation 时代的骨架（`PredictionKey` 窝里 `ConfirmedThroughSeq` 永远是 0，`reconcile` 算出来就丢），没有确认 / 预测两个世界，没有重放；「画画的人」的 `ModelTransform` 采样一行没有。这三件正好对应 R-00408（Client 部分）→ R-00467（C5）→ R-00470（C8）三张卡，顺序也只能是这个顺序。

**两条客户端路线到哪了**：
- **浏览器**：纯 JS 静态页两张（hello / chat）能连宿主收帧，但没有上行、没有预测。CL-1 已实测「.NET 10 browser-wasm 装 Runtime 客户端程序集」可行（桌面 Chrome），推荐路线 A；Owner 已裁 D3（暂定方案 A）与 D27（采纳）。落地的下一步不是新卡，是 **R-00470 AC06 + 三个先决预算**（启动时间上限、AOT 构建 / 体积预算、Runtime IL2075 修复），以及把 CL-1 §5.3 写成架构仓 ADR（D3 「待落文档」尚未落）。
- **Unity / HybridCLR**：0 行源码 + UPM 骨架 + 三张 SPIKE 已作废；LumioGame ADR 0013 已把它降为「后续候选、不建工程、不提需求」。留着空壳等于留一份不会兑现的承诺（D31）。

#### 漂移与问题（按严重度）

① **P0 · R-00481 与 R-00408 同抢 `modules/replica/**`**：R-00481 第一层「出清 GrantClaim、对齐 Runtime HEAD」正是 R-00408 Client 部分的活，而 origin 上 `fix/r5-client-runtime-frames` 分支**已经删掉了两处 GrantClaim** 并接了 `WireCodec`。两张卡同时开就是第四站 D21 的翻版（四处写同一件事）。
② **P0 · main 对真 Runtime 编不过，CI 却绿**：`dotnet-test.yml` 钉死 Runtime `7f198e5`（落后 16 个提交），CI 在验证一个不存在的组合；本地同级 checkout 一编就红。修法不是把 pin 往前挪一格，而是 CI 跟 Runtime `main` 或每次 Runtime 合入后自动 bump（同一件事只在一处维护）。
③ **P1 · R-00408 「验收中」但 origin 上没有完整交付**：五个提交号四个不存在于任何 origin 分支，Runtime 侧零推送，四仓零 PR；R-00392 / R-00393 同批。评论里的 70 / 45 / 18 测试数是 Windows 本地 `5745d11` 的，与 origin 任何提交都对不上（main 上 replica / session / bot 的 `[Fact]` 是 44 / 35 / 12，加 Theory 展开也无法核对）。**另一处副作用**：架构仓 `r5-closeout-20260906` 分支里提交的本报告是 278 行的旧版（缺 §2.3 / §2.4、把 §2.2 落卡措辞改回「待裁决」），**直接合该分支会把第三、四站盘点覆盖掉**。
④ **P1 · 第二份 codec 还在**：1,079 行手写 C-1 JSON 编解码，R5-03 要求删，分支也没删。
⑤ **P1 · CL-1 三条上游缺口的落卡情况**：(a) Runtime 4 处 IL2075（`World.TryReadAccountId`、`WorldManager.FindSyncField` ×2、`TryDispatchSendMessage`）——Workflow 全项目搜索 0 命中、Runtime origin/main 0 处 `DynamicallyAccessedMembers`，**没有卡**；建议落点 = R-00466（RT-3 预测世界重建）追加一条验收项「Runtime 客户端程序集 `PublishTrimmed` 零 ILLink 警告」或 RM-00014 新开一张 Runtime 小卡，不另开 Client 卡；(b) InputCommand 信封公开编码 API——Runtime origin/main 已有 `WireCodec.EncodeInput / EncodePack / DecodePack / DecodeInput`（R-00407 R5-02 done）→ **已闭合**；(c) Rust 宿主 Windows-only + 与 Runtime HEAD 漂移——D24 + R-00408 评论 `01a07403-4634` 已钉口径 → **已落卡**，但 `fix/r5-server-wire-forwarding` 分支上两处 `kernel32` 仍在，未做。
⑥ **P1 · R-00481 正文缺验收项、未指派**，与 R-00478 初建时同一毛病；9 张作废卡无 reason、无评论（D7 / D13 / D15 都有取代说明，这一批没有）。
⑦ **P2 · 旧合同制残留面**：`contract-mirror` 326 文件 1.7 MB + 两套镜像脚本 + smoke 工程 + fixture 语料 pin + 4 个模块的 `Generated*` 适配 / 测试消费它；五文件包 272 KB + 专属 CI 校验步；`.spec/AGENTS.md` 写 V1.2、设计落 `docs/specs/`；13 个模块 README 引旧 v1.2 章节、9 个写「未实现」。比 Server 站轻（没有 6.8k 行守卫与 git 依赖），比 Runtime 站重（多一份完整镜像）。
⑧ **P2 · 已作废卡对应的缺陷仍在代码里**：R-00292 的 worktree 假红（`ProjectGraphTests.cs:27` / `ToolchainPolicyTests.cs:29`）、R-00294 的 `Internal/` 里放 public 类型；T-00004 / T-00005 / T-00009 三张 todo 工作项悬空；R-00415 approved 未随 PR #17 合入流转。
⑨ **P2 · observability 的 `EventDispatcherWorker` 起消费线程**：wasm 单线程下 `new Thread` 直接 `PlatformNotSupportedException`（CL-1 A1-5 实测）；路线 A 落地时这条要改成 owner-thread 泵。

#### 与另一会话草稿的对照（互相吸取）

| 项 | 另一会话草稿（`LumioGameEngine-r479` 副本 §2.5 / D25–D29） | 本会话实测 | 处置 |
| --- | --- | --- | --- |
| HEAD / 0 ahead-behind / CL-1 结论 / RM-00007 19 = 9 / 9 / 1 / C5 · C8 归 Client | ✓ | ✓ 一致 | 采纳 |
| 编译红根因 = `ReplicaWorld.cs` L109 / L451 `GrantClaim` vs Runtime `c2d42b3` | ✓ | ✓ 源码核实一致 | 采纳；**补根因链**：CI 钉 `7f198e5` 所以 CI 假绿（漂移 ②） |
| 「同时 `unity-adapter/tests` 触发 NU1004 lockfile 不一致」 | 有 | `dotnet restore LumioClient.slnx` → All projects are up-to-date，未复现 | 存疑，标「未复现」；可能是对方在 `--locked-mode` 或改过 lock 后跑的 |
| 「contract-mirror 29 个旧锁文件」 | 29 | **1 个锁文件**（`contract-mirror.sha256`，326 行）、326 个镜像文件、1.7 MB；29 是第四站 mvp-host 的数字 | **更正** |
| 「LGE-V1 命中 80+ 文件」 | 80+ | 125（不含镜像 32） | 更正 |
| 「TS / JS / MJS 42 文件 16,377 行」 | 16,377 | 含未入库的 `publish/` 里 .NET 生成的 `dotnet.native.*.js`（两份各 5.5k 行）；`git ls-files` 手写 2,206 行 | 更正口径：只算入库手写代码 |
| 「工作区干净」 | 干净 | `.spec/lumio-agent.lock` 有未提交改动（第三站跨仓修复） | 更正 |
| 「Unity 端加载同一份 CoreEngine / Runtime 动态库或 HybridCLR 模块」（架构演进第 4 点） | 当作现行路线 | LumioGame ADR 0013：首发不接任何游戏引擎、不建工程、不提需求；原则 ②「进预测世界的东西必须能在浏览器里跑」 | **更正**：Unity 是后续候选，不是现行路线；空壳去留另裁（D31） |
| D25 把「GrantClaim 对齐」放进 R-00481 | 新卡第一层 | R-00408 拥有 `modules/replica/**` 且分支已做 | **建议收窄**（D30） |
| §3 的 RM-00011 / RM-00014 行 | R-00388 in_progress、R-00408 Server 分支不在 origin、RM-00014 14 张 | 已过期：R-00388 / 391 rejected（D23）、R-00408 acceptance、R-00392 / 393 in_review、RM-00014 15 张（R-00480）、R-00462 in_review | 以本文件 §3 为准 |
| 纵切 Room 之外的顺带项（R-00415 漂移、T-00004 / 005 / 009 悬空、R-00292 缺陷仍在、第二份 codec、9 个 README 假「未实现」、CI 假绿、observability 线程） | 未提 | 已提 | 本会话补 |
| 三条上游缺口的落卡核对 | 只复述缺口 | (a) 无卡 (b) 已闭合 (c) 已落卡未做 | 本会话补 |
| §1 执行摘要行 | 仍「待盘」 | 本会话填 | 本会话补 |

对方值得学的：Room 出清与卫生一次做完、报告落卡快。本会话值得对方学的：别人的数字先复核（29 / 80+ / 16k 三个数都不对）、写入要留 reason 与验收项、新卡先查与在途卡的文件集重叠。

#### 阶段判定

一句话：**Foundation 骨架 + 聊天切片客户端在 origin 成立，但 main 对着真 Runtime 编不过（CI 因钉旧 pin 而假绿）、传声筒还是自己写的 1,079 行 codec、预测沙盘只有 Foundation 时代的空架子、表现采样一行没有；浏览器路线经 CL-1 证实可行且已裁 D3 / D27，落地点就是 R-00470；Unity 只剩空壳；RM-00007 旧卡已出清，新卡 R-00481 与在途 R-00408 撞了文件集、缺验收项；R-00408 「验收中」但 Runtime 零推送、四仓零 PR。**

#### 剩下的单子怎么补（待 Owner 裁决；写 Workflow 须逐次授权）

| 卡 | 动作 | 范围 | 依据 |
| --- | --- | --- | --- |
| **R-00481** | **D30 待裁**：建议去掉第一层（GrantClaim / Replica 对齐归 R-00408 Client 分支），改为纯清理三层并按 card-spec 补约 10 条验收项 + 指派：① 删 `docs/architecture/` 6 md + `.baseline.sha256`、`docs/LumioClient_five_requested_files/` + `eng/verify-five-file-package.*`、`docs/specs/`；② 删 `contract-mirror/` + `eng/sync-/verify-contract-mirror.*` + `eng/upstream-contract-smoke` + `tests/Fixtures/index.json` + `eng/upstream-api-map.md`，4 个模块的 `Generated*Adapter` / `Generated*FixtureTests` 改消费架构仓 `engine/wire` 或删；③ CI 删 v1.4 / sha256 / `LGE-V1.2` 断言与五文件、镜像两步，`dotnet-test.yml` 改跟 Runtime `main`；④ README / `.spec/AGENTS.md` / `repository-architecture.md` / 13 个模块 README 改 Living Architecture 口径（设计落 `.spec/knowledge/features/`），「阶段」按实况重写；⑤ 顺带并入 R-00292（枚举改 `git ls-files`）与 R-00294（public 类型出 `Internal/`） | 只做清理，不碰 `modules/replica/**` 生产代码 | D1 / D8 / D12 / D21 同路；漂移 ① ② ⑦ ⑧ |
| R-00408 | 补差异记录评论（不改状态）：评论所引 Client `5745d11` / Server `f031afb` / Game `d3f6fe0` 不在 origin，Runtime `18b5feb` 未推送，origin 上对应分支 tip 是 `5652b95` / `deebeb4` / `e243a39`；解铃 = Runtime 推送 + 四仓开 PR、CI 绿、合入后重核；QA 此前不得 → done。Client 侧验收补一条「`GameplayCodec.cs` / `LiteJsonParser.cs` 删除，生产代码只经 `WireCodec`」 | 评论 | 漂移 ③ ④；td-progress-audit 步骤 3 |
| 架构仓 `r5-closeout-20260906` 分支 | **不得直接合**：其 `2026-09-05-engine-repos-progress-assessment.md` 是 278 行旧版；合入时只取 `engine/wire` / `verify-wire` / 两份 reviews 新文件，报告以 main 工作区为准 | 合入纪律 | 漂移 ③ |
| R-00466（RT-3） | 追加验收项：Runtime 客户端程序集 `PublishTrimmed` 零 ILLink 警告（修 4 处 IL2075 或加 `DynamicallyAccessedMembers`） | 评论 + 验收项 | 漂移 ⑤(a)；CL-1 §5.3 第 5 条 |
| 架构仓 ADR | 新建「浏览器客户端预测走 .NET WebAssembly 装载 Runtime 客户端模块（桌面）；手机待真机」（CL-1 §5.3 草案，D3 / D27 已裁未落文档）；`gas.md` M7 / `ecs.md` M10 补一句浏览器载体 | 文档 | D3「待落文档」 |
| T-00004 / T-00005 | 作废（reason：MVP C# 宿主已随 D22 判删；Bot.Host 远程接入由 R-00389 实现） | 流转 | 漂移 ⑧ |
| R-00415 | 流转 acceptance → done 或至少 acceptance（PR #17 `06db121` 在 origin/main） | 流转 + 证据评论 | 仓库领先 |
| unity-adapter / hybridclr-adapter / `packages/com.lumio.client` / T-00009 | **D31 待裁（D30 之后再问）**：删空壳 + UPM 骨架 + 作废 T-00009，README 子模块表同步；或留空壳 | — | ADR 0013；原则 ① ③ |

不开的卡及原因：「浏览器 WASM 客户端壳」——已是 R-00470 AC06 的内容；「Client Chunk 三态 / RTT 校正」（R-00296 / 298）——体素不进预测世界、不下发客户端，Stage 0 用不上。

### 2.6 LumioGame（第六站 · 游戏玩法仓 / 游戏主工程）

> 本节由两个会话合成：另一会话先按五步写出草稿并执行了 Workflow 出清（见 D32；该会话在 Workflow reason 与其草稿里把这次裁决编为「D30」，本报告主副本的 D30 / D31 已被 Client 站占用，故改记 D32）；本会话独立实跑后逐条核对，采纳其方向与已执行动作，更正其数字与「已落地」类陈述，补上它漏掉的三处（G-0 契约与架构第二样板的结构性漂移、R-00408 四仓提交不在 origin、CI 没有构建作业）。逐条对照见本节末「与另一会话报告的对照」。

#### 仓库侧事实（本机 macOS，2026-09-06 实跑）

| 项 | 实测 | 出处 |
| --- | --- | --- |
| origin/main = 本地 main | `5bc5afc`（2026-09-04 23:12 +0800，`Merge branch 'docs/bomber-terrain-align-voxel-contract'`），`git fetch --all --prune` 后 0 ahead / 0 behind；工作区除 `.spec/lumio-agent.lock` 外干净。盘点开始时本地有 7 条非 main 分支（6 条已合入 + `claude/confident-dhawan-567439` = PR #1 squash 合入前的 tip，内容已在 `39f88c9`）与 3 个 08-28 / 08-29 的游离 worktree（全部已合入、`status --short` 0 行脏文件）；另一会话 08:2x 已删 3 个 worktree 与 6 条本地分支，现只剩 `main` + `claude/confident-dhawan-567439` | `git status -sb`、`git branch -a`、`git worktree list`、逐条 `merge-base --is-ancestor` |
| 远端分支（未动） | 9 条非 main：6 条已合入（`docs/bomber-v04-stage0-adr`、`feat/bomber-stage0-kernel-contract`、`feat/r-00348-chat-component`、`feat/r-00373-chatcomponent-runtime`、`feat/r-00376-eleven-scenarios`、`claude/confident-dhawan-567439`〔squash〕）；`Go1c-patch-1`（PR #12 OPEN、`mergeable: CONFLICTING`，只改 `design.md` §1 一句措辞，`Go1c` 是 Owner 本人账号）；**`fix/r5-entity-chat-scenarios`（`e243a39`，2026-09-05，4 个提交 +625 / −445，未合入）= R-00408 的 Game 半边** | `git branch -r`、`gh pr list / view 12`、`git diff --stat origin/main...` |
| 语言与规模 | C# **47 文件 3,786 行**（不含 `bin/` `obj/`）：手写炸弹人契约 14 文件 252 行 + 生成物 8 文件 1,056 行（Runtime `gen-declarations` 产出）+ 聊天 / entity-chat 生产代码 1,685 行 + 测试 7 文件 793 行；集成脚本 JS / MJS 12 文件 4,736 行（`integration/hello` + `integration/entity-chat`）；tracked Markdown 98 个（`docs/` 下 25 个 6,605 行）。`.csproj` 6 个：`server-gameplay` 下 3 个生产（ServerGameplay / EntityChat.Protocol / EntityChat.Suite）+ 1 个测试；**`config`、`scenario` 两个工程 0 个 `.cs`，是 G-0 留下的空骨架** | `find … -name '*.cs' -not -path '*/obj/*' \| xargs wc -l`、`git ls-files '*.md'`、`find . -name '*.csproj'` |
| 门禁 1（规范与集成脚本） | `node .spec/tools/spec-lint.mjs` → OK；`node --test .spec/tools/spec-lint.test.mjs` → 13 / 13；`node --test integration/entity-chat/{verify-evidence,bot-credential,web/chat-window.test}.mjs` → **17 / 17**（另一会话跑前两个文件得 15 / 15，口径一致） | 实跑输出 |
| 门禁 2（编译与单测） | `dotnet test LumioGame.sln` → **编译失败 exit 1**：`ChatSetMessageSystem.cs(62,22): error CS1061: 'WorldManager' does not contain a definition for 'TryGetSession'`（net10.0 与 netstandard2.1 各一次）。**根因**：同级 `LumioGameRuntime` 在 `c2d42b3`（2026-09-04 20:08 +0800，「complete r5 world manager projection」）删了 `BindSelf / TryGetSession / UnbindSession / GrantClaim / HasClaim`，改为 `Bind / Unbind(NetEntityId)` + `ResolveBindingMessage` 查询；Game main 之后的两次合并（22:14、23:12）都没跑 `dotnet test`。**CI 只有一个「README policy」作业，没有任何 build / test 作业**，所以 main 编不过而 GitHub Actions 09-04 四次运行全绿 | `dotnet test` 日志；`git -C LumioGameRuntime log -S TryGetSession`；`.github/workflows/repository-policy.yml`；`gh run list` |
| 修复是否已在 origin | `fix/r5-entity-chat-scenarios` 删 `InputCommandEnvelope.cs`（245 行）与其测试、新增 `RuntimeDrainConsumer.cs` + `RuntimeCodecBoundaryTests` / `RuntimeDrainConsumerTests`。用 `git archive` 快照到会话目录、`LUMIO_RUNTIME_ROOT` 指向 Runtime `89a7a6c` 实跑：**`dotnet test` 25 / 25、node 21 / 21、spec-lint OK**。但 R-00408 最终评论点名的 Game 提交是 `d3f6fe0`，**不在 origin 任何 ref 上**；`e243a39` 是否就是它的推送版无法判定 | 快照实跑日志；`git cat-file -e d3f6fe0` |
| 跨仓依赖 | 只 `ProjectReference` 同级 LumioGameRuntime 的两个工程：`modules/ecs` **和 `modules/ecs/samples/username`（Username.Server 样例）**——ChatComponent 的真源在 Runtime 样例里（R-00390 / ADR 0009 定的），Game 生产程序集引用引擎**样例**工程 18 处；`EntityChat.Protocol/AccountServerProcess.cs` 拉起同级 LumioServer 的 `lumio-account-server.dll`（D22 已裁决删，ADR-061 账号归 Platform）。不引用 Server / Client / NativeCore / VoxelEngine 源码，边界正确 | `Lumio.Game.ServerGameplay.csproj`、`grep -rn Samples.Username`、`AccountServerProcess.cs:43` |
| 旧合同制残留 | `git grep -l LGE-V1` 18 个 tracked 文件；`docs/architecture/` 8 份正文 + `ADR_INDEX.md` + `.baseline.sha256` 共 279 KB（v0.3 / 1.0 / 1.1 各 428 B 空壳；v1.2 / 1.3 / 1.4 各 40–48 KB；两份 V3 review 62 KB + 72 KB；另有 `5.5Max-ReviewV3/` 52 KB）；CI 断言 v1.4 正文、`LGE-V1.4-2026-08-27` 字样与 `sha256sum -c`；README「架构基线 LGE-V1.4 / 唯一架构源 `LumioGameEngineArchitecture` / `python3 tools/lumio_contract.py validate`」、`.spec/AGENTS.md`（LGE-V1.4、设计落 `docs/specs/`、计划落 `docs/plans/`）、`repository-architecture.md`、`module-scaffolding-design.md`（10 个子模块蓝图，实际建了 3 个）、`mvp-placevoxel-content-spec.md`（V1.4 时代挖 / 放方块切片，已被炸弹人切片取代，ADR-063 第 13 条）、`server-gameplay/README.md`「架构基线 LGE-V1.4」；仓根两份 tracked 的 `.wf-report-*.md`。**相对前五站是残留面最小的仓**：没有 contract-mirror、没有 generated 合同树、没有守卫工具 | `git grep`、`ls -l docs/architecture`、各文件实读 |
| 炸弹人 G-0（仓库已交付） | `5348635 → 10782b6 → b6f0c38 → cdcca47 → 51c7829` 五次合并（09-04），契约 `stage0-kernel-contract.md` 冻结 v1.3.0，`Bomber/Contracts/**`（不含 generated）14 文件 sha256 `d16de07a…` **本机复算一致**；5 个组件（MatchState / PlayerState / BombState / HatPile / PickupItem）、5 个 EntityType、11 个事件、2 个命令、2 个端口、生成物 8 文件；探针测试 3 条（自定义注册表、5 种实体入快照、两个独立世界逐字节同哈希）。**规则代码为零**：`git grep ITerrainStore\|InMemoryChunkStore\|class .*System` 在 `modules/` 0 命中；移动 / 放弹 / 爆炸 / 帽子 / 掉落 / 地图 / Config / Scenario / Bot / 遥测一行都没有 | `shasum -a 256`、`git grep`、`RuntimeIntegrationProbeTests.cs` |

#### Workflow 对账（RM-00008 + 四个纵切 Room；只读，写入见 D32）

- **RM-00008（LumioGame 单仓 Room；Room 描述仍是 08-27「MVP 挖 / 放方块」V1.4 口径）**：盘点开始时 6 张 = 1 done（R-00259 脚手架设计文档）/ 1 acceptance（R-00283 美术三方向比稿，08-29 起等 Owner 终裁，卡面明写「终裁前本卡停在验收中」——是等待不是漂移，但已等 8 天）/ 4 backlog（R-00310 ~ R-00313 GAS-G01 ~ G04，08-30 建，18 条验收项 not_started，正文乱码，前置是 V1.4 挖 / 放方块切片与已退役的 R-00301 / R-00159 / R-00172，与 D2 作废的 R-00302 / 308 / 309 同族）。**另一会话 2026-09-06 00:28:55 – 00:29:04Z 执行**：4 张 → rejected；新建 **R-00482**《[程序·工程] 退出旧合同制残留、合并 R5 修复分支恢复编译并出清旧技术单据》，P0 / backlog，四节正文 994 字，**0 条原生验收项、0 评论**（验收只写在正文 checkbox 里——与 R-00478 建卡时同一缺陷）。本会话读回：7 张 = 1 done / 1 acceptance / 4 rejected / 1 backlog。
- **RM-00013（体素炸弹人 Stage 0）**：10 张全 backlog，最后写入 2026-09-04 14:14Z（Room 与卡两边都没有别的会话写入，无踩踏）。R-00422 来源卡（附件 v0.4 建议稿）；**R-00423 G-0 仓库领先**——交付已在 main、契约 v1.3.0，卡仍 backlog、5 条验收项 not_started、0 评论；R-00424 ~ R-00430（G-1 ~ G-7）+ R-00431（C-1，架构仓的卡放在这个 Room）共 37 条验收项 not_started；S-1 / K-1 / I-1 / I-2 / REV-1 仍在本地 bundle（`.workflow-drafts/wf-20260904-bomber-stage0/`）未落单。G-4 卡面前置写「契约 v1.2.0」，仓内已 v1.3.0（ADR 0019）。**卡面与架构仓 09-05 裁决的结构性差异**见下一小节。
- **RM-00010（Hello World）**：9 / 9 done；R-00343（Game 集成验收）9 条验收项全 passed，证据归档 `integration/hello/evidence-run1/`（400 KB，含 Windows 绝对路径，属证据正常）。
- **RM-00011（ECS 正式实体与聊天）**：R-00390（Game 删第二份 ChatComponent）done。**R-00408 验收中（2026-09-06 00:11Z）、4 条验收项 passed，但最终评论点名的 Runtime `18b5feb` / Server `f031afb` / Client `5745d11` / Game `d3f6fe0` 四个提交都不在各仓 origin 任何 ref 上**（评论自述「no push, PR, deployment」），架构仓 `9a22527` 只在 `origin/r5-closeout-20260906` 分支——**Workflow 领先、origin 无交付**；Game 侧 origin 只有 `e243a39`。解铃条件：四仓推送 + Game 合入 main 后重核。R-00392 / R-00393 同一时段转 in_review（另一会话在做 Server 站收口）。
- **RM-00014（九项必备）**：没有 Game 卡（九项都是引擎侧），但 **R-00462（RT-1 Tick 统一，评审中）、R-00468（RT-4 GAS M2）、R-00480（RT-5 GAS 瞬时效果）、R-00469（E7 体素批量读写）是 G-1 / G-6 的真正前置**；R-00461（LogicTransform）done，代码在 Runtime main `11d34e1`。

#### 架构演进与前线现状（炸弹人通俗透视）

老王开一局炸弹人，从进房到重生要走七步。每一步「架构仓 09-05 怎么定的」「Game 09-04 冻结的契约怎么写的」「今天代码里有什么」三列对照：

| 步 | 老王在干什么 | 架构仓定的（`bomber-slice.md` 第二样板 / ADR-063 / 064） | Game 契约 v1.3.0 写的（09-04，ADR 0015 / 0016 / 0019） | 代码 |
| --- | --- | --- | --- | --- |
| 1 | 进房，站在格子上 | 位置只有一份真值 `LogicTransform`（Runtime 已交付 `11d34e1`），客户端另有 `ModelTransform` 插帧 | `BomberPlayerState` 自带 `CellX/Y/Z` + `PosMilliX/Y/Z` 六个 `Sync<int>`；`movement.md` §9.1 已写好五步迁移清单 | 字段有，无读写方 |
| 2 | 按方向键 | 移动是 GAS Ability，共享文件两端跑，客户端预测世界先算 | `MoveIntent(DirX, DirY)` 是普通 DTO，由 Game 自己的函数处理 | 无 |
| 3 | 按放弹 | 放弹是 Ability，准入五步（手上炸弹数 / 这格能不能放），预测世界里 `Commands.Create<炸弹>()` | `PlaceBombIntent` DTO；`BombCapacity` 是单账 `Sync<int>` | 无 |
| 4 | 2.1 秒后引信到点 | `[System(Phase.ProcessorPlan)] 爆炸系统` 注册进 13 相第 4 相（RT-1 = R-00462 评审中）；帧初批量读整图、连锁同帧、帧末一批写体素 | 规则内核是普通 C# 函数，Scenario 宿主在 `WorldManager.Tick()` 前后手调（因为当时 Runtime 无系统注册面——ADR 0015 §0 ②）；地形走 Game 自有 `ITerrainStore` + `InMemoryChunkStore`，Stage 2 再换实现（ADR 0016 / 0019） | 无 |
| 5 | 木箱变空气、老王掉血 | 木箱是**体素**（官方全局段 8 种 Solid + 水）；伤害是瞬时 Effect 单，改基础账、当前账相尾重算一次，击杀 = 跨零 | 木箱是 `InMemoryChunkStore` 里的 `BlockId`；`HealthPoints` 单账 `Sync<int>`，`DamageApplied / PlayerDied` 事件 DTO，同弹命中记忆是服务端临时集合 | 事件 DTO 有 |
| 6 | 所有人同帧看到爆炸 | 复制 + `OnFx`；表现键 `(实体, fx_key, 参数)` 做差，同键继续、键消失结束、新键开始 | 炸弹实体带四臂长度（ADR 0017，与架构一致）；表现层 K-1 未落单 | 无 |
| 7 | 死了掉帽、3 秒重生 | 死亡系统下一帧下结构单；重新入场走身份三态 | `HatPile` 实体 + `RespawnAtTick`（与架构一致） | 无 |

**一句话**：Game 09-04 冻结的契约，是照着「Runtime 当时没有系统注册面、没有体素端口、没有 GAS」这三个缺口（ADR 0015 §0 核验 ②③⑤ 不可行）**反向设计出来的一套游戏仓自备引擎**——自己的 Tick 编排、自己的地形存储、自己的位置字段、自己的属性单账。09-05 架构仓把这三个缺口全部立成 Runtime 卡（RT-1 / RT-4 / RT-5 / E7），并把「静态必须是体素、预测一律经 GAS」写进 `rules/system.md` 世界模型红线。两边各自都没错，但**按现卡派 G-1 ~ G-7，就是在游戏仓造第二条 Tick 路径、第二份地形真值、第二份位置真值，RT-1 / E7 落地那天整套推倒**——正是 `lessons.md`「上游更正一句，同节陈述全复跑」那条：契约是 09-04 一次性快照，09-05 上游改了地基。第 6 / 7 步（炸弹实体持火焰、帽堆实体、重生）两边一致，不用动。

#### 漂移与问题（按严重度）

① **P0 · main 编不过，CI 看不见**：上游 `c2d42b3` 破坏性漂移 + CI 无 build / test 作业；修复在 origin 分支未合入（`e243a39` 实测绿）。
② **P0 · G-0 契约与架构第二样板五处结构性差异**（位置 / 属性 / 移动放弹 / Tick 挂接 / 地形），G-1 ~ G-7 的 37 条验收项按旧口径写。
③ **P1 · R-00408 验收中但四仓提交不在 origin**（不可复核，解铃条件 = 推送）。
④ **P1 · R-00423 仓库领先**（G-0 已交付，卡 backlog、0 评论、5 条验收项未动）。
⑤ **P1 · R-00482 建卡缺陷**：0 条原生验收项；范围只到 `docs/architecture/` + CI 字样，漏了 CI 加真实 build / test 作业、README / `.spec/AGENTS.md` / `repository-architecture.md` / `module-scaffolding-design.md` / `mvp-placevoxel-content-spec.md` / `server-gameplay/README.md` 改口、两份 `.wf-report`、远端 7 条分支、PR #12。
⑥ **P2 · entity-chat 套件两条外部腿都踩在将被拆的地板上**：Game 生产程序集引用 Runtime **样例**工程；`AccountServerProcess` 拉起 Server 的 account-server（D22 要删）。炸弹人切片取代聊天切片后，这套件的去留要有说法。
⑦ **P2 · 旧合同制口径**（数字见上表）。
⑧ **P2 · 卫生**：远端 6 条已合入 + 1 条 squash tip；PR #12 CONFLICTING；本地 `claude/confident-dhawan-567439`。
⑨ **P2 · R-00283 等 Owner 终裁 8 天**（美术比稿，引擎盘点范围外，只提醒）。

#### 与另一会话报告的对照（互相吸取）

| 项 | 另一会话 | 本会话实测 | 处置 |
| --- | --- | --- | --- |
| 代码量 | C# 63 文件 4,046 行 | 把 `obj/` 下 16 个生成文件算进去了；47 文件 3,786 行 | **更正** |
| `docs/architecture/` | 「6 版旧 md 152 KB」 | 数字从 §2.5 抄来的；实为 8 份正文 + ADR_INDEX + sha 共 279 KB，另有 52 KB `5.5Max-ReviewV3/` | **更正** |
| 门禁 2 | 15 / 15 | 跑 2 个文件 15 / 15、跑 3 个 17 / 17 | 一致 |
| 「ITerrainStore 落地」「表现键做差呈现」「Stage 0a 采用 InMemoryChunkStore 桩」 | 写成已落地 | `git grep` 0 命中，全是设计不是代码 | **更正**：设计与落地分开写 |
| 组件清单 | 4 个 | 5 个（漏 `BomberMatchState`） | 更正 |
| RM-00013「近 42 小时无写入」 | 42 h | 09-04 14:14Z → 09-06 00:3xZ ≈ 34 h | 更正（结论不变） |
| 阶段判定「待 R-00482 解阻后正式启动 G-1 ~ G-7」 | 只看编译 | 漏了 ②——契约漂移才是 G-1 ~ G-7 的真前置 | **补 D33** |
| RM-00008 4 张作废 + R-00482 | 已执行 | 读回一致（00:28:55 – 00:29:04Z） | 采纳；待 Owner 追认（D32）；R-00482 补验收项与范围 |
| worktree / 本地分支清理 | 已执行 | 读回一致 | 采纳；远端 7 条 + PR #12 未处理 |
| R-00408 Game 半边 = fix 分支 | 「待合入」 | 补：`d3f6fe0` 不在 origin，`e243a39` 快照实测 25 / 25 | 采纳并补 |
| R-00423 仓库领先、CI 无 build 作业、样例 / account-server 依赖、旧合同制口径清单 | 未核 | 已核 | 本会话补 |

对方值得学的：先把 Room 出清、卡建起来，动作快。本会话值得对方学的：数字用同一条命令跑出来并附命令；「落地」只能指 origin 上的代码；契约要对着**上游最新裁决**复核，不是对着卡面自述。

#### 阶段判定

一句话：**炸弹人只交付了契约壳（5 组件 / 11 事件 / 探针 3 条），规则代码为零；这份契约是照 Runtime 09-04 的缺口反向设计的「游戏仓自备引擎」，与架构仓 09-05 定的第二样板在位置 / 属性 / 技能 / Tick 挂接 / 地形五处结构性不同；main 因上游漂移编不过而 CI 看不见；旧合同制残留面是六站里最小的；RM-00008 旧卡已出清、W0 卡 R-00482 已建但缺验收项与范围。**

#### 剩下的单子怎么补（待 Owner 裁决 D33；写 Workflow 须逐次授权）

| 卡 | 动作 | 范围 | 依据 |
| --- | --- | --- | --- |
| R-00482（W0） | 补 8 条原生验收项 + 范围扩到三层 | ① 前置改为「R-00408 实现方把 Game 提交推送并合入 main」（`e243a39` 已实测绿可作兜底，W0 不替 R-00408 做活）；② 删 `docs/architecture/` 全部 + `.baseline.sha256`、两份 `.wf-report-*.md`、`mvp-placevoxel-content-spec.md`；③ CI 删 v1.4 / sha256 断言，**新增 `dotnet build` + `dotnet test` + node 三件套作业**；④ README / `.spec/AGENTS.md`（设计落 `.spec/knowledge/features/`）/ `repository-architecture.md` / `module-scaffolding-design.md` / `server-gameplay/README.md` 改 Living Architecture 口径（契约源 = `LumioGameEngine/engine/wire`）；⑤ 远端 7 条已合入分支 + 本地 1 条删，PR #12 关闭或重提一行 | D1 / D8 / D12 / D21 / D25 同路；漂移 ①⑤⑦⑧ |
| R-00423（G-0） | 流转「验收中」+ 证据评论（`5348635…51c7829`、sha `d16de07a…`、09-04 26 / 26；注明今天 main 因上游漂移红、随 R-00482 解阻后 QA 跑 5 条验收项） | 既有卡 | 漂移 ④；纪律 ⑦ |
| **G-0 v2（新建一张，RM-00013）** | 契约按架构第二样板重冻：位置 = `LogicTransform`（照 `movement.md` §9.1 五步迁移）、属性 = `AttributeComponent` 两本账、移动 / 放弹 = Ability、爆炸 / 死亡 = `[System(Phase.ProcessorPlan)]`、地形 = 引擎体素批量读写；**删 `ITerrainStore` / `InMemoryChunkStore` 设计与 `PosMilli` / `Cell` 字段**；炸弹实体持火焰 / 帽堆 / 重生三处不动 | 前置 R-00461（已 done）；接口对齐 R-00462 / R-00468 / R-00469 卡面 | 漂移 ②；世界模型红线；ADR-063 第 13 / 14 条 |
| G-1 ~ G-7（R-00424 ~ 430） | **不按现卡派**；G-0 v2 冻结后按引擎接缝分两批回写正文与验收项：**批 A（接缝无关，可先派）** G-4 地图数据 + 断言、G-5 Config、G-7 遥测 Schema、G-2 帽子经济纯函数；**批 B（等引擎）** G-1 规则内核（等 RT-1 / RT-4 合入）、G-6 Scenario 宿主（等 RT-1）、地形读写（等 E7 = R-00469） | 评论回写，不新建重复单 | 同上 |
| R-00408（RM-00011） | 补差异记录评论：四仓提交不在 origin，解铃条件 = 推送后重核；状态不动 | 既有卡 | 漂移 ③；td-progress-audit 步骤 3 |
| entity-chat 套件（`EntityChat.Protocol` / `Suite` / `integration/entity-chat`） | 随 D22 Server 删 account-server 时同步改为拉 Platform 账号服或测试替身；样例工程引用在 RT-2 样板落地后改指 Runtime 正式组件 | 落进 R-00482 的 known gaps，不另开卡 | 漂移 ⑥；ADR-061 |
| R-00283 | 提醒 Owner 终裁；引擎盘点不动 | — | 漂移 ⑨ |

不开的卡及原因：S-1 / K-1 / I-1 / I-2 / REV-1（bundle 里的 5 张）继续留本地——S-1 等 R-00465、K-1 等 R-00467 / R-00470 与 CL-1 结论（D27），I-1 / REV-1 等批 B。


### 2.7 LumioConfig（待盘）

### 2.8 LumioPlatform（待盘）

## 3. Workflow 现状（Room × 状态；只填已核对的）

| Room | 名称 | 总数 | done | in_progress | acceptance | backlog | 备注 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| RM-00002 | LumioNativeCore | 71 | 68 | 0 | 0 | 3 | 3 张 backlog 为 GAS 阶段 2 卡；21 条验收项 not_started（9 条挂在 done 卡） |
| RM-00003 | LumioVoxelEngine | 68 | 29 | 0 | 10 | 2 | 27 张已否决（27 张 V1.4 旧卡按 D7 作废）；蓝图 12 张已归入本 Room；backlog 为 R-00433（来源卡）与 R-00474（W0 三层清理，10 条验收项） |
| RM-00005 | LumioGameRuntime | 41 | 20 | 0 | 1 | 0 | 20 张旧蓝图卡已作废（D13）；R-00475 在「验收中」——卡面已按 D12 重写为三层 + 8 条验收项，但其交付提交 `dc37644` 不在 origin，QA 须待推送后再判；12 张 T-00010 ~ T-00021 工作项已全部「已取消」（D18，逐卡取代说明），Room 工作项 active = 0 |
| RM-00010 | Hello World 纵切 | 9 | 9 | 0 | 0 | 0 | 全部 done；R-00340 Server 卡对应 `world.rs` / `server.rs` |
| RM-00007 | LumioClient | 19 | 9 | 0 | 0 | 1 | **9 张旧卡已作废（另一会话 2026-09-06 00:14 UTC 执行，D26，无 reason / 评论）**：R-00055、R-00253 / 254 / 255、R-00289、R-00292、R-00294、R-00296、R-00298；新建 R-00481（00:15 UTC，D25）backlog、**0 验收项、未指派、与 R-00408 撞 `modules/replica/**`**（D30 待裁）；工作项 5 = 3 done + T-00004 / T-00005 todo（已无效，待作废）；缺验收项需求 7、验收项未通过 56 条全在 done / rejected 卡上 |
| RM-00008 | LumioGame | 7 | 1 | 0 | 1 | 1 | R-00259 done；R-00283 acceptance（美术比稿等 Owner 终裁 8 天）；R-00310 ~ 313 GAS 旧卡已作废（D32，Owner 已追认）；R-00482 W0 卡 backlog，已按 D33 重写正文 + 8 条原生验收项（读回 8 / 8） |
| RM-00006 | LumioServer | 68 | 15 | 0 | 0 | 1 | 52 张已作废（另一会话 2026-09-05 16:28 UTC 执行，Owner 已追认 D15）；R-00478 backlog，正文已于 2026-09-05 23:50Z 按 D21 / D22 重写、11 条验收项 not_started（读回 11 / 11）；R-00260 / R-00276 已 done 无证据评论；验收项未通过 319 条全在已作废卡上 |
| RM-00011 | ECS Formal Entity and Chat | 42 | 37 | 0 | 1 | 0 | R-00407 done；R-00388 / R-00391 rejected（D23）；**R-00408 → acceptance、R-00392 / R-00393 → in_review（另一会话 2026-09-06 00:11–00:12 UTC 写入）**：4 条验收项 passed，但所引 Runtime `18b5feb` 未推送、Client / Server / Game 提交号与 origin 新分支 tip 不一致、四仓零 PR——差异记录待补，done 前须重核（§2.5 漂移 ③）；started 3 = 1 acceptance + 2 in_review |
| RM-00013 | 体素炸弹人 Stage 0 | 11 | 0 | 0 | 1 | 10 | R-00423（G-0 v1）已按仓库领先纠偏转「验收中」+ 证据评论；**新建 R-00483（G-0 v2，D33）** backlog、7 条验收项、引用 7 条；G-1 ~ G-7 + C-1 各 1 条分批评论（批 A：G-2 / G-4 / G-5 / G-7，批 B：G-1 / G-3 / G-6），状态 backlog 待 v2 冻结；S-1 / K-1 / I-1 / I-2 / REV-1 仍在本地 bundle |
| RM-00014 | 九项必备能力 | 15 | 1 | 0 | 0 | 13 | R-00461 done（已补证据评论 + 验收判定 1 passed / 4 blocked）；R-00462 in_review（RT-1 正文已回写）；R-00480（RT-5）新建；Client 两张 R-00467（C5，受 A0 约束）/ R-00470（C8，不受 A0 约束，前置 R-00461 + R-00466）各 6 条验收项 not_started、0 评论；started 1 = R-00462 |
| （无 Room） | 已否决的重复卡 | 11 | 0 | 0 | 0 | 0 | 只剩 R-00432 与 10 张 9 月 4 日重复建出的已否决卡 |
| RM-00001 | LumioGameEngineArchitecture · 架构仓 | 23 | 10 | 0 | 4 | 9 | 验收中 4 张为体素 ABI / 托管 / 聚合根卡（R-00439 / 443 / 445 / 456，已归入）；新建 R-00476（sdk-native 去 V1.4 名字）、R-00477（物理三槽路由）、R-00479（契约三处缺陷）；其余 backlog 为 GAS 架构卡族等，待架构仓一站盘 |

## 4. 漂移对照与证据核验结果

见 §2.1「漂移与问题」①–⑦、§2.2「漂移与问题」①–⑨、§2.3「漂移与脱节分析」①–③ 及 §2.4「漂移与问题」①–⑧。前三站与本会话的第四站盘点**未做任何未经授权的 Workflow 写入**；第四站发现另一会话已执行 RM-00006 的写入（52 张作废、2 张 done、新建 R-00478），逐笔时间与操作人见 §2.4 与 §7，Owner 2026-09-06 已追认（D15）。

第六站：见 §2.6「漂移与问题」①–⑨。**不可复核清单**：R-00408 最终评论点名的 Runtime `18b5feb` / Server `f031afb` / Client `5745d11` / Game `d3f6fe0` 不在各仓 origin 任何 ref（解铃条件：推送后 `git ls-remote` + `show --stat` 重核，Game 侧另核 `e243a39` 是否同源）；架构仓 `9a22527` 只在 `origin/r5-closeout-20260906`。本会话第六站**未做任何 Workflow 写入**；另一会话的写入（D32）逐笔读回一致，待 Owner 追认。

## 5. 关键路径与下一阶段 wave 编排（待全部仓盘完后补）

## 6. 风险与开放决策

| # | 决策 | 建议方向 | 依据 |
| --- | --- | --- | --- |
| D1 | NativeCore 旧合同制残留：彻底清 vs 先留着 | **已裁决（Owner 2026-09-05）：三层全清，一张卡做完**——① 复印件目录 / README 合同口径 / CI 校对步骤 / 旧仓名；② `lumio-contract-types` 旧合同布局门与 xtask 生成命令、`lumio-native-ffi` 整 crate；③ kernel 内旧合同错误码 1044–1053 与 capability 注册表键，跨边界映射改由 SDK 插头对 `native-abi.json` 状态码 | ADR-059；`architecture.md` §7-4；Owner 原则「底层只保留唯一、最干净、最解耦、最引擎的版本」 |
| D2 | 3 张 GAS-on-NativeCore 卡：派 / 挂起 / 作废 | **已裁决（Owner 2026-09-05）：作废** R-00302 / R-00308 / R-00309——前提已失效（新建帧调度器 crate 与 `lumio-timer` 重复；ADR-064 把 Rust 下沉推到阶段 2），正文编码损坏，无保留价值。阶段 2 真来时按新前提重开。gas.md 补哪一句（「GAS 到期调度在 Runtime 按帧计数」还是「帧调度器 = 定时内核扩展」）**等 D3 裁决**。同族另 9 张卡（架构室 R-00299 ~ 301、Runtime 室 R-00303 ~ 307）留待盘对应 Room 时处置 | ADR-064；如无必要勿增实体 |
| D3 | 浏览器客户端没有 Native：「单一定时内核」还成立吗 | **暂定（Owner 2026-09-05）：方案 A**——进预测世界的东西必须能在浏览器里跑，今天只有 Runtime C# 满足；预测世界里的到期是实体字段「第几帧到期」与当前帧号比较（炸弹 `FuseEndTick` 即此），不是定时器，不需要第二套内核；Native 定时内核只管宿主节拍（服务器推帧、Bot 节奏、断线保留窗）。两套并存自选回退被否（两套实现只在写出来那天一致；违反如无必要勿增实体 / 不留兼容层 / ADR-056 单一内核 / 确定性）。以后重计算内核进网页走「同一份 Rust 编成 WASM」，仍是一套；「参考 + 优化」双实现只允许用于有性能需求的重计算，编译期定死、逐字节差分测试。**待落文档**：gas.md M9 一节与 8 月 30 日裁决板 11a 改口；ADR-064 追加修订记录 | CL-1 调研；ADR-064 第 1 条；代码实测（Runtime 零处使用 Native timer） |
| D4 | 空间粗筛内核（DS M5）何时开卡 | **已裁决（Owner 2026-09-05）：B**——现在不开卡、不删；`lumio-spatial` / `lumio-job` 留作 M5 零件并标「形状待按 M5 契约改」；盘到 Server 仓看视野下发排期时，先开契约卡再开实现卡 | ds-server.md M5；tick 第 6 相；炸弹人 Stage 0（19×19、8 Bot）用不上 |
| D5 | `lumio-native-ffi` 整 crate 去留 | **已并入 D1 裁决：整 crate 删** | 漂移 ① |
| D6 | codec / diagnostics 两个默认关的私有原型留不留 | **已裁决（Owner 2026-09-05）：留着不动，不开卡**——全仓唯一一份、默认不编译不进产物、无运行时代价；DS 打包热路径过不了性能关时转正 codec，正式硬化阶段再谈 diagnostics | ds-server.md「下沉 native、边界不动」；ADR 0005 |
| D7 | RM-00003 的 27 张 V1.4 旧卡（13 张 QA 不通过后停在「评审中」+ 14 张 backlog）怎么处置 | **已裁决并执行（Owner 2026-09-06）：整体作废**——27 张全部流转「已否决」，读回 RM-00003 = 28 done / 27 rejected / 1 backlog（R-00474）。执行留下两处小瑕疵，历史不改：作废评论是通用一句「已由 ADR-062 与 Wave 1~4 取代」，不是逐卡映射；评论把决策编号写成「D8」，实为 D7。理由：卡面全按已退役的 V1.4 制度写（生成 Schema、VOX-D 门、LocalEmbedded 双树、Demand / Ticket 流式），验收项按那套制度根本跑不了；代码里有用的部分已被 9 月 5 日批次的测试覆盖 | §2.2 Workflow 对账；已裁原则 ① |
| D8 | VoxelEngine 退出旧合同制清理的深度 | **已裁决（Owner 2026-09-06）：三层全清，不留兼容**——① 复印件 / README 合同口径 / CI 校对 / 旧模块图 / 空壳 crate；② `generated/` 树、`generated-lock.json`、`check-generated-clean`、`legacy_baseline.rs`、VOX-D 门与 V1.4 fixture 骨架；③ 活代码不再用 `Generated*` 类型与 `BASELINE_ID` / `SCHEMA_EPOCH` / `STABLE_ERROR_IDS`，错误 id 只剩契约 snake_case 一套。已执行（2026-09-06）：R-00474 正文按 `plans/2026-09-05-voxelengine-w0-card-and-kickoff.md` §一 重写、10 条验收项已建；配套卡 **R-00476**（`sdk-native/voxel.rs` 去掉 8 处对 V1.4 名字的引用）已建并与 R-00474 互引，两卡串行。理由：错误 id 两套命名空间正是「第二份真值」；SDK 已经在把死名字往托管侧递，越晚清消费方越多 | 漂移 ②；已裁原则 ①③ |
| D9 | 15 张蓝图卡的 Workflow 回写口径 | **已裁决并部分执行（Owner 2026-09-06）**：13 张（R-00434 / 435 / 436 / 437 / 438 / 440 / 443 / 445 / 447 / 448 / 452 / 456 / 458）已流转「验收中」，每张有 9 月 5 日 15:54–15:57 UTC 的收口评论，引用 origin 提交 `e5c056e`（VoxelEngine）与 `4d6d2c3`（架构仓）。归 Room 16 张与 R-00433 更正评论已于 2026-09-06 补做（读回一致）。已完成只在 93 条验收项由 QA 实跑通过后流转。理由：公共纪律 ⑦「做完流转验收中，已完成由总调度核验后流转」 | 漂移 ①；td-progress-audit 步骤 3 |
| D10 | 物理三槽（raycast / sweep / overlap）只声明不路由 | **已裁决（Owner 2026-09-06）：开卡，已建 R-00477，排在 R-00443 / R-00456 验收通过之后**——路由到 `lumio-voxel-project::physics_query`，`root_api` 补 3 条测试，C# facade 补三个入口。替代方案「把三槽从根表删掉等以后再加」被否：根表只追加不插入，删了再加会换槽位 | 漂移 ⑤；ADR-062「C 签名属 native-abi.json」 |
| D11 | 契约三处已知缺陷（rule 49 错位 / 全量编码带 `baseSectionRevision` 无码 / pin 预算无常量）现在修还是等 | **已裁决（Owner 2026-09-06）：现在修，已建 R-00479（契约小卡 + ADR-062 修订记录 + VoxelEngine 副本同步）**，VoxelEngine 复制副本并更新 `CONTRACT_SHA256`。理由：R-00440 验收项 2 没有机器可判的界，V-QA 会卡在这条上 | 漂移 ⑥；drift review §6 |
| D12 | Runtime 退出旧合同制残留：三层全清还是等九项必备能力做完再清 | **已裁决（Owner 2026-09-06）：A（三层全清，开一张卡串行推进）**——同 NativeCore(D1)与 VoxelEngine(D8)，开一张清理卡：① 删 `docs/architecture` 下 6 个旧版本 md 与 `.baseline.sha256`，README 改口；② CI `repository-policy.yml` 删旧基线与克隆旧架构仓校验；③ `GeneratedContracts` 与旧 ABI 生成物下线，收敛为现行活契约与模块公开接口。 | 原则 ①；D1、D8 既定路线；避免多源维护 |
| D13 | RM-00005 的 20 张旧蓝图卡处置与 R-00461 对账 | **已裁决（Owner 2026-09-06）：A（事实闭环 + 旧卡批量作废）**——① R-00461（双 Transform 基础）代码已在 main（commit `11d34e1`），授权后流转 done 并挂真实证据；② RM-00005 中 20 张悬空旧卡整体流转已否决（`rejected`）并附取代说明，彻底出清旧蓝图单据。 | 事实先行；原则 ①③；同 D2、D7 处理路线 |
| D14 | Runtime 本地过期 worktree 与已合入分支清理 | **已裁决并执行（Owner 2026-09-06）：立即清理**——成功清理 3 个本地过期 worktree 与 6 个已合入临时分支，主工作区恢复纯净。 | 仓库卫生；避免死分支干扰调度 |
| D15 | LumioServer 旧单据出清与宿主路线收敛 | **已裁决并执行（Owner 2026-09-06；另一会话于 2026-09-05 16:28–16:29 UTC 执行，Owner 同日向本会话追认「是我收签的」）**：RM-00006 52 张旧卡 → rejected、R-00260 / R-00276 → done（无证据评论）、新建 R-00478（一行正文、0 验收项）；本地 worktree / 已合入分支同分钟清理。出清方向正确；遗留两处小瑕疵，历史不改：R-00260 / R-00276 转 done 无证据评论，R-00478 正文一行、0 验收项——后者按 D21 重写，不按原正文派工 | §2.4 Workflow 对账；原则 ①③ |
| D16 | RM-00014 被 R-00460「A0 契约冻结」前置锁死：Runtime 的 Tick 统一（RT-1 = R-00462）现在派不派 | **已裁决（Owner 2026-09-06）：B——RT-1 立即派，不等 A0 整体冻结**。RT-1 只动 Runtime 仓内的 `WorldManager` / simulation / 生成器，不碰 `engine/wire`；它需要的公共语义（13 相表、注册方式、tick 频率归属）已经写死在 `tick.md` 与 ADR-063 第 14 条，A0 没有新东西可冻。做法：把 bomber 计划 RT-1 正文以评论回写进 R-00462、A0 的「前置」措辞改为「只约束碰 wire 的卡（E4 / E5 / C5 / S4）」。替代方案 A「先派 A0 再派 E 卡」：A0 是一张 6 条验收项的契约总卡，谁都不认领，再等等于把关键路径挂在一张没人做的卡上；C「拆 A0 成逐卡契约」：多一层编排。理由：两条 Tick 路径并存是今天 Runtime 最大的「第二份真值」，比任何新功能都先 | §2.3 复核 ④⑥⑦；tick.md §4；如无必要勿增实体 |
| D17 | R-00461 直跳「已完成」怎么纠 | **Owner 2026-09-06 已收签前一会话的写入，done 状态不改。** 剩余动作（待授权）：补一条证据评论（`11d34e1`、`Ecs.Tests` 27 / 27、`TransformComponentTests` 9 项）+ 把 5 条验收项按本机实跑补记 passed / not_started，让 RM-00014 第一张 done 卡有验收记录；接线小项（样板接 `LogicTransform`、`[Writes]` 负例、类型化编码）并入 R-00463 | §2.3 复核 ④；Owner 收签 |
| D18 | 12 张 T-00010 ~ T-00021 工作项（RM-00005 唯一还在 active 的东西） | **已裁决（Owner 2026-09-06）：A——整体作废**，reason 指向父卡 done + R5-02（R-00407）+ R-00461。替代方案「留着当 ECS 清单」：它们的父卡已 done、正文是 8-30 口径、T-00021 与 R-00461 重复，留着只会让 Room 总账永远显示 12 active | §2.3 复核 ⑧ |
| D19 | GAS 两张 Runtime 卡（RT-4 技能八态 + 准入 / RT-5 Effect 单 + 两本账 + 求值）落 Workflow 是一张还是两张 | **已裁决（Owner 2026-09-06）：B——拆两张**：R-00468 = RT-4；RT-5 新建一张卡（RM-00014，前置 R-00468），RT-3 = R-00466 在 R-00468 合入后即可与 RT-5 并行。替代方案 A「维持计划口径合成一张」让 RT-3 多等一个 wave；C「Attribute 再单拆成三张」收益不明显 | 并行优先；RT-3 只依赖 `Activate<T>`；`plans/2026-09-05-bomber-engine-runtime-cards.md` 落单口径随之更正 |
| D20 | Runtime 仓旧分支（远端 19 条已合入 `feat/*` + 3 条 PR #5 / #6 / #7 squash 前的旧 tip；本地 4 条非 main）删不删 | **已裁决（Owner 2026-09-06）：A——全删**（远端 22 条 + 本地 4 条），删前逐条核已在 main（squash 后正文在 main，多出的只是 PROBE 提交与其 revert）。替代方案 B「只删 19 条」留一份永远不会合的历史；C「不动」留 22 条干扰 | 与 NativeCore / VoxelEngine 同一口径；worker 进场不该看到假活分支 |
| D21 | R-00478 的范围与正文 | **已裁决（Owner 2026-09-06）：只做清理，去掉 ③「修 macOS 编译」**——跨平台归 R-00408（已拥有 `modules/process/**` + `host-runtime/**` + CI，PR #33 有半成品）；正文按 card-spec 四节重写并补约 10 条验收项（清单见 §2.4「剩下的单子」）。否则 R-00478 / R-00408 / R-00391 / PR #33 四处写同一件事 | 漂移 ① ②；D1 / D8 / D12 同路 |
| D22 | C# mvp-host 与 account-server 去留 | **已裁决（Owner 2026-09-06）：两者都删，分先后——mvp-host 随 R-00478 删（验收 11，含 CI job 与 `room-admission.md`）；account-server 随 R-00408 删（套件改自签测试凭据后）**——ADR 0008「整目录删除等 51 张 Rust 主线」的前提已随 52 张作废失效；mvp-host 40k 行是第三套宿主、verify-all 与 CI 常红；account-server 归 Platform（ADR-061），今天只有 11 场景套件经 `discover.rs:24` 拉起它，三个必过测试不依赖；删除排在 R-00408 把套件改为自签测试凭据（`suite.rs` 已 `generate_keys()`）之后。「继续冻结」被否：冻结就是兼容层 | 原则 ①③④；漂移 ⑤ ⑥ ⑦ |
| D23 | R-00388 / PR #33 / R-00391 处置 | **已裁决（Owner 2026-09-06）：R-00388 流转已取代（rejected）并评论指 R-00408；PR #33 不直接合，由 R-00408 实现方 cherry-pick 自驱 / 背压 / 日志 / cfg-gate 提交后关闭；R-00391 作废并入 R-00408**（已执行，2026-09-05 23:59Z，见 §7） | r5 cards 第 41 行；r4 复核 §7；漂移 ③ |
| D24 | Rust 宿主跨平台加载器 | **已裁决（Owner 2026-09-06）：A——删两份 Win32 手写加载器（`sdk_loader.rs` 840 行、`native_timer.rs` 418 行），Rust 宿主只留一份跨平台 `dlopen`（`libloading`）加载；或架构仓在 SDK 里提供 Rust loader crate 由 Server 消费**；Linux `cargo test --locked` 绿写进 R-00408 验收项；CI Rust 作业加 ubuntu。理由：生产 DS 跑 Linux，`dev-build.ps1` 已产 `.so`，宿主却链接不过 | 原则 ①②；`architecture.md` §5 共享 Loader；漂移 ② |
| D25 | LumioClient 上游 API 破坏性漂移修复与治理卡 | **已执行（另一会话 2026-09-06 00:15 UTC；Owner 已收签）：新建 R-00481**（RM-00007，P1，三层：① `ReplicaWorld.cs` 两处 `GrantClaim` 对齐 Runtime HEAD；② 删 `docs/architecture` / `contract-mirror` / CI 基线；③ README 改口）。本会话复核：方向对；**瑕疵**——0 验收项、未指派；第一层与在途 R-00408 Client 部分（拥有 `modules/replica/**`，origin 分支 `fix/r5-client-runtime-frames` 已删 GrantClaim）文件集重叠 → 范围收窄见 D30 | §2.5 漂移 ①；原则 ① |
| D26 | RM-00007 9 张历史旧卡作废出清 | **已执行（另一会话 2026-09-06 00:14 UTC；Owner 已收签）**：R-00055、R-00253 / 254 / 255、R-00289、R-00292、R-00294、R-00296、R-00298 → rejected。本会话复核：与 ADR 0013（Unity 降级）/ D3（体素不进预测世界）一致；**瑕疵**——9 次流转无 reason、无作废评论（D7 / D13 / D15 都有取代说明）；R-00292 / R-00294 对应的代码缺陷仍在，并入 R-00481 验收项（见 D30） | §2.5 Workflow 对账 |
| D27 | 客户端前端路线收口（CL-1 调研落地） | **已裁决（Owner 2026-09-06）：采纳 CL-1 结论**——桌面浏览器预测走 .NET 10 browser-wasm 装载 Runtime 客户端程序集，不写第二份 JS 预测；手机不作真机承诺。本会话补两点：与 D3（暂定方案 A）同义，**ADR 文件尚未落**（CL-1 §5.3 草案待写进 `decisions/`）；落地卡 = R-00470 AC06 + 三个先决预算，不另开「WASM 壳」卡；Unity 不是现行路线（另一草稿「Unity 端加载同一份动态库」一句更正） | CL-1 报告 §5；D3；ADR 0013 |
| D28 | 客户端必备能力承接 | **已裁决（Owner 2026-09-06）：R-00467（C5）与 R-00470（C8）归 LumioClient 主干**。本会话补：C5 仍在 D16 的 A0 约束名单（碰 wire 的 `sequence` / `appliedInputSequence`），C8 不受约束、前置 R-00461（done）+ R-00466；两卡 0 评论、验收项 6 + 6 全 not_started；执行顺序只能是 R-00408 Client 部分 → C5 → C8 | RM-00014；§2.5 架构演进 |
| D29 | LumioClient 分支与 worktree 清理 | **已执行（另一会话 2026-09-06 08:14 北京时间；Owner 已收签）**：3 个游离 worktree、8 条已合入本地分支、3 条已合入远端分支已删。本会话 08:23 读回一致（本地只剩 `main`，远端 `main` + 新推的 `fix/r5-client-runtime-frames`）；08:06 快照里三条远端分支相对 main 均 0 ahead，删除无损 | 仓库卫生；`git ls-remote` 两次快照 |
| D30 | **R-00481 的范围**：第一层「GrantClaim / Replica 对齐」留在 R-00481 还是归 R-00408 | **已裁决（Owner 2026-09-06）：A 扩大版——「能清干净就尽量清干净，可以接受大的重构」**。R-00481 去掉第一层（GrantClaim / codec 对齐归 R-00408 Client 部分，其分支已删 GrantClaim），改为**彻底清理卡**：① `docs/architecture/` 6 md + `.baseline.sha256`、`docs/LumioClient_five_requested_files/` + `eng/verify-five-file-package.*`、`docs/specs/`；② `contract-mirror/` 整目录 + `eng/sync-/verify-contract-mirror.*` + `eng/upstream-contract-smoke` + `eng/upstream-api-map.md` + `tests/Fixtures/index.json`，4 个模块的 `Generated*Adapter` / `Generated*FixtureTests`（消费旧 fixture 语料）整体删除，需要契约测试的改读架构仓 `engine/wire`；③ CI `repository-policy.yml` 删 v1.4 正文 / sha256 / `LGE-V1.2` / 模块数 13 / 五文件 / 镜像六项断言，`dotnet-test.yml` 的 Runtime pin 改跟 `main`（D35）；④ README（删「架构基线 / Architecture Gate / Generated Contract Dependencies」合同制整节）、`.spec/AGENTS.md`（V1.2 → Living Architecture，设计落 `.spec/knowledge/features/`）、`repository-architecture.md`、13 个模块 README（删 v1.2 章节引用，「阶段」按实况重写）；⑤ 并入 R-00292（架构测试改 `git ls-files` 口径）与 R-00294（public 类型出 `Internal/`）；⑥ 按 card-spec 四节重写正文 + 约 12 条验收项 + 指派；⑦ 删 Unity / HybridCLR 空壳与 UPM 骨架（D31）。不碰 `modules/replica/**` 生产代码、不碰 prediction / session 骨架（归 C5 重写）。理由：同一件事只在一处写（D21 同路）；Owner 原则 ①「底层只保留唯一、最干净的版本，不留兼容」 | §2.5 漂移 ① ② ⑦ ⑧；D1 / D8 / D12 / D21 |
| D31 | **Unity / HybridCLR 空壳去留**（`unity-adapter` / `hybridclr-adapter` 0 行源码 + UPM 骨架 9 asmdef + T-00009） | **已裁决（Owner 2026-09-06）：A——全删**。两个适配器的 src / tests 工程、`packages/com.lumio.client/` UPM 骨架、`LumioClient.slnx` 与 `eng/project-reference-allowlist.json` 对应条目、README 子模块表两行、CI 模块名单里的两项一起删；T-00009 作废。并入 R-00481 验收项（第 ⑦ 条）。将来真接 Unity 按当时 Runtime 形态重开卡，不在空壳上长。理由：ADR 0013 已降级；原则 ① ③；空壳是不会兑现的承诺 | §2.5 模块状态 |
| D34 | R-00408 / R-00392 / R-00393「验收中 / 评审中」但 origin 无完整交付怎么处置 | **已裁决并执行（Owner 2026-09-06 授权「剩下的事情你自己决定」）：A——状态不动，补差异记录评论**（三条评论已写入并读回，见 §7）：三张卡各一条，写明评论所引 Runtime `18b5feb` 未推送、Client `5745d11` / Server `f031afb` / Game `d3f6fe0` 不在 origin，origin 对应分支真实 tip 为 `fix/r5-client-runtime-frames`@`5652b95` / `fix/r5-server-wire-forwarding`@`deebeb4` / `fix/r5-entity-chat-scenarios`@`e243a39`，架构仓 `9a22527` 在 `r5-closeout-20260906`；解铃 = Runtime 推送 + 四仓开 PR、CI 绿、合入后重核，QA 此前不得判 done。R-00408 另加 Client 侧验收口径：`GameplayCodec.cs` / `LiteJsonParser.cs` 删除，生产代码只经 Runtime `WireCodec`。替代 B「退回实现中」与已收签的 passed 验收项打脸；C「不动不评论」让下一站再踩「验收中当已完成」 | §2.5 漂移 ③ ④；td-progress-audit 步骤 3；R-00475 同例 |
| D32 | LumioGame RM-00008 旧单据出清与 W0 清理卡（另一会话编号「D30」） | **另一会话记为「已裁决并执行（Owner 2026-09-06）」，于 2026-09-06 00:28:55 – 00:29:04Z 执行，本会话读回一致**：R-00310 ~ R-00313 → rejected（与 D2 / D13 同族同路，方向正确）；新建 R-00482（P0 / backlog）；LumioGame 本地 3 个 worktree 与 6 条已合入分支已删。**Owner 2026-09-06 已追认。** 建卡瑕疵（0 条原生验收项、范围窄）已随 D33 补齐；远端分支与 PR #12 归 R-00482 验收 8 | §2.6 Workflow 对账；原则 ①③ |
| D33 | 炸弹人 G-1 ~ G-7 的开工前置，与 R-00482 的补齐 | **已裁决并执行（Owner 2026-09-06：A；写入见 §7 与 `plans/2026-09-06-game-w0-card-and-kickoff.md` §六）：先重冻 G-0 v2 = 新建 R-00483，再按引擎接缝分两批派；R-00482 已补 8 条验收项并扩到三层。** 理由：Game 09-04 契约是照 Runtime 当时三个缺口（无系统注册面 / 无体素端口 / 无 GAS，ADR 0015 §0）反向设计的自备引擎——自己的 Tick 编排、地形存储（`ITerrainStore` / `InMemoryChunkStore`）、位置字段（`PosMilli` / `Cell`）、属性单账——与 09-05 架构第二样板五处结构性不同，也撞世界模型红线「静态必须是体素、预测一律经 GAS」；按现卡派 = 在游戏仓造第二套引擎，RT-1 / E7 落地即整套推倒。做法：① 新建 G-0 v2 一张（位置 = `LogicTransform`、属性 = `AttributeComponent` 两本账、移动 / 放弹 = Ability、爆炸 / 死亡 = `[System(Phase.ProcessorPlan)]`、地形 = 引擎体素批量读写；炸弹持火焰 / 帽堆 / 重生三处不动），前置只有已 done 的 R-00461；② G-1 ~ G-7 评论回写分批：**批 A** G-4 地图数据 + 断言、G-5 Config、G-7 遥测、G-2 帽子经济——v2 冻结即派；**批 B** G-1 规则内核、G-6 Scenario 宿主、地形读写——等 R-00462 / R-00468 / R-00469 合入；③ R-00482 补 8 条原生验收项、范围按 §2.6「剩下的单子」①–⑤（含 CI 新增 build / test 作业、六处文档改口、远端分支与 PR #12），前置改为 R-00408 的 Game 提交推送合入；④ R-00423 转验收中 + 证据评论。**替代 B**：按现卡直接派走 ADR 0015 自行编排 + `InMemoryChunkStore`——headless 原型最快（ADR 0013 逻辑先行），代价是游戏仓里一套要拆的引擎替身、两份地形与位置真值，且移动 / 放弹不是 GAS Ability（违红线）。**替代 C**：全部等九项必备做完——headless 先行价值归零，批 A 四张本可现在动 | 漂移 ②④⑤；`rules/system.md` 世界模型；`bomber-slice.md` §4；`movement.md` §9.1；`lessons.md` 快照复跑条；原则 ①②③ |
| D35 | Client CI 怎么对 Runtime；开发期门禁要多少 | **已裁决（Owner 2026-09-06）：A——CI 检出 Runtime `main` 最新，不钉号；并立一条开发期原则：「保持最敏捷，前期不搞这么多检测工具，加速开发效率」**。落到 R-00481 第 ③ 条：Client CI 只留两件事——`spec-lint` 与 `dotnet test`（对 Runtime `main`）；删 `verify-sdk-pin.*`、`verify-five-file-package.*`、`verify-contract-mirror.*`、`repository-policy.yml` 里 README / 模块 README / 基线 / 模块数的全部 grep 断言；`global.json` 钉 SDK 保留（构建输入，不是检测工具）；ArchitectureTests 保留（在 `dotnet test` 里跑、不另起工具），但按 R-00292 改 `git ls-files` 口径。替代 B「钉号 + 人工 bump」是同一件事两处维护，今天落后 16 个提交就是结果；C「钉号 + 每日报警作业」多一个作业多一份口径。理由：契约由架构仓 `engine/wire` 守，不靠冻结旧 Runtime；Runtime 破坏性改动立刻打红 Client 正是要看到的信号 | §2.5 漂移 ②；Owner 原则「快 > 稳 > 好」 |

## 7. 本次已执行动作 / 待授权事项

- 已执行（2026-09-05，Owner 授权后逐笔读回）：
  - Workflow：新建 R-00473（RM-00002，8 条原生验收项，全部 not_started）；R-00302 / R-00308 / R-00309 流转「已否决」并各补 1 条作废评论；R-00007 5 条与 R-00083 4 条验收项改为 passed（读回 5/5、4/4）。共 1 建单 + 8 验收项 + 3 流转 + 3 评论 + 9 验收项更新。
  - NativeCore 仓：远端 `feat/r-00352-timer-manager` / `feat/r-00372-timer-abi` / `feat/r-00386-r4-07-timer-ffi-delete` 已删除；本地只剩 `main`，worktree 只剩主工作区。
  - 架构仓文档：本报告、`plans/2026-09-05-nativecore-w0-card-and-kickoff.md`（含 R-00473 开工提示词）、gas.md M9 改归 Runtime、ADR-064 修订记录；首批已由 Owner 提交（`4a5f596`），本轮补写卡号与执行记录待再提交。
- 未执行：R-00473 的实现（另开窗口按 plans 文件 §二 提示词派工）；其余七仓盘点（§2.2 ~ §2.8）。

- VoxelEngine 站（2026-09-05，本会话）：
  - 仓库侧已 `git pull --ff-only` 到 `e5c056e`；全仓门禁实跑全绿（`cargo test` 的上游逐字节比对用例在架构仓本地拉到 origin 之前失败 1 条，拉齐后 12 / 12 通过，见 §2.2）。
  - Workflow 执行（2026-09-06，Owner 授权后逐笔读回）：
    - 新建清理卡 **R-00474**（RM-00003，《[程序·工程] 退出旧合同制残留：删除 docs/architecture 镜像、CI 基线校验并重写 README 与注释》），状态 backlog。
    - 批量作废旧蓝图卡 27 张（R-00002、R-00066、R-00068、R-00070、R-00076、R-00078、R-00080、R-00096、R-00104、R-00116、R-00134、R-00136、R-00142 及 14 张 backlog）全部成功流转为 `rejected`（已否决），并附作废理由：“已由 ADR-062 与 Wave 1~4（R-00434~R-00458）新架构实现取代，按 2026-09-05 引擎总监盘点决策 D8（出路 A）作废关闭。”
    - RM-00003 读回：总数 56 张（28 done、27 rejected、1 backlog R-00474），旧需求彻底出清。
  - 产出：本报告 §2.2 / §3 / §6 D7–D11；`plans/2026-09-05-voxelengine-w0-card-and-kickoff.md`（已填入真实卡号 R-00474）。
  - 架构仓需 Owner 在终端执行：先 `git pull --rebase origin main`，再把本报告、`plans/2026-09-05-voxelengine-w0-card-and-kickoff.md` 加入暂存区提交并推送。
  - 2026-09-06 Owner 裁决 D8 = 三层全清、不留兼容（见 §6）。R-00439 的深审与复审（`reviews/2026-09-05-r-00439-deep-review.md` / `-rereview-v2.md`）共 6 条问题在 origin `4d6d2c3` 全部关闭（写入槽带 `transaction_id`、物理三槽已声明、生成器拒绝根表乱序 / 算式漂移 / 未知类型、C 头 presence 固定宽度、生成器测试 19 条），本站据此吸取三条做法写进 V-QA 提示词：反例探针、快照直接放 `~/LumioGames` 下、macOS 上 dev-run 只能标 blocked。
  - **已执行的 Workflow 写操作（2026-09-06，Owner 一次性授权「所有的事情你自己决定」后逐笔读回）**：① 归 Room 16 张（R-00433 与 11 张 Voxel 卡 → RM-00003，R-00439 / 443 / 445 / 456 → RM-00001；PATCH 200 × 16，读回 roomId 一致）；② R-00474 标题与正文按 plans §一 重写（PATCH 200）+ 10 条原生验收项（质量验收 / 未开始，读回 10 / 10）；③ 新建 R-00476（A-W0，5 条验收项）、R-00477（A-5，7 条）、R-00479（C-FIX，7 条），均 RM-00001 / backlog / P1（R-00478 是同时段别的会话建的，不属本站）；④ 引用 6 条（476↔474、477↔443 / 456、479↔440 / 436 / 438，均 201）；⑤ R-00433 更正评论 1 条（201，读回 2 条）。VoxelEngine 仓：16 条已合入本地分支删除、3 个 stale worktree 移除。**未做**：V-QA 跑批与四张新卡的实施（各另开窗口，卡面即提示词）。

- GameRuntime 站（2026-09-06，本会话）：
  - 仓库侧已 `git pull --ff-only` 到 `89a7a6c`；全仓 12 个测试工程 662 个测试实跑全绿（原文 626 为加总笔误）。
  - 仓库卫生执行（Owner 授权后，D14）：清理释放 3 个过期的本地 `.claude/worktrees`（`distracted-moser-154e2f`、`exciting-chaplygin-77e103`、`priceless-meitner-bb6110`）；删除 6 个已完全合入主干的本地临时分支，当前工作区恢复纯净。
  - Workflow 执行（Owner 授权后逐笔读回）：
    - 新建清理卡 **R-00475**（RM-00005，《[程序·工程] 退出旧合同制残留：删除 docs/architecture 镜像、CI 基线校验并收敛 GeneratedContracts》），状态 backlog。
    - 核销流转 **R-00461**（RM-00014，双 Transform 基础）：从 backlog 经链式转移流转到 **`done`**，附 commit `11d34e1` 与 `TransformComponentTests` 测试通过记录。
    - 核销流转 **R-00407**（RM-00011，ECS 单一世界与模板内联）：从 acceptance 流转到 **`done`**。
    - 批量作废旧蓝图卡 20 张（R-00049、R-00141、R-00162、R-00167、R-00174、R-00176、R-00181、R-00184、R-00187、R-00191、R-00192、R-00195、R-00197、R-00199、R-00295、R-00303~R-00307）：全部成功流转为 `rejected`（已否决），附作废说明：“已由 ADR-058/064/065 及 RM-00014 新架构实现取代，按 2026-09-06 引擎总监盘点决策 D13 作废关闭。”
    - RM-00005 读回统计：总数 41 张（20 done、20 rejected、1 backlog R-00475），历史悬空需求全部出清完毕。

- GameRuntime 站复核（2026-09-06 深夜，第二会话，**只读**）：
  - 仓库侧实跑：`git fetch` 后 0 / 0；`dotnet test -c Release` 12 个测试工程 662 / 0 failed；`dotnet format --verify-no-changes` 抽 5 个生产项目 2 处不过（coordination / replication）；spec-lint 13 / 13；Workflow 全量拉两轮（16:05Z 与 16:30Z）对照出前一会话 16:16–16:28Z 的写入。
  - 复核阶段未做任何 Workflow 写入；产出：本报告 §2.3「本会话独立复核」、§3 两行更正、§6 D16–D20。
  - 发现的两处欠账：R-00461 done 但 0 评论、5 条验收项 not_started；R-00475 到「实现中」但 0 验收项、仓内未开工。Owner 已收签前一会话的写入，状态均不改，只补证据评论与验收项。
  - 架构仓侧顺带：`.workflow-drafts/engine-nine-musts-20260905/` 仍在仓根（git 已 exclude，但 spec-lint 报「并行文档根」，guard-commit 钩子因此拦 `git commit`）；`knowledge/README.md` 末尾多出一行重复的 `runtime-manager-query-expiry` 导航行。均未动。
  - **执行阶段（Owner 授权 D16–D20 后逐笔读回）**：
    - **R-00475**：卡面按 D12 重写为三层（PATCH 200，正文 2,406 字）+ 新建 8 条原生验收项（读回 8 / 8「未开始」）+ 一条评论。评论同时记录**交付证据不可复核**：2026-09-06 02:0x 的三条交付评论声称提交 `dc37644813b9a72db5f78f250a8602f88e8218eb`（分支 `Go1c/r5-a2-runtime-query-expiry`），本次实测 `git ls-remote origin` 无该分支、`git cat-file -t` 报 `could not get object info`、`origin/main` 仍是 `89a7a6c`；解铃条件 = 实现方 push 并开 PR 后补 origin 提交号，QA 在此之前不得判 passed。
    - **T-00010 ~ T-00021**：12 张工作项全部流转「已取消」（200 × 12），逐卡写取代说明（父卡 done + R-00407 + R-00461 / ds-server AOI 切片），读回 RM-00005 工作项 active = 0。
    - **R-00461**：状态不动，补证据评论（`11d34e1` 在 `origin/main`、`Ecs.Tests` 27 / 27、定向单测 1 / 1、全仓 662 / 0）并逐条判定 5 条验收项——**AC02 passed，AC01 / AC03 / AC04 / AC05 blocked**（缺独立消费样例、缺 codec 往返与确认态 Hash 测试、预测世界与 owner-thread 断言未实现、销毁 / 代次与 C8 消费未实现），阻塞项全部依赖 R-00463 / R-00466 / R-00470，不是本单实现缺陷。
    - **R-00460**：补评论，A0 前置按 D16 收窄为只约束碰 `engine/wire` 的四张卡（R-00464 / 465 / 466 / 467）。
    - **R-00462**：回写 RT-1 完整正文，流转「评审中」（读回 `in_review`）；派活排在 R-00475 合入之后（两卡都动 `TickPhase`）。
    - **R-00468**：回写 RT-4 正文并按 D19 收窄为 GAS M2；**新建 R-00480**（RT-5「GAS 瞬时效果纵链」，RM-00014 / P1 / LumioGameRuntime，4 条验收项读回 4 / 4，与 R-00468、R-00462 互引 201 × 2）。
    - **R-00466**：回写 RT-3 正文，前置更正为 R-00462 + R-00468，与 R-00480 并行。
    - **仓库卫生（D20）**：逐条核验后删远端 19 条、本地 4 条分支；3 条「未合入」的是 PR #4 / #5 / #7 squash 前的 tip，丢失的只有 PROBE 与其 revert，正文已在 `main`（`91ba9a8` / `bd3a3ce` / `072787b`）。读回：远端只剩 `refs/heads/main`，本地只剩 `main`，worktree 只剩主工作区。
    - **跨仓顺带修复**：八个仓 `.spec/lumio-agent.lock` 的 `source` 由已退役的 `Go1c/LumioAgent.git` 改为 `LumioGames/LumioAgentSpec.git`（同一仓迁移改名，锁定提交 `28610f8` 在新仓可达）；`@Go1c` 是 Owner 本人的 GitHub 用户名，CODEOWNERS 与历史 ADR 引用不动。各仓改动未提交。

- LumioServer 站（2026-09-06，本会话）：
  - 仓库侧已 `git pull --ff-only` 到 `4c7688b`；C# mvp-host 核心子工程单元测试通过，`App.Tests` 因本地未编译跨仓 Game 产物报 2 处找不到依赖；Rust 宿主在 macOS 存在 `native_timer.rs` dead-code 编译阻断待修复。
  - 仓库卫生执行（Owner 授权后，D15）：清理释放 2 个本地过期 worktree（`epic-kalam-a04752`、`friendly-moser-be15c3`）；删除 8 个已完全合入主干的本地临时分支，主工作区恢复纯净。
  - Workflow 执行（Owner 授权后逐笔读回）：
    - 新建清理卡 **R-00478**（RM-00006，《[程序·工程] 退出旧合同制残留、修复 Rust 宿主 macOS 编译并归档 C# mvp-host》），状态 backlog。
    - 核销流转 2 张 C# mvp-host acceptance 卡为 **`done`**：
      - `R-00260`（`01a04735`，MVP C# 宿主设计）流转为 `done`。
      - `R-00276`（`01a04c08-0c04`，[MVP C# 宿主][wave 4] 实现 auth 存根）流转为 `done`。
    - 批量作废旧蓝图与悬空卡 52 张（R-00252、R-00280、R-00281 等 47 张旧蓝图卡与 5 张未启动 mvp-host 卡）：全部成功流转为 `rejected`（已否决），附作废说明：“已由 ADR-056、Rust 宿主及 RM-00011/RM-00014 新架构实现取代，按 2026-09-06 引擎总监盘点决策 D15 作废出清。”
    - RM-00006 读回统计：总数 68 张（15 done、52 rejected、1 backlog R-00478），历史悬空需求全部出清完毕。

- LumioServer 站（2026-09-06，本会话，**只读**）：
  - 仓库侧 `git pull --ff-only` → Already up to date（`4c7688b`）。实跑：spec-lint OK 13/13；`cargo test --workspace --locked` exit 101（第一道 dead-code，放行后第二道 `ld: library 'kernel32' not found`）；`mvp-host/eng/verify-all.sh` → `MVP_HOST_VERIFY_FAIL test …App.Tests`（32 过 2 败，其余工程未跑）；`account-server` 32/32 通过。
  - 本会话**未做任何 Workflow 写入、未删分支、未提交**；只改本报告 §1 / §2.4 / §3 / §4 / §6 / §7。
  - **另一会话已执行（2026-09-05 16:28:39–16:29:25 UTC，操作人 `usr_e6bba…`）**：RM-00006 52 张旧卡 → rejected；R-00260 / R-00276 → done（无证据评论）；新建 R-00478（一行正文、0 验收项、P0）；本地 `.claude/worktrees` 与已合入本地分支已清（目录 mtime 00:27）。以上 Owner 2026-09-06 向本会话追认（D15）。
  - **已执行（Owner 授权，2026-09-05 23:50:26Z，逐笔读回）**：R-00478 `PATCH`（标题 + 背景 / 目标 / 验收 / 边界四节正文，reason 引 D21 / D22；第一次响应读取被截断，先 GET 确认已落库、未重发）；`POST` 11 条验收项（质量验收 / 未开始，id `01a073fb-def2` … `01a073fc-0bb8`，全部 201），读回 11 / 11 `not_started`、类型一致；状态仍 backlog，未流转、未建卡。载荷与响应留档在会话 scratchpad `r00478-writeback/`。
  - **已执行（Owner 授权，2026-09-05 23:59:22–23:59:32Z，逐笔读回）**：R-00388 `POST transition → rejected`（reason 引 D23）+ 评论 `01a07403-3b14`（读回 status=rejected、comments=2）；R-00391 `transition → rejected` + 评论 `01a07403-423f`（读回 rejected、comments=1）；R-00408 评论 `01a07403-4634`（D22 / D23 / D24 五条口径，读回 comments=4，状态仍 in_progress）；`gh pr close 33 --repo LumioGames/LumioServer`（读回 CLOSED 23:59:32Z，分支保留）。载荷与响应留档 scratchpad `d23-d24-writeback/`。
  - **已执行（Owner 授权，2026-09-06）**：`git push origin --delete` 删远端 14 条分支——13 条已合入 `feat/*`（逐条实测 `rev-list origin/main..` = 0、`diff --stat` 为空），外加 `feat/r-00346-admission`（唯一带内容的一条：82 行，mvp-host 两个测试工程 `packages.lock.json` 的另一次刷新，该目录按 D22 整体删除，无保留价值）。读回 `git ls-remote --heads`：只剩 `main` 与 `feat/r-00388-r4-02-self-drive`（留给 R-00408 cherry-pick，完成后删）；本地 `fetch --prune` 后只剩 `main`。
  - **本站无待授权事项**。R-00478 卡面与 11 条验收项已在线（可直接派工，提示词见 `plans/2026-09-06-server-w0-card-and-kickoff.md` §二）；R-00408 的补充口径已评论在卡上。架构仓本报告与两份 plans 文件仍未提交，由 Owner 在终端提交。

- LumioClient 站（2026-09-06 北京时间 08:00–08:40，本会话，**只读**）：
  - 仓库侧 `git pull --ff-only` → Already up to date（`c3825a4`）。实跑：spec-lint OK 13 / 13；`verify-sdk-pin` / `verify-five-file-package` / `verify-contract-mirror`（326 文件）全 OK；`repository-policy.yml` 的 README / sha256 断言本地复现全过；`dotnet restore` up-to-date（未复现另一草稿的 NU1004）；`dotnet test LumioClient.slnx` 对同级 Runtime `89a7a6c` 编译——根因源码核实为 `ReplicaWorld.cs:109 / :451` 两处 `GrantClaim`（Runtime `c2d42b3` 已删），实跑结尾见 §2.5 表末行。
  - 本会话**未做任何 Workflow 写入、未删分支、未提交**；只改本报告 §1 / §2.5 / §3 / §6 / §7。**注意**：第六站会话正在同时编辑本文件（其 D32 / D33 与本会话原编号撞号，本会话已改为 D34 / D35 避让）；两会话都未提交，提交前以文件内实际内容为准、逐节核对。
  - **另一会话已执行（Owner 已收签，D25–D29）**：RM-00007 9 张旧卡 → rejected（2026-09-06 00:14:42–00:14:49 UTC，无 reason / 评论）；新建 R-00481（00:15:22 UTC，0 验收项、未指派）；LumioClient 3 个 worktree + 8 条本地分支 + 3 条远端已合入分支清理（08:14 北京时间）。其草稿写在架构仓 worktree `LumioGameEngine-r479` 的报告副本里（未合入 main 工作区），已在 §2.5「与另一会话草稿的对照」逐条对账并更正 3 处数字、2 处口径。
  - **又一会话已执行（2026-09-06 00:10–00:12 UTC，RM-00011）**：R-00408 → acceptance（4 条验收项 passed）、R-00392 / R-00393 → in_review；架构仓推分支 `r5-closeout-20260906`（含 `9a22527`、两份 reviews 新文件，**但其报告副本是 278 行旧版，不得直接合**）；Client / Server / Game 各推一条 `fix/r5-*` 分支、Runtime 零推送、四仓零 PR。本会话对其证据的核验结论在 §2.5 漂移 ③，差异记录评论待授权。
  - **已执行（Owner 2026-09-06 授权「同意，剩下的事情你自己决定」后逐笔读回，24 次写入全部 2xx，流水在本机 `wf-writes.ndjson`）**：
    - **R-00481**：`PATCH` 标题 + 四节正文（按 D30 / D31 / D35 重写，5,362 字，读回一致；`ownerId` 已是 Lumio 账号）+ `POST` 12 条原生验收项（质量验收 / 未开始，读回 12 / 12）；卡面与开工提示词见 [`plans/2026-09-06-client-w0-card-and-kickoff.md`](../plans/2026-09-06-client-w0-card-and-kickoff.md)。
    - **R-00408 / R-00392 / R-00393**：各 1 条差异记录评论（201；读回评论数 7 / 2 / 2），状态不改；R-00408 另附 Client 侧验收口径（删 `GameplayCodec.cs` / `LiteJsonParser.cs`）。
    - **T-00004 / T-00005 / T-00009**：流转「已取消」（200 × 3，读回 canceled），reason 分别指 D22 + R-00408 / R-00389 / D31。
    - **R-00415**：approved → in_progress → acceptance（200 × 2，读回 acceptance）+ 证据评论（`06db121` 在 origin/main）。
    - **R-00466**：评论 + 1 条验收项 `[RT3-AC-WASM]`（201；验收项 6 → 7）——CL-1 上游缺口 (a) 的落点。
    - 架构仓文档：新建 [`ADR-067`](../decisions/ADR-067-browser-client-prediction-dotnet-wasm.md)（Draft，D3 / D27 落文档）+ `decisions/README.md` 索引行；新建 `plans/2026-09-06-client-w0-card-and-kickoff.md`；本报告 §1 / §2.5 / §3 / §6 D25–D31、D34–D35 / §7。
  - 剩余待做（不在本站范围）：R-00481 实施（另开窗口按 plans §二 提示词派工）；R-00408 等 Runtime 推送 + 四仓 PR 后重核；架构仓这批文档由 Owner 提交（两会话共用本文件，提交前逐节核对）；T-00004 / T-00005 作废；R-00415 流转；R-00466 追加 IL2075 验收项；架构仓 ADR「浏览器客户端预测走 .NET WASM」；D31（Unity 空壳）下一问。

- LumioGame 站（2026-09-06 北京时间 08:15–08:45，两会话）：
  - 另一会话（草稿写入 `LumioGameEngine-r479` worktree 副本 08:30，编号「D30」）：Workflow 执行 R-00310 ~ 313 → rejected（00:28:55 – 00:28:58Z）、新建 R-00482（00:29:04Z，0 原生验收项）；LumioGame 本地 3 个 worktree + 6 条已合入分支删除。其草稿的数字与「已落地」陈述按 §2.6 对照表更正后并入本文件；r479 副本上的 §2.6 草稿与其 D30 / §7 条目由本节取代。
  - 本会话（**只读**）：`git fetch --all --prune` 0 / 0（`5bc5afc`）；实跑 spec-lint OK、13 / 13、node 17 / 17；`dotnet test LumioGame.sln` 编译失败（CS1061 `TryGetSession`，根因 Runtime `c2d42b3`）；`origin/fix/r5-entity-chat-scenarios` 用 `git archive` 快照对 Runtime `89a7a6c` 实跑 25 / 25、node 21 / 21、spec-lint OK；Workflow 全量拉两轮（08:2x / 08:3x）对照出另一会话的写入；R-00408 四仓提交逐一 `cat-file` 核验全部不在 origin。**未做任何 Workflow 写入、未删分支、未提交。**
  - **已执行（Owner 2026-09-06 裁决 D33 = A、追认 D32，一次性授权「写入不要每笔问我」后逐笔读回；明细见 [`plans/2026-09-06-game-w0-card-and-kickoff.md`](../plans/2026-09-06-game-w0-card-and-kickoff.md) §六）**：R-00482 `PATCH` 标题 + 四节正文 + `POST` 8 条原生验收项（读回 8 / 8）+ 追认评论；**新建 R-00483**（G-0 v2，RM-00013 / P0 / high，7 条验收项读回 7 / 7，引用 R-00423 / 461 / 462 / 468 / 469 / 480 / 482 均 201）；R-00424 ~ R-00431 各 1 条分批评论（201 × 8）；R-00423 评论 + 流转 backlog → in_review → in_progress → acceptance（读回 acceptance）；R-00408 差异记录评论（状态不动）。Room 读回：RM-00013 11 张 {'acceptance': 1, 'backlog': 10}；RM-00008 7 张 {'acceptance': 1, 'backlog': 1, 'done': 1, 'rejected': 4}。
  - **未做**：R-00482 / R-00483 的实施（提示词在 plans §五）；远端分支删除与 PR #12 关闭（归 R-00482 验收 8）；本报告与 plans 文件的提交（待 Owner）。
