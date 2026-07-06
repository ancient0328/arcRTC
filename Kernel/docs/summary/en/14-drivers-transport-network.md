# drivers-transport-network

Status: public summary projection
Date: 2026-07-06 JST

## Purpose

This chapter internalizes the current complete specification of the transport / network driver family of arcRTC v0.2 Kernel (`drivers/network`, `drivers/webrtc-str0m`, `drivers/browser`, `drivers/native`, and the TURN/STUN wire driver), at a granularity sufficient for re-implementation from this chapter alone. This chapter internalizes driver conversion (the conversion rules between external types and core-owned types), the network I/O boundary (tokio UDP/TCP/HTTP/WebSocket/socket), the transport driver (the WebRTC transport port implementation including str0m), the browser/native driver boundary, and the TURN wire driver (the STUN/TURN decode/encode boundary), down to owners, rules, failure mapping, closed-set vocabulary, prohibitions/permissions, and fail-closed conditions, omitting none.

Dependency direction notation: `A <- B` means "B depends on A". A driver implements a core-owned port, and external types are converted into core-owned types at the driver boundary. `drivers -> entrypoints`, `drivers -> regulated`, and `driver -> driver` (direct dependency) are prohibited. A driver performs only syntax / transport / framework conversion and does not perform a domain decision. Cross-driver composition is performed by entrypoints through core-owned ports.

---

## 1. Driver Conversion (conversion rules between external and core-owned types)

### 1.1 Conversion Direction (MUST)

Driver conversion is syntax / transport / framework conversion and is not a domain decision. The conversion direction is fixed as follows.

```text
external input
  -> driver decode / validate syntax
  -> core-owned command / event / observation / packet view
  -> core decision
  -> driver encode / execute
```

External error projection (the external projection after conversion failure) follows the failure mapping in Section 1.5 of this chapter.

### 1.2 Driver Required Validation (mandatory checks before calling core, closed set)

The driver MUST check the following before producing a core-owned type. If it cannot be checked, the driver treats it as a closed conversion failure without calling core (fail-closed).

- external payload is decodable
- frame size is within driver bound
- required wire field exists before mapping
- byte buffer shape is parseable
- external enum value maps to core enum
- transport connection is readable / writable

### 1.3 Driver Must Not Decide (what the driver MUST NOT decide, closed set, prohibition)

The driver MUST NOT decide the following.

- Signaling join accepted / rejected
- TURN allocation accepted / rejected
- TURN refresh accepted / rejected / expired
- TURN permission accepted / rejected / revoked
- TURN channel bind accepted / rejected / expired
- SFU route selected / suppressed
- quality violation semantics
- backpressure policy
- protocol / contract version accept-reject semantics
- TURN method support semantics
- regulated domain meaning
- SDK public contract semantics

### 1.4 Semantic Delegation Rule

The driver MAY validate only external encoding and syntactic shape before core entry. The driver MUST preserve the protocol / contract version fields and method identifiers required by core. Unsupported command version, unsupported TURN method, unsupported TURN contract version, and unsupported SFU/media contract version are core decisions and MUST NOT be treated as driver conversion failures.

### 1.5 Conversion Failure Rule (conversion failure -> closed reason, closed set)

A conversion failure MUST map to a closed reason. The `Reason` column MUST name a cataloged reason code, and category-only reason values are prohibited at the driver/core boundary.

| Failure | Reason |
|---|---|
| external payload cannot decode | `external_decode_failed` |
| frame size exceeds driver bound | `frame_size_bound_exceeded` |
| unsupported driver wire version | `unsupported_driver_wire_version` |
| correlation ID field missing | `missing_correlation_id` |
| required wire field missing | `missing_required_wire_field` |
| byte buffer shape is not parseable | `external_decode_failed` |
| external enum has no core mapping | `external_enum_unmapped` |
| transport connection is not readable | `network_receive_failed` |
| transport connection is not writable | `network_send_failed` |
| external type would leak into core | `external_type_leak_blocked` |
| core event cannot encode | `external_encode_failed` |

### 1.6 Conversion Failure Audit Rule

- `frame_size_bound_exceeded` is a driver-local resource-bound drop and emits `driver_resource_bound_decision` with outcome `dropped`.
- Other conversion failures in the table above emit `driver_error_converted` with outcome `converted_failure` and the listed cataloged reason.
- `missing_correlation_id` is audited as `driver_error_converted` with outcome `converted_failure`; the command does not enter the core state machine.
- When a client-supplied `CorrelationId` cannot be decoded or validated before failure, audit reference presence follows the audit event pre-materialization rule.

