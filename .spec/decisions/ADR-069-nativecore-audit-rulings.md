# ADR-069：NativeCore 审计复核 Owner 裁决——hfsm 为唯一状态机内核、门禁等消费者再加、底层能力开放优先

状态：Draft（2026-09-06；Owner 当日三次口头裁决，本文落成文档；随首个消费者接入验证后转 Accepted）
关联：[`2026-09-06-nativecore-source-audit-reassessment.md`](../reviews/2026-09-06-nativecore-source-audit-reassessment.md)（复核与证据）、[`native-core.md`](../knowledge/features/native-core.md)（能力目录）、[ADR-068](ADR-068-development-verification-follow-main.md)（不堆检测工具、不钉号）、[ADR-067](ADR-067-browser-client-prediction-dotnet-wasm.md)（浏览器无 Native）、[ADR-064](ADR-064-gas-slice-contracts.md)（GAS 八态机）、NativeCore 仓 ADR 0010 / 0011
Owner：`LumioGameEngine`（规则）；NativeCore 与七个消费仓照此执行

## 背景（大白话）

外部审计给 NativeCore 提了 14 条，PR #9 全修了，同时又照审计建议加了一批门（Python 检查器、三系统 CI、编译器钉号），另一个提交加了一个 3,900 行的状态机内核 `lumio-hfsm`，八个仓没有一处用它。三个问题摆到 Owner 面前：hfsm 留不留、门要不要、没人用的模块（job / spatial）怎么办。

一个游戏例子，说清 hfsm 的争议点：

1. 玩家按放弹，技能从「待命」进「施法」——这是一次状态机迁移。
2. 客户端要在浏览器里预测这一步，而浏览器没有 Rust 原生库（ADR-067）。
3. 如果 C# 自己写一套迁移算法，Rust 里再有一套，就是两份实现，「全引擎一套状态机」不成立。
4. 出路只有一条：同一份 Rust 编成 WASM 进浏览器，C# 两端都只持状态快照、算 Guard、执行 Action，迁移算法永远只在 Rust。

## 决策

1. **`lumio-hfsm` 是引擎必备基础框架。引擎内一切状态机（连接 / 加载 / 界面流程 / 技能与效果阶段 / 离散行为）的迁移语义只有这一份实现；C# 与任何上游不得手写第二套迁移算法，只持 Snapshot、算 Guard、执行 Action。** 后果：`lumio-hfsm` 与其唯一依赖 `lumio-kernel` **必须能编成 wasm32**（浏览器预测世界要用）——这是本决策的 kill criterion，接入契约卡第一项就是它；编不成则回到本 ADR 重裁。
2. **接入顺序：先挑第一个真实消费者，再开契约卡**（wasm32 证明 → `native-abi.json` 槽位 → `sdk-native` 插头 → C# facade → 消费者）。候选：Runtime 连接生命周期（[`runtime-manager-controls.md`](../knowledge/features/runtime-manager-controls.md)）、GAS 八态机（随炸弹人 RT 卡）。消费者由 Owner 指定，本 ADR 不定。
3. **NativeCore 现在不算 Release，不加检测工具。** 依据：八个 crate 只有 timer 有消费者，公开 Rust API 仍随第一个消费者改（PR #9 刚破坏式改过一轮并把 SDK 编红）。PR #9 带进的 `tools/check_repository.py` 及其单测并回 Rust `xtask`（同样读 `cargo metadata`，一种语言）；CI 只留一个系统；外部依赖版本交给 `Cargo.lock`，不另维护精确锁表。每个模块接上第一个消费者、API 停下来后再评估加门。
4. **Rust 工具链号只有一个来源：架构仓 `engine/native/rust-toolchain.toml`，值 1.98.0；NativeCore 与 VoxelEngine 的 `rust-toolchain.toml` 同号。** 钉工具链是构建输入（同 `global.json`、`Cargo.lock`），不是 ADR-068 禁止的「钉依赖仓 SHA」；三仓三个号（1.88 / 1.89 / 1.98）今天已让 clippy 红过一次。
5. **job / spatial 等零消费者模块不删、不回退，改为「开放优先」：** 架构仓维护 [`native-core.md`](../knowledge/features/native-core.md) 能力目录（每项能力一个推荐入口、谁在用、怎么到达、现状），上游仓（Voxel / Runtime / Server / Client / Game / Config / Platform）做通用、领域无关的底层能力前必须先查目录，有则用、无则先向架构仓主会话提需求（由主会话给 NativeCore 开契约卡），不得在上游另造。PR #9 留下的死导出（`CompletionBatch` / `BoundedJobQueue` / `CancellationSource` / `JobStateMachine` / `TimedOut` / `DispatchTarget`）登记为 known gap，不开清理卡，目录里不作为推荐入口。
6. **审计里与本仓规则冲突的建议一律不采纳**：多平台 CI 矩阵、「真实消费方固定提交组合」（ADR-068 第 1 条）、「先接真后端再对拍」（第一性原理：spatial 形状本身待 M5 改）。F14「Native 高风险改动不按行数免审」已在 NativeCore `.spec/AGENTS.md` 落一段，架构仓不另抄。

## 替代方案与否决理由

- **冻结或删除 `lumio-hfsm`，等有消费者再说**（主会话推荐）：Owner 否——它是以后所有状态机的地基，现在没接入不等于不需要；代价是必须补 wasm32 证明与接入卡。
- **认定 NativeCore 已是 Release、保留 PR #9 全部门禁**：Owner 否——可用面约 15%、API 未停，加门只会让每次改公开 API 都要同步过三系统 CI 与 Python 检查。
- **删 job / spatial 死导出、退回 rstar**（主会话推荐）：Owner 否——底层库要更开放让上游多用，先解决「上游不知道有什么」，清理等消费者来了按形状一起做。

## 兼容影响

改：`knowledge/features/native-core.md`（新建，能力目录）、`knowledge/features/gas.md`（分工段：八态 / 六态机迁移语义归 `lumio-hfsm`）、`knowledge/standards/development-verification.md`（工具链号单一来源）、`rules/system.md`（新节「底层库」）。NativeCore 仓：`rust-toolchain.toml` → 1.98.0、CI 单系统、xtask 收回检查、hfsm 文档改引 ADR 0011（卡面见 [`2026-09-06-nativecore-w1-cards-and-kickoff.md`](../plans/2026-09-06-nativecore-w1-cards-and-kickoff.md)）。
