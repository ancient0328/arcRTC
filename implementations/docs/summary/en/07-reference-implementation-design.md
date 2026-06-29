# Chapter 07 reference implementation design

Status: SSOT consolidated edition
Date: 2026-06-28 JST

## Purpose

This chapter fixes, at a granularity sufficient for reproducible implementation, the structure, responsibility boundaries, per-plane (Signaling / TURN / SFU) implementation design, composition design, runtime topology, transport boundary, configuration profile boundary, and composition runtime bridge of the reference implementation (the minimal implementation layer that consumes the Kernel frozen contract) within the arcRTC v0.2 implementations area. This chapter is fully self-contained and is understandable without consulting other documents or source code. Only other chapter numbers within this same specification are referenced.

The reference implementation MUST NOT claim production readiness. The reference implementation is the minimal implementation layer that uses the Kernel frozen contract (contract / SDK / command surface); the dependency direction is `implementations -> Kernel` (contract only).

---

## 1. Owned surface of the reference implementation

The reference implementation owns the following surfaces.

| Surface | Owns |
|---|---|
| `reference-implementation/output/` | reference output outcome types; product input subset is the 3-type allow-list (`ReferenceSignalingOutcome` / `ReferenceTurnOutcome` / `ReferenceSfuOutcome`) |
| `reference-implementation/signaling/` | Signaling reference wiring |
| `reference-implementation/turn/` | TURN reference wiring |
| `reference-implementation/sfu/` | SFU reference wiring |
| `reference-implementation/composition/` | Signaling / TURN / SFU reference composition |
| `reference-implementation/deployment-profiles/` | local / benchmark-oriented profile |
| `reference-implementation/ops/` | reference execution helper |

The reference implementation MUST NOT own the following.

- product policy
- production deployment
- public distribution
- production readiness
- live readiness
- Kernel semantic authority
- Kernel contract modification

### 1.1 Package shape (file contract)

The reference implementation packages follow the workspace members. Each package has the following required files.

| Package | Required files |
|---|---|
| `arcrtc-reference-output` | `src/lib.rs`, `src/signaling.rs`, `src/turn.rs`, `src/sfu.rs`, `src/composition.rs`, `src/error.rs` |
| `arcrtc-reference-signaling` | `src/lib.rs`, `src/kernel_contract.rs`, `src/state.rs`, `src/fixture_identity.rs`, `src/local_auth.rs`, `src/error.rs` |
| `arcrtc-reference-turn` | `src/lib.rs`, `src/kernel_contract.rs`, `src/state.rs`, `src/fixture_credential.rs`, `src/error.rs` |
| `arcrtc-reference-sfu` | `src/lib.rs`, `src/kernel_contract.rs`, `src/state.rs`, `src/fixture_route_auth.rs`, `src/error.rs` |
| `arcrtc-reference-composition` | `src/lib.rs`, `src/step_input.rs`, `src/composition_state.rs`, `src/runtime_bridge.rs`, `src/error.rs` |
| `arcrtc-reference-ops` | `src/lib.rs`, `src/runtime.rs`, `src/evidence.rs`, `src/reason.rs`, `src/error.rs` |
| `reference-implementation/deployment-profiles` | `local.toml`, `benchmark.toml` |

Profile files are not Rust workspace members.

### 1.2 Build direction (required direction)

The reference implementation MUST be built in the following order.

1. reference output package
2. Signaling reference implementation
3. TURN reference implementation
4. SFU reference implementation
5. reference composition
6. local / benchmark profile
7. execution helper

### 1.3 Evidence boundary

What reference implementation tests can show is limited to reference behavior and composition behavior. Reference implementation tests MUST NOT show production readiness, live readiness, native application readiness, or public distribution readiness.

### 1.4 Collapse conditions

If any of the following occurs, the authority of this chapter collapses (fail-closed remediation target).

- The reference implementation is treated as the same claim as the product implementation.
- A reference test pass is adopted as production readiness.
- The reference implementation owns product policy, monitoring, or rollback.
- The reference implementation modifies the Kernel contract.
- The reference implementation holds a database / queue / external storage.
- A fixture identity is treated as a production credential.
- A reference output outcome type is duplicated outside `arcrtc-reference-output`.

---

## 2. Signaling reference implementation design

