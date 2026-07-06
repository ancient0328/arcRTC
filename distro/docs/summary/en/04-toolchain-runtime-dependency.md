# Toolchain / runtime / dependency

Status: SSOT consolidated edition
Date: 2026-06-28 JST

## Purpose

This chapter fixes completely the distro language / toolchain boundary, the runtime (executor) boundary, the workspace composition, and the closed set of dependency admission in the distro area of arcRTC v0.2 (the non-Kernel distro area). This chapter internalizes the SFU / TURN / Signaling primary distro language, the Rust workspace root / members, the ownership and non-ownership of the runtime executor, the Kernel local path dependency matrix, the distro-local dependency matrix, the external dependency allow-list, the required fields of the admission record, the prohibited patterns, and the fail-closed conditions. This chapter is self-contained and is written at a granularity sufficient for re-implementation without opening any other document.

## Dependency-direction invariant

A distro-side dependency is a technical dependency for consuming the Kernel contract and does not own Kernel semantics (`distro -> Kernel`, `Kernel -X-> distro`). Neither the runtime executor nor any external dependency MUST own Kernel semantic authority / Kernel reason catalog / Kernel port definition. Kernel semantics MUST NOT be changed for the convenience of a dependency API.

## 1. Distro language / toolchain boundary

The SFU / TURN / Signaling primary distro language of distro is fixed to Rust (MUST). The Kernel exists as a Rust workspace; the distro side does not change the Kernel and is designed as a source package that consumes the Kernel contract.

| Surface | Decision | Authority boundary |
|---|---|---|
| reference distro | Rust | owns the reference runtime behavior of SFU / TURN / Signaling |
| product distro | Rust as primary | product policy / persistence / deployment is owned on the product distro side |
| benchmark harness | Rust primary | executes comparable workloads of Kernel / distro |
| real-device harness | browser / native / shell harness MAY be added when required | limited to real-device command evidence; does not own distro authority |
| TypeScript / Swift / Kotlin | limited to harness / projection | not made an SFU / TURN / Signaling distro owner |

Rust packages MUST be placed under `distro/`. The Kernel source MUST NOT be copied to the distro side. Kernel crates are used in accordance with the version / path pin boundary.

### Toolchain rule (closed set)

| Toolchain | Rule |
|---|---|
| Rust / Cargo | distro primary toolchain |
| pnpm | used only when introducing a TypeScript / browser harness |
| npm / yarn | not used |
| Swift / Gradle | adopted only when a real-device / native harness is required |

The authority boundary for the toolchain version / command / working directory / package name is fixed in this chapter. The concrete command surface is specified in Chapter 14.

## 2. Workspace root

The Rust workspace root of distro is fixed as follows.

| Item | Fixed value |
|---|---|
| working directory | `distro/` |
| workspace manifest | `distro/Cargo.toml` |
| edition | `2021` |
| rust-version | `1.96` |
| resolver | `2` |
| license | `Apache-2.0` |
| publish | `true` |
| unsafe code | `forbid` |

A distro package MUST NOT be added to the Kernel workspace as a member. distro uses the Kernel local path dependency as a separate workspace.

## 3. Workspace members (closed set)

The initial workspace members are fixed as follows.

| Path | Package name | Role |
|---|---|---|
| `distro-support/evidence` | `arcrtc-distro-evidence` | shared distro evidence / reason types |
| `reference-distro/output` | `arcrtc-reference-output` | reference output outcome types; product input subset is the 3-type allow-list |
| `reference-distro/signaling` | `arcrtc-reference-signaling` | Signaling reference distro |
| `reference-distro/turn` | `arcrtc-reference-turn` | TURN reference distro |
| `reference-distro/sfu` | `arcrtc-reference-sfu` | SFU reference distro |
| `reference-distro/composition` | `arcrtc-reference-composition` | reference composition |
| `reference-distro/ops` | `arcrtc-reference-ops` | reference execution helper |
| `product-distro/signaling` | `arcrtc-product-signaling` | product Signaling distro |
| `product-distro/turn` | `arcrtc-product-turn` | product TURN distro |
| `product-distro/sfu` | `arcrtc-product-sfu` | product SFU distro |
| `product-distro/product-policy` | `arcrtc-product-policy` | product policy |
| `product-distro/persistence-topology` | `arcrtc-product-persistence-topology` | product persistence topology |
| `product-distro/deployment` | `arcrtc-product-deployment` | product deployment profile |
| `product-distro/monitoring` | `arcrtc-product-monitoring` | product monitoring / probes |
| `product-distro/rollback` | `arcrtc-product-rollback` | product rollback / drain |

