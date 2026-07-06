# 第13章 testing-and-assertions

状態: SSOT 統合版
日付: 2026-06-28 JST

## 目的

本章は、arcRTC v0.2 distro 領域における test package 構成と assertion 規則の完全仕様を固定します。具体的には、test package set（全7 package とその path・name・ownership）、boundary / reference / product / benchmark / real-device / readiness の全 test file と各 assertion、extension validator negative branch rule（benchmark / real-device / readiness の全 validation error と expected reason）、expected failure reason rule、test package manifest shape、manifest path command、test dependency admission、各 test package の `[dependencies]` literal stanza、path rule、そして本領域の testing scope と non-claim を、再現実装可能な粒度で記述します。本章は完全自己完結であり、他文書・実コードを参照せずに理解できます。同一仕様書内の他章番号のみ参照します。

依存規則として、distro は Kernel に対して contract / SDK / command surface のみ依存します。本章は test 実装 shape を定義するものであり、test pass、benchmark pass、real-device success、production readiness、live readiness のいずれも主張しません。close-like claim は test 定義のみを根拠には行いません。

## 1. Testing 領域の役割と non-claim

本章は distro 側 test evidence の定義領域であり、test 実装 shape を定義するものです。test 実装完了、test pass、benchmark pass、production readiness、live readiness を主張しません（禁止）。

## 2. Test Package Set（全7 package）

test package は distro workspace member ではありません。test package manifest、dependency admission、manifest-path command は本章第8〜13節が固定します。

| Package path | Package name | Owns |
|---|---|---|
| `tests/boundary` | `arcrtc-distro-boundary-tests` | docs / dependency / export boundary tests |
| `tests/reference` | `arcrtc-distro-reference-tests` | reference behavior tests |
| `tests/product` | `arcrtc-distro-product-tests` | product boundary tests |
| `tests/benchmark` | `arcrtc-distro-benchmark-tests` | benchmark scenario tests |
| `tests/real-device` | `arcrtc-distro-real-device-tests` | real-device command evidence wrapper tests |
| `tests/production-readiness` | `arcrtc-distro-production-readiness-tests` | production readiness matrix tests |
| `tests/live` | `arcrtc-distro-live-tests` | live readiness matrix tests |

## 3. Boundary Test Files（assertion 全件）

| Test file | Assertions |
|---|---|
| `tests/boundary/tests/docs_authority.rs` | canonical index がすべての現行 canonical file を含む。root index authority order がすべての canonical file を含む。 |
| `tests/boundary/tests/dependency_admission.rs` | package manifest が dependency admission 規則および workspace manifest と dependency path 規則が admit する dependency のみを含む。 |
| `tests/boundary/tests/module_export.rs` | すべての `src/lib.rs` の public module / re-export が module export 規則に一致する。 |
| `tests/boundary/tests/evidence_schema.rs` | evidence JSON record が evidence schema を満たす。`correlation_id` を含むすべての base string field が secret-like または raw-payload marker を含む場合 `RawSecretLikeValue` を返す。 |
| `tests/boundary/tests/command_evidence.rs` | command evidence record の required field、command class、working directory、output directory、non-claim scope、evidence writer owner が command evidence runner 規則および第14章で規定する command CI quality gate 規則に一致する。 |
| `tests/boundary/tests/secret_literal.rs` | source、report の current-authority text が raw secret / credential literal を含まない。redacted または hash-form の device identifier は real-device wrapper command 規則が admit する箇所でのみ受理される。 |
| `tests/boundary/tests/test_package_shape.rs` | test package manifest、package path、manifest-path command、assertion file set が test workspace manifest 規則および本章に一致する。 |
| `tests/boundary/tests/evidence_owner.rs` | shared evidence / reason owner、evidence wire value、non-claim scope closed set、extension evidence owner file が distro evidence と reason ownership 規則、evidence wire format 規則、evidence extension record 規則に一致する。 |
| `tests/boundary/tests/old_vocabulary.rs` | old vocabulary scan が archived material を除き current-authority hit を返さない。 |
| `tests/boundary/tests/product_reference_output_boundary.rs` | product plane package manifest は、full reference output package を product input として扱うためではなく、3-type product input allow-list へのアクセスのためにのみ `arcrtc-reference-output` に依存する。product source は `arcrtc_reference_output` から exact allow-list（product signaling 用 `ReferenceSignalingOutcome`、product TURN 用 `ReferenceTurnOutcome`、product SFU 用 `ReferenceSfuOutcome`）でのみ import する。product source は `arcrtc_reference_output::*`、`arcrtc_reference_output` module path、re-export、type alias path、`ReferenceCompositionOutcome`、`arcrtc_reference_signaling`、`arcrtc_reference_turn`、`arcrtc_reference_sfu`、`arcrtc_reference_composition`、`arcrtc_reference_ops`、`Reference*State`、`ReferenceSfuAction`、`apply_reference_*`、`validate_reference_*`、fixture、local auth、runtime、composition symbol のいずれも import しない。違反は `COMMAND_SCOPE_MISMATCH` に map される。 |

