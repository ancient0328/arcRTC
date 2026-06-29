# Architecture and Boundaries
Status: SSOT consolidated edition
Date: 2026-06-28 JST

## Purpose

This chapter fixes the layer model of the arcRTC v0.2 Kernel, the dependency directions (all permitted/forbidden directions), what each layer owns and does not own, the crate / package boundary and package roles, the semantic modular monolith principle, source shard modularity, and the no-copy / selective extraction policy. It presents, self-contained, the list and responsibilities of the core submodules, the boundary decision order, and the collapse conditions. From this chapter alone, the v0.2 architecture boundary can be understood at a granularity sufficient for re-implementation.

## Layer Model

arcRTC v0.2 adopts DDD / hexagonal architecture. The design center is to not confuse the Kernel's semantic authority, external drivers, in-Kernel entrypoints, the SDK, regulated support, and out-of-Kernel implementations.

```text
core <- drivers
core <- entrypoints
drivers <- entrypoints

sdk: independent Signaling-only boundary
regulated: optional domain support
```

The dependency-direction notation `A <- B` means "B depends on A (B may reference A)." `Kernel/` is the Kernel root. The in-Kernel `core` is the semantic nucleus, center, and highest authority, and is not an alias for the whole Kernel. Out-of-Kernel implementations are consumers of the Kernel contract and own SFU / TURN / Signaling reference or product implementations. The in-Kernel `entrypoints/*-server` are not product systems; they are limited to executable contract / composition evidence surfaces.

## Freeze Scope

After Kernel completion, the Kernel contract, semantic vocabulary, port ownership, and dependency direction are freeze targets. A post-freeze change to any of these requires a new versioned contract and a compatibility / deprecation rule (the protocol versioning / deprecation rules of Chapter 06). Freeze does not mean production / live / native application readiness; those are separate claims owned by out-of-Kernel implementations.

## Dependency Directions (all permitted/forbidden directions)

The permitted dependency directions are limited to the following.

```text
core <- drivers
core <- entrypoints
drivers <- entrypoints
```

The only conditionally permitted direction is the following.

```text
core <- regulated
```

`core <- regulated` MAY be permitted only when needed for communication-event types or opaque ID references, and only under explicit specification admission. Specifically, it is limited to references of specification-declared opaque communication events, audit pointers, and non-sensitive tags. `sdk` is an independent Signaling-only boundary and MUST NOT directly own regulated.

The forbidden dependency directions (Hard Prohibitions) are as follows. None are relaxed.

```text
core -> drivers
core -> entrypoints
core -> regulated
drivers -> entrypoints
drivers -> regulated
entrypoints -> regulated
sdk -> regulated
regulated -> sdk
regulated -> drivers
regulated -> entrypoints
```

In addition, the following concrete contaminations are forbidden.

- Placing WebSocket / HTTP / UDP / TCP socket concrete types in core.
- Placing str0m concrete types in core.
- Placing PostgreSQL / Redis / S3 / filesystem sink concrete implementations in core.
- Placing environment variable parsing in core.
- Placing protocol decisions in entrypoints.
- Treating entrypoints as SFU / TURN / Signaling product implementations.
- Adopting implementations product policy as a Kernel contract.

## What Each Layer Owns / Does Not Own

| Layer | Owns | Does not own |
|---|---|---|
| core | Kernel semantic nucleus, domain semantics, use case, port, contract, state, decision | I/O, runtime, framework, DB, cloud SDK, browser/native types, implementations product policy |
| drivers | port implementation, external type conversion, I/O | domain rule, accept/reject rule |
| entrypoints | executable contract evidence, CLI, demo, composition root | domain rule, protocol semantics, SFU / TURN / Signaling product implementation |
| sdk | Signaling-only public client contract | media, auth issuance, regulated workflow |
| regulated | optional domain support, enrichment, pointer mapping | the authority of the generic communication protocol |

