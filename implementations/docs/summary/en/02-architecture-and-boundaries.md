# Architecture and Boundaries

Status: SSOT consolidated edition
Date: 2026-06-28 JST

## Purpose

This chapter fixes the architecture structure of the arcRTC v0.2 implementations domain: the root model, the responsibilities of the 4-layer structure (implementation-support / reference-implementation / product-implementation / tests), the dependency rule, the workspace manifest and dependency path rules, the module export boundary (the closed set of public / non-public), the source / package boundary, the branch identity boundary, and the collapse conditions, at a granularity that is re-implementable without referring to any other document. All rules, vocabulary, and boundaries in this chapter are self-contained within this specification.

## Root Model

The root model of the implementations domain is the following structure.

```text
.
├── Kernel/
└── implementations/
    ├── implementation-support/
    ├── reference-implementation/
    ├── product-implementation/
    └── tests/
```

implementations is treated as an independent source root outside the Kernel. The Kernel is confined to `Kernel/`, and implementations MUST NOT modify the Kernel source.

## Dependency Rule

The dependency direction is fixed as follows.

```text
implementations -> Kernel public contract
implementations -> Kernel SDK projection
implementations -> documented Kernel command surface

Kernel -X-> implementations
implementations -X-> Kernel semantic authority overwrite
```

- implementations MAY depend on the Kernel public contract / SDK projection / documented command surface.
- The Kernel MUST NOT depend on implementations (invariant).
- implementations MUST NOT overwrite the Kernel semantic authority (invariant).

## Responsibilities of the 4 Layers

### Implementation Support Layer

implementation support is the layer that holds the implementation-local types shared by reference / product / test.

| Surface | Responsibility |
|---|---|
| `implementation-support/evidence/` | evidence reason / record / validation / common label enum |

implementation support MUST NOT own the Kernel reason catalog, Kernel semantics, product policy, or readiness claim.

### Reference Implementation Layer

reference implementation is the minimal implementation layer using the Kernel frozen contract.

| Surface | Responsibility |
|---|---|
| `reference-implementation/signaling/` | session / room / control-plane wiring |
| `reference-implementation/turn/` | allocation / permission / relay wiring |
| `reference-implementation/sfu/` | routing / forwarding / quality wiring |
| `reference-implementation/output/` | reference output outcome types; product input subset is the 3-type allow-list |
| `reference-implementation/composition/` | combination of Signaling / TURN / SFU |
| `reference-implementation/deployment-profiles/` | local / controlled benchmark profile |
| `reference-implementation/ops/` | reference execution helper |

reference implementation MUST NOT claim production readiness.

### Product Implementation Layer

product implementation is the layer that consumes only `ReferenceSignalingOutcome` / `ReferenceTurnOutcome` / `ReferenceSfuOutcome` of `arcrtc-reference-output` as input and owns product policy, deployment, monitoring, persistence topology, and rollback on separate surfaces.

product implementation MUST NOT inherit `ReferenceCompositionOutcome`, reference source, internal state, reference function, or success claim. product implementation MUST NOT fork / copy the reference implementation and mix in product policy.

| Surface | Responsibility |
|---|---|
| `product-implementation/signaling/` | product Signaling runtime composition |
| `product-implementation/turn/` | product TURN runtime composition |
| `product-implementation/sfu/` | product SFU runtime composition |
| `product-implementation/product-policy/` | product-specific policy |
| `product-implementation/persistence-topology/` | product persistence topology |
| `product-implementation/deployment/` | deployment packaging / profile |
| `product-implementation/monitoring/` | SLO / metrics / operational probes |
| `product-implementation/rollback/` | rollback / drain / restore operation |

product readiness / production readiness / live readiness are each handled only by implementations-side evidence and a verdict.

### Test and Evidence Layer

| Surface | Responsibility |
|---|---|
| `tests/reference/` | reference implementation behavior evidence |
| `tests/product/` | product implementation behavior evidence |
| `tests/production-readiness/` | production readiness evidence |
| `tests/real-device/` | bounded Android / iOS / browser real-device command evidence; not live readiness |
| `tests/live/` | live readiness evidence; not real-device command evidence |

## Source / Package Boundary (full owns / must not own table)

