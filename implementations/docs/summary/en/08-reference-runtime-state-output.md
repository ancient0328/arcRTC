# Chapter 08 reference runtime state output

Status: SSOT consolidated edition
Date: 2026-06-28 JST

## Purpose

This chapter fixes, at a granularity sufficient for reproducible implementation, the phase / fixture payload (all phases and payloads), state mutation rules (all state transitions), reference-local API shape and state types, output boundary (the 3-type allow-list the product can consume), runtime lifecycle (all states), and command evidence runner (execution and evidence generation procedure) of the reference implementation within the arcRTC v0.2 implementations area. This chapter is fully self-contained and is understandable without consulting other documents or source code. Only other chapter numbers within this same specification are referenced.

Reference state is a local deterministic behavior projection; it is not Kernel state authority, database state, production state, or readiness evidence. A reference fixture is deterministic local input; it is not a production credential, real SDP, real ICE candidate, real packet payload, or readiness evidence.

---

## 1. Signaling local phase and fixture payload

### 1.1 Signaling local phase enums

`reference-implementation/signaling/src/state.rs` holds the following local phase enums.

| Enum | Variants | Rule |
|---|---|---|
| `ReferenceRoomPhase` | `Open`, `Closed` | an absent room is represented by the absence of a map key |
| `ReferenceParticipantPhase` | `Joined`, `Left` | an absent participant is represented by the absence of a map key |

`JoinRoom` success creates `ReferenceRoomPhase::Open` and `ReferenceParticipantPhase::Joined`. `LeaveRoom` success does not delete the participant; it sets `ReferenceParticipantPhase::Left`. A join to a `Closed` room is rejected with `STATE_BOUNDARY_VIOLATION`.

### 1.2 Signaling fixture types

`reference-implementation/signaling/src/fixture_identity.rs` holds the following.

| Type | Required fields | Value rule |
|---|---|---|
| `FixtureIdentity` | `identity_id: String`, `participant_id: ParticipantId`, `room_id: RoomId` | `identity_id` is `fixture-identity-{n}` |
| `FixtureSessionDescription` | `session_id: SessionId`, `fixture_sdp_id: String`, `direction: FixtureSessionDescriptionDirection` | `fixture_sdp_id` is `fixture-sdp-{n}` |
| `FixtureSessionDescriptionDirection` | enum `Offer`, `Answer` | offer / answer fixture classification only |
| `FixtureIceCandidate` | `session_id: SessionId`, `fixture_candidate_id: String` | `fixture_candidate_id` is `fixture-ice-candidate-{n}` |

Real SDP text and real ICE candidate body MUST NOT be stored in fixture types.

### 1.3 Signaling payload variants

`ReferenceSignalingPayload` variants are fixed to:

- `JoinRoom`
- `LeaveRoom`
- `SendOffer { session: FixtureSessionDescription }`
- `SendAnswer { session: FixtureSessionDescription }`
- `SendIceCandidate { candidate: FixtureIceCandidate }`
- `RequestTurnCredential`
- `AcknowledgeForward`

`SendOffer` requires `FixtureSessionDescriptionDirection::Offer`. `SendAnswer` requires `FixtureSessionDescriptionDirection::Answer`. A direction mismatch is `FIXTURE_IDENTITY_INVALID`.

### 1.4 TURN fixture types

`reference-implementation/turn/src/fixture_credential.rs` is fixed to:

| Type / Function | Required shape | Rule |
|---|---|---|
| `FixtureTurnCredential` | `credential_ref: CredentialRef`, `allocation_id: AllocationId`, `requested_lifetime: TurnRequestedLifetimeSeconds` | deterministic fixture credential only |
| `validate_fixture_turn_credential` | `(&FixtureTurnCredential) -> Result<(), ReferenceTurnError>` | rejects missing / invalid fixture identity |

`FixtureTurnCredential` MUST NOT contain a raw secret, password, private key, or provider token.

### 1.5 SFU fixture types

`reference-implementation/sfu/src/fixture_route_auth.rs` is fixed to:

| Type / Function | Required shape | Rule |
|---|---|---|
| `FixtureRouteAdmission` | `route_id: RouteId`, `stream_id: StreamId`, `endpoint_id: EndpointId`, `allowed: bool` | deterministic route admission only |
| `authorize_reference_route` | `(&FixtureRouteAdmission) -> Result<(), ReferenceSfuError>` | `allowed == false` rejects with `FIXTURE_IDENTITY_INVALID` |

### 1.6 SFU local phase enums

`reference-implementation/sfu/src/state.rs` owns the following local phase enums.

| Enum | Variants | Rule |
|---|---|---|
| `SfuSessionState` | `Open`, `Draining`, `Closed` | an absent session is represented by the absence of a map key |
| `SfuEndpointState` | `Observed`, `AdmissionPending`, `Admitted`, `Degraded`, `Draining`, `Removed`, `Rejected` | an absent endpoint is represented by the absence of a map key |
| `SfuRouteState` | `Candidate`, `Selected`, `Delayed`, `SuppressedByBackpressure`, `SuppressedByQuality`, `Degraded`, `Dropped`, `Closed` | an absent route is represented by the absence of a map key |

`ReferenceSfuAction` is the only local input enum allowed to mutate SFU reference state. These local SFU state enums mirror the frozen Kernel state variant names and MUST NOT introduce implementation-only variants.

### 1.7 Cross-plane fixture rule

Composition binding uses concrete Kernel identity values already present in reference state. Composition MUST NOT synthesize production identity, production credential, or provider state.

| Binding | Required source |
|---|---|
| signaling to TURN | a joined reference signaling participant and an active reference TURN allocation |
| signaling to SFU | a joined reference signaling participant and an existing reference SFU route |

### 1.8 Collapse conditions

- A real SDP body, real ICE candidate body, raw TURN secret, or real provider token is stored in a fixture.
- absent state and rejected state are represented by the same enum variant.
- `LeaveRoom` deletes a participant record and loses the evidence trace.
- A fixture validation failure is treated as `UNKNOWN` or success.
- A fixture identity is treated as the product identity source of truth.

