---
name: 2026-09-06-agent-contracts-ci-review
description: 外部评审《Agent 契约 / 确定性 / 跨仓 CI》落地版——与远端 main 23401e1 对账、PR #95 核查与 Owner 采纳裁决；改验证链或 CI 前查
metadata:
  type: doc
  status: 已交付
---

# LumioGameEngine：AI Agent 开发期契约、确定性与跨仓 CI 评审

- 日期：2026-09-06
- 主审仓库：`LumioGames/LumioGameEngine`
- 主审提交：`23401e178fdf346a0361b51a1ff881daf4d42554`
- 补充抽查：Runtime `50da4bb62610de6163171ce010e73b848a2f92bf`；Server `ce34ac75303f4746c48ba74b706cff6189b038c5`。
- 方法：通过已连接 GitHub 读取当前源码、生成器、开发脚本、工作流与架构文档；参考 Rust、.NET 与 GitHub 官方资料。
- 边界：这是源码与设计审查，不是全仓测试通过证明。本环境未成功取得可执行的完整检出，没有运行 Cargo/.NET/真实 DS 联调，也未修改远端仓库。文档所称历史端到端验收不等于本次提交已重新验收。

## 0. 与仓库最新对账（2026-09-06，主 loop 核查）

> 本节与 §13 是架构仓主 loop 追加的；§1–§12 是外部评审原文（只改了主审 SHA 与三处「未采纳」标记）。

**结论：九条发现在远端 main `23401e1` 上全部仍然成立；PR #95（Go1c，`9057d83`，21 文件）已按本文逐条实现，但连 Owner 不要的锁文件与 required-gate 一起做了进去，且 CI 四个作业红。** 仓里此前没有任何卡、计划或 ADR 跟进本文。

核对方式：文档引用的 `repository-policy.yml`、`eng/dev-run.sh`、`NativeEngineLoader.cs` 在 `23401e1` 与 `b947e8f` 的 git blob sha 一致；`generate-abi.mjs` 的 `expectedRootFields` 仍在；`verify-wire.mjs` 的 `validatorCheck:false` 仍只查声明完整性。

| 发现 | 最新 main 状态 | PR #95 是否处理 | 裁决（§13） |
|---|---|---|---|
| F1 CI 未跑 IntegrationTests / 宿主链 | 仍成立 | 是：`integration` 作业跑 `cargo test` + `NativeLoader.IntegrationTests` + 两轮 DS/Bot | 采纳 |
| F2 dev-run 复用旧产物 | 仍成立 | 是：`dev-run.mjs` 每次 `cargo build` / `dotnet build`，去 `--no-build` | 采纳 |
| F3 只校验 client 一行 | 仍成立 | 是：buildId / abiHash / binarySha256 三项都比 | 采纳 |
| F4 Loader 不比 `BinarySha256` | 仍成立 | 是：sidecar ABI = 编译期 ABI、二进制 SHA = sidecar、根表 = 期望 | 采纳 |
| F5 CI 依赖浮动 | 仍成立 | 是：`workspace-lock.json` 五仓钉 SHA | **不采纳**，返工删除 |
| F6 生成器内嵌第二份真值 | 仍成立 | 否（PR 明确不含） | 真实问题，后排契约工具卡 |
| F7 摘要含绝对路径 / 原文 Hash | 仍成立（修正：`.ps1` 早已相对路径，问题是两平台算法不同） | 一半：相对路径 + 工具链参数进 BuildId；ABI 仍原文 Hash | 采纳已做的一半 |
| F8 全链收口串行 | 仍成立 | 一半：纯文档不拉 DS；候选契约消费未动 | 采纳已做的一半 |
| F9 声明级用例混计 | 仍成立 | 否 | 后排 |

本文两处遗漏：① `eng/dev-run.sh` / `dev-build.sh` 只支持 Linux，Owner 主机 macOS 跑不了收口门槛（R4 审查 P2-2 已记录），这才是最直接的「环境红灯」；PR #95 的 `dev-build.mjs` 仍只支持 linux/win32 x64，本次不改（Server Rust 宿主本就只在 Windows/Linux 链接，D24 已裁）。② F7 只说对一半，见上表。

### PR #95 三处红灯根因

| 作业 | 现象 | 根因 | 性质 |
|---|---|---|---|
| tools | spec-lint 38 处 | 37 处是 `reviews/2026-09-05-dual-transform-bomber-research-gap-audit.md` 指向兄弟仓的相对链接，CI 没检出兄弟仓即悬空（本机有兄弟仓则 OK）；1 处是 PR 新文档未登记进 `knowledge/README.md` | 环境红灯，PR 首次把 spec-lint 接进 CI 才暴露 |
| integration windows | `generate-abi.mjs` 报 voxel wire 源哈希不匹配 | `.gitattributes` 只有 `* text=auto`，JSON 在 Windows 检出成 CRLF，生成器对原文字节算哈希 | 环境红灯，即本文 F7 的「原文 Hash」坑 |
| integration ubuntu | `cargo test` 找不到 `../../../../abi/native-abi.json` | `voxel.rs:1494` 的 `include_str!` 按 `engine/` 布局向上四级找 `abi/`；PR 把 `engine/native` 拷到 `.build/native-workspace-*/` 再测，上四级落到 `.build/` | PR 自己引入 |

