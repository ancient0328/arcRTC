# Chapter 13 testing-and-assertions

Status: SSOT consolidated edition
Date: 2026-06-28 JST

## Purpose

This chapter fixes the complete specification of the test package structure and assertion rules in the arcRTC v0.2 distro domain. Specifically, it describes, at a granularity sufficient for reproducible re-implementation: the test package set (all 7 packages with their paths, names, and ownership); all boundary / reference / product / benchmark / real-device / readiness test files with each assertion; the extension validator negative branch rule (all benchmark / real-device / readiness validation errors with expected reasons); the expected failure reason rule; the test package manifest shape; manifest path commands; test dependency admission; each test package's `[dependencies]` literal stanza; the path rule; and the testing scope and non-claim of this domain. This chapter is fully self-contained and is understandable without consulting other documents or source code. It references only other chapter numbers within this same specification.

As a dependency rule, distro depends on the Kernel only through contract / SDK / command surface. This chapter defines the test distro shape; it does not claim test pass, benchmark pass, real-device success, production readiness, or live readiness. A close-like claim MUST NOT be made on the basis of test definitions alone.

## 1. Role and Non-Claim of the Testing Domain

This chapter is the definition domain of test evidence on the distro side and defines the test distro shape; it MUST NOT claim test distro completion, test pass, benchmark pass, production readiness, or live readiness.

## 2. Test Package Set (all 7 packages)

The test packages are not distro workspace members. The test package manifest, dependency admission, and manifest-path command are fixed by sections 8 through 13 of this chapter.

| Package path | Package name | Owns |
|---|---|---|
| `tests/boundary` | `arcrtc-distro-boundary-tests` | docs / dependency / export boundary tests |
| `tests/reference` | `arcrtc-distro-reference-tests` | reference behavior tests |
| `tests/product` | `arcrtc-distro-product-tests` | product boundary tests |
| `tests/benchmark` | `arcrtc-distro-benchmark-tests` | benchmark scenario tests |
| `tests/real-device` | `arcrtc-distro-real-device-tests` | real-device command evidence wrapper tests |
| `tests/production-readiness` | `arcrtc-distro-production-readiness-tests` | production readiness matrix tests |
| `tests/live` | `arcrtc-distro-live-tests` | live readiness matrix tests |

## 3. Boundary Test Files (all assertions)

| Test file | Assertions |
|---|---|
| `tests/boundary/tests/docs_authority.rs` | the canonical index contains all current canonical files; root index authority order contains all canonical files. |
| `tests/boundary/tests/dependency_admission.rs` | package manifests contain only admitted dependencies as specified by the dependency admission rule and the workspace manifest and dependency path rule. |
| `tests/boundary/tests/module_export.rs` | every `src/lib.rs` public module / re-export matches the module export rule. |
| `tests/boundary/tests/evidence_schema.rs` | evidence JSON records satisfy the evidence schema; all base string fields including `correlation_id` containing secret-like or raw-payload markers return `RawSecretLikeValue`. |
| `tests/boundary/tests/command_evidence.rs` | command evidence record required fields, command class, working directory, output directory, non-claim scope, and evidence writer owner match the command evidence runner rule and the command CI quality gate rule defined in Chapter 14. |
| `tests/boundary/tests/secret_literal.rs` | source and report current-authority text contain no raw secret / credential literal; redacted or hash-form device identifiers are accepted only where the real-device wrapper command rule admits them. |
| `tests/boundary/tests/test_package_shape.rs` | test package manifest, package path, manifest-path command, and assertion file set match the test workspace manifest rule and this chapter. |
| `tests/boundary/tests/evidence_owner.rs` | shared evidence / reason owner, evidence wire values, non-claim scope closed set, and extension evidence owner files match the distro evidence and reason ownership rule, the evidence wire format rule, and the evidence extension record rule. |
| `tests/boundary/tests/old_vocabulary.rs` | old vocabulary scan returns no current-authority hit except archived material. |
| `tests/boundary/tests/product_reference_output_boundary.rs` | product plane package manifests depend on `arcrtc-reference-output` only to access the 3-type product input allow-list, not to treat the full reference output package as product input; product source imports from `arcrtc_reference_output` only by exact allow-list: `ReferenceSignalingOutcome` for product signaling, `ReferenceTurnOutcome` for product TURN, and `ReferenceSfuOutcome` for product SFU; product source imports none of `arcrtc_reference_output::*`, the `arcrtc_reference_output` module path, re-export, type alias path, `ReferenceCompositionOutcome`, `arcrtc_reference_signaling`, `arcrtc_reference_turn`, `arcrtc_reference_sfu`, `arcrtc_reference_composition`, `arcrtc_reference_ops`, `Reference*State`, `ReferenceSfuAction`, `apply_reference_*`, `validate_reference_*`, fixture, local auth, runtime, or composition symbols; violation maps to `COMMAND_SCOPE_MISMATCH`. |

