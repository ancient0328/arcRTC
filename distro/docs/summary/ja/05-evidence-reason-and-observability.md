# Evidence Reason と Observability

状態: SSOT 統合版
日付: 2026-06-28 JST

## 目的

本章は、arcRTC v0.2 distro 領域（Kernel 外実装領域）における evidence reason の所有、observability evidence reason、error reason mapping、evidence schema、evidence wire format、evidence extension record を、再現実装が可能な粒度で完全に固定します。本章は完全に自己完結しており、本仕様書内の他章番号参照のみを許可し、外部ファイル・Kernel 文書・実コードを開かずに理解および再現実装できます。

依存規則は `distro -> Kernel`（contract / SDK / command surface のみ）です。distro は Kernel reason catalog を所有しません。distro は自領域の evidence reason owner を持ち、Kernel reason は補助的な参照 field として扱います。

---

## 1. Evidence Reason 所有

### 1.1 所有原則

distro 全体で共有する evidence reason、evidence record、command class、layer、plane、environment class、non-claim scope の Rust 型所有を本領域内に固定します。これらは reference 領域にも product 領域にも偏らない共有 owner（shared owner）に属します。

この所有は Kernel reason catalog を所有しません。distro-local evidence reason は Kernel reason を補助的に参照できますが、Kernel reason に昇格しません。

### 1.2 Owner Package

共有 evidence / reason 型の唯一の Rust owner package を次に固定します。

| Path | Package name | 所有対象 |
|---|---|---|
| `distro-support/evidence` | `arcrtc-distro-evidence` | distro evidence reason / record / validation / readiness extension / common labels / non-claim scope |

`arcrtc-reference-ops` と `arcrtc-product-monitoring` は evidence helper / writer を所有できます。ただし、`DistroEvidenceReason` と `DistroEvidenceRecord` の型 authority は `arcrtc-distro-evidence` に置きます。

### 1.3 Required Files

`distro-support/evidence` は次の file contract を持たなければなりません（MUST）。

| File | Role |
|---|---|
| `src/lib.rs` | public export |
| `src/reason.rs` | `DistroEvidenceReason` closed enum |
| `src/record.rs` | `DistroEvidenceRecord`、closed label enums、`DistroNonClaimScope` |
| `src/readiness_extension.rs` | `ReadinessClaim`、`ReadinessAdmissionState`、`ReadinessValidationContext`、`ReadinessEvidenceRecord`、`ReadinessEvidenceValidationError`、`validate_readiness_evidence_record` |
| `src/validation.rs` | evidence record validation function |
| `src/error.rs` | validation / writer から独立した typed error |

### 1.4 Public Export

`distro-support/evidence/src/lib.rs` は次の export に固定します。

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

### 1.5 Record / Readiness Extension の所有

`DistroEvidenceRecord` は本章 4 節の canonical Rust type と一致しなければなりません（MUST）。`non_claim_scope` は `Vec<DistroNonClaimScope>` に固定し、自由文字列 vector として入力してはなりません（MUST NOT）。

`ReadinessClaim`、`ReadinessAdmissionState`、`ReadinessValidationContext`、`ReadinessEvidenceRecord`、`ReadinessEvidenceValidationError`、`validate_readiness_evidence_record` の型 authority は `distro-support/evidence/src/readiness_extension.rs` に限定します。

`arcrtc-reference-ops/src/evidence.rs` と `arcrtc-product-monitoring/src/evidence.rs` は次のいずれかに限定しなければなりません（MUST）。

1. `arcrtc-distro-evidence` の型を re-export する。
2. `arcrtc-distro-evidence` の型を受け取り JSON writer / helper を実装する。

同名の独立 enum / struct を再定義してはなりません（MUST NOT）。

### 1.6 Dependency Rule

`DistroEvidenceReason` を field、return type、error mapping に使う package は、必ず `arcrtc-distro-evidence` に local path dependency を持たなければなりません（MUST）。