---

## 2. State mutation rules (all state transitions)

Reference state is in-memory state mutation. This section fixes map keys, duplicate handling, and reject branches.

### 2.1 Shared state rule (map keys)

All reference state maps use canonical key strings derived from Kernel identity values.

| ID type | Key rule |
|---|---|
| `RoomId` | `room_id.as_str().to_owned()` |
| `ParticipantId` | `participant_id.as_str().to_owned()` |
| `AllocationId` | `allocation_id.as_str().to_owned()` |
| `PermissionId` | `permission_id.as_str().to_owned()` |
| `ChannelBindId` | `channel_bind_id.as_str().to_owned()` |
| `SessionId` | `session_id.as_str().to_owned()` |
| `EndpointId` | `endpoint_id.as_str().to_owned()` |
| `RouteId` | `route_id.as_str().to_owned()` |

No state map key may use a provider id, random id, wall-clock time, memory address, raw packet bytes, or fixture display text as authority (MUST NOT).

### 2.2 Signaling mutation table

| Command | Pre-state | Mutation | Outcome | Reject branch |
|---|---|---|---|---|
| `JoinRoom` | room absent | insert `ReferenceRoomState { phase: Open }`; insert participant key into room set; insert participant state `Joined` | `SignalingEventKind::Joined` | none |
| `JoinRoom` | room `Open`, participant absent | insert participant key into room set; insert participant state `Joined` | `SignalingEventKind::Joined` | none |
| `JoinRoom` | participant `Left` in same room | keep room; set participant `Joined`; ensure room set contains participant key | `SignalingEventKind::Joined` | none |
| `JoinRoom` | participant `Joined` in same room | no mutation | `SignalingEventKind::Joined` | none |
| `JoinRoom` | room `Closed` or participant joined in different room | no mutation | none | `ReferenceSignalingError::StateBoundaryViolation` |
| `LeaveRoom` | participant `Joined` in target room | set participant `Left`; keep participant key in room set for evidence trace | `SignalingEventKind::Left` | none |
| `LeaveRoom` | participant absent / left / wrong room | no mutation | none | `ReferenceSignalingError::StateBoundaryViolation` |
| `SendOffer` | participant `Joined` in room and fixture direction `Offer` | no state mutation | `SignalingEventKind::OfferReceived` | fixture mismatch: `InvalidFixtureIdentity`; state mismatch: `StateBoundaryViolation` |
| `SendAnswer` | participant `Joined` in room and fixture direction `Answer` | no state mutation | `SignalingEventKind::AnswerReceived` | fixture mismatch: `InvalidFixtureIdentity`; state mismatch: `StateBoundaryViolation` |
| `SendIceCandidate` | participant `Joined` in room | no state mutation | `SignalingEventKind::IceCandidateReceived` | fixture mismatch: `InvalidFixtureIdentity`; state mismatch: `StateBoundaryViolation` |
| `RequestTurnCredential` | participant `Joined` in room | no state mutation | `SignalingEventKind::TurnCredentialAvailable` | state mismatch: `StateBoundaryViolation` |
| `AcknowledgeForward` | participant `Joined` in room | no state mutation | `SignalingEventKind::Joined` | state mismatch: `StateBoundaryViolation` |

### 2.3 TURN mutation table

| Command | Pre-state | Mutation | Outcome | Reject branch |
|---|---|---|---|---|
| `Allocate` | allocation absent and credential fixture valid | insert allocation `AllocationState::Active`; store credential ref | `TurnDecisionKind::Allocation` | invalid credential: `InvalidFixtureCredential` |
| `Allocate` | allocation active | no mutation | `TurnDecisionKind::Allocation` | none |
| `Refresh` | allocation active | keep allocation active | `TurnDecisionKind::Refresh` | missing allocation: `StateBoundaryViolation` |
| `CreatePermission` | allocation active, permission absent, peer address present | insert permission `PermissionState::Active` | `TurnDecisionKind::Permission` | missing allocation / peer: `StateBoundaryViolation` |
| `CreatePermission` | permission active for same allocation / peer | no mutation | `TurnDecisionKind::Permission` | none |
| `ChannelBind` | permission active and channel bind absent | insert channel bind `ChannelBindState::Active` | `TurnDecisionKind::ChannelBind` | missing permission / bind id: `StateBoundaryViolation` |
| `ChannelBind` | channel bind active | no mutation | `TurnDecisionKind::ChannelBind` | none |
| `RelayData` | allocation active, permission active, packet id present | no state mutation | `TurnDecisionKind::Relay` | missing allocation / permission / packet: `StateBoundaryViolation` |

TURN validation rejects: a permission whose allocation key is absent; a channel bind whose permission key is absent; a permission with peer address absent in the command path.

### 2.4 SFU mutation table

SFU mutation is driven by `ReferenceSfuAction`, not by reading private fields from Kernel `SfuReferenceSet`. Outcome `SfuDecisionKind` values are the frozen Kernel values (see the mapping table in section 3). `ReferenceSfuAction` fields are typed local input; a typed action field is not treated as missing inside the mutation table. Reject branches below refer to state absence, state mismatch, or disallowed phase, not to absent Rust fields.

