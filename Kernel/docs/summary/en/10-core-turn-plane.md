# core-turn-plane

Status: public summary projection
Date: 2026-07-06 JST

## Purpose

This chapter internalizes the current complete specification of the TURN contract and TURN lifecycle owned by `core/turn` of arcRTC v0.2 Kernel, at a granularity sufficient for re-implementation from this chapter alone. This chapter makes TURN allocation / permission / channel bind / relay semantics core-owned and separates wire driver execution such as UDP/TCP/socket I/O, tokio, SIMD backend, and HMAC/crypto concrete implementation as driver-owned. This chapter internalizes all states and all transitions of allocation / permission / channel bind, guards, reject/deny conditions, expiry/refresh/release rules, credential verification boundary, and fail-closed conditions.

Dependency direction notation: `A <- B` means "B depends on A". The pure semantics of TURN are owned by core and separated from I/O. v0.2 Kernel does not own the TURN product system; the reference distro / product distro is placed in distro outside the Kernel. arcRTC does not own credential issuance and holds only the verification boundary of externally issued credential / token.

---

## 1. TURN Contract

### 1.1 Boundary (owner)

| Domain | Owner | Rule |
|---|---|---|
| TURN semantics | `core/turn` | STUN/TURN message semantic model, allocation/permission/channel bind/relay decision, credential verification boundary, fail-closed rule |
| UDP / TCP / socket I/O | `drivers/network` | concrete network I/O |
| executable contract / composition evidence surface | `entrypoints/turn-server` | server executable / configuration / listener setup / dependency wiring / shutdown signal handling |

The STUN/TURN wire decode/encode driver boundary follows the TURN wire driver rule. The TURN credential shared secret / generation / overlap / revocation lifecycle follows the secret rotation lifecycle rule. cross-plane identity / session binding follows the cross-plane binding rule.

### 1.2 Core Model (core-owned model, closed set)

core MUST own the following as model.

- transaction ID
- request / response / indication
- allocation
- permission
- channel binding
- channel binding reference
- peer address as core-owned address type
- relay decision
- credential verification outcome
- lifetime / expiry
- closed error reason

### 1.3 Core Decisions (core-owned decision, closed set)

core MUST own the following decisions.

- allocation accepted / rejected
- allocation released / expired
- refresh accepted / rejected / expired
- permission accepted / rejected / revoked
- permission expired
- channel bind accepted / rejected
- channel bind expired
- relay allowed / denied
- malformed message classification
- expired credential classification
- unauthorized request classification

### 1.4 Driver Responsibilities

driver MUST NOT bring the following into core. UDP socket, TCP listener/stream, tokio task, raw external byte buffer ownership, SIMD backend type, concrete HMAC library type, OS network interface type.

### 1.5 Fail-Closed Rule (decision handling and reason code closed set)

The following cases MUST be fail-closed per the handling column and connected to a cataloged reason code.

| Failure | Decision handling | Reason code |
|---|---|---|
| malformed STUN/TURN message | rejected | `malformed_turn_message` |
| invalid transaction ID | rejected | `malformed_turn_message` |
| missing credential proof | rejected for allocation/refresh/permission/channel bind commands; denied for relay data | `credential_missing` |
| invalid credential proof | rejected for allocation/refresh/permission/channel bind commands; denied for relay data | `credential_invalid` |
| expired credential | rejected for allocation/refresh/permission/channel bind commands; denied for relay data | `credential_expired` |
| credential generation not accepted by active rotation policy | rejected for allocation/refresh/permission/channel bind commands; denied for relay data | `secret_generation_not_accepted` |
| credential key or generation revoked | rejected for allocation/refresh/permission/channel bind commands; denied for relay data | `secret_key_revoked` |
| prior-generation overlap window expired | rejected for allocation/refresh/permission/channel bind commands; denied for relay data | `secret_overlap_window_expired` |
| rotation state source unavailable | rejected for allocation/refresh/permission/channel bind commands; denied for relay data | `secret_rotation_state_unavailable` |
| allocation capacity exceeded | rejected | `allocation_capacity_exceeded` |
| permission capacity exceeded | rejected | `permission_capacity_exceeded` |
| unauthorized peer | rejected | `peer_not_allowed` |
| permission revoked by peer policy | revoked | `peer_not_allowed` |
| missing or non-active permission where active permission is required | rejected for commands requiring active permission, including channel bind; denied for relay data | `permission_not_found` |
| relay denied | denied | `relay_denied` |
| allocation missing or non-active | rejected for refresh/permission/channel bind/release commands; denied for relay data | `allocation_not_found` |
| unsupported method | rejected | `unsupported_turn_method` |
| unsupported TURN contract version | rejected | `unsupported_turn_contract_version` |
| lifetime violation | rejected | `turn_lifetime_violation` |
| allocation lifetime cap exceeded | expired | `allocation_lifetime_exceeded` |
| permission lifetime expired | expired | `permission_lifetime_exceeded` |
| channel bind lifetime expired | expired | `channel_bind_lifetime_exceeded` |
| refresh count or cumulative refresh cap exceeded | expired | `refresh_limit_exceeded` |

