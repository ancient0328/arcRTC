# Chapter 09 product implementation

Status: SSOT consolidated edition
Date: 2026-06-28 JST

## Purpose

This chapter fixes, at a granularity sufficient for reproducible implementation, the product implementation layer (the product implementation domain outside the Kernel) of the arcRTC v0.2 implementations domain: its owned surfaces, required package shape, construction order, non-inheritance rules from reference, product API policy, the exact API signature of each plane (fully enumerated), product policy decision rules, persistence topology, deployment / runtime profile mapping, production provider admission, live endpoint traversal admission, monitoring / evidence, rollback / drain, fail-closed rules, and readiness boundary. This chapter is fully self-contained and is understandable without consulting other documents or source code. It references only other chapter numbers within this same specification.

product implementation is not the Kernel. product implementation MAY use the Kernel frozen contract. product implementation MUST NOT own Kernel semantics. product implementation MUST NOT fork / copy the reference implementation. product implementation MUST consume only the 3-type allow-list `ReferenceSignalingOutcome` / `ReferenceTurnOutcome` / `ReferenceSfuOutcome` of `arcrtc-reference-output` at the product boundary.

---

## 1. Owned surfaces

product implementation MUST own the following surfaces.

| Surface | Owns |
|---|---|
| `product-implementation/signaling/` | product Signaling runtime composition |
| `product-implementation/turn/` | product TURN runtime composition |
| `product-implementation/sfu/` | product SFU runtime composition |
| `product-implementation/product-policy/` | product-specific policy (tenant / quota / admission / operational policy) |
| `product-implementation/persistence-topology/` | product persistence topology |
| `product-implementation/deployment/` | deployment packaging / environment profile |
| `product-implementation/monitoring/` | SLO / metrics / logs / probes / operational probes |
| `product-implementation/rollback/` | rollback / drain / restore operation |

The decision class product implementation MAY own is limited to product policy. The owner relation between Kernel semantics and product policy is fixed as follows.

| Decision class | Owner |
|---|---|
| protocol semantics | Kernel |
| routing / allocation / signaling acceptability | Kernel |
| reason catalog | Kernel |
| tenant / quota / deployment policy | product implementation |
| persistence topology | product implementation |
| operational SLO | product implementation |
| rollback / drain | product implementation |

## 2. Non-ownership

product implementation MUST NOT own the following.

- Kernel semantic authority
- Kernel completion / freeze claim
- Kernel final completion authority
- Kernel contract modification
- reference implementation completion claim
- reference source / internal state / fixture / local auth / runtime / composition ownership

## 3. Required package shape

product implementation packages MUST follow the implementations workspace members. Each package MUST hold the following file contract.

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

`ProductEvidenceRecord` MUST be limited to a public alias / re-export of `arcrtc_implementation_evidence::ImplementationEvidenceRecord`. product implementation MUST NOT define an independent evidence record struct.

## 4. Construction order (required direction) and reference dependency

product implementation MUST be built after `ReferenceSignalingOutcome` / `ReferenceTurnOutcome` / `ReferenceSfuOutcome` of `arcrtc-reference-output` are fixed as product input. `ReferenceCompositionOutcome`, reference package public API, reference state, reference function, fixture, local auth, runtime, and composition symbol are not product input (MUST NOT consume). product implementation MUST NOT inherit the reference implementation success claim.

product implementation MUST be built in the following order.

1. product policy
2. product Signaling
3. product TURN
4. product SFU
5. persistence topology
6. deployment
7. monitoring
8. rollback / drain

product admission (the admission to proceed from reference to product) MUST be treated as product scaffold admission, not a reference completion claim. Admission proceeds in the following order.

1. `ReferenceSignalingOutcome` / `ReferenceTurnOutcome` / `ReferenceSfuOutcome` are fixed as product input.
2. The reference composition boundary is fixed.
3. The Kernel contract consumption boundary is fixed.
4. The product-owned surfaces are fixed.
5. The product scaffold scope is separated from the reference scope.
6. The product policy / persistence / deployment / monitoring / rollback owners are separated.

product admission MUST NOT claim the following: reference implementation completion / product implementation completion / production readiness / live readiness / benchmark threshold satisfaction / public distribution readiness.

## 5. Product plane API (public API and return boundary per package)

The public API of each product plane package MUST be fixed as follows.

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

### 5.1 Shared product rule

The public API naming of product plane packages MUST be fixed as follows.

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

