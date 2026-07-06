# Chapter 10 benchmark

Status: SSOT consolidated edition
Date: 2026-06-28 JST

## Purpose

This chapter fixes, at a granularity sufficient for reproducible re-implementation, the benchmark domain of the arcRTC v0.2 distro domain: the scenario set (all 20 scenarios), workload parameters, Criterion configuration, benchmark command, harness package layout, scenario function naming, measurement boundary, evidence conversion, measurement report schema, comparison row admission schema, comparability boundary (scenario-level / measurement-level / threshold-level), the claim boundary of benchmark / real-device / readiness evidence, the acceptance-threshold rule, and fail-closed rules. This chapter is fully self-contained and is understandable without consulting other documents or source code. It references only other chapter numbers within this same specification.

benchmark is a measurement of performance / capacity / resource behavior. benchmark MUST NOT claim product completion, production readiness, live readiness, native application readiness, or Kernel completion / freeze by itself. A threshold MUST be adopted before measurement, and MUST NOT be set after the fact to match the result (fail-closed).

---

## 1. Benchmark package ownership

Benchmark source is owned by the test surface. The initial benchmark package path is fixed.

```text
tests/benchmark
```

The initial benchmark package name is fixed.

```text
arcrtc-distro-benchmark-tests
```

The `criterion` dependency is allowed only in this package (MAY). The benchmark package MUST NOT be made a distro workspace member before it is admitted into the workspace.

## 2. Criterion configuration and command

All Criterion benchmark groups MUST use the following.

| Field | Value |
|---|---|
| warm up time | 3 seconds |
| measurement time | 10 seconds |
| sample size | 100 |
| noise threshold | 0.05 |
| confidence level | 0.95 |
| significance level | 0.05 |

The benchmark command from the distro root (`distro/`) is fixed.

```text
cargo bench --manifest-path tests/benchmark/Cargo.toml --bench benchmark_scenarios
```

The benchmark output path is fixed.

```text
target/criterion
```

The evidence JSON output path is fixed.

```text
target/distro-evidence/benchmark/
```

## 3. Scenario set (BENCH-001 through BENCH-020)

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

## 4. Workload fixture rule

benchmark workload MUST use deterministic fixture values.

| Fixture class | Source |
|---|---|
| room / participant / session | `reference-distro/signaling/src/fixture_identity.rs` |
| TURN credential | `reference-distro/turn/src/fixture_credential.rs` |
| SFU route admission | `reference-distro/sfu/src/fixture_route_auth.rs` |
| packet bytes | benchmark-owned stack or vector allocated before measured function |
| profile | `reference-distro/deployment-profiles/benchmark.toml` |

The measured function MUST NOT allocate packet bytes inside the timed closure unless the scenario is explicitly about evidence JSON serialization.

## 5. Harness package layout

`tests/benchmark` MUST hold the following file contract.

| File | Role |
|---|---|
| `Cargo.toml` | benchmark test package manifest |
| `benches/benchmark_scenarios.rs` | Criterion entrypoint |
| `src/lib.rs` | benchmark helper export |
| `src/scenario.rs` | scenario id / workload model |
| `src/workload.rs` | deterministic fixture workload builder |
| `src/evidence.rs` | measurement-to-evidence conversion |

### 5.1 Criterion entrypoint

`benches/benchmark_scenarios.rs` MUST define exactly one Criterion group.

```rust
criterion_group!(distro_benchmark_scenarios, bench_all_scenarios);
criterion_main!(distro_benchmark_scenarios);
```

`bench_all_scenarios` MUST register `BENCH-001` through `BENCH-020`. A scenario MUST NOT be registered outside `bench_all_scenarios`.

### 5.2 Scenario function naming

Each scenario MUST use the following.

```text
bench_{scenario_id_lower_snake}_{scenario_name}
```

Example:

```text
bench_bench_001_signaling_join_room_single
```

The function name MUST include both scenario id and scenario name.

### 5.3 Measurement boundary

The timed closure MAY execute only the measured function and the prebuilt deterministic workload reference. Packet byte vector allocation MUST occur before the timed closure except for the evidence JSON serialization scenario.

## 6. Evidence conversion

`src/evidence.rs` MUST expose the following.

```rust
pub fn build_benchmark_evidence_record(
    base: DistroEvidenceRecord,
    measurement: BenchmarkMeasurement,
) -> Result<BenchmarkEvidenceRecord, BenchmarkEvidenceValidationError>
```

`build_benchmark_evidence_record` MUST call `validate_benchmark_evidence_record(&record)` before returning success.

`BenchmarkMeasurement` fields are fixed.

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

`BenchmarkEvidenceRecord` extends `DistroEvidenceRecord` and MUST include all fields of the measurement report schema. All benchmark scenarios MUST provide a finite positive `throughput_items_per_second`.

## 7. Measurement report schema

Each benchmark evidence report MUST include the following.

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