## 1. 结论

**保留当前分层和 Living Architecture；把统一的“全链路收口门槛”拆成按变更影响选择的可执行验证。不要恢复旧的全仓 Baseline、镜像和统一 Fixture 发布制度，也不要用跳过 ABI 安全来换开发速度。**

目标工作方式：

> 局部开发不等待全引擎；边界变更验证真实消费者；可交付 SDK 必须来自一组已经联测通过的精确依赖。

真正要避免的不是一切红灯，而是无关红灯、环境红灯和不能定位的红灯。真实 ABI 错误、权威状态错误、数据损坏、协议安全错误仍必须阻断相关交付。

三个概念必须分开：

1. **开发进度**：某模块可以在受控替身与候选依赖下继续实现。
2. **集成完成度**：真实供应方和真实消费方是否已经联通。
3. **可发布程度**：指定 SDK 组合和目标平台是否达到已承诺质量。

模块单测完成不能被写成跨仓集成完成；环境不可用不能被写成测试成功。

## 2. 当前仓库已经做对的事

README 和当前架构正文已经明确：开发期不强制 Baseline、架构镜像或全量 Fixture；允许破坏式 API/ABI 变化，修改唯一来源后重编消费者。[E1][E3]

API、ABI、Wire 的初步划分正确：源码 API 由源码和编译器约束，Native 根接口由 `engine/abi/native-abi.json` 定义，网络和跨进程消息由 `engine/wire/` 定义。SDK 组装根吸收原 CoreEngine 职责，不应重新增加一个 Core 仓。[E1][E3]

Runtime 并不是没有确定性基础。已经存在 `DeterminismContext`、基于逻辑 Tick 的 RNG 流，以及按提供者排序、长度前缀编码并检查完整性的 `StateHashCoordinator`。应继续利用这些能力，而不是另造全引擎 Hash 框架。[E13][E14]

已有真实 Native 集成测试 `LoadsTheStagedNativeImageAndExecutesTheRootApi`，说明最小跨语言测试入口也不是从零开始。[E10]

## 3. 源码级发现与优先级

### F1 / P1：CI 名称与实际执行证据不一致

`.github/workflows/repository-policy.yml` 的 job 叫 `Build SDK and prove Host loading`。当前步骤生成 ABI、构建 Native、执行 SDK Rust 测试和 Loader 单测，再构建 C# MVP Server 与 Bot；**没有运行 Host 加载脚本，没有执行现有 `NativeLoader.IntegrationTests` 项目，也没有执行 Rust Server → CoreCLR → Runtime → Bot 的消息闭环**。[E4][E10]

这是工作流能力的缺口，不表示仓库从未做过人工集成。架构正文确实记录了 LumioGame 中的历史 Hello 验收，但不是当前 CI 自动重放的证据。[E3]

**建议**：首先接入现有 Native IntegrationTests；随后为真实 Rust Server 增加最小无界面集成 job。若 C# MVP Host 仍保留，应标为 legacy/独立兼容目标，不以它的构建代替当前 Rust DS 主链验证。

### F2 / P1：开发脚本可能复用旧 Server、Runtime 与 Bot

`eng/dev-run.sh` 和 `eng/dev-run.ps1` 都在找不到 Server 二进制时才执行 Cargo build；Runtime HelloEntry 从已有 bin 目录寻找；Bot 使用 `--no-build`。[E5][E6]

因此这些脚本**不能单独证明“所有参与模块都是本次源码对应的产物”**。Native 本次构建并不能证明 Runtime DLL 或 Server EXE 也最新。

**建议**：每次调用正常增量构建，让 Cargo/MSBuild 判断是否需要重编；不能用“文件存在”替代增量构建。受影响构建目标按依赖闭包选择，构建结果返回显式产物路径。集成进程启动前验证每个产物的输入指纹和 SHA。

### F3 / P1：加载证明还不等于双端一致，更不等于 Gameplay 正确

Bash 脚本观察 `SERVER_READY` 与 Client 的 `ENGINE_NATIVE` 行，只对 Client 行验证 BuildId 和 ABI Hash。PowerShell 脚本读取了 expectedAbiHash，但成功分支只验证 Client BuildId。[E5][E6]

脚本没有完整比较两端的构建清单，也没有断言 Bot 实际向该 Server 发出命令并收到权威结果。观察进程启动和观察真实消息闭环是两种证据。

**建议**：拆成 `native-load-smoke` 与 `ds-bot-smoke` 两个测试。前者验证真实 ABI 调用和产物身份；后者用 readiness 返回的实际地址连接，发命令、检查 Tick/Revision/结果、关闭并确认退出。两种测试分别报结果，不互相冒充。

### F4 / P1：共享 Loader 的身份与完整性验证没有闭合