## 4. Reference Test Files (all assertions)

| Test file | Assertions |
|---|---|
| `tests/reference/tests/signaling_contract.rs` | `build_kernel_signaling_command` uses a fixed constructor chain and command type literal. |
| `tests/reference/tests/signaling_state.rs` | join / leave / offer / answer / candidate transitions match the reference state mutation rule. |
| `tests/reference/tests/signaling_fixture.rs` | offer direction mismatch and answer direction mismatch return `InvalidFixtureIdentity`. |
| `tests/reference/tests/turn_contract.rs` | `build_kernel_turn_command` delegates to `TurnCommand::try_new` and maps constructor failure. |
| `tests/reference/tests/turn_state.rs` | allocate / refresh / permission / channel bind / relay transitions match the mutation table. |
| `tests/reference/tests/turn_fixture.rs` | invalid fixture TURN credential returns `InvalidFixtureIdentity`; accepted fixture credential remains deterministic and does not create product credential authority. |
| `tests/reference/tests/sfu_contract.rs` | `build_kernel_sfu_item` and `build_borrowed_packet_view` use fixed constructors. |
| `tests/reference/tests/sfu_state.rs` | `AdmitParticipant`, `RejectParticipant`, `PublishStream`, `SubscribeRoute`, `SelectRoute`, `SuppressForwarding`, `DropForwarding`, and `CloseSession` transitions match the mutation table; typed `ReferenceSfuAction` fields are not tested as missing fields; session absent admission creates session / endpoint; closed / draining session, rejected / removed endpoint, route session / endpoint / stream mismatch, non-selected suppression, dropped / closed route, and absent / closed session close return `ReferenceSfuError::StateBoundaryViolation`. |
| `tests/reference/tests/sfu_fixture.rs` | fixture route admission accepts only deterministic reference-local route authorization and rejects missing session / endpoint / stream / route fields as `StateBoundaryViolation` or `InvalidFixtureIdentity` according to the fixture canonical. |
| `tests/reference/tests/borrowed_packet_view.rs` | the borrowed packet view builder preserves raw packet / payload slices without allocation or copy and never stores packet bytes in reference state. |
| `tests/reference/tests/reference_api_signature.rs` | public reference API signatures, phase enum, fixture payload, package ownership, and product input allow-list boundaries match the reference API and state rule, the reference phase and fixture payload rule, the reference distro rule, and the reference output boundary rule. |
| `tests/reference/tests/profile.rs` | reference local and benchmark profile literals match the reference profile rule and do not imply production or live readiness. |
| `tests/reference/tests/composition.rs` | signaling-to-turn and signaling-to-sfu bindings accept only existing Signaling / TURN / SFU state and reject orphan references by direct `ReferenceCompositionState` execution. |
| `tests/reference/tests/runtime.rs` | reference runtime start / shutdown states match the runtime lifecycle rule; shutdown before start fail-closes; reference ops build evidence accepts only the reference ops build scope. |
| `tests/reference/tests/error_reason.rs` | every reference error variant maps to a fixed distro reason. |

## 5. Product Test Files (all assertions)

