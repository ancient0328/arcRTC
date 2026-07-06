# Evidence Reason and Observability

Status: SSOT consolidated edition
Date: 2026-06-28 JST

## Purpose

This chapter fixes, at a granularity sufficient for re-implementation, the evidence reason ownership, observability evidence reason, error reason mapping, evidence schema, evidence wire format, and evidence extension record of the arcRTC v0.2 distro domain (the non-Kernel distro domain). This chapter is fully self-contained; it permits only references to chapter numbers within this specification, and it can be understood and re-implemented without opening any external file, Kernel document, or source code.

The dependency rule is `distro -> Kernel` (contract / SDK / command surface only). distro MUST NOT own the Kernel reason catalog. distro owns the evidence reason owner of its own domain and treats Kernel reason as an auxiliary reference field.

---

## 1. Evidence Reason Ownership

### 1.1 Ownership Principle

The Rust type ownership of the evidence reason, evidence record, command class, layer, plane, environment class, and non-claim scope shared across distro is fixed within this domain. These belong to a shared owner that is biased toward neither the reference domain nor the product domain.

This ownership MUST NOT own the Kernel reason catalog. A distro-local evidence reason MAY reference a Kernel reason as auxiliary, but it MUST NOT be promoted to a Kernel reason.

### 1.2 Owner Package

The single Rust owner package for the shared evidence / reason types is fixed as follows.

| Path | Package name | Owns |
|---|---|---|
| `distro-support/evidence` | `arcrtc-distro-evidence` | distro evidence reason / record / validation / readiness extension / common labels / non-claim scope |

`arcrtc-reference-ops` and `arcrtc-product-monitoring` MAY own evidence helpers / writers. However, the type authority of `DistroEvidenceReason` and `DistroEvidenceRecord` MUST reside in `arcrtc-distro-evidence`.

### 1.3 Required Files

`distro-support/evidence` MUST hold the following file contract.

| File | Role |
|---|---|
| `src/lib.rs` | public export |
| `src/reason.rs` | `DistroEvidenceReason` closed enum |
| `src/record.rs` | `DistroEvidenceRecord`, closed label enums, `DistroNonClaimScope` |
| `src/readiness_extension.rs` | `ReadinessClaim`, `ReadinessAdmissionState`, `ReadinessValidationContext`, `ReadinessEvidenceRecord`, `ReadinessEvidenceValidationError`, `validate_readiness_evidence_record` |
| `src/validation.rs` | evidence record validation function |
| `src/error.rs` | typed error independent of validation / writer |

### 1.4 Public Export

`distro-support/evidence/src/lib.rs` is fixed to the following export.

```rust
pub mod error;
pub mod reason;
pub mod record;
pub mod readiness_extension;
pub mod validation;

pub use error::DistroEvidenceError;
pub use readiness_extension::{validate_readiness_evidence_record, ReadinessAdmissionState, ReadinessClaim, ReadinessEvidenceRecord, ReadinessEvidenceValidationError, ReadinessValidationContext};
pub use reason::DistroEvidenceReason;
pub use record::{
    DistroCommandClass, DistroEnvironmentClass, DistroEvidenceRecord,
    DistroLayer, DistroNonClaimScope, DistroPlane,
};
pub use validation::{
    validate_evidence_record, EvidenceValidationError, DISTRO_COMMAND_ROOT,
    DISTRO_EVIDENCE_ROOT, DISTRO_TARGET_ROOT,
};
```

### 1.5 Record / Readiness Extension Ownership

`DistroEvidenceRecord` MUST match the canonical Rust type in Section 4 of this chapter. `non_claim_scope` is fixed to `Vec<DistroNonClaimScope>` and MUST NOT be entered as a free string vector.

The type authority of `ReadinessClaim`, `ReadinessAdmissionState`, `ReadinessValidationContext`, `ReadinessEvidenceRecord`, `ReadinessEvidenceValidationError`, and `validate_readiness_evidence_record` is limited to `distro-support/evidence/src/readiness_extension.rs`.

`arcrtc-reference-ops/src/evidence.rs` and `arcrtc-product-monitoring/src/evidence.rs` MUST be limited to one of the following.

1. Re-export the types of `arcrtc-distro-evidence`.
2. Receive the types of `arcrtc-distro-evidence` and implement a JSON writer / helper.

They MUST NOT redefine an independent enum / struct of the same name.

### 1.6 Dependency Rule

A package that uses `DistroEvidenceReason` in a field, return type, or error mapping MUST hold a local path dependency on `arcrtc-distro-evidence`.

`arcrtc-distro-evidence` MUST NOT depend on a Kernel crate. When referencing a Kernel reason, it MUST be confined to a `String` / `Option<String>` field, and the Kernel reason type MUST NOT be imported into the support package.

### 1.7 Failure Mapping Rule

`distro_reason(&self) -> DistroEvidenceReason` on each distro-local error enum MUST return `arcrtc_distro_evidence::DistroEvidenceReason`. A package-local copy, an alternate authority via type alias, and redefinition of a same-name enum are forbidden (MUST NOT).

### 1.8 Serialization Rule

`DistroEvidenceRecord` is the canonical serialization source of JSON evidence. The serialization of closed enums is fixed to the wire format in Section 5 of this chapter.

The `serde` dependency MAY be introduced limited to `arcrtc-distro-evidence` as the canonical evidence record / closed enum serialization owner. Non-evidence local serialization such as product profile / persistence topology / deployment config is limited to packages separately admitted. File writes via `serde_json` MUST be limited to the evidence writer owner package.

---

## 2. Observability / Evidence Reason Boundary

### 2.1 Reason Boundary

Kernel reason belongs to the Kernel semantic authority. distro MAY reference a Kernel reason as a `kernel_reason` field, but MUST NOT add to or change the Kernel reason catalog. `kernel_reason` does not transfer ownership of the Kernel reason catalog to distro.