## 4. Reference Test Files（assertion 全件）

| Test file | Assertions |
|---|---|
| `tests/reference/tests/signaling_contract.rs` | `build_kernel_signaling_command` が fixed constructor chain と command type literal を使用する。 |
| `tests/reference/tests/signaling_state.rs` | join / leave / offer / answer / candidate の transition が reference state mutation 規則に一致する。 |
| `tests/reference/tests/signaling_fixture.rs` | offer direction mismatch と answer direction mismatch が `InvalidFixtureIdentity` を返す。 |
| `tests/reference/tests/turn_contract.rs` | `build_kernel_turn_command` が `TurnCommand::try_new` に delegate し constructor failure を map する。 |
| `tests/reference/tests/turn_state.rs` | allocate / refresh / permission / channel bind / relay の transition が mutation table に一致する。 |
| `tests/reference/tests/turn_fixture.rs` | invalid fixture TURN credential が `InvalidFixtureIdentity` を返す。accepted fixture credential は deterministic を維持し product credential authority を生成しない。 |
| `tests/reference/tests/sfu_contract.rs` | `build_kernel_sfu_item` と `build_borrowed_packet_view` が fixed constructor を使用する。 |
| `tests/reference/tests/sfu_state.rs` | `AdmitParticipant`、`RejectParticipant`、`PublishStream`、`SubscribeRoute`、`SelectRoute`、`SuppressForwarding`、`DropForwarding`、`CloseSession` の transition が mutation table に一致する。typed `ReferenceSfuAction` field は missing field として test されない。session absent admission は session / endpoint を生成する。closed / draining session、rejected / removed endpoint、route session / endpoint / stream mismatch、non-selected suppression、dropped / closed route、absent / closed session close は `ReferenceSfuError::StateBoundaryViolation` を返す。 |
| `tests/reference/tests/sfu_fixture.rs` | fixture route admission は deterministic reference-local route authorization のみを受理し、missing session / endpoint / stream / route field を fixture canonical に従い `StateBoundaryViolation` または `InvalidFixtureIdentity` として拒否する。 |
| `tests/reference/tests/borrowed_packet_view.rs` | borrowed packet view builder が raw packet / payload slice を allocation または copy なしに保持し、packet bytes を reference state に格納しない。 |
| `tests/reference/tests/reference_api_signature.rs` | public reference API signature、phase enum、fixture payload、package ownership、product input allow-list boundary が reference API と state 規則、reference phase と fixture payload 規則、reference distro 規則、reference output boundary 規則に一致する。 |
| `tests/reference/tests/profile.rs` | reference local / benchmark profile literal が reference profile 規則に一致し、production または live readiness を含意しない。 |
| `tests/reference/tests/composition.rs` | signaling-to-turn と signaling-to-sfu の binding が既存の Signaling / TURN / SFU state のみを受理し、orphan reference を direct `ReferenceCompositionState` execution で拒否する。 |
| `tests/reference/tests/runtime.rs` | reference runtime start / shutdown state が runtime lifecycle 規則に一致する。shutdown before start が fail-close する。reference ops build evidence は reference ops build scope のみを受理する。 |
| `tests/reference/tests/error_reason.rs` | すべての reference error variant が fixed distro reason に map される。 |

## 5. Product Test Files（assertion 全件）

| Test file | Assertions |
|---|---|
| `tests/product/tests/product_api.rs` | public product API signature が product API policy 規則、product plane API signature 規則、reference output boundary 規則に一致する。product policy input は `arcrtc_reference_output::ReferenceSignalingOutcome`、`ReferenceTurnOutcome`、`ReferenceSfuOutcome` のみを受理する。`ReferenceCompositionOutcome`、wildcard import、module import、re-export、type alias import は product API input surface として拒否される。 |
| `tests/product/tests/product_policy.rs` | product policy decision が fixture identity を allow し、missing identity を `FixtureIdentityInvalid` で deny し、blank fixture identity を reject し、production / live readiness を claim しない。 |
| `tests/product/tests/persistence_topology.rs` | in-memory projection topology が session / allocation / route / evidence projection を admit する。`ProviderDeferred` と `NotAdmitted` が fail-close する。mismatched plane-to-record-class projection が fail-close する。 |
| `tests/product/tests/deployment.rs` | live readiness admission なしでは `public_endpoint_claimed` が false である。 |
| `tests/product/tests/monitoring.rs` | monitoring build / test evidence は product package、product scope、product layer、Build/Test command shape、non-Composition/Ops plane、non-readiness non-claim scope のみを受理する。 |
| `tests/product/tests/rollback.rs` | drain / restore plan は admitted local / in-memory path に対してのみ `DistroOk` を返す。empty drain と production/provider-deferred path は live readiness claim なしの fail-closed distro reason を返す。 |
| `tests/product/tests/error_reason.rs` | すべての product error variant が fixed distro reason に map される。 |