| Test file | Assertions |
|---|---|
| `tests/product/tests/product_api.rs` | public product API signatures match the product API policy rule, the product plane API signature rule, and the reference output boundary rule; product policy input accepts only `arcrtc_reference_output::ReferenceSignalingOutcome`, `ReferenceTurnOutcome`, and `ReferenceSfuOutcome`; `ReferenceCompositionOutcome`, wildcard import, module import, re-export, and type alias import are rejected as product API input surface. |
| `tests/product/tests/product_policy.rs` | product policy decisions allow fixture identity, deny missing identity with `FixtureIdentityInvalid`, reject blank fixture identity, and do not claim production / live readiness. |
| `tests/product/tests/persistence_topology.rs` | in-memory projection topology admits session / allocation / route / evidence projections; `ProviderDeferred` and `NotAdmitted` fail closed; mismatched plane-to-record-class projection fails closed. |
| `tests/product/tests/deployment.rs` | `public_endpoint_claimed` is false without live readiness admission. |
| `tests/product/tests/monitoring.rs` | monitoring build / test evidence accepts only product package, product scope, product layer, Build/Test command shape, non-Composition/Ops plane, and non-readiness non-claim scopes. |
| `tests/product/tests/rollback.rs` | drain / restore plans return `DistroOk` only for admitted local / in-memory paths; empty drain and production/provider-deferred paths return fail-closed distro reasons without live readiness claim. |
| `tests/product/tests/error_reason.rs` | every product error variant maps to a fixed distro reason. |

## 6. Benchmark / Real-Device / Readiness Test Files (all assertions)

| Test file | Assertions |
|---|---|
| `tests/benchmark/tests/scenario_set.rs` | all `BENCH-001` through `BENCH-020` are present and have a workload summary. |
| `tests/benchmark/tests/measurement_schema.rs` | benchmark report includes all fields required by the benchmark scenario workload rule; every `BENCH-001` through `BENCH-020` report has a finite positive `throughput_items_per_second`; Criterion configuration fields match warm up `3`, measurement `10`, sample size `100`, noise threshold `0.05`, confidence level `0.95`, significance level `0.05`; benchmark extension validator negative branches match the rule in section 7. |
| `tests/benchmark/tests/comparison_row.rs` | every admitted benchmark comparison row has `scenario_id`, `distro_layer`, `target_plane`, `workload_id`, `metric`, `unit`, `aggregation_rule`, `environment_class`, `toolchain_runtime`, `sample_count`, `warmup_rule`, `timestamp`, and `non_claim_scope`; invalid `scenario_id`, layer/plane mismatch, empty or mismatched `workload_id`, metric mismatch, unit mismatch, empty or mismatched `aggregation_rule`, empty `environment_class`, empty `toolchain_runtime`, source sample-count mismatch, empty or mismatched `warmup_rule`, empty `timestamp`, and missing `BenchmarkThresholdNotClaimed` reject the row from comparison admission. |
| `tests/benchmark/tests/harness_layout.rs` | benchmark package layout, Criterion group name, function naming, timed-closure allocation boundary, and evidence conversion function match the benchmark harness layout rule. |
| `tests/real-device/tests/command_matrix.rs` | all real-device evidence ids and device classes are in the closed set; successful real-device evidence uses `redacted` or `sha256:<64 lowercase hex characters>` for `redacted_device_identifier`; raw identifier markers such as serial / udid / android_id / device_id / imei / meid / account / token / private_key / device_name return `UnsafeDeviceIdentifier`; base-validator-first conditions such as missing required non-claim scope or missing exit status return `Base(error)`; real-device extension validator negative branches match the rule in section 7. |
| `tests/real-device/tests/non_claim_scope.rs` | real-device evidence includes the required non-claim scope. |
| `tests/real-device/tests/wrapper_command.rs` | real-device wrapper CLI, platform command closed set, post-execution requiredness, exit status mapping, and manifest-path invocation match the real-device wrapper command rule. |
| `tests/production-readiness/tests/production_matrix.rs` | all `PRD-001` through `PRD-009` gates are present; every record has `readiness_claim == ProductionReadiness`; each gate matches the required extension ref fields in the readiness matrix defined in Chapter 12. |
| `tests/production-readiness/tests/fail_closed.rs` | `PRD-003` with `ReadinessValidationContext.auth_provider_authority == Absent` returns `READINESS_NOT_ADMITTED`; `PRD-004` with `ReadinessValidationContext.persistence_provider_authority == Absent` returns `READINESS_NOT_ADMITTED`; `readiness_claim == ProductionReadiness` with any `LIVE-001` through `LIVE-008` returns `ReadinessGateClaimMismatch` mapped to `READINESS_NOT_ADMITTED`; production-readiness extension validator negative branches match the rule in section 7. |
| `tests/production-readiness/tests/success_matrix.rs` | `PRD-001` through `PRD-009` success records are accepted only with their gate-specific ref field; auth provider admission, persistence provider admission, production profile, production monitoring probe, production drain / restore plan, security scan ref, and production readiness Closed Gate ref are all connected to concrete source or report files. |
| `tests/live/tests/live_matrix.rs` | all `LIVE-001` through `LIVE-008` gates are present; every record has `readiness_claim == LiveReadiness`; `LIVE-005` requires `rollback_drain_execution_ref`; each gate matches the required extension ref fields in the readiness matrix defined in Chapter 12. |
| `tests/live/tests/fail_closed.rs` | any live gate with `ReadinessValidationContext.live_endpoint_authority == Absent` returns `READINESS_NOT_ADMITTED`; `LIVE-001` with `production_readiness_report == Absent` returns `READINESS_NOT_ADMITTED`; `LIVE-003` with `public_traversal_authority == Absent` returns `READINESS_NOT_ADMITTED`; `readiness_claim == LiveReadiness` with any `PRD-001` through `PRD-009` returns `ReadinessGateClaimMismatch` mapped to `READINESS_NOT_ADMITTED`; live-readiness extension validator negative branches match the rule in section 7. |
| `tests/live/tests/success_matrix.rs` | `LIVE-001` through `LIVE-008` success records are accepted only with their gate-specific ref field; production readiness report ref, live endpoint admission, public traversal admission, live monitoring probe, live shutdown drain, live restore, Signaling / TURN / SFU live public endpoint branch, and live readiness Closed Gate ref are all connected to concrete source or report files. |