`reference-distro/deployment-profiles/` is an area for runtime profile data and is not made a Rust package in the initial state. Packages under `tests/` are owned separately under `tests/`.

## 4. Runtime executor

The reference / product runtime executor adopts a Tokio-compatible runtime as the primary executor (MUST). The runtime executor is a distro-side dependency, not a Kernel dependency.

### What the runtime executor owns (closed set)

- task scheduling
- socket polling
- local timer / timeout
- process start / stop orchestration
- graceful shutdown signal handling
- benchmark / integration command execution profile

### What the runtime executor does not own (closed set)

- Kernel protocol semantics
- Kernel reason catalog
- Kernel port definition
- product readiness claim
- live readiness claim
- security policy authority

### Runtime surface owner files

| Runtime surface | Owner file to create | Owns |
|---|---|---|
| reference executor | `reference-distro/ops/src/runtime.rs` | local process orchestration, timer, shutdown signal |
| reference composition runner | `reference-distro/composition/src/runtime_bridge.rs` | Signaling / TURN / SFU local composition execution |
| product executor | `product-distro/deployment/src/runtime.rs` | product process profile |
| product shutdown / drain | `product-distro/rollback/src/drain.rs` | product drain orchestration |

## 5. Initial source file contract

`Cargo.toml` and `src/lib.rs` are required for every workspace package (MUST). The `Cargo.toml` literal shape is defined by the workspace manifest / dependency path rules in Chapter 02, and the `src/lib.rs` public export list is defined by the module export rules in Chapter 02.

`src/kernel_contract.rs` is not a universal file. It is required only for packages that directly adapt Kernel public contracts.

| Package class | Packages | Required extra files |
|---|---|---|
| distro support | `arcrtc-distro-evidence` | `src/reason.rs`, `src/record.rs`, `src/readiness_extension.rs`, `src/validation.rs`, `src/error.rs` |
| reference output | `arcrtc-reference-output` | `src/signaling.rs`, `src/turn.rs`, `src/sfu.rs`, `src/composition.rs`, `src/error.rs` |
| reference plane | `arcrtc-reference-signaling`, `arcrtc-reference-turn`, `arcrtc-reference-sfu` | `src/kernel_contract.rs`, `src/state.rs`, the fixture/auth files fixed for the reference distro, `src/error.rs` |
| reference composition | `arcrtc-reference-composition` | `src/step_input.rs`, `src/composition_state.rs`, `src/runtime_bridge.rs`, `src/error.rs` |
| reference ops | `arcrtc-reference-ops` | `src/runtime.rs`, `src/evidence.rs`, `src/reason.rs`, `src/error.rs` |
| product plane | `arcrtc-product-signaling`, `arcrtc-product-turn`, `arcrtc-product-sfu` | `src/kernel_contract.rs`, `src/error.rs` |
| product policy | `arcrtc-product-policy` | `src/auth_policy.rs`, `src/security_reason.rs`, `src/error.rs` |
| product persistence | `arcrtc-product-persistence-topology` | `src/topology.rs`, `src/mapper.rs`, `src/error.rs` |
| product deployment | `arcrtc-product-deployment` | `src/runtime.rs`, `src/profile.rs`, `src/error.rs` |
| product monitoring | `arcrtc-product-monitoring` | `src/observability.rs`, `src/evidence.rs`, `src/error.rs` |
| product rollback | `arcrtc-product-rollback` | `src/drain.rs`, `src/restore.rs`, `src/error.rs` |