`NativeEngineLoader.Load` 计算了实际文件 SHA，但没有与 `NativeBuildInfo.BinarySha256` 对比。`LoadFromBuildInfo` 虽读取 sidecar，随后仅把其中 BuildId/AbiHash 传入 `Load`。[E11]

另外，生成文件已有 `AbiConstants.DefinitionSha256`，但这个入口没有拿它和加载表的 ABI Hash 比较。读取 sidecar 后让 sidecar 与 DLL 自比，只能证明某种产物自洽，不能独立证明当前消费端绑定与 DLL 一致。[E11][E12]

**建议**：开发期先保留严格的编译期 ABI 匹配；同时检查运行清单的构建身份和实际二进制 SHA。将“消费端需要什么 ABI”和“本次预期运行哪个构建”作为独立输入。不要让旁边的 sidecar 自己充当全部可信期望。

这是已读共享入口的缺口；本次没有逐个核查所有上层调用方是否另做补充验证，也没有动态演示错误加载。

### F5 / P1：多仓 CI 依赖浮动，失败不一定由当前 PR 引入

工作流 checkout NativeCore、Voxel、Server、Client、Runtime 时未指定 ref。[E4]

这样的 CI 同时受多个仓默认分支移动影响，难以重复得到相同输入组合。要提高 Agent 效率，应减少漂移，不应取消契约验证。

**建议**：正常组件 PR 对“最近验证通过的精确依赖集合”运行；跨仓候选联测显式覆盖相关 SHA；追最新各仓的组合放到定时集成中。这里的精确集合是自动维护的构建锁文件，不是旧的人工架构 Baseline 制度。

### F6 / P2：单一契约来源尚未变成完整的单次编辑生成链

`generate-abi.mjs` 除读取 JSON，还手写 `expectedRootFields`、固定 Root 数量/顺序、布局偏移和错误数量；C# Loader 仍直接定义 `RootApi` 字段；Server 的 `native_abi.rs` 也声明自己的消费前缀类型。[E8][E9][E15]

这些保护并非毫无价值：独立布局测试、稳定槽位断言能够发现生成器错误。问题在于**日常生产声明与生成器内部又承担了一份规范真值**，使增改接口需要同步多个位置。

**建议**：生产 Header、Rust/C# 消费布局、函数签名、状态码由同一声明生成。保留少量独立审阅的 golden 布局和真实 ABI 测试；不要让生产字段清单长期藏在生成器代码里。Server 可以消费合法稳定前缀，但前缀也应生成，而不是手工复制。

### F7 / P2：Hash 的变化范围过大，又没有覆盖全部真正构建输入

生成器直接对 ABI JSON 原始字节做 Hash，并要求 Voxel wire 原文 Hash 与 ABI 元数据一致。因此纯格式或说明文字变化也会改变 Hash。[E8]

`dev-build.sh` 的 `source_digest` 扫描三个仓的广泛文件，把绝对路径写进摘要。由代码可推知：相同源码换 worktree 路径会产生不同 BuildId，很多无关文档也参与摘要；配置、目标 triple、实际编译器身份等并没有全部被该函数显式纳入。Native 指纹也不覆盖 Runtime/Server/Client 源码。[E7]

**建议**：区分 ABI 布局指纹、合约行为变化、构建输入指纹和二进制 SHA；Native 指纹明确叫 Native 指纹，不声称代表整套产品。详见第 5 节。

### F8 / P2：实际规则仍有“全链收口”和上游先合入的串行约束

`.spec/AGENTS.md` 的开发态收口门槛仍固定为 ABI 生成加 PowerShell dev-run；架构质量边界对 Native/托管内部改动要求单测、SDK 构建和双端加载。[E2][E3]

`engine/wire/README.md` 还要求 C 卡先合入 architecture main，下游生产代码才能出现在 main。[E16]

**建议**：将局部任务完成、相关 PR 合入和 SDK 组合晋级分开；允许下游基于同源候选契约并行编译与测试。不能通过下游自定义替代协议绕过唯一来源，但也不应要求所有候选验证先等待上游 main。

### F9 / P2：声明级校验必须与实际语义执行分开报告

`verify-wire.mjs` 明确把 `validatorCheck:false` 用例仅做声明完整性检查。文件自身的说明是诚实的，但若上层只显示“全部用例通过”，容易被误读为生产语义全部验收。[E17]

**建议**：报告区分 `schema_validated`、`case_executed`、`integration_executed`。保留声明校验，但不能计为相应 Runtime/Host 行为测试通过。

## 4. 跨层关系：不同边界用不同验证