TURN relay queue capacity or wait limit exceeded is not a `turn_relay_decision` denial. It is a resource-bound scheduling drop and MUST use `resource_bound_decision` with reason `turn_relay_queue_bound_exceeded`, active `AllocationId`, active `PermissionId`, resource policy owner `core`, and physical resource owner `driver`. `permission_lifetime_exceeded` and `channel_bind_lifetime_exceeded` are expiry reasons; permission or channel bind request rejection for invalid requested lifetime uses `turn_lifetime_violation`.

### 1.6 Auth Boundary

- arcRTC does not own credential issuance (MUST).
- arcRTC holds the verification boundary of externally issued credential / token.
- secret generation acceptance, overlap, revocation, and rotation state observation are verification inputs, not credential issuance.
- TURN core may decide against a credential generation, but MUST NOT fetch, mint, persist, or rotate raw secrets.
- TURN credential delivery / allocation / permission / channel bind success is not Signaling/SFU/secure media success without an admitted cross-plane binding and target-plane decision (MUST).

### 1.7 collapse conditions (TURN contract)

- socket loop holds the authority of TURN decision.
- core references UDP/TCP concrete type.
- driver owns permission rule.
- TURN credential issuance is treated as core responsibility.
- TURN credential rotation failure is collapsed into generic credential failure.
- raw secret / generation material is exposed in core model, audit, or evidence.
- Signaling participant or SFU endpoint relation is inferred from TURN credential, allocation, permission, or relay observation without cross-plane binding.

---

## 2. TURN Boundary (plane separation decision)

### 2.1 Decision

The core semantics of TURN are owned by `core/turn`. UDP / TCP / socket / tokio / SIMD backend / process lifecycle are owned by drivers or entrypoints (MUST).

### 2.2 Core TURN Responsibilities

`core/turn` MUST own the following. STUN/TURN message semantic model, request/response/indication/error semantics, allocation lifecycle, refresh semantics, permission lifecycle, channel bind semantics, relay decision, credential verification boundary, nonce/realm/timestamp policy as abstract rule, fail-closed rule for malformed/expired/unauthorized requests.

### 2.3 Driver / Entrypoint Responsibilities

TURN driver: UDP socket I/O, TCP framing I/O, packet read/write, runtime task scheduling, HMAC/crypto library concrete implementation, SIMD/CPU feature optimization, external byte buffer conversion, metrics export.

TURN entrypoint: server executable / product-system wiring detail, configuration loading, network listener setup, dependency wiring, shutdown signal handling.

### 2.4 Prohibited Placement / collapse conditions

Prohibitions: placing the authority of allocation decision in the socket read loop; placing the authority of permission rule in TCP framing; core depending on `tokio::net` / `socket2` / SIMD backend concrete type; entrypoint owning TURN protocol error classification; arcRTC owning TURN credential issuance (arcRTC holds only the verification boundary).

Collapse: core references socket/runtime/SIMD concrete implementation; driver owns allocation/permission/relay rule; entrypoint branches the protocol semantics implementation; TURN auth is mistaken for token issuance.

---

## 3. TURN Lifecycle

The authority of TURN lifecycle semantics is owned by `core/turn`.

### 3.1 Lifecycle Owners

| Lifecycle | Owner | Driver role |
|---|---|---|
| Allocation lifecycle | core | socket / packet I/O |
| Permission lifecycle | core | address conversion |
| Channel bind lifecycle | core | byte framing conversion |
| Credential verification outcome | core | crypto / token verifier implementation |
| Timer execution | driver | core-owned expiry rule execution |

