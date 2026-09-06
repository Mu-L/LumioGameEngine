# ADR-067：浏览器客户端预测走 .NET WebAssembly 装载 Runtime 客户端模块（桌面浏览器）；手机浏览器待真机数据

状态：Draft（2026-09-06；Owner 已裁 D3「暂定方案 A」与 D27「采纳 CL-1 结论」，本文把两条裁决落成文档；实现随 R-00470 验证，验证通过后转 Accepted）
关联需求：R-00470（C8，浏览器最小消费宿主 = 本路线落地点）、R-00466（RT-3，追加 `PublishTrimmed` 零警告验收项）、R-00408（Client 部分：删第二份 codec）、R-00481（Client 清理）
来源：LumioClient CL-1 调研（PR #18，`docs/spikes/2026-09-05-spike-runtime-wasm.md` §5.3 ADR 草案建议）；架构仓 `.spec/reviews/2026-09-05-engine-repos-progress-assessment.md` §6 D3 / D27
Owner：`LumioGameEngine`（路线与失败语义）、`LumioClient`（浏览器宿主）、`LumioGameRuntime`（客户端程序集可裁剪）

## 背景（大白话）

老王在网页里按右键，人要立刻动，不能等服务器一个来回。要做到这一点，网页里必须有一份「预测世界」在本地先算一步。这份预测世界的规则代码今天只有一份：Runtime 的 C#（ECS、`WireCodec`、预测世界重建）。摆在面前的三条路：

1. **A**：把 Runtime 的客户端程序集原样编成 .NET WebAssembly，装进网页跑。
2. **B**：网页只画不预测，每次按键等服务器回包。
3. **C**：再用 JS / TS 写一份预测代码。

CL-1 用桌面 Chrome 实测（数字全在调研报告 §4）：A 能编、能跑、能对话，浏览器与桌面算出的世界哈希逐位相同；启动 0.6–5.6 s、AOT 后每包重建中位 0.5–1.7 ms；B 每次按键要等 300 ms 以上（150 ms 单向链路）；C 违反「同一件事只有一份代码」。手机一台真机没测。

## 决策

1. **浏览器客户端的规则代码 = Runtime 客户端程序集经 .NET browser-wasm 装载。** 不得另写 JS / TS 版 ECS、codec 或预测；JS 只做三件事：WebSocket 搬字节、Canvas / DOM 表现、输入采集。出现第二份浏览器专用 codec 或预测实现 → 审查退回。
2. **预测世界重建（RT-3 / R-00466）在浏览器与 C# 客户端是同一份代码。** 双端对账哈希（ADR-064 第 10 条）在浏览器同样成立；回归用 CL-1「桌面 vs wasm 逐位一致」三档快照（100 / 300 / 1000 实体）。
3. **交付形态**：publish 必须开裁剪与 AOT；`InvariantGlobalization` 默认开（Runtime 不依赖 ICU 语义，待 Runtime 在 R-00466 确认）；**单线程运行时**，不开 `WasmEnableThreads`（JS 互操作只在主线程，开了也不能把 `WorldManager` 挪出主线程），因此落地站点**不需要** COOP / COEP；整个运行时可整体挪进 Web Worker（CL-1 A1-5 实测成立），主线程只收 `postMessage`。
4. **站点**：静态托管 + `application/wasm` MIME + brotli 预压缩协商；wss 与落地站点沿用 LumioClient decisions/0003 与 LumioPlatform launch 端口（R-00415）。
5. **Runtime 义务**（落 R-00466 验收项）：修掉 Ecs 里 4 处 IL2075 反射（`World.TryReadAccountId`、`WorldManager.FindSyncField` ×2、`WorldManager.TryDispatchSendMessage`）或给生成器加 `DynamicallyAccessedMembers` 标注，使 `PublishTrimmed` 零警告；不得在客户端路径上 `new Thread` / `Task.Run`（Observability 的 `EventDispatcherWorker` 若进浏览器要改 owner-thread 泵）。InputCommand 信封编码 API 已由 `WireCodec.EncodeInput`（R-00407）提供，缺口已闭合。
6. **宿主义务**（D24，R-00408）：Rust 宿主跨平台加载器一份，Linux + Windows 双绿；浏览器验收用真实 Rust 宿主，不再用 C# 替身。
7. **落地卡**：不另开「WASM 客户端壳」卡。R-00470 AC06（浏览器通过正式样本端口真实显示移动 / 纠偏 / 销毁）就是本路线的验收；R-00470 开工前先定三个预算：桌面 Chrome 空缓存到「WorldManager 可用」上限（建议 ≤ 3 s，AOT + 去 ICU 后复测）、AOT 构建时间与 brotli 体积上限、Runtime 裁剪零警告。
8. **手机浏览器不下结论**：iPhone（iOS Safari 近两代）与中端 Android Chrome 各一台真机跑完 CL-1 §4.B 的 B1–B4 再定；此前产品承诺沿用 LumioGame ADR 0013「桌面浏览器优先，触屏浏览器不承诺」。

## 替代方案与否决理由

- **B（只画不预测）**：按键必等往返，体验差是结构性的；且浏览器仍要一份 codec 才能上行，要么写第二份、要么就是 A 的子集。
- **C（第二份 JS / TS 实现）**：违反第一性原理与 ADR-058「一处维护」；每次 Runtime 改动同步两份，哈希对账跨语言。
- **Blazor**：同样是 .NET WASM，多一层 UI 框架，未评估，不进入。
- **Unity / HybridCLR 客户端**：LumioGame ADR 0013 已降为后续候选；LumioClient 的空壳按 D31 删除（R-00481）。

## 失败语义（回图触发）

1. 桌面 Chrome 空缓存到「WorldManager 可用」超过预算 → 先 AOT + `InvariantGlobalization` 复测，仍超 → 回图。
2. AOT 后 100 实体重建中位仍 > 50 ms 帧预算的 20 % → 先按 `gas.md` kill criterion 2 收窄克隆域，再回图。
3. 真机 iOS Safari 装不下（内存拒绝 / 下载超 30 s）→ 手机走 B 或不承诺，不影响桌面路线。
4. 任何仓出现第二份浏览器专用 codec / 预测 → 审查退回，不接受「先兼容」。

## 后果

- `gas.md` M7 与 `ecs.md` M10 各补一句「浏览器载体 = 同一份 Runtime 客户端程序集经 wasm 装载」（随 R-00470 交付时同步）。
- 与 D3 一致：进预测世界的东西必须能在浏览器里跑，Native 只管宿主节拍与重计算；以后重计算内核进网页走「同一份 Rust 编成 WASM」，仍是一套。
- CL-1 探针 `spikes/runtime-wasm/` 保留为回归基准来源（`RebuildBench` / `WorldHash`），不进生产 `modules/`。