The Signaling reference implementation is the first plane of the reference composition. It provides the connection surface for session / room / participant / control-plane commands but MUST NOT own Kernel core semantics. The owned surface is `reference-implementation/signaling/`.

### 2.1 Owned items

| Owned item | Meaning |
|---|---|
| reference signaling runtime wrapper | reference runtime surface that receives Signaling commands |
| reference room/session mapping | converts room/session identifiers of the reference execution profile into Kernel input |
| reference participant mapping | converts participant identifiers of the reference execution profile into Kernel input |
| reference signaling command wrapper | command surface for join / leave / offer / answer / candidate, etc. |
| reference signaling observation mapper | projects command result / reason / audit pointer into evidence shape |

### 2.2 Items it MUST NOT own

- Kernel Signaling semantics
- Kernel reason catalog
- authentication issuance
- product tenant policy
- production deployment
- live endpoint operation
- media routing
- TURN relay allocation

### 2.3 Dependency rule

```text
reference-implementation/signaling -> Kernel public contract
reference-implementation/signaling -> Kernel SDK projection
reference-implementation/signaling -> implementations evidence shape

reference-implementation/signaling -X-> Kernel core source modification
reference-implementation/signaling -X-> product policy
```

### 2.4 Input / output boundary (execution order)

The Signaling reference implementation MUST handle external input in the following order.

1. Receive reference command input.
2. Pass the input through reference-local validation.
3. Map to Kernel contract input.
4. Call the Kernel contract.
5. Map Kernel output / reason into reference evidence shape.

Reference-local validation is limited to checking command shape and required fields. accept / reject semantics are treated as the Kernel-side decision.

### 2.5 Evidence boundary

Signaling reference implementation tests MAY show:

- that the reference Signaling command is connected to the Kernel contract,
- that the mapping is deterministic,
- that reason / output is projected into evidence shape.

They MUST NOT show: production readiness, live readiness, product authentication correctness, SFU routing correctness, TURN relay correctness.

### 2.6 Collapse conditions

- The Signaling reference implementation reimplements Kernel Signaling semantics.
- Reference-local validation replaces the Kernel accept / reject decision.
- A Signaling reference result is treated as production readiness.
- The Signaling reference implementation owns product policy or live endpoint operation.

---

## 3. TURN reference implementation design

The TURN reference implementation builds the reference runtime surface for allocation / permission / relay. It handles the execution surface of network relay but MUST NOT own Kernel TURN semantics. The owned surface is `reference-implementation/turn/`.

### 3.1 Owned items

| Owned item | Meaning |
|---|---|
| reference allocation runtime wrapper | connects allocation command / lifecycle to the reference runtime |
| reference permission runtime wrapper | connects permission command / expiry to the reference runtime |
| reference relay runtime wrapper | connects relay packet flow to Kernel contract input/output |
| reference relay address mapper | converts address / candidate of the reference profile into Kernel input |
| reference TURN observation mapper | projects allocation / permission / relay result into evidence shape |

### 3.2 Items it MUST NOT own

- Kernel TURN allocation semantics
- Kernel permission semantics
- Kernel relay semantics
- product network policy
- public deployment topology
- production relay capacity claim
- live endpoint operation

### 3.3 Dependency rule

```text
reference-implementation/turn -> Kernel public contract
reference-implementation/turn -> Kernel core-owned port definition
reference-implementation/turn -> implementations evidence shape

reference-implementation/turn -X-> Kernel core source modification
reference-implementation/turn -X-> product network policy
reference-implementation/turn -X-> production readiness claim
```

### 3.4 Packet / relay boundary (execution order)

The TURN reference implementation MUST handle packet / relay data in the following order.

1. The reference runtime receives relay input.
2. Map runtime-specific input into Kernel-owned contract input.
3. The Kernel contract returns the allocation / permission / relay decision.
4. The reference runtime executes the relay action according to the decision.
5. Project result / reason / metric into evidence shape.

The reference runtime MAY take charge of relay execution. The semantics of allocation / permission / relay acceptability are treated as the Kernel-side decision.

### 3.5 Evidence boundary

TURN reference implementation tests MAY show:

- that the allocation / permission / relay runtime wrappers are connected to the Kernel contract,
- that relay input / output mapping is deterministic,
- that the Kernel reason is projected into evidence shape.