Every success outcome MUST include non-claim scope. No product outcome MUST imply production readiness or live readiness.

## 6. Product Signaling API (exact signature)

| Type / Function | File | Required shape |
|---|---|---|
| `ProductSignalingRuntime` | `product-implementation/signaling/src/kernel_contract.rs` | `target_plane: ImplementationPlane`, `public_endpoint_claimed: bool`, `live_endpoint_evidence_ref: Option<String>`, `implementation_reason: ImplementationEvidenceReason` |
| `ProductSignalingPolicyInput` | `product-implementation/signaling/src/kernel_contract.rs` | `correlation_id: CorrelationId`, `reference_outcome: arcrtc_reference_output::ReferenceSignalingOutcome`, `policy_decision: ProductPolicyDecision` |
| `ProductSignalingOutcome` | `product-implementation/signaling/src/kernel_contract.rs` | `correlation_id: CorrelationId`, `allowed: bool`, `implementation_reason: ImplementationEvidenceReason`, `non_claim_scope: Vec<ImplementationNonClaimScope>` |
| `build_product_signaling_runtime` | `product-implementation/signaling/src/kernel_contract.rs` | `() -> ProductSignalingRuntime` |
| `build_live_product_signaling_runtime` | `product-implementation/signaling/src/kernel_contract.rs` | `(&str) -> Result<ProductSignalingRuntime, ProductSignalingError>` |
| `apply_product_signaling_policy` | `product-implementation/signaling/src/kernel_contract.rs` | `(&ProductSignalingPolicyInput) -> Result<ProductSignalingOutcome, ProductSignalingError>` |

`build_product_signaling_runtime` MUST set `public_endpoint_claimed = false`. `build_live_product_signaling_runtime` MUST reject empty live endpoint evidence ref and MUST set `public_endpoint_claimed = true` only on success.

## 7. Product TURN API (exact signature)

| Type / Function | File | Required shape |
|---|---|---|
| `ProductTurnRuntime` | `product-implementation/turn/src/kernel_contract.rs` | `target_plane: ImplementationPlane`, `relay_public_endpoint_claimed: bool`, `live_endpoint_evidence_ref: Option<String>`, `implementation_reason: ImplementationEvidenceReason` |
| `ProductTurnPolicyInput` | `product-implementation/turn/src/kernel_contract.rs` | `correlation_id: CorrelationId`, `reference_outcome: arcrtc_reference_output::ReferenceTurnOutcome`, `policy_decision: ProductPolicyDecision` |
| `ProductTurnOutcome` | `product-implementation/turn/src/kernel_contract.rs` | `correlation_id: CorrelationId`, `allowed: bool`, `implementation_reason: ImplementationEvidenceReason`, `non_claim_scope: Vec<ImplementationNonClaimScope>` |
| `build_product_turn_runtime` | `product-implementation/turn/src/kernel_contract.rs` | `() -> ProductTurnRuntime` |
| `build_live_product_turn_runtime` | `product-implementation/turn/src/kernel_contract.rs` | `(&str) -> Result<ProductTurnRuntime, ProductTurnError>` |
| `apply_product_turn_policy` | `product-implementation/turn/src/kernel_contract.rs` | `(&ProductTurnPolicyInput) -> Result<ProductTurnOutcome, ProductTurnError>` |

`build_product_turn_runtime` MUST set `relay_public_endpoint_claimed = false`. `build_live_product_turn_runtime` MUST reject empty live endpoint evidence ref and MUST set `relay_public_endpoint_claimed = true` only on success.

## 8. Product SFU API (exact signature)

| Type / Function | File | Required shape |
|---|---|---|
| `ProductSfuRuntime` | `product-implementation/sfu/src/kernel_contract.rs` | `target_plane: ImplementationPlane`, `media_public_endpoint_claimed: bool`, `live_endpoint_evidence_ref: Option<String>`, `implementation_reason: ImplementationEvidenceReason` |
| `ProductSfuPolicyInput` | `product-implementation/sfu/src/kernel_contract.rs` | `correlation_id: CorrelationId`, `reference_outcome: arcrtc_reference_output::ReferenceSfuOutcome`, `policy_decision: ProductPolicyDecision` |
| `ProductSfuOutcome` | `product-implementation/sfu/src/kernel_contract.rs` | `correlation_id: CorrelationId`, `allowed: bool`, `implementation_reason: ImplementationEvidenceReason`, `non_claim_scope: Vec<ImplementationNonClaimScope>` |
| `build_product_sfu_runtime` | `product-implementation/sfu/src/kernel_contract.rs` | `() -> ProductSfuRuntime` |
| `build_live_product_sfu_runtime` | `product-implementation/sfu/src/kernel_contract.rs` | `(&str) -> Result<ProductSfuRuntime, ProductSfuError>` |
| `apply_product_sfu_policy` | `product-implementation/sfu/src/kernel_contract.rs` | `(&ProductSfuPolicyInput) -> Result<ProductSfuOutcome, ProductSfuError>` |