| Reference action | Pre-state | Mutation | Outcome | Reject branch |
|---|---|---|---|---|
| `AdmitParticipant` | session absent and endpoint absent | insert session `SfuSessionState::Open`; insert endpoint `SfuEndpointState::Admitted` bound to session | `SfuDecisionKind::ParticipantAdmission` | none |
| `AdmitParticipant` | session open and endpoint absent / observed / admission pending / degraded / draining | set endpoint `SfuEndpointState::Admitted` bound to session | `SfuDecisionKind::ParticipantAdmission` | none |
| `AdmitParticipant` | session open and endpoint admitted for same session | no mutation | `SfuDecisionKind::ParticipantAdmission` | none |
| `AdmitParticipant` | session closed / draining or endpoint rejected / removed / bound to different session | no mutation | none | `ReferenceSfuError::StateBoundaryViolation` |
| `RejectParticipant` | session absent and endpoint absent | insert session `SfuSessionState::Open`; insert endpoint `SfuEndpointState::Rejected` bound to session | `SfuDecisionKind::ParticipantAdmission` | none |
| `RejectParticipant` | session open and endpoint absent / observed / admission pending | set endpoint `SfuEndpointState::Rejected` bound to session | `SfuDecisionKind::ParticipantAdmission` | none |
| `RejectParticipant` | session open and endpoint rejected for same session | no mutation | `SfuDecisionKind::ParticipantAdmission` | none |
| `RejectParticipant` | session closed / draining, endpoint admitted / degraded / draining / removed, or endpoint bound to different session | no mutation | none | `ReferenceSfuError::StateBoundaryViolation` |
| `PublishStream` | session open and endpoint admitted for same session | no state mutation; publication is represented by outcome only | `SfuDecisionKind::Publication` | none |
| `PublishStream` | session absent / closed / draining, endpoint absent / not admitted, or endpoint bound to different session | no mutation | none | `ReferenceSfuError::StateBoundaryViolation` |
| `SubscribeRoute` | session open, endpoint admitted for same session, and route absent | insert route `SfuRouteState::Candidate` with route id, session id, endpoint id, and stream id carried by action | `SfuDecisionKind::Subscription` | none |
| `SubscribeRoute` | session open, endpoint admitted for same session, and route candidate for same session / endpoint / stream | no mutation | `SfuDecisionKind::Subscription` | none |
| `SubscribeRoute` | session absent / closed / draining, endpoint absent / not admitted, endpoint bound to different session, route closed / dropped, or route session / endpoint / stream mismatch | no mutation | none | `ReferenceSfuError::StateBoundaryViolation` |
| `SelectRoute` | route candidate or selected for same session / endpoint / stream | set route `SfuRouteState::Selected` | `SfuDecisionKind::RouteSelection` | none |
| `SelectRoute` | route absent / dropped / closed, or route session / endpoint / stream mismatch | no mutation | none | `ReferenceSfuError::StateBoundaryViolation` |
| `SuppressForwarding { source: Backpressure }` | route selected | set route `SfuRouteState::SuppressedByBackpressure` | `SfuDecisionKind::Forwarding` | none |
| `SuppressForwarding { source: Quality }` | route selected | set route `SfuRouteState::SuppressedByQuality` | `SfuDecisionKind::Forwarding` | none |
| `SuppressForwarding` | route absent / not selected / dropped / closed | no mutation | none | `ReferenceSfuError::StateBoundaryViolation` |
| `DropForwarding` | route candidate / selected / delayed / suppressed by backpressure / suppressed by quality / degraded | set route `SfuRouteState::Dropped` | `SfuDecisionKind::BackpressureAction` | none |
| `DropForwarding` | route absent / closed | no mutation | none | `ReferenceSfuError::StateBoundaryViolation` |
| `CloseSession` | session open / draining | set session `SfuSessionState::Closed`; set endpoints in that session to `SfuEndpointState::Removed`; set routes in that session to `SfuRouteState::Closed` | `SfuDecisionKind::DegradationRecovery` | none |
| `CloseSession` | session absent / closed | no mutation | none | `ReferenceSfuError::StateBoundaryViolation` |

SFU state MUST NOT store raw packet bytes or borrowed packet slices. SFU state does not contain a stream map. `stream_id` is retained only in `ReferenceRouteState` for route validation and MUST NOT become a separate stream authority store.

### 2.5 Composition mutation table

| Function | Pre-state | Mutation | Reject branch |
|---|---|---|---|
| `bind_signaling_to_turn` | room exists/open and allocation active | insert binding key `correlation_id + room_id + allocation_id` | missing room / allocation: `ReferenceCompositionError::StateBoundaryViolation` |
| `bind_signaling_to_sfu` | room exists/open and route exists | insert binding key `correlation_id + room_id + session_id + route_id` | missing room / route: `ReferenceCompositionError::StateBoundaryViolation` |
| `validate_reference_composition_state` | all binding references exist | no mutation | orphan binding: `ReferenceCompositionError::StateBoundaryViolation` |

Composition binding records references only. Composition MUST NOT copy plane state into a new authority store.

### 2.6 Collapse conditions

- absent state and rejected state are represented by the same mutation result.
- A state map key uses random, provider, wall-clock, memory, or raw payload data.
- A state map key uses the `Debug` representation instead of the Kernel `as_str()` value.
- `LeaveRoom` deletes the participant evidence trace.
- SFU state stores raw packet bytes or borrowed packet slices.
- Composition creates product authority state.

---

## 3. reference-local API and state types

This section fixes the reference-local API, state fields, state transitions, and Kernel contract builder mapping. This chapter does not redefine Kernel semantics. implementations-local types are limited to input, projection, and evidence for building the Kernel public contract.

### 3.1 Shared API rule

The reference-local API of reference packages is fixed to the following shape.

| API class | Required naming | Return boundary |
|---|---|---|
| input constructor | `Reference{Plane}Input::new(...)` | implementations-local input |
| Kernel contract builder | `build_kernel_{plane}_command(...)` or `build_kernel_{plane}_item(...)` | Kernel public contract type |
| state application | `apply_reference_{plane}(...)` | implementations-local outcome |
| event / evidence projection | `project_reference_{plane}_event(...)` | Kernel event or evidence record |
| validation | `validate_reference_{plane}_state(...)` | implementations-local validation result |

A Kernel contract object that does not expose payload accessors MUST NOT be treated as a local source of truth. The original implementations-local input remains the source for reference behavior application.

### 3.2 Product input allow-list boundary