### 1.7 External Type Prohibition (external types the core boundary MUST NOT expose, closed set, prohibition)

The core boundary MUST NOT expose the following.

- framework request / response
- WebSocket frame type
- tokio socket type
- str0m event type
- sqlx row / pool
- AWS SDK type
- browser API type
- Android / iOS platform type
- driver buffer handle

### 1.8 Lossless Mapping Rule

Driver conversion MUST preserve information required by core semantics. Information required only for external execution remains driver-local. If conversion cannot construct a core-owned input because external syntax or encoding is invalid, the driver returns a conversion failure. If conversion reaches a protocol / contract / policy question, the driver preserves the undecided field and delegates the decision to core.

### 1.9 Driver Conversion Collapse Conditions

The judgment of this rule collapses if any of the following hold. Driver conversion performs a domain decision. An external concrete type crosses into core. A conversion failure is treated as success. The driver adds semantic fields not defined by core. The driver drops core-required information silently. The driver rejects a protocol / contract version or TURN method using driver-local semantics. A conversion failure is exposed externally without preserving cataloged reason traceability.

---

## 2. Network I/O Boundary (`drivers/network`)

### 2.1 Purpose and Ownership Principle

The network driver owns external network I/O and wire framing but does not own the domain semantics of Signaling / SFU / TURN. The conversion boundary between external wire envelope and semantic envelope follows the wire protocol envelope rule; external error mapping follows the external error mapping rule; public endpoint admission and connection lifecycle follow the public endpoint connection lifecycle rule; edge/proxy trusted metadata follows the edge/proxy trust boundary rule; service discovery / endpoint resolution follows the service discovery endpoint resolution rule.

### 2.2 Ownership (ownership assignment, closed set)

| Responsibility | Owner | Rule |
|---|---|---|
| UDP / TCP socket | driver | concrete listener, send, receive |
| HTTP / WebSocket endpoint | driver | external protocol binding and frame boundary |
| frame size / connection concurrency local bound | driver, unless core policy bound is listed | connected to the resource bounds / backpressure rule |
| external wire decode / encode | driver | converts to core-owned type |
| core command / event semantics | core | driver does not change |
| process entrypoint | entrypoints | listener selection and wiring only |
| network target resolution | entrypoints/drivers | concrete endpoint lookup, not semantic authority |

### 2.3 Inbound Flow (fixed order, MUST)

The network inbound flow is limited to the following order.

1. driver receives external bytes / frame / request.
2. driver enforces driver-local shape and frame bounds.
3. driver converts to core-owned command or packet view.
4. core evaluates domain / protocol semantics.
5. driver maps core response / command to external output.

The driver MUST NOT call domain aggregate internals directly. The driver enters core through the application use case or the core-owned port boundary.

### 2.4 Failure Mapping (closed set)

| Failure | Required reason |
|---|---|
| external payload cannot decode to core type | `external_decode_failed` |
| external wire version unsupported | `unsupported_driver_wire_version` |
| required wire field absent | `missing_required_wire_field` |
| external enum has no mapping | `external_enum_unmapped` |
| inbound frame size bound exceeded | `frame_size_bound_exceeded` |
| connection concurrency bound exceeded | `connection_concurrency_exceeded` |
| concrete receive failed | `network_receive_failed` |
| concrete send failed | `network_send_failed` |
| driver is shutting down | `driver_shutdown` |

Pre-core conversion failure follows the driver conversion rule in Section 1; resource-bound failure follows the resource bounds / backpressure rule; endpoint-class / lifecycle failure follows the public endpoint connection lifecycle rule; proxy/header/source-address trust failure follows the edge/proxy trust boundary rule; service discovery / endpoint resolution failure follows the service discovery endpoint resolution rule.

### 2.5 Encoding Rule

External JSON, HTTP status, WebSocket close code, binary frame, UDP datagram layout, and TCP framing are driver-owned. Their mapping MUST preserve the core reason category/code and correlation references where present. The driver MAY choose an external representation but MUST NOT invent a new semantic reason or hide a cataloged rejection behind generic success.

### 2.6 Direct Dependency Rule