`build_product_sfu_runtime` MUST set `media_public_endpoint_claimed = false`. `build_live_product_sfu_runtime` MUST reject empty live endpoint evidence ref and MUST set `media_public_endpoint_claimed = true` only on success.

## 9. Product policy API and decision rules

| Type / Function | File | Required shape |
|---|---|---|
| `ProductPolicyInput` | `product-implementation/product-policy/src/auth_policy.rs` | `correlation_id: CorrelationId`, `target_plane: ImplementationPlane`, `fixture_identity: Option<String>`, `requested_action: ProductAction` |
| `ProductPolicyDecision` | `product-implementation/product-policy/src/auth_policy.rs` | `allowed: bool`, `implementation_reason: ImplementationEvidenceReason`, `non_claim_scope: Vec<ImplementationNonClaimScope>` |
| `ProductAction` | `product-implementation/product-policy/src/auth_policy.rs` | enum `Join`, `Relay`, `Publish`, `Subscribe`, `Observe`, `Drain`, `Restore` |
| `evaluate_product_auth_policy` | `product-implementation/product-policy/src/auth_policy.rs` | `(&ProductPolicyInput) -> Result<ProductPolicyDecision, ProductPolicyError>` |
| `map_product_security_reason` | `product-implementation/product-policy/src/security_reason.rs` | `(ProductPolicyError) -> ImplementationEvidenceReason` |

Decision rule:

| Input condition | Result |
|---|---|
| `fixture_identity.is_some()` | `allowed = true`, reason `IMPLEMENTATION_OK` |
| `fixture_identity.is_none()` | `allowed = false`, reason `FIXTURE_IDENTITY_INVALID` |
| requested readiness claim while readiness is not admitted | error `ProductPolicyError::ReadinessNotAdmitted` |

Every `allowed = true` decision MUST include `ProductionReadinessNotClaimed` and `LiveReadinessNotClaimed` in `non_claim_scope`. `ProductPolicyDecision.allowed == true` MUST NOT imply production readiness. `ProductPolicyDecision.allowed == false` MUST use a closed `ImplementationEvidenceReason`. `ProductPolicyDecision.allowed` MUST NOT be treated as Kernel authorization semantics.

## 10. Persistence topology types

| Type | File | Required fields |
|---|---|---|
| `ProductPersistenceTopology` | `product-implementation/persistence-topology/src/topology.rs` | `mode: ProductPersistenceMode`, `record_classes: Vec<ProductPersistenceRecordClass>` |
| `ProductPersistenceMode` | `product-implementation/persistence-topology/src/topology.rs` | enum `NotAdmitted`, `InMemoryProjectionOnly`, `ProviderDeferred` |
| `ProductPersistenceRecordClass` | `product-implementation/persistence-topology/src/topology.rs` | enum `SessionProjection`, `AllocationProjection`, `RouteProjection`, `EvidenceProjection` |
| `ProductProjectionMapping` | `product-implementation/persistence-topology/src/mapper.rs` | `source_plane: ImplementationPlane`, `record_class: ProductPersistenceRecordClass`, `implementation_reason: ImplementationEvidenceReason` |

The initial product persistence mode is `InMemoryProjectionOnly`. Provider-specific schema MUST NOT be admitted until a provider is admitted by the provider admission rule.

## 11. Deployment / runtime types and runtime profile mapping

| Type | File | Required fields |
|---|---|---|
| `ProductRuntimeProfile` | `product-implementation/deployment/src/profile.rs` | `profile_name: &'static str`, `host_class: ProductHostClass`, `environment_class: ImplementationEnvironmentClass`, `public_endpoint_claimed: bool` |
| `ProductHostClass` | `product-implementation/deployment/src/profile.rs` | enum `LocalSingleHost`, `ControlledMultiProcess`, `ProductionDeferred`, `ProductionAdmitted`, `LiveDeferred`, `LiveAdmitted` |
| `ProductRuntimeSelection` | `product-implementation/deployment/src/runtime.rs` | `profile: ProductRuntimeProfile`, `implementation_reason: ImplementationEvidenceReason` |
| `ProductRuntimeOutcome` | `product-implementation/deployment/src/runtime.rs` | `correlation_id`, `state`, `implementation_reason`, `non_claim_scope` |