In drivers, external types MUST be converted to core-owned types at the drivers boundary. entrypoints are limited to in-Kernel executable contract / composition evidence surfaces and dependency wiring, and MUST NOT own domain rules.

## Boundary Decision Order

Which layer a subject belongs to is decided in the following order.

1. Is the subject protocol / domain semantics? -> If yes, a core candidate.
2. Is the subject an external technology implementation? -> If yes, a drivers candidate.
3. Is the subject an executable composition root? -> If yes, an entrypoints candidate.
4. Is the subject a public SDK client boundary? -> If yes, an sdk candidate.
5. Is the subject optional regulated support? -> If yes, a regulated candidate.

If it matches multiple, separate them; do not mix them in the same file / module.

## List and Responsibilities of core Submodules

`core/` is the Kernel's semantic nucleus, center, and highest authority, with no dependency on external I/O. The whole core is a single semantic authority, and the internal module/package boundaries are semantic boundaries, not deployment boundaries. The list and responsibilities of the core submodules are as follows.

| Path | Responsibility |
|---|---|
| `core/transport/` | Sans-IO transport contract, RTP / RTCP / ICE-adjacent semantics |
| `core/signaling/` | Signaling contract, room state, command semantics, accept / reject boundary |
| `core/sfu/` | routing, quality decision, backpressure semantics |
| `core/turn/` | allocation, permission, relay semantics |
| `core/security/` | token verification boundary, identity-neutral primitives |
| `core/audit/` | audit event model, hash-chain contract, audit ports |
| `core/quality/` | metrics model, quality decisions |
| `core/ports/` | ports such as network, persistence, clock, metrics, audit sink |
| `core/domain/` | DDD domain model, aggregate, domain service, application use case |
| `core/identity/` | opaque references, correlation IDs, external identity separation |
| `core/command/` | command, decision, event, response, idempotency, replay, correlation |
| `core/reason/` | closed reason categories and codes |
| `core/protocol/` | deterministic encoding, protocol versioning, compatibility/deprecation |
| `core/configuration/` | core-owned configuration boundary types |
| `core/features/` | out-of-scope feature admission and rejection |
| `core/cross-plane/` | cross-plane identity / session binding semantics |
| `core/operation/` | operation semantics, operational boundary model |
| `core/runtime/` | runtime abstraction, task / worker lifecycle contract |
| `core/time/` | time, clock, timestamp trust semantics |
| `core/state/` | state transition, state ownership semantics |
| `core/recovery/` | recovery, retry, compensation semantics |

These MAY belong to the same workspace / release unit. However, the semantic owner and dependency direction are fixed at the module/package boundary.

## Submodules of drivers / entrypoints

`drivers/` are implementations of core ports.

| Path | Responsibility |
|---|---|
| `drivers/webrtc-str0m/` | WebRTC transport port implementation using str0m |
| `drivers/network/` | tokio UDP / TCP, HTTP, WebSocket, socket I/O |
| `drivers/persistence/` | PostgreSQL, S3, filesystem, in-memory |
| `drivers/observability/` | tracing, metrics exporter, log sinks |
| `drivers/security/` | key source, verifier driver, secret rotation |
| `drivers/browser/` | browser-facing helper boundary |
| `drivers/native/` | native platform binding boundary |

`entrypoints/` are in-Kernel startup units, contract probes, dependency wiring, and composition evidence surfaces. They MUST NOT own the SFU / TURN / Signaling product system.

| Path | Responsibility |
|---|---|
| `entrypoints/signaling-server/` | Signaling executable contract / composition evidence surface |
| `entrypoints/sfu-server/` | SFU executable contract / composition evidence surface |
| `entrypoints/turn-server/` | TURN executable contract / composition evidence surface |
| `entrypoints/cli/` | operator / developer CLI |
| `entrypoints/demo/` | demo composition |
| `entrypoints/configuration/` | configuration profile, policy bundle, feature/capability wiring |
| `entrypoints/endpoints/` | public endpoint lifecycle, edge/proxy trust wiring |
| `entrypoints/topology/` | deployment topology, service discovery, endpoint resolution wiring |
| `entrypoints/internal-control/` | internal service identity and control-plane wiring |
| `entrypoints/admin/` | health/readiness/liveness/admin/operator wiring |

