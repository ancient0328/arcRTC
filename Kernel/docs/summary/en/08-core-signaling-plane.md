# core-signaling-plane

Status: public summary projection
Date: 2026-07-06 JST

## Purpose

This chapter internalizes the current complete specification of the Signaling contract and Signaling state machine owned by `core/signaling` of arcRTC v0.2 Kernel, at a granularity sufficient for re-implementation from this chapter alone. Signaling is a protocol contract shared by SDK and server, and is not defined for the convenience of a network framework. This chapter internalizes room state, command/event semantics, accept/reject boundary, all states and all transitions, guards, reject conditions, idempotency/ordering/correlation rules, fail-closed conditions, and the boundaries with driver/entrypoint/SDK.

Dependency direction notation: `A <- B` means "B depends on A". The pure semantics of Signaling are owned by core and separated from I/O (WebSocket/HTTP/encoding/runtime). v0.2 Kernel does not own the Signaling product system; the reference distro / product distro is placed in distro outside the Kernel.

---

## 1. Signaling Contract

### 1.1 Boundary (owner)

| Domain | Owner | Rule |
|---|---|---|
| Signaling semantics | `core/signaling` | room identity, participant transport identity, command/event schema semantics, state transition, accept/reject, correlation, ordering/idempotency policy, TURN credential request/delivery abstract contract, protocol violation fail-closed rule |
| transport and encoding | `drivers/network` | WebSocket frame, HTTP request/response, external JSON/binary encoding, network timeout, connection close mapping, external error code mapping, driver-local logging/metrics |
| executable contract / composition evidence surface | `entrypoints/signaling-server` | binary entrypoint, configuration loading, dependency wiring, driver selection, process lifecycle |
| Signaling-only client boundary | `sdk/*` | exposes Signaling-only contract to users |

command idempotency / replay / correlation, authorization context / communication policy, cross-plane identity / session binding, SDK public API projection, ICE candidate / connectivity lifecycle, and out-of-scope feature admission follow their respective rule systems (this chapter specifies only the Signaling contract internalized portion).

### 1.2 Identity Rule

- Signaling does not own user identity (MUST).
- client-provided ID MUST NOT be treated as trusted identity.
- The externally exposed participant identity is a server-generated opaque transport ID (MUST).
- Signaling join success is not SFU endpoint admission, TURN allocation, or secure media authorization without an admitted cross-plane binding and target-plane decision (MUST).

### 1.3 Command Set (closed set, public)

The public command set of v0.2 initial Signaling contract is limited to the following. When adding, it MUST be explicitly declared as a compatible capability or a new contract version per the protocol versioning rule.

| Command | Core responsibility |
|---|---|
| `JoinRoom` | room membership request, accept/reject |
| `LeaveRoom` | membership termination |
| `SendOffer` | SDP offer relay intent |
| `SendAnswer` | SDP answer relay intent |
| `SendIceCandidate` | ICE candidate relay intent |
| `RequestTurnCredential` | TURN credential delivery request boundary |
| `AcknowledgeForward` | accepted forwarding acknowledgement |

### 1.4 Event Set (closed set, public)

The public event set of v0.2 initial Signaling contract is limited to the following. When adding, it MUST be explicitly declared as a compatible capability or a new contract version per the protocol versioning rule.

| Event | Core responsibility |
|---|---|
| `Joined` | accepted room membership |
| `Rejected` | fail-closed rejection with closed reason |
| `ParticipantJoined` | room-visible participant lifecycle |
| `ParticipantLeft` | room-visible participant lifecycle |
| `OfferReceived` | offer relay event |
| `AnswerReceived` | answer relay event |
| `IceCandidateReceived` | ICE relay event |
| `TurnCredentialAvailable` | credential delivery event |
| `ProtocolViolation` | closed violation classification |

### 1.5 Fail-Closed Rule (reason code closed set)

The following cases MUST be treated as reject / violation and connected to a cataloged reason code.