`drivers/network` MUST NOT directly depend on `drivers/persistence`, `drivers/observability`, `drivers/webrtc-str0m`, `entrypoints/*`, or `regulated`. Cross-driver composition is performed by entrypoints through core-owned ports.

### 2.7 Prohibitions (prohibition, closed set)

- The network driver accepts / rejects join, route, allocation, permission, or relay by its own policy.
- The network driver turns decode failure into domain rejection after core entry.
- A WebSocket close code becomes the authoritative core reason.
- An HTTP status becomes the authoritative core reason.
- The driver sends a success response when core returned a rejected / denied / failed decision.
- Drivers depend on each other directly to bypass the entrypoints composition root.
- An external response loses cataloged reason traceability.
- A network listener bind or WebSocket upgrade is treated as public endpoint admission.
- The network driver trusts forwarded headers or proxy source metadata without an edge trust policy.
- The network driver treats endpoint resolution as domain readiness or internal control success.

### 2.8 Network I/O Collapse Conditions

The network I/O layer owns protocol semantics. External wire format leaks into core. The core reason is replaced by driver-local status text. A resource bound event is not audited through the required canonical mapping. The network driver directly composes persistence / observability implementation. The public/internal endpoint class is inferred from route naming alone. A source address, host, origin, or forwarded header becomes core identity. A resolved network endpoint becomes a semantic owner, authorization proof, or readiness proof.

---

## 3. Transport Driver (the WebRTC transport port implementation including str0m)

### 3.1 Purpose and Principle

The transport driver only implements a core-owned port and does not own protocol semantics. The platform-specific boundary of the browser / native driver follows Section 5.

### 3.2 Driver Set (closed set)

The transport drivers are limited to the following.

| Driver | Target |
|---|---|
| `drivers/webrtc-str0m` | str0m concrete implementation |
| `drivers/network` | UDP / TCP / HTTP / WebSocket |
| `drivers/browser` | browser-facing boundary |
| `drivers/native` | native platform boundary |

`drivers/observability`, `drivers/persistence`, audit sink, and metrics sink are not transport drivers. They are a driver family that implements core-owned ports but are out of scope of this section.

### 3.3 Conversion Rule

The driver converts external types into core-owned types before calling core. It converts the command / decision / event returned from core back into external types. When handling RTP / RTCP packet bytes, the driver owns the raw bytes and buffer lifecycle and passes only a borrowed abstract view to core.

### 3.4 Driver May Own (what the driver MAY own, closed set, permission)

The driver MAY own the following.

- external library initialization
- network listener
- socket read/write
- byte buffer codec
- buffer pool
- buffer lease
- bounded packet cache
- transmit queue
- external error mapping
- retry transport detail
- serialization format
- TLS / platform-specific transport setting

The "may own" in this section means ownership of physical implementation / execution detail. The buffer pool, packet cache, transmit queue, and retry transport do not permit an unbounded resource or driver-owned policy. Bounded resource policy, closed action, and audit owner tuple follow the resource bounds / backpressure rule.

### 3.5 Driver Must Not Own (what the driver MUST NOT own, closed set, prohibition)

The driver MUST NOT own the following.

- Signaling accept / reject rule
- TURN allocation / refresh / permission / channel bind / relay rule
- SFU routing / quality / backpressure rule
- SDK public contract semantics
- regulated domain semantics
- audit event meaning

### 3.6 str0m Rule

str0m is a driver implementation. The str0m event, state, error, and SDP/ICE concrete types are not exposed as core API. Only the WebRTC transport port and core-owned event / command types are placed in core.

### 3.7 Transport Driver Collapse Conditions

The driver reimplements a core-owned rule. An external concrete type crosses the core boundary. Driver-owned packet bytes, buffer lease, packet cache, or transmit queue move to core ownership. Drivers depend on each other directly and bypass the entrypoints composition root. A driver-local error is not converted into core error classification. The observability / persistence driver is treated as a transport driver.

---

## 4. TURN Wire Driver (the STUN/TURN decode/encode boundary)

### 4.1 Purpose and Principle

The TURN / STUN wire decode/encode driver does not own core TURN semantics. The TURN contract rule and the TURN lifecycle rule own core TURN semantics, and this section fixes external bytes, method/code mapping, attribute mapping, and socket failure mapping as driver-owned detail.

### 4.2 Boundary (ownership assignment, closed set)

