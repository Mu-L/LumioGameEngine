---
name: development-verification
description: 开发期三档验证入口（tools / managed / integration）与依赖仓跟 main 的规则——开发、改 CI 或收口前查
metadata:
  type: doc
  status: 实施中
---

# 开发期验证

局部任务不等待完整引擎环境；真实边界修改必须通过对应联测。本规范替换旧的“一律生成 ABI 并启动全部 Host”收口方式，不改变世界模型、公共 ABI 布局或 wire 语义。

## 入口

从 SDK 仓根目录运行。所有入口需要 Node.js 22.16.0 或兼容的 Node 22；托管测试需要 .NET 10；集成需要 Rust 工具链、.NET 10、对应原生编译工具和五个实现仓。

| 命令 | 作用 | 是否需要其他仓 | CI 作业 |
| --- | --- | --- | --- |
| `node eng/test.mjs tools` | Node 回归测试、wire 验证、隔离生成物一致性、spec-lint | 否 | `tools`，每次都跑 |
| `node eng/test.mjs managed` | Loader 单测和错包/短表反例 | 否 | `managed`，每次都跑 |
| `node eng/test.mjs integration` | Native Rust 测试、真实 C# Native 装载、Rust DS + CoreCLR + Runtime + C# Bot 两轮闭环 | 是 | `integration`（ubuntu + windows），每次都跑 |
| `node eng/test.mjs all` | 顺序执行以上检查 | 是 | — |
| `node eng/dev-build.mjs` | 增量构建 Native，输出本次确切产物路径和构建身份 | NativeCore、Voxel | — |
| `node eng/dev-run.mjs` | 构建当前 Native/Runtime/Server/Foundation Bot，验证装载并正常关闭 DS | 是 | — |
| `node eng/dev-run.mjs --keep-running` | 构建后保留 DS，Ctrl-C 请求正常关闭 | 是 | — |

现有 `dev-build.sh/.ps1`、`dev-run.sh/.ps1` 只负责参数转发，不再维护四套构建/启动实现。PowerShell 保留 `-VoxelRoot`、`-NativeCoreRoot`、`-KeepRunning`，新增 `-Verify` 对应 `--verify`。纯文档改动执行 tools，不需要安装 .NET 或检出五个依赖。

生成物过期时，先运行 `node eng/generate-abi.mjs`，审阅生成 diff 后再测。检查命令在临时目录生成，不改写开发工作区；不通过“先提交再测试”解除检查。

## 依赖仓跟 main

五个依赖仓 `LumioNativeCore`、`LumioVoxelEngine`、`LumioGameRuntime`、`LumioServer`、`LumioClient` 一律跟各自 `main`：CI 直接 checkout 各仓默认分支，本地默认取同级目录的当前状态。不钉 SHA、不维护锁文件、不做候选组合选择；跨仓改动把 CI 弄红了当天修，不靠钉旧版本绕过。

本地允许用 `NATIVE_CORE_ROOT`、`VOXEL_ROOT`、`LumioRuntimeRoot`、`LumioServerRoot`、`LumioClientRoot` 指定候选 worktree；工具不会自动 checkout、reset 或修改其他仓。

`verification.json` 仍记录本次实际用到的每仓 SHA 与 dirty 标志、Native 构建参数、工具链输出以及本次 Server/Runtime/Bot 文件 SHA 作为证据。源码可快速变化，但“测试用了什么”必须可回溯。Rust stable 和 .NET 10.0.x 仍由安装器解析；证据记录实际版本，这不是完全密闭的可重现工具链保证。

## 构建与装载

每次运行都调用 Cargo/MSBuild 的增量构建，不以旧文件存在作为跳过条件。Runtime/Bot 输出到本次唯一目录，Server 可执行路径来自 Cargo 的 compiler-artifact 结果。Native 输出同样使用唯一 staging 子目录，不覆盖旧进程映射的文件。

Native 源码摘要使用仓内相对路径和仓库标识，不包含绝对 worktree 路径或 `.spec` 文档。有效 crate 输入、已生成绑定、Cargo 锁文件仍参与摘要。BuildId 另外包含目标、configuration、工具链和选定构建环境参数；构建后文件 SHA 单独计算。构建过程中输入发生变化则拒绝发布该次产物，要求隔离 worktree 后重试。摘要不是发行签名，也不保证所有外部编译器环境均已密闭。

ABI Hash 本次继续采用原有原文定义 Hash，避免只改 SDK 一端导致跨仓身份语义漂移。布局 Hash 与行为契约 Hash 的进一步拆分不属于本次已完成范围。

Loader 检查三件不同的事：sidecar ABI 必须匹配编译进消费端的 ABI；实际二进制 SHA 必须匹配 sidecar；Native 根表中的 ABI/BuildId 必须匹配消费端和本次选择。不能用 DLL 与自己的 sidecar 相互匹配替代消费端绑定校验。先读取固定头的大小，再读取完整表，避免先读取后检查的短表问题。开发期也不允许跳过 ABI、文件完整性或真实错误检查。

## CI 与证据

CI 只有 tools、managed、integration 三个作业，每个 PR / push 都全部跑，不按改动路径选择、不设汇总门禁作业。三个作业全绿才算通过；skipped、cancelled、failure 都不能冒充通过。哪些作业设为分支保护必选项由仓库 Owner 管理。

集成证据保存在 `.run/verification/run-*/`。失败不删除日志。`verification.json` 区分 PASS、FAIL、BLOCKED_ENV，并保存实际输入与产物身份；CI 的 artifact 上传即使失败也执行，但没有文件不等于测试通过。

两轮 Hello 验证复用真实 LumioClient C# Bot、Rust DS 与 Runtime。Node 仅作为测试用的 browser-role 协议对端，字段常量从当前 hello-wire 定义取得，不是第二个生产 Browser SDK。每轮校验 baseline、双向命令与 Delta、真实 ingress/tick/egress 审计、正常进程退出；随后比较两轮 sender/sequence/tick/revision/payloadHash。网络 Echo、缺提交、错 Hash、重复结果和缺 shutdown 都有负例测试。

该投影只证明 Hello 窄闭环的复现，不代表完整 ECS/GAS/Voxel 世界确定性，也不替代 LumioGame 的真实 Chromium/UI 验收、性能验收、存档迁移或多平台长回放。客户端 Local Entity、墙钟延迟不应混入此投影。后续完整世界回放仍必须要求权威状态提供者完整，缺失不得降级为通过。

## 完成声明

模块测试通过、边界联测通过、SDK 组合可用、完整游戏可交付是不同结论。环境不可用要记录 BLOCKED_ENV；测试替身、单次 Ping、静态结构检查、手工报告均不能替代真实供应方联测。不得自动刷新 Golden、删除断言或放宽必跑集合来修复红灯。验证器、ABI/Loader 和 Golden 变更必须接受审查。

当前仍待后续实施：全量 Root Binding 单源生成清理、ABI/行为 Hash 分离、完整世界首差异诊断、长回放及发行矩阵。不得将本次基础验证链的落地写成这些工作全部完成。
