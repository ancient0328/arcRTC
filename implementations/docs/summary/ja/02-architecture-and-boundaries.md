# アーキテクチャと境界

状態: SSOT 統合版
日付: 2026-06-28 JST

## 目的

本章は、arcRTC v0.2 の implementations 領域のアーキテクチャ構造、すなわち root model、4 層構造（implementation-support / reference-implementation / product-implementation / tests）の責務、依存規則、workspace manifest と dependency path 規則、module export 境界（公開 / 非公開の閉集合）、source / package 境界、branch identity 境界、および collapse conditions を、他文書を参照せずに再現実装可能な粒度で確定することを目的とします。本章の規則・語彙・境界はすべて本仕様書内で完結します。

## Root Model

implementations 領域の root model は次の構造です。

```text
.
├── Kernel/
└── implementations/
    ├── implementation-support/
    ├── reference-implementation/
    ├── product-implementation/
    └── tests/
```

implementations は Kernel 外の独立 source root として扱います。Kernel は `Kernel/` に閉じ、implementations は Kernel source を変更しません（禁止）。

## Dependency Rule（依存規則）

依存方向は次に固定します。

```text
implementations -> Kernel public contract
implementations -> Kernel SDK projection
implementations -> documented Kernel command surface

Kernel -X-> implementations
implementations -X-> Kernel semantic authority overwrite
```

- implementations は Kernel public contract / SDK projection / documented command surface へ依存できます（許可）。
- Kernel は implementations へ依存しません（禁止／不変条件）。
- implementations は Kernel semantic authority を上書きしません（禁止／不変条件）。

## 4 層の責務

### Implementation Support Layer

implementation support は、reference / product / test が共有する implementation-local type を置く層です。

| Surface | 責務 |
|---|---|
| `implementation-support/evidence/` | evidence reason / record / validation / common label enum |

implementation support は Kernel reason catalog、Kernel semantics、product policy、readiness claim を所有しません（禁止）。

### Reference Implementation Layer

reference implementation は、Kernel frozen contract を使う最小実装層です。

| Surface | 責務 |
|---|---|
| `reference-implementation/signaling/` | session / room / control-plane wiring |
| `reference-implementation/turn/` | allocation / permission / relay wiring |
| `reference-implementation/sfu/` | routing / forwarding / quality wiring |
| `reference-implementation/output/` | reference output outcome types; product input subset は 3-type allow-list |
| `reference-implementation/composition/` | Signaling / TURN / SFU の結合 |
| `reference-implementation/deployment-profiles/` | local / controlled benchmark profile |
| `reference-implementation/ops/` | reference execution helper |

reference implementation は production readiness を主張しません（禁止）。

### Product Implementation Layer

product implementation は、`arcrtc-reference-output` の `ReferenceSignalingOutcome` / `ReferenceTurnOutcome` / `ReferenceSfuOutcome` だけを入力として消費し、product policy、deployment、monitoring、persistence topology、rollback を別 surface で所有する層です。

product implementation は `ReferenceCompositionOutcome`、reference source、internal state、reference function、success claim を継承しません（禁止）。product implementation は reference implementation を fork / copy して product policy を混入しません（禁止）。

| Surface | 責務 |
|---|---|
| `product-implementation/signaling/` | product Signaling runtime composition |
| `product-implementation/turn/` | product TURN runtime composition |
| `product-implementation/sfu/` | product SFU runtime composition |
| `product-implementation/product-policy/` | product-specific policy |
| `product-implementation/persistence-topology/` | product persistence topology |
| `product-implementation/deployment/` | deployment packaging / profile |
| `product-implementation/monitoring/` | SLO / metrics / operational probes |
| `product-implementation/rollback/` | rollback / drain / restore operation |

product readiness / production readiness / live readiness は、それぞれ implementations 側の evidence と verdict でのみ扱います。

### Test and Evidence Layer

| Surface | 責務 |
|---|---|
| `tests/reference/` | reference implementation behavior evidence |
| `tests/product/` | product implementation behavior evidence |
| `tests/production-readiness/` | production readiness evidence |
| `tests/real-device/` | bounded Android / iOS / browser real-device command evidence; not live readiness |
| `tests/live/` | live readiness evidence; not real-device command evidence |