They MUST NOT show: production relay capacity, public NAT traversal success, live endpoint readiness, product network policy correctness, SFU media routing correctness.

### 3.6 Collapse conditions

- The TURN reference implementation reimplements Kernel allocation / permission / relay semantics.
- Reference relay success is treated as production relay capacity.
- product network policy is mixed into the reference TURN layer.
- live endpoint operation is claimed by the reference TURN test alone.

---

## 4. SFU reference implementation design

The SFU reference implementation builds the reference runtime surface for media routing / forwarding / quality observation. It handles the execution surface of packet forwarding but MUST NOT own Kernel SFU routing semantics. The owned surface is `reference-implementation/sfu/`.

### 4.1 Owned items

| Owned item | Meaning |
|---|---|
| reference media runtime wrapper | connects media packet / stream input to the reference runtime |
| reference subscriber mapper | converts subscriber / publisher relation into Kernel input |
| reference routing action executor | executes reference forwarding action according to the Kernel routing decision |
| reference quality observation mapper | projects loss / jitter / bitrate / backpressure observation into evidence shape |
| reference packet observation wrapper | maps packet metadata into Kernel contract input |

### 4.2 Items it MUST NOT own

- Kernel SFU routing semantics
- Kernel quality decision semantics
- Kernel congestion / pacing semantics
- codec negotiation semantics
- product media policy
- production media capacity claim
- live media readiness

### 4.3 Dependency rule

```text
reference-implementation/sfu -> Kernel public contract
reference-implementation/sfu -> Kernel core-owned port definition
reference-implementation/sfu -> implementations evidence shape

reference-implementation/sfu -X-> Kernel core source modification
reference-implementation/sfu -X-> product media policy
reference-implementation/sfu -X-> live media readiness claim
```

### 4.4 Packet boundary (execution order)

The SFU reference implementation MUST handle packet / media input in the following order.

1. The reference media runtime receives packet / stream observation.
2. Map runtime-specific metadata into Kernel-owned packet / routing input.
3. The Kernel contract returns the routing / quality / backpressure decision.
4. The reference runtime executes the forwarding action according to the decision.
5. Project result / reason / metric into evidence shape.

Packet payload ownership, buffer lifetime, and zero-copy / copy policy follow the Kernel contract and the driver/runtime boundary. The SFU reference implementation MUST NOT redefine packet semantics.

### 4.5 Evidence boundary

SFU reference implementation tests MAY show:

- that packet / stream input is connected to the Kernel routing contract,
- that the routing action executor follows the Kernel decision,
- that quality observation is projected into evidence shape.

They MUST NOT show: production media capacity, live media readiness, codec compatibility completeness, product media policy correctness, public endpoint availability.

### 4.6 Collapse conditions

- The SFU reference implementation reimplements Kernel routing / quality semantics.
- Reference forwarding success is treated as live media readiness.
- product media policy is mixed into the reference SFU layer.
- packet ownership / buffer lifetime is defined unrelated to the Kernel contract.

---

## 5. reference composition design

The value of the reference composition is to combine the 3 planes (Signaling / TURN / SFU) according to the Kernel frozen contract and produce a composition surface that becomes the basis for benchmark / real-device / product implementation. The composition is not the product implementation and MUST NOT claim production readiness / live readiness. The owned surface is `reference-implementation/composition/`.

### 5.1 Owned items

| Owned item | Meaning |
|---|---|
| plane wiring | connection of the Signaling / TURN / SFU reference implementations |
| identity/session bridge | joins reference profile identity/session across planes |
| command flow orchestration | defines the command order of the reference execution flow |
| observation correlation | binds cross-plane evidence by correlation id |
| benchmark profile binding | starts the reference composition from a benchmark profile |

### 5.2 Items it MUST NOT own

- Kernel cross-plane semantics
- product policy
- production deployment
- live endpoint operation
- public distribution
- readiness claim

### 5.3 Composition flow

The reference composition owns the following flow.

1. Load the reference profile.
2. Start the Signaling reference implementation.
3. Start the TURN reference implementation.
4. Start the SFU reference implementation.
5. Pass identity/session/correlation id across planes.
6. Execute the reference command flow.
7. Aggregate per-plane results into evidence shape.

Flow orchestration owns plane invocation order and data handoff. Plane semantics and cross-plane validity are treated as the Kernel contract decision.