`build_product_runtime_profile` maps `ProductHostClass` to `ProductRuntimeProfile` fields as follows. `select_product_runtime` then returns the listed `ProductRuntimeSelection.implementation_reason`.

| ProductHostClass | profile_name | environment_class | public_endpoint_claimed | selection implementation_reason |
|---|---|---|---|---|
| `LocalSingleHost` | `product-local-single-host` | `LocalSingleHost` | `false` | `ImplementationOk` |
| `ControlledMultiProcess` | `product-controlled-multi-process` | `ControlledProcess` | `false` | `ImplementationOk` |
| `ProductionDeferred` | `product-production-deferred` | `ProductionDeferred` | `false` | `ReadinessNotAdmitted` |
| `ProductionAdmitted` | `product-production-admitted` | `ProductionDeferred` | `false` | `ImplementationOk` |
| `LiveDeferred` | `product-live-deferred` | `LiveDeferred` | `false` | `ReadinessNotAdmitted` |
| `LiveAdmitted` | `product-live-admitted` | `LiveDeferred` | `true` | `ImplementationOk` |

If `ProductRuntimeProfile.host_class` and `environment_class` do not match this table, `select_product_runtime` MUST return `ProductRuntimeError::ReadinessNotAdmitted`. No profile other than `ProductHostClass::LiveAdmitted` (admitted by live endpoint traversal authority) MUST set `public_endpoint_claimed = true`. Product runtime lifecycle outcome is not command evidence. Build / test command evidence MUST be generated only by the product monitoring evidence helper.

Additional API signatures for deployment / persistence / monitoring / rollback:

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

## 12. Production provider admission types

| Type / Function | File | Required boundary |
|---|---|---|
| `ProductAuthProviderClass`, `ProductAuthProviderAdmission`, `admit_product_auth_provider` | `product-implementation/product-policy/src/provider_admission.rs` | auth provider admission only |
| `ProductPersistenceProviderClass`, `ProductPersistenceProviderAdmission`, `admit_product_persistence_provider` | `product-implementation/persistence-topology/src/provider_admission.rs` | persistence provider admission only |
| `build_product_production_profile` | `product-implementation/deployment/src/production_profile.rs` | production runtime profile only |
| `ProductProductionMonitoringProbe`, `build_production_monitoring_probe` | `product-implementation/monitoring/src/production_probe.rs` | production monitoring probe only |
| `plan_production_drain`, `plan_production_restore` | `product-implementation/rollback/src/production_operation.rs` | production drain / restore plan only |

A product auth provider or persistence provider MUST NOT be introduced without provider admission.

## 13. Live endpoint traversal types

| Type / Function | File | Required boundary |
|---|---|---|
| `ProductLiveEndpointClass`, `ProductLiveEndpointAdmission`, `admit_product_live_endpoint`, `build_product_live_profile` | `product-implementation/deployment/src/live_endpoint.rs` | live endpoint admission only |
| `ProductPublicTraversalClass`, `ProductPublicTraversalAdmission`, `admit_public_traversal` | `product-implementation/deployment/src/live_endpoint.rs` | public traversal admission only |
| `ProductLiveMonitoringProbe`, `build_live_monitoring_probe` | `product-implementation/monitoring/src/live_probe.rs` | live monitoring probe only |
| `execute_live_shutdown_drain`, `execute_live_restore` | `product-implementation/rollback/src/live_operation.rs` | live drain / restore operation only |
| `build_live_product_signaling_runtime` | `product-implementation/signaling/src/kernel_contract.rs` | Signaling public endpoint branch with admitted live endpoint evidence ref only |
| `build_live_product_turn_runtime` | `product-implementation/turn/src/kernel_contract.rs` | TURN public endpoint branch with admitted live endpoint evidence ref only |
| `build_live_product_sfu_runtime` | `product-implementation/sfu/src/kernel_contract.rs` | SFU public endpoint branch with admitted live endpoint evidence ref only |

`public_endpoint_claimed` is false until live endpoint traversal authority admits it through `ProductHostClass::LiveAdmitted` (MUST). A live endpoint admission / public traversal admission / live operation source MUST NOT be introduced without traversal authority.

