# 第09章 product implementation

状態: SSOT 統合版
日付: 2026-06-28 JST

## 目的

本章は、arcRTC v0.2 implementations 領域における product implementation（Kernel 外の製品実装層）の所有 surface、required package shape、構築順序、reference からの非継承規則、product API policy、各 plane の exact API signature（全列挙）、product policy decision 規則、persistence topology、deployment / runtime profile mapping、production provider admission、live endpoint traversal admission、monitoring / evidence、rollback / drain、fail-closed 規則、readiness boundary を、再現実装可能な粒度で固定します。本章は完全自己完結であり、他文書・実コードを参照せずに理解できます。同一仕様書内の他章番号のみ参照します。

product implementation は Kernel ではありません。product implementation は Kernel frozen contract を利用します（MAY）。product implementation は Kernel semantics を所有しません（MUST NOT）。product implementation は reference implementation を fork / copy しません（MUST NOT）。product implementation は `arcrtc-reference-output` の `ReferenceSignalingOutcome` / `ReferenceTurnOutcome` / `ReferenceSfuOutcome` の 3-type allow-list だけを product boundary で消費します（MUST）。

---

## 1. 所有 surface（owned surface）

product implementation は次の surface を所有します（MUST）。

| Surface | Owns |
|---|---|
| `product-implementation/signaling/` | product Signaling runtime composition |
| `product-implementation/turn/` | product TURN runtime composition |
| `product-implementation/sfu/` | product SFU runtime composition |
| `product-implementation/product-policy/` | product-specific policy（tenant / quota / admission / operational policy） |
| `product-implementation/persistence-topology/` | product persistence topology |
| `product-implementation/deployment/` | deployment packaging / environment profile |
| `product-implementation/monitoring/` | SLO / metrics / logs / probes / operational probes |
| `product-implementation/rollback/` | rollback / drain / restore operation |

product implementation が所有できる decision class は product policy に限定します（MUST）。Kernel semantics と product policy の owner 関係は次に固定します。

| Decision class | Owner |
|---|---|
| protocol semantics | Kernel |
| routing / allocation / signaling acceptability | Kernel |
| reason catalog | Kernel |
| tenant / quota / deployment policy | product implementation |
| persistence topology | product implementation |
| operational SLO | product implementation |
| rollback / drain | product implementation |

## 2. 非所有（non-ownership）

product implementation は次を所有しません（MUST NOT）。

- Kernel semantic authority
- Kernel completion / freeze claim
- Kernel final completion authority
- Kernel contract modification
- reference implementation completion claim
- reference source / internal state / fixture / local auth / runtime / composition ownership

## 3. required package shape

product implementation package は implementations workspace members に従います（MUST）。各 package は次の file contract を持ちます（MUST）。

| Package | Required files |
|---|---|
| `arcrtc-product-signaling` | `src/lib.rs`, `src/kernel_contract.rs`, `src/error.rs` |
| `arcrtc-product-turn` | `src/lib.rs`, `src/kernel_contract.rs`, `src/error.rs` |
| `arcrtc-product-sfu` | `src/lib.rs`, `src/kernel_contract.rs`, `src/error.rs` |
| `arcrtc-product-policy` | `src/lib.rs`, `src/auth_policy.rs`, `src/provider_admission.rs`, `src/security_reason.rs`, `src/error.rs` |
| `arcrtc-product-persistence-topology` | `src/lib.rs`, `src/topology.rs`, `src/mapper.rs`, `src/provider_admission.rs`, `src/error.rs` |
| `arcrtc-product-deployment` | `src/lib.rs`, `src/runtime.rs`, `src/profile.rs`, `src/production_profile.rs`, `src/live_endpoint.rs`, `src/error.rs` |
| `arcrtc-product-monitoring` | `src/lib.rs`, `src/observability.rs`, `src/evidence.rs`, `src/production_probe.rs`, `src/live_probe.rs`, `src/error.rs` |
| `arcrtc-product-rollback` | `src/lib.rs`, `src/drain.rs`, `src/restore.rs`, `src/production_operation.rs`, `src/live_operation.rs`, `src/error.rs` |

