# ADR-068：开发期验证——依赖仓跟 main 不钉号、红当天修、验证规则只保留一套

状态：Accepted（2026-09-06；Owner 当日两次口头裁决，本文落成文档；不可改写，只能新 ADR 取代）
关联：[`2026-09-06-agent-contracts-ci-review.md`](../reviews/2026-09-06-agent-contracts-ci-review.md)（外部评审与对账）、PR #95（实现）、[`development-verification.md`](../knowledge/standards/development-verification.md)（唯一验证规范）、[ADR-059](ADR-059-lumiocoreengine-repository-retirement.md)（Baseline / 镜像制度废止）
Owner：`LumioGameEngine`（规则）；八个实现仓照此执行

## 背景（大白话）

引擎拆在八个仓里，每个仓的 CI 都要检出别的仓来编。检出「哪个版本」有两种做法：

1. 跟 main：别人今天改了，我今天就对着改后的编；编不过就当天红、当天修。
2. 钉号：我的 CI 只对「上次联测通过的那个版本」编；别人改了我不受影响，等定时任务再追。

一个游戏例子，让问题自己露出来：

1. Runtime 仓改了聊天消息的字段名。
2. Client 仓的 CI 钉着上周的 Runtime，跑出来全绿。
3. 合进 main 后，对着真 Runtime 编不过——这就是 2026-09-05 盘点抓到的 Client 假绿（CI 钉 `7f198e5`，落后 16 个提交）。
4. 钉号越多，绿灯越不可信，Agent 越不知道该信哪个。

2026-09-06 的外部评审建议走做法 2（自动维护的锁文件 + `required-gate`），PR #95 照做了。Owner 否决。

## 决策

1. **五个依赖仓（NativeCore / VoxelEngine / GameRuntime / Server / Client）在 CI 与本地一律跟各自 `main` / 同级目录当前状态。不钉 SHA、不维护锁文件、不做「候选组合晋级」。** 证据文件（`verification.json`）只记录实际用到的每仓 SHA 与 dirty 标志，不反过来约束检出。
2. **红了当天修。** 跨仓改动把下游编红，责任在改动方当天补下游 PR；不允许用钉号、跳过作业、放宽断言、刷新 golden 把灯变绿。
3. **CI 只有三个作业，每个 PR / push 都跑：`tools`（Node 回归 + wire 校验 + 生成物一致性 + spec-lint）、`managed`（Loader 单测）、`integration`（Linux + Windows：真实 Native 装载 + Rust DS / CoreCLR / Runtime / C# Bot 两轮闭环）。** 不做路径影响选择、不设汇总 gate；GitHub 上「skipped 显示 success」的问题因为没有 skipped 而不存在。集成时长若成为痛点，另开 ADR 议路径过滤，不回到钉号。
4. **验证规则只保留一套：`.spec/knowledge/standards/development-verification.md` 是唯一规范。** `AGENTS.md`「收口门槛」、`architecture.md` §6、`engine/wire/README.md` 只引用它，不另写一份口径。此前一切「钉 sha / 合同镜像 / Baseline / 独立守卫脚本」口径一律作废，Agent 不得再据以行事。
5. **三档入口**：`node eng/test.mjs tools`（纯文档 / 单仓改动收口）、`node eng/test.mjs managed`、`node eng/test.mjs all`（碰 ABI / Native / 宿主链）。本地 `dev-run` / `dev-build` 只有 Node 一份实现，Shell / PowerShell 只转发。
6. **采纳的证据修正（PR #95 已实现）**：每次调用都走 Cargo / MSBuild 增量构建，不以旧文件存在为跳过条件；Loader 三方核验（sidecar ABI = 编译期 ABI、二进制 SHA = sidecar、根表 ABI / BuildId = 期望）；Native BuildId 用仓内相对路径 + 工具链 + 目标 + 构建参数，不含绝对路径与 `.spec/` 文档。

## 替代方案与否决理由

- **自动维护的依赖锁文件 + 定时追新**（评审 §8）：把钉号常态化；假绿的来路正是钉号；多一份需要维护的真值。否决。
- **`plan` 影响选择 + `required-gate`**（评审 §9）：为「跳过作业还不假绿」加两个作业和一个策略脚本，前期不堆工具；没有 skipped 就不需要 gate。否决。
- **四类指纹拆分、五层测试、确定性分级**（评审 §5–§7）：作原则记录，进入硬化阶段再议，现在不建卡。

## 兼容影响

改：`knowledge/standards/development-verification.md`（PR #95 建、按本 ADR 返工）、`.spec/AGENTS.md`「收口门槛」、`knowledge/features/architecture.md` §6、`engine/wire/README.md`（改为引用唯一规范）、`.github/workflows/repository-policy.yml`（三作业）。删：`eng/workspace-lock.json`。