However, a `kernel_reason` adopted into official evidence MUST NOT carry an unclassified sentinel. `UNKNOWN`, `Unknown`, `unknown`, and `Other` MUST NOT be adopted as a `kernel_reason` either.

The distro evidence reason is treated as the `distro_reason` field. The Rust type owner follows Section 1 of this chapter and is fixed to `distro-support/evidence/src/reason.rs`.

### 2.2 Reason Class Ownership

| Reason class | Owner | Rule |
|---|---|---|
| Kernel reason | Kernel | not changed; referenced as a source field |
| distro evidence reason | distro | defined as a closed set |
| command failure reason | command / CI surface | defined per command class |
| benchmark reason | benchmark surface | defined per benchmark scenario |
| real-device reason | real-device surface | defined per command / device / environment |

`UNKNOWN` reason MUST NOT be adopted into official evidence. Unclassified output is diagnostic and MUST NOT be made the basis on which an official report is established.

### 2.3 Observability Ownership

What an observability surface MAY own:

- correlation id propagation
- structured log field
- metric name / unit
- trace span name
- command evidence artifact
- benchmark evidence artifact
- distro evidence reason mapping

What an observability surface MUST NOT own:

- Kernel semantics
- Kernel reason catalog mutation
- product readiness claim
- live readiness claim
- production SLO

### 2.4 Distro Evidence Reason Closed Set

The distro evidence reason is fixed to the following closed set (12 items). `UNKNOWN` is forbidden as an official evidence reason (MUST NOT).

| Reason | Meaning |
|---|---|
| `DISTRO_OK` | expected distro command / behavior succeeded |
| `KERNEL_CONTRACT_UNAVAILABLE` | required Kernel public contract is absent |
| `KERNEL_CONTRACT_MISMATCH` | imported Kernel contract shape does not match the distro contract shape fixed in this specification |
| `DEPENDENCY_NOT_ADMITTED` | dependency lacks admission record |
| `RUNTIME_EXECUTOR_ERROR` | runtime executor failed within bounded command |
| `STATE_BOUNDARY_VIOLATION` | state / persistence boundary was violated |
| `FIXTURE_IDENTITY_INVALID` | fixture identity / credential was invalid |
| `EVIDENCE_FIELDS_INCOMPLETE` | required evidence fields are missing |
| `COMMAND_SCOPE_MISMATCH` | command working directory / target scope mismatch |
| `BENCHMARK_SCOPE_MISMATCH` | benchmark workload / environment scope mismatch |
| `REAL_DEVICE_SCOPE_MISMATCH` | real-device command scope mismatch |
| `READINESS_NOT_ADMITTED` | readiness claim lacks its required admission record |

### 2.5 Required Observability Fields

logs / metrics / traces / reports MUST hold the following fields.

| Field | Required use |
|---|---|
| `correlation_id` | connects command / report / span |
| `distro_layer` | `DistroLayer` wire value closed set: `reference` / `product` / `benchmark` / `real_device` / `readiness` |
| `target_plane` | `DistroPlane` closed set (Section 4 of this chapter) |
| `kernel_reason` | set only when referencing a Kernel reason |
| `distro_reason` | set from the closed set |
| `non_claim_scope` | non-claim scope of readiness / completion |

### 2.6 Required Files (observability)

| Package | File | Role |
|---|---|---|
| `arcrtc-distro-evidence` | `src/reason.rs` | distro evidence reason enum owner |
| `arcrtc-distro-evidence` | `src/record.rs` | common evidence record / label enum owner |
| `arcrtc-reference-ops` | `src/evidence.rs` | reference command evidence struct |
| `arcrtc-reference-ops` | `src/reason.rs` | shared reason re-export / closed-set helper |
| `arcrtc-product-monitoring` | `src/observability.rs` | product observability field mapping |
| `arcrtc-product-monitoring` | `src/evidence.rs` | product evidence struct |

### 2.7 Required Evidence Fields

An evidence record MUST hold the following fields.

- correlation id
- command
- working directory
- target scope
- expected outcome
- actual outcome
- environment / toolchain
- rerun condition
- closed reason classification
- non-claim scope

Output that does not satisfy this field set is diagnostic.

---

## 3. Error / Reason Mapping

### 3.1 Shared Error Rule

Each distro-local error enum MUST satisfy the following.

- Has no `Unknown` / `Other` / raw `String` variant.
- Does not make a raw dependency error a public variant.
- Has `distro_reason(&self) -> arcrtc_distro_evidence::DistroEvidenceReason`.
- `Display` is limited to diagnostic text and is not adopted as a claim reason.
- The `std::error::Error` distro is optional, but implementing it does not replace the evidence reason.

### 3.2 Reference Error Mapping