| 边界 | 保留的所有权 | 首选验证 | 不要增加的负担 |
|---|---|---|---|
| NativeCore ↔ Voxel ↔ SDK 聚合内部 | 各 Rust crate 的源码 API；SDK 负责聚合 | Cargo 编译、单测、直接及传递消费者构建 | 每次内部函数修改登记公共 ABI |
| Runtime ↔ Client/Server/Game 托管代码 | 公共托管 API 归语义拥有仓 | C# 编译、行为测试、受影响消费项目构建 | 把全部内部 C# 类型做成公共 Schema |
| Rust Host / C# ↔ SDK 动态库 | `engine/abi/native-abi.json` | 生成、布局、真实装载、真实调用、生命周期测试 | 手写第二套 Root 或关闭 ABI 检查 |
| SDK ↔ CoreCLR 托管入口 | 明确的入口签名、缓冲与失败语义 | 独立进程调用、容量探测、错误传播、关闭测试 | 用 Ping 代替真实 CLR 调用 |
| Server ↔ Client/Bot | `engine/wire/` | 实际两端 codec、有效/无效包、最小连接闭环 | 每次普通实现改动要求全量协议兼容矩阵 |
| 存档 ↔ Runtime/Voxel | 各自数据语义及已有持久化边界 | round-trip、损坏拒绝、明确的开发期重置/迁移规则 | 开发期强制兼容全部历史临时存档 |

特别说明：ABI 不是“只有 C# 调 Rust 才存在”。实际 Rust Server 也动态装载 SDK 并读取 C ABI 根表。[E15] 更准确的规则是：**是否跨动态二进制边界决定 ABI 验证，而不是两边是否使用不同语言。**

DS 是权威 Host，不是第二套 Runtime。不要在 DS 中复制 Runtime Tick 规则、在客户端复制一份权威提交逻辑，或在 SDK 聚合层增加玩法状态。测试要覆盖这些状态所有权，不能只验证类型名相同。

## 5. 四类指纹和“确定性”不要混为一谈

### 5.1 ABI 布局指纹

对参与二进制布局和调用的机器可读投影计算：调用约定、类型宽度、字段/槽位顺序、对齐、函数签名、相关枚举值等。JSON 对象键可规范化；数组顺序不能擅自排序，Root 槽位顺序本身就是语义。

当前从原文 Hash 迁移到投影 Hash 应作为一次明确的契约工具变更，生成器、Loader 和消费者一起验证。不应在迁移期间静默忽略旧 Hash。

### 5.2 行为契约变化

缓冲所有权、错误优先级、线程约束、提交语义发生变化，即使布局相同也需要相关契约测试。当前很多行为写在 `doc` 中；不能简单“删除所有 doc 再 Hash”后声称所有契约语义不变。

可将确实需要机械比较的少数行为结构化，其余由审阅和行为测试负责。不要为了 Hash 再造一个大型契约语言。

### 5.3 构建输入指纹

建议组成：逻辑仓名 + 仓内相对路径 + 有效输入内容 + 精确依赖 + 工具链 + target triple + features + 构建参数。

普通说明文档、评审输出、临时 worktree 和构建输出不属于 Native 构建输入；若某文档确实被生成器作为输入，仍须纳入。每个实际产物有自己的输入指纹，组合清单再记录它们。

### 5.4 二进制完整性与世界状态 Hash

二进制 SHA 对实际 DLL/SO/托管程序集计算，验证复制和选择了正确文件；它不证明 ABI 兼容或 Gameplay 正确。sidecar 与二进制一起被替换也不构成供应链真实性证明，正式分发时另按安全需求处理签名。

世界状态 Hash 对同一规范化输入下的权威状态计算，验证模拟行为。不能拿 BuildId 或二进制 SHA 代替。

Windows DLL 与 Linux SO 无须 SHA 一样；不同机器路径也不该导致相同有效源码的逻辑输入指纹变化。开发期无需要求跨机器编译产物字节完全可重复，除非已经承诺这种构建能力。

## 6. 推荐的测试分层

以下时间是建议的暖缓存反馈预算，不是当前实测值。先测量，再优化；不作为本次工作耗时承诺。

| 层次 | 内容 | 运行时机 | 阻断范围 | 目标预算 |
|---|---|---|---|---|
| L0 局部测试 | 模块编译、单测、静态分析；只在相关变化时检查生成物 | Agent 编辑与局部交付 | 当前模块 | 30–120 秒 |
| L1 边界测试 | 直接消费方构建、真实 ABI 布局/调用、小型 codec 向量 | 涉及边界的 PR | 相关变更 | 约 5 分钟 |
| L2 最小集成 | Rust DS + CoreCLR + Runtime + Native + 无界面 Bot | 相关 PR、跨仓候选晋级 | 相应组合 | 约 10–15 分钟 |
| L3 广覆盖 | 多平台、长回放、故障注入、压力、Unity/AOT 目标 | 定时集成、目标平台相关 PR | 默认不阻塞无关局部开发；不得晋级失败组合 | 独立运行 |
| L4 发行验证 | 兼容窗口、迁移、签名、部署和性能承诺 | 真正发行阶段 | 发布产物 | 按发行目标 |

不能把所有真正的 Native 测试推到夜间。ABI、Loader、内存、线程边界、Tick/提交点变更应在相关 PR 上触发真实测试。普通 UI 文案和纯文档改动则不应自动拉起 DS。

