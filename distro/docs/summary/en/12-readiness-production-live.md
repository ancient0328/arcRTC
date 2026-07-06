# Chapter 12 readiness-production-live

Status: SSOT consolidated edition
Date: 2026-06-28 JST

## Purpose

This chapter fixes the complete specification of readiness (production readiness and live readiness) in the arcRTC v0.2 distro domain. Specifically, it describes, at a granularity sufficient for reproducible re-implementation: the non-derivation principle of readiness; the separation of production readiness and live readiness; the production provider admission surface (separated auth / persistence authority, production runtime profile, monitoring probe, drain / restore plan, security scan); the live endpoint traversal surface (live endpoint admission, public traversal admission, live monitoring probe, live drain / restore execution, product plane public endpoint branch); the readiness evidence record extension fields; failure classification; and the fixed goal of the distro domain. This chapter is fully self-contained and is understandable without consulting other documents, source code, or Kernel documents. It references only other chapter numbers within this same specification.

As a dependency rule, distro depends on the Kernel only through contract / SDK / command surface. Production readiness and live readiness MUST NOT be auto-derived from Kernel evidence; each is established only by its own dedicated readiness surface. Production provider admission and live endpoint traversal own only the product distro and readiness surfaces of distro; they do not own Kernel source, Kernel semantic authority, or (for production provider admission) live endpoint authority.

## 1. Non-Derivation Principle of Readiness

Readiness MUST NOT be established by any one of the following alone.

- source scaffold
- build pass
- test pass
- benchmark result
- real-device command success

Readiness is not admitted by source scaffold, build pass, test pass, benchmark result, or real-device command success alone.

## 2. Readiness Claim Separation

Production readiness and live readiness are separate claims. Production readiness establishes the controlled production identity provider, controlled persistence provider, production runtime profile, production monitoring probe, and production drain / restore plan. Live readiness establishes the controlled public live endpoint, public traversal admission, live monitoring probe, and live rollback / drain / restore execution, and has production readiness as a prerequisite.

| Claim | May depend on | Must not depend on |
|---|---|---|
| production readiness | product build / test evidence, deployment profile, monitoring evidence, rollback plan evidence, provider admission evidence | live endpoint availability alone |
| live readiness | production readiness, live endpoint evidence, public traversal evidence, monitoring evidence, rollback / drain execution evidence, shutdown drain evidence, restore evidence | Kernel completion / freeze alone |

Production readiness success is not live readiness success. Live endpoint availability alone is not production readiness. Kernel completion / freeze alone is not live readiness.

## 3. Production Readiness Required Surfaces

A production-ready system establishes the following surfaces. Each surface is bounded to its own result and does not infer a higher readiness.

| Required surface | Expected boundary |
|---|---|
| product workspace build | build command result only |
| product behavior test | tested product behavior only |
| product auth provider admission | provider boundary only |
| product persistence provider admission | provider boundary only |
| product deployment profile | deployment profile only |
| product monitoring | observability field availability only |
| rollback / drain plan | plan generation only |
| security secret scan | absence of committed secret in scope only |

The auth provider authority and the persistence provider authority are validated separately. They do not share a generic single provider authority. When the auth provider authority is absent, auth provider admission fails closed with `READINESS_NOT_ADMITTED`. When the persistence provider authority is absent, persistence provider admission fails closed with `READINESS_NOT_ADMITTED`.

## 4. Live Readiness Required Surfaces

A live-ready system establishes the following surfaces, with production readiness as a prerequisite.

| Required surface | Expected boundary |
|---|---|
| production readiness | production readiness prerequisite only |
| bounded live endpoint | endpoint command result only |
| public traversal | traversal command result only |
| live monitoring probe | probe command result only |
| live rollback / drain execution | bounded operation result only |
| live shutdown drain | bounded shutdown result only |
| live restore | bounded restore result only |

If live endpoint authority is absent, every live readiness surface fails closed with `READINESS_NOT_ADMITTED`. Live endpoint admission and public traversal admission are separate authorities and MUST NOT be treated as one authority.

## 5. Readiness Evidence Record Fields

A readiness evidence record extends the distro evidence record (the common base record). The extension fields are as follows.

- `readiness_claim`
- `build_evidence_ref`
- `behavior_test_evidence_ref`
- `auth_provider_admission_ref`
- `persistence_provider_admission_ref`
- `deployment_profile_ref`
- `monitoring_probe_ref`
- `rollback_plan_ref`
- `security_scan_ref`
- `production_readiness_report_ref`
- `live_endpoint_evidence_ref`
- `public_traversal_evidence_ref`
- `rollback_drain_execution_ref`
- `shutdown_drain_evidence_ref`
- `restore_evidence_ref`

