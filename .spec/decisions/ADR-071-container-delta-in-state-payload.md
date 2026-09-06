# ADR-071：SyncList / SyncDict 容器条目差量线上编码与状态载荷承载（不走事件通道）

状态：Draft（2026-09-06；关联 R-00512 / PR35-A；契约文件 `engine/wire/container-delta-v1.json`，随 PR35 闭环后转 Accepted）
关联：[`engine/wire/container-delta-v1.json`](../../engine/wire/container-delta-v1.json)、[ADR-049](ADR-049-replication-state-payload-and-input-command.md)（状态载荷）、[ADR-058](ADR-058-ecs-world-manager-and-annotation-registry.md)（Sync<T> 与容器）、[ADR-060](ADR-060-rm00011-r5-owner-rulings-pack-wire-and-observer-projection.md)（C-1 WorldChange）、[ADR-064](ADR-064-gas-slice-contracts.md)（GAS 切片契约与表现缓冲 ClientRpc 记录）
Owner：`LumioGameEngine`（规则）；Runtime 与各端照此消费

## 背景

在 ECS 属性同步设计（`ecs.md` M4）与 GAS 战斗切片（`gas.md`）中，均明确「容器条目差量是一等公民，改动 100 格背包的第 5 格，线上字节只包含那一格」。然而此前 `engine/wire/` 仅有标量字段的同步描述，缺乏容器条目级变更的 wire 规范。这导致 Runtime 在处理 `SyncList` / `SyncDict` 时只能回退为整表复制，或产生私造临时格式的风险。

同时，关于「容器条目变化是作为状态载荷（State Payload）同步，还是作为网络事件（ClientRpc / Event）下发」，需要从第一性原理做清晰决策。

## 决策

1. **容器条目差量归属状态载荷（State Payload），严格禁止走事件通道（ClientRpc / Event）。**
   - **理由**：
     - 容器内容（如玩家背包、装备栏、GAS 技能/效果列表）是**权威实体状态**，必须具备最终一致性、状态快照与断线恢复能力；
     - 事件（`[ClientRpc]`）是一次性通知，服务器不存不查不回放（ADR-058 / ADR-060）；若将条目差量作为事件广播，重连或中途进入视野的客户端将因缺少事件流水而无法重建容器状态；
     - 状态载荷内的差量与全量条目集享有同一本版本账与同一事务生效保证。
2. **线上编码契约落于 `engine/wire/container-delta-v1.json`（contractId: `lumio.container-delta.v1`）。**
   - **列表（SyncList）**：支持按下标的操作集 `set(index, value)`、`insert(index, value)`、`remove(index)`、`clear()`，以及全量状态 `fullEntries`；
   - **字典（SyncDict）**：支持按键的操作集 `set(key, value)`、`remove(key)`、`clear()`，以及全量状态 `fullEntries`；字典键使用一等标量/string 编码；
   - **同帧折叠（Same-frame Folding）**：同帧对同一位置/键的多次修改必须在写时记账折叠为至多一次最终操作；加了又删抵消，不发无意义冗余操作；
   - **每容器尺寸上限（Bounded Capacity）**：每个容器定义与载荷必须携带 `maxCapacity`（u32），超出上限拒绝；
   - **初次/重进视野/重连只发全量条目集**：不回放历史增量流水，直接下发当前最新 `fullEntries`；
   - **与标量同一事务生效**：容器差量在接收端的 staging 提交相与标量字段原子生效，随后触发条目级回调（`ListChange` / `DictChange`）。

## 替代方案与否决理由

- **方案 A：将容器操作作为专用 RPC 事件广播（如 `OnItemAdded` / `OnItemRemoved`）**
  - **否决**：违背「字段 = 最后状态（可存可查可同步），事件 = 一次性通知（不存不查不回放）」原则。事件通道没有状态持久与视野投影基线，会导致进出视野、断线重连时状态断裂，必须依赖外部机制弥补，增加两套系统漂移风险。
- **方案 B：无论大小一律发送容器完整快照（Full Snapshot）**
  - **否决**：违背 M4 带宽控制原则。大容量列表（如上百格背包、复杂字典）在单条目高频修改时将产生巨大带宽开销。

## 兼容影响

- 新增公共契约：[`engine/wire/container-delta-v1.json`](../../engine/wire/container-delta-v1.json)。
- 索引更新：`engine/wire/README.md` 注册该文件。
- 文档更新：`.spec/knowledge/features/ecs.md` §5 阶段 0 卡 0-4 回指此契约与本 ADR。