`arcrtc-distro-evidence` は Kernel crate に依存してはなりません（MUST NOT）。Kernel reason を参照する場合は `String` / `Option<String>` field に閉じ、Kernel reason type を support package に取り込んではなりません（MUST NOT）。

### 1.7 Failure Mapping Rule

各 distro-local error enum の `distro_reason(&self) -> DistroEvidenceReason` は、`arcrtc_distro_evidence::DistroEvidenceReason` を返さなければなりません（MUST）。package-local copy、type alias による別 authority、同名 enum の再定義は禁止します（MUST NOT）。

### 1.8 Serialization Rule

`DistroEvidenceRecord` は JSON evidence の canonical serialization source です。closed enum の serialization は本章 5 節の wire format に固定します。

`serde` dependency は、canonical evidence record / closed enum serialization owner としては `arcrtc-distro-evidence` に限定して導入できます（MAY）。product profile / persistence topology / deployment config などの non-evidence local serialization は、別途許可された package に限ります。`serde_json` による file write は evidence writer owner package に限定します（MUST）。

---

## 2. Observability / Evidence Reason Boundary

### 2.1 Reason Boundary

Kernel reason は Kernel semantic authority に属します。distro は Kernel reason を `kernel_reason` field として参照できますが（MAY）、Kernel reason catalog を追加・変更してはなりません（MUST NOT）。`kernel_reason` は Kernel reason catalog の所有権を distro に移しません。

ただし、正式 evidence に採用する `kernel_reason` は未分類 sentinel を持ってはなりません（MUST NOT）。`UNKNOWN`、`Unknown`、`unknown`、`Other` は `kernel_reason` としても採用しません（MUST NOT）。

distro evidence reason は `distro_reason` field として扱います。Rust type owner は本章 1 節に従い、`distro-support/evidence/src/reason.rs` に固定します。

### 2.2 Reason Class 所有

| Reason class | Owner | Rule |
|---|---|---|
| Kernel reason | Kernel | 変更しない。source field として参照する |
| distro evidence reason | distro | closed set で定義する |
| command failure reason | command / CI surface | command class ごとに定義する |
| benchmark reason | benchmark surface | benchmark scenario ごとに定義する |
| real-device reason | real-device surface | command / device / environment ごとに定義する |

`UNKNOWN` reason を正式 evidence に採用してはなりません（MUST NOT）。未分類 output は diagnostic とし、正式 report の成立根拠にしてはなりません（MUST NOT）。

### 2.3 Observability Ownership

observability surface が所有してよいもの（MAY）は次です。

- correlation id propagation
- structured log field
- metric name / unit
- trace span name
- command evidence artifact
- benchmark evidence artifact
- distro evidence reason mapping

observability surface が所有してはならないもの（MUST NOT）は次です。

- Kernel semantics
- Kernel reason catalog mutation
- product readiness claim
- live readiness claim
- production SLO

### 2.4 Distro Evidence Reason 閉集合

distro evidence reason は次の閉集合（12 項目）に固定します。`UNKNOWN` は正式 evidence reason として禁止します（MUST NOT）。

| Reason | 意味 |
|---|---|
| `DISTRO_OK` | 期待された distro command / behavior が成功した |
| `KERNEL_CONTRACT_UNAVAILABLE` | 必要な Kernel public contract が存在しない |
| `KERNEL_CONTRACT_MISMATCH` | import した Kernel contract shape が本仕様で固定した distro contract shape と一致しない |
| `DEPENDENCY_NOT_ADMITTED` | dependency が admission record を欠く |
| `RUNTIME_EXECUTOR_ERROR` | runtime executor が bounded command 内で失敗した |
| `STATE_BOUNDARY_VIOLATION` | state / persistence boundary が侵犯された |
| `FIXTURE_IDENTITY_INVALID` | fixture identity / credential が無効だった |
| `EVIDENCE_FIELDS_INCOMPLETE` | 必要な evidence field が欠落している |
| `COMMAND_SCOPE_MISMATCH` | command working directory / target scope の不一致 |
| `BENCHMARK_SCOPE_MISMATCH` | benchmark workload / environment scope の不一致 |
| `REAL_DEVICE_SCOPE_MISMATCH` | real-device command scope の不一致 |
| `READINESS_NOT_ADMITTED` | readiness claim が必要な admission record を欠く |