## 6. Benchmark / Real-Device / Readiness Test Files（assertion 全件）

| Test file | Assertions |
|---|---|
| `tests/benchmark/tests/scenario_set.rs` | すべての `BENCH-001`〜`BENCH-020` が存在し workload summary を持つ。 |
| `tests/benchmark/tests/measurement_schema.rs` | benchmark report が benchmark scenario workload 規則が要求する全 field を含む。すべての `BENCH-001`〜`BENCH-020` report が finite positive な `throughput_items_per_second` を持つ。Criterion configuration field が warm up `3`、measurement `10`、sample size `100`、noise threshold `0.05`、confidence level `0.95`、significance level `0.05` に一致する。benchmark extension validator の negative branch が第7節の規則に一致する。 |
| `tests/benchmark/tests/comparison_row.rs` | すべての admitted benchmark comparison row が `scenario_id`、`distro_layer`、`target_plane`、`workload_id`、`metric`、`unit`、`aggregation_rule`、`environment_class`、`toolchain_runtime`、`sample_count`、`warmup_rule`、`timestamp`、`non_claim_scope` を持つ。invalid `scenario_id`、layer/plane mismatch、empty or mismatched `workload_id`、metric mismatch、unit mismatch、empty or mismatched `aggregation_rule`、empty `environment_class`、empty `toolchain_runtime`、source sample-count mismatch、empty or mismatched `warmup_rule`、empty `timestamp`、missing `BenchmarkThresholdNotClaimed` は row を comparison admission から reject する。 |
| `tests/benchmark/tests/harness_layout.rs` | benchmark package layout、Criterion group name、function naming、timed-closure allocation boundary、evidence conversion function が benchmark harness layout 規則に一致する。 |
| `tests/real-device/tests/command_matrix.rs` | すべての real-device evidence id と device class が closed set に含まれる。successful real-device evidence は `redacted_device_identifier` に `redacted` または `sha256:<64 lowercase hex characters>` を使用する。serial / udid / android_id / device_id / imei / meid / account / token / private_key / device_name などの raw identifier marker は `UnsafeDeviceIdentifier` を返す。missing required non-claim scope または missing exit status などの base-validator-first 条件は `Base(error)` を返す。real-device extension validator の negative branch が第7節の規則に一致する。 |
| `tests/real-device/tests/non_claim_scope.rs` | real-device evidence が required non-claim scope を含む。 |
| `tests/real-device/tests/wrapper_command.rs` | real-device wrapper CLI、platform command closed set、post-execution requiredness、exit status mapping、manifest-path invocation が real-device wrapper command 規則に一致する。 |
| `tests/production-readiness/tests/production_matrix.rs` | すべての `PRD-001`〜`PRD-009` gate が存在する。すべての record が `readiness_claim == ProductionReadiness` を持つ。各 gate が第12章で規定する readiness matrix の required extension ref field に一致する。 |
| `tests/production-readiness/tests/fail_closed.rs` | `ReadinessValidationContext.auth_provider_authority == Absent` の `PRD-003` が `READINESS_NOT_ADMITTED` を返す。`ReadinessValidationContext.persistence_provider_authority == Absent` の `PRD-004` が `READINESS_NOT_ADMITTED` を返す。`readiness_claim == ProductionReadiness` で任意の `LIVE-001`〜`LIVE-008` を伴う場合 `READINESS_NOT_ADMITTED` に map される `ReadinessGateClaimMismatch` を返す。production-readiness extension validator の negative branch が第7節の規則に一致する。 |
| `tests/production-readiness/tests/success_matrix.rs` | `PRD-001`〜`PRD-009` success record が gate 固有 ref field を伴う場合にのみ受理される。auth provider admission、persistence provider admission、production profile、production monitoring probe、production drain / restore plan、security scan ref、production readiness Closed Gate ref がすべて concrete source または report file に接続される。 |
| `tests/live/tests/live_matrix.rs` | すべての `LIVE-001`〜`LIVE-008` gate が存在する。すべての record が `readiness_claim == LiveReadiness` を持つ。`LIVE-005` が `rollback_drain_execution_ref` を必須とする。各 gate が第12章で規定する readiness matrix の required extension ref field に一致する。 |
| `tests/live/tests/fail_closed.rs` | `ReadinessValidationContext.live_endpoint_authority == Absent` の任意の live gate が `READINESS_NOT_ADMITTED` を返す。`production_readiness_report == Absent` の `LIVE-001` が `READINESS_NOT_ADMITTED` を返す。`public_traversal_authority == Absent` の `LIVE-003` が `READINESS_NOT_ADMITTED` を返す。`readiness_claim == LiveReadiness` で任意の `PRD-001`〜`PRD-009` を伴う場合 `READINESS_NOT_ADMITTED` に map される `ReadinessGateClaimMismatch` を返す。live-readiness extension validator の negative branch が第7節の規則に一致する。 |
| `tests/live/tests/success_matrix.rs` | `LIVE-001`〜`LIVE-008` success record が gate 固有 ref field を伴う場合にのみ受理される。production readiness report ref、live endpoint admission、public traversal admission、live monitoring probe、live shutdown drain、live restore、Signaling / TURN / SFU live public endpoint branch、live readiness Closed Gate ref がすべて concrete source または report file に接続される。 |