The reference types the product implementation can consume are limited to `ReferenceSignalingOutcome`, `ReferenceTurnOutcome`, and `ReferenceSfuOutcome` owned by `arcrtc-reference-output`.

`Reference*State`, `ReferenceSfuAction`, fixture, local auth, runtime, composition state, `apply_reference_*`, and `validate_reference_*` are for reference behavior / reference tests / reference composition, and are not the input source of product plane packages.

### 3.3 Signaling API

**Required types**

| Type | File | Required fields |
|---|---|---|
| `ReferenceSignalingCommandInput` | `reference-implementation/signaling/src/kernel_contract.rs` | `correlation_id: CorrelationId`, `room_id: RoomId`, `participant_id: Option<ParticipantId>`, `kind: SignalingCommandKind`, `payload: ReferenceSignalingPayload` |
| `ReferenceSignalingPayload` | `reference-implementation/signaling/src/kernel_contract.rs` | enum variants of section 1.3 |
| `ReferenceSignalingState` | `reference-implementation/signaling/src/state.rs` | `rooms: BTreeMap<String, ReferenceRoomState>`, `participants: BTreeMap<String, ReferenceParticipantState>` |
| `ReferenceRoomState` | `reference-implementation/signaling/src/state.rs` | `room_id: RoomId`, `phase: ReferenceRoomPhase`, `participant_ids: BTreeSet<String>` |
| `ReferenceParticipantState` | `reference-implementation/signaling/src/state.rs` | `participant_id: ParticipantId`, `room_id: RoomId`, `phase: ReferenceParticipantPhase` |
| `ReferenceSignalingOutcome` | `reference-implementation/output/src/signaling.rs` | `correlation_id: CorrelationId`, `kind: SignalingEventKind`, `room_id: RoomId`, `participant_id: Option<ParticipantId>`, `implementation_reason: ImplementationEvidenceReason` |

**Required functions**

| Function | Required signature shape | Rule |
|---|---|---|
| `build_kernel_signaling_command` | `(&ReferenceSignalingCommandInput) -> Result<SignalingCommand<ReferenceSignalingPayload>, ReferenceSignalingError>` | builds `CommandEnvelope<SignalingSubject>` with `TargetSurface::Signaling` |
| `apply_reference_signaling` | `(&mut ReferenceSignalingState, &ReferenceSignalingCommandInput) -> Result<ReferenceSignalingOutcome, ReferenceSignalingError>` | uses local input, not Kernel command private payload |
| `project_reference_signaling_event` | `(&ReferenceSignalingOutcome, ReferenceSignalingPayload) -> SignalingEvent<ReferenceSignalingPayload>` | emits Kernel event projection |
| `validate_reference_signaling_state` | `(&ReferenceSignalingState) -> Result<(), ReferenceSignalingError>` | rejects impossible room / participant relation |

**State transitions**

| Command kind | Required precondition | Success effect | Reject reason |
|---|---|---|---|
| `JoinRoom` | room absent/open and participant absent/left | room open, participant joined | `STATE_BOUNDARY_VIOLATION` |
| `JoinRoom` | participant already joined in same room | idempotent success; no mutation | none |
| `LeaveRoom` | participant joined in room | participant left | `STATE_BOUNDARY_VIOLATION` |
| `SendOffer` | participant joined | event `OfferReceived` | `FIXTURE_IDENTITY_INVALID` or `STATE_BOUNDARY_VIOLATION` |
| `SendAnswer` | participant joined | event `AnswerReceived` | `FIXTURE_IDENTITY_INVALID` or `STATE_BOUNDARY_VIOLATION` |
| `SendIceCandidate` | participant joined | event `IceCandidateReceived` | `FIXTURE_IDENTITY_INVALID` or `STATE_BOUNDARY_VIOLATION` |
| `RequestTurnCredential` | participant joined | event `TurnCredentialAvailable` | `FIXTURE_IDENTITY_INVALID` or `STATE_BOUNDARY_VIOLATION` |
| `AcknowledgeForward` | participant joined | event `Joined` or forwarding acknowledgement | `STATE_BOUNDARY_VIOLATION` |

### 3.4 TURN API

**Required types**

| Type | File | Required fields |
|---|---|---|
| `ReferenceTurnCommandInput` | `reference-implementation/turn/src/kernel_contract.rs` | `kind: TurnCommandKind`, `transaction_id: TurnTransactionId`, `references: TurnReferenceSet`, `allocation_id: Option<AllocationId>`, `permission_id: Option<PermissionId>`, `channel_bind_id: Option<ChannelBindId>`, `credential_ref: Option<CredentialRef>`, `peer_address: Option<CorePeerAddress>`, `requested_lifetime: Option<TurnRequestedLifetimeSeconds>`, `relay_packet_id: Option<PacketId>` |
| `ReferenceTurnState` | `reference-implementation/turn/src/state.rs` | `allocations: BTreeMap<String, ReferenceAllocationState>`, `permissions: BTreeMap<String, ReferencePermissionState>`, `channel_binds: BTreeMap<String, ReferenceChannelBindState>` |
| `ReferenceAllocationState` | `reference-implementation/turn/src/state.rs` | `allocation_id: AllocationId`, `phase: AllocationState`, `credential_ref: Option<CredentialRef>` |
| `ReferencePermissionState` | `reference-implementation/turn/src/state.rs` | `permission_id: PermissionId`, `allocation_id: AllocationId`, `peer_address: CorePeerAddress`, `phase: PermissionState` |
| `ReferenceChannelBindState` | `reference-implementation/turn/src/state.rs` | `channel_bind_id: ChannelBindId`, `permission_id: PermissionId`, `phase: ChannelBindState` |
| `ReferenceTurnOutcome` | `reference-implementation/output/src/turn.rs` | `kind: TurnDecisionKind`, `implementation_reason: ImplementationEvidenceReason` |

**Required functions**

