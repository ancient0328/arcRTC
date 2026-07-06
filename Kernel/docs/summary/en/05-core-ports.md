# core-ports

Status: public summary projection
Date: 2026-07-06 JST

## Purpose

This chapter is the authority on the ports owned by arcRTC v0.2 Kernel core. It internalizes, at a re-implementable level of detail, the port families that core owns (clock / random / token verifier / network / WebRTC transport / packet view / persistence / audit sink / metrics sink / runtime) and their responsibilities, the standard shape of a port contract (method shape, input/output types, error type, ownership rules, call shape), and the port ownership policy.

Dependency notation `A <- B` reads as "B depends on A". The layering is `core <- drivers <- entrypoints`, and a port is a core-owned interface, not a driver-owned interface.

## 1. Port Ownership Policy

arcRTC v0.2 is grounded in DDD / hexagonal architecture. In this structure, merely placing the contact point with external technology on the driver side is insufficient. Unless port ownership is fixed to core, there is a risk that a driver owns domain rule and entrypoints hold judgment beyond the composition root.

Therefore, in v0.2 core owns every primary / secondary port.

```text
core owns port
drivers implement port
entrypoints compose port implementations
```

- core owns the port type, input, output, error vocabulary, and contract.
- drivers implement the core-owned port.
- entrypoints wire the port implementation.

## 2. Port Families Owned by core

The ports core owns are limited to the following.

| Port | Core-owned contract (responsibility) | Driver implementation examples |
|---|---|---|
| ClockPort | now, deadline comparison, monotonic time (time retrieval, deadline judgment) | system clock, test clock |
| RandomPort | nonce, opaque ID, challenge entropy | OS RNG, deterministic test RNG |
| TokenVerifierPort | external token verification result (verification of externally-issued token) | JWT RS256 verifier |
| NetworkPort | abstract datagram/stream send/receive (UDP/TCP/HTTP/WebSocket boundary) | tokio UDP/TCP, WebSocket, HTTP |
| WebRtcTransportPort | transport event / command exchange (transport boundary such as str0m) | str0m |
| PacketViewPort | borrowed packet abstract view / routing input (constructs a core-owned borrowed packet view from a driver-owned RTP/RTCP buffer lease) | driver-owned RTP / RTCP buffer lease |
| PersistencePort | state / audit persistence boundary (room state, audit, checkpoint) | PostgreSQL, Redis, filesystem, memory |
| AuditSinkPort | audit event export (external output of audit events) | file, HTTP, syslog, S3 |
| MetricsSinkPort | metrics export (metrics / telemetry output) | tracing, Prometheus-style exporter |
| RuntimePort | timer / spawn / cancellation abstraction (runtime-independent scheduling boundary) | tokio runtime driver |

Test double / fake driver implementations of these ports follow the test-double boundary authority.

## 3. Port Contract Rule (common)

Every core-owned port MUST have:

- a core-owned input type
- a core-owned output type
- a closed error classification
- fail-closed behavior
- a correlation ID propagation rule
- no external concrete type in the signature
- no ownership transfer of driver packet bytes
- no transfer of a concrete runtime task / join handle into core semantic state

## 4. Port Shape Field Classes (general form)

Every core-owned port makes the following field classes explicit.

| Field class | Rule |
|---|---|
| input type | core-owned type only |
| output type | core-owned type only |
| error type | closed reason category/code, or a port-specific closed wrapper with catalog connection |
| correlation | required when the command/request is user or protocol correlated |
| ownership | no external concrete type, no driver buffer ownership transfer |
| call shape | one of command, query, sink-submit, stream-observation, or scheduler |
| backpressure / bound | connects to the resource bounds authority when bounded |
| retry / timeout / cancellation | connects to the retry/timeout/cancellation authority when the operation may be retried, timed out, or cancelled |

A port contract MUST NOT depend on async runtime, socket, DB, cloud SDK, browser/native SDK, or str0m concrete types.

## 5. Required Shape per Port

| Port | Call shape | Input | Output | Error / reason |
|---|---|---|---|---|
| ClockPort | query | clock request / monotonic comparison input | core time observation | `runtime_config_invalid` only when driver cannot initialize; domain expiry reason remains core-owned |
| RandomPort | query | entropy request purpose | opaque entropy bytes / generated reference material | `runtime_config_invalid`, `driver_shutdown` |
| TokenVerifierPort | command/query | credential/token verification request | verification result | token reason codes or `token_key_unavailable` |
| NetworkPort | command / stream-observation | core-owned outbound envelope or network observation request | delivery observation / converted inbound observation | `network_send_failed`, `network_receive_failed`, `driver_shutdown` |
| WebRtcTransportPort | command / stream-observation | transport command | transport event / command result | transport conversion or driver failure reason |
| PacketViewPort | query / borrowed view | driver-provided borrowed packet semantic source | packet semantic view | `external_decode_failed`, `frame_size_bound_exceeded`, `buffer_release_failed` |
| PersistencePort | command / query | checkpoint, audit, retry, or lookup intent | persistence acknowledgement / loaded core-owned state | `persistence_unavailable`, retry bound reasons |
| AuditSinkPort | sink-submit | audit event / hash-chain record intent | sink acknowledgement | audit backlog / persistence / driver failure reason |
| MetricsSinkPort | sink-submit | metric export intent | export acknowledgement | `metrics_export_failed`, `metrics_backlog_bound_exceeded` |
| RuntimePort | scheduler | timer / spawn / cancellation request | scheduled handle reference / cancellation observation | `runtime_config_invalid`, `driver_shutdown`, relevant resource bound reason |