The source package boundary is fixed as follows. Each surface owns the left column and MUST NOT own the right column.

| Package / module surface | Owns | Must not own |
|---|---|---|
| `reference-implementation/signaling/` | reference Signaling runtime wrapper / mapper | product policy / Kernel semantics |
| `reference-implementation/turn/` | reference TURN runtime wrapper / mapper | product network policy / Kernel semantics |
| `reference-implementation/sfu/` | reference SFU runtime wrapper / mapper | product media policy / Kernel semantics |
| `reference-implementation/output/` | reference output outcome types; product input subset is the 3-type allow-list | reference state mutation / fixture validation / local auth / runtime execution / composition binding / product policy / readiness claim |
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

- MUST NOT place reference and product under the same package ownership.
- MUST NOT place test / benchmark / readiness source under the same owner as implementation source.
- MUST NOT include Kernel source in an implementations package.
- when creating a shared helper, MUST NOT overwrite the owner of `reference` / `product` / `tests`.

### Shared Code Rule

shared code is limited to the following (allowed set).

| Shared class | Allowed use |
|---|---|
| implementation-local type projection | project Kernel output into an implementation-facing evidence shape |
| profile loader | load reference / product profile |
| command runner helper | assist local command orchestration |
| evidence serializer | generate reportable output |

shared code MUST NOT own Kernel semantics, product policy, or readiness conclusion.

## Workspace Manifest Rule

### Workspace Root Manifest

`implementations/Cargo.toml` is fixed to the following form.

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

`reference-implementation/deployment-profiles/` and the subtree under `tests/` MUST NOT be included in this workspace member list. The test package is owned separately under `tests/`.

### Package Manifest Common Shape

Each package's `Cargo.toml` has the following common shape.

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

A package-specific dependency is written in `[dependencies]` as a direct path / version requirement. A workspace-wide dependency alias MUST NOT be created in the initial state.

## Dependency Path Rule

### Kernel Local Path Table

The local path from every implementations package to a Kernel package is fixed to `../../../Kernel/...` relative to the package directory.

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

Each Kernel dependency is written with the literal path `<name> = { path = "../../../Kernel/core/<dir>" }` (e.g. `arcrtc-core-signaling = { path = "../../../Kernel/core/signaling" }`).

### Reference Local Path Table

The local path among implementation-support / reference / product packages is fixed as follows.

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

product policy / persistence-topology / deployment / monitoring / rollback have no reference package dependency in the initial state. A product plane package MUST NOT have a reference package dependency other than `arcrtc-reference-output`.

### External Dependency Placement

An external dependency is placed only in the following owner package.

| Dependency | Owner package |
|---|---|
| `tokio` | `arcrtc-reference-ops`, `arcrtc-product-deployment`, `arcrtc-product-rollback` |
| `serde` | `arcrtc-implementation-evidence`, `arcrtc-reference-ops`, `arcrtc-product-monitoring`, `arcrtc-product-deployment`, `arcrtc-product-persistence-topology` |
| `serde_json` | `arcrtc-reference-ops`, `arcrtc-product-monitoring` |
| `tracing` | `arcrtc-reference-ops`, `arcrtc-product-monitoring` |
| `tracing-subscriber` | `arcrtc-reference-ops` |
| `clap` | `arcrtc-reference-ops` |
| `criterion` | benchmark package only; MUST NOT be placed in an implementation package |

`arcrtc-implementation-evidence` is the canonical evidence serialization owner. The `serde` of `arcrtc-reference-ops`, `arcrtc-product-monitoring`, `arcrtc-product-deployment`, and `arcrtc-product-persistence-topology` is limited to serialization of the config / wrapper / product-local projection struct respectively, and MUST NOT be used to independently redefine `ImplementationEvidenceRecord`.

The complete `[dependencies]` section of each package is the union of the following three. An external dependency fragment MUST NOT be misread as a complete table such that a local path dependency is dropped. A duplicate `[dependencies]` table MUST NOT be created.

1. Kernel Local Path Table
2. Reference Local Path Table
3. that package's External Dependency Literal Stanza

## Module Export Boundary (closed set of public / non-public)

Each `src/lib.rs` is fixed to the following order.

1. crate-level docs
2. `pub mod ...;`
3. `pub use ...;`