| Function | Required signature shape | Rule |
|---|---|---|
| `build_kernel_turn_command` | `(&ReferenceTurnCommandInput) -> Result<TurnCommand, ReferenceTurnError>` | delegates shape validation to `TurnCommand::try_new` |
| `apply_reference_turn` | `(&mut ReferenceTurnState, &ReferenceTurnCommandInput) -> Result<ReferenceTurnOutcome, ReferenceTurnError>` | updates in-memory state only; local reference fields on `ReferenceTurnCommandInput` are the source of mutation because frozen Kernel `TurnReferenceSet` does not expose reference accessors |
| `validate_reference_turn_state` | `(&ReferenceTurnState) -> Result<(), ReferenceTurnError>` | rejects orphan permission / channel bind |

**State transitions**

| Command kind | Required precondition | Success effect | Reject reason |
|---|---|---|---|
| `Allocate` | credential fixture valid | allocation active | `FIXTURE_IDENTITY_INVALID` |
| `Refresh` | allocation active | lifetime projection refreshed | `STATE_BOUNDARY_VIOLATION` |
| `CreatePermission` | allocation active and peer address present | permission active | `STATE_BOUNDARY_VIOLATION` |
| `ChannelBind` | permission active and channel bind id present | channel bind active | `STATE_BOUNDARY_VIOLATION` |
| `RelayData` | allocation and permission active, packet id present | relay allowed outcome | `STATE_BOUNDARY_VIOLATION` |

### 3.5 SFU API

**Required types**

| Type | File | Required fields |
|---|---|---|
| `ReferenceSfuContractInput<Payload>` | `reference-implementation/sfu/src/kernel_contract.rs` | `model_kind: SfuModelKind`, `references: SfuReferenceSet`, `payload: Payload` |
| `ReferenceSfuAction` | `reference-implementation/sfu/src/state.rs` | enum variants listed below |
| `ReferenceSfuSuppressionSource` | `reference-implementation/sfu/src/state.rs` | enum variants `Backpressure`, `Quality` |
| `ReferenceSfuState` | `reference-implementation/sfu/src/state.rs` | `sessions: BTreeMap<String, ReferenceSfuSessionState>`, `endpoints: BTreeMap<String, ReferenceEndpointState>`, `routes: BTreeMap<String, ReferenceRouteState>` |
| `ReferenceSfuSessionState` | `reference-implementation/sfu/src/state.rs` | `session_id: SessionId`, `phase: SfuSessionState` |
| `ReferenceEndpointState` | `reference-implementation/sfu/src/state.rs` | `endpoint_id: EndpointId`, `session_id: SessionId`, `phase: SfuEndpointState` |
| `ReferenceRouteState` | `reference-implementation/sfu/src/state.rs` | `route_id: RouteId`, `session_id: SessionId`, `endpoint_id: EndpointId`, `stream_id: StreamId`, `phase: SfuRouteState` |
| `ReferenceSfuOutcome` | `reference-implementation/output/src/sfu.rs` | `kind: SfuDecisionKind`, `implementation_reason: ImplementationEvidenceReason` |

`ReferenceSfuAction` is implementations-local. It carries the IDs needed by reference state mutation because frozen Kernel `SfuReferenceSet` only exposes `session_id()` publicly. Variants are fixed to:

- `AdmitParticipant { session_id: SessionId, endpoint_id: EndpointId }`
- `RejectParticipant { session_id: SessionId, endpoint_id: EndpointId }`
- `PublishStream { session_id: SessionId, endpoint_id: EndpointId, stream_id: StreamId }`
- `SubscribeRoute { session_id: SessionId, endpoint_id: EndpointId, stream_id: StreamId, route_id: RouteId }`
- `SelectRoute { session_id: SessionId, endpoint_id: EndpointId, stream_id: StreamId, route_id: RouteId }`
- `SuppressForwarding { route_id: RouteId, source: ReferenceSfuSuppressionSource }`
- `DropForwarding { route_id: RouteId }`
- `CloseSession { session_id: SessionId }`

`SuppressForwarding` with `source: Backpressure` maps to `SfuRouteState::SuppressedByBackpressure`; with `source: Quality` it maps to `SfuRouteState::SuppressedByQuality`.

**SFU action to Kernel decision mapping**

| ReferenceSfuAction | Kernel `SfuDecisionKind` | Kernel `SfuModelKind` |
|---|---|---|
| `AdmitParticipant` | `ParticipantAdmission` | `ParticipantEndpoint` |
| `RejectParticipant` | `ParticipantAdmission` | `RejectionReason` |
| `PublishStream` | `Publication` | `Publication` |
| `SubscribeRoute` | `Subscription` | `Subscription` |
| `SelectRoute` | `RouteSelection` | `RouteCandidate` |
| `SuppressForwarding` | `Forwarding` | `ForwardingIntent` |
| `DropForwarding` | `BackpressureAction` | `BackpressureState` |
| `CloseSession` | `DegradationRecovery` | `SfuSession` |

**Required functions**

| Function | Required signature shape | Rule |
|---|---|---|
| `build_kernel_sfu_item` | `(ReferenceSfuContractInput<Payload>) -> SfuContractItem<Payload>` | creates a Kernel contract item without owning Kernel semantics |
| `apply_reference_sfu` | `(&mut ReferenceSfuState, &ReferenceSfuAction) -> Result<ReferenceSfuOutcome, ReferenceSfuError>` | updates the in-memory routing projection using implementations-local action fields |
| `build_borrowed_packet_view` | `(&PacketId, &StreamId, &EndpointId, PacketHeaderSemanticView, &[u8], &[u8]) -> SfuPacketView<'_>` | keeps packet bytes borrowed |
| `validate_reference_sfu_state` | `(&ReferenceSfuState) -> Result<(), ReferenceSfuError>` | rejects orphan endpoint / route |

**Packet boundary**

`build_borrowed_packet_view` MUST NOT copy raw packet or payload bytes. The returned `SfuPacketView<'packet>` cannot outlive the driver-owned byte slice. Any rewrite request must return `PacketCopyPolicy` or `PacketRewriteTransformIntent`; it MUST NOT mutate Kernel packet view state.