### 5.4 Evidence boundary

reference composition tests MAY show:

- that the Signaling / TURN / SFU reference implementations are connected,
- that identity/session/correlation id is passed deterministically,
- that per-plane results are aggregated into evidence shape.

They MUST NOT show: product completion, production readiness, live readiness, public endpoint availability, benchmark threshold satisfaction.

### 5.5 Collapse conditions

- The composition reimplements Kernel cross-plane semantics.
- Composition success is treated as product completion.
- Composition success is treated as production readiness / live readiness.
- product deployment / monitoring / rollback is mixed into the reference composition.

---

## 6. composition runtime bridge

The composition runtime bridge is local orchestration that calls reference Signaling / TURN / SFU in an ordered manner. It MUST NOT own Kernel semantics, product policy, or readiness claims. This section fixes the input types, execution order, output types, and fail-closed branches.

### 6.1 Required types

| Type | File | Required fields |
|---|---|---|
| `ReferenceCompositionStepInput` | `reference-implementation/composition/src/step_input.rs` | `correlation_id: CorrelationId`, `step: ReferenceCompositionStep` |
| `ReferenceCompositionStep` | `reference-implementation/composition/src/step_input.rs` | enum variants listed below |
| `ReferenceCompositionPlaneOutcome` | `reference-implementation/composition/src/runtime_bridge.rs` | enum variants `Signaling(ReferenceSignalingOutcome)`, `Turn(ReferenceTurnOutcome)`, `Sfu(ReferenceSfuOutcome)`, `Composition` |
| `ReferenceCompositionStepOutcome` | `reference-implementation/composition/src/runtime_bridge.rs` | `correlation_id: CorrelationId`, `implementation_reason: ImplementationEvidenceReason`, `applied_plane: ImplementationPlane`, `plane_outcome: ReferenceCompositionPlaneOutcome`, `non_claim_scope: Vec<ImplementationNonClaimScope>` |

`ReferenceCompositionStep` variants are fixed to:

- `ApplySignaling(ReferenceSignalingCommandInput)`
- `ApplyTurn(ReferenceTurnCommandInput)`
- `ApplySfu(ReferenceSfuAction)`
- `BindSignalingToTurn { room_id: RoomId, allocation_id: AllocationId }`
- `BindSignalingToSfu { room_id: RoomId, session_id: SessionId, route_id: RouteId }`
- `Validate`

`ApplySfu` MUST NOT take `SfuReferenceSet` or `SfuDecisionKind` as input. The SFU decision kind is observed only from `ReferenceSfuOutcome` returned by `apply_reference_sfu`.

### 6.2 Required function

| Function | Required signature shape | Rule |
|---|---|---|
| `run_reference_composition_step` | `(&mut ReferenceCompositionState, ReferenceCompositionStepInput) -> Result<ReferenceCompositionStepOutcome, ReferenceCompositionError>` | executes one local reference composition step |

### 6.3 Execution rule

| Step | Required operation |
|---|---|
| `ApplySignaling` | call `apply_reference_signaling` against `state.signaling` |
| `ApplyTurn` | call `apply_reference_turn` against `state.turn` |
| `ApplySfu` | call `apply_reference_sfu(&mut state.sfu, &action)` and store the result as `ReferenceCompositionPlaneOutcome::Sfu` |
| `BindSignalingToTurn` | call `bind_signaling_to_turn` |
| `BindSignalingToSfu` | call `bind_signaling_to_sfu` |
| `Validate` | call `validate_reference_composition_state` |

Each step executes at most one plane mutation or one binding validation. No step may open a public endpoint, spawn provider runtime, write database state, or claim readiness (MUST NOT).

### 6.4 Outcome rule

Every success outcome MUST set:

- `implementation_reason = ImplementationOk`
- `non_claim_scope` containing `ProductionReadinessNotClaimed` and `LiveReadinessNotClaimed`
- `applied_plane` matching the executed plane, or `Composition` for binding / validation

Every failure maps through `ReferenceCompositionError::implementation_reason()`.

### 6.5 Fail-closed rule