| Surface | Owner | Rule |
|---|---|---|
| raw STUN/TURN bytes | driver | socket buffer and parser input |
| STUN/TURN parser / encoder | driver | concrete library/detail |
| method / attribute wire code | driver maps | core sees semantic command/result |
| TURN allocation / permission / relay semantics | core | lifecycle and decision owner |
| credential verification outcome | core semantics, driver crypto/key implementation | no credential issuance |
| UDP/TCP socket | driver | I/O execution only |

### 4.3 Inbound Mapping (wire class -> core semantic target, closed set, MUST)

The driver maps an inbound wire message into a core-owned TURN command before core entry. Raw attribute objects, parser errors, socket addresses, and byte buffers MUST NOT become core state.

| Wire class | Core semantic target |
|---|---|
| Allocate request | `Allocate` command |
| Refresh request | `Refresh` command |
| CreatePermission request | `CreatePermission` command |
| ChannelBind request | `ChannelBind` command |
| Send/Data indication | `RelayData` intent |
| unsupported method | rejection before or at core boundary with `unsupported_turn_method` |
| malformed message | pre-core rejection with `malformed_turn_message` |

### 4.4 Outbound Mapping (core outcome -> driver mapping, closed set)

A core decision is mapped to an external TURN response / indication without changing the semantic outcome. The wire error code is not the authoritative reason. The authoritative reason remains the cataloged core reason and the audit event.

| Core outcome | Driver mapping rule |
|---|---|
| allocation accepted | success response with mapped allocation fields |
| refresh accepted | success response with mapped lifetime fields |
| permission/channel bind accepted | success response |
| relay allowed | outbound data forwarding |
| rejected / denied / expired / revoked | error response or drop/deny behavior preserving cataloged reason in audit |

### 4.5 Attribute Rule

The driver owns binary attribute decode/encode. Core owns semantic validation after conversion. Examples: the transaction ID maps to a core-owned transaction reference; the peer address maps to a core-owned address type; the credential proof maps to a verification request/result; the lifetime field maps to a core-owned requested lifetime; the channel number maps to a core-owned channel bind reference. A missing required attribute maps to `malformed_turn_message` unless a more specific credential reason applies.

### 4.6 Socket Failure Rule (closed set)

A socket failure MUST NOT be reported as a TURN authorization denial unless core actually denied relay semantics.

| Failure | Required reason |
|---|---|
| receive failure | `network_receive_failed` |
| send failure | `network_send_failed` |
| frame size bound exceeded | `frame_size_bound_exceeded` |
| relay queue bound exceeded | `turn_relay_queue_bound_exceeded` |
| driver shutdown | `driver_shutdown` |

### 4.7 Prohibitions (prohibition, closed set)

- The driver parser owns allocation/permission/channel state.
- The socket loop decides relay authorization.
- A wire error code replaces the core catalog reason.
- A raw STUN/TURN attribute object crosses into core.
- A malformed message enters the TURN lifecycle state machine.
- The relay queue bound is emitted as a `turn_relay_decision` denial.
- The driver issues TURN credentials.

### 4.8 TURN Wire Driver Collapse Conditions

Wire parser state becomes core TURN state. A socket failure is confused with a core relay denial. An unsupported method or malformed message lacks a cataloged reason. A TURN wire attribute shape is exposed as core API. Driver credential issuance is introduced.

---

## 5. Browser / Native Driver Boundary (`drivers/browser`, `drivers/native`)

### 5.1 Purpose and Principle

The browser / native driver bridges platform API and core-owned port but does not own the SDK public contract, domain rule, regulated workflow, or media UI. Admission / exclusion of out-of-scope features follows the out-of-scope feature admission rule. The browser / native driver MAY implement core-owned ports but MUST NOT define a port, MUST NOT expose platform concrete types through core, and MUST NOT reinterpret Signaling / SFU / TURN semantics.

### 5.2 Boundary (ownership assignment, closed set)

| Surface | Owner | Rule |
|---|---|---|
| browser Web API / native platform API | driver | concrete platform interaction only |
| core port contract | core | type, input, output, error vocabulary, ownership rule |
| SDK public API | sdk | Signaling-only public client contract |
| entrypoint lifecycle wiring | entrypoints | selected driver wiring and process/entrypoint startup |
| regulated workflow | regulated | optional support only, not driver-owned |

### 5.3 Initial Scope (v0.2 initial scope)

The v0.2 initial browser / native driver scope is limited to the platform binding needed by approved ports.

Allowed initial responsibilities (permission):