### 2.5 Required Observability Fields

logs / metrics / traces / reports は次の field を持たなければなりません（MUST）。

| Field | 必須用途 |
|---|---|
| `correlation_id` | command / report / span を接続する |
| `distro_layer` | `DistroLayer` wire value closed set: `reference` / `product` / `benchmark` / `real_device` / `readiness` |
| `target_plane` | `DistroPlane` closed set（本章 4 節） |
| `kernel_reason` | Kernel reason を参照する場合のみ設定 |
| `distro_reason` | closed set から設定 |
| `non_claim_scope` | readiness / completion の非主張範囲 |

### 2.6 Required Files（observability）

| Package | File | Role |
|---|---|---|
| `arcrtc-distro-evidence` | `src/reason.rs` | distro evidence reason enum owner |
| `arcrtc-distro-evidence` | `src/record.rs` | common evidence record / label enum owner |
| `arcrtc-reference-ops` | `src/evidence.rs` | reference command evidence struct |
| `arcrtc-reference-ops` | `src/reason.rs` | shared reason re-export / closed-set helper |
| `arcrtc-product-monitoring` | `src/observability.rs` | product observability field mapping |
| `arcrtc-product-monitoring` | `src/evidence.rs` | product evidence struct |

### 2.7 Required Evidence Fields

evidence record は次の field を持たなければなりません（MUST）。

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

この field 集合を満たさない output は diagnostic とします。

---

## 3. Error / Reason Mapping

### 3.1 Shared Error Rule

各 distro-local error enum は次を満たさなければなりません（MUST）。

- `Unknown` / `Other` / raw `String` variant を持たない。
- raw dependency error を public variant にしない。
- `distro_reason(&self) -> arcrtc_distro_evidence::DistroEvidenceReason` を持つ。
- `Display` は diagnostic text に限定し、claim reason として採用しない。
- `std::error::Error` 実装は optional だが、実装しても evidence reason を置き換えない。

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

### 3.4 Evidence Reason Ownership（mapping 側）

`DistroEvidenceReason` は `distro-support/evidence/src/reason.rs` が唯一の canonical Rust enum を持ちます。`arcrtc-reference-ops/src/reason.rs` と `arcrtc-product-monitoring/src/evidence.rs` は再定義せず（MUST NOT）、`arcrtc_distro_evidence` の型を利用または re-export します。variant set は本章 2.4 の closed set と一致しなければなりません（MUST）。

---

## 4. Evidence Schema

### 4.1 Canonical Rust Type

`distro-support/evidence/src/record.rs` は、次の canonical Rust type を持たなければなりません（MUST）。`arcrtc-reference-ops/src/evidence.rs` と `arcrtc-product-monitoring/src/evidence.rs` は、この型を利用または re-export し、同等の独立型を再定義してはなりません（MUST NOT）。

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

この schema のいかなる field も、raw secret、raw token、raw packet payload、private key を保持してはなりません（MUST NOT）。

本 schema は evidence shape を固定します。evidence success、test pass、benchmark pass、readiness を主張しません。

### 4.2 Closed Enums