## 14. Monitoring / evidence types

| Type | File | Required fields |
|---|---|---|
| `ProductObservabilityRecord` | `product-implementation/monitoring/src/observability.rs` | `correlation_id: CorrelationId`, `target_plane: ImplementationPlane`, `metric_name: &'static str`, `implementation_reason: ImplementationEvidenceReason` |
| `ProductEvidenceRecord` | `product-implementation/monitoring/src/evidence.rs` | `pub type ProductEvidenceRecord = ImplementationEvidenceRecord` |

Monitoring record success is not production readiness (MUST NOT be treated as such). `build_product_evidence_record` MUST reject records matching any of the following.

- `implementation_layer` is not `Product`
- `command_class` is not `Build` or `Test`
- `target_plane` is `Composition` or `Ops`
- `target_package` is not an `arcrtc-product-*` package
- `target_scope` is outside `product-implementation/`

## 15. Rollback / drain types

| Type | File | Required fields |
|---|---|---|
| `ProductDrainPlan` | `product-implementation/rollback/src/drain.rs` | `correlation_id: CorrelationId`, `planes: Vec<ImplementationPlane>`, `mode: ProductDrainMode`, `implementation_reason: ImplementationEvidenceReason` |
| `ProductDrainMode` | `product-implementation/rollback/src/drain.rs` | enum `ReferenceLocal`, `ControlledProduct`, `ProductionDeferred`, `LiveAdmitted` |
| `ProductRestorePlan` | `product-implementation/rollback/src/restore.rs` | `correlation_id: CorrelationId`, `source: ProductRestoreSource`, `implementation_reason: ImplementationEvidenceReason` |
| `ProductRestoreSource` | `product-implementation/rollback/src/restore.rs` | enum `InMemoryProjection`, `EvidenceReport`, `ProviderDeferred`, `LiveEvidenceReport` |

`ProviderDeferred` MUST fail closed for production readiness claims.

## 16. Fail-closed rule

| Failure | Required error / reason |
|---|---|
| policy decision missing | `STATE_BOUNDARY_VIOLATION` |
| provider mode `ProviderDeferred` used for readiness without production provider admission | `READINESS_NOT_ADMITTED` |
| runtime profile claims public endpoint without live admission | `READINESS_NOT_ADMITTED` |
| evidence record missing non-claim scope | `EVIDENCE_FIELDS_INCOMPLETE` |

## 17. Readiness boundary

product implementation test MAY demonstrate product behavior. production readiness requires `PRD-001` through `PRD-009` of the readiness matrix (MUST). live readiness requires `LIVE-001` through `LIVE-008` of the readiness matrix and MUST NOT be claimed until the production readiness evidence, live endpoint evidence, public traversal evidence, monitoring probe evidence, rollback / drain execution evidence, shutdown drain evidence, and restore evidence are all in place.

product implementation evidence MUST be layered as follows: 1. product behavior evidence, 2. production readiness evidence, 3. live readiness evidence. Lower evidence MUST NOT automatically prove higher readiness. Each readiness claim MUST be established by its own dedicated evidence.

## 18. Fail-closed / collapse conditions

If any of the following holds, the product implementation specification collapses (treated as fail-closed and not established).

- product implementation includes Kernel source modification as a normal task.
- product test pass is automatically adopted as production readiness.
- production readiness is treated as live readiness.
- live readiness is claimed while the live readiness gate is missing.
- a product auth provider or persistence provider is introduced without provider admission.
- a live endpoint admission / public traversal admission / live operation source is introduced without traversal authority.
- `ProductEvidenceRecord` is defined as a struct independent from the shared `ImplementationEvidenceRecord`.
- a product plane package holds a reference package dependency / reference symbol import other than `arcrtc-reference-output`.
- a product plane treats `ReferenceCompositionOutcome`, reference package public API, reference state, reference function, fixture, local auth, runtime, or composition symbol as product input.
- a product API redefines `ImplementationEvidenceReason` as a package-local enum.
- a product policy input receives a reference package type other than `arcrtc-reference-output`.
- a local product runtime builder claims public endpoint availability.
- a live product runtime builder makes a public endpoint claim without a live endpoint evidence ref.
- `ProviderDeferred` or `LiveDeferred` is treated as success readiness.
- `ProductPolicyDecision.allowed` is treated as Kernel authorization semantics.