`distro-support/evidence` does not have `src/kernel_contract.rs`. Product support packages also do not create `src/kernel_contract.rs` unless listed above. A binary entrypoint MUST NOT be created before reference / product completion. When a CLI / command binary is required, it is added in accordance with the command surface fixed in Chapter 14.

## 6. Dependency admission record (required fields)

When adding a dependency to source, record the following in the relevant `Cargo.toml` and report (MUST). A dependency without a record is treated as not admitted.

| Field | Required value |
|---|---|
| dependency name | crate name or local package name |
| dependency source | `local-kernel-path` / `crates-io` / `workspace` |
| class | `kernel-contract` / `reference-output` / `runtime` / `serialization` / `observability` / `command` / `benchmark` / `test-only` |
| owner package | distro package name |
| allowed use | one sentence |
| forbidden use | one sentence |
| version rule | exact lockfile version or local path |
| feature rule | explicit feature list |
| non-claim scope | readiness / completion claim not implied |

### Dependency admission class (closed set)

| Class | Examples | Admission owner | Claim boundary |
|---|---|---|---|
| runtime | async executor / timer | runtime surface | limited to executor behavior |
| transport | UDP / TCP / WebSocket / TLS crate | transport surface | limited to I/O distro |
| serialization | JSON / binary codec | mapping surface | limited to wire mapping |
| observability | logging / metrics / tracing | observability surface | limited to evidence emission |
| command | CLI / config loader | command surface | limited to operator command surface |
| benchmark / test | criterion / harness helper | benchmark surface | limited to benchmark / test execution |

## 7. Allowed Kernel dependency matrix (closed set)

The Kernel local path dependency is limited to the following range. Importing any Kernel package not listed in the table is forbidden (MUST NOT).

| distro package | Allowed Kernel packages |
|---|---|
| `arcrtc-distro-evidence` | none |
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

## 8. Allowed distro-local dependency matrix (closed set)

The distro-local path dependency is limited to the following range. Depending on any distro-local package not listed in the table is forbidden (MUST NOT).

| distro package | Allowed distro-local packages |
|---|---|
| `arcrtc-distro-evidence` | none |
| `arcrtc-reference-output` | `arcrtc-distro-evidence` |
| `arcrtc-reference-signaling` | `arcrtc-distro-evidence`, `arcrtc-reference-output` |
| `arcrtc-reference-turn` | `arcrtc-distro-evidence`, `arcrtc-reference-output` |
| `arcrtc-reference-sfu` | `arcrtc-distro-evidence`, `arcrtc-reference-output` |
| `arcrtc-reference-composition` | `arcrtc-distro-evidence`, `arcrtc-reference-output`, `arcrtc-reference-signaling`, `arcrtc-reference-turn`, `arcrtc-reference-sfu` |
| `arcrtc-reference-ops` | `arcrtc-distro-evidence`, `arcrtc-reference-composition` |
| `arcrtc-product-signaling` | `arcrtc-distro-evidence`, `arcrtc-reference-output`, `arcrtc-product-policy` |
| `arcrtc-product-turn` | `arcrtc-distro-evidence`, `arcrtc-reference-output`, `arcrtc-product-policy` |
| `arcrtc-product-sfu` | `arcrtc-distro-evidence`, `arcrtc-reference-output`, `arcrtc-product-policy` |
| `arcrtc-product-policy` | `arcrtc-distro-evidence` |
| `arcrtc-product-persistence-topology` | `arcrtc-distro-evidence` |
| `arcrtc-product-deployment` | `arcrtc-distro-evidence`, `arcrtc-product-rollback` |
| `arcrtc-product-monitoring` | `arcrtc-distro-evidence` |
| `arcrtc-product-rollback` | `arcrtc-distro-evidence` |

When an additional Kernel package is required, this matrix MUST be updated before any source change. The literal local path and package manifest shape follow the workspace manifest / dependency path rules in Chapter 02. Test package dependency admission is owned by the test workspace manifest and MUST NOT be mixed with the dependency admission of distro workspace packages. The reference-facing dependency that a product plane package may reference is limited to `arcrtc-reference-output` only. A product plane package MUST NOT depend on `arcrtc-reference-signaling`, `arcrtc-reference-turn`, `arcrtc-reference-sfu`, `arcrtc-reference-composition`, or `arcrtc-reference-ops`.

