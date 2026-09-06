---
name: 2026-09-06-voxel-closeout-batch-report
description: 体素收口批次的执行记录——五件授权事项的落地、V-QA 93 条验收跑批、四次退回与两次停卡、以及留给后续的 known gaps；复盘本批次或接手遗留卡前查
metadata:
  type: doc
  status: 已交付
---

# 体素收口批次 · 收口报告（2026-09-06）

Owner 2026-09-06 授权的五件事全部执行完毕。本报告记录改动清单、每卡验证证据、known gaps 与沉淀落点。

**总量**：21 张卡流转「已完成」，**139 条验收项通过 / 2 条阻塞 / 0 条不通过**；两仓合计合入 12 个 PR。批次中途新开 7 张卡（2 张为解阻塞、5 张为遗留项归档）。

---

## 一、五件授权事项的落地

| # | 事项 | 结果 |
|---|---|---|
| ① | R-00479 契约三处缺陷修订 | ✅ 7/7 通过，已完成。架构仓 `66c8d31` + VoxelEngine `da43343` |
| ② | V-QA 93 条验收项逐条实跑 | ✅ 88 通过 / 5 不通过 / 0 阻塞；5 条不通过全部修复并收口 |
| ③ | R-00474 退出旧合同制三层清理 | ✅ 10/10 通过，已完成。VoxelEngine `f7d28f3`，净删约 17,800 行 |
| ④ | R-00476 sdk-native 去 V1.4 名字 | ⚠️ 3 通过 / 2 阻塞（均为宿主依赖，见 §四），已合入 `dd48fda` |
| ⑤ | R-00477 根表物理三槽路由 | ⏸ 见 §三「两次停卡」 |

### 中途新开的 7 张卡

| 卡 | 为什么开 |
|---|---|
| R-00486 | R-00477 停卡回报的三处契约硬缺口，Owner 指示回唯一真值处补全。**已完成 7/7** |
| R-00487 | R-00448 验收 5 改判为静态守卫落地。**已完成 5/5** |
| R-00488 ~ R-00492 | 五张遗留项归档卡，backlog 未派活，见 §五 |

---

## 二、改动清单（按合入顺序）

### LumioGameEngine（架构仓）

| 提交 | 卡 | 内容 |
|---|---|---|
| `66c8d31` | R-00479 | 契约 `errorCodes` 52 → 54（只追加）、rules 57 → 60、invalidCases 57 → 62；`residency.pinnedRegions.budget.residentSectionBudget` 新字段；`verify-wire` 新增 voxel 专属断言；ABI 与三种绑定随源重生成 |
| `413c6b0` | — | 修 mvp-host 归档后的两处悬空跨仓链接（解除 guard-commit 全会话封锁，见 §六） |
| `ca44c06` | R-00445 / R-00456 | 补齐 1052/1053 两处手写错误码镜像 + 两侧穷尽性断言 |
| `f5202cf` | — | `docs/plans` 与 `docs/reviews` 并回 `.spec/`，恢复单一文档根 |
| `d011e79` | — | `voxel.md` M8 驻留预算改用契约字段 `residentSectionBudget = 65536`（量纲歧义定案） |
| `dd48fda` | R-00476 | `sdk-native/voxel.rs` 只用活契约面类型；`approved_snapshot()` 删死基线三件套 |
| `b947e8f` | — | `spec-lint` 不再把宿主托管的 worktree 扫成第二套框架（见 §六） |
| `23401e1` | R-00486 | `physicsQuery` 补三处声明：形状按值内联、材质掩码位分配、材质类表 Native 入口 |

### LumioVoxelEngine

| 提交 | 卡 | 内容 |
|---|---|---|
| `da43343` | R-00479 ⑦ | 契约副本同步至 `d05dbc52…` |
| `f7d28f3` | R-00474 | 退出旧合同制三层：删镜像 / CI 校对 / 空壳 crate（一层）、删 `generated/` 树与 `legacy_baseline`（二层）、活代码清死基线符号（三层）+ CI 护栏 |
| `6d386a6` | R-00436 / R-00452 | 全量编码与过期回执按契约报自己的错码；CI 护栏加固 |
| `d5efaf5` | R-00487 | 碰撞行为硬编码的静态守卫 |
| `0f41e8b` | R-00486 ⑦ | 契约副本同步至 `523aec6e…` |