The `auth_provider_admission_ref` and the `persistence_provider_admission_ref` are separate fields and MUST NOT be merged. No readiness evidence field MUST contain raw secret, raw token, private key, raw packet payload, or direct personal identifier.

## 6. Failure Classification (common to the readiness surfaces)

| Failure | Distro reason |
|---|---|
| readiness surface not admitted | `READINESS_NOT_ADMITTED` |
| required field missing | `EVIDENCE_FIELDS_INCOMPLETE` |
| command scope mismatch | `COMMAND_SCOPE_MISMATCH` |
| auth provider not admitted | `READINESS_NOT_ADMITTED` |
| persistence provider not admitted | `READINESS_NOT_ADMITTED` |
| live endpoint authority absent | `READINESS_NOT_ADMITTED` |

## 7. Production Provider Admission

This section fixes the production provider admission, production profile, monitoring probe, rollback / drain plan, and security scan of the production readiness surface. This admission is limited to the product distro and readiness evidence of distro; it does not own Kernel source, Kernel semantic authority, or live endpoint authority.

### 7.1 Authority Split

The auth provider authority and the persistence provider authority are separate authorities.

| Authority state field | Evidence ref field | Source function |
|---|---|---|
| `ReadinessValidationContext.auth_provider_authority` | `ReadinessEvidenceRecord.auth_provider_admission_ref` | `admit_product_auth_provider` |
| `ReadinessValidationContext.persistence_provider_authority` | `ReadinessEvidenceRecord.persistence_provider_admission_ref` | `admit_product_persistence_provider` |

When `auth_provider_authority == Absent`, auth provider admission fails closed with `AuthProviderAuthorityNotAdmitted`. When `persistence_provider_authority == Absent`, persistence provider admission fails closed with `PersistenceProviderAuthorityNotAdmitted`. A single generic provider authority cannot express the fail-closed state where only one of the two is unadopted.

### 7.2 Auth Provider Admission (source contract)

`product-distro/product-policy/src/provider_admission.rs` defines the following.

| Item | Required shape |
|---|---|
| `ProductAuthProviderClass` | enum `NotAdmitted`, `ControlledProductionIdentity` |
| `ProductAuthProviderAdmission` | `provider_class`, `credential_source`, `token_validation_rule`, `secret_storage_boundary`, `failure_reason_closed_set`, `audit_evidence_boundary`, `distro_reason` |
| `admit_product_auth_provider` | `(ProductAuthProviderClass) -> Result<ProductAuthProviderAdmission, ProductPolicyError>` |

Only `ControlledProductionIdentity` is the admitted authority. `NotAdmitted` returns `ProductPolicyError::ReadinessNotAdmitted`.

### 7.3 Persistence Provider Admission (source contract)

`product-distro/persistence-topology/src/provider_admission.rs` defines the following.

| Item | Required shape |
|---|---|
| `ProductPersistenceProviderClass` | enum `NotAdmitted`, `ControlledProjectionStore` |
| `ProductPersistenceProviderAdmission` | `provider_class`, `record_shape_boundary`, `storage_scope`, `schema_owner`, `failure_reason_closed_set`, `distro_reason` |
| `admit_product_persistence_provider` | `(ProductPersistenceProviderClass) -> Result<ProductPersistenceProviderAdmission, ProductPersistenceTopologyError>` |

Only `ControlledProjectionStore` is the admitted authority. `NotAdmitted` returns `ProductPersistenceTopologyError::ProviderNotAdmitted`.

### 7.4 Production Runtime Profile (source contract)

`product-distro/deployment/src/production_profile.rs` defines the following.

| Item | Required shape |
|---|---|
| `build_product_production_profile` | `() -> ProductRuntimeProfile` |

`ProductHostClass` has `ProductionAdmitted`. `build_product_production_profile()` returns `ProductHostClass::ProductionAdmitted`, `DistroEnvironmentClass::ProductionDeferred`, and `public_endpoint_claimed = false`. `select_product_runtime` treats the `ProductionAdmitted` profile as `DistroOk`. The production profile MUST NOT claim public endpoint availability.

### 7.5 Production Monitoring Probe (source contract)

`product-distro/monitoring/src/production_probe.rs` defines the following.

| Item | Required shape |
|---|---|
| `ProductProductionMonitoringProbe` | `correlation_id`, `target_plane`, `metric_name`, `probe_boundary`, `distro_reason` |
| `build_production_monitoring_probe` | `(CorrelationId, DistroPlane, &'static str) -> Result<ProductProductionMonitoringProbe, ProductMonitoringError>` |

An empty metric name returns `ProductMonitoringError::EvidenceFieldsIncomplete`.