## 7. Extension Validator Negative Branch Rule

Each extension validator test MUST assert `Base(error)` passthrough based on the evidence wire format.

### 7.1 Benchmark extension validator negative branches

| Validation error | Expected reason |
|---|---|
| `CommandClassMismatch` | `EVIDENCE_FIELDS_INCOMPLETE` |
| `ScenarioIdNotListed` | `BENCHMARK_SCOPE_MISMATCH` |
| `ScenarioNameMismatch` | `BENCHMARK_SCOPE_MISMATCH` |
| `ScenarioLayerMismatch` | `BENCHMARK_SCOPE_MISMATCH` |
| `ScenarioPlaneMismatch` | `BENCHMARK_SCOPE_MISMATCH` |
| `WorkloadSummaryMismatch` | `BENCHMARK_SCOPE_MISMATCH` |
| `CriterionGroupMismatch` | `BENCHMARK_SCOPE_MISMATCH` |
| `CriterionConfigurationMismatch` | `BENCHMARK_SCOPE_MISMATCH` |
| `InvalidTimingValue` | `EVIDENCE_FIELDS_INCOMPLETE` |
| `InvalidThroughputValue` | `EVIDENCE_FIELDS_INCOMPLETE` |

### 7.2 Real-device extension validator negative branches

| Validation error | Expected reason |
|---|---|
| `CommandClassMismatch` | `EVIDENCE_FIELDS_INCOMPLETE` |
| `PlatformDeviceClassMismatch` | `REAL_DEVICE_SCOPE_MISMATCH` |
| `InternalCommandClassMismatch` | `REAL_DEVICE_SCOPE_MISMATCH` |
| `RuntimeVersionClassMismatch` | `REAL_DEVICE_SCOPE_MISMATCH` |
| `ExecutionSurfaceMismatch` | `REAL_DEVICE_SCOPE_MISMATCH` |
| `NetworkClassMismatch` | `REAL_DEVICE_SCOPE_MISMATCH` |
| `PlatformCommandMissing` | `REAL_DEVICE_SCOPE_MISMATCH` |
| `PlatformCommandUnexpected` | `REAL_DEVICE_SCOPE_MISMATCH` |
| `PlatformCommandNotAdmitted` | `REAL_DEVICE_SCOPE_MISMATCH` |
| `MissingPostExecutionField` | `EVIDENCE_FIELDS_INCOMPLETE` |
| `EmptyLogsMetricsLocation` | `EVIDENCE_FIELDS_INCOMPLETE` |
| `LogsMetricsLocationOutsideEvidenceDir` | `COMMAND_SCOPE_MISMATCH` |
| `EmptyDeviceIdentifier` | `EVIDENCE_FIELDS_INCOMPLETE` |
| `DeviceIdentifierFormatMismatch` | `EVIDENCE_FIELDS_INCOMPLETE` |
| `UnexpectedDeviceIdentifier` | `EVIDENCE_FIELDS_INCOMPLETE` |
| `UnsafeDeviceIdentifier` | `EVIDENCE_FIELDS_INCOMPLETE` |