A private module MUST NOT be created in the initial scaffold. `pub use` is limited to only the types / functions called from outside as fixed by the public export closed set below. A public module not listed in that closed set MUST NOT be added. A private helper MUST NOT be exposed to the public API surface via `pub use`.

The public export closed set of each package is the following.

| Package | Public modules | Main public re-exports |
|---|---|---|
| `arcrtc-implementation-evidence` | `error`, `reason`, `record`, `readiness_extension`, `validation` | `ImplementationEvidenceError`, `ImplementationEvidenceReason`, `ImplementationEvidenceRecord`, `ImplementationCommandClass`, `ImplementationEnvironmentClass`, `ImplementationLayer`, `ImplementationNonClaimScope`, `ImplementationPlane`, `validate_evidence_record`, `EvidenceValidationError`, `IMPLEMENTATIONS_COMMAND_ROOT`, `IMPLEMENTATIONS_EVIDENCE_ROOT`, `IMPLEMENTATIONS_TARGET_ROOT`, readiness items (`validate_readiness_evidence_record`, `ReadinessAdmissionState`, `ReadinessClaim`, `ReadinessEvidenceRecord`, `ReadinessEvidenceValidationError`, `ReadinessValidationContext`) |
| `arcrtc-reference-output` | `composition`, `error`, `sfu`, `signaling`, `turn` | `ReferenceCompositionOutcome`, `ReferenceOutputError`, `ReferenceSfuOutcome`, `ReferenceSignalingOutcome`, `ReferenceTurnOutcome` |
| `arcrtc-reference-signaling` | `error`, `fixture_identity`, `kernel_contract`, `local_auth`, `state` | `ReferenceSignalingOutcome` (re-exported from `arcrtc_reference_output`), `ReferenceSignalingError`, `FixtureIdentity`, `FixtureSessionDescription`, `FixtureIceCandidate`, `build_kernel_signaling_command`, `ReferenceSignalingCommandInput`, `ReferenceSignalingPayload`, `authorize_reference_signaling`, `ReferenceLocalAuthDecision`, state items (`apply_reference_signaling` etc.) |
| `arcrtc-reference-turn` | `error`, `fixture_credential`, `kernel_contract`, `state` | `ReferenceTurnOutcome` (re-export), `ReferenceTurnError`, `FixtureTurnCredential`, `validate_fixture_turn_credential`, `build_kernel_turn_command`, `ReferenceTurnCommandInput`, state items |
| `arcrtc-reference-sfu` | `error`, `fixture_route_auth`, `kernel_contract`, `state` | `ReferenceSfuOutcome` (re-export), `ReferenceSfuError`, `authorize_reference_route`, `FixtureRouteAdmission`, `build_borrowed_packet_view`, `build_kernel_sfu_item`, `ReferenceSfuContractInput`, state items |
| `arcrtc-reference-composition` | `composition_state`, `error`, `runtime_bridge`, `step_input` | `ReferenceCompositionOutcome` (re-export), composition_state items, `ReferenceCompositionError`, `run_reference_composition_step` etc., `ReferenceCompositionStep`, `ReferenceCompositionStepInput` |
| `arcrtc-reference-ops` | `error`, `evidence`, `reason`, `runtime` | re-export of `arcrtc_implementation_evidence` evidence types / ROOT constants, `ReferenceRuntimeError`, evidence items, `implementation_reason_closed_set`, runtime items |
| `arcrtc-product-signaling` | `error`, `kernel_contract` | `ProductSignalingError`, `apply_product_signaling_policy`, `build_live_product_signaling_runtime`, `build_product_signaling_runtime`, `ProductSignalingOutcome`, `ProductSignalingPolicyInput`, `ProductSignalingRuntime` |
| `arcrtc-product-turn` | `error`, `kernel_contract` | `ProductTurnError`, `apply_product_turn_policy`, `build_live_product_turn_runtime`, `build_product_turn_runtime`, `ProductTurnOutcome`, `ProductTurnPolicyInput`, `ProductTurnRuntime` |
| `arcrtc-product-sfu` | `error`, `kernel_contract` | `ProductSfuError`, `apply_product_sfu_policy`, `build_live_product_sfu_runtime`, `build_product_sfu_runtime`, `ProductSfuOutcome`, `ProductSfuPolicyInput`, `ProductSfuRuntime` |
| `arcrtc-product-policy` | `auth_policy`, `error`, `provider_admission`, `security_reason` | `evaluate_product_auth_policy`, `ProductAction`, `ProductPolicyDecision`, `ProductPolicyInput`, `ProductPolicyError`, `admit_product_auth_provider`, `ProductAuthProviderAdmission`, `ProductAuthProviderClass`, `map_product_security_reason` |
| `arcrtc-product-persistence-topology` | `error`, `mapper`, `provider_admission`, `topology` | `ProductPersistenceTopologyError`, `map_product_projection`, `ProductProjectionMapping`, `admit_product_persistence_provider`, `ProductPersistenceProviderAdmission`, `ProductPersistenceProviderClass`, topology items |
| `arcrtc-product-deployment` | `error`, `live_endpoint`, `profile`, `production_profile`, `runtime` | `ProductRuntimeError`, live_endpoint items, profile items, `build_product_production_profile`, runtime items |
| `arcrtc-product-monitoring` | `error`, `evidence`, `live_probe`, `observability`, `production_probe` | `ProductMonitoringError`, evidence items, `build_live_monitoring_probe`, `ProductLiveMonitoringProbe`, observability items, production_probe items |
| `arcrtc-product-rollback` | `drain`, `error`, `live_operation`, `production_operation`, `restore` | drain items, `ProductRollbackError`, `execute_live_restore`, `execute_live_shutdown_drain`, `plan_production_drain`, `plan_production_restore`, restore items |