## 6. Port-Specific Narrowing Rules

### 6.1 NetworkPort Narrowing Rule

NetworkPort is not permission for raw external wire objects to enter core. Inbound external bytes MUST be decoded by the driver into the semantic envelope (its structure is internalized in chapter 06) before any domain use case entry. If NetworkPort is used for receiving observation, its output MUST be a core-owned observation type. It MUST NOT return an HTTP request, WebSocket frame, UDP socket address object, or STUN/TURN parser object.

### 6.2 PacketViewPort Ownership Rule

PacketViewPort MAY expose a borrowed semantic header view only. It MUST NOT expose a driver `BufferLease`, packet cache, transmit queue, parser crate object, or raw bytes ownership to core. The view lifetime MUST be no longer than the driver-owned lease defined by the SFU packet buffer lifecycle authority.

### 6.3 PersistencePort Shape Rule

PersistencePort MUST separate: state checkpoint intent; audit persistence intent; hash-chain record persistence intent; retry store intent. Each intent MUST declare whether it is source-of-truth, checkpoint, audit-only, or driver retry data, per the state persistence policy authority.

### 6.4 RuntimePort Shape Rule

RuntimePort MUST NOT expose runtime task handles as domain state. core MAY receive only opaque schedule/cancellation references and execution observations. Runtime task class, supervision scope, join/cancel bound, and panic outcome follow the runtime task / worker lifecycle authority. Timer execution delay does not redefine core expiry semantics.

## 7. Error Shape Rule

Port errors MUST be closed. A driver-local error MAY be carried as non-authoritative detail, but decision, audit, and SDK mapping MUST use the cataloged category/code. External exposure of port errors follows the external error mapping authority (internalized in chapter 04).

## 8. Forbidden Signatures (prohibited imported types)

A core-owned port signature MUST NOT contain:

- `tokio::net::*`
- `axum::*`
- `sqlx::*`
- `aws_sdk_s3::*`
- `reqwest::*`
- `str0m::*`
- driver `BufferLease`
- driver `PacketCache`
- driver `TxQueue`
- concrete runtime task handle / join handle
- browser-native Web API types
- Android / iOS platform SDK types

External types are converted to core-owned types at the driver boundary. As prohibited examples, core MUST NOT receive an `axum` request/response, `tokio::net::UdpSocket`, `str0m` concrete event, driver `BufferLease`/packet cache/transmit queue, `sqlx` row/pool, or browser `WebSocket`/native SDK type.

## 9. Port Naming Rule

A port name expresses a role. A technology name MUST NOT appear in a port name.

- Allowed examples: `WebRtcTransportPort`, `AuditSinkPort`, `TokenVerifierPort`
- Prohibited examples: `Str0mPort`, `PostgresAuditPort`, `S3BackupPort`, `AxumSignalingPort`

## 10. Prohibited Ownership / Prohibitions

- drivers define a port trait.
- entrypoints define a port trait.
- drivers own Signaling / SFU / TURN accept / reject rules.
- entrypoints own a domain decision.
- core imports a driver concrete type.
- core depends on framework / runtime / OS / browser / DB / cloud SDK.
- port input or output includes an external concrete type.
- a port returns an open-ended string as the sole error.
- NetworkPort bypasses driver conversion.
- PacketViewPort transfers byte ownership to core.
- RuntimePort exposes a concrete task handle as domain state.
- RuntimePort omits task class or supervision scope where task execution affects the claim.
- PersistencePort uses schema/table/key layout as the core API.
- entrypoints define port variants per binary.
- a core-owned port receives ownership of packet bytes, buffer lease, packet cache, or transmit queue.

## 11. Consequences

- The equivalent of `core/src/transport/str0m_transport*` is not left directly in v0.2 core.
- PostgreSQL / S3 / HTTP / file / syslog sinks are separated to the driver side.
- JWT verification library dependencies are placed in the driver, and core owns the verification result and decision rule.
- entrypoints are limited to the Kernel executable contract / CLI / demo / dependency wiring.

## 12. Fail-Closed / Invariants

- Every port has fail-closed behavior.
- Port errors are closed (connected to the reason catalog). An open-ended string alone is not allowed.
- A core-owned port does not receive ownership of driver packet bytes, buffer lease, packet cache, or transmit queue.
- RuntimePort does not pass a concrete task handle into core as domain state.
- The port signature contains no external concrete type.
- For commands/requests that require correlation, correlation ID propagation is required.

## 13. Collapse Conditions

- a driver defines a port.
- a port signature contains an external concrete type.
- a core-owned port receives ownership of packet bytes, buffer lease, packet cache, or transmit queue.
- RuntimePort passes a concrete task handle into core as domain state.
- entrypoints define a port contract per branch.
- a port error becomes an open-ended string only.
- a port lacks a closed error mapping.
- a port call shape allows an unbounded queue or hidden retry.
- a port's retry/timeout/cancellation behavior is implicit.
- runtime task lifecycle evidence is implicit where port behavior depends on worker execution.
- an inbound wire object enters core through NetworkPort.
- a PersistencePort state class is not declared.
- entrypoints hold protocol semantics or accept / reject rules.
- a port is moved to the driver side for implementation convenience.
