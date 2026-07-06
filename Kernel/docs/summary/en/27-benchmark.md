# Benchmark

Status: public summary projection
Date: 2026-07-06 JST

## Purpose

This chapter specifies the benchmark scope and scenario matrix of arcRTC v0.2 Kernel. Benchmark is the measurement of performance / capacity / resource behavior; it does not by itself prove correctness, security, runtime readiness, or production readiness. This chapter fixes the measured targets (benchmark surfaces) and non-targets, the point that no acceptance threshold is claimed, every scenario and measured item of the scenario matrix, the fields a report must record to be usable as evidence, and the boundary of command classes.

Normative terms used here mean: **MUST** = a condition that has to be satisfied; **MUST NOT** = a prohibited action; **MAY** = an allowed action; **fail-closed** = when a MUST condition cannot be satisfied, fall to the failing side. Benchmark output lacking the required fields below is treated as diagnostic output only.

## Benchmark Surfaces (measured targets)

The v0.2 initial benchmark surface is limited to the following. A new benchmark surface MUST require an ADR or design decision.

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

The benchmark scope declares non-goals, environment fields, workload fields, and adoption fields. Measurement evidence MUST include command, working directory, toolchain, workload, and result unit. Benchmark measurement evidence is not correctness proof.

## Required Report Fields (scope-side required fields)

A benchmark result MAY be adopted only when the record contains:

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

Benchmark output lacking these fields is diagnostic output only.

## Command Class Boundary

The benchmark command class MUST be fixed before execution.

Workspace-wide benchmark command:

- `cargo bench --workspace`

Criterion target-specific observation command:

- `cargo bench -p arcrtc-roadmap-tests --bench benchmark_scenarios_criterion -- --noplot`

v0.1.2 ice-pool shape comparison target:

- `cargo bench -p arcrtc-roadmap-tests --bench v01_ice_shape_criterion -- --noplot`

This target aligns the benchmark function names, Criterion group name, and input sizes with v0.1.2 `core/benches/ice_pool_bench.rs`. It is a v0.2 test-side comparison surface only; it is not production performance acceptance, not production readiness, not live readiness, and not native command success. The comparable workload MUST stay under the testing crate unless an explicit architecture decision admits an equivalent production ICE candidate pool into v0.2 core semantics.

`--noplot` is a Criterion option. It MUST be passed only to a concrete Criterion bench target. It MUST NOT be passed through `cargo bench --workspace -- --noplot`, because workspace bench execution also runs non-Criterion libtest binaries that reject Criterion-only options. If this mistake occurs, the failed command is not benchmark evidence. The correction is to rerun the workspace benchmark without Criterion-only options and rerun each Criterion bench target explicitly when plot suppression is needed. The failed command, failure reason, corrected command, and corrected result MUST be recorded before any benchmark comparison claim is made.

## Non-Goal Rule

The initial v0.2 architecture does not set a production performance target. Benchmark documentation MAY define measurement method and evidence requirements but MUST NOT invent a production readiness threshold without an explicit acceptance criterion.

If performance acceptance is admitted, a threshold decision MUST exist before any production-completion claim. If it is not admitted, the benchmark threshold remains unclaimed and benchmark evidence remains measurement / scenario coverage evidence only.

## v0.1 Benchmark Rule

v0.1 benchmark code and results are historical evidence-only input. They MAY inform scenario selection but MUST NOT be reused as v0.2 benchmark evidence without rerun or explicit requalification.

## Correctness Boundary

Benchmark passing does not prove correctness. Correctness MUST be proven by tests for the target scope. Benchmark failure may reveal implementation risk, but it does not by itself redefine core semantics.

## Unit and Fixture Boundary

Benchmark measurements MUST report normalized units and the raw source class when useful. Generated packet fixtures, synthetic sessions, captured-redacted samples, and v0.1-derived scenarios MUST be labeled and MUST NOT be adopted as correctness evidence.

## Scenario Matrix (all scenarios and measured items)

The scenario matrix does not define a production threshold; it fixes which boundary each benchmark scenario measures and under what conditions it MAY be adopted as evidence. A new scenario class MUST require an ADR or design decision.

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

The scenario matrix covers the Signaling, SFU, TURN, cross-plane, SDK, and driver overhead classes. Measurement evidence MUST keep scenario identity and workload explicit.

### Scenario Adoption Set (all 16)

Every scenario class in the scenario matrix MUST be explicitly measured or marked non-adopted with a closed reason. Omitting a scenario is failure.

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

## Required Report Fields (scenario-matrix-side required fields)

A benchmark evidence report MUST record:

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

In initial v0.2, no production threshold exists unless an explicit acceptance criterion defines it. Without a threshold, benchmark output is measurement only and MUST NOT be used as readiness proof. Benchmark evidence evaluates scenario coverage, measurement evidence, and report completeness, not a performance value. If performance acceptance is admitted, a threshold decision MUST be fixed before any production-completion claim.

## Mock / Fake Rule

Benchmark MAY use fake clock, fake network, fake persistence, or generated packet fixtures. The report MUST label them. A fake benchmark proves only the measured code path under fake conditions. Generated or captured-redacted scenario data MUST be classified under the fixture/scenario data rules.

## Structure of the Benchmark Report

The benchmark report structures the measurement evidence of the benchmark. The benchmark command and report classification MUST include command, environment, workload, measured unit, adoption state, and a non-correctness-proof warning. Measurement evidence is reportable only when reproducibility metadata exists. The benchmark report is not correctness proof. The adoption state of the report is either measured or closed non-adopted with a closed reason, and it keeps scenario identity and workload explicit.

## Prohibitions

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

This judgement collapses if:

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