| Failure | Required error |
|---|---|
| missing correlation id equivalent | `ReferenceCompositionError::EvidenceFieldsIncomplete` |
| orphan room / allocation / route | `ReferenceCompositionError::StateBoundaryViolation` |
| unsupported Kernel contract shape | `ReferenceCompositionError::KernelContractMismatch` |
| readiness claim attempted through bridge | `ReferenceCompositionError::EvidenceFieldsIncomplete` |

### 6.6 Collapse conditions

- The bridge mixes multiple plane mutations into a single step.
- The bridge owns product policy, provider runtime, or database state.
- The bridge treats reference success as product completion / readiness.
- The bridge outcome lacks the non-claim scope.
- The bridge reads a Kernel private field.

---

## 7. runtime topology

The initial runtime topology of the reference implementation is fixed to a single host / controlled process topology.

| Topology | Decision | Claim boundary |
|---|---|---|
| in-process composition | MAY be adopted in reference unit / composition test | does not claim live readiness |
| single host controlled process | adopted for reference benchmark / controlled integration | does not claim public live readiness |
| multi host deployment | handled from product implementation readiness onward | not adopted in reference scope |
| public live endpoint | handled in live readiness scope | not adopted in reference scope |

### 7.1 Runtime ownership

What the reference runtime topology owns:

- process start / stop
- local port allocation
- controlled clock / timeout profile
- local correlation id propagation
- local metrics / logs capture
- benchmark profile binding

What it MUST NOT own: public endpoint operation, production deployment topology, production SLO, live monitoring / alerting, rollback / drain operation.

### 7.2 Plane topology

The reference runtime connects the following planes on the same host.

```text
reference signaling process/surface
reference turn process/surface
reference sfu process/surface
reference composition controller
```

The initial topology does not claim public internet traversal. NAT traversal / public relay / public media readiness are handled in live readiness scope.

### 7.3 Non-goals

- Fixing a production process supervisor.
- Fixing container / orchestrator / cloud deployment.
- Claiming live endpoint availability.

### 7.4 Collapse conditions

- Reference runtime success is treated as live readiness.
- A single host controlled process is treated as production deployment.
- multi host / public endpoint is mixed into the reference scope.
- The topology owns Kernel semantics.

---

## 8. transport boundary (staged transport)

The transport of the reference implementation is treated as a staged transport.

| Stage | Transport | Owner | Claim boundary |
|---|---|---|---|
| RT0 | in-memory / direct command transport | reference tests | protocol wiring only |
| RT1 | loopback process transport | reference controlled integration | local runtime behavior |
| RT2 | loopback UDP/TCP/WebSocket where applicable | reference benchmark / integration | controlled network behavior |
| RT3 | public network / browser / native device transport | real-device / live readiness scope | not adopted in reference scope |

### 8.1 Plane transport rule

| Plane | Initial reference transport |
|---|---|
| Signaling | RT0 then RT1; WebSocket-like behavior is handled at RT2 |
| TURN | RT0 then RT2; UDP/TCP relay behavior is limited to controlled loopback |
| SFU | RT0 then RT2; packet forwarding is limited to controlled loopback |
| composition | switch RT0/RT1/RT2 by profile |

### 8.2 Transport ownership

What it MAY own: input/output framing, local socket / loopback binding, command dispatch, packet receive/send wrapper, transport-level timeout / retry, transport observation mapping.

What it MUST NOT own: Kernel accept / reject semantics, Kernel routing / allocation / signaling decision, production network policy, public traversal success, live endpoint readiness.

### 8.3 Upgrade rule

When raising a transport stage, the following MUST be stated explicitly.

1. stage id
2. target plane
3. command surface
4. environment class
5. expected outcome
6. non-claim scope

### 8.4 Collapse conditions

- RT0 / RT1 success is treated as public network success.
- RT2 loopback success is treated as live readiness.
- The transport wrapper owns Kernel protocol semantics.
- browser / native transport is mixed into the reference scope unconditionally.

---

## 9. configuration profile boundary

The reference implementation profiles are fixed to the following.

| Profile | Purpose | May claim | MUST NOT claim |
|---|---|---|---|
| `local-reference` | local controlled execution | reference wiring behavior | benchmark / readiness |
| `benchmark-reference` | controlled benchmark execution | benchmark observation | benchmark threshold satisfaction |
| `real-device-reference` | bounded device command input | command result | live readiness |
| `product-preflight` | product implementation preflight | product behavior input | production readiness |