---

## 三、四次退回与两次停卡

本批次最有价值的产出不是改了多少行，而是**证伪机制发挥了作用**。

### 退回 1 · R-00474 深审：验收 4 的 grep 是空转门

卡面判据写作 `git grep … -- 'crates/*/src'`。git pathspec 的 `*` 跨 `/` 匹配，该式要求路径**以 `src` 结尾**，实测恒命中 0 个文件：

```
git grep -l 'fn ' -- 'crates/*/src'    → 0 个文件
git grep -l 'fn ' -- 'crates/*/src/*'  → 84 个文件
```

前一轮交回物用「命中 0 文件」证明「活代码清干净了」——**那条证据是空的**，而且日后旧符号回流也没有门抓得到。已订正判据（含 `SCHEMA_EPOCH` 加词边界以免误伤裁决 3 允许的自持常量），并落进 CI 作护栏，护栏**内建自检**：pathspec 匹配不到任何源文件时脚本自己红。

### 退回 2 · R-00474 深审：四处测试删断言后不再约束错误 id

审查方逐个扫完基线全部 99 个 `assert_stable_error` 位点，证实实现方「每处紧邻一行都有 `assert_eq!`」的说法**大多数成立、但有 4 处被证伪**。修复时 id 不是猜的——先插 `__PROBEn__` 断言跑出 panic 读实际值再换真值。

### 停卡 1 · R-00477：三处契约硬缺口，零代码改动

实现方在动手前的契约核对中发现 `shape` / `pose` 是无目标布局的裸指针（契约里 `pose` 零命中）、`material_mask` 无位分配、材质类表无 Native 入口，且 `generate-abi.mjs:98` 硬比对 wire SHA 意味着任何补声明都必然重生成 `native-abi.json`、与本卡「逐字节不变」的硬约束正面冲突。**它按纪律停下回报，一行实现代码没写。** Owner 指示不兜底不绕过，遂开前置卡 R-00486 补全。

### 停卡 2 · R-00448 验收 5：拒绝造假检测点

`collision_behavior_not_from_material_table` 全仓只有一处错误码清单字符串，无构造器、无 raise site。实现方与审查方**独立得出同一结论**：该 invalidCase 的 `given` 是一段源码而不是触发载荷（契约自身标了 `validatorCheck: false`），不存在可构造的入参；能造的两种检测点都是坏的（贴错信号 / 死构造器换马甲）。Owner 改判为静态守卫落地（R-00487）。

### 一次自我纠错 · R-00479 深审的 P2 定级错了

R-00479 追加两个错误码后，两处手写镜像（`voxel.rs::status_for_error` 52 个 match 臂、`VoxelFacade.cs` 的 `VoxelError` 52 个枚举）未同步。深审记为 P2「今天零影响，无 producer 产出这两个字符串」——**该定级有误**：它只查了产出方，没查消费方的穷尽性，而 V-QA 随即证明它打挂了 R-00445 验收 3 与 R-00456 验收 5。已修复并在两侧各加穷尽性断言（以 `native-abi.json` 的 `voxel.errorCodes` 为唯一真值），下次契约追加错误码而漏改镜像必然变红。

---

## 四、验证证据摘要

### 收口门槛（两仓最终状态）

```
架构仓 origin/main = 23401e1
  node eng/generate-abi.mjs（连跑两次）→ DEFINITION_SHA256=fd76885a…，第二次后 git status 全空（幂等）
  node eng/verify-wire.mjs             → 7 份契约全绿，voxel 115/115 clean passes
  node --test eng/generate-abi.test.mjs → 19/19
  node .spec/tools/spec-lint.mjs       → OK
  cargo build/test -p lumio-engine-native → 3 + 14 passed
  dotnet test …NativeLoader.Tests      → Passed! Failed: 0, Passed: 20

LumioVoxelEngine origin/main = 0f41e8b
  cargo fmt / clippy -D warnings / check --no-default-features → 全 exit 0
  cargo test --workspace --all-features → 390 passed, 0 failed
  check-crate-dag → OK: 6 crates
  spec-lint + spec-lint.test → OK / 13 pass
```

契约 SHA 演进：`56d555fd…`（批次开始）→ `d05dbc52…`（R-00479）→ `523aec6e…`（R-00486）。两仓副本每一步都经 `cmp` 逐字节核对。