## Branch Identity Boundary

The implementations source / test snapshot is published under Git management as the `implementations/` subtree of the `arcRTC` repository.

| Item | Rule |
|---|---|
| remote repository | `https://github.com/ancient0328/arcRTC.git` |
| tracked source root | `implementations/` |
| admitted tracked content | implementations production source, support source, test source, workspace manifests, lockfiles, README |

A repository commit is adopted only as source / test snapshot identity. A repository commit MUST NOT be adopted as specification or report authority. A repository commit MUST NOT be adopted as the basis for Kernel completion, Kernel freeze, production readiness, live readiness, or benchmark threshold satisfaction.

## Collapse Conditions

The architecture and boundaries of this chapter collapse in the following cases.

- implementations owns Kernel core semantics.
- implementations modifies the Kernel port definition.
- the reference implementation is treated as production readiness.
- Kernel evidence is treated as implementations readiness proof.
- production / live readiness is claimed without dedicated evidence and a verdict.
- reference / product / tests / readiness source is mixed in the same owner file.
- a shared helper owns Kernel semantics or product policy.
- Kernel source is copied into an implementations package.
- a product plane consumes, as input, a reference package public API, reference state, reference function, or `ReferenceCompositionOutcome` other than the 3 outcome types of `arcrtc-reference-output`.
- an implementations package is added to the Kernel workspace member.
- a Kernel local path not in this chapter is added to a package manifest.
- a workspace dependency alias makes the dependency-admission owner package ambiguous.
- an external dependency fragment is misread as a complete `[dependencies]` table, dropping a local path dependency.
- a duplicate `[dependencies]` table is created.
- `criterion` is placed into an implementation package runtime dependency.
- `reference-implementation/deployment-profiles/` or `tests/` is placed into an implementation workspace member.
- a package using `ImplementationEvidenceReason` has no `arcrtc-implementation-evidence` dependency.
- a public module not listed in the public export closed set is added to `src/lib.rs`.
- a private helper is exposed to the public API surface via `pub use`.
- a product package re-exports an internal module of a reference package.
- a product package imports / re-exports a reference package symbol other than `arcrtc-reference-output`.
- a reference package duplicately defines `ReferenceSignalingOutcome` / `ReferenceTurnOutcome` / `ReferenceSfuOutcome` / `ReferenceCompositionOutcome` outside `arcrtc-reference-output`.
- evidence / runtime / policy / persistence exports are mixed in the same module.
- a repository commit is treated as specification or report authority.
- a repository commit is treated as the basis for Kernel completion / freeze / production readiness / live readiness / benchmark threshold satisfaction.