Reference state mutation MUST NOT read endpoint / stream / route / packet fields from Kernel `SfuReferenceSet`. Those fields are not public accessors in the frozen Kernel contract. Reference mutation uses `ReferenceSfuAction` fields as implementations-local input and maps only the outcome kind to Kernel `SfuDecisionKind`.

### 3.6 Composition API

**Required types**

| Type | File | Required fields |
|---|---|---|
| `ReferenceCompositionState` | `reference-implementation/composition/src/composition_state.rs` | `signaling: ReferenceSignalingState`, `turn: ReferenceTurnState`, `sfu: ReferenceSfuState`, `bindings: BTreeMap<String, ReferenceCrossPlaneBinding>` |
| `ReferenceCrossPlaneBinding` | `reference-implementation/composition/src/composition_state.rs` | `correlation_id: CorrelationId`, `room_id: Option<RoomId>`, `session_id: Option<SessionId>`, `allocation_id: Option<AllocationId>`, `route_id: Option<RouteId>` |
| `ReferenceCompositionOutcome` | `reference-implementation/output/src/composition.rs` | `correlation_id: CorrelationId`, `implementation_reason: ImplementationEvidenceReason` |

**Required functions**

| Function | Required signature shape | Rule |
|---|---|---|
| `bind_signaling_to_turn` | `(&mut ReferenceCompositionState, CorrelationId, RoomId, AllocationId) -> Result<ReferenceCompositionOutcome, ReferenceCompositionError>` | records cross-plane reference only |
| `bind_signaling_to_sfu` | `(&mut ReferenceCompositionState, CorrelationId, RoomId, SessionId, RouteId) -> Result<ReferenceCompositionOutcome, ReferenceCompositionError>` | records cross-plane reference only |
| `validate_reference_composition_state` | `(&ReferenceCompositionState) -> Result<(), ReferenceCompositionError>` | rejects orphan cross-plane binding |

For the exact input / step / outcome of the composition runtime bridge, see Chapter 07.

### 3.7 Error rule

Each `Reference*Error` enum MUST map to `ImplementationEvidenceReason`. No reference error enum may contain `Unknown` (MUST NOT).

### 3.8 Collapse conditions

- A Kernel command private payload is treated as the source of truth for implementations behavior.
- Reference state reads a database / queue / external storage.
- Raw packet / payload bytes of `SfuPacketView` are stored in reference state.
- `Unknown` is added to a reference error enum.
- The Kernel reason catalog is extended with an implementations-local reason.
- A product plane package treats a reference type / function / module other than `arcrtc-reference-output` as an input source.

---

## 4. Output boundary (3-type allow-list)

This section limits the reference implementation surface the product implementation can consume to the outcome types of the `arcrtc-reference-output` package. The product implementation does not inherit reference source, reference internal state, fixture, local auth, runtime, composition state, or reference success claims.

### 4.1 Package boundary

| Item | Fixed value |
|---|---|
| package path | `reference-implementation/output` |
| package name | `arcrtc-reference-output` |
| owner | implementations reference-output boundary |
| consumers | reference plane packages, product plane packages |
| non-consumers | Kernel |

`arcrtc-reference-output` owns only reference output types. It does not own state mutation, fixture validation, local auth, runtime execution, composition binding, or Kernel contract builder (MUST NOT).

### 4.2 Public output types

| Type | File | Reference output fields |
|---|---|---|
| `ReferenceSignalingOutcome` | `reference-implementation/output/src/signaling.rs` | `correlation_id: CorrelationId`, `kind: SignalingEventKind`, `room_id: RoomId`, `participant_id: Option<ParticipantId>`, `implementation_reason: ImplementationEvidenceReason` |
| `ReferenceTurnOutcome` | `reference-implementation/output/src/turn.rs` | `kind: TurnDecisionKind`, `implementation_reason: ImplementationEvidenceReason` |
| `ReferenceSfuOutcome` | `reference-implementation/output/src/sfu.rs` | `kind: SfuDecisionKind`, `implementation_reason: ImplementationEvidenceReason` |
| `ReferenceCompositionOutcome` | `reference-implementation/output/src/composition.rs` | `correlation_id: CorrelationId`, `implementation_reason: ImplementationEvidenceReason` |

The reference types a product plane package can receive as policy input are limited to the three: `ReferenceSignalingOutcome`, `ReferenceTurnOutcome`, `ReferenceSfuOutcome`. `ReferenceCompositionOutcome` is output for reference composition / reference ops and MUST NOT be used as product plane policy input.

### 4.3 Product import rule

A product plane package allows only the following imports.

| Product package | Allowed reference-output import |
|---|---|
| `arcrtc-product-signaling` | `arcrtc_reference_output::ReferenceSignalingOutcome` |
| `arcrtc-product-turn` | `arcrtc_reference_output::ReferenceTurnOutcome` |
| `arcrtc-product-sfu` | `arcrtc_reference_output::ReferenceSfuOutcome` |

A product plane package's `arcrtc_reference_output` import is limited to exactly the allow-list above. wildcard import, module import, re-export, import via type alias, and `ReferenceCompositionOutcome` import are prohibited (MUST NOT).

A product plane package MUST NOT import the following.

- `arcrtc_reference_output::*`
- `arcrtc_reference_output` (module import)
- `pub use arcrtc_reference_output::*`
- `pub use arcrtc_reference_output::ReferenceCompositionOutcome`
- `ReferenceCompositionOutcome`
- `arcrtc_reference_signaling`, `arcrtc_reference_turn`, `arcrtc_reference_sfu`, `arcrtc_reference_composition`, `arcrtc_reference_ops`
- `ReferenceSignalingState`, `ReferenceRoomState`, `ReferenceParticipantState`
- `ReferenceTurnState`, `ReferenceAllocationState`, `ReferencePermissionState`, `ReferenceChannelBindState`
- `ReferenceSfuState`, `ReferenceSfuSessionState`, `ReferenceEndpointState`, `ReferenceRouteState`
- `ReferenceSfuAction`, `ReferenceSfuSuppressionSource`
- `apply_reference_*`, `validate_reference_*`
- fixture / local auth modules
- reference runtime / composition modules