## Source / Package 境界（owns / must not own 全表）

source package 境界は次に固定します。各 surface は左欄を所有し、右欄を所有してはなりません（禁止）。

| Package / module surface | Owns | Must not own |
|---|---|---|
| `reference-implementation/signaling/` | reference Signaling runtime wrapper / mapper | product policy / Kernel semantics |
| `reference-implementation/turn/` | reference TURN runtime wrapper / mapper | product network policy / Kernel semantics |
| `reference-implementation/sfu/` | reference SFU runtime wrapper / mapper | product media policy / Kernel semantics |
| `reference-implementation/output/` | reference output outcome types; product input subset は 3-type allow-list | reference state mutation / fixture validation / local auth / runtime execution / composition binding / product policy / readiness claim |
| `reference-implementation/composition/` | reference plane composition | product deployment / readiness claim |
| `reference-implementation/deployment-profiles/` | local / benchmark profile definition | product deployment authority |
| `reference-implementation/ops/` | reference execution helpers | production operation authority |
| `product-implementation/signaling/` | product Signaling runtime composition | Kernel Signaling semantics |
| `product-implementation/turn/` | product TURN runtime composition | Kernel TURN semantics |
| `product-implementation/sfu/` | product SFU runtime composition | Kernel SFU semantics |
| `product-implementation/product-policy/` | tenant / quota / admission policy | protocol semantics |
| `product-implementation/persistence-topology/` | product persistence topology | Kernel persistence port ownership |
| `product-implementation/deployment/` | deployment profile / packaging | Kernel final Closed Gate |
| `product-implementation/monitoring/` | metrics / logs / probes / SLO observation | Kernel evidence authority |
| `product-implementation/rollback/` | rollback / drain / restore operation | Kernel freeze claim |
| `tests/boundary/` | docs / dependency / export boundary test source | implementation source completion proof |
| `tests/reference/` | reference implementation test source | production readiness proof |
| `tests/product/` | product behavior test source | live readiness proof |
| `tests/benchmark/` | benchmark scenario / Criterion evidence test source | benchmark threshold proof |
| `tests/real-device/` | bounded real-device wrapper test source | native application readiness proof |
| `tests/production-readiness/` | production readiness evidence source | live readiness proof |
| `tests/live/` | live readiness evidence source | Kernel completion proof |

### Package Form Rule

- reference と product を同一 package ownership にしない（禁止）。
- test / benchmark / readiness source を implementation source と同一 owner にしない（禁止）。
- Kernel source を implementations package に含めない（禁止）。
- shared helper を作る場合、`reference` / `product` / `tests` の owner を上書きしない（禁止）。

### Shared Code Rule

shared code は次に限定します（許可集合）。

| Shared class | 許可される用途 |
|---|---|
| implementation-local type projection | Kernel output を implementation-facing evidence shape へ投影する |
| profile loader | reference / product profile を読み込む |
| command runner helper | local command orchestration を補助する |
| evidence serializer | reportable output を生成する |

shared code は Kernel semantics、product policy、readiness conclusion を所有しません（禁止）。

## Workspace Manifest 規則

### Workspace Root Manifest

`implementations/Cargo.toml` は次の形に固定します。

```toml
[workspace]
resolver = "2"
members = [
  "implementation-support/evidence",
  "reference-implementation/output",
  "reference-implementation/signaling",
  "reference-implementation/turn",
  "reference-implementation/sfu",
  "reference-implementation/composition",
  "reference-implementation/ops",
  "product-implementation/signaling",
  "product-implementation/turn",
  "product-implementation/sfu",
  "product-implementation/product-policy",
  "product-implementation/persistence-topology",
  "product-implementation/deployment",
  "product-implementation/monitoring",
  "product-implementation/rollback",
]

[workspace.package]
edition = "2021"
rust-version = "1.96"
license = "Apache-2.0"
publish = true

[workspace.lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"

[workspace.metadata.arcrtc]
root = "implementations"
kernel_root = "Kernel"
authority = "implementations"
kernel_mutation = "forbidden"
```