### 9.1 Configuration ownership

What it MAY own: local port assignment, local address / bind configuration, controlled timeout, deterministic clock / randomness setting, fixture / workload identifier, log / metric output path, correlation id prefix.

What it MUST NOT own: Kernel semantics, product tenant policy, production secret value, public endpoint ownership, readiness conclusion.

### 9.2 Secret rule

The reference profile MUST NOT handle production secrets. When secret-like input is required, it is limited to: deterministic local test token, local fixture credential, non-production key material. Production secret handling is handled in product implementation / production readiness scope.

### 9.3 Clock / randomness rule

The reference profile MAY use a deterministic clock / randomness. Deterministic execution success does not show production runtime readiness.

### 9.4 Profile file specification (local.toml / benchmark.toml)

Both profile files use the same top-level schema.

| Section | Keys |
|---|---|
| `[profile]` | `name`, `environment_class`, `opens_public_endpoint`, `non_claim_scope` |
| `[runtime]` | `planes`, `shutdown_mode`, `timeout_ms` |
| `[fixtures]` | `rooms`, `participants_per_room`, `turn_allocations`, `sfu_routes`, `packet_payload_bytes` |
| `[evidence]` | `emit_json`, `output_dir`, `correlation_prefix` |

`opens_public_endpoint` MUST be `false` in all reference profiles. Profile files are not Rust workspace members.

`local.toml` is fixed to:

```toml
[profile]
name = "reference-local"
environment_class = "local_single_host"
opens_public_endpoint = false
non_claim_scope = ["production_readiness_not_claimed", "live_readiness_not_claimed", "public_distribution_readiness_not_claimed"]

[runtime]
planes = ["signaling", "turn", "sfu", "composition"]
shutdown_mode = "GracefulLocal"
timeout_ms = 5000

[fixtures]
rooms = 2
participants_per_room = 3
turn_allocations = 2
sfu_routes = 4
packet_payload_bytes = 1200

[evidence]
emit_json = true
output_dir = "target/reference-evidence/local"
correlation_prefix = "impl-reference-local"
```

`benchmark.toml` is fixed to:

```toml
[profile]
name = "reference-benchmark"
environment_class = "benchmark_host"
opens_public_endpoint = false
non_claim_scope = ["benchmark_threshold_not_claimed", "production_readiness_not_claimed", "live_readiness_not_claimed"]

[runtime]
planes = ["signaling", "turn", "sfu", "composition"]
shutdown_mode = "GracefulLocal"
timeout_ms = 30000

[fixtures]
rooms = 8
participants_per_room = 6
turn_allocations = 8
sfu_routes = 24
packet_payload_bytes = 1200

[evidence]
emit_json = true
output_dir = "target/reference-evidence/benchmark"
correlation_prefix = "impl-reference-benchmark"
```

### 9.5 Profile validation rule

Profile validation rejects (fail-closed):

- `opens_public_endpoint = true`
- empty `planes`
- `timeout_ms = 0`
- `packet_payload_bytes = 0`
- missing `non_claim_scope`
- a `non_claim_scope` item outside the allowed non-claim scope wire set
- an output path outside `target/reference-evidence/`

### 9.6 Collapse conditions

- A reference profile opens a public endpoint.
- A profile file contains a real credential, real endpoint, secret, token, or private key.
- benchmark profile success is treated as benchmark threshold satisfaction.
- local profile success is treated as production readiness or live readiness.
- A production secret is placed in the reference profile.
- deterministic clock / randomness success is treated as production readiness.
- A profile owns a readiness conclusion.
- A profile modifies Kernel behavior.

---

## 10. Invariants of this chapter (summary)

1. The dependency direction is `implementations -> Kernel` (contract only). The reference implementation MUST NOT modify the Kernel contract.
2. Reference success does not mean production readiness / live readiness / benchmark threshold satisfaction (fail-closed).
3. The reference types the product implementation can consume are limited to the 3-type allow-list (`ReferenceSignalingOutcome` / `ReferenceTurnOutcome` / `ReferenceSfuOutcome`). See Chapter 08 for details.
4. Every reference error maps to `ImplementationEvidenceReason` and has no `Unknown`.
5. Every outcome carries a correlation id and a non-claim scope. If these are missing, it is rejected fail-closed.