## crate / package Boundary and Package Roles

The v0.2 package layer follows the architecture layer. The package boundary is not permission to create a Cargo.toml; it fixes the package dependency rule that MUST NOT be broken during implementation scaffolding.

| Layer | Package role | Dependency rule |
|---|---|---|
| core | domain, use case, port, contract | does not depend on drivers/entrypoints |
| drivers | port implementation | may depend on core |
| entrypoints | Kernel executable contract, CLI, demo, wiring | may depend on core and selected drivers |
| sdk | Signaling-only public client package | does not depend on core internals / drivers / regulated |
| regulated | optional domain support | only specification-permitted opaque core references |

### Naming Rule

Rust package naming MUST reflect layer and role. A technology-specific name MAY appear only in driver package names.

Allowed naming pattern examples:

| Layer | Pattern |
|---|---|
| core | `arcrtc-core` |
| driver | `arcrtc-driver-network`, `arcrtc-driver-webrtc-str0m`, `arcrtc-driver-persistence-*` |
| entrypoint | `arcrtc-signaling-server`, `arcrtc-sfu-server`, `arcrtc-turn-server`, `arcrtc-cli` |
| sdk | `arcrtc-sdk-*` |
| regulated | `arcrtc-regulated-*` |

Forbidden naming pattern examples:

- `arcrtc-core-str0m`
- `arcrtc-core-postgres`
- `arcrtc-entrypoint-domain`
- `arcrtc-sdk-regulated`

### Feature Flag Rule

Feature flags MUST NOT invert the architecture dependency.

Forbidden:

- A core feature pulls a driver dependency into core.
- A core default feature enables DB / S3 / HTTP / str0m / tokio concrete implementation.
- An entrypoint feature changes domain semantics.
- A driver feature defines an alternate reason catalog.
- An sdk feature imports regulated support.

Allowed:

- A driver package feature selects an external implementation detail.
- An entrypoint package feature selects a composition profile.
- A core test feature exposes test helpers without an external implementation dependency.

### Workspace Rule

Workspace membership does not imply architecture permission. Dependency admission, license, vulnerability, lockfile, and toolchain gates MUST satisfy supply-chain policy. Even inside one workspace, the dependency direction MUST remain:

```text
core <- drivers
core <- entrypoints
drivers <- entrypoints
```

`sdk` remains an independent Signaling-only boundary. `regulated -> core` is allowed only under explicit specification constraints.

### Test Utility Rule

Test utility packages MUST NOT become a hidden production dependency. A test helper MAY depend on core test APIs and fake drivers, but the core production package MUST NOT depend on the fake driver package. v0.1 tests and benches are evidence-only until requalified by v0.2 testing / benchmark authority.

### Package Boundary Prohibitions

- A core package depends on a driver package.
- A driver package depends on an entrypoint package.
- An entrypoint package defines port traits.
- An SDK package depends on a regulated package.
- A regulated package depends on drivers/entrypoints/sdk.
- Feature flags bypass specification boundaries.
- The workspace root re-exports concrete driver types as core API.
- A source shard is used to move the semantic owner or hide the dependency direction.
- A source-shape test inspects only a root file while source shards carry the actual implementation.
- Package manager install success is treated as dependency policy acceptance.
- Dependency license/vulnerability/toolchain status is hidden for a release/readiness claim.
- Package build output is treated as a distributable release artifact without provenance.
- Source file line count is treated as a stronger close condition than semantic owner, dependency direction, or forbidden import absence.
- A core package introduces an undeclared core peer dependency.
- Production source uses local `#[allow(clippy::too_many_arguments)]` instead of the workspace policy and semantic record boundary.