| Enum | Variants |
|---|---|
| `DistroCommandClass` | `Format`, `Build`, `Test`, `Benchmark`, `RealDevice`, `ProductionReadiness`, `LiveReadiness` |
| `DistroLayer` | `Reference`, `Product`, `Benchmark`, `RealDevice`, `Readiness` |
| `DistroPlane` | `Signaling`, `Turn`, `Sfu`, `Composition`, `Ops`, `Policy`, `Persistence`, `Deployment`, `Monitoring`, `Rollback` |
| `DistroEnvironmentClass` | `LocalDocsOnly`, `LocalSingleHost`, `ControlledProcess`, `BenchmarkHost`, `RealDeviceBounded`, `ProductionDeferred`, `LiveDeferred` |
| `DistroNonClaimScope` | `SourceDistroCompletionNotClaimed`, `ProductCompletionNotClaimed`, `TestPassNotClaimed`, `BenchmarkThresholdNotClaimed`, `CommandTargetSuccessNotClaimed`, `BehaviorCorrectnessNotClaimed`, `NativeApplicationReadinessNotClaimed`, `PublicDistributionReadinessNotClaimed`, `ProductionReadinessNotClaimed`, `LiveReadinessNotClaimed`, `KernelCompletionNotClaimed`, `KernelFreezeNotClaimed` |

`DistroEvidenceReason` は本章 2.4 の closed set であり、本章 1 節が所有します。すべての closed enum の wire serialization は本章 5 節のみが定義します。

### 4.3 JSON Shape

Serialize した evidence は、上記 Rust field 名と一致する snake_case の JSON field 名を使わなければなりません（MUST）。

Required JSON fields（必須）:

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

Optional JSON fields（任意）:

- `target_package`
- `input_fixture_or_workload`
- `exit_status`
- `kernel_reason`

### 4.4 Evidence Validation Rule

evidence は次のいずれかが真である場合に reject されます（fail-closed）。

- `correlation_id` が空。
- `command` が空。
- `working_directory` が `distro/` の下にない。
- `target_scope` が空。
- `toolchain_runtime_version` が空。
- `expected_outcome` が空。
- `distro_reason` が closed set にない。
- `distro_reason` が `UNKNOWN`。
- `kernel_reason` が `UNKNOWN`、`Unknown`、`unknown`、`Other` のいずれか。
- `non_claim_scope` が空。
- `non_claim_scope` が本章 5 節の command-class required item を欠く。
- `actual_outcome` が空。
- `rerun_condition` が空。
- command-class required な `target_package` が absent または空。
- command-class required な `input_fixture_or_workload` が absent または空。
- command-class required な `exit_status` が absent。
- 下記の string field のいずれかが secret-like / raw-payload marker を含む:
  `correlation_id`、`command`、`working_directory`、`target_package`、`target_scope`、`toolchain_runtime_version`、`input_fixture_or_workload`、`expected_outcome`、`actual_outcome`、`kernel_reason`、`rerun_condition`。

Secret-like / raw-payload marker は次に固定します。

- `secret=`
- `token=`
- `private_key`
- `-----BEGIN`
- `packet_payload=`
- `raw_packet=`
- `authorization:`
- `bearer `

検出は ASCII case-insensitive です。違反は `EvidenceValidationError::RawSecretLikeValue` を返します。

`validate_evidence_record(&record)` は本章に列挙した base field のみを validate します。readiness admission reference は `DistroEvidenceRecord` の field ではないため、validate しません。`command_class` が `ProductionReadiness` または `LiveReadiness` の base record は、`ReadinessEvidenceRecord` に埋め込まれ本章 6 節の readiness extension validator を通過しない限り diagnostic-only です。

### 4.5 Command-Class Required Field Matrix

`DistroEvidenceRecord` は `target_package`、`input_fixture_or_workload`、`exit_status` を JSON shape level で optional に保ちます（すべての evidence class が 3 field を使うわけではないため）。正式 command evidence は採用前に次の command-class matrix を適用しなければなりません（MUST）。matrix cell が `required` の場合、`target_package` と `input_fixture_or_workload` は `Some(non-empty trimmed string)`、`exit_status` は `Some(i32)` でなければなりません。この matrix は `validate_evidence_record(&record)` の一部です。

| command_class | `target_package` | `input_fixture_or_workload` | `exit_status` |
|---|---|---|---|
| `Format` | optional | optional | required |
| `Build` | required | optional | required |
| `Test` | required | required | required |
| `Benchmark` | required | required | required |
| `RealDevice` | required | required | required |
| `ProductionReadiness` | required | required | required |
| `LiveReadiness` | required | required | required |