| Failure | Reason code |
|---|---|
| missing correlation ID | `missing_correlation_id` |
| malformed command | `malformed_command` |
| unauthorized token verification result | `token_verification_failed` |
| room policy violation | `room_not_accepting_join` |
| room materialization or active room capacity exceeded | `room_capacity_exceeded` |
| room participant admission capacity exceeded | `admission_capacity_exceeded` |
| room lifecycle duration exceeded | `room_lifetime_exceeded` |
| room is draining and rejects new join or command | `room_draining` |
| room is already closed | `room_closed` |
| room close/drain transition is invalid | `room_close_not_allowed` |
| invalid participant state transition | `participant_not_joined` |
| participant verification or join policy rejected membership | `participant_rejected` |
| duplicate command outside idempotency rule | `duplicate_command` |
| idempotency replay conflict | `idempotency_payload_mismatch` |
| replay window expired | `idempotency_window_expired` |
| required authorization context missing or invalid | `authorization_context_missing` or `authorization_context_invalid` |
| authorization policy denied requested communication action | `authorization_policy_denied` |
| ordering violation | `command_order_violation` |
| signaling command queue capacity or wait limit exceeded | `signaling_command_queue_bound_exceeded` |
| unsupported command version | `unsupported_command_version` |
| cross-plane binding required but absent or invalid | cross-plane binding reason |

### 1.6 External Encoding Rule

- JSON, MessagePack, binary frame, WebSocket close code, and HTTP status are handled at the driver boundary (MUST).
- core handles core-owned command / event type, not encoded payload (MUST).
- `token_verification_failed` is a Signaling public wrapper reason. The concrete token verification failure MUST be preserved in the `token_verification_decision` audit event.

### 1.7 Prohibited semantics (MUST NOT be included in the Signaling contract)

medical role, application workflow state, chat semantics, recording semantics, screen share semantics, DataChannel application semantics, UI / end-user workflow, user authentication issuance, regulated payload.

### 1.8 collapse conditions (Signaling contract)

- WebSocket handler holds the authority of Signaling state transition.
- SDK and server have different Signaling command semantics.
- Signaling payload depends on domain identity.
- protocol violation is handled with an open-ended string only.
- Signaling duplicate/replay behavior is defined by SDK or network driver.
- token verification success is treated as Signaling authorization success without authorization context policy.
- Signaling participant success is treated as SFU/TURN/secure media success without cross-plane binding.
- out-of-scope feature is added to Signaling command/event set without specification admission.

---

## 2. Signaling Boundary (plane separation decision)

### 2.1 Decision

The core semantics of Signaling are owned by `core/signaling`. WebSocket, HTTP, Axum, tokio task, socket, request parsing, and response encoding are owned by drivers or entrypoints (MUST).

### 2.2 Core Signaling Responsibilities

`core/signaling` MUST own the following.

- handling of room identity
- handling of participant transport identity
- semantics of command / event schema
- state transition of join / leave / offer / answer / ice candidate / forward accepted
- accept / reject boundary
- correlation ID requirement
- policy of ordering / idempotency / duplicate handling
- abstract contract of TURN credential request / delivery
- fail-closed rule of protocol violation

### 2.3 Driver Responsibilities

The Signaling driver MUST own the following. WebSocket frame reception, HTTP request/response, external JSON/binary encoding, network timeout, connection close mapping, external error code mapping, driver-local logging/metrics export.

### 2.4 Entrypoint Responsibilities

The Signaling entrypoint MUST own the following. binary entrypoint, configuration loading, dependency wiring, driver selection, process lifecycle.

### 2.5 SDK Relation

SDK exposes the Signaling-only contract to users. SDK MUST NOT own browser/native PeerConnection, media capture, screen share, recording, or regulated support.

### 2.6 Prohibited Placement

- placing the authority of room state transition inside the WebSocket handler.
- placing the rule of join accept / reject in entrypoints.
- placing server-side authorization decision in the SDK.
- giving the Signaling message an entrypoint-specific user role / medical role / domain meaning.
- treating a client-provided participant ID as trusted identity.

### 2.7 collapse conditions (boundary)

- something other than `core/signaling` holds the authority of Signaling state transition.
- WebSocket / HTTP types enter core.
- SDK owns media / regulated / auth issuance.
- Signaling depends on domain identity rather than opaque transport ID.

---

## 3. Signaling State Machine

The authority of Signaling state transition is owned by `core/signaling`.

### 3.1 State Owners

| State family | Owner | Driver role |
|---|---|---|
| Room state | core | external event conversion |
| Participant state | core | connection observation |
| Command idempotency state | core | optional storage implementation for core-owned idempotency semantics |
| Wire connection state | driver | converted to core event if relevant |
| Process lifecycle | entrypoints | no protocol semantics |

Process lifecycle does not authorize entrypoints or drivers to mutate Signaling state without the transitions below (MUST).

### 3.2 Room States (closed set)

| State | Meaning |
|---|---|
| `room_absent` | room does not exist or is not materialized |
| `room_open` | room accepts join and relay commands |
| `room_draining` | room rejects new joins but allows controlled leave / close |
| `room_closed` | room rejects all protocol commands except idempotent close observation |

