---
name: 2026-09-06-rm-00011-r4-independent-review
description: R-00393 cold-start review of the R4 fixture set against the merged R5 main snapshots
metadata:
  type: doc
  status: 已交付
---

# RM-00011 R4 Independent Review

## Verdict

独立冷启动复核已执行。R5 当前主线实现与真实 launcher 证据满足主要运行时、客户端、服务端和浏览器路径；R-00393 的原始 R4 验收口径仍有两项不能标为全绿：

1. Server 独立 acceptance 测试在未设置 `LUMIO_GAME_ROOT` 时按契约返回 `BLOCKED`；同一真实进程场景经最终 launcher 的 Pack A/B oracle 通过。
2. `node .spec/tools/spec-lint.mjs` 仍报告 10 个既有问题（9 个旧 review 悬空链接，1 个根目录 `.sdd`）；本次未删除或篡改这些既有资产。

因此本卡结论为 **部分放行，保留上述两个 known gaps**。本报告不修改任何 ADR 状态，不把旧 R4 的 `Accepted` 结论替代为新的决策。

## Snapshot And Commands

| Repository | Snapshot | Fresh command | Result |
| --- | --- | --- | --- |
| Architecture | `9a22527` | `node eng/verify-wire.mjs` | 7 contracts green |
| Architecture | `9a22527` | `node --test eng/verify-wire.mjs` | 34 passed, 0 failed |
| Runtime | `18b5feb` | `dotnet run --project modules/ecs/tests/Lumio.GameRuntime.Ecs.Tests/Lumio.GameRuntime.Ecs.Tests.csproj --no-build -- --progress off --minimum-expected-tests 1` | 35 passed |
| Runtime | `18b5feb` | `dotnet run --project modules/replication/tests/Lumio.GameRuntime.Replication.Tests/Lumio.GameRuntime.Replication.Tests.csproj --no-build -- --progress off --minimum-expected-tests 1` | 221 passed |
| Server | `.acceptance-server` / `f031afb` | `cargo test --locked -p lumio-server-process --test entity_chat_architecture --test entity_chat_host --test entity_chat_wire` | 48 + 45 + 15 passed |
| Server | `.acceptance-server` / `f031afb` | `cargo test --locked --lib --tests` | 126 unit/process passed; standalone acceptance blocked by missing `LUMIO_GAME_ROOT` |
| Client | `.acceptance-client` / `5745d11` | `dotnet test modules/replica/tests/Lumio.Client.Replica.Tests.csproj --no-restore --nologo` | 70 passed |
| Client | `.acceptance-client` / `5745d11` | `dotnet test modules/session/tests/Lumio.Client.Session.Tests.csproj --no-restore --nologo` | 45 passed |
| Client | `.acceptance-client` / `5745d11` | `dotnet test modules/bot/tests/Lumio.Client.Bot.Tests.csproj --no-restore --nologo` | 18 passed |
| Game | `.acceptance-game` / `d3f6fe0` | `npm test` in `integration/entity-chat` | 87 passed |

## Seventeen Fixtures

| # | Fixture | Command / evidence | Result |
| ---: | --- | --- | --- |
| 1 | ADR-056 dependency direction | `rg` over Server/Client/Game project and source files for Runtime ECS/Replication references | Pass; references are present in all three consumers |
| 2 | ADR-056 no host binding table | Runtime production-source grep for `_liveConnectionByAccount`, `_eventsByRoomTick`, `ChatIngressWorld`, `_values`, `_displayed`, `next-issued-counter` | Pass; 0 hits |
| 3 | ADR-056 generated declarations | `node eng/verify-wire.mjs`; `node --test eng/verify-wire.mjs` | Pass; generated declaration/hash and negative cases green |
| 4 | ADR-056 browser broadcast | `.tmp-acceptance-r5-pack5/verify-report.json` | Pass; Pack A/B `ok=true`, 0 pack failures |
| 5 | ADR-056 persisted snapshot | Pack A/B round evidence, `persist-snapshot.bin`, `process-b/restore-result.json` | Pass in launcher; process B restore observed for both packs and both rounds |
| 6 | ADR-056 timer ownership | Server/Client tests plus bot-host logs in Pack A/B | Pass in launcher; Client Bot.Host timer path observed; no Server startup-hook injection |
| 7 | ADR-056 takeover ordering | Pack A/B reconnect evidence | Pass; `ConnectionSuperseded` observed before old connection retirement |
| 8 | ADR-057 log replay oracle | `node C:/Work/LumioGames/.acceptance-game/integration/entity-chat/verify-evidence.mjs --dir <pack-a|pack-b>` | Pass; both commands exit 0 on the authoritative main snapshot |
| 9 | ADR-057 one ruler | Server three acceptance suites + Game verifier against the same Pack logs | Partial; suites and launcher are green, direct Rust acceptance remains environment-blocked without `LUMIO_GAME_ROOT` |
| 10 | ADR-057 Bot ownership | Server architecture tests and Client Bot tests; grep for `DOTNET_STARTUP_HOOKS` in Server process sources | Pass; 48 architecture tests and 18 Bot tests pass; no injection token |
| 11 | ADR-057 self-driven expiry | Pack evidence `traces.expiry` and `expiry` records | Pass in launcher; expiry/tombstone and stale-generation rejection are observed |
| 12 | ADR-058 single world | Runtime ECS structure tests and production-source banned-token grep | Pass; ECS structure suite included in 35 passed; 0 banned-token hits |
| 13 | ADR-058 generated registry/sync/declaration trio | Runtime ECS/Replication suites and generator output unchanged message | Pass; generator reports inputs unchanged; 35 + 221 tests pass |
| 14 | ADR-058 query reads truth | Runtime `EntityBindingQuery` tests plus Pack query outcomes | Pass; formal query outcomes are present in launcher evidence |
| 15 | ADR-058 deterministic order | Pack A/B formal verifier; 101 client window lines per round | Pass under current canonical event-set verifier; old feature-branch strict verifier is not authoritative |
| 16 | ADR-058 identity restore | Pack `persist-snapshot.bin` and process-B restore evidence | Pass in launcher; same NetEntityId is retained across restore/rebind |
| 17 | ADR-058 client ECS/no history | Client Replica/Session/Bot tests and `node .spec/tools/spec-lint.mjs` | Partial; 70/45/18 client tests pass, but spec-lint has 10 pre-existing documentation/root violations |

## Real Pack Evidence

Evidence root: `.tmp-acceptance-r5-pack5/`.

- Pack A and Pack B each contain two real process rounds.
- Each round observes 101 entities: 100 bots and 1 player.
- Client browser network evidence contains 101 received WorldChange window lines.
- Runtime `InputCommand` / `chat.input` and 101 `OnChatMessage` RPC records are present.
- Query, persist/restore, same-ID rebind, `ConnectionSuperseded`, old-input rejection, and tombstone expiry are all represented in the round evidence.
- The formal verifier from `.acceptance-game` reports `ok=true` for both packs.

## Findings And Follow-up

- Do not mark the two known gaps as passed: set up `LUMIO_GAME_ROOT` and rerun the standalone Rust acceptance, then resolve the 10 pre-existing `spec-lint` findings in their owning documentation assets.
- The current R5 launcher uses the authoritative canonical event-set verifier from Game `d3f6fe0`; no acceptance claim here relies on the older strict-order feature branch.
- ADR-057 / ADR-058 remain unchanged. Any decision to alter their acceptance ruler requires an Owner ADR decision, not a review-side relaxation.
