# ベンチマーク

状態: SSOT 統合版
日付: 2026-06-28 JST

## 目的

本章は arcRTC v0.2 Kernel の benchmark scope と scenario matrix を規定します。benchmark は performance / capacity / resource behavior の測定であり、correctness、security、runtime readiness、production readiness を単独では証明しません。本章は、測定対象（benchmark surface）と非対象、acceptance threshold を claim しない点、scenario matrix の全シナリオと測定項目、report が evidence として使用可能であるために記録すべき field、command class の境界を固定します。

本章で規範語として用いる語の意味は次のとおりです。**必須**＝満たさなければならない条件、**禁止**＝行ってはならない事項、**許可**＝行ってよい事項、**fail-closed**＝必須条件が満たせない場合に失敗側へ倒すことです。benchmark output は、後述の必須 field を欠く場合は diagnostic output のみとして扱います。

## Benchmark surface（測定対象）

v0.2 initial benchmark surface は次に限定します。新 benchmark surface の追加は ADR または設計判断が必須です。

| Surface | Example metric | Related boundary |
|---|---|---|
| core decision latency | Signaling/SFU/TURN decision duration | core state machine / contract |
| driver conversion cost | decode/encode/conversion time | driver conversion |
| packet path cost | packet semantic view creation, fan-out command count | SFU packet lifecycle |
| packet rewrite / transform cost | header rewrite, copy allowance, transform admission, driver execution cost | packet rewrite / media transform |
| resource bound behavior | queue/cache/backlog saturation behavior | resource bounds |
| service discovery cost | endpoint resolution/cache/fallback behavior | service discovery / endpoint resolution |
| distributed failover diagnostic cost | owner lookup, affinity check, failover evidence preparation | distributed state / failover |
| runtime task lifecycle cost | spawn, queue/mailbox, join/cancel, supervision observation | runtime task / worker lifecycle |
| internal service trust cost | peer proof mapping, credential verification, trust policy lookup | internal service identity / trust |
| cross-plane binding cost | binding validation, lifecycle relation, reference mapping | cross-plane identity / session binding |
| persistence/export cost | bounded retry/export behavior | persistence/observability |
| SDK signaling latency | public client command/response latency | SDK Signaling-only |

benchmark scope は non-goals、environment fields、workload fields、adoption fields を declare します。measurement evidence は command、working directory、toolchain、workload、result unit を含むことが必須です。benchmark measurement evidence は correctness proof ではありません。

## 必須 report field（scope 側必須 field）

benchmark result を採用できるのは、record が次を含む場合に限ります。

- correlation ID
- command
- working directory
- code revision or source snapshot identifier when available
- hardware / OS / runtime/toolchain context
- benchmark input size and scenario
- fixture/scenario class and source when benchmark data is generated or fixture-backed
- warmup rule
- sample count
- measured metric and unit
- measurement window, precision, and aggregation method
- pass/fail threshold if a threshold is claimed
- rerun condition

これらの field を欠く benchmark output は diagnostic output のみです。

## Command Class Boundary

benchmark command class は実行前に固定することが必須です。

workspace-wide benchmark command:

- `cargo bench --workspace`

Criterion target-specific observation command:

- `cargo bench -p arcrtc-roadmap-tests --bench benchmark_scenarios_criterion -- --noplot`

v0.1.2 ice-pool shape comparison target:

- `cargo bench -p arcrtc-roadmap-tests --bench v01_ice_shape_criterion -- --noplot`

この target は benchmark function name、Criterion group name、input size を v0.1.2 の `core/benches/ice_pool_bench.rs` と整合させます。これは v0.2 test-side comparison surface のみであり、production performance acceptance ではなく、production readiness でも live readiness でも native command success でもありません。comparable workload は、equivalent production ICE candidate pool を v0.2 core semantics に admit する明示的 architecture decision がない限り、testing crate 配下に留めることが必須です。

`--noplot` は Criterion option です。concrete Criterion bench target にのみ渡すことが必須です。`cargo bench --workspace -- --noplot` を通じて渡してはなりません。workspace bench execution は非 Criterion libtest binary も実行し、それらは Criterion-only option を reject するためです。この誤りが生じた場合、その failed command は benchmark evidence ではありません。是正は、workspace benchmark を Criterion-only option なしで rerun し、plot suppression が必要なら各 Criterion bench target を明示的に rerun することです。failed command、failure reason、corrected command、corrected result は、benchmark comparison claim の前に記録することが必須です。