### 4.4 Reference producer rule

A reference plane package returns the outcome types of `arcrtc-reference-output`. A reference plane package MUST NOT define a reference output outcome type independently inside `state.rs`.

| Reference package | Producer function | Output type owner |
|---|---|---|
| `arcrtc-reference-signaling` | `apply_reference_signaling` | `arcrtc-reference-output::ReferenceSignalingOutcome` |
| `arcrtc-reference-turn` | `apply_reference_turn` | `arcrtc-reference-output::ReferenceTurnOutcome` |
| `arcrtc-reference-sfu` | `apply_reference_sfu` | `arcrtc-reference-output::ReferenceSfuOutcome` |

### 4.5 Test assertion rule

Boundary tests MUST fail closed when a product package imports any prohibited reference package, module, type, function, fixture, local auth, runtime, or composition symbol. Boundary tests MUST also fail closed when a product package imports `arcrtc_reference_output` by wildcard / module path / re-export, or imports `ReferenceCompositionOutcome`. The expected failure reason for a source import scope violation is `COMMAND_SCOPE_MISMATCH`.

### 4.6 Collapse conditions

- A product plane package depends on a reference package other than `arcrtc-reference-output`.
- A product plane package has `arcrtc_reference_output::*`, `arcrtc_reference_output` module import, re-export, import via type alias, or `ReferenceCompositionOutcome` import.
- A product plane package imports a reference state, action, fixture, local auth, runtime, or composition symbol.
- A reference plane package duplicates a reference output outcome type outside `arcrtc-reference-output`.
- `ReferenceCompositionOutcome` is treated as product plane policy input.
- A reference success claim is repurposed as product behavior / production readiness / live readiness.

---

## 5. runtime lifecycle (all states)

This section fixes the lifecycle API for reference runtime, product runtime, startup, shutdown, drain, and correlation propagation. The runtime lifecycle is implementations process orchestration and does not own Kernel semantics. A runtime lifecycle outcome is not command evidence. build / test command evidence is generated only by the runner helper of section 6.

### 5.1 Runtime states

| Type | File | Variants |
|---|---|---|
| `ImplementationRuntimeState` | `reference-implementation/ops/src/runtime.rs` and `product-implementation/deployment/src/runtime.rs` | `Created`, `Starting`, `Running`, `Draining`, `Stopped`, `Failed` |
| `ImplementationShutdownMode` | `reference-implementation/ops/src/runtime.rs` and `product-implementation/rollback/src/drain.rs` | `Immediate`, `GracefulLocal`, `DrainThenStop` |
| `ImplementationRuntimePlane` | `reference-implementation/ops/src/runtime.rs` | `Signaling`, `Turn`, `Sfu`, `Composition` |
| `ReferenceRuntimeOutcome` | `reference-implementation/ops/src/runtime.rs` | `correlation_id`, `state`, `implementation_reason`, `non_claim_scope` |
| `ProductRuntimeOutcome` | `product-implementation/deployment/src/runtime.rs` | `correlation_id`, `state`, `implementation_reason`, `non_claim_scope` |

### 5.2 Reference runtime API

| Function | Required signature shape | Rule |
|---|---|---|
| `ReferenceRuntime::new` | `(ReferenceCompositionState) -> Self` | starts in `Created` |
| `ReferenceRuntime::start` | `(&mut self, CorrelationId) -> Result<ReferenceRuntimeOutcome, ReferenceRuntimeError>` | transition `Created -> Starting -> Running` |
| `ReferenceRuntime::shutdown` | `(&mut self, CorrelationId, ImplementationShutdownMode) -> Result<ReferenceRuntimeOutcome, ReferenceRuntimeError>` | transition `Running -> Draining -> Stopped` |
| `ReferenceRuntime::state` | `(&self) -> ImplementationRuntimeState` | observes local runtime state only |
| `ReferenceRuntime::composition` | `(&self) -> &ReferenceCompositionState` | returns borrowed composition state |

Reference runtime MUST NOT open public endpoints. Reference runtime MUST NOT claim production readiness or live readiness.

### 5.3 Product runtime API

| Function | Required signature shape | Rule |
|---|---|---|
| `ProductRuntime::new` | `(ProductRuntimeProfile) -> Self` | starts in `Created` |
| `ProductRuntime::select` | `(&ProductRuntimeProfile) -> ProductRuntimeSelection` | no readiness claim |
| `ProductRuntime::start` | `(&mut self, CorrelationId) -> Result<ProductRuntimeOutcome, ProductRuntimeError>` | bounded product runtime start outcome |
| `ProductRuntime::drain` | `(&mut self, ProductDrainPlan) -> Result<ProductRuntimeOutcome, ProductRuntimeError>` | bounded drain outcome |
| `ProductRuntime::restore` | `(&mut self, ProductRestorePlan) -> Result<ProductRuntimeOutcome, ProductRuntimeError>` | bounded restore outcome |

A product runtime with `ProductionDeferred` or `LiveDeferred` environment class MUST fail closed for readiness claims.

### 5.4 Correlation propagation

Every runtime command MUST carry one `CorrelationId`. The same correlation id MUST appear in: the runtime lifecycle outcome, the plane-specific evidence record, and the composition binding record. If a runtime action lacks a correlation id equivalent, the runtime action MUST fail before creating an adopted outcome (fail-closed).

### 5.5 Timeout / cancellation rule

Runtime timeout and cancellation are implementations-local execution facts.

| Event | Required reason |
|---|---|
| task failed before start | `RUNTIME_EXECUTOR_ERROR` |
| shutdown lacks evidence fields | `EVIDENCE_FIELDS_INCOMPLETE` |
| command targets wrong working directory | `COMMAND_SCOPE_MISMATCH` |
| readiness command lacks its readiness admission record | `READINESS_NOT_ADMITTED` |