### L0：模块独立，但替身不能冒充生产

复用现有 Cargo test、xUnit 和仓内测试设施。不另建大型全引擎测试框架。

时间、随机数、外部 IO 使用测试可控入口。测试替身实现正式端口，放在测试设施，不定义第二套 wire/ABI。上游实现未完成时可以验证模块内部逻辑；对应集成状态必须标为未验证。

### L1：真实跨语言测试至少覆盖什么

小而真实的一组用例：Root 装载和 Ping；合法生命周期；版本/短表/必需槽位拒绝；字段宽度/对齐/offset；缓冲容量不足和实际长度；空指针的约定；stale Handle；关闭后访问；Rust 错误与托管错误不能越界传播。

复用已有 `NativeEngineIntegrationTests`，然后补业务端口调用。纯反射或 `Marshal.SizeOf` 测试不能代替对真实 DLL/SO 的调用。危险负例必须运行在独立进程中，避免一个 Native 崩溃杀掉所有测试。

布局规范遵守 Rust `repr(C)` 与 .NET 明确的互操作签名，不假设外层 `repr(C)` 自动修正内层不稳定布局。[O1][O2]

### L2：先接已有链路，再扩玩法

先复用当前 Hello/Entity-Chat 主链，自动完成：

`Rust Server 启动 → CoreCLR 入口 → Bot 连接 → 命令入队 → 权威 Tick → 响应/复制 → Bot 断言 → 正常退出`。

进而加入一个真实玩法纵切，例如炸弹倒计时触发、实体变化、体素破坏和对应复制；没有真实实现的步骤应作为后续目标，不用 Fake “验收通过”。

测试必须检查请求/响应关联、目标 World、Tick/Revision、结果内容、至少一次真实 Native 调用和进程退出。readiness 应提供实际端口；不使用固定 sleep 来猜是否启动完毕。

失败注入先选择高价值少量场景：重复命令、旧 generation、错误包、队列满、提交失败、断线重连。重复/幂等/重连行为严格按当前契约断言，不重新发明规则。

内部字段写入失败可能按现有 Fail-stop 语义使 World Faulted，不应为了测试好看承诺不存在的字段级自动回滚。核心断言是：不得把失败 Tick 发布成成功，不得继续使用不再有效的 World。

## 7. 确定性：按范围和等级验收

### 7.1 保留当前基础，避免把全世界完整性要求套在每个小单测上

`StateHashCoordinator` 已有固定必需提供者、13 相记录以及 `IsComplete`。这对完整权威回放是有价值的，不能为了通过测试把它们删掉。[E13]

模块单测可以只比较自身状态；测试报告注明 `scope=module`。完整回放必须保留完整性检查；声明启用的 ECS/GAS/Voxel 状态缺失时，不得出具“全引擎确定性通过”。测试层可以组合不同 Harness，但不能静默降低生产完整性判据。

### 7.2 开发期优先要求同构运行可复现

固定初始快照、配置、随机种子、已经裁决并规范化的输入序列，在两个新进程或独立 World 上重复运行；比较逐 Tick Hash，并输出首差异。

记录网络/IO 被主循环接纳的顺序。不能把两次随机网络到达先后不同，却合法分配到不同 Tick 的运行，误判为同一输入下的确定性失败。

并行调度测试可以扰动工作任务完成顺序，但保持规范化输入与承诺的归并语义一致。RNG 不仅要种子一致，还需要稳定的流划分和消费顺序。

### 7.3 跨平台先明确承诺，不承诺无限范围位级一致

整数 ID、计数、顺序和协议整数应严格一致。浮点路径对跨 OS/CPU/运行时的保证要单独制定；不要默认所有 .NET/Unity/Native 目标都会得到逐位一致结果。[O3]

如果核心需求真的是跨平台位级锁步，就需要相应数值策略和跨平台测试，不能用容差掩盖。如果只是表现平滑，容差也必须有清晰范围，不得用于金额、ID、伤害计数或掩盖重复提交。

### 7.4 比较权威语义投影，不比较所有内存

纳入 Entity 身份、权威组件、GAS 有效状态、Voxel 状态/Revision、影响未来的 RNG 与 Tick Timer 等。排除对象地址、日志时间、线程 ID、纯渲染缓存。

Runtime 当前 Hash 包含 session/world/release/manifest 等身份项。[E13] 同版本回放应复用其逻辑身份；做跨构建语义比较时，需明确规范化测试身份和单独比较的语义投影，不能因为 BuildId 改变就宣称玩法不同，也不能随意删除会影响行为的配置。

首差异输出示例（目标格式，不代表已存在接口）：

```text
Scenario: bomb-chain
Tick: 184
Provider: state.gas
Entity: 1024
Field: Health.Current
Expected: 70
Actual: 60
Seed: 424242
Replay: artifacts/replay-inputs.json
BuildSet: artifacts/resolved-build.json
```

### 7.5 Golden 更新必须解释行为变化