## 7. Extension Validator Negative Branch Rule

各 extension validator test は、evidence wire format に基づく `Base(error)` passthrough を assert しなければなりません（必須）。

### 7.1 Benchmark extension validator negative branch

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

### 7.2 Real-device extension validator negative branch

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

### 7.3 Readiness extension validator negative branch

readiness extension validator の negative branch は、production-readiness fail-closed test と live-readiness fail-closed test の両方によって次の通り assert されます。

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

## 9. Test Workspace Manifest（root rule と manifest shape）

test package は distro workspace member ではありません。test execution は distro 完了条件ではありません。

test package は `distro/` から explicit manifest path で起動します。`tests/*` package は distro workspace root members に追加しません（禁止）。

各 test package の `Cargo.toml` は次を使用します。

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

`[workspace]` は意図的に空です。これは Cargo が `tests/*` の explicit-manifest package を親 `distro` workspace の unlisted member と扱うことを防ぎつつ、test package が distro workspace member ではないという規則を維持します。

## 10. Test Package Commands（manifest path command 全件）

| Package path | Package name | Command from distro root |
|---|---|---|
| `tests/boundary` | `arcrtc-distro-boundary-tests` | `cargo test --manifest-path tests/boundary/Cargo.toml` |
| `tests/reference` | `arcrtc-distro-reference-tests` | `cargo test --manifest-path tests/reference/Cargo.toml` |
| `tests/product` | `arcrtc-distro-product-tests` | `cargo test --manifest-path tests/product/Cargo.toml` |
| `tests/benchmark` | `arcrtc-distro-benchmark-tests` | `cargo test --manifest-path tests/benchmark/Cargo.toml` および `cargo bench --manifest-path tests/benchmark/Cargo.toml --bench benchmark_scenarios` |
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

`criterion` は `arcrtc-distro-benchmark-tests` でのみ admit されます。`serde_json` は evidence JSON parsing / writing が assertion の一部である場合にのみ test package で admit されます。

## 12. Path Rule

`tests/<package>` から distro package への local path dependency は `../../` を使用します。

例:

```toml
arcrtc-distro-evidence = { path = "../../distro-support/evidence" }
```

## 13. Test Package Dependency Literal Stanza

以下の dependency stanza は各 test package の完全な `[dependencies]` section です。fragment ではありません。

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

`tests/benchmark/Cargo.toml` は次も含まなければなりません。

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

## 14. Collapse Conditions（不変条件・fail-closed 条件）

本章の正典は次の場合に崩れます。

- test package を distro package completion として数える。
- test pass を production readiness または live readiness として扱う。
- test assertion が Kernel private field に依存する。
- test が deterministic source なしの random fixture を使う。
- test が closed set 外の error reason を受理する。
- `tests/*` package を distro workspace root member に追加する。
- `tests/*` package manifest から空の `[workspace]` table を削除し、Cargo が parent workspace member と誤認する状態に戻す。
- test pass を distro completion / readiness claim として扱う。
- test package が dependency admission matrix 外の Kernel crate を直接 import する。
- real-device command を manifest path なしの package command として固定する。
- benchmark package 以外に `criterion` を入れる。
- production readiness / live readiness を単一 test package owner に再統合する。
