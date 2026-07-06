# 第10章 benchmark

状態: SSOT 統合版
日付: 2026-06-28 JST

## 目的

本章は、arcRTC v0.2 distro 領域における benchmark の scenario set（全 20 scenario）、workload parameter、Criterion configuration、benchmark command、harness package layout、scenario function naming、measurement boundary、evidence conversion、measurement report schema、comparison row admission schema、comparability boundary（scenario-level / measurement-level / threshold-level）、benchmark / real-device / readiness evidence の claim boundary、acceptance-threshold 規則、fail-closed 規則を、再現実装可能な粒度で固定します。本章は完全自己完結であり、他文書・実コードを参照せずに理解できます。同一仕様書内の他章番号のみ参照します。

benchmark は performance / capacity / resource behavior の測定です。benchmark は product completion、production readiness、live readiness、native application readiness、Kernel completion / freeze を単独では主張しません（MUST NOT）。閾値は測定前に採用済みでなければならず、測定後に結果へ合わせて設定しません（MUST NOT、fail-closed）。

---

## 1. benchmark package ownership

benchmark source は test surface が所有します。初期 benchmark package path は固定です。

```text
tests/benchmark
```

初期 benchmark package name は固定です。

```text
arcrtc-distro-benchmark-tests
```

`criterion` dependency はこの package だけで許可します（MAY）。benchmark package は workspace に admit される前に distro workspace member にしません（MUST NOT）。

## 2. Criterion configuration とコマンド

すべての Criterion benchmark group は次を使います（MUST）。

| Field | Value |
|---|---|
| warm up time | 3 seconds |
| measurement time | 10 seconds |
| sample size | 100 |
| noise threshold | 0.05 |
| confidence level | 0.95 |
| significance level | 0.05 |

distro root（`distro/`）からの benchmark command は固定です。

```text
cargo bench --manifest-path tests/benchmark/Cargo.toml --bench benchmark_scenarios
```

benchmark output path は固定です。

```text
target/criterion
```

evidence JSON output path は固定です。

```text
target/distro-evidence/benchmark/
```

## 3. scenario set（BENCH-001 through BENCH-020）

| Scenario id | Scenario name | Layer | Plane | Workload |
|---|---|---|---|---|
| `BENCH-001` | `signaling_join_room_single` | reference | signaling | one room, one participant, one `JoinRoom` |
| `BENCH-002` | `signaling_join_room_batch` | reference | signaling | 8 rooms, 6 participants per room, `JoinRoom` for all participants |
| `BENCH-003` | `signaling_offer_answer_candidate` | reference | signaling | joined pair, offer, answer, 4 candidates |
| `BENCH-004` | `signaling_turn_credential_request` | reference | signaling | joined participant, one `RequestTurnCredential` |
| `BENCH-005` | `turn_allocate_single` | reference | turn | one fixture credential, one `Allocate` |
| `BENCH-006` | `turn_permission_batch` | reference | turn | 8 allocations, 16 permissions |
| `BENCH-007` | `turn_channel_bind_batch` | reference | turn | 8 allocations, 16 permissions, 16 channel binds |
| `BENCH-008` | `turn_relay_data_path` | reference | turn | active allocation / permission, 1200 byte packet id fixture |
| `BENCH-009` | `sfu_contract_item_create` | reference | sfu | one `SfuContractItem` per route |
| `BENCH-010` | `sfu_borrowed_packet_view` | reference | sfu | borrowed 1200 byte packet / payload slices |
| `BENCH-011` | `sfu_route_select_batch` | reference | sfu | 24 routes selected across 8 sessions |
| `BENCH-012` | `sfu_packet_fanout_intent` | reference | sfu | one source packet, 6 target endpoint references |
| `BENCH-013` | `composition_signaling_turn_binding` | reference | composition | 8 joined rooms and 8 active allocations |
| `BENCH-014` | `composition_signaling_sfu_binding` | reference | composition | 8 joined rooms and 24 SFU routes |
| `BENCH-015` | `runtime_start_shutdown` | reference | ops | start reference runtime then graceful shutdown |
| `BENCH-016` | `evidence_json_record_write` | reference | ops | build one `DistroEvidenceRecord` and serialize JSON |
| `BENCH-017` | `product_policy_evaluate` | product | policy | one product policy input per plane |
| `BENCH-018` | `product_projection_mapping` | product | persistence | one projection mapping per record class |
| `BENCH-019` | `product_runtime_select` | product | deployment | select local product runtime profile |
| `BENCH-020` | `product_drain_plan_create` | product | rollback | create drain plan for signaling / turn / sfu |

## 4. workload fixture 規則