Driver socket shutdown does not authorize silent restoration, silent release, or unaudited mutation of allocation/permission/channel bind lifecycle (MUST). cross-plane shutdown/drain, command/decision/event result shape, concurrency/ordering/lock ownership, retry/timeout/cancellation, and cross-plane identity/session binding follow their respective rule systems.

### 3.2 Allocation States (closed set)

| State | Meaning |
|---|---|
| `allocation_absent` | no allocation exists |
| `allocation_requested` | request is being evaluated |
| `allocation_active` | allocation can relay when permission allows |
| `allocation_refreshing` | refresh is being evaluated |
| `allocation_expired` | lifetime ended |
| `allocation_released` | allocation intentionally released |
| `allocation_rejected` | request rejected |

### 3.3 Permission States (closed set)

| State | Meaning |
|---|---|
| `permission_absent` | no permission for peer |
| `permission_requested` | permission request is being evaluated |
| `permission_active` | peer relay is allowed |
| `permission_expired` | permission lifetime ended |
| `permission_revoked` | permission is no longer allowed |
| `permission_rejected` | request rejected |

### 3.4 Channel Bind States (closed set)

| State | Meaning |
|---|---|
| `channel_unbound` | no channel binding exists |
| `channel_bind_requested` | bind request is being evaluated |
| `channel_bound` | channel can be used |
| `channel_expired` | binding lifetime ended |
| `channel_rejected` | bind request rejected |

### 3.5 Transition Rules (all transitions table)

Pre-state tuple notation:

- `family={a, b}` means one state in the same family is required.
- `family=x; other_family=y` means both family states are required at the same time.
- an omitted family means there is no precondition.
- `success-only` in failure columns means the row has no failure branch; invalid pre-states must be represented by separate rejection rows.