`reference-implementation/deployment-profiles/` と `tests/` 配下は、この workspace member list に含めません（禁止）。test package は `tests/` 配下で別途所有します。

### Package Manifest Common Shape

各 package の `Cargo.toml` は次の common shape を持ちます。

```toml
[package]
name = "<package-name>"
version = "0.0.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
publish.workspace = true

[lib]
path = "src/lib.rs"

[lints]
workspace = true
```

package-specific dependency は `[dependencies]` に direct path / version requirement として書きます。workspace-wide dependency alias は初期状態では作りません。

## Dependency Path 規則

### Kernel Local Path Table

すべての implementations package から Kernel package への local path は、package directory 起点で `../../../Kernel/...` に固定します。

| Package | Required Kernel dependencies |
|---|---|
| `arcrtc-implementation-evidence` | none |
| `arcrtc-reference-output` | `arcrtc-core-signaling`, `arcrtc-core-turn`, `arcrtc-core-sfu`, `arcrtc-core-identity` |
| `arcrtc-reference-signaling` | `arcrtc-core-signaling`, `arcrtc-core-command`, `arcrtc-core-identity`, `arcrtc-core-reason`, `arcrtc-core-ports`, `arcrtc-core-protocol`, `arcrtc-core-configuration` |
| `arcrtc-reference-turn` | `arcrtc-core-turn`, `arcrtc-core-identity`, `arcrtc-core-transport`, `arcrtc-core-security`, `arcrtc-core-time`, `arcrtc-core-reason`, `arcrtc-core-ports`, `arcrtc-core-configuration` |
| `arcrtc-reference-sfu` | `arcrtc-core-sfu`, `arcrtc-core-identity`, `arcrtc-core-transport`, `arcrtc-core-quality`, `arcrtc-core-state`, `arcrtc-core-reason`, `arcrtc-core-ports`, `arcrtc-core-configuration` |
| `arcrtc-reference-composition` | `arcrtc-core-cross-plane`, `arcrtc-core-operation`, `arcrtc-core-runtime`, `arcrtc-core-reason`, `arcrtc-core-identity` |
| `arcrtc-reference-ops` | `arcrtc-core-command`, `arcrtc-core-operation`, `arcrtc-core-runtime`, `arcrtc-core-identity` |
| `arcrtc-product-signaling` | `arcrtc-core-signaling`, `arcrtc-core-command`, `arcrtc-core-identity`, `arcrtc-core-reason` |
| `arcrtc-product-turn` | `arcrtc-core-turn`, `arcrtc-core-transport`, `arcrtc-core-security`, `arcrtc-core-identity`, `arcrtc-core-reason` |
| `arcrtc-product-sfu` | `arcrtc-core-sfu`, `arcrtc-core-transport`, `arcrtc-core-quality`, `arcrtc-core-state`, `arcrtc-core-identity`, `arcrtc-core-reason` |
| `arcrtc-product-policy` | `arcrtc-core-command`, `arcrtc-core-identity`, `arcrtc-core-reason` |
| `arcrtc-product-persistence-topology` | `arcrtc-core-state`, `arcrtc-core-reason` |
| `arcrtc-product-deployment` | `arcrtc-core-configuration`, `arcrtc-core-operation`, `arcrtc-core-runtime`, `arcrtc-core-identity` |
| `arcrtc-product-monitoring` | `arcrtc-core-audit`, `arcrtc-core-quality`, `arcrtc-core-identity`, `arcrtc-core-reason` |
| `arcrtc-product-rollback` | `arcrtc-core-recovery`, `arcrtc-core-operation`, `arcrtc-core-identity`, `arcrtc-core-reason` |

各 Kernel dependency は `<name> = { path = "../../../Kernel/core/<dir>" }` の literal path で記述します（例: `arcrtc-core-signaling = { path = "../../../Kernel/core/signaling" }`）。

### Reference Local Path Table

implementation-support / reference / product package 間の local path は次に固定します。