| Error enum | Required variants | DistroEvidenceReason |
|---|---|---|
| `ReferenceSignalingError` | `KernelContractUnavailable` | `KERNEL_CONTRACT_UNAVAILABLE` |
| `ReferenceSignalingError` | `KernelContractMismatch` | `KERNEL_CONTRACT_MISMATCH` |
| `ReferenceSignalingError` | `InvalidFixtureIdentity` | `FIXTURE_IDENTITY_INVALID` |
| `ReferenceSignalingError` | `StateBoundaryViolation` | `STATE_BOUNDARY_VIOLATION` |
| `ReferenceSignalingError` | `EvidenceFieldsIncomplete` | `EVIDENCE_FIELDS_INCOMPLETE` |
| `ReferenceTurnError` | `KernelContractUnavailable` | `KERNEL_CONTRACT_UNAVAILABLE` |
| `ReferenceTurnError` | `KernelContractMismatch` | `KERNEL_CONTRACT_MISMATCH` |
| `ReferenceTurnError` | `InvalidFixtureCredential` | `FIXTURE_IDENTITY_INVALID` |
| `ReferenceTurnError` | `StateBoundaryViolation` | `STATE_BOUNDARY_VIOLATION` |
| `ReferenceTurnError` | `EvidenceFieldsIncomplete` | `EVIDENCE_FIELDS_INCOMPLETE` |
| `ReferenceSfuError` | `KernelContractUnavailable` | `KERNEL_CONTRACT_UNAVAILABLE` |
| `ReferenceSfuError` | `KernelContractMismatch` | `KERNEL_CONTRACT_MISMATCH` |
| `ReferenceSfuError` | `InvalidFixtureRouteAdmission` | `FIXTURE_IDENTITY_INVALID` |
| `ReferenceSfuError` | `StateBoundaryViolation` | `STATE_BOUNDARY_VIOLATION` |
| `ReferenceSfuError` | `EvidenceFieldsIncomplete` | `EVIDENCE_FIELDS_INCOMPLETE` |
| `ReferenceCompositionError` | `KernelContractMismatch` | `KERNEL_CONTRACT_MISMATCH` |
| `ReferenceCompositionError` | `StateBoundaryViolation` | `STATE_BOUNDARY_VIOLATION` |
| `ReferenceCompositionError` | `EvidenceFieldsIncomplete` | `EVIDENCE_FIELDS_INCOMPLETE` |
| `ReferenceRuntimeError` | `RuntimeExecutorError` | `RUNTIME_EXECUTOR_ERROR` |
| `ReferenceRuntimeError` | `CommandScopeMismatch` | `COMMAND_SCOPE_MISMATCH` |
| `ReferenceRuntimeError` | `EvidenceFieldsIncomplete` | `EVIDENCE_FIELDS_INCOMPLETE` |
| `ReferenceRuntimeError` | `EvidenceWriteError` | `EVIDENCE_FIELDS_INCOMPLETE` |

### 3.3 Product Error Mapping

| Error enum | Required variants | DistroEvidenceReason |
|---|---|---|
| `ProductSignalingError` | `KernelContractUnavailable` | `KERNEL_CONTRACT_UNAVAILABLE` |
| `ProductSignalingError` | `KernelContractMismatch` | `KERNEL_CONTRACT_MISMATCH` |
| `ProductSignalingError` | `StateBoundaryViolation` | `STATE_BOUNDARY_VIOLATION` |
| `ProductSignalingError` | `ReadinessNotAdmitted` | `READINESS_NOT_ADMITTED` |
| `ProductTurnError` | `KernelContractUnavailable` | `KERNEL_CONTRACT_UNAVAILABLE` |
| `ProductTurnError` | `KernelContractMismatch` | `KERNEL_CONTRACT_MISMATCH` |
| `ProductTurnError` | `StateBoundaryViolation` | `STATE_BOUNDARY_VIOLATION` |
| `ProductTurnError` | `ReadinessNotAdmitted` | `READINESS_NOT_ADMITTED` |
| `ProductSfuError` | `KernelContractUnavailable` | `KERNEL_CONTRACT_UNAVAILABLE` |
| `ProductSfuError` | `KernelContractMismatch` | `KERNEL_CONTRACT_MISMATCH` |
| `ProductSfuError` | `StateBoundaryViolation` | `STATE_BOUNDARY_VIOLATION` |
| `ProductSfuError` | `ReadinessNotAdmitted` | `READINESS_NOT_ADMITTED` |
| `ProductPolicyError` | `InvalidFixtureIdentity` | `FIXTURE_IDENTITY_INVALID` |
| `ProductPolicyError` | `SecurityReasonMappingFailed` | `STATE_BOUNDARY_VIOLATION` |
| `ProductPolicyError` | `ReadinessNotAdmitted` | `READINESS_NOT_ADMITTED` |
| `ProductPersistenceTopologyError` | `ProviderNotAdmitted` | `READINESS_NOT_ADMITTED` |
| `ProductPersistenceTopologyError` | `ProjectionMappingViolation` | `STATE_BOUNDARY_VIOLATION` |
| `ProductRuntimeError` | `RuntimeExecutorError` | `RUNTIME_EXECUTOR_ERROR` |
| `ProductRuntimeError` | `CommandScopeMismatch` | `COMMAND_SCOPE_MISMATCH` |
| `ProductRuntimeError` | `ReadinessNotAdmitted` | `READINESS_NOT_ADMITTED` |
| `ProductMonitoringError` | `EvidenceFieldsIncomplete` | `EVIDENCE_FIELDS_INCOMPLETE` |
| `ProductMonitoringError` | `CommandScopeMismatch` | `COMMAND_SCOPE_MISMATCH` |
| `ProductMonitoringError` | `ReadinessNotAdmitted` | `READINESS_NOT_ADMITTED` |
| `ProductRollbackError` | `RuntimeExecutorError` | `RUNTIME_EXECUTOR_ERROR` |
| `ProductRollbackError` | `ReadinessNotAdmitted` | `READINESS_NOT_ADMITTED` |

### 3.4 Evidence Reason Ownership (mapping side)

`DistroEvidenceReason` holds its single canonical Rust enum in `distro-support/evidence/src/reason.rs`. `arcrtc-reference-ops/src/reason.rs` and `arcrtc-product-monitoring/src/evidence.rs` MUST NOT redefine it; they use or re-export the type of `arcrtc_distro_evidence`. The variant set MUST match the closed set in Section 2.4 of this chapter.

---

## 4. Evidence Schema

### 4.1 Canonical Rust Type

`distro-support/evidence/src/record.rs` MUST hold the following canonical Rust type. `arcrtc-reference-ops/src/evidence.rs` and `arcrtc-product-monitoring/src/evidence.rs` use or re-export this type and MUST NOT redefine an equivalent independent type.

```rust
pub struct DistroEvidenceRecord {
    pub correlation_id: String,
    pub command: String,
    pub working_directory: String,
    pub target_package: Option<String>,
    pub target_scope: String,
    pub command_class: DistroCommandClass,
    pub distro_layer: DistroLayer,
    pub target_plane: DistroPlane,
    pub environment_class: DistroEnvironmentClass,
    pub toolchain_runtime_version: String,
    pub input_fixture_or_workload: Option<String>,
    pub expected_outcome: String,
    pub actual_outcome: String,
    pub exit_status: Option<i32>,
    pub kernel_reason: Option<String>,
    pub distro_reason: DistroEvidenceReason,
    pub non_claim_scope: Vec<DistroNonClaimScope>,
    pub rerun_condition: String,
}
```