## Non-Goal Rule

initial v0.2 architecture は production performance target を設定しません。benchmark documentation は measurement method と evidence requirement を定義してよいですが、明示的 acceptance criterion なしに production readiness threshold を発明してはなりません。

performance acceptance を admit する場合、production-completion claim の前に threshold decision が存在することが必須です。admit しない場合、benchmark threshold は unclaimed のままで、benchmark evidence は measurement / scenario coverage evidence のみです。

## v0.1 Benchmark Rule

v0.1 benchmark code と results は historical evidence-only input です。scenario selection の参考にしてよいですが、rerun または explicit requalification なしに v0.2 benchmark evidence として再利用してはなりません。

## Correctness Boundary

benchmark passing は correctness を証明しません。correctness は対象 scope の tests で証明することが必須です。benchmark failure は implementation risk を露呈し得ますが、それ単独で core semantics を再定義しません。

## Unit and Fixture Boundary

benchmark measurements は normalized unit を報告し、有用なら raw source class を報告することが必須です。generated packet fixtures、synthetic sessions、captured-redacted samples、v0.1-derived scenarios は label することが必須で、correctness evidence として採用できません。

## Scenario matrix（全シナリオと測定項目）

scenario matrix は production threshold を定義せず、benchmark scenario がどの境界を測るか、どの条件で evidence として採用できるかを固定します。新 scenario class の追加は ADR または設計判断が必須です。

| Scenario | Measures | Boundary |
|---|---|---|
| core signaling decision latency | join/leave/relay decision duration | Signaling state machine |
| core SFU routing decision latency | route selection/suppression duration | SFU contract/state |
| packet semantic view cost | view creation and validation cost | packet semantic view/lifecycle |
| packet rewrite / transform cost | rewrite intent mapping, copy allowance, driver execution path | packet rewrite / media transform |
| driver conversion cost | external wire to core envelope conversion | driver conversion / wire envelope |
| service discovery resolution cost | resolver lookup, cache/staleness check, fallback rejection | service discovery / endpoint resolution |
| distributed owner/failover diagnostic cost | affinity lookup, owner conflict check, failover evidence preparation | distributed state / failover |
| runtime task lifecycle cost | spawn/schedule/join/cancel and worker supervision observation | runtime task / worker lifecycle |
| internal service trust cost | peer proof mapping, credential verification, scope/trust policy decision | internal service identity / trust |
| cross-plane binding cost | source/target reference mapping, lifecycle validation, binding decision | cross-plane identity / session binding |
| TURN wire decode/encode cost | STUN/TURN mapping overhead | TURN wire driver |
| resource bound saturation | queue/cache/backlog behavior | resource bounds |
| audit hash-chain append/verify cost | record canonicalization and hash cost | audit hash-chain |
| canonical serialization cost | deterministic field ordering, normalization, and digest input construction | canonical serialization |
| persistence/export cost | bounded store/export/retry behavior | persistence/observability |
| SDK signaling roundtrip | public client command/response latency | SDK parity/signaling-only |

scenario matrix は Signaling、SFU、TURN、cross-plane、SDK、driver overhead の各 class を covers します。measurement evidence は scenario identity と workload を明示し続けることが必須です。

### scenario 採用集合（全 16 件）

scenario matrix の各 scenario class を measured または closed reason 付き non-adopted として明示することが必須です。scenario の省略は失敗です。

| Scenario ID | Scenario class | Required benchmark test |
|---|---|---|
| BENCH-001 | core signaling decision latency | measured or closed non-adopted |
| BENCH-002 | core SFU routing decision latency | measured or closed non-adopted |
| BENCH-003 | packet semantic view cost | measured or closed non-adopted |
| BENCH-004 | packet rewrite / transform cost | measured or closed non-adopted |
| BENCH-005 | driver conversion cost | measured or closed non-adopted |
| BENCH-006 | service discovery resolution cost | measured or closed non-adopted |
| BENCH-007 | distributed owner/failover diagnostic cost | measured or closed non-adopted |
| BENCH-008 | runtime task lifecycle cost | measured or closed non-adopted |
| BENCH-009 | internal service trust cost | measured or closed non-adopted |
| BENCH-010 | cross-plane binding cost | measured or closed non-adopted |
| BENCH-011 | TURN wire decode/encode cost | measured or closed non-adopted |
| BENCH-012 | resource bound saturation | measured or closed non-adopted |
| BENCH-013 | audit hash-chain append/verify cost | measured or closed non-adopted |
| BENCH-014 | canonical serialization cost | measured or closed non-adopted |
| BENCH-015 | persistence/export cost | measured or closed non-adopted |
| BENCH-016 | SDK signaling roundtrip | measured or closed non-adopted |