### 7.6 Production Operation Plan (source contract)

`product-distro/rollback/src/production_operation.rs` defines the following.

| Item | Required shape |
|---|---|
| `plan_production_drain` | `(CorrelationId, Vec<DistroPlane>) -> Result<ProductDrainPlan, ProductRollbackError>` |
| `plan_production_restore` | `(CorrelationId) -> Result<ProductRestorePlan, ProductRollbackError>` |

An empty plane drain returns `ProductRollbackError::RuntimeExecutorError`. The restore source is fixed to `ProductRestoreSource::EvidenceReport`. A production drain / restore plan MUST NOT be repurposed as live operation execution.

### 7.7 Production Source Admission (owner package limited)

The production readiness source is limited to the following owners.

| Source | Owner package | Purpose |
|---|---|---|
| `provider_admission.rs` | `arcrtc-product-policy` | auth provider admission |
| `provider_admission.rs` | `arcrtc-product-persistence-topology` | persistence provider admission |
| `production_profile.rs` | `arcrtc-product-deployment` | production runtime profile |
| `production_probe.rs` | `arcrtc-product-monitoring` | production monitoring probe |
| `production_operation.rs` | `arcrtc-product-rollback` | production drain / restore plan |

Existing files MAY only be updated for module registration, enum admission, or delegation. Kernel source modification is forbidden.

### 7.8 Production Provider Admission Failure Classification

| Failure | Classification |
|---|---|
| auth provider authority absent | `READINESS_NOT_ADMITTED` |
| persistence provider authority absent | `READINESS_NOT_ADMITTED` |
| missing required ref | `EVIDENCE_FIELDS_INCOMPLETE` |
| empty required ref | `EVIDENCE_FIELDS_INCOMPLETE` |
| unexpected ref populated | `COMMAND_SCOPE_MISMATCH` |
| secret-like marker in ref | `EVIDENCE_FIELDS_INCOMPLETE` |

### 7.9 Production Provider Admission Non-Claim Scope

Production provider admission does not claim the following: live readiness, public endpoint availability, Kernel completion, Kernel freeze.

## 8. Live Endpoint Traversal

This section fixes the live endpoint admission, public traversal admission, live monitoring probe, live drain / restore operation, and product plane public endpoint branch of the live readiness surface. This section does not own Kernel source, Kernel semantic authority, or Kernel completion / freeze evidence. Production readiness is a prerequisite of live readiness and does not substitute for live endpoint, public traversal, live drain / restore, or live readiness.

### 8.1 Deployment Source Contract

`product-distro/deployment/src/live_endpoint.rs` owns the following.

- `ProductLiveEndpointClass`
- `ProductPublicTraversalClass`
- `ProductLiveEndpointAdmission`
- `ProductPublicTraversalAdmission`
- `admit_product_live_endpoint`
- `admit_public_traversal`
- `build_product_live_profile`

`ProductLiveEndpointClass` is a closed set.

- `NotAdmitted`
- `ControlledPublicEndpoint`

`ProductPublicTraversalClass` is a closed set.

- `NotAdmitted`
- `ControlledPublicTraversal`

`admit_product_live_endpoint(ProductLiveEndpointClass::ControlledPublicEndpoint)` returns `DistroOk`. `admit_product_live_endpoint(ProductLiveEndpointClass::NotAdmitted)` returns `ProductRuntimeError::ReadinessNotAdmitted`.

`admit_public_traversal(ProductPublicTraversalClass::ControlledPublicTraversal)` returns `DistroOk`. `admit_public_traversal(ProductPublicTraversalClass::NotAdmitted)` returns `ProductRuntimeError::ReadinessNotAdmitted`.

`build_product_live_profile(&ProductLiveEndpointAdmission)` returns the following.

- `ProductHostClass::LiveAdmitted`
- `DistroEnvironmentClass::LiveDeferred`
- `public_endpoint_claimed == true`

`select_product_runtime` accepts a public endpoint claim only when the following hold.

- `host_class == ProductHostClass::LiveAdmitted`
- `environment_class == DistroEnvironmentClass::LiveDeferred`
- `public_endpoint_claimed == true`

Every other `public_endpoint_claimed == true` combination MUST fail closed with `ReadinessNotAdmitted`. `ProductHostClass::LiveAdmitted` is used limited to the live profile generated from `ProductLiveEndpointAdmission`.

### 8.2 Monitoring Source Contract

`product-distro/monitoring/src/live_probe.rs` owns the following.

- `ProductLiveMonitoringProbe`
- `build_live_monitoring_probe`

`build_live_monitoring_probe` requires a non-empty `metric_name` and returns `ProductMonitoringError::EvidenceFieldsIncomplete` for empty metrics. A successful live monitoring probe uses `DistroOk`.