No field in this schema is allowed to carry raw secret, raw token, raw packet payload, or private key (MUST NOT).

This schema fixes the evidence shape. It does not claim evidence success, test pass, benchmark pass, or readiness.

### 4.2 Closed Enums

| Enum | Variants |
|---|---|
| `DistroCommandClass` | `Format`, `Build`, `Test`, `Benchmark`, `RealDevice`, `ProductionReadiness`, `LiveReadiness` |
| `DistroLayer` | `Reference`, `Product`, `Benchmark`, `RealDevice`, `Readiness` |
| `DistroPlane` | `Signaling`, `Turn`, `Sfu`, `Composition`, `Ops`, `Policy`, `Persistence`, `Deployment`, `Monitoring`, `Rollback` |
| `DistroEnvironmentClass` | `LocalDocsOnly`, `LocalSingleHost`, `ControlledProcess`, `BenchmarkHost`, `RealDeviceBounded`, `ProductionDeferred`, `LiveDeferred` |
| `DistroNonClaimScope` | `SourceDistroCompletionNotClaimed`, `ProductCompletionNotClaimed`, `TestPassNotClaimed`, `BenchmarkThresholdNotClaimed`, `CommandTargetSuccessNotClaimed`, `BehaviorCorrectnessNotClaimed`, `NativeApplicationReadinessNotClaimed`, `PublicDistributionReadinessNotClaimed`, `ProductionReadinessNotClaimed`, `LiveReadinessNotClaimed`, `KernelCompletionNotClaimed`, `KernelFreezeNotClaimed` |

`DistroEvidenceReason` is the closed set in Section 2.4 of this chapter and is owned by Section 1 of this chapter. The wire serialization of all closed enums is defined only by Section 5 of this chapter.

### 4.3 JSON Shape

Serialized evidence MUST use snake_case JSON field names matching the Rust field names above.

Required JSON fields:

- `correlation_id`
- `command`
- `working_directory`
- `target_scope`
- `command_class`
- `distro_layer`
- `target_plane`
- `environment_class`
- `toolchain_runtime_version`
- `expected_outcome`
- `actual_outcome`
- `distro_reason`
- `non_claim_scope`
- `rerun_condition`

Optional JSON fields:

- `target_package`
- `input_fixture_or_workload`
- `exit_status`
- `kernel_reason`

### 4.4 Evidence Validation Rule

Evidence is rejected (fail-closed) if any of the following is true.

- `correlation_id` is empty.
- `command` is empty.
- `working_directory` is not under `distro/`.
- `target_scope` is empty.
- `toolchain_runtime_version` is empty.
- `expected_outcome` is empty.
- `distro_reason` is not in the closed set.
- `distro_reason` is `UNKNOWN`.
- `kernel_reason` is `UNKNOWN`, `Unknown`, `unknown`, or `Other`.
- `non_claim_scope` is empty.
- `non_claim_scope` lacks a command-class required item from Section 5 of this chapter.
- `actual_outcome` is empty.
- `rerun_condition` is empty.
- command-class required `target_package` is absent or empty.
- command-class required `input_fixture_or_workload` is absent or empty.
- command-class required `exit_status` is absent.
- any string field listed below contains a secret-like or raw-payload marker:
  `correlation_id`, `command`, `working_directory`, `target_package`, `target_scope`, `toolchain_runtime_version`, `input_fixture_or_workload`, `expected_outcome`, `actual_outcome`, `kernel_reason`, `rerun_condition`.

Secret-like or raw-payload markers are fixed to:

- `secret=`
- `token=`
- `private_key`
- `-----BEGIN`
- `packet_payload=`
- `raw_packet=`
- `authorization:`
- `bearer `

Detection is ASCII case-insensitive. A violation returns `EvidenceValidationError::RawSecretLikeValue`.

`validate_evidence_record(&record)` validates only the base fields listed in this chapter. It does not validate readiness admission references because those fields are not part of `DistroEvidenceRecord`. Any base record whose `command_class` is `ProductionReadiness` or `LiveReadiness` is diagnostic-only unless it is embedded in `ReadinessEvidenceRecord` and passes the readiness extension validator defined in Section 6 of this chapter.

### 4.5 Command-Class Required Field Matrix

`DistroEvidenceRecord` keeps `target_package`, `input_fixture_or_workload`, and `exit_status` optional at the JSON shape level because not every evidence class uses all three fields. Official command evidence MUST apply the following command-class matrix before adoption. When a matrix cell is `required`, `target_package` and `input_fixture_or_workload` must be `Some(non-empty trimmed string)`, and `exit_status` must be `Some(i32)`. This matrix is part of `validate_evidence_record(&record)`.

| command_class | `target_package` | `input_fixture_or_workload` | `exit_status` |
|---|---|---|---|
| `Format` | optional | optional | required |
| `Build` | required | optional | required |
| `Test` | required | required | required |
| `Benchmark` | required | required | required |
| `RealDevice` | required | required | required |
| `ProductionReadiness` | required | required | required |
| `LiveReadiness` | required | required | required |

Readiness admission references are not base fields. They are required extension fields in `ReadinessEvidenceRecord` and are validated by Section 6 of this chapter.

### 4.6 Claim Boundary

| command_class | Evidence may support | Evidence must not support |
|---|---|---|
| `Format` | formatting | build/test success |
| `Build` | build success | behavior correctness |
| `Test` | tested behavior | production readiness |
| `Benchmark` | measurement output | benchmark threshold unless admitted |
| `RealDevice` | bounded command result | general live readiness |
| `ProductionReadiness` | bounded production readiness | live readiness |
| `LiveReadiness` | bounded live readiness | Kernel completion / freeze |