- platform WebSocket / HTTP client binding when used as network implementation
- platform timer / clock / cancellation execution when used as RuntimePort / ClockPort implementation
- platform randomness source when used as RandomPort implementation
- platform logging / metrics sink when used as observability implementation
- platform storage only when selected as PersistencePort implementation
- platform-specific conversion between external API errors and cataloged core reasons

Out of scope (prohibition, closed set):

- camera / microphone capture
- media track rendering
- PeerConnection public abstraction in SDK
- screen sharing
- recording
- chat
- DataChannel application semantics
- UI / end-user workflow
- push notification workflow
- regulated workflow ownership
- user account or auth issuance

### 5.4 Type Conversion Rule

Browser / native concrete types stop at the driver boundary. Types forbidden in core signatures (closed set, prohibition):

- browser `WebSocket`, `MessageEvent`, `Blob`, `ArrayBuffer`, `ReadableStream`
- DOM event types
- Android SDK / Kotlin platform types
- iOS Foundation / AVFoundation / Network framework concrete types
- platform permission result objects
- platform lifecycle callback objects

The driver converts platform input into a core-owned command / event / reference / port result before invoking core. The driver converts core decisions back into platform output without changing the semantic category/code.

### 5.5 Lifecycle Rule (lifecycle observation rule, closed set)

Platform lifecycle events are driver observations unless mapped into a core-owned event by the core contract. A driver lifecycle observation MUST NOT close a room, drop a route, revoke a permission, or reject a command by platform-local policy. The core state transition remains core-owned.

| Platform observation | Default owner | Core entry condition |
|---|---|---|
| browser tab/page visibility | driver | only if mapped to approved command/event |
| mobile entrypoint foreground/background | driver | only if mapped to approved command/event |
| network availability change | driver | only if mapped to network/transport port event |
| platform cancellation | driver/runtime | only through RuntimePort cancellation result |
| platform shutdown | driver/entrypoints | cataloged driver shutdown or startup failure |

### 5.6 Failure Mapping (closed set)

A new platform-specific failure reason requires a core reason catalog update before use.

| Failure | Required reason |
|---|---|
| platform input cannot map to core type | `external_decode_failed` |
| platform output cannot be encoded | `external_encode_failed` |
| concrete network receive failed | `network_receive_failed` |
| concrete network send failed | `network_send_failed` |
| required runtime/platform configuration missing | `runtime_config_missing` |
| platform/runtime cannot initialize selected driver | `runtime_config_invalid` |
| driver shutdown ended operation | `driver_shutdown` |

### 5.7 SDK Relation

The SDK MAY call the browser / native driver only through its own platform implementation layer. The SDK MUST still expose the Signaling-only public client contract. The browser / native driver MUST NOT add SDK-only semantics to core and MUST NOT expose the SDK event shape as a core event shape.

### 5.8 Prohibitions (prohibition, closed set)

- The browser / native driver defines a core port trait.
- A platform API type crosses into core.
- A platform lifecycle callback owns a domain transition.
- The browser / native driver owns SDK public contract semantics.
- The browser / native driver owns a regulated workflow.
- Platform permission or media device state is treated as generic communication core state in v0.2 initial scope.
- Platform-specific error text becomes the authoritative reason.

### 5.9 Browser / Native Driver Collapse Conditions

Core imports a browser/native concrete type. The SDK public API and the driver implementation boundary collapse. A platform lifecycle event directly mutates Signaling / SFU / TURN state. The browser/native driver introduces media or regulated workflow into the v0.2 initial scope. The browser/native driver admits an out-of-scope feature outside the v0.2 initial scope. A platform-local free-text error bypasses the reason catalog.

---

## 6. Chapter-wide Fail-closed Invariants

The fail-closed invariants common to all drivers in this chapter are as follows (MUST). If conversion / validation does not hold, the driver returns a closed reason without calling core. Every failure names a cataloged reason code and does not permit a category-only value at the boundary. External concrete types stop at the driver boundary and do not cross into core. The driver does not own a domain decision (accept/reject/route/relay/permission/quality/backpressure/version acceptance). A resource-bound drop passes through the audit mapping of the resource bounds / backpressure rule. Drivers do not depend on each other directly and go through the entrypoints composition root. An external response that has lost cataloged reason traceability, or a path that has hidden a required rejection behind generic success, MUST NOT be adopted as close / complete / ready evidence.