### 7.3 Readiness extension validator negative branches

The readiness extension validator negative branches are asserted by both the production-readiness and live-readiness fail-closed tests as follows.

| Validation error | Expected reason |
|---|---|
| `CommandClassClaimMismatch` | `EVIDENCE_FIELDS_INCOMPLETE` |
| `ReadinessGateClaimMismatch` | `READINESS_NOT_ADMITTED` |
| `GateIdNotListed` | `READINESS_NOT_ADMITTED` |
| `ReadinessAuthorityRefMismatch` | `READINESS_NOT_ADMITTED` |
| `MissingRequiredExtensionRef` | `EVIDENCE_FIELDS_INCOMPLETE` |
| `EmptyRequiredExtensionRef` | `EVIDENCE_FIELDS_INCOMPLETE` |
| `UnexpectedExtensionRefPopulated` | `EVIDENCE_FIELDS_INCOMPLETE` |
| `AuthProviderAuthorityNotAdmitted` | `READINESS_NOT_ADMITTED` |
| `PersistenceProviderAuthorityNotAdmitted` | `READINESS_NOT_ADMITTED` |
| `LiveEndpointAuthorityNotAdmitted` | `READINESS_NOT_ADMITTED` |
| `ProductionReadinessReportNotAdmitted` | `READINESS_NOT_ADMITTED` |
| `PublicTraversalAuthorityNotAdmitted` | `READINESS_NOT_ADMITTED` |
| `SecretLikeExtensionRef` | `EVIDENCE_FIELDS_INCOMPLETE` |

## 8. Expected Failure Reason Rule

| Test category | Expected failure reason |
|---|---|
| dependency outside admission matrix | `DEPENDENCY_NOT_ADMITTED` |
| wrong working directory | `COMMAND_SCOPE_MISMATCH` |
| missing evidence field | `EVIDENCE_FIELDS_INCOMPLETE` |
| fixture identity mismatch | `FIXTURE_IDENTITY_INVALID` |
| invalid state transition | `STATE_BOUNDARY_VIOLATION` |
| readiness gate without admission | `READINESS_NOT_ADMITTED` |
| readiness claim/gate mismatch | `READINESS_NOT_ADMITTED` |
| benchmark workload mismatch | `BENCHMARK_SCOPE_MISMATCH` |
| real-device scope mismatch | `REAL_DEVICE_SCOPE_MISMATCH` |

## 9. Test Workspace Manifest (root rule and manifest shape)

Test packages are not distro workspace members. Test execution is not a distro completion condition.

Test packages are invoked by explicit manifest path from `distro/`. No `tests/*` package is added to the distro workspace root members.

Each test package `Cargo.toml` uses the following.

```toml
[package]
name = "<test-package-name>"
version = "0.0.0"
edition = "2021"
rust-version = "1.96"
license = "Apache-2.0"
publish = false

[workspace]

[lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"
```

`[workspace]` is intentionally empty. It prevents Cargo from treating `tests/*` explicit-manifest packages as unlisted members of the parent `distro` workspace while preserving the rule that test packages are not distro workspace members.

## 10. Test Package Commands (all manifest path commands)

| Package path | Package name | Command from distro root |
|---|---|---|
| `tests/boundary` | `arcrtc-distro-boundary-tests` | `cargo test --manifest-path tests/boundary/Cargo.toml` |
| `tests/reference` | `arcrtc-distro-reference-tests` | `cargo test --manifest-path tests/reference/Cargo.toml` |
| `tests/product` | `arcrtc-distro-product-tests` | `cargo test --manifest-path tests/product/Cargo.toml` |
| `tests/benchmark` | `arcrtc-distro-benchmark-tests` | `cargo test --manifest-path tests/benchmark/Cargo.toml` and `cargo bench --manifest-path tests/benchmark/Cargo.toml --bench benchmark_scenarios` |
| `tests/real-device` | `arcrtc-distro-real-device-tests` | `cargo run --manifest-path tests/real-device/Cargo.toml -- <platform> --profile reference-local --device-class <device-class>` |
| `tests/production-readiness` | `arcrtc-distro-production-readiness-tests` | `cargo test --manifest-path tests/production-readiness/Cargo.toml` |
| `tests/live` | `arcrtc-distro-live-tests` | `cargo test --manifest-path tests/live/Cargo.toml` |