benchmark workload は deterministic fixture value を使います（MUST）。

| Fixture class | Source |
|---|---|
| room / participant / session | `reference-distro/signaling/src/fixture_identity.rs` |
| TURN credential | `reference-distro/turn/src/fixture_credential.rs` |
| SFU route admission | `reference-distro/sfu/src/fixture_route_auth.rs` |
| packet bytes | benchmark-owned stack or vector allocated before measured function |
| profile | `reference-distro/deployment-profiles/benchmark.toml` |

measured function は、evidence JSON serialization scenario を除き、timed closure 内で packet bytes を allocate しません（MUST NOT）。

## 5. harness package layout

`tests/benchmark` は次の file contract を持ちます（MUST）。

| File | Role |
|---|---|
| `Cargo.toml` | benchmark test package manifest |
| `benches/benchmark_scenarios.rs` | Criterion entrypoint |
| `src/lib.rs` | benchmark helper export |
| `src/scenario.rs` | scenario id / workload model |
| `src/workload.rs` | deterministic fixture workload builder |
| `src/evidence.rs` | measurement-to-evidence conversion |

### 5.1 Criterion entrypoint

`benches/benchmark_scenarios.rs` は厳密に 1 つの Criterion group を定義します（MUST）。

```rust
criterion_group!(distro_benchmark_scenarios, bench_all_scenarios);
criterion_main!(distro_benchmark_scenarios);
```

`bench_all_scenarios` は `BENCH-001` through `BENCH-020` を登録します（MUST）。scenario を `bench_all_scenarios` の外で登録しません（MUST NOT）。

### 5.2 scenario function naming

各 scenario は次を使います（MUST）。

```text
bench_{scenario_id_lower_snake}_{scenario_name}
```

例:

```text
bench_bench_001_signaling_join_room_single
```

function name は scenario id と scenario name の両方を含みます（MUST）。

### 5.3 measurement boundary

timed closure は measured function と prebuilt deterministic workload reference だけを実行できます（MUST）。packet byte vector allocation は、evidence JSON serialization scenario を除き、timed closure の前に行います（MUST）。

## 6. evidence conversion

`src/evidence.rs` は次を expose します（MUST）。

```rust
pub fn build_benchmark_evidence_record(
    base: DistroEvidenceRecord,
    measurement: BenchmarkMeasurement,
) -> Result<BenchmarkEvidenceRecord, BenchmarkEvidenceValidationError>
```

`build_benchmark_evidence_record` は success を返す前に `validate_benchmark_evidence_record(&record)` を呼びます（MUST）。

`BenchmarkMeasurement` の field は固定です。

- `scenario_id`
- `scenario_name`
- `criterion_group`
- `sample_size`
- `warm_up_seconds`
- `measurement_seconds`
- `noise_threshold`
- `confidence_level`
- `significance_level`
- `median_ns`
- `mean_ns`
- `std_dev_ns`
- `p95_ns`
- `throughput_items_per_second: f64`

`BenchmarkEvidenceRecord` は `DistroEvidenceRecord` を拡張し、measurement report schema の全 field を含みます（MUST）。すべての benchmark scenario は finite positive な `throughput_items_per_second` を提供します（MUST）。

## 7. measurement report schema

各 benchmark evidence report は次を含みます（MUST）。

- `scenario_id`
- `scenario_name`
- `layer`
- `plane`
- `workload_summary`
- `criterion_group`
- `sample_size`
- `warm_up_seconds`
- `measurement_seconds`
- `noise_threshold`
- `confidence_level`
- `significance_level`
- `median_ns`
- `mean_ns`
- `std_dev_ns`
- `p95_ns`
- `throughput_items_per_second`
- `non_claim_scope`

`BENCH-001` through `BENCH-020` のすべての scenario は `throughput_items_per_second` を emit します（MUST）。値は Criterion と同じ measurement window で完了した scenario workload item 数 / second として測定します。

各 evidence report はさらに次の共通 field を持ちます（MUST）。correlation id / command / working directory / target distro layer / target plane / environment class / toolchain / runtime version / input fixture / workload / expected outcome / actual outcome / reason classification / non-claim scope / rerun condition。reason classification は distro evidence reason の closed set に従います（MUST）。UNKNOWN reason を evidence に採用しません（MUST NOT）。

## 8. comparison row admission schema

benchmark comparison table row は report projection です。`BenchmarkEvidenceRecord` の代替ではなく、standalone evidence として使いません（MUST NOT）。comparison table に admit される各 comparison row は次を含みます（MUST）。