| Event | Allowed pre-state | Success state | Failure state | Decision reason code |
|---|---|---|---|---|
| `Allocate` | `allocation=allocation_absent` | `allocation_requested` | `allocation_rejected` | `credential_missing`, `credential_invalid`, `credential_expired`, `allocation_capacity_exceeded` |
| `AcceptAllocation` | `allocation=allocation_requested` | `allocation_active` | `allocation_rejected` | `credential_invalid`, `credential_expired`, `allocation_capacity_exceeded` |
| `RejectAllocation` | `allocation=allocation_requested` | `allocation_rejected` | `allocation_rejected` | `credential_missing`, `credential_invalid`, `credential_expired`, `allocation_capacity_exceeded` |
| `Refresh` | `allocation=allocation_active` | `allocation_refreshing` | `allocation_active` | `credential_missing`, `credential_invalid`, `credential_expired` |
| `RejectRefreshLifetimeViolation` | `allocation=allocation_active` | `allocation_active` | `allocation_active` | `turn_lifetime_violation` |
| `AcceptRefresh` | `allocation=allocation_refreshing` | `allocation_active` | `allocation_active` | `credential_missing`, `credential_invalid`, `credential_expired` |
| `RejectRefresh` | `allocation=allocation_refreshing` | `allocation_active` | `allocation_active` | `credential_missing`, `credential_invalid`, `credential_expired` |
| `RejectRefreshAbsent` | `allocation=allocation_absent` | `allocation_absent` | `allocation_absent` | `allocation_not_found` |
| `RejectRefreshRequested` | `allocation=allocation_requested` | `allocation_requested` | `allocation_requested` | `allocation_not_found` |
| `RejectRefreshExpired` | `allocation=allocation_expired` | `allocation_expired` | `allocation_expired` | `allocation_not_found` |
| `RejectRefreshReleased` | `allocation=allocation_released` | `allocation_released` | `allocation_released` | `allocation_not_found` |
| `RejectRefreshRejected` | `allocation=allocation_rejected` | `allocation_rejected` | `allocation_rejected` | `allocation_not_found` |
| `Release` | `allocation={allocation_active, allocation_refreshing}` | `allocation_released` | success-only | success-only |
| `RejectReleaseAbsent` | `allocation=allocation_absent` | `allocation_absent` | `allocation_absent` | `allocation_not_found` |
| `RejectReleaseRequested` | `allocation=allocation_requested` | `allocation_requested` | `allocation_requested` | `allocation_not_found` |
| `RejectReleaseExpired` | `allocation=allocation_expired` | `allocation_expired` | `allocation_expired` | `allocation_not_found` |
| `RejectReleaseReleased` | `allocation=allocation_released` | `allocation_released` | `allocation_released` | `allocation_not_found` |
| `RejectReleaseRejected` | `allocation=allocation_rejected` | `allocation_rejected` | `allocation_rejected` | `allocation_not_found` |
| `ExpireAllocationLifetime` | `allocation={allocation_active, allocation_refreshing}` | `allocation_expired` | `allocation_expired` | `allocation_lifetime_exceeded` |
| `ExpireRefreshCap` | `allocation={allocation_active, allocation_refreshing}` | `allocation_expired` | `allocation_expired` | `refresh_limit_exceeded` |
| `CreatePermission` | `allocation=allocation_active; permission=permission_absent` | `permission_requested` | `permission_rejected` | `credential_missing`, `credential_invalid`, `credential_expired`, `allocation_not_found`, `peer_not_allowed`, `permission_capacity_exceeded`, `turn_lifetime_violation` |
| `AcceptPermission` | `allocation=allocation_active; permission=permission_requested` | `permission_active` | `permission_rejected` | `credential_missing`, `credential_invalid`, `credential_expired`, `allocation_not_found`, `peer_not_allowed`, `permission_capacity_exceeded`, `turn_lifetime_violation` |
| `RejectPermission` | `permission=permission_requested` | `permission_rejected` | `permission_rejected` | `credential_missing`, `credential_invalid`, `credential_expired`, `allocation_not_found`, `peer_not_allowed`, `permission_capacity_exceeded`, `turn_lifetime_violation` |
| `ExpirePermission` | `permission=permission_active` | `permission_expired` | `permission_expired` | `permission_lifetime_exceeded` |
| `RevokePermission` | `permission=permission_active` | `permission_revoked` | `permission_revoked` | `peer_not_allowed` |
| `ChannelBind` | `allocation=allocation_active; permission=permission_active; channel=channel_unbound` | `channel_bind_requested` | `channel_rejected` | `credential_missing`, `credential_invalid`, `credential_expired`, `allocation_not_found`, `permission_not_found`, `relay_denied`, `turn_lifetime_violation` |
| `AcceptChannelBind` | `allocation=allocation_active; permission=permission_active; channel=channel_bind_requested` | `channel_bound` | `channel_rejected` | `credential_missing`, `credential_invalid`, `credential_expired`, `allocation_not_found`, `permission_not_found`, `relay_denied`, `turn_lifetime_violation` |
| `RejectChannelBind` | `channel=channel_bind_requested` | `channel_rejected` | `channel_rejected` | `credential_missing`, `credential_invalid`, `credential_expired`, `allocation_not_found`, `permission_not_found`, `relay_denied`, `turn_lifetime_violation` |
| `ExpireChannelBind` | `channel=channel_bound` | `channel_expired` | `channel_expired` | `channel_bind_lifetime_exceeded` |
| `RelayData` | `allocation=allocation_active; permission=permission_active` | `allocation_active`, `permission_active` | `allocation_active`, `permission_active` | `credential_missing`, `credential_invalid`, `credential_expired`, `relay_denied` |
| `DenyRelayDataAllocationAbsent` | `allocation=allocation_absent` | `allocation_absent` | `allocation_absent` | `allocation_not_found` |
| `DenyRelayDataAllocationRequested` | `allocation=allocation_requested` | `allocation_requested` | `allocation_requested` | `allocation_not_found` |
| `DenyRelayDataAllocationExpired` | `allocation=allocation_expired` | `allocation_expired` | `allocation_expired` | `allocation_not_found` |
| `DenyRelayDataAllocationReleased` | `allocation=allocation_released` | `allocation_released` | `allocation_released` | `allocation_not_found` |
| `DenyRelayDataAllocationRejected` | `allocation=allocation_rejected` | `allocation_rejected` | `allocation_rejected` | `allocation_not_found` |
| `DenyRelayDataPermissionAbsent` | `allocation=allocation_active; permission=permission_absent` | `allocation_active`, `permission_absent` | `allocation_active`, `permission_absent` | `permission_not_found` |
| `DenyRelayDataPermissionRequested` | `allocation=allocation_active; permission=permission_requested` | `allocation_active`, `permission_requested` | `allocation_active`, `permission_requested` | `permission_not_found` |
| `DenyRelayDataPermissionExpired` | `allocation=allocation_active; permission=permission_expired` | `allocation_active`, `permission_expired` | `allocation_active`, `permission_expired` | `permission_not_found` |
| `DenyRelayDataPermissionRevoked` | `allocation=allocation_active; permission=permission_revoked` | `allocation_active`, `permission_revoked` | `allocation_active`, `permission_revoked` | `permission_not_found` |
| `DenyRelayDataPermissionRejected` | `allocation=allocation_active; permission=permission_rejected` | `allocation_active`, `permission_rejected` | `allocation_active`, `permission_rejected` | `permission_not_found` |