## 11. Test Dependency Admission

| Test package | Required local dependencies |
|---|---|
| `arcrtc-distro-boundary-tests` | `arcrtc-distro-evidence` |
| `arcrtc-distro-reference-tests` | `arcrtc-core-identity`, `arcrtc-core-signaling`, `arcrtc-core-turn`, `arcrtc-core-sfu`, `arcrtc-distro-evidence`, all reference distro packages |
| `arcrtc-distro-product-tests` | `arcrtc-core-identity`, `arcrtc-core-sfu`, `arcrtc-core-signaling`, `arcrtc-core-turn`, `arcrtc-distro-evidence`, `arcrtc-reference-output`, all product distro packages |
| `arcrtc-distro-benchmark-tests` | `arcrtc-core-identity`, `arcrtc-core-sfu`, `arcrtc-core-signaling`, `arcrtc-core-turn`, `arcrtc-distro-evidence`, `arcrtc-reference-composition`, `arcrtc-reference-ops`, `arcrtc-reference-sfu`, `arcrtc-reference-signaling`, `arcrtc-reference-turn`, `arcrtc-product-deployment`, `arcrtc-product-persistence-topology`, `arcrtc-product-policy`, `arcrtc-product-rollback` |
| `arcrtc-distro-real-device-tests` | `arcrtc-distro-evidence` |
| `arcrtc-distro-production-readiness-tests` | `arcrtc-core-identity`, `arcrtc-distro-evidence`, product policy, product persistence topology, product monitoring, product rollback, product deployment |
| `arcrtc-distro-live-tests` | `arcrtc-distro-evidence`, product monitoring, product rollback, product deployment |

`criterion` is admitted only in `arcrtc-distro-benchmark-tests`. `serde_json` is admitted in test packages only when evidence JSON parsing / writing is part of the assertion.

## 12. Path Rule

Local path dependencies from `tests/<package>` to distro packages use `../../`.

Example:

```toml
arcrtc-distro-evidence = { path = "../../distro-support/evidence" }
```

## 13. Test Package Dependency Literal Stanza

The following dependency stanzas are complete `[dependencies]` sections for each test package. They are not fragments.

### `tests/boundary/Cargo.toml`

```toml
[dependencies]
arcrtc-distro-evidence = { path = "../../distro-support/evidence" }
```

### `tests/reference/Cargo.toml`

```toml
[dependencies]
arcrtc-core-identity = { path = "../../../Kernel/core/identity" }
arcrtc-core-sfu = { path = "../../../Kernel/core/sfu" }
arcrtc-core-signaling = { path = "../../../Kernel/core/signaling" }
arcrtc-core-turn = { path = "../../../Kernel/core/turn" }
arcrtc-distro-evidence = { path = "../../distro-support/evidence" }
arcrtc-reference-composition = { path = "../../reference-distro/composition" }
arcrtc-reference-ops = { path = "../../reference-distro/ops" }
arcrtc-reference-sfu = { path = "../../reference-distro/sfu" }
arcrtc-reference-signaling = { path = "../../reference-distro/signaling" }
arcrtc-reference-turn = { path = "../../reference-distro/turn" }
```

### `tests/product/Cargo.toml`