`ProductEvidenceRecord` は `arcrtc_implementation_evidence::ImplementationEvidenceRecord` の public alias / re-export に限定します（MUST）。product implementation は independent evidence record struct を定義しません（MUST NOT）。

## 4. 構築順序（required direction）と reference 依存

product implementation は、`arcrtc-reference-output` の `ReferenceSignalingOutcome` / `ReferenceTurnOutcome` / `ReferenceSfuOutcome` を product input として固定した後に構築します（MUST）。`ReferenceCompositionOutcome`、reference package public API、reference state、reference function、fixture、local auth、runtime、composition symbol は product input ではありません（MUST NOT consume）。product implementation は reference implementation の success claim を継承しません（MUST NOT）。

product implementation は次の順に構築します（MUST）。

1. product policy
2. product Signaling
3. product TURN
4. product SFU
5. persistence topology
6. deployment
7. monitoring
8. rollback / drain

product admission（reference から product へ進む許可）は reference completion claim ではなく product scaffold admission として扱います（MUST）。admission は次の順で行います。

1. `ReferenceSignalingOutcome` / `ReferenceTurnOutcome` / `ReferenceSfuOutcome` が product input として固定されている。
2. reference composition boundary が固定されている。
3. Kernel contract consumption boundary が固定されている。
4. product-owned surfaces が固定されている。
5. product scaffold scope が reference scope から分離されている。
6. product policy / persistence / deployment / monitoring / rollback owner が分離されている。

product admission は次を主張しません（MUST NOT）。reference implementation completion / product implementation completion / production readiness / live readiness / benchmark threshold satisfaction / public distribution readiness。

## 5. product plane API（package ごとの public API と return boundary）

各 product plane package の public API は次に固定します（MUST）。

| Package | Required public API | Return boundary |
|---|---|---|
| `arcrtc-product-signaling` | `build_product_signaling_runtime`, `build_live_product_signaling_runtime`, `apply_product_signaling_policy` | product signaling outcome only |
| `arcrtc-product-turn` | `build_product_turn_runtime`, `build_live_product_turn_runtime`, `apply_product_turn_policy` | product TURN outcome only |
| `arcrtc-product-sfu` | `build_product_sfu_runtime`, `build_live_product_sfu_runtime`, `apply_product_sfu_policy` | product SFU outcome only |
| `arcrtc-product-policy` | `evaluate_product_auth_policy`, `map_product_security_reason` | product policy decision only |
| `arcrtc-product-persistence-topology` | `build_persistence_topology`, `map_product_projection` | topology / projection only |
| `arcrtc-product-deployment` | `build_product_runtime_profile`, `select_product_runtime` | deployment profile only |
| `arcrtc-product-monitoring` | `build_observability_record`, `build_product_evidence_record` | observability / evidence only |
| `arcrtc-product-rollback` | `plan_drain`, `plan_restore` | rollback / drain plan only |
| `arcrtc-product-policy` | `admit_product_auth_provider` | production auth provider admission only |
| `arcrtc-product-persistence-topology` | `admit_product_persistence_provider` | production persistence provider admission only |
| `arcrtc-product-deployment` | `build_product_production_profile` | production deployment profile only |
| `arcrtc-product-monitoring` | `build_production_monitoring_probe` | production monitoring probe only |
| `arcrtc-product-rollback` | `plan_production_drain`, `plan_production_restore` | production operation plan only |
| `arcrtc-product-deployment` | `admit_product_live_endpoint`, `admit_public_traversal`, `build_product_live_profile` | live endpoint / traversal admission only |
| `arcrtc-product-monitoring` | `build_live_monitoring_probe` | live monitoring probe only |
| `arcrtc-product-rollback` | `execute_live_shutdown_drain`, `execute_live_restore` | live drain / restore operation only |

### 5.1 shared product rule

product plane package の public API naming は次の形に固定します（MUST）。

| API class | Required naming | Return boundary |
|---|---|---|
| runtime builder | `build_product_{plane}_runtime(...)` | product-local runtime descriptor |
| live runtime builder | `build_live_product_{plane}_runtime(...)` | product-local runtime descriptor with admitted live endpoint evidence ref |
| policy application | `apply_product_{plane}_policy(...)` | product-local outcome |
| product policy | `evaluate_product_auth_policy(...)` | product policy decision only |
| projection mapping | `map_product_projection(...)` | product persistence projection only |
| deployment selection | `select_product_runtime(...)` | runtime selection only |
| monitoring evidence | `build_product_evidence_record(...)` | implementation evidence only |
| rollback / drain | `plan_drain(...)`, `plan_restore(...)` | plan only |