同一实现跑两次相同，只能证明可重复，不能证明符合玩法期望。还需固定场景的语义断言、边界反例和少量独立审阅的 golden。

玩法规则合法变化可以更新 golden；必须同时提交语义说明与差异。Agent 不能为过 CI 自动全量刷新 expected 值。

## 8. 多仓频繁变化：采用精确候选组合，不采用全局冻结【未采纳——见 §13，撞 Owner「跟 main 不钉号」裁决】

建议只新增一份轻量构建依赖锁文件与一份每次运行产生的清单：

- `eng/dependencies.lock.json`：自动维护的最近可用依赖集合，记录精确 SHA 或不可变产物标识。
- `artifacts/resolved-build.json`：本次实际 checkout、工具链、平台、features、产物 SHA 和测试范围；这是运行证据，不是新的公共协议。

本地可以显式用 sibling worktree 覆盖锁定依赖，但应在开跑时固定快照；并发 Agent 不应从正在变动的另一工作区读取“某个时刻的源码”。如果允许 dirty 开发树，必须记录内容指纹，不伪装成 clean commit。

初版可以只用精确 Git SHA + checkout/worktree 组合，不必先建设完整包注册平台。已有可用包机制时，再以同样语义消费不可变包。

### 一个跨仓 ABI 修改的正确顺序

1. 在 Engine 的唯一定义上提出候选修改并生成绑定。
2. Runtime/Server/Client 的受影响 Agent 使用该精确候选版本并行实现，不必等正式 main。
3. 集成任务把候选 SHA 组成一个隔离工作区，构建并运行相关 L1/L2。
4. 测试通过后按仓依赖顺序合入；最终落地 SHA 或 merge commit 变化时重新验证受影响组合。
5. 只有通过的最终组合才成为默认 SDK/依赖锁；失败时旧组合继续可用，相关候选不被发布。

多 Git 仓不能靠一句“同步合入”获得原子事务。需要保证的是默认消费集合一致，而不是所有仓 main 的最新提交天然能够任意搭配。

开发期允许破坏式修改，不需要提前建设永久 N/N-1 双协议。已联调通过的精确组合可以整体升级；只有明确有滚动升级/外部客户端兼容需求时才引入兼容窗口。

## 9. CI 如何既不误卡又不假绿【未采纳 required-gate / plan 影响选择——见 §13】

建议以一个稳定的必选汇总 job `required-gate` 收口，其他 job 由影响分析选择。该名称是建议，不是仓库已有实现。

```text
plan-impact
  ├─ unit / compile（选中的模块）
  ├─ contracts / generated-check（选中的边界）
  ├─ native-interop（真实 ABI）
  ├─ deterministic-smoke（权威行为相关变化）
  └─ ds-bot-smoke（连接与权威链相关变化）
            ↓
       required-gate
```

`required-gate` 无论前置成功、失败或取消都需给出结果。它核对计划应跑项与实际结果；不是简单看是否存在失败字符串。

| 结果 | 含义 | 必选 Gate |
|---|---|---|
| PASS | 所有应跑断言成功 | 通过 |
| FAIL | 编译/断言/崩溃失败 | 不通过 |
| BLOCKED_ENV | SDK、权限、Runner 或依赖不可用 | 不通过；分类为环境，不伪装代码缺陷 |
| SKIPPED_SCOPE | 影响计划判定无关 | 可接受，但不得跳过应跑检查 |
| NOT_IMPLEMENTED | 能力尚未实现 | 不可对已声明支持的能力签发通过 |
| CANCELLED / TIMEOUT | 验证未完成 | 应跑项不得按通过处理 |

GitHub 官方提醒：用路径过滤跳过整个必选 workflow，会使相关检查停留 Pending；而条件跳过某个 job 又可能显示 Success。因此应让汇总 Gate 始终运行，并验证哪些 job 本次本来就必须执行。[O4]

补充规则：生成物检查应“生成到临时目录后比较”或生成后做精确 diff，不让 CI 悄悄修好未提交文件；缓存键包含依赖/工具链/目标/生成器相关指纹；影响分析结果未知时保守扩到相关构建层，而不是默认全部跳过。

定时全量跑所有测试用于检测影响选择器漏测和多仓漂移。发现红灯后冻结坏组合晋级，而不是冻结所有无关仓开发。安全、损坏、确定性等核心用例不应通过 quarantine 永久规避；非核心 flaky 的临时隔离必须有负责人和退出条件。

## 10. 对 AI Agent 友好的最小工作流【`check.mjs --profile` 与依赖锁未采纳；三档入口已由 `eng/test.mjs` 实现——见 §13】

不新增一套大型流程平台。优先给现有脚本加薄编排入口，复用 Cargo、dotnet test、Node 验证器和已有 Fixture。

建议入口，以下尚未实现：

```bash
node eng/check.mjs --profile local --changed
node eng/check.mjs --profile pr --changed
node eng/check.mjs --profile integration --lock eng/dependencies.lock.json
```

本地入口不强迫纯 Runtime 单测先装 Unity，也不强迫纯 Native 单测先启动账号平台。ABI/Host 集成才准备所需 Native/CoreCLR/Server 环境。