### 反例探针总账

本批次的一条硬性要求是「绿门证明不了门在」。各次审查合计执行反例探针 **80+ 条**，且每次都先做**控制探针**（原样重序列化 / 只插注释空行）排除「碰一下就红」的假阳性：

| 审查 | 探针 | 关键发现 |
|---|---|---|
| R-00479 深审 | 23 条全红 | 改名新码并同步改 `sourceSha256` 绕过哈希钉后，错误码映射门**仍然变红** |
| R-00479 ⑦ 快审 | 5 条 | 验了 `LUMIO_ENGINE_WIRE_DIR` 指向不存在目录时输出**可区分**，证明比对不是静默跳过 |
| V-QA | 7 条 | 实证 `verify-wire` 是软门：`cellOffset` stride 改坏它照报全绿 |
| R-00474 重审 | 双向 + 4 道自检 | 用 `yaml.safe_load` 程序化抽出 CI 脚本原文执行；正则换成非法 `'['` → exit 128 失败关闭 |
| R-00452 深审 | 双向 | golden 哈希双向探针**都红在两腿比对而非哈希断言**——假独立会让前者恒绿 |
| R-00487 快审 | 15 组 + 4 道自检破坏 | 十一种自然拼法全红；唯一真漏是刻意规避 |
| R-00486 深审 | 9 条 + 控制 | 用**真实编译器** `cc` + `offsetof` 复核结构体布局，另用独立实现重算全部结构体 mismatches: 0 |

---

## 五、Known gaps

### 阻塞项（2 条，均为宿主依赖，非缺陷）

`R-00476` 验收 2 的 clippy 部分与验收 4 的 dev-run 部分。本机 macOS：

- `cargo clippy -- -D warnings` 因 `clr-host/src/sys.rs:20` 的常量唯一使用点在 `#[cfg(windows)]` 内、非 Windows 宿主必然 dead code 而 exit 101。经三重证伪与本卡无关（diff 单文件、该文件 blob sha 与基线同一 git 对象、单独跑 `-p lumio-clr-host` 同样 exit 101）；**sdk-native 自身 clippy 诊断条数 = 0**。
- `eng/dev-run.ps1` 需 PowerShell/Windows、`dev-run.sh` 只支持 Linux。`SERVER_READY` / `ENGINE_NATIVE` 未验证。

**两条都需要在 Linux / Windows 宿主复跑一次才算收口门槛完整过。** 全批次涉及 dev-run 的证据一律标 blocked，未有一处写成 passed。

### 已归档为卡的遗留项（5 张，backlog 未派活）

| 卡 | 优先级 | 仓 | 内容 |
|---|---|---|---|
| R-00488 | P2 | 架构仓 | Rust 侧解析 `native-abi.json` 用全局 `find` 未先定位 `voxel` 段（**将来别的子系统引入同名键会静默读错数组**）；缺反方向（多余臂）检查 |
| R-00489 | P2 | 架构仓 | `verify-wire` 只校声明不校语义；`unknown_material_class` 之外唯一没有 invalidCase 的 `unregistered_block_type` |
| **R-00490** | **P1** | VoxelEngine | **材质类表是碰撞的唯一真值**：`MaterialProfile::collision()` 零生产调用点、`MaterialClass` 枚举定义两处（违反契约 `materialClasses.singleTable`）、四个内置哨兵只认了 air |
| **R-00491** | **P1** | 架构仓 | **架构仓自己还留着 `LGE-V1.4` 生成树**与内嵌 schema 副本；R-00474 在 VoxelEngine 清完了，这边没清 |
| R-00492 | P2 | VoxelEngine | 守卫扫描面语义锚点；差分 `partial` 探针误名；活代码 37 处 `generated` 措辞 |

### 未归档、待 Owner 裁决

- **`degenerateShape` 只禁不立**（R-00486 引入）：契约同时说「由调用方保证」（前置条件）与「体素侧不得静默当成 Miss」（被调方必须检测），两句互相拉扯，而 54 条 `errorCodes` 里没有对应退化形状的码。这正是本批次要根治的失败模式在新声明里重演了一次。
- **三条新语义是纯声明**（`zeroMatchesNothing` / `reservedBitsMustBeZero` / `degenerateShape`），无 `rules` + `invalidCase` 支撑，R-00479 立的那道机器闸门覆盖不到它们。