---

## 5. Evidence Wire Format

This section fixes the evidence format. It does not claim command success, test pass, benchmark threshold, production readiness, or live readiness.

### 5.1 Reason Wire Format

`DistroEvidenceReason` is implemented in `distro-support/evidence/src/reason.rs` as the following Rust enum.

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DistroEvidenceReason {
    DistroOk,
    KernelContractUnavailable,
    KernelContractMismatch,
    DependencyNotAdmitted,
    RuntimeExecutorError,
    StateBoundaryViolation,
    FixtureIdentityInvalid,
    EvidenceFieldsIncomplete,
    CommandScopeMismatch,
    BenchmarkScopeMismatch,
    RealDeviceScopeMismatch,
    ReadinessNotAdmitted,
}
```

Wire values are fixed to:

| Variant | Wire value |
|---|---|
| `DistroOk` | `DISTRO_OK` |
| `KernelContractUnavailable` | `KERNEL_CONTRACT_UNAVAILABLE` |
| `KernelContractMismatch` | `KERNEL_CONTRACT_MISMATCH` |
| `DependencyNotAdmitted` | `DEPENDENCY_NOT_ADMITTED` |
| `RuntimeExecutorError` | `RUNTIME_EXECUTOR_ERROR` |
| `StateBoundaryViolation` | `STATE_BOUNDARY_VIOLATION` |
| `FixtureIdentityInvalid` | `FIXTURE_IDENTITY_INVALID` |
| `EvidenceFieldsIncomplete` | `EVIDENCE_FIELDS_INCOMPLETE` |
| `CommandScopeMismatch` | `COMMAND_SCOPE_MISMATCH` |
| `BenchmarkScopeMismatch` | `BENCHMARK_SCOPE_MISMATCH` |
| `RealDeviceScopeMismatch` | `REAL_DEVICE_SCOPE_MISMATCH` |
| `ReadinessNotAdmitted` | `READINESS_NOT_ADMITTED` |

`Unknown`, `Other`, raw `String`, or untyped reason is forbidden (MUST NOT).

### 5.2 Evidence Label Wire Format

The following enums are owned by `distro-support/evidence/src/record.rs`.

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DistroCommandClass {
    Format,
    Build,
    Test,
    Benchmark,
    RealDevice,
    ProductionReadiness,
    LiveReadiness,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DistroLayer {
    Reference,
    Product,
    Benchmark,
    RealDevice,
    Readiness,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DistroPlane {
    Signaling,
    Turn,
    Sfu,
    Composition,
    Ops,
    Policy,
    Persistence,
    Deployment,
    Monitoring,
    Rollback,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DistroEnvironmentClass {
    LocalDocsOnly,
    LocalSingleHost,
    ControlledProcess,
    BenchmarkHost,
    RealDeviceBounded,
    ProductionDeferred,
    LiveDeferred,
}
```

### 5.3 Non-Claim Scope Closed Set

`non_claim_scope` is not a free string list. It is a list of `DistroNonClaimScope`.

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DistroNonClaimScope {
    SourceDistroCompletionNotClaimed,
    ProductCompletionNotClaimed,
    TestPassNotClaimed,
    BenchmarkThresholdNotClaimed,
    CommandTargetSuccessNotClaimed,
    BehaviorCorrectnessNotClaimed,
    NativeApplicationReadinessNotClaimed,
    PublicDistributionReadinessNotClaimed,
    ProductionReadinessNotClaimed,
    LiveReadinessNotClaimed,
    KernelCompletionNotClaimed,
    KernelFreezeNotClaimed,
}
```

Wire values are fixed to:

| Variant | Wire value |
|---|---|
| `SourceDistroCompletionNotClaimed` | `source_distro_completion_not_claimed` |
| `ProductCompletionNotClaimed` | `product_completion_not_claimed` |
| `TestPassNotClaimed` | `test_pass_not_claimed` |
| `BenchmarkThresholdNotClaimed` | `benchmark_threshold_not_claimed` |
| `CommandTargetSuccessNotClaimed` | `command_target_success_not_claimed` |
| `BehaviorCorrectnessNotClaimed` | `behavior_correctness_not_claimed` |
| `NativeApplicationReadinessNotClaimed` | `native_application_readiness_not_claimed` |
| `PublicDistributionReadinessNotClaimed` | `public_distribution_readiness_not_claimed` |
| `ProductionReadinessNotClaimed` | `production_readiness_not_claimed` |
| `LiveReadinessNotClaimed` | `live_readiness_not_claimed` |
| `KernelCompletionNotClaimed` | `kernel_completion_not_claimed` |
| `KernelFreezeNotClaimed` | `kernel_freeze_not_claimed` |

### 5.4 Command-Class Required Non-Claim Scope

| Command class | Required non-claim scopes |
|---|---|
| `Format` | `CommandTargetSuccessNotClaimed` |
| `Build` | `BehaviorCorrectnessNotClaimed`, `ProductionReadinessNotClaimed`, `LiveReadinessNotClaimed` |
| `Test` | `ProductionReadinessNotClaimed`, `LiveReadinessNotClaimed` |
| `Benchmark` | `BenchmarkThresholdNotClaimed`, `ProductionReadinessNotClaimed`, `LiveReadinessNotClaimed` |
| `RealDevice` | `NativeApplicationReadinessNotClaimed`, `PublicDistributionReadinessNotClaimed`, `ProductionReadinessNotClaimed`, `LiveReadinessNotClaimed` |
| `ProductionReadiness` | `LiveReadinessNotClaimed`, `KernelCompletionNotClaimed`, `KernelFreezeNotClaimed` |
| `LiveReadiness` | `KernelCompletionNotClaimed`, `KernelFreezeNotClaimed` |

### 5.5 Validation Error Closed Set

`distro-support/evidence/src/validation.rs` owns the validation error enum.

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvidenceValidationError {
    EmptyCorrelationId,
    EmptyCommand,
    EmptyWorkingDirectory,
    WorkingDirectoryOutsideDistro,
    EmptyTargetScope,
    EmptyToolchainRuntimeVersion,
    EmptyExpectedOutcome,
    EmptyActualOutcome,
    EmptyRerunCondition,
    EmptyNonClaimScope,
    MissingRequiredTargetPackage,
    MissingRequiredInputFixtureOrWorkload,
    MissingRequiredExitStatus,
    MissingRequiredNonClaimScope,
    InvalidReadinessAdmission,
    RawSecretLikeValue,
    ForbiddenUnclassifiedReason,
}
```