Every `BENCH-001` through `BENCH-020` scenario MUST emit `throughput_items_per_second`. The value is measured as completed scenario workload items per second for the same measurement window used by Criterion.

Each evidence report MUST additionally hold the following common fields: correlation id / command / working directory / target distro layer / target plane / environment class / toolchain / runtime version / input fixture / workload / expected outcome / actual outcome / reason classification / non-claim scope / rerun condition. reason classification MUST follow the distro evidence reason closed set. UNKNOWN reason MUST NOT be adopted in evidence.

## 8. Comparison row admission schema

A benchmark comparison table row is a report projection. It is not a replacement for `BenchmarkEvidenceRecord` and MUST NOT be used as standalone evidence. Each comparison row admitted into a comparison table MUST include the following.

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

The row MUST be rejected from the comparison table if:

- `scenario_id` is not one of `BENCH-001` through `BENCH-020`.
- `distro_layer` and `target_plane` do not match the scenario table row for `scenario_id`.
- `workload_id` is empty or does not identify the scenario workload row.
- `metric` is not the benchmark measurement metric named by the source evidence.
- `unit` is not the unit emitted by the source evidence.
- `aggregation_rule` is empty or does not name the aggregation used by the source evidence.
- `environment_class` is empty.
- `toolchain_runtime` is empty.
- `sample_count` is not the source evidence sample count.
- `warmup_rule` is empty or does not name the source evidence warm up rule.
- `timestamp` is empty.
- `non_claim_scope` lacks `BenchmarkThresholdNotClaimed` unless a separate threshold admission has admitted threshold evaluation for that exact row scope.

## 9. Comparability boundary

benchmark comparability MUST be separated into the following three classes.

| Comparability class | Meaning | Required before comparison |
|---|---|---|
| scenario-level | measures the same-purpose workload / flow | match of scenario id / target plane / workload |
| measurement-level | compares numbers in the same table | match of metric / unit / aggregation / environment class |
| threshold-level | renders a pass / fail verdict | prior adoption of a threshold admission |

When comparing against a past version or Kernel benchmark, the following MUST be made explicit: historical source id / current distro scenario id / comparable field / non-comparable field / comparison class / not-inherited scope. A past benchmark result MUST NOT be a current distro readiness proof. measurement-level comparison MUST NOT be performed with only scenario-level comparability. A measurement-level comparison MUST NOT be treated as a threshold pass.

## 10. Claim boundary of benchmark / real-device / readiness evidence

| Evidence class | Required owner | May claim | Must not claim |
|---|---|---|---|
| reference benchmark | source: `tests/benchmark` | reference performance / capacity observation | production readiness / live readiness |
| product benchmark | source: `tests/benchmark` | product performance / capacity observation | production readiness / live readiness |
| real-device command | source: `tests/real-device` | bounded device command result | general live readiness |
| production readiness | source: `tests/production-readiness` | bounded production readiness | live readiness |
| live readiness | source: `tests/live` | bounded live readiness | Kernel completion / freeze |

production readiness and live readiness MUST be treated as separate claims. production readiness evidence MUST NOT automatically indicate live readiness. live readiness evidence MUST NOT indicate Kernel completion / freeze.

## 11. Acceptance threshold rule

The scenario / workload definitions of this specification do not admit any threshold by themselves. A threshold admission MUST define the following.

scenario id / target distro layer / target plane / workload / environment class / metric / unit / sample count / warmup / aggregation / comparison baseline / threshold value / comparison rule / failure classification / rerun condition / adoption scope / non-claim scope.

A threshold MUST be adopted before the measurement used for verdict.


The actual measured benchmark numbers (all scenarios, with environment and command) are published separately in `docs/benchmark-results/en.md`, outside this specification. They are measurement evidence only; adopting any value as a pass/fail threshold is the separate decision described in §11.

## 12. Fail-closed / collapse conditions

If any of the following holds, the benchmark specification collapses (treated as fail-closed and not established).

- scenario workload uses a synthetic fallback not enumerated in this specification.
- a benchmark report lacks scenario id or workload summary.
- a benchmark comparison row lacks a required comparison admission field.
- a threshold is decided after observing the measurement result.
- Criterion output is treated as production readiness or live readiness.
- the benchmark package is made a distro workspace member before it is admitted into the workspace.
- a scenario function omits scenario id or scenario name.
- a benchmark scenario is registered outside `bench_all_scenarios`.
- the timed closure allocates packet bytes except for the evidence JSON scenario.
- Criterion measurement is treated as a threshold pass without a threshold admission.
- evidence conversion drops non-claim scope.
- a scenario id is not one of `BENCH-001` through `BENCH-020`.
- benchmark threshold satisfaction is used as a substitute evidence for production readiness / live readiness / real-device success / Kernel completion / freeze.
- a benchmark result is adopted as a readiness proof.
- real-device command success is adopted as live readiness.
- production readiness and live readiness are treated as the same claim.
- UNKNOWN reason is adopted in evidence.