### 8.3 Rollback Source Contract

`product-distro/rollback/src/live_operation.rs` owns the following.

- `execute_live_shutdown_drain`
- `execute_live_restore`

`execute_live_shutdown_drain` requires a non-empty plane list and returns `ProductRollbackError::RuntimeExecutorError` for an empty list. A successful live shutdown drain uses `DistroOk`. `execute_live_restore` returns `DistroOk` when invoked with a correlation id. Live operation source is placed in the rollback owner package and MUST NOT be mixed into deployment / monitoring / plane packages.

### 8.4 Product Plane Public Endpoint Branch

The product plane owner files own public endpoint branch functions.

| package | function | branch input |
|---|---|---|
| `product-distro/signaling` | `build_live_product_signaling_runtime` | admitted live endpoint evidence ref |
| `product-distro/turn` | `build_live_product_turn_runtime` | admitted live endpoint evidence ref |
| `product-distro/sfu` | `build_live_product_sfu_runtime` | admitted live endpoint evidence ref |

Each function rejects an empty or whitespace-only live endpoint evidence ref with `ReadinessNotAdmitted`. Each successful function sets the plane-specific public endpoint flag to `true` and uses `DistroOk`. Existing local runtime builders keep public endpoint flags `false`. A Signaling / TURN / SFU public endpoint branch MUST NOT be enabled without an admitted live endpoint evidence ref.

### 8.5 Live Endpoint Traversal Non-Claim Scope

This live endpoint traversal does not claim the following: live readiness alone, Kernel completion, Kernel freeze.

## 9. Fixed Goal of the distro Domain

The fixed goal of distro is to fully and entirely achieve production readiness, live readiness, benchmark threshold satisfaction, real-device success, and Kernel completion / freeze. This fixed goal MUST NOT adopt a pragmatic compromise, a reinterpretation, or a reduction to a limited claim. The frozen Kernel is consumed without distro-side Kernel source mutation; a Kernel contract insufficiency is fail-closed and requires a Kernel-side versioned authority first.

The fixed goal connects the following independent evidence surfaces.

- The distro domain uses only admitted Kernel crates and performs no Kernel source mutation.
- The reference distro executes Signaling / TURN / SFU / composition behavior via the Kernel contract.
- The product distro executes product Signaling / TURN / SFU / policy / persistence / deployment / monitoring / rollback surfaces without owning Kernel semantics.
- A local controlled profile can start, execute a bounded command flow, shut down, and reject an invalid lifecycle order.
- The benchmark surface executes actual workload runners and emits measurement records, and benchmark threshold satisfaction is judged by pre-adopted threshold values and comparison rules (Chapter 10).
- Real-device success is proven by bounded device-class command execution evidence across the required device class set, not by wrapper-only fixture success (Chapter 11).
- Production readiness establishes its required surfaces and is established only by the dedicated production readiness surface, not by benchmark result, product behavior evidence, real-device result, or Kernel evidence alone.
- Live readiness establishes its required surfaces with production readiness as a prerequisite, and is established only by the dedicated live readiness surface, not by production readiness alone, benchmark result, real-device fixture, or Kernel evidence alone.
- Kernel completion / freeze is evidenced without distro-side Kernel source mutation.

The only final claim admitted by this fixed goal is the following.

```text
distro satisfies production readiness, live readiness, benchmark threshold satisfaction, real-device success, and Kernel completion / freeze, while consuming the frozen Kernel without distro-side Kernel mutation.
```

## 10. Collapse Conditions (invariants and fail-closed conditions)

The authority of this chapter collapses if any of the following occur.

- product test pass is used as production readiness.
- production readiness is used as live readiness without live evidence.
- a benchmark or real-device command result is used as readiness by itself.
- provider-deferred state is treated as admitted provider state.
- readiness evidence includes secret, token, private key, or raw packet payload.
- auth provider admission and persistence provider admission are treated as one generic provider authority.
- `auth_provider_admission_ref` and `persistence_provider_admission_ref` are merged.
- a `NotAdmitted` provider class is treated as success.
- the production profile claims public endpoint availability.
- monitoring probe success is repurposed as live readiness.
- a production drain / restore plan is repurposed as live operation execution.
- production readiness is claimed without running a security scan.
- production readiness success is treated as live readiness success.
- live endpoint admission and public traversal admission are treated as one authority.
- a Signaling / TURN / SFU public endpoint branch is enabled without a live endpoint evidence ref.
- live operation source is mixed into deployment, monitoring, Signaling, TURN, or SFU owner packages.
- Kernel source modification is used to satisfy a distro readiness surface.