readiness admission reference は base field ではありません。これらは `ReadinessEvidenceRecord` の required extension field であり、本章 6 節が validate します。

### 4.6 Claim Boundary

| command_class | Evidence が支持してよい | Evidence が支持してはならない |
|---|---|---|
| `Format` | formatting | build/test success |
| `Build` | build success | behavior correctness |
| `Test` | tested behavior | production readiness |
| `Benchmark` | measurement output | benchmark threshold（admitted でない限り） |
| `RealDevice` | bounded command result | general live readiness |
| `ProductionReadiness` | bounded production readiness | live readiness |
| `LiveReadiness` | bounded live readiness | Kernel completion / freeze |

---

## 5. Evidence Wire Format

本節は evidence format を固定します。command success、test pass、benchmark threshold、production readiness、live readiness を主張しません。

### 5.1 Reason Wire Format

`DistroEvidenceReason` は `distro-support/evidence/src/reason.rs` に次の Rust enum として実装します。

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

Wire value は次に固定します。

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

`Unknown`、`Other`、raw `String`、untyped reason は禁止します（MUST NOT）。

### 5.2 Evidence Label Wire Format

次の enum は `distro-support/evidence/src/record.rs` が所有します。

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

`non_claim_scope` は自由文字列 list ではありません。`DistroNonClaimScope` の list です。

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

Wire value は次に固定します。

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

`distro-support/evidence/src/validation.rs` は validation error enum を所有します。

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

validator signature は次に固定します。

```rust
pub fn validate_evidence_record(
    record: &DistroEvidenceRecord,
) -> Result<(), EvidenceValidationError>
```

各 validation error は `DistroEvidenceReason::EvidenceFieldsIncomplete` に map します。ただし次は例外です。

| Validation error | Reason |
|---|---|
| `WorkingDirectoryOutsideDistro` | `CommandScopeMismatch` |
| `InvalidReadinessAdmission` | `ReadinessNotAdmitted` |
| `ForbiddenUnclassifiedReason` | `EvidenceFieldsIncomplete` |

---

## 6. Evidence Extension Record

本節は、benchmark / real-device / readiness evidence が `DistroEvidenceRecord` をどう拡張するかを Rust struct と JSON shape で固定します。

Rust には struct inheritance がないため、すべての extension record は `#[serde(flatten)] base: DistroEvidenceRecord` を持たなければなりません（MUST）。JSON では base field と extension field が同一 object の top-level field として serialize されます。本節は evidence shape を固定します。benchmark threshold、real-device success、production readiness、live readiness を主張しません。

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

Validation は次の場合に record を reject します。

- `base.command_class != DistroCommandClass::Benchmark`
- `base.non_claim_scope` が `BenchmarkThresholdNotClaimed` を欠く
- `scenario_id` が benchmark scenario workload authority に listed されていない
- `scenario_name` が `scenario_id` の scenario table row と一致しない
- `layer != base.distro_layer`
- `plane != base.target_plane`
- `layer` または `plane` が `scenario_id` の scenario table row と一致しない
- `workload_summary` が `scenario_id` の scenario table row と一致しない
- `criterion_group != "distro_benchmark_scenarios"`
- `sample_size != 100`
- `warm_up_seconds != 3`
- `measurement_seconds != 10`
- `noise_threshold != 0.05`
- `confidence_level != 0.95`
- `significance_level != 0.05`
- いずれかの Criterion 設定 float が finite でない
- いずれかの timing value が負値または finite でない
- `throughput_items_per_second` が finite でない、または `0.0` 以下

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

Validation は次の場合に record を reject します。