### 5.6 Collapse conditions

- Runtime start success is treated as production readiness.
- Runtime drain success is treated as live readiness.
- Runtime lifecycle state is treated as Kernel state authority.
- A runtime outcome without a correlation id is adopted.
- A runtime lifecycle outcome is adopted as build / test command evidence.
- The runtime API claims public endpoint availability.

---

## 6. command evidence runner (execution and evidence generation procedure)

This section fixes the implementations command evidence runner, output path, JSON writer, and validation commands. The command evidence runner only generates evidence records; it does not automatically claim build success, test pass, benchmark pass, or readiness success.

### 6.1 Runner ownership

| Runner | Owner package | File | Command class |
|---|---|---|---|
| reference build command evidence helper | `arcrtc-reference-ops` | `src/runtime.rs` | `Build` when called by bounded build command |
| product monitoring evidence helper | `arcrtc-product-monitoring` | `src/evidence.rs` | `Build` / `Test` |
| benchmark evidence helper | benchmark package | `src/evidence.rs` | `Benchmark` |
| real-device evidence helper | real-device package | `src/evidence.rs` | `RealDevice` |
| production readiness evidence matrix | production-readiness package | `tests/production-readiness/tests/*.rs` | `ProductionReadiness` |
| live readiness evidence matrix | live package | `tests/live/tests/*.rs` | `LiveReadiness` |

The shared evidence record / reason type owner is `arcrtc-implementation-evidence`. implementations command root / target root / evidence root constants are owned by `implementation-support/evidence/src/validation.rs` and re-exported by `arcrtc-implementation-evidence`. No runner may write evidence outside `implementations/target/` (MUST NOT).

### 6.2 Output path rule

| Evidence class | Output directory |
|---|---|
| format | `target/implementation-evidence/format/` |
| build | `target/implementation-evidence/build/` |
| test | `target/implementation-evidence/test/` |
| benchmark | `target/implementation-evidence/benchmark/` |
| real-device | `target/implementation-evidence/real-device/` |
| production readiness | `target/implementation-evidence/production-readiness/` |
| live readiness | `target/implementation-evidence/live-readiness/` |

The output filename is fixed to `{correlation_id}.json`. The writer MUST reject a `correlation_id` that is not safe as a single filename component. The writer MUST NOT silently rewrite or sanitize the filename, because that would break the `{correlation_id}.json` evidence binding.

### 6.3 JSON writer rule

The JSON writer MUST serialize `ImplementationEvidenceRecord` using snake_case field names and the defined wire values. Required validation before write:

- `validate_evidence_record(&record)` from `arcrtc-implementation-evidence` returns `Ok(())`.
- The output path is under the allowed evidence directory.
- The reference ops writer accepts only `Reference` / `Ops` records whose command class and owner binding match the reference runner table.
- The product monitoring evidence helper accepts only `Product` records whose command class is `Build` or `Test`, whose target package is an `arcrtc-product-*` package, and whose target scope is under `product-implementation/`.
- The build evidence helper rejects commands other than `cargo build --workspace --all-targets`.
- The test evidence helper rejects commands other than `cargo test --workspace --all-targets`.
- The reference ops writer MUST NOT accept `Test` evidence records; reference ops owns build evidence only.

Base validation failure MUST return `EVIDENCE_FIELDS_INCOMPLETE`, `COMMAND_SCOPE_MISMATCH`, or `READINESS_NOT_ADMITTED`. Benchmark extension validation failure may additionally return `BENCHMARK_SCOPE_MISMATCH`, and real-device extension validation failure may additionally return `REAL_DEVICE_SCOPE_MISMATCH`.

### 6.4 Initial validation commands

| Command | Working directory | Expected result |
|---|---|---|
| `rg` for placeholder / forbidden tokens (`{PROJECT}` / `{MISSION}` / `TODO` / `TBD` / undefined markers, etc.) | `implementations` | no output |
| `rg` for boundary tokens (`v0.2/userland` / `userland` / `adapters/` / `apps/`, etc.) | `implementations` | no output |
| `rg` for `UNKNOWN` / `Unknown` / `Other` / `raw String` | `implementations` | prohibition text only |

test / benchmark / real-device command invocation uses the explicit manifest path.

### 6.5 Claim boundary

| Runner result | May support | MUST NOT support |
|---|---|---|
| evidence JSON write success | evidence shape generation | command target success |
| build command evidence | build command result | behavior correctness |
| test command evidence | tested behavior | production readiness |
| benchmark command evidence | measurement record | threshold satisfaction |
| real-device command evidence | bounded device command result | live readiness |

### 6.6 Collapse conditions

- The evidence writer accepts an empty correlation id.
- The evidence writer writes outside `target/implementation-evidence/`.
- The runner emits JSON with missing required fields.
- The runner writes JSON without calling `validate_evidence_record`.
- The runner converts command execution success into a readiness claim.
- The runner adopts a Kernel working directory command result as implementations evidence.

---

## 7. Invariants of this chapter (summary)

1. Reference state / fixtures are local deterministic projections; they are not Kernel state authority, database, production state, or readiness evidence.
2. absent state and rejected state MUST NOT be represented by the same enum variant / mutation result.
3. Every state map key is derived from a Kernel `as_str()` value and does not use random / provider / wall-clock / memory / raw payload / `Debug` representation.
4. The reference types the product can consume are limited to the 3-type allow-list (`ReferenceSignalingOutcome` / `ReferenceTurnOutcome` / `ReferenceSfuOutcome`). `ReferenceCompositionOutcome` is not a product policy input.
5. Every outcome / runtime command carries a correlation id, and reference error enums have no `Unknown`.
6. The command evidence runner only generates evidence records; it does not automatically claim build / test / benchmark / readiness success (fail-closed).