## 9. External dependency admission (closed set)

The initial external dependency class is limited to the following. Introducing any external dependency not listed in the table is forbidden (MUST NOT).

| Dependency | Version requirement | Features | Class | Allowed use | Forbidden use |
|---|---|---|---|---|---|
| `tokio` | `1` | `rt-multi-thread`, `macros`, `net`, `time`, `signal`, `sync` | runtime | async executor, socket polling, timer, shutdown signal | Kernel semantics / Kernel reason ownership |
| `serde` | `1` | `derive` | serialization | config / evidence struct serialization | protocol authority ownership |
| `serde_json` | `1` | none | serialization | JSON command / evidence artifact | readiness proof by itself |
| `tracing` | `0.1` | none | observability | structured distro log / span | adoption as evidence by itself |
| `tracing-subscriber` | `0.3` | `fmt`, `env-filter` | observability | local subscriber for command execution | production monitoring readiness |
| `clap` | `4` | `derive` | command | bounded CLI argument parsing | product policy authority |
| `criterion` | `0.5` | none | benchmark / test-only | benchmark harness | benchmark threshold satisfaction |

The exact crate version is fixed in `Cargo.lock` by the source scaffold task (MUST). A dependency whose version does not exist in `Cargo.lock` MUST NOT be adopted as formal evidence.

## 10. Prohibited dependency pattern (closed set)

The following are not permitted (MUST NOT).

- Adopt a dependency's raw error as a Kernel reason.
- Treat dependency default behavior as readiness evidence.
- Add a product persistence provider to a reference distro.
- Add a cloud provider SDK to a reference distro.
- Change a dependency version to match success after a report.
- Omit the `arcrtc-distro-evidence` dependency from a package that uses `DistroEvidenceReason`.
- Add a reference package dependency other than `arcrtc-reference-output` from a product plane package.
- Change Kernel semantics to fit a dependency API.
- Regard dependency admission as product admission or production readiness.

## 11. Invariants (collapse conditions)

The boundary defined by this chapter collapses the moment any of the following occurs. These are forbidden, and on occurrence the system MUST fail closed.

- Add a distro package to a Kernel workspace member.
- Make a TypeScript / Swift / Kotlin / browser harness the SFU / TURN / Signaling primary distro owner.
- Copy the Kernel source to the distro side.
- Adopt npm / yarn as a distro toolchain.
- Use a toolchain choice / dependency choice as a basis for production readiness or live readiness.
- Leave the toolchain / language fixed by this chapter undefined and decide it later elsewhere.
- A runtime executor owns the Kernel reason catalog or Kernel port definition.
- `distro-support/evidence` has a Kernel crate dependency.
- A product support package creates an unnecessary `src/kernel_contract.rs` and is mistaken for a Kernel contract owner.
- A product plane package has a reference package dependency other than `arcrtc-reference-output`.
- Mix a `tests/` package into the completion conditions of the distro workspace.
- Treat binary entrypoint success / runtime executor success as reference / product completion.
- Add a dependency without an admission record.
- Perform an import beyond the Kernel local path dependency matrix.
- Perform an import beyond the distro-local dependency matrix.
- A product plane package directly depends on a reference state / fixture / local auth / runtime / composition package.
- An external dependency owns Kernel semantic authority.
- Adopt a dependency result not fixed in `Cargo.lock` as formal evidence.
- Introduce an external dependency without a dependency admission class into source.
- Mix multiple runtimes without a formal versioned decision.

## 12. Non-claim

This chapter does not claim dependency list finalization, supply-chain readiness, reference distro completion, product distro completion, benchmark execution, benchmark threshold satisfaction, real-device command success, production readiness, live readiness, security readiness, or a change to the Kernel toolchain. This chapter fixes only the boundary of distro-side language / toolchain / runtime executor / dependency admission.