### 3.6 Expiry Rule

- expiry decision is core semantics; timer execution is driver implementation.
- driver MUST provide core-owned time observations through the port boundary.
- allocation lifetime, permission lifetime, channel bind lifetime, refresh count, and cumulative refresh duration MUST have bounds.
- refresh MUST NOT extend the absolute lifetime cap.
- `permission_lifetime_exceeded` and `channel_bind_lifetime_exceeded` are expiry outcomes, not permission/channel bind rejection outcomes.
- Invalid requested lifetime before activation uses `turn_lifetime_violation`.
- Refresh lifetime violation before entering `allocation_refreshing` preserves `allocation_active` and MUST NOT use `expired` outcome.
- Refresh credential rejection preserves `allocation_active`.
- `allocation_lifetime_exceeded` is an allocation expiry outcome and uses `ExpireAllocationLifetime`.
- `refresh_limit_exceeded` is a refresh cap expiry outcome and uses `ExpireRefreshCap`.

### 3.7 Refresh Audit Rule

- TURN refresh decisions use `turn_refresh_decision`, not initial `turn_allocation_decision`.
- `Refresh` success only enters `allocation_refreshing`; it is a pending transition and MUST NOT emit final `turn_refresh_decision` outcome `accepted`.
- `AcceptRefresh` emits final `turn_refresh_decision` with outcome `accepted`.
- `Refresh` immediate failure, `RejectRefresh`, `RejectRefreshLifetimeViolation`, and non-active refresh rejection rows emit `turn_refresh_decision` with outcome `rejected`.
- Refresh cap expiry uses `turn_refresh_decision` with outcome `expired` and reason `refresh_limit_exceeded`.
- Allocation absolute lifetime expiry uses `turn_allocation_decision` with outcome `expired` and reason `allocation_lifetime_exceeded`.

### 3.8 Relay Queue Bound Rule

- `turn_relay_decision` owns relay authorization only.
- TURN relay queue capacity or wait limit failure is a scheduling resource-bound drop, not a relay authorization denial.
- `turn_relay_queue_bound_exceeded` MUST be audited as `resource_bound_decision` with outcome `dropped`, resource policy owner `core`, physical resource owner `driver`, active `AllocationId`, and active `PermissionId`.
- RelayData against a non-active allocation or permission emits `turn_relay_decision` with outcome `denied`, preserves the current non-active state, and uses `allocation_not_found` or `permission_not_found`.
- RelayData MUST NOT use allocation or permission expiry reasons; time-based expiry uses the dedicated expiry transitions.

### 3.9 Release Rule

- `Release` from `allocation_active` or `allocation_refreshing` is a reasonless success and uses audit outcome `released`.
- A release request without an active or refreshing allocation is rejected with `allocation_not_found`, preserves the current non-active allocation state, and MUST NOT use audit outcome `released`.

### 3.10 Allocation Precondition Rule

- A TURN command requiring `allocation_active` or `allocation_refreshing` MUST reject absent, requested, expired, released, or rejected allocation state with `allocation_not_found`.
- A TURN pending accept transition MUST revalidate parent allocation/permission state before moving to active or bound state.
- A TURN command requiring `permission_active` MUST reject absent, requested, expired, revoked, or rejected permission state with `permission_not_found`.
- A cross-plane relation to Signaling participant or SFU endpoint MUST NOT override TURN lifecycle preconditions.

### 3.11 Credential Rule

- arcRTC verifies externally issued credential.
- arcRTC does not issue credential (MUST).

### 3.12 collapse conditions (TURN lifecycle)

- socket loop owns allocation state.
- driver owns permission decision.
- timer implementation defines expiry semantics.
- TURN credential issuance becomes core responsibility.
- channel bind state is treated as TCP framing state.
- driver shutdown is treated as successful TURN lifecycle transition without core/audit relation.
- concurrent TURN allocation / permission conflict is resolved by driver socket order.
- cross-plane relation silently bypasses TURN allocation, permission, or channel bind lifecycle precondition.