| Package | Required local dependencies |
|---|---|
| `arcrtc-implementation-evidence` | none |
| `arcrtc-reference-output` | `arcrtc-implementation-evidence` (`../../implementation-support/evidence`) |
| `arcrtc-reference-signaling` | `arcrtc-implementation-evidence`, `arcrtc-reference-output` (`../output`) |
| `arcrtc-reference-turn` | `arcrtc-implementation-evidence`, `arcrtc-reference-output` (`../output`) |
| `arcrtc-reference-sfu` | `arcrtc-implementation-evidence`, `arcrtc-reference-output` (`../output`) |
| `arcrtc-reference-composition` | `arcrtc-implementation-evidence`, `arcrtc-reference-output` (`../output`), `arcrtc-reference-signaling` (`../signaling`), `arcrtc-reference-turn` (`../turn`), `arcrtc-reference-sfu` (`../sfu`) |
| `arcrtc-reference-ops` | `arcrtc-implementation-evidence`, `arcrtc-reference-composition` (`../composition`) |
| `arcrtc-product-signaling` | `arcrtc-implementation-evidence`, `arcrtc-reference-output` (`../../reference-implementation/output`), `arcrtc-product-policy` (`../product-policy`) |
| `arcrtc-product-turn` | `arcrtc-implementation-evidence`, `arcrtc-reference-output` (`../../reference-implementation/output`), `arcrtc-product-policy` (`../product-policy`) |
| `arcrtc-product-sfu` | `arcrtc-implementation-evidence`, `arcrtc-reference-output` (`../../reference-implementation/output`), `arcrtc-product-policy` (`../product-policy`) |
| `arcrtc-product-policy` | `arcrtc-implementation-evidence` |
| `arcrtc-product-persistence-topology` | `arcrtc-implementation-evidence` |
| `arcrtc-product-deployment` | `arcrtc-implementation-evidence`, `arcrtc-product-rollback` (`../rollback`) |
| `arcrtc-product-monitoring` | `arcrtc-implementation-evidence` |
| `arcrtc-product-rollback` | `arcrtc-implementation-evidence` |

product policy / persistence-topology / deployment / monitoring / rollback は、初期状態では reference package dependency を持ちません。product plane package は `arcrtc-reference-output` 以外の reference package dependency を持ってはなりません（禁止）。

### External Dependency Placement

external dependency は次の owner package にのみ置きます。

| Dependency | Owner package |
|---|---|
| `tokio` | `arcrtc-reference-ops`, `arcrtc-product-deployment`, `arcrtc-product-rollback` |
| `serde` | `arcrtc-implementation-evidence`, `arcrtc-reference-ops`, `arcrtc-product-monitoring`, `arcrtc-product-deployment`, `arcrtc-product-persistence-topology` |
| `serde_json` | `arcrtc-reference-ops`, `arcrtc-product-monitoring` |
| `tracing` | `arcrtc-reference-ops`, `arcrtc-product-monitoring` |
| `tracing-subscriber` | `arcrtc-reference-ops` |
| `clap` | `arcrtc-reference-ops` |
| `criterion` | benchmark package only; implementation package に入れない（禁止） |

`arcrtc-implementation-evidence` が canonical evidence serialization owner です。`arcrtc-reference-ops`、`arcrtc-product-monitoring`、`arcrtc-product-deployment`、`arcrtc-product-persistence-topology` の `serde` は、それぞれ config / wrapper / product-local projection struct の serialization に限定し、`ImplementationEvidenceRecord` の独立再定義に使ってはなりません（禁止）。

各 package の complete `[dependencies]` section は次の 3 つの union です。external dependency fragment を complete table と誤読し、local path dependency を落としてはなりません（禁止）。duplicate `[dependencies]` table を作成してはなりません（禁止）。

1. Kernel Local Path Table
2. Reference Local Path Table
3. その package の External Dependency Literal Stanza

## Module Export 境界（公開 / 非公開の閉集合）

各 `src/lib.rs` は次の順序に固定します。

1. crate-level docs
2. `pub mod ...;`
3. `pub use ...;`

private module を初期 scaffold に作りません。`pub use` は、以下の公開 export 閉集合で固定された、外部から呼ぶ型・関数だけに限定します。当該閉集合に未記載の public module を追加してはなりません（禁止）。private helper を `pub use` して public API surface に出してはなりません（禁止）。

各 package の公開 export 閉集合は次です。