すべての success outcome は non-claim scope を含みます（MUST）。どの product outcome も production readiness / live readiness を含意しません（MUST NOT）。

## 6. product Signaling API（exact signature）

| Type / Function | File | Required shape |
|---|---|---|
| `ProductSignalingRuntime` | `product-implementation/signaling/src/kernel_contract.rs` | `target_plane: ImplementationPlane`, `public_endpoint_claimed: bool`, `live_endpoint_evidence_ref: Option<String>`, `implementation_reason: ImplementationEvidenceReason` |
| `ProductSignalingPolicyInput` | `product-implementation/signaling/src/kernel_contract.rs` | `correlation_id: CorrelationId`, `reference_outcome: arcrtc_reference_output::ReferenceSignalingOutcome`, `policy_decision: ProductPolicyDecision` |
| `ProductSignalingOutcome` | `product-implementation/signaling/src/kernel_contract.rs` | `correlation_id: CorrelationId`, `allowed: bool`, `implementation_reason: ImplementationEvidenceReason`, `non_claim_scope: Vec<ImplementationNonClaimScope>` |
| `build_product_signaling_runtime` | `product-implementation/signaling/src/kernel_contract.rs` | `() -> ProductSignalingRuntime` |
| `build_live_product_signaling_runtime` | `product-implementation/signaling/src/kernel_contract.rs` | `(&str) -> Result<ProductSignalingRuntime, ProductSignalingError>` |
| `apply_product_signaling_policy` | `product-implementation/signaling/src/kernel_contract.rs` | `(&ProductSignalingPolicyInput) -> Result<ProductSignalingOutcome, ProductSignalingError>` |

`build_product_signaling_runtime` は `public_endpoint_claimed = false` を設定します（MUST）。`build_live_product_signaling_runtime` は空の live endpoint evidence ref を拒否し、success 時のみ `public_endpoint_claimed = true` を設定します（MUST）。

## 7. product TURN API（exact signature）

| Type / Function | File | Required shape |
|---|---|---|
| `ProductTurnRuntime` | `product-implementation/turn/src/kernel_contract.rs` | `target_plane: ImplementationPlane`, `relay_public_endpoint_claimed: bool`, `live_endpoint_evidence_ref: Option<String>`, `implementation_reason: ImplementationEvidenceReason` |
| `ProductTurnPolicyInput` | `product-implementation/turn/src/kernel_contract.rs` | `correlation_id: CorrelationId`, `reference_outcome: arcrtc_reference_output::ReferenceTurnOutcome`, `policy_decision: ProductPolicyDecision` |
| `ProductTurnOutcome` | `product-implementation/turn/src/kernel_contract.rs` | `correlation_id: CorrelationId`, `allowed: bool`, `implementation_reason: ImplementationEvidenceReason`, `non_claim_scope: Vec<ImplementationNonClaimScope>` |
| `build_product_turn_runtime` | `product-implementation/turn/src/kernel_contract.rs` | `() -> ProductTurnRuntime` |
| `build_live_product_turn_runtime` | `product-implementation/turn/src/kernel_contract.rs` | `(&str) -> Result<ProductTurnRuntime, ProductTurnError>` |
| `apply_product_turn_policy` | `product-implementation/turn/src/kernel_contract.rs` | `(&ProductTurnPolicyInput) -> Result<ProductTurnOutcome, ProductTurnError>` |

`build_product_turn_runtime` は `relay_public_endpoint_claimed = false` を設定します（MUST）。`build_live_product_turn_runtime` は空の live endpoint evidence ref を拒否し、success 時のみ `relay_public_endpoint_claimed = true` を設定します（MUST）。

## 8. product SFU API（exact signature）