## The semantic modular monolith Principle

Core is a semantic modular monolith. The principal problem of v0.2 is not a lack of network hops but the semantic crosstalk between planes. If the relations of Signaling accepted, SFU admitted, TURN allocated, ICE connectivity, secure media protected, audit recorded, and reason classified become ambiguous, the system boundary will not hold even if core is split small. Therefore, making core small, or keeping source files under a certain line count, is not the higher-order goal. The higher-order goal is to strictly divide the interior of a core with a single semantic authority into module/packages by semantic unit.

The principles are as follows.

1. The whole core is a single semantic authority.
2. The in-core module/package is a semantic boundary, not a deployment boundary.
3. The split axis of the module/package is not technology such as WebSocket, UDP, str0m, DB, metrics, or HTTP, but Signaling state, SFU routing, TURN lifecycle, identity, reason, audit, quality, ports, protocol, configuration, features, and cross-plane binding.
4. A core module/package MAY depend only on the foundation core module and explicitly declared peer core contracts.
5. A core module/package MUST NOT import drivers, entrypoints, framework, async runtime, socket, DB, browser/native SDK, cloud SDK, or concrete transport types.
6. Cross-plane access goes through core-owned identity, command, reason, event, and binding types.
7. Source file line count is an auditability guideline, not a semantic boundary.
8. 600 lines is treated as a source comprehension review signal, but is not a hard gate above the core semantic modular monolith boundary.

Core package peer dependencies are a closed set. Each `core/*/Cargo.toml` MAY depend only on the foundation/peer core packages declared by the semantic modular monolith boundary tests. Adding a new `arcrtc-core-*` dependency requires updating the peer dependency matrix and the relevant specification relation.

### Semantic Boundary per core Submodule

| Surface | Allowed | Forbidden |
|---|---|---|
| core.signaling | dependency on core identity / command / reason / protocol / ports | WebSocket / HTTP / tokio runtime / entrypoint server ownership |
| core.sfu | dependency on core identity / reason / quality / ports / cross-plane | str0m concrete event / driver packet buffer ownership |
| core.turn | dependency on core identity / reason / security / ports / cross-plane | UDP/TCP socket behavior / entrypoint server lifecycle |
| core.audit | dependency on core identity / reason / protocol / audit model | DB row shape / concrete storage retry policy |
| core.reason | ownership of the closed reason vocabulary | reason extension by drivers/entrypoints |
| core.cross-plane | ownership of admitted binding class and target-plane relation | implicit binding from CorrelationId, network address, or external user ID alone |

## source shard modularity

A source shard is a physical split permitted for auditability within the bounds of not moving the semantic owner. It is a subordinate rule of the semantic modular monolith boundary. As implementation progresses, multiple types, guards, failures, and projections accumulate in a single `lib.rs` or a single SDK surface file, making the responsibility boundary hard to read during audit. A source file exceeding 600 lines does not directly mean a boundary violation, but because it weakens the source comprehension of audit, review, and follow-on work and may induce overlooked dead code, legacy paths, duplicate implementations, and fail-open search, it is treated as a review signal.

The rules are as follows.

1. Keep one source file at a readable granularity in principle, and treat 600 lines as a review signal.
2. When the public API of a Rust crate needs to be kept at the crate root, allow source shards via `include!`.
3. A Rust source shard is included in the same module scope and MUST NOT move owner, public API, or semantic authority. This is a physical source split, not a semantic owner transfer.
4. The Swift SDK divides files by responsibility using normal source set splitting of the Swift Package target. The split MUST preserve the Signaling-only SDK surface and MUST NOT move media, auth issuance, regulated workflow, or driver internals into the SDK.
5. Source-shape tests inspect the entire source set of the target crate/package, not the strings of a single `lib.rs`.
6. A source shard is not an evidence substitution for tests or docs. It MUST NOT be adopted as a substitute for build/test/lint.
7. Tests MUST fail on collapse of semantic owner / dependency direction / forbidden import / source-set visibility. They MUST NOT fail solely because a source file exceeds 600 lines.