| Package | Public modules | 主な public re-export |
|---|---|---|
| `arcrtc-implementation-evidence` | `error`, `reason`, `record`, `readiness_extension`, `validation` | `ImplementationEvidenceError`, `ImplementationEvidenceReason`, `ImplementationEvidenceRecord`, `ImplementationCommandClass`, `ImplementationEnvironmentClass`, `ImplementationLayer`, `ImplementationNonClaimScope`, `ImplementationPlane`, `validate_evidence_record`, `EvidenceValidationError`, `IMPLEMENTATIONS_COMMAND_ROOT`, `IMPLEMENTATIONS_EVIDENCE_ROOT`, `IMPLEMENTATIONS_TARGET_ROOT`, readiness 系（`validate_readiness_evidence_record`, `ReadinessAdmissionState`, `ReadinessClaim`, `ReadinessEvidenceRecord`, `ReadinessEvidenceValidationError`, `ReadinessValidationContext`） |
| `arcrtc-reference-output` | `composition`, `error`, `sfu`, `signaling`, `turn` | `ReferenceCompositionOutcome`, `ReferenceOutputError`, `ReferenceSfuOutcome`, `ReferenceSignalingOutcome`, `ReferenceTurnOutcome` |
| `arcrtc-reference-signaling` | `error`, `fixture_identity`, `kernel_contract`, `local_auth`, `state` | `ReferenceSignalingOutcome`（`arcrtc_reference_output` から re-export）, `ReferenceSignalingError`, `FixtureIdentity`, `FixtureSessionDescription`, `FixtureIceCandidate`, `build_kernel_signaling_command`, `ReferenceSignalingCommandInput`, `ReferenceSignalingPayload`, `authorize_reference_signaling`, `ReferenceLocalAuthDecision`, state 系（`apply_reference_signaling` 他） |
| `arcrtc-reference-turn` | `error`, `fixture_credential`, `kernel_contract`, `state` | `ReferenceTurnOutcome`（re-export）, `ReferenceTurnError`, `FixtureTurnCredential`, `validate_fixture_turn_credential`, `build_kernel_turn_command`, `ReferenceTurnCommandInput`, state 系 |
| `arcrtc-reference-sfu` | `error`, `fixture_route_auth`, `kernel_contract`, `state` | `ReferenceSfuOutcome`（re-export）, `ReferenceSfuError`, `authorize_reference_route`, `FixtureRouteAdmission`, `build_borrowed_packet_view`, `build_kernel_sfu_item`, `ReferenceSfuContractInput`, state 系 |
| `arcrtc-reference-composition` | `composition_state`, `error`, `runtime_bridge`, `step_input` | `ReferenceCompositionOutcome`（re-export）, composition_state 系, `ReferenceCompositionError`, `run_reference_composition_step` 他, `ReferenceCompositionStep`, `ReferenceCompositionStepInput` |
| `arcrtc-reference-ops` | `error`, `evidence`, `reason`, `runtime` | `arcrtc_implementation_evidence` の evidence 型 / ROOT 定数の re-export, `ReferenceRuntimeError`, evidence 系, `implementation_reason_closed_set`, runtime 系 |
| `arcrtc-product-signaling` | `error`, `kernel_contract` | `ProductSignalingError`, `apply_product_signaling_policy`, `build_live_product_signaling_runtime`, `build_product_signaling_runtime`, `ProductSignalingOutcome`, `ProductSignalingPolicyInput`, `ProductSignalingRuntime` |
| `arcrtc-product-turn` | `error`, `kernel_contract` | `ProductTurnError`, `apply_product_turn_policy`, `build_live_product_turn_runtime`, `build_product_turn_runtime`, `ProductTurnOutcome`, `ProductTurnPolicyInput`, `ProductTurnRuntime` |
| `arcrtc-product-sfu` | `error`, `kernel_contract` | `ProductSfuError`, `apply_product_sfu_policy`, `build_live_product_sfu_runtime`, `build_product_sfu_runtime`, `ProductSfuOutcome`, `ProductSfuPolicyInput`, `ProductSfuRuntime` |
| `arcrtc-product-policy` | `auth_policy`, `error`, `provider_admission`, `security_reason` | `evaluate_product_auth_policy`, `ProductAction`, `ProductPolicyDecision`, `ProductPolicyInput`, `ProductPolicyError`, `admit_product_auth_provider`, `ProductAuthProviderAdmission`, `ProductAuthProviderClass`, `map_product_security_reason` |
| `arcrtc-product-persistence-topology` | `error`, `mapper`, `provider_admission`, `topology` | `ProductPersistenceTopologyError`, `map_product_projection`, `ProductProjectionMapping`, `admit_product_persistence_provider`, `ProductPersistenceProviderAdmission`, `ProductPersistenceProviderClass`, topology 系 |
| `arcrtc-product-deployment` | `error`, `live_endpoint`, `profile`, `production_profile`, `runtime` | `ProductRuntimeError`, live_endpoint 系, profile 系, `build_product_production_profile`, runtime 系 |
| `arcrtc-product-monitoring` | `error`, `evidence`, `live_probe`, `observability`, `production_probe` | `ProductMonitoringError`, evidence 系, `build_live_monitoring_probe`, `ProductLiveMonitoringProbe`, observability 系, production_probe 系 |
| `arcrtc-product-rollback` | `drain`, `error`, `live_operation`, `production_operation`, `restore` | drain 系, `ProductRollbackError`, `execute_live_restore`, `execute_live_shutdown_drain`, `plan_production_drain`, `plan_production_restore`, restore 系 |