The validator signature is fixed to:

```rust
pub fn validate_evidence_record(
    record: &DistroEvidenceRecord,
) -> Result<(), EvidenceValidationError>
```

Each validation error maps to `DistroEvidenceReason::EvidenceFieldsIncomplete`, except:

| Validation error | Reason |
|---|---|
| `WorkingDirectoryOutsideDistro` | `CommandScopeMismatch` |
| `InvalidReadinessAdmission` | `ReadinessNotAdmitted` |
| `ForbiddenUnclassifiedReason` | `EvidenceFieldsIncomplete` |

---

## 6. Evidence Extension Record

This section fixes, with Rust struct and JSON shape, how benchmark / real-device / readiness evidence extends `DistroEvidenceRecord`.

Because Rust has no struct inheritance, every extension record MUST hold `#[serde(flatten)] base: DistroEvidenceRecord`. In JSON, base fields and extension fields are serialized as top-level fields of the same object. This section fixes the evidence shape. It does not claim benchmark threshold, real-device success, production readiness, or live readiness.

### 6.1 Benchmark Evidence Record

Owner file: `tests/benchmark/src/evidence.rs`

```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkEvidenceRecord {
    #[serde(flatten)]
    pub base: DistroEvidenceRecord,
    pub scenario_id: String,
    pub scenario_name: String,
    pub layer: DistroLayer,
    pub plane: DistroPlane,
    pub workload_summary: String,
    pub criterion_group: String,
    pub sample_size: usize,
    pub warm_up_seconds: u64,
    pub measurement_seconds: u64,
    pub noise_threshold: f64,
    pub confidence_level: f64,
    pub significance_level: f64,
    pub median_ns: f64,
    pub mean_ns: f64,
    pub std_dev_ns: f64,
    pub p95_ns: f64,
    pub throughput_items_per_second: f64,
}
```

Validation rejects the record if:

- `base.command_class != DistroCommandClass::Benchmark`
- `base.non_claim_scope` lacks `BenchmarkThresholdNotClaimed`
- `scenario_id` is not listed in the benchmark scenario workload authority
- `scenario_name` does not match the scenario table row for `scenario_id`
- `layer != base.distro_layer`
- `plane != base.target_plane`
- `layer` or `plane` does not match the scenario table row for `scenario_id`
- `workload_summary` does not match the scenario table row for `scenario_id`
- `criterion_group != "distro_benchmark_scenarios"`
- `sample_size != 100`
- `warm_up_seconds != 3`
- `measurement_seconds != 10`
- `noise_threshold != 0.05`
- `confidence_level != 0.95`
- `significance_level != 0.05`
- any Criterion configuration float is not finite
- any timing value is negative or not finite
- `throughput_items_per_second` is not finite or is less than or equal to `0.0`

### 6.2 Real-Device Evidence Record