```toml
[dependencies]
arcrtc-core-identity = { path = "../../../Kernel/core/identity" }
arcrtc-core-sfu = { path = "../../../Kernel/core/sfu" }
arcrtc-core-signaling = { path = "../../../Kernel/core/signaling" }
arcrtc-core-turn = { path = "../../../Kernel/core/turn" }
arcrtc-distro-evidence = { path = "../../distro-support/evidence" }
arcrtc-reference-output = { path = "../../reference-distro/output" }
arcrtc-product-deployment = { path = "../../product-distro/deployment" }
arcrtc-product-monitoring = { path = "../../product-distro/monitoring" }
arcrtc-product-persistence-topology = { path = "../../product-distro/persistence-topology" }
arcrtc-product-policy = { path = "../../product-distro/product-policy" }
arcrtc-product-rollback = { path = "../../product-distro/rollback" }
arcrtc-product-sfu = { path = "../../product-distro/sfu" }
arcrtc-product-signaling = { path = "../../product-distro/signaling" }
arcrtc-product-turn = { path = "../../product-distro/turn" }
```

### `tests/benchmark/Cargo.toml`

```toml
[dependencies]
arcrtc-core-identity = { path = "../../../Kernel/core/identity" }
arcrtc-core-sfu = { path = "../../../Kernel/core/sfu" }
arcrtc-core-signaling = { path = "../../../Kernel/core/signaling" }
arcrtc-core-turn = { path = "../../../Kernel/core/turn" }
arcrtc-distro-evidence = { path = "../../distro-support/evidence" }
arcrtc-product-deployment = { path = "../../product-distro/deployment" }
arcrtc-product-persistence-topology = { path = "../../product-distro/persistence-topology" }
arcrtc-product-policy = { path = "../../product-distro/product-policy" }
arcrtc-product-rollback = { path = "../../product-distro/rollback" }
arcrtc-reference-composition = { path = "../../reference-distro/composition" }
arcrtc-reference-ops = { path = "../../reference-distro/ops" }
arcrtc-reference-sfu = { path = "../../reference-distro/sfu" }
arcrtc-reference-signaling = { path = "../../reference-distro/signaling" }
arcrtc-reference-turn = { path = "../../reference-distro/turn" }
criterion = "0.5"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

`tests/benchmark/Cargo.toml` must also include the following.

```toml
[[bench]]
name = "benchmark_scenarios"
harness = false
```

### `tests/real-device/Cargo.toml`

```toml
[dependencies]
arcrtc-distro-evidence = { path = "../../distro-support/evidence" }
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

### `tests/production-readiness/Cargo.toml`

```toml
[dependencies]
arcrtc-core-identity = { path = "../../../Kernel/core/identity" }
arcrtc-distro-evidence = { path = "../../distro-support/evidence" }
arcrtc-product-deployment = { path = "../../product-distro/deployment" }
arcrtc-product-monitoring = { path = "../../product-distro/monitoring" }
arcrtc-product-persistence-topology = { path = "../../product-distro/persistence-topology" }
arcrtc-product-policy = { path = "../../product-distro/product-policy" }
arcrtc-product-rollback = { path = "../../product-distro/rollback" }
serde_json = "1"
```

### `tests/live/Cargo.toml`

```toml
[dependencies]
arcrtc-core-identity = { path = "../../../Kernel/core/identity" }
arcrtc-distro-evidence = { path = "../../distro-support/evidence" }
arcrtc-product-deployment = { path = "../../product-distro/deployment" }
arcrtc-product-monitoring = { path = "../../product-distro/monitoring" }
arcrtc-product-rollback = { path = "../../product-distro/rollback" }
arcrtc-product-sfu = { path = "../../product-distro/sfu" }
arcrtc-product-signaling = { path = "../../product-distro/signaling" }
arcrtc-product-turn = { path = "../../product-distro/turn" }
serde_json = "1"
```

## 14. Collapse Conditions (invariants and fail-closed conditions)

The authority of this chapter collapses if any of the following occur.

- a test package is counted as distro package completion.
- a test pass is treated as production readiness or live readiness.
- a test assertion depends on a Kernel private field.
- a test uses a random fixture without a deterministic source.
- a test accepts an error reason outside the closed set.
- a `tests/*` package is added to the distro workspace root members.
- the empty `[workspace]` table is removed from a `tests/*` package manifest, reverting to a state where Cargo misrecognizes it as a parent workspace member.
- a test pass is treated as a distro completion / readiness claim.
- a test package directly imports a Kernel crate outside the dependency admission matrix.
- a real-device command is fixed as a package command without a manifest path.
- `criterion` is added to a package other than the benchmark package.
- production readiness / live readiness is re-merged into a single test package owner.