### 3.3 Participant States (closed set)

| State | Meaning |
|---|---|
| `participant_new` | transport observed but not accepted |
| `participant_verifying` | token / policy verification in progress |
| `participant_joined` | participant is accepted in room |
| `participant_leaving` | leave is accepted and cleanup is in progress |
| `participant_left` | membership ended |
| `participant_rejected` | join or command rejected |

### 3.4 Command Transition Rules (all transitions table)

Pre-state tuple notation:

- `family={a, b}` means one state in the same family is required.
- `family=x; other_family=y` means both family states are required at the same time.
- an omitted family means the transition has no precondition for that family.

| Command / core event | Allowed pre-state | Success state | Reject reason code |
|---|---|---|---|
| `JoinRoom` | `room={room_absent, room_open}; participant=participant_new` | `participant_verifying` | `room_not_accepting_join`, `room_capacity_exceeded`, `admission_capacity_exceeded`, `room_lifetime_exceeded`, `token_verification_failed`, `room_draining`, `room_closed` |
| `AcceptJoinVerification` | `room={room_absent, room_open}; participant=participant_verifying` | `room_open`, `participant_joined` | `room_not_accepting_join`, `room_capacity_exceeded`, `admission_capacity_exceeded`, `room_lifetime_exceeded`, `room_draining`, `room_closed` |
| `RejectJoinVerification` | `participant=participant_verifying` | `participant_rejected` | `token_verification_failed`, `participant_rejected` |
| `LeaveRoom` | `room={room_open, room_draining}; participant=participant_joined` | `participant_leaving` | `participant_not_joined`, `room_closed` |
| `FinishLeave` | `room={room_open, room_draining}; participant=participant_leaving` | `participant_left` | `participant_not_joined`, `room_closed` |
| `BeginRoomDrain` | `room=room_open` | `room_draining` | `room_close_not_allowed` |
| `CloseRoom` | `room={room_open, room_draining, room_closed}` | `room_closed` | `room_close_not_allowed` |
| `ObserveClosedRoom` | `room=room_closed` | `room_closed` | `room_close_not_allowed` |
| `SendOffer` | `room=room_open; participant=participant_joined` | `room_open`, `participant_joined` | `participant_not_joined`, `command_order_violation`, `room_draining`, `room_closed` |
| `SendAnswer` | `room=room_open; participant=participant_joined` | `room_open`, `participant_joined` | `participant_not_joined`, `command_order_violation`, `room_draining`, `room_closed` |
| `SendIceCandidate` | `room=room_open; participant=participant_joined` | `room_open`, `participant_joined` | `participant_not_joined`, `command_order_violation`, `room_draining`, `room_closed` |
| `RequestTurnCredential` | `room=room_open; participant=participant_joined` | `room_open`, `participant_joined` | `participant_not_joined`, `token_verification_failed`, `room_draining`, `room_closed` |
| `AcknowledgeForward` | `room=room_open; participant=participant_joined` | `room_open`, `participant_joined` | `participant_not_joined`, `duplicate_command`, `room_draining`, `room_closed` |

### 3.5 Implicit transition rules (draining / closed coverage)

- All commands without an explicit `room=room_closed` allowed pre-state are rejected in `room_closed` with `room_closed`.
- Commands without an explicit `room=room_draining` allowed pre-state are rejected in `room_draining` with `room_draining`.
- New joins in `room_draining` are rejected with `room_draining`; controlled leave and close transitions remain allowed.
- `CloseRoom` and `ObserveClosedRoom` in `room_closed` are idempotent success, not rejection.

### 3.6 Idempotency Rule

- idempotency is core semantics.
- duplicate command handling MUST produce a closed result: accepted duplicate with same effect, or rejected duplicate.
- driver MUST NOT decide idempotency semantics.

### 3.7 Ordering Rule

- ordering rule is core semantics.
- driver MAY preserve or observe wire order, but command ordering validity belongs to core (MUST).

### 3.8 Correlation Rule

- all commands that cross the driver/core boundary MUST carry `CorrelationId`.
- missing correlation ID is fail-closed at driver conversion boundary with `missing_correlation_id`; the command does not enter the state machine (MUST).

### 3.9 collapse conditions (state machine)

- WebSocket connection state is treated as participant state.
- driver decides join / leave / relay state transition.
- duplicate command handling is driver-local.
- missing correlation ID is tolerated.
- SDK defines a different Signaling state machine.
- process shutdown is treated as Signaling state authority without core transition.
- concurrent Signaling command conflict is resolved outside core ordering/idempotency rule.