| Type / Function | File | Required shape |
|---|---|---|
| `ProductSfuRuntime` | `product-implementation/sfu/src/kernel_contract.rs` | `target_plane: ImplementationPlane`, `media_public_endpoint_claimed: bool`, `live_endpoint_evidence_ref: Option<String>`, `implementation_reason: ImplementationEvidenceReason` |
| `ProductSfuPolicyInput` | `product-implementation/sfu/src/kernel_contract.rs` | `correlation_id: CorrelationId`, `reference_outcome: arcrtc_reference_output::ReferenceSfuOutcome`, `policy_decision: ProductPolicyDecision` |
| `ProductSfuOutcome` | `product-implementation/sfu/src/kernel_contract.rs` | `correlation_id: CorrelationId`, `allowed: bool`, `implementation_reason: ImplementationEvidenceReason`, `non_claim_scope: Vec<ImplementationNonClaimScope>` |
| `build_product_sfu_runtime` | `product-implementation/sfu/src/kernel_contract.rs` | `() -> ProductSfuRuntime` |
| `build_live_product_sfu_runtime` | `product-implementation/sfu/src/kernel_contract.rs` | `(&str) -> Result<ProductSfuRuntime, ProductSfuError>` |
| `apply_product_sfu_policy` | `product-implementation/sfu/src/kernel_contract.rs` | `(&ProductSfuPolicyInput) -> Result<ProductSfuOutcome, ProductSfuError>` |

`build_product_sfu_runtime` は `media_public_endpoint_claimed = false` を設定します（MUST）。`build_live_product_sfu_runtime` は空の live endpoint evidence ref を拒否し、success 時のみ `media_public_endpoint_claimed = true` を設定します（MUST）。

## 9. product policy API と decision 規則

| Type / Function | File | Required shape |
|---|---|---|
| `ProductPolicyInput` | `product-implementation/product-policy/src/auth_policy.rs` | `correlation_id: CorrelationId`, `target_plane: ImplementationPlane`, `fixture_identity: Option<String>`, `requested_action: ProductAction` |
| `ProductPolicyDecision` | `product-implementation/product-policy/src/auth_policy.rs` | `allowed: bool`, `implementation_reason: ImplementationEvidenceReason`, `non_claim_scope: Vec<ImplementationNonClaimScope>` |
| `ProductAction` | `product-implementation/product-policy/src/auth_policy.rs` | enum `Join`, `Relay`, `Publish`, `Subscribe`, `Observe`, `Drain`, `Restore` |
| `evaluate_product_auth_policy` | `product-implementation/product-policy/src/auth_policy.rs` | `(&ProductPolicyInput) -> Result<ProductPolicyDecision, ProductPolicyError>` |
| `map_product_security_reason` | `product-implementation/product-policy/src/security_reason.rs` | `(ProductPolicyError) -> ImplementationEvidenceReason` |

decision 規則:

| Input condition | Result |
|---|---|
| `fixture_identity.is_some()` | `allowed = true`, reason `IMPLEMENTATION_OK` |
| `fixture_identity.is_none()` | `allowed = false`, reason `FIXTURE_IDENTITY_INVALID` |
| requested readiness claim while readiness is not admitted | error `ProductPolicyError::ReadinessNotAdmitted` |

すべての `allowed = true` decision は `non_claim_scope` に `ProductionReadinessNotClaimed` と `LiveReadinessNotClaimed` を含みます（MUST）。`ProductPolicyDecision.allowed == true` は production readiness を含意しません（MUST NOT）。`ProductPolicyDecision.allowed == false` は closed な `ImplementationEvidenceReason` を使います（MUST）。`ProductPolicyDecision.allowed` を Kernel authorization semantics として扱いません（MUST NOT）。

## 10. persistence topology types

| Type | File | Required fields |
|---|---|---|
| `ProductPersistenceTopology` | `product-implementation/persistence-topology/src/topology.rs` | `mode: ProductPersistenceMode`, `record_classes: Vec<ProductPersistenceRecordClass>` |
| `ProductPersistenceMode` | `product-implementation/persistence-topology/src/topology.rs` | enum `NotAdmitted`, `InMemoryProjectionOnly`, `ProviderDeferred` |
| `ProductPersistenceRecordClass` | `product-implementation/persistence-topology/src/topology.rs` | enum `SessionProjection`, `AllocationProjection`, `RouteProjection`, `EvidenceProjection` |
| `ProductProjectionMapping` | `product-implementation/persistence-topology/src/mapper.rs` | `source_plane: ImplementationPlane`, `record_class: ProductPersistenceRecordClass`, `implementation_reason: ImplementationEvidenceReason` |