A shard file MUST NOT create a new core/drivers/entrypoints/sdk/regulated boundary, dependency permission, or evidence class. High-arity semantic evidence / guard record constructors MUST NOT be hidden by a local `#[allow(clippy::too_many_arguments)]`. The workspace MAY define a central Clippy threshold, but production source MUST NOT introduce local suppressions.

### source shard Boundary

| Surface | Allowed | Forbidden |
|---|---|---|
| Rust crate root | public API-preserving `include!` source shard | moving the semantic owner |
| Rust source-shape test | crate/package source set scan | inspection treating `lib.rs` alone as the authority |
| SDK iOS | target source set split | changing the Signaling-only surface |
| source-shape test scope | semantic modular monolith collapse detection and source-size review signal | making line count a hard gate above the semantic boundary |

## no-copy / selective extraction Policy

v0.2 forbids the mechanical copy of v0.1.2 and adopts selective extraction. Selective extraction is taking from the v0.1.2 source / docs / evidence only the semantics, contracts, types, state, and ports explicitly adopted by the v0.2 specification.

The following MUST NOT be copied directly into v0.2.

- `server/signaling`
- `server/sfu`
- `server/turn`
- the entire `core` crate
- the entire `sdk` implementation
- the entire `regulated` implementation
- completion words, close words, and evidence claims of the v0.1 development records
- ignored / generated / local artifacts

The conditions for adopting a v0.1.2-derived element into v0.2 are as follows.

1. The target element exists in an inventory report or manifest.
2. It is made explicit which of v0.2's `core / drivers / entrypoints / sdk / regulated` the adopted responsibility belongs to.
3. The v0.2 specification records the adoption reason, forbidden boundary, and collapse conditions.
4. External dependencies, runtime, I/O, DB, cloud SDK, and browser/native types do not remain in core.
5. Past measurements or close-like words are not reused as the grounds for v0.2 viability.

v0.1.2 is a reference inventory, not the v0.2 current canonical. The fact that the v0.1.2 implementation functioned does not prove v0.2 operational viability. v0.2 does not copy the v0.1 file layout and adopts only content re-adopted by the v0.2 specification. Following the protocol compatibility / deprecation rule, v0.1 shapes MUST NOT be treated as automatically compatible.

## Collapse Conditions

The architecture boundary judgment of this chapter collapses if any of the following hold.

- core depends on an external concrete implementation.
- drivers own a domain rule.
- entrypoints own protocol semantics beyond the composition root.
- entrypoints are treated as an implementations product implementation.
- the SDK exceeds the Signaling-only boundary.
- regulated becomes a required dependency of the generic communication core.
- regulated depends on sdk / drivers / entrypoints.
- the Cargo package dependency graph violates the architecture dependency direction.
- a feature flag pulls concrete I/O into core.
- the SDK imports internal core/drivers APIs instead of the Signaling public contract.
- the core module/package boundary is cut by concrete technology instead of semantic authority.
- source file size is used as a hard gate without semantic owner / dependency collapse.
- a source shard changes module scope, public API, or owner boundary.
- `core.signaling` imports concrete WebSocket / HTTP / entrypoint server types.
- `core.sfu` imports str0m packet/event objects or driver-owned buffer/cache types.
- `core.turn` owns UDP/TCP socket behavior.
- `core.audit` depends on a concrete DB row shape or persistence driver retry state.
- `core.reason` is extended by drivers or entrypoints.
- core modules share mutable global state outside declared aggregate/state owners.
- cross-plane binding is inferred from CorrelationId, token subject, network address, ICE username fragment, endpoint name, or external user ID alone.
- line count is treated as stronger evidence than semantic owner and dependency boundary.
- the v0.1.2 file layout is treated as the v0.2 current canonical.
- the close-like words of the v0.1 development records are used as grounds for v0.2 viability.