任务卡只需明确目标、可改文件、正式输入端口、首个失败测试、应跑命令和完成定义。跨模块变更自动附依赖影响报告，不让每个 Agent 重新阅读全部历史 ADR。

独立 worktree、进程、端口、临时目录和可写构建输出避免互相污染；允许共享受控的只读/内容寻址缓存。当前 AGENTS 已记录并发 MSBuild 的 obj 争用，应该保留隔离方向。[E2]

Agent 最终报告至少包含：实际构建集合、运行测试、失败位置、未验证面和可复现命令。错误输出应包含契约字段/函数、expected/actual、哪个消费者失败、首差异 Tick 和 replay 文件。

生成器、验证器、golden、影响选择器、应跑测试清单属于高风险面：需要独立审阅。它们不能因为 diff 很小就套用普通小改豁免；也不能允许实现 Agent 删除断言或改期望值来通过任务。

## 11. 落地顺序：先减少误阻塞和假通过

| 顺序 | 文件/模块 | 具体交付 | 验收 |
|---|---|---|---|
| 1 | `eng/dev-run.*`、`eng/dev-build.*`、Loader | 正常增量构建；显式产物清单；比较 binding ABI、产物 SHA；保存失败日志 | 修改 Server/Runtime 源码后不会复用旧产物；替换 sidecar 或 DLL 能被正确识别 |
| 2 | `.github/workflows/repository-policy.yml` | 接入现有真实 Native 集成测试、wire 验证和当前 Rust DS 链路；生成物差异检查 | 真实 ABI/绑定改坏时 CI 红；单靠 Host 编译成功不再签发加载证明 |
| 3 | `.spec/AGENTS.md`、架构质量边界、`engine/wire/README.md` | 分开模块完成、PR Gate、组合晋级；允许候选契约消费 | 无关局部任务不再被 dev-run 环境阻塞，跨仓 PR 可并行验证 |
| 4 | 新增轻量 `eng/check.mjs` 与依赖锁 | 按影响选测试；固定依赖；输出结构化结果 | 重跑同一输入组合可复现；应跑项跳过不假绿 |
| 5 | `generate-abi.mjs`、C#/Rust 消费绑定 | 生产布局单源生成，保留独立布局证据；指纹语义拆分 | 新增槽位不需手工复制多个 Root；格式变更不伪装为布局变更 |
| 6 | Runtime testing、Engine 集成、Game 玩法场景 | 短回放首差异；真实玩法纵切；扩充故障测试 | 单测能定位、集成能联通、玩法场景能证明预期行为 |

不要等第 6 步完成才允许各模块开发。第一步修证据，第二步复用已有测试，第三步立即解除无关全链门槛，其余增量推进。

## 12. 最终验收定义

**模块完成**：模块编译、模块测试、正式接口消费和已声明行为通过；依赖替身仍存在时标明未集成。

**边界完成**：真实供应方与真实消费方，在精确产物组合上完成正常调用、拒绝路径和生命周期测试。

**引擎组合可用**：Rust DS、Native、CoreCLR、Runtime 和 Client/Bot 完成最小闭环；声明确定性场景通过；产物身份可复现；退出无残留。

**产品场景完成**：LumioGame 的真实玩法命令、ECS/GAS/体素效果、复制和用户可观察结果按游戏需求验收。

没有有限测试集能保证永不出现新 bug。上述方案要保证的是：重要边界有明确、自动、可复验的判据；失败能定位；坏组合不成为默认 SDK；局部开发不被无关验证拖住。


## 13. Owner 裁决与落地（2026-09-06）

Owner 原话：「我希望尽可能的开发效率，不要太多的锁定我们这些东西。有问题我们修就好了」「都跟 main，红就当天红、当天修——一定要按这个规范做」「完成之后把文档规则整理成一个约束规范，以后只保留这一套，之前的决策不要误导 Agent」。机器落点：[ADR-068](../decisions/ADR-068-development-verification-follow-main.md)。

| 条目 | 裁决 | 一句理由 |
|---|---|---|
| F1 / F2 / F3 / F4 / F7（相对路径 + 工具链）/ F8（纯文档不拉 DS） | 采纳，PR #95 已实现 | 修的是「旧产物冒充新的」和「假绿」，不新增任何锁 |
| F5 + §8 依赖锁文件 | **不采纳** | 撞「跨仓跟 main 不钉号」；Client CI 钉旧 Runtime 就是上周假绿的来路 |
| §9 `plan` 影响选择 + `required-gate` | **不采纳** | 前期不堆工具；三个作业每个 PR 都跑，一种写法 Agent 一眼看全；若集成时长成痛点再单独议路径过滤 |
| §10 `check.mjs --profile`、`resolved-build.json` | 不采纳形式，采纳三档入口 | `eng/test.mjs tools / managed / integration` 已是三档入口；`verification.json` 记录实际 SHA 与 dirty 作证据即可 |
| §5 四类指纹、§6 五层、§7 确定性分级 | 只作原则记录，不建卡 | 进入硬化阶段再议 |
| F6 生成器单源、F9 汇总分计 | 真实问题，后排 | 契约工具改造卡，等 A 系列 ABI 卡稳定后开 |