---

## 六、guard-commit 三次封锁全会话（工程教训）

本批次被 `guard-commit` 钩子拦死提交三次。钩子按 `CLAUDE_PROJECT_DIR` 跑 spec-lint，所以**架构仓主检出一红，同会话所有仓的 `git commit` 全被拦**。三次病灶各不相同：

1. **`.workflow-drafts/` 残留**——另一会话已完成的上传回执（146 操作全 verified）。原样挪到 `~/LumioGames/.workflow-drafts-parked/`，未删。
2. **跨仓悬空链接**——LumioServer 的 R-00478 归档 C# mvp-host 删掉整个目录，架构仓一份**已提交**文档的 `../../../LumioServer/…` 链接随之悬空。修法是降级为 code span 并注明去向（PR #87）。
3. **`spec-lint` 自身的误报**——`.claude/worktrees/<name>/` 是宿主建的**本仓完整检出**，各自带一份 `.spec/` 与 `docs/`，被扫成「第二套框架」「并行文档根」。**任何用原生 worktree 的会话都会踩。** 已修（PR #92）：按路径精确排除该目录，且刻意不整体跳过 `.claude/`、不笼统跳过 gitignore 目录（`.workflow-drafts/` 同样被 gitignore，而它正是这条检查要抓的）。双向探针验证修前误报 2 条、修后 OK，同时深层 `engine/native/docs` 与 `.workflow-drafts` 仍被抓。

**教训**：跨仓相对链接在多仓并行清理时极易断，而它会经钩子放大成全会话封锁。`.spec/reviews/2026-09-05-dual-transform-bomber-research-gap-audit.md` 一份文档就有 37 条 `../../../<repo>/…` 链接，指向 7 个仓——每个仓的清理都可能打断它。

---

## 七、沉淀落点

| 落点 | 内容 |
|---|---|
| `ADR-062` | 两段修订记录：R-00479（三处缺陷）、R-00486（三处声明缺口） |
| VoxelEngine `ADR 0014` | 退出旧合同制；`0007` / `0009` / `0010` 标被取代，**`0013` 保持生效**（0014 是它的延续，不是推翻——Owner 裁决），`0006` 只加「crate 清单被 0014 部分取代」状态行 |
| `voxel.md` | M5 / M6 / M8 随 R-00479 更新；M8 驻留预算量纲定案（`residentSectionBudget = 65536` Section = 4096 Chunk）；M7 随 R-00486 新增 ③a / ⑤a / ⑤b 三段与三条「做完的标准」 |
| `2026-09-04-voxel-card-contract-drift.md` §六 | 加 2026-09-06 更新块，逐条标注第 1–4 条已修 |
| CI 护栏 | VoxelEngine 新增两个独立 job：`legacy-contract-symbols`（ADR 0014）、`collision-from-material-table`（契约 rule `query.collision-comes-from-material-table`）。两者都自带自检，pathspec 失效或判据在阳性样本上零命中时脚本自己红 |

**建议但未做**（交 Owner 定）：`git pathspec 的 * 跨 /，'dir/*/src' 恒零命中` 这条坑值得进 `.spec/knowledge/lessons.md`——它让一条验收项和一轮交回物证据同时失效，且是任何仓都会踩的通用坑。同理「差分预言机是同一契约的独立第二实现，改 golden 哈希前必须先过双向探针」也值得固化。

---

## 八、纪律执行情况

- **无一次 `--no-verify`、无一次 force push、无一次直接推 main。** 三次被钩子拦死时都是停下修病灶，没有绕过。
- **实现与审查全程隔离**：每张卡的 worker 与 reviewer 在各自独立的 git worktree，全部直接放在 `~/LumioGames/` 下（放深一层会让架构仓的 `../../../../../` 路径依赖解析不到同级仓）。
- **子代理的成功报告一律不作数**：每次合入前主 loop 独立复核 diff 与关键断言；多处审查发现实现方声称与事实有出入（`GeneratedVoxelConfig` 被误归为「纯改名」实则删了两个字段、守卫注释误读契约、`\b` 缓解在 snake_case 上不成立），均已订正。
- **不把 blocked 写成 passed**：全批次 2 条阻塞项如实标注并写明需要什么宿主。