- `base.command_class != DistroCommandClass::RealDevice`
- `base.non_claim_scope` が `NativeApplicationReadinessNotClaimed`、`PublicDistributionReadinessNotClaimed`、`ProductionReadinessNotClaimed`、`LiveReadinessNotClaimed` を欠く
- `platform`、`device_class`、`internal_command_class` が real-device wrapper command authority と一致しない
- `runtime_version_class`、`execution_surface`、`network_class` のいずれかが device context matrix と一致しない
- Android / iOS platform tool path で `platform_command` が absent
- browser wrapper-local capability path で `platform_command` が present
- platform tool 使用時に `platform_command` が platform command closed set にない
- `base.exit_status` が absent
- `base.actual_outcome` が空
- `preflight_outcome` が空
- `logs_metrics_location` が空
- `logs_metrics_location` が `target/distro-evidence/real-device/` の外
- `base.exit_status == Some(0)` のとき `redacted_device_identifier` が absent
- `base.exit_status != Some(0)` のとき `redacted_device_identifier` が present
- `redacted_device_identifier` が trim 後に空
- `redacted_device_identifier` が literal `redacted` でも `sha256:<64 lowercase hex characters>` でもない
- `redacted_device_identifier` が raw identifier marker のいずれかを含む: `serial:`, `serial=`, `udid:`, `udid=`, `android_id:`, `android_id=`, `device_id:`, `device_id=`, `imei:`, `imei=`, `meid:`, `meid=`, `account:`, `account=`, `token:`, `token=`, `private_key:`, `private_key=`, `device_name:`, `device_name=`

post-execution field の requiredness matrix は real-device wrapper command authority が governs します。

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

Benchmark validation error の reason mapping:

| Benchmark validation error | Distro reason |
|---|---|
| `Base(error)` | `EvidenceValidationError` 用 mapping（本章 5.5） |
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

Real-device validation error の reason mapping:

| Real-device validation error | Distro reason |
|---|---|
| `Base(error)` | `EvidenceValidationError` 用 mapping（本章 5.5） |
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

Validation は次の場合に record を reject します。

- `base.command_class` が `readiness_claim` と一致しない
- `readiness_gate_id` が readiness matrix authority に listed されていない
- `readiness_claim == ProductionReadiness` で `readiness_gate_id` が `PRD-001` から `PRD-009` のいずれでもない
- `readiness_claim == LiveReadiness` で `readiness_gate_id` が `LIVE-001` から `LIVE-008` のいずれでもない
- `readiness_gate_id` が `PRD-003` で `context.auth_provider_authority == Absent`
- `readiness_gate_id` が `PRD-004` で `context.persistence_provider_authority == Absent`
- `readiness_claim == LiveReadiness` で `context.live_endpoint_authority == Absent`
- `readiness_gate_id` が `LIVE-001` で `context.production_readiness_report == Absent`
- `readiness_gate_id` が `LIVE-003` で `context.public_traversal_authority == Absent`
- `readiness_adr_ref != "READINESS_CLAIM_BOUNDARY"`
- `readiness_canonical_ref != "READINESS_MATRIX"`
- `readiness_claim == ProductionReadiness` で `base.non_claim_scope` が `LiveReadinessNotClaimed` を欠く
- `readiness_claim == LiveReadiness` で `base.non_claim_scope` が `KernelCompletionNotClaimed` または `KernelFreezeNotClaimed` を欠く
- readiness matrix authority の `Gate id Required Extension Ref Matrix` が要求する ref field が absent
- 要求された ref field が present だが trim 後に空
- `Gate id Required Extension Ref Matrix` が許可しない ref field が populated
- いずれかの readiness extension ref が raw secret、raw token、private key、raw packet payload、direct personal identifier を含む

readiness validator signature、閉 validation error set、validation-error-to-reason mapping は本章 1 節（readiness extension 所有）が所有します。`tests/production-readiness/tests/*.rs` は、opposite claim の fail-closed reject を明示的に assert する場合を除き、`readiness_claim == ProductionReadiness` の record のみを構築・validate できます。`tests/live/tests/*.rs` は、同様に `readiness_claim == LiveReadiness` の record のみを構築・validate できます。いずれの readiness test package も `ReadinessClaim` enum や `ReadinessEvidenceRecord` struct を所有しません。