## Branch Identity 境界

implementations の source / test snapshot は、`arcRTC` repository の `implementations/` subtree として Git 管理下に公開します。

| Item | Rule |
|---|---|
| remote repository | `https://github.com/ancient0328/arcRTC.git` |
| tracked source root | `implementations/` |
| admitted tracked content | implementations production source, support source, test source, workspace manifests, lockfiles, README |

repository commit は source / test snapshot identity としてだけ採用します。repository commit を仕様書・report authority として採用してはなりません（禁止）。repository commit を Kernel completion、Kernel freeze、production readiness、live readiness、benchmark threshold satisfaction の根拠として採用してはなりません（禁止）。

## Collapse Conditions（崩壊条件）

本章のアーキテクチャと境界は次の場合に崩れます。

- implementations が Kernel core semantics を所有する。
- implementations が Kernel port definition を変更する。
- reference implementation を production readiness として扱う。
- Kernel evidence を implementations readiness proof として扱う。
- production / live readiness を専用 evidence と verdict なしに主張する。
- reference / product / tests / readiness source が同一 owner file に混在する。
- shared helper が Kernel semantics または product policy を所有する。
- Kernel source が implementations package にコピーされる。
- product plane が `arcrtc-reference-output` の 3 outcome 型以外の reference package public API、reference state、reference function、または `ReferenceCompositionOutcome` を入力として消費する。
- implementations package を Kernel workspace member に追加する。
- 本章にない Kernel local path を package manifest に追加する。
- workspace dependency alias で dependency admission の owner package を曖昧にする。
- external dependency fragment を complete `[dependencies]` table と誤読し、local path dependency を落とす。
- duplicate `[dependencies]` table を作成する。
- `criterion` を implementation package の runtime dependency に入れる。
- `reference-implementation/deployment-profiles/` または `tests/` を implementation workspace member に入れる。
- `ImplementationEvidenceReason` を使う package が `arcrtc-implementation-evidence` dependency を持たない。
- `src/lib.rs` に公開 export 閉集合 未記載の public module を追加する。
- private helper を `pub use` して public API surface に出す。
- product package が reference package の internal module を re-export する。
- product package が `arcrtc-reference-output` 以外の reference package symbol を import / re-export する。
- reference package が `ReferenceSignalingOutcome` / `ReferenceTurnOutcome` / `ReferenceSfuOutcome` / `ReferenceCompositionOutcome` を `arcrtc-reference-output` 外で重複定義する。
- evidence / runtime / policy / persistence の export を同一 module に混在させる。
- repository commit を仕様書・report authority として扱う。
- repository commit を Kernel completion / freeze / production readiness / live readiness / benchmark threshold satisfaction の根拠として扱う。