初期 product persistence mode は `InMemoryProjectionOnly` です。provider 固有の schema は provider admission 規則で provider が admit されるまで admit しません（MUST NOT）。

## 11. deployment / runtime types と runtime profile mapping

| Type | File | Required fields |
|---|---|---|
| `ProductRuntimeProfile` | `product-implementation/deployment/src/profile.rs` | `profile_name: &'static str`, `host_class: ProductHostClass`, `environment_class: ImplementationEnvironmentClass`, `public_endpoint_claimed: bool` |
| `ProductHostClass` | `product-implementation/deployment/src/profile.rs` | enum `LocalSingleHost`, `ControlledMultiProcess`, `ProductionDeferred`, `ProductionAdmitted`, `LiveDeferred`, `LiveAdmitted` |
| `ProductRuntimeSelection` | `product-implementation/deployment/src/runtime.rs` | `profile: ProductRuntimeProfile`, `implementation_reason: ImplementationEvidenceReason` |
| `ProductRuntimeOutcome` | `product-implementation/deployment/src/runtime.rs` | `correlation_id`, `state`, `implementation_reason`, `non_claim_scope` |

`build_product_runtime_profile` は `ProductHostClass` を `ProductRuntimeProfile` field へ次のように mapping します。`select_product_runtime` は listed の `ProductRuntimeSelection.implementation_reason` を返します。

| ProductHostClass | profile_name | environment_class | public_endpoint_claimed | selection implementation_reason |
|---|---|---|---|---|
| `LocalSingleHost` | `product-local-single-host` | `LocalSingleHost` | `false` | `ImplementationOk` |
| `ControlledMultiProcess` | `product-controlled-multi-process` | `ControlledProcess` | `false` | `ImplementationOk` |
| `ProductionDeferred` | `product-production-deferred` | `ProductionDeferred` | `false` | `ReadinessNotAdmitted` |
| `ProductionAdmitted` | `product-production-admitted` | `ProductionDeferred` | `false` | `ImplementationOk` |
| `LiveDeferred` | `product-live-deferred` | `LiveDeferred` | `false` | `ReadinessNotAdmitted` |
| `LiveAdmitted` | `product-live-admitted` | `LiveDeferred` | `true` | `ImplementationOk` |

`ProductRuntimeProfile.host_class` と `environment_class` がこの表に一致しない場合、`select_product_runtime` は `ProductRuntimeError::ReadinessNotAdmitted` を返します（MUST）。`ProductHostClass::LiveAdmitted`（live endpoint traversal authority による admit）以外の profile は `public_endpoint_claimed = true` を設定しません（MUST NOT）。product runtime lifecycle outcome は command evidence ではありません。build / test command evidence は product monitoring evidence helper だけが生成します（MUST）。

deployment / persistence / monitoring / rollback の追加 API signature:

| Type / Function | File | Required shape |
|---|---|---|
| `build_persistence_topology` | `product-implementation/persistence-topology/src/topology.rs` | `(ProductPersistenceMode) -> Result<ProductPersistenceTopology, ProductPersistenceTopologyError>` |
| `map_product_projection` | `product-implementation/persistence-topology/src/mapper.rs` | `(ImplementationPlane, ProductPersistenceRecordClass) -> Result<ProductProjectionMapping, ProductPersistenceTopologyError>` |
| `build_product_runtime_profile` | `product-implementation/deployment/src/profile.rs` | `(ProductHostClass) -> ProductRuntimeProfile` |
| `build_product_live_profile` | `product-implementation/deployment/src/live_endpoint.rs` | `(&ProductLiveEndpointAdmission) -> Result<ProductRuntimeProfile, ProductRuntimeError>` |
| `select_product_runtime` | `product-implementation/deployment/src/runtime.rs` | `(&ProductRuntimeProfile) -> ProductRuntimeSelection` |
| `build_observability_record` | `product-implementation/monitoring/src/observability.rs` | `(CorrelationId, ImplementationPlane, &'static str) -> ProductObservabilityRecord` |
| `build_live_monitoring_probe` | `product-implementation/monitoring/src/live_probe.rs` | `(CorrelationId, ImplementationPlane, &'static str) -> Result<ProductLiveMonitoringProbe, ProductMonitoringError>` |
| `build_product_evidence_record` | `product-implementation/monitoring/src/evidence.rs` | `(ImplementationEvidenceRecord) -> Result<ProductEvidenceRecord, ProductMonitoringError>` |
| `plan_drain` | `product-implementation/rollback/src/drain.rs` | `(CorrelationId, Vec<ImplementationPlane>, ProductDrainMode) -> ProductDrainPlan` |
| `plan_restore` | `product-implementation/rollback/src/restore.rs` | `(CorrelationId, ProductRestoreSource) -> ProductRestorePlan` |
| `execute_live_shutdown_drain` | `product-implementation/rollback/src/live_operation.rs` | `(CorrelationId, Vec<ImplementationPlane>) -> Result<ProductDrainPlan, ProductRollbackError>` |
| `execute_live_restore` | `product-implementation/rollback/src/live_operation.rs` | `(CorrelationId) -> Result<ProductRestorePlan, ProductRollbackError>` |

## 12. production provider admission types

| Type / Function | File | Required boundary |
|---|---|---|
| `ProductAuthProviderClass`, `ProductAuthProviderAdmission`, `admit_product_auth_provider` | `product-implementation/product-policy/src/provider_admission.rs` | auth provider admission only |
| `ProductPersistenceProviderClass`, `ProductPersistenceProviderAdmission`, `admit_product_persistence_provider` | `product-implementation/persistence-topology/src/provider_admission.rs` | persistence provider admission only |
| `build_product_production_profile` | `product-implementation/deployment/src/production_profile.rs` | production runtime profile only |
| `ProductProductionMonitoringProbe`, `build_production_monitoring_probe` | `product-implementation/monitoring/src/production_probe.rs` | production monitoring probe only |
| `plan_production_drain`, `plan_production_restore` | `product-implementation/rollback/src/production_operation.rs` | production drain / restore plan only |

product auth provider または persistence provider を provider admission なしに導入しません（MUST NOT）。

## 13. live endpoint traversal types

| Type / Function | File | Required boundary |
|---|---|---|
| `ProductLiveEndpointClass`, `ProductLiveEndpointAdmission`, `admit_product_live_endpoint`, `build_product_live_profile` | `product-implementation/deployment/src/live_endpoint.rs` | live endpoint admission only |
| `ProductPublicTraversalClass`, `ProductPublicTraversalAdmission`, `admit_public_traversal` | `product-implementation/deployment/src/live_endpoint.rs` | public traversal admission only |
| `ProductLiveMonitoringProbe`, `build_live_monitoring_probe` | `product-implementation/monitoring/src/live_probe.rs` | live monitoring probe only |
| `execute_live_shutdown_drain`, `execute_live_restore` | `product-implementation/rollback/src/live_operation.rs` | live drain / restore operation only |
| `build_live_product_signaling_runtime` | `product-implementation/signaling/src/kernel_contract.rs` | Signaling public endpoint branch with admitted live endpoint evidence ref only |
| `build_live_product_turn_runtime` | `product-implementation/turn/src/kernel_contract.rs` | TURN public endpoint branch with admitted live endpoint evidence ref only |
| `build_live_product_sfu_runtime` | `product-implementation/sfu/src/kernel_contract.rs` | SFU public endpoint branch with admitted live endpoint evidence ref only |

`public_endpoint_claimed` は `ProductHostClass::LiveAdmitted` を通じて live endpoint traversal authority が admit するまで false です（MUST）。live endpoint admission / public traversal admission / live operation source を traversal authority なしに導入しません（MUST NOT）。

## 14. monitoring / evidence types

| Type | File | Required fields |
|---|---|---|
| `ProductObservabilityRecord` | `product-implementation/monitoring/src/observability.rs` | `correlation_id: CorrelationId`, `target_plane: ImplementationPlane`, `metric_name: &'static str`, `implementation_reason: ImplementationEvidenceReason` |
| `ProductEvidenceRecord` | `product-implementation/monitoring/src/evidence.rs` | `pub type ProductEvidenceRecord = ImplementationEvidenceRecord` |