### 6.5 Readiness Extension Context と Validator

Readiness admission context は次に固定します。

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

Validator signature は次に固定します。

```rust
pub fn validate_readiness_evidence_record(
    record: &ReadinessEvidenceRecord,
    context: &ReadinessValidationContext,
) -> Result<(), ReadinessEvidenceValidationError>
```

`ReadinessEvidenceValidationError` は次に固定します。

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

各 readiness validation error は `DistroEvidenceReason` に次のように map します。

| Readiness validation error | Distro reason |
|---|---|
| `Base(error)` | `EvidenceValidationError` 用 mapping（本章 5.5） |
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

gate id / required ref validation は readiness matrix authority を authority とします。auth provider authority absent、persistence provider authority absent、live endpoint authority absent、production readiness report absent、public traversal authority absent は、`MissingRequiredExtensionRef` ではなく上記の専用 not-admitted variant を使わなければなりません（MUST）。

### 6.6 Writer Rule

extension evidence writer は、extension JSON を書く前に `validate_evidence_record(&record.base)` と extension-specific validator を呼ばなければなりません（MUST）。base validation が失敗した場合、extension writer は map された `DistroEvidenceReason` を返し、JSON を書きません（fail-closed）。readiness の場合、extension-specific validator は `arcrtc-distro-evidence` の `validate_readiness_evidence_record(&record, &context)` です。context type と admission-state closed set は本章 1 節が所有します。

extension writer は、second base schema を作る、base field を手動で複製する、base を nested `base` object として serialize することのいずれもしてはなりません（MUST NOT）。

---

## 7. 不変条件と fail-closed 条件（Collapse Conditions）

本章の正典は、次のいずれかが起きた場合に崩れます。これらは fail-closed の境界であり、いずれも禁止です（MUST NOT）。

### 7.1 Reason / Ownership 不変条件

- `DistroEvidenceReason` を package ごとに再定義する。
- `DistroNonClaimScope` を raw string list として再定義する。
- `arcrtc-reference-ops` を reason owner として扱い、reference plane package から逆依存させる。
- product package が `arcrtc-product-monitoring` の local enum を独立 reason authority として使う。
- shared evidence package が Kernel reason catalog または Kernel semantic type を所有する。
- evidence writer success を command success / readiness success として扱う。
- `UNKNOWN` reason を正式 evidence に採用する。
- distro reason が Kernel reason catalog を変更する。
- correlation id なしの output を close 根拠にする。
- diagnostic log を readiness evidence として採用する。

### 7.2 Error Mapping 不変条件

- error enum に `Unknown` / `Other` / raw `String` variant を追加する。
- dependency raw error を evidence reason として採用する。
- product error を Kernel reason catalog に昇格する。
- `ReadinessNotAdmitted` を success として扱う。
- mapping されない error variant を追加する。
- package-local `DistroEvidenceReason` enum を再定義する。

### 7.3 Schema / Wire Format 不変条件

- evidence record が required JSON field を欠く。
- raw secret、raw token、raw packet payload、private key を evidence field に含める。
- `non_claim_scope` を free string list として受け入れる。
- command class の claim boundary を越えて evidence を転用する。
- Kernel working directory の evidence を distro evidence として採用する。
- `DistroEvidenceRecord` を reference / product package ごとに再定義する。
- enum JSON value を本章と異なる文字列で serialize する。
- command class ごとの required non-claim scope を validation しない。
- readiness class を必要な readiness admission record なしに valid とする。

### 7.4 Extension Record 不変条件

- extension evidence record が `#[serde(flatten)] base` を省略する。
- extension JSON が base evidence を `base` object の下に nest する。
- extension writer が `validate_evidence_record` を skip する。
- benchmark / real-device / readiness extension field を、専用の admission record なしに readiness / threshold proof として扱う。