## 必須 report field（scenario matrix 側必須 field）

benchmark evidence report は次を記録することが必須です。

- scenario class
- command
- working directory
- package/module target
- source snapshot identifier when available
- hardware / OS / runtime/toolchain
- warmup rule
- sample count
- input size / participant count / packet count when relevant
- fixture/scenario class and source when data is generated, captured-redacted, v0.1-derived, or manually authored
- metric and unit
- measurement window, precision, and aggregation method
- threshold if pass/fail is claimed
- mock/fake/concrete driver status
- packet rewrite/media transform class when measured
- service discovery source and resolution state when measured
- distributed state class and owner scope when measured
- runtime task class and supervision scope when measured
- internal service trust class and accepted scope when measured
- cross-plane binding class and source/target plane when measured
- rerun condition

## Threshold Rule

initial v0.2 では、明示的 acceptance criterion が定義しない限り production threshold は存在しません。threshold がない場合、benchmark output は measurement のみであり、readiness proof として使ってはなりません。benchmark evidence は performance value ではなく scenario coverage、measurement evidence、report completeness を評価します。performance acceptance を admit する場合、production-completion claim の前に threshold decision を固定することが必須です。

## Mock / Fake Rule

benchmark は fake clock、fake network、fake persistence、generated packet fixtures を使ってよいです。report はそれらを label することが必須です。fake benchmark は fake 条件下の measured code path のみを証明します。generated または captured-redacted scenario data は fixture/scenario data 規則の下で classify することが必須です。

## Benchmark report の構造

benchmark report は benchmark の measurement evidence を構造化します。benchmark command と report classification は、command、environment、workload、measured unit、adoption state、non-correctness-proof warning を含むことが必須です。measurement evidence は reproducibility metadata が存在する場合に限り reportable です。benchmark report は correctness proof ではありません。report の adoption state は、measured または closed reason 付き non-adopted のいずれかであり、scenario identity と workload を明示し続けます。

## 禁止事項

- benchmark result is used as unit/integration test replacement.
- benchmark result is used as production readiness without an acceptance threshold.
- v0.1 benchmark is copied as v0.2 evidence.
- hardware/runtime context is omitted.
- measurement unit/window/precision is hidden.
- benchmark scenario hides use of mocks/fakes.
- benchmark scenario hides generated, captured, or v0.1-derived fixture status.
- benchmark scenario hides packet rewrite, service discovery, or distributed state class when measured.
- benchmark scenario hides runtime task or internal service trust class when measured.
- benchmark scenario hides cross-plane binding class when measured.
- performance metric is used to justify moving domain rule into driver.
- Criterion-only option is forwarded to a mixed workspace benchmark command.
- threshold is implied from a single local run.
- benchmark output is adopted without the required fields.

## Collapse Conditions

この判断が崩れる条件は次のとおりです。

- benchmark evidence lacks rerun condition.
- benchmark threshold is implied but not documented.
- performance result is used to override architecture boundary.
- v0.1 benchmark is treated as current v0.2 proof.
- benchmark passes are described as correctness or production readiness.
- a scenario is omitted without an explicit non-goal basis.
- benchmark evidence compares unnormalized units or unlabeled fixture classes.
- scenario lacks boundary mapping.
- hardware/runtime/toolchain context is omitted.
- measurement unit/window/precision is omitted.
- threshold is claimed without an acceptance criterion.
- mock/fake status is hidden.
- fixture/scenario class is hidden.
- benchmark evidence hides packet rewrite/media transform, service discovery, or distributed state class.
- benchmark evidence hides runtime task/worker lifecycle or internal service identity/trust class.
- benchmark evidence hides cross-plane identity/session binding class.
- workspace benchmark command and Criterion bench-target command are treated as interchangeable.