monitoring record success は production readiness ではありません（MUST NOT 扱い）。`build_product_evidence_record` は次のいずれかに一致する record を拒否します（MUST）。

- `implementation_layer` が `Product` でない
- `command_class` が `Build` または `Test` でない
- `target_plane` が `Composition` または `Ops` である
- `target_package` が `arcrtc-product-*` package でない
- `target_scope` が `product-implementation/` の外にある

## 15. rollback / drain types

| Type | File | Required fields |
|---|---|---|
| `ProductDrainPlan` | `product-implementation/rollback/src/drain.rs` | `correlation_id: CorrelationId`, `planes: Vec<ImplementationPlane>`, `mode: ProductDrainMode`, `implementation_reason: ImplementationEvidenceReason` |
| `ProductDrainMode` | `product-implementation/rollback/src/drain.rs` | enum `ReferenceLocal`, `ControlledProduct`, `ProductionDeferred`, `LiveAdmitted` |
| `ProductRestorePlan` | `product-implementation/rollback/src/restore.rs` | `correlation_id: CorrelationId`, `source: ProductRestoreSource`, `implementation_reason: ImplementationEvidenceReason` |
| `ProductRestoreSource` | `product-implementation/rollback/src/restore.rs` | enum `InMemoryProjection`, `EvidenceReport`, `ProviderDeferred`, `LiveEvidenceReport` |

`ProviderDeferred` は production readiness claim に対して fail-closed でなければなりません（MUST）。

## 16. fail-closed 規則

| Failure | Required error / reason |
|---|---|
| policy decision missing | `STATE_BOUNDARY_VIOLATION` |
| provider mode `ProviderDeferred` used for readiness without production provider admission | `READINESS_NOT_ADMITTED` |
| runtime profile claims public endpoint without live admission | `READINESS_NOT_ADMITTED` |
| evidence record missing non-claim scope | `EVIDENCE_FIELDS_INCOMPLETE` |

## 17. readiness boundary

product implementation test は product behavior を示し得ます（MAY）。production readiness は readiness matrix の `PRD-001` through `PRD-009` を必要とします（MUST）。live readiness は readiness matrix の `LIVE-001` through `LIVE-008` を必要とし、production readiness evidence、live endpoint evidence、public traversal evidence、monitoring probe evidence、rollback / drain execution evidence、shutdown drain evidence、restore evidence が揃うまで主張しません（MUST NOT）。

product implementation evidence は次を階層化します（MUST）。1. product behavior evidence、2. production readiness evidence、3. live readiness evidence。下位 evidence は上位 readiness を自動証明しません（MUST NOT）。各 readiness claim は専用の evidence で成立させます（MUST）。

## 18. fail-closed / collapse 条件

次のいずれかに該当する場合、product implementation 仕様は崩れます（fail-closed で不成立として扱います）。

- product implementation が Kernel source modification を通常 task として含む。
- product test pass を production readiness として自動採用する。
- production readiness を live readiness として扱う。
- live readiness gate を欠いたまま live readiness を主張する。
- product auth provider または persistence provider を provider admission なしに導入する。
- live endpoint admission / public traversal admission / live operation source を traversal authority なしに導入する。
- `ProductEvidenceRecord` を shared `ImplementationEvidenceRecord` から独立した struct として定義する。
- product plane package が `arcrtc-reference-output` 以外の reference package dependency / reference symbol import を持つ。
- product plane が `ReferenceCompositionOutcome`、reference package public API、reference state、reference function、fixture、local auth、runtime、composition symbol を product input として扱う。
- product API が `ImplementationEvidenceReason` を package-local enum として再定義する。
- product policy input が `arcrtc-reference-output` 以外の reference package 型を受け取る。
- local product runtime builder が public endpoint availability を主張する。
- live product runtime builder が live endpoint evidence ref なしに public endpoint claim を立てる。
- `ProviderDeferred` または `LiveDeferred` を success readiness として扱う。
- `ProductPolicyDecision.allowed` を Kernel authorization semantics として扱う。