- `scenario_id`
- `distro_layer`
- `target_plane`
- `workload_id`
- `metric`
- `unit`
- `aggregation_rule`
- `environment_class`
- `toolchain_runtime`
- `sample_count`
- `warmup_rule`
- `timestamp`
- `non_claim_scope`

row は次の場合に comparison table から拒否します（MUST）。

- `scenario_id` が `BENCH-001` through `BENCH-020` のいずれでもない。
- `distro_layer` と `target_plane` が `scenario_id` の scenario table row に一致しない。
- `workload_id` が空、または scenario workload row を識別しない。
- `metric` が source evidence の benchmark measurement metric でない。
- `unit` が source evidence の emit unit でない。
- `aggregation_rule` が空、または source evidence の aggregation を名指さない。
- `environment_class` が空。
- `toolchain_runtime` が空。
- `sample_count` が source evidence の sample count でない。
- `warmup_rule` が空、または source evidence warm up rule を名指さない。
- `timestamp` が空。
- `non_claim_scope` に `BenchmarkThresholdNotClaimed` を欠く（その exact row scope に対して別途 threshold admission が threshold evaluation を admit している場合を除く）。

## 9. comparability boundary

benchmark comparability は次の 3 class に分離します（MUST）。

| Comparability class | Meaning | Required before comparison |
|---|---|---|
| scenario-level | 同じ目的の workload / flow を測る | scenario id / target plane / workload の一致 |
| measurement-level | 数値を同一表で比較する | metric / unit / aggregation / environment class の一致 |
| threshold-level | pass / fail 判定する | threshold admission の事前採用 |

過去版または Kernel benchmark と比較する場合、次を明示します（MUST）。historical source id / current distro scenario id / comparable field / non-comparable field / comparison class / not-inherited scope。過去 benchmark result は current distro readiness proof ではありません（MUST NOT）。scenario-level comparability だけで measurement-level comparison を行いません（MUST NOT）。measurement-level comparison を threshold pass として扱いません（MUST NOT）。

## 10. benchmark / real-device / readiness evidence の claim boundary

| Evidence class | Required owner | May claim | Must not claim |
|---|---|---|---|
| reference benchmark | source: `tests/benchmark` | reference performance / capacity observation | production readiness / live readiness |
| product benchmark | source: `tests/benchmark` | product performance / capacity observation | production readiness / live readiness |
| real-device command | source: `tests/real-device` | bounded device command result | general live readiness |
| production readiness | source: `tests/production-readiness` | bounded production readiness | live readiness |
| live readiness | source: `tests/live` | bounded live readiness | Kernel completion / freeze |

production readiness と live readiness は別 claim として扱います（MUST）。production readiness evidence は live readiness を自動的に示しません（MUST NOT）。live readiness evidence は Kernel completion / freeze を示しません（MUST NOT）。

## 11. acceptance threshold 規則

本仕様の scenario / workload 定義自体は threshold を admit しません。threshold admission は次を定義します（MUST）。

scenario id / target distro layer / target plane / workload / environment class / metric / unit / sample count / warmup / aggregation / comparison baseline / threshold value / comparison rule / failure classification / rerun condition / adoption scope / non-claim scope。

threshold は verdict に使う measurement より前に採用済みでなければなりません（MUST）。


全 scenario の実測ベンチマーク数値（環境・コマンド込み）は、本仕様の外の `docs/benchmark-results/ja.md` に別途掲載しています。これは測定の実測値のみであり、いずれかの値を合否 threshold として採用するのは §11 に述べた別判断です。

## 12. fail-closed / collapse 条件

次のいずれかに該当する場合、benchmark 仕様は崩れます（fail-closed で不成立として扱います）。

- scenario workload が本仕様に列挙されない synthetic fallback を使う。
- benchmark report が scenario id または workload summary を欠く。
- benchmark comparison row が required comparison admission field を欠く。
- threshold を measurement result の観測後に決める。
- Criterion output を production readiness または live readiness として扱う。
- benchmark package を workspace に admit される前に distro workspace member にする。
- scenario function が scenario id または scenario name を欠く。
- benchmark scenario を `bench_all_scenarios` の外で登録する。
- timed closure が evidence JSON scenario 以外で packet bytes を allocate する。
- Criterion measurement を threshold admission なしに threshold pass として扱う。
- evidence conversion が non-claim scope を drop する。
- scenario id が `BENCH-001` through `BENCH-020` 以外である。
- benchmark threshold satisfaction を production readiness / live readiness / real-device success / Kernel completion / freeze の代替証跡にする。
- benchmark result を readiness proof として採用する。
- real-device command success を live readiness として採用する。
- production readiness と live readiness を同一 claim として扱う。
- UNKNOWN reason を evidence に採用する。