落地顺序：① PR #95 返工（删锁文件 / plan / required-gate、修三处红灯）→ 三作业全绿合入；② 验证规则收成唯一一套：`knowledge/standards/development-verification.md` 是唯一规范，`AGENTS.md` 收口门槛、`architecture.md` §6、`engine/wire/README.md` 只引用不另写；③ ADR-068 Accepted，取代本文 §8–§10 及此前一切「钉 sha / 镜像 / 基线」口径。

---

## 证据索引

主仓条目均固定到本次主审 SHA，避免阅读时 main 已变化。

- [E1 README](https://github.com/LumioGames/LumioGameEngine/blob/23401e178fdf346a0361b51a1ff881daf4d42554/README.md)
- [E2 Agent 开发规范](https://github.com/LumioGames/LumioGameEngine/blob/23401e178fdf346a0361b51a1ff881daf4d42554/.spec/AGENTS.md)
- [E3 当前架构正文](https://github.com/LumioGames/LumioGameEngine/blob/23401e178fdf346a0361b51a1ff881daf4d42554/.spec/knowledge/features/architecture.md)
- [E4 CI 工作流](https://github.com/LumioGames/LumioGameEngine/blob/23401e178fdf346a0361b51a1ff881daf4d42554/.github/workflows/repository-policy.yml)
- [E5 Bash dev-run](https://github.com/LumioGames/LumioGameEngine/blob/23401e178fdf346a0361b51a1ff881daf4d42554/eng/dev-run.sh)
- [E6 PowerShell dev-run](https://github.com/LumioGames/LumioGameEngine/blob/23401e178fdf346a0361b51a1ff881daf4d42554/eng/dev-run.ps1)
- [E7 Bash dev-build](https://github.com/LumioGames/LumioGameEngine/blob/23401e178fdf346a0361b51a1ff881daf4d42554/eng/dev-build.sh)
- [E8 ABI 生成器](https://github.com/LumioGames/LumioGameEngine/blob/23401e178fdf346a0361b51a1ff881daf4d42554/eng/generate-abi.mjs)
- [E9 ABI JSON](https://github.com/LumioGames/LumioGameEngine/blob/23401e178fdf346a0361b51a1ff881daf4d42554/engine/abi/native-abi.json)
- [E10 Native 集成测试](https://github.com/LumioGames/LumioGameEngine/blob/23401e178fdf346a0361b51a1ff881daf4d42554/engine/managed/Lumio.Engine.NativeLoader.IntegrationTests/NativeEngineIntegrationTests.cs)
- [E11 共享 Loader](https://github.com/LumioGames/LumioGameEngine/blob/23401e178fdf346a0361b51a1ff881daf4d42554/engine/managed/Lumio.Engine.NativeLoader/NativeEngineLoader.cs)
- [E12 生成的 ABI 常量](https://github.com/LumioGames/LumioGameEngine/blob/23401e178fdf346a0361b51a1ff881daf4d42554/engine/managed/Lumio.Engine.NativeLoader/AbiConstants.g.cs)
- [E13 Runtime StateHashCoordinator](https://github.com/LumioGames/LumioGameRuntime/blob/50da4bb62610de6163171ce010e73b848a2f92bf/modules/simulation/src/Lumio.GameRuntime.Simulation/Determinism/StateHashCoordinator.cs)
- [E14 Runtime DeterminismContext](https://github.com/LumioGames/LumioGameRuntime/blob/50da4bb62610de6163171ce010e73b848a2f92bf/modules/simulation/src/Lumio.GameRuntime.Simulation/Determinism/DeterminismContext.cs)
- [E15 Rust Server Native ABI 消费](https://github.com/LumioGames/LumioServer/blob/ce34ac75303f4746c48ba74b706cff6189b038c5/modules/host-runtime/src/native_abi.rs)
- [E16 Wire 契约规则](https://github.com/LumioGames/LumioGameEngine/blob/23401e178fdf346a0361b51a1ff881daf4d42554/engine/wire/README.md)
- [E17 Wire 校验器](https://github.com/LumioGames/LumioGameEngine/blob/23401e178fdf346a0361b51a1ff881daf4d42554/eng/verify-wire.mjs)
- [O1 Rust Reference：Type Layout](https://doc.rust-lang.org/reference/type-layout.html)
- [O2 Microsoft：Native interoperability best practices](https://learn.microsoft.com/en-us/dotnet/standard/native-interop/best-practices)
- [O3 Microsoft：System.Double](https://learn.microsoft.com/en-us/dotnet/api/system.double?view=net-10.0)
- [O4 GitHub：Troubleshooting required status checks](https://docs.github.com/en/pull-requests/how-tos/merge-and-close-pull-requests/troubleshooting-required-status-checks)