Owner file: `tests/real-device/src/evidence.rs`

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RealDevicePlatform {
    Android,
    Ios,
    Browser,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RealDeviceClass {
    AndroidPhysical,
    AndroidEmulator,
    IosPhysical,
    IosSimulator,
    DesktopBrowser,
    MobileBrowser,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RealDeviceInternalCommandClass {
    AndroidDevice,
    IosDevice,
    IosSimulator,
    Browser,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RealDeviceVersionClass {
    AndroidApiLevel,
    IosSystemVersion,
    BrowserVersion,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RealDeviceExecutionSurface {
    NativeSdkCommand,
    BrowserWebrtcCommand,
    WrapperLocalCapability,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RealDeviceNetworkClass {
    LocalUsb,
    EmulatorLoopback,
    SimulatorLoopback,
    LocalBrowser,
    MobileBrowserEmulation,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RealDeviceEvidenceRecord {
    #[serde(flatten)]
    pub base: DistroEvidenceRecord,
    pub platform: RealDevicePlatform,
    pub device_class: RealDeviceClass,
    pub internal_command_class: RealDeviceInternalCommandClass,
    pub runtime_version_class: RealDeviceVersionClass,
    pub execution_surface: RealDeviceExecutionSurface,
    pub network_class: RealDeviceNetworkClass,
    pub platform_command: Option<String>,
    pub preflight_outcome: String,
    pub logs_metrics_location: String,
    pub redacted_device_identifier: Option<String>,
}
```

Validation rejects the record if:

- `base.command_class != DistroCommandClass::RealDevice`
- `base.non_claim_scope` lacks `NativeApplicationReadinessNotClaimed`, `PublicDistributionReadinessNotClaimed`, `ProductionReadinessNotClaimed`, and `LiveReadinessNotClaimed`
- `platform`, `device_class`, and `internal_command_class` do not match the real-device wrapper command authority
- `runtime_version_class`, `execution_surface`, or `network_class` does not match the device context matrix
- `platform_command` is absent for the Android / iOS platform tool path
- `platform_command` is present for the browser wrapper-local capability path
- `platform_command` is not in the platform command closed set when a platform tool is used
- `base.exit_status` is absent
- `base.actual_outcome` is empty
- `preflight_outcome` is empty
- `logs_metrics_location` is empty
- `logs_metrics_location` is outside `target/distro-evidence/real-device/`
- `redacted_device_identifier` is absent when `base.exit_status == Some(0)`
- `redacted_device_identifier` is present when `base.exit_status != Some(0)`
- `redacted_device_identifier` is present but empty after trimming whitespace
- `redacted_device_identifier` is neither literal `redacted` nor `sha256:<64 lowercase hex characters>`
- `redacted_device_identifier` contains one of the raw identifier markers: `serial:`, `serial=`, `udid:`, `udid=`, `android_id:`, `android_id=`, `device_id:`, `device_id=`, `imei:`, `imei=`, `meid:`, `meid=`, `account:`, `account=`, `token:`, `token=`, `private_key:`, `private_key=`, `device_name:`, or `device_name=`

The post-execution field requiredness matrix is governed by the real-device wrapper command authority.

### 6.3 Benchmark / Real-Device Extension Validation API

Benchmark validator owner: `tests/benchmark/src/evidence.rs`

Benchmark validator signature:

```rust
pub fn validate_benchmark_evidence_record(
    record: &BenchmarkEvidenceRecord,
) -> Result<(), BenchmarkEvidenceValidationError>
```

`BenchmarkEvidenceValidationError`:

```rust
pub enum BenchmarkEvidenceValidationError {
    Base(EvidenceValidationError),
    CommandClassMismatch,
    ScenarioIdNotListed,
    ScenarioNameMismatch,
    ScenarioLayerMismatch,
    ScenarioPlaneMismatch,
    WorkloadSummaryMismatch,
    CriterionGroupMismatch,
    CriterionConfigurationMismatch,
    InvalidTimingValue,
    InvalidThroughputValue,
}
```

Benchmark validation errors map to `DistroEvidenceReason` as follows.

| Benchmark validation error | Distro reason |
|---|---|
| `Base(error)` | mapping for `EvidenceValidationError` (Section 5.5 of this chapter) |
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

Real-device validator owner: `tests/real-device/src/evidence.rs`

Real-device validator signature:

```rust
pub fn validate_real_device_evidence_record(
    record: &RealDeviceEvidenceRecord,
) -> Result<(), RealDeviceEvidenceValidationError>
```

`RealDeviceEvidenceValidationError`:

```rust
pub enum RealDeviceEvidenceValidationError {
    Base(EvidenceValidationError),
    CommandClassMismatch,
    PlatformDeviceClassMismatch,
    InternalCommandClassMismatch,
    RuntimeVersionClassMismatch,
    ExecutionSurfaceMismatch,
    NetworkClassMismatch,
    PlatformCommandMissing,
    PlatformCommandUnexpected,
    PlatformCommandNotAdmitted,
    MissingPostExecutionField,
    EmptyLogsMetricsLocation,
    LogsMetricsLocationOutsideEvidenceDir,
    EmptyDeviceIdentifier,
    DeviceIdentifierFormatMismatch,
    UnexpectedDeviceIdentifier,
    UnsafeDeviceIdentifier,
    PreflightOutcomeNotAdmitted,
    ProfileNotAdmitted,
}
```

Real-device validation errors map to `DistroEvidenceReason` as follows.

| Real-device validation error | Distro reason |
|---|---|
| `Base(error)` | mapping for `EvidenceValidationError` (Section 5.5 of this chapter) |
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
| `PreflightOutcomeNotAdmitted` | `EVIDENCE_FIELDS_INCOMPLETE` |
| `ProfileNotAdmitted` | `EVIDENCE_FIELDS_INCOMPLETE` |

### 6.4 Readiness Evidence Record

Owner file: `distro-support/evidence/src/readiness_extension.rs`

Readiness record construction / validation files:

- `tests/production-readiness/tests/production_matrix.rs`
- `tests/production-readiness/tests/fail_closed.rs`
- `tests/production-readiness/tests/success_matrix.rs`
- `tests/live/tests/live_matrix.rs`
- `tests/live/tests/fail_closed.rs`
- `tests/live/tests/success_matrix.rs`

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadinessClaim {
    ProductionReadiness,
    LiveReadiness,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ReadinessEvidenceRecord {
    #[serde(flatten)]
    pub base: DistroEvidenceRecord,
    pub readiness_gate_id: String,
    pub readiness_claim: ReadinessClaim,
    pub readiness_adr_ref: String,
    pub readiness_canonical_ref: String,
    pub build_evidence_ref: Option<String>,
    pub behavior_test_evidence_ref: Option<String>,
    pub auth_provider_admission_ref: Option<String>,
    pub persistence_provider_admission_ref: Option<String>,
    pub deployment_profile_ref: Option<String>,
    pub monitoring_probe_ref: Option<String>,
    pub rollback_plan_ref: Option<String>,
    pub security_scan_ref: Option<String>,
    pub production_readiness_report_ref: Option<String>,
    pub live_endpoint_evidence_ref: Option<String>,
    pub public_traversal_evidence_ref: Option<String>,
    pub rollback_drain_execution_ref: Option<String>,
    pub shutdown_drain_evidence_ref: Option<String>,
    pub restore_evidence_ref: Option<String>,
    pub closed_gate_report_ref: Option<String>,
}
```

Validation rejects the record if:

- `base.command_class` does not match `readiness_claim`
- `readiness_gate_id` is not listed in the readiness matrix authority
- `readiness_claim == ProductionReadiness` and `readiness_gate_id` is not one of `PRD-001` through `PRD-009`
- `readiness_claim == LiveReadiness` and `readiness_gate_id` is not one of `LIVE-001` through `LIVE-008`
- `readiness_gate_id` is `PRD-003` and `context.auth_provider_authority == Absent`
- `readiness_gate_id` is `PRD-004` and `context.persistence_provider_authority == Absent`
- `readiness_claim == LiveReadiness` and `context.live_endpoint_authority == Absent`
- `readiness_gate_id` is `LIVE-001` and `context.production_readiness_report == Absent`
- `readiness_gate_id` is `LIVE-003` and `context.public_traversal_authority == Absent`
- `readiness_adr_ref != "READINESS_CLAIM_BOUNDARY"`
- `readiness_canonical_ref != "READINESS_MATRIX"`
- `readiness_claim == ProductionReadiness` and `base.non_claim_scope` lacks `LiveReadinessNotClaimed`
- `readiness_claim == LiveReadiness` and `base.non_claim_scope` lacks `KernelCompletionNotClaimed` or `KernelFreezeNotClaimed`
- a ref field required by the `Gate id Required Extension Ref Matrix` in the readiness matrix authority is absent
- a required ref field is present but empty after trimming whitespace
- a ref field not allowed by the `Gate id Required Extension Ref Matrix` is populated
- any readiness extension ref contains raw secret, raw token, private key, raw packet payload, or direct personal identifier

The readiness validator signature, the closed validation error set, and the validation-error-to-reason mapping are owned by Section 1 (readiness extension ownership) of this chapter. `tests/production-readiness/tests/*.rs` may construct and validate only records whose `readiness_claim == ProductionReadiness` unless the test is explicitly asserting fail-closed rejection of the opposite claim. `tests/live/tests/*.rs` may likewise construct and validate only records whose `readiness_claim == LiveReadiness` unless asserting fail-closed rejection of the opposite claim. Neither readiness test package owns the `ReadinessClaim` enum or `ReadinessEvidenceRecord` struct.

### 6.5 Readiness Extension Context and Validator

The readiness admission context is fixed to:

```rust
pub enum ReadinessAdmissionState {
    Admitted,
    Absent,
}

pub struct ReadinessValidationContext {
    pub auth_provider_authority: ReadinessAdmissionState,
    pub persistence_provider_authority: ReadinessAdmissionState,
    pub live_endpoint_authority: ReadinessAdmissionState,
    pub production_readiness_report: ReadinessAdmissionState,
    pub public_traversal_authority: ReadinessAdmissionState,
}
```

The validator signature is fixed to:

```rust
pub fn validate_readiness_evidence_record(
    record: &ReadinessEvidenceRecord,
    context: &ReadinessValidationContext,
) -> Result<(), ReadinessEvidenceValidationError>
```

`ReadinessEvidenceValidationError` is fixed to:

```rust
pub enum ReadinessEvidenceValidationError {
    Base(EvidenceValidationError),
    CommandClassClaimMismatch,
    ReadinessGateClaimMismatch,
    GateIdNotListed,
    ReadinessAuthorityRefMismatch,
    MissingRequiredExtensionRef,
    EmptyRequiredExtensionRef,
    UnexpectedExtensionRefPopulated,
    AuthProviderAuthorityNotAdmitted,
    PersistenceProviderAuthorityNotAdmitted,
    LiveEndpointAuthorityNotAdmitted,
    ProductionReadinessReportNotAdmitted,
    PublicTraversalAuthorityNotAdmitted,
    SecretLikeExtensionRef,
}
```

Each readiness validation error maps to `DistroEvidenceReason` as follows.

| Readiness validation error | Distro reason |
|---|---|
| `Base(error)` | mapping for `EvidenceValidationError` (Section 5.5 of this chapter) |
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

Gate id / required ref validation uses the readiness matrix authority as authority. Auth provider authority absent, persistence provider authority absent, live endpoint authority absent, production readiness report absent, and public traversal authority absent MUST use the dedicated not-admitted variants above instead of `MissingRequiredExtensionRef`.

### 6.6 Writer Rule

Extension evidence writers MUST call `validate_evidence_record(&record.base)` and the extension-specific validator before writing extension JSON. If base validation fails, the extension writer returns the mapped `DistroEvidenceReason` and does not write JSON (fail-closed). For readiness, the extension-specific validator is `validate_readiness_evidence_record(&record, &context)` from `arcrtc-distro-evidence`. The context type and admission-state closed set are owned by Section 1 of this chapter.

Extension writers MUST NOT create a second base schema, duplicate base fields manually, or serialize base under a nested `base` object.

---

## 7. Invariants and Fail-Closed Conditions (Collapse Conditions)

The authority of this chapter collapses if any of the following occurs. These are fail-closed boundaries, and all are forbidden (MUST NOT).

### 7.1 Reason / Ownership Invariants

- Redefining `DistroEvidenceReason` per package.
- Redefining `DistroNonClaimScope` as a raw string list.
- Treating `arcrtc-reference-ops` as the reason owner and creating a reverse dependency from a reference plane package.
- A product package using the local enum of `arcrtc-product-monitoring` as an independent reason authority.
- A shared evidence package owning the Kernel reason catalog or a Kernel semantic type.
- Treating evidence writer success as command success / readiness success.
- Adopting an `UNKNOWN` reason into official evidence.
- A distro reason changing the Kernel reason catalog.
- Using output without a correlation id as a basis for closure.
- Adopting a diagnostic log as readiness evidence.

### 7.2 Error Mapping Invariants

- Adding an `Unknown` / `Other` / raw `String` variant to an error enum.
- Adopting a dependency raw error as an evidence reason.
- Promoting a product error into the Kernel reason catalog.
- Treating `ReadinessNotAdmitted` as success.
- Adding an unmapped error variant.
- Redefining a package-local `DistroEvidenceReason` enum.

### 7.3 Schema / Wire Format Invariants

- An evidence record lacking a required JSON field.
- Including raw secret, raw token, raw packet payload, or private key in an evidence field.
- Accepting `non_claim_scope` as a free string list.
- Reusing evidence across a command class's claim boundary.
- Adopting evidence from a Kernel working directory as distro evidence.
- Redefining `DistroEvidenceRecord` per reference / product package.
- Serializing an enum JSON value with a string different from this chapter.
- Not validating the required non-claim scope per command class.
- Treating a readiness class as valid without its required readiness admission record.

### 7.4 Extension Record Invariants

- An extension evidence record omitting `#[serde(flatten)] base`.
- Extension JSON nesting base evidence under a `base` object.
- An extension writer skipping `validate_evidence_record`.
- Treating a benchmark / real-device / readiness extension field as readiness / threshold proof without its dedicated admission record.
