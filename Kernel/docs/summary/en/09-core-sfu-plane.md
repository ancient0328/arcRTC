# core-sfu-plane

Status: SSOT consolidated edition
Date: 2026-06-28 JST

## Purpose

This chapter internalizes the current complete specification of the SFU contract, SFU state machine, the field boundary of the packet semantic view, and the policy/execution boundary of congestion/pacing/retransmission owned by `core/sfu` of arcRTC v0.2 Kernel, at a granularity sufficient for reimplementation from this chapter alone. This chapter makes routing / quality decision / backpressure semantics core-owned and separates execution such as RTP/RTCP byte I/O, str0m, socket, runtime worker, and metrics exporter as driver-owned. Ownership, lifetime, and copy admission of packet bytes (all packet buffer lifecycle states) are defined in Chapter 07 §7. This chapter internalizes the field boundary of the packet semantic view and the SFU-specific routing/state/congestion semantics.

Dependency direction notation: `A <- B` means "B depends on A". The pure semantics of SFU are owned by core and separated from I/O. v0.2 Kernel does not own the SFU product system; the reference implementation / product implementation is placed in implementations outside the Kernel.

---

## 1. SFU Contract

### 1.1 Boundary (owner)

| Domain | Owner | Rule |
|---|---|---|
| routing, forwarding intent, quality decision, backpressure policy | `core/sfu` | authority of SFU domain semantics |
| transport I/O, str0m, RTP byte parsing backend, runtime worker, metrics exporter | drivers | execution detail |
| executable contract / composition evidence surface and wiring | entrypoints | server executable / dependency wiring / configuration / driver selection / process lifecycle |

Ownership, borrowed view, cache, queue, and copy admission of RTP/RTCP packet bytes follow the packet buffer lifecycle (Chapter 07 §7). The policy/execution boundary of congestion/pacing/retransmission follows §4 of this chapter. media codec/track/layer negotiation follows Chapter 07 §4. secure media session lifecycle follows Chapter 07 §8. cross-plane identity / session binding and out-of-scope feature admission follow their respective rule systems.

### 1.2 Core Model (core-owned model, closed set)

core MUST own the following as model.

- SFU session
- participant endpoint
- media stream
- media codec/track/layer reference after accepted negotiation
- publication
- subscription
- forwarding intent
- route candidate
- borrowed packet abstract view
- quality observation as core-owned type
- backpressure state
- admission decision
- rejection reason

### 1.3 Core Decisions (core-owned decision, closed set)

core MUST own the following decisions.

- participant admission / rejection
- publication accepted / rejected
- subscription accepted / rejected
- route selected / not selected
- forwarding allowed / suppressed
- degradation / recovery decision
- backpressure action
- quality violation classification

### 1.4 Driver Responsibilities

driver MUST own the following. RTP/RTCP byte I/O, raw packet bytes, buffer lease, packet cache, transmit queue, pacing queue, retransmission cache, str0m event conversion, UDP/TCP/socket I/O, runtime worker execution, SIMD backend, concrete metrics exporter, external logging sink, legacy interop conversion if retained.

### 1.5 Prohibited semantics (MUST NOT be included in SFU core)

medical workflow priority, application-specific room policy, UI state, recording policy, chat semantics, screen share workflow, DataChannel application semantics, UI / end-user workflow, regulated data classification, concrete worker thread strategy, packet bytes ownership, packet cache, transmit queue, codec implementation / transcoding backend, SFU reference implementation / product implementation.

### 1.6 Fail-Closed Rule (decision handling and reason code closed set)

The following cases MUST be fail-closed per the handling column and connected to a cataloged reason code.

| Failure | Decision handling | Reason code |
|---|---|---|
| unknown participant | rejected | `participant_not_admitted` |
| SFU session is not accepting the decision | rejected | `sfu_session_not_accepting` |
| endpoint admission capacity exceeded | rejected | `endpoint_capacity_exceeded` |
| endpoint admission rejected by quality policy | rejected | `endpoint_quality_not_allowed` |
| endpoint degraded by quality policy | degraded | `endpoint_degraded_by_quality` |
| endpoint or route recovery rejected by quality policy | rejected | `quality_recovery_not_allowed` |
| endpoint or route target unavailable | rejected for lifecycle/routing selection; failed for forwarding execution | `target_unavailable` |
| invalid stream identity | rejected | `stream_not_found` |
| unauthorized publication | rejected | `publication_not_allowed` |
| publication rejected or suppressed by quality policy | rejected or suppressed by decision type | `publication_quality_not_allowed` |
| unauthorized subscription | rejected | `subscription_not_allowed` |
| subscription rejected or suppressed by quality policy | rejected or suppressed by decision type | `subscription_quality_not_allowed` |
| route conflict | rejected for route construction/selection; suppressed for packet forwarding | `route_conflict` |
| route candidate bound exceeded | rejected | `route_candidate_bound_exceeded` |
| route suppressed by quality policy | suppressed | `route_suppressed_by_quality` |
| route suppressed by backpressure policy | suppressed | `route_suppressed_by_backpressure` |
| quality policy violation | suppressed | `packet_suppressed_by_quality` |
| backpressure threshold violation | suppressed | `packet_suppressed_by_backpressure` |
| packet dropped by backpressure policy | dropped | `packet_dropped_by_backpressure` |
| subscription suppressed by backpressure policy | suppressed | `subscription_backpressure_suppressed` |
| action delayed by backpressure policy | delayed | `action_delayed_by_backpressure` |
| route degraded by backpressure policy | degraded | `route_degraded_by_backpressure` |
| endpoint closed by backpressure policy | closed_by_policy | `endpoint_closed_by_backpressure` |
| recovery from backpressure-delayed, backpressure-degraded, or backpressure-suppressed route state rejected | rejected | `backpressure_recovery_not_allowed` |
| SFU transmit queue capacity or wait limit exceeded | dropped | `sfu_transmit_queue_bound_exceeded` |
| unsupported media contract version | rejected | `unsupported_media_contract_version` |
| unsupported codec/profile | rejected | `media_codec_not_supported` |
| track publication/subscription not allowed by media policy | rejected | `media_track_not_allowed` |
| requested media layer unavailable | rejected or suppressed | `media_layer_not_available` |
| invalid payload/SSRC/RID/MID mapping | rejected | `media_payload_mapping_invalid` |

Rate limit / quota / admission decisions for SFU endpoint admission and route candidate construction follow the rate limit/quota/admission rule. When SFU endpoint admission, publication, subscription, route, or forwarding depends on a Signaling participant/session relation, cross-plane binding evidence MUST exist before the claim can cross planes.

### 1.7 collapse conditions (SFU contract)

- worker/runtime layer holds the authority of routing decision.
- metrics exporter holds the authority of quality decision.
- core depends on str0m / tokio / socket concrete type.
- core owns packet bytes, buffer lease, packet cache, transmit queue.
- core owns pacing queue or retransmission cache.
- regulated-specific priority enters generic SFU core.
- codec implementation or payload parser becomes SFU routing authority.
- out-of-scope feature becomes SFU routing or forwarding authority.
- Signaling participant, TURN allocation, ICE candidate, or secure media observation is treated as SFU endpoint/route authority without cross-plane binding.

---

## 2. SFU Boundary (plane separation decision)

### 2.1 Decision

The core semantics of SFU are owned by `core/sfu`. RTP/RTCP packet I/O, str0m, UDP, tokio runtime, worker scheduling, metrics exporter, and legacy bridge are placed in drivers or entrypoints (MUST).

### 2.2 Core SFU Responsibilities

`core/sfu` MUST own the following. participant/endpoint model, media stream identity model, forwarding intent, routing decision, subscription/publication semantics, congestion/backpressure policy, quality threshold/degradation decision, admission/rejection semantics, fault classification, audit-relevant SFU event model.

### 2.3 Driver / Entrypoint Responsibilities

SFU driver: RTP/RTCP byte I/O, str0m event conversion, UDP/TCP/socket concrete transport, runtime worker execution, SIMD/CPU-specific packet processing, metrics exporter, tracing integration, legacy interop bridge if retained.

SFU entrypoint: server executable / product-system wiring detail, dependency wiring, configuration loading, driver selection, process lifecycle.

### 2.4 Prohibited Placement / collapse conditions

Prohibitions: placing the authority of routing decision in the worker loop; placing the authority of quality decision in the metrics exporter; mixing legacy bridge into core's protocol model; core depending on str0m concrete type/tokio runtime/socket/metrics exporter; putting regulated-specific quality semantics into generic SFU core.

Collapse: subtree-copying `server/sfu` into v0.2; routing/quality/backpressure decision scattering into drivers/entrypoints; conflating runtime tuning with core semantics; legacy bridge becoming the center of the current architecture.

---

## 3. SFU State Machine

The authority of SFU state transition is owned by `core/sfu`.

### 3.1 State Owners

| State family | Owner | Driver role |
|---|---|---|
| SFU session state | core | lifecycle observation |
| Endpoint state | core | transport observation conversion |
| Publication state | core | media source observation conversion |
| Subscription state | core | target readiness observation conversion |
| Route state | core | forwarding execution |
| Backpressure action state | core | delayed execution observation |
| Packet buffer lifecycle | driver | governed by packet buffer lifecycle (Chapter 07 §7) |

Process lifecycle or driver worker shutdown does not authorize direct mutation of SFU state without the transitions below (MUST).

### 3.2 Session States (closed set)

| State | Meaning |
|---|---|
| `sfu_session_open` | admits endpoints and route decisions |
| `sfu_session_draining` | suppresses new admission but drains existing routes |
| `sfu_session_closed` | rejects all new SFU decisions |

### 3.3 Endpoint States (closed set)

| State | Meaning |
|---|---|
| `endpoint_observed` | driver observed endpoint |
| `endpoint_admission_pending` | admission decision in progress |
| `endpoint_admitted` | endpoint can publish or subscribe |
| `endpoint_degraded` | endpoint remains admitted with degraded quality state |
| `endpoint_draining` | endpoint is leaving |
| `endpoint_removed` | endpoint is no longer routable |
| `endpoint_rejected` | endpoint admission rejected |

### 3.4 Publication States (closed set)

| State | Meaning |
|---|---|
| `publication_absent` | no active publication |
| `publication_requested` | publication is being evaluated |
| `publication_active` | stream can be routed |
| `publication_suppressed` | stream exists but routing is suppressed |
| `publication_closed` | publication ended |
| `publication_rejected` | publication rejected |

### 3.5 Subscription States (closed set)

| State | Meaning |
|---|---|
| `subscription_absent` | no subscription |
| `subscription_requested` | subscription is being evaluated |
| `subscription_active` | target can receive route |
| `subscription_suppressed` | target subscription exists but forwarding is suppressed |
| `subscription_closed` | subscription ended |
| `subscription_rejected` | subscription rejected |

### 3.6 Route States (closed set)

| State | Meaning |
|---|---|
| `route_candidate` | route may be selected |
| `route_selected` | route is selected by core |
| `route_delayed` | route action is delayed by backpressure policy |
| `route_suppressed_by_backpressure` | route is suppressed by backpressure policy |
| `route_suppressed_by_quality` | route is suppressed by quality policy |
| `route_degraded` | route remains usable only in degraded form |
| `route_dropped` | route cannot be used |
| `route_closed` | route ended |

### 3.7 Transition Rules (all transitions table)

Pre-state tuple notation:

- `family={a, b}` means one state in the same family is required.
- `family=x; other_family=y` means both family states are required at the same time.
- an omitted family means there is no precondition.

| Event | Allowed pre-state | Success state | Decision reason code |
|---|---|---|---|
| `ObserveEndpoint` | `session=sfu_session_open` | `endpoint_observed` | `sfu_session_not_accepting` |
| `BeginEndpointAdmission` | `session=sfu_session_open; endpoint=endpoint_observed` | `endpoint_admission_pending` | `sfu_session_not_accepting`, `endpoint_capacity_exceeded` |
| `AdmitEndpoint` | `endpoint=endpoint_admission_pending` | `endpoint_admitted` | `participant_not_admitted`, `endpoint_capacity_exceeded`, `endpoint_quality_not_allowed` |
| `RejectEndpoint` | `endpoint=endpoint_admission_pending` | `endpoint_rejected` | `participant_not_admitted`, `endpoint_capacity_exceeded`, `endpoint_quality_not_allowed` |
| `DegradeEndpoint` | `endpoint=endpoint_admitted` | `endpoint_degraded` | `endpoint_degraded_by_quality` |
| `RecoverEndpoint` | `endpoint=endpoint_degraded` | `endpoint_admitted` | `quality_recovery_not_allowed` |
| `DrainEndpoint` | `session={sfu_session_open, sfu_session_draining}; endpoint={endpoint_admitted, endpoint_degraded}` | `endpoint_draining` | `sfu_session_not_accepting`, `target_unavailable` |
| `RemoveEndpoint` | `session={sfu_session_open, sfu_session_draining}; endpoint=endpoint_draining` | `endpoint_removed` | `sfu_session_not_accepting`, `target_unavailable` |
| `StartPublication` | `session=sfu_session_open; endpoint=endpoint_admitted; publication=publication_absent` | `publication_requested` | `sfu_session_not_accepting`, `publication_not_allowed`, `publication_quality_not_allowed` |
| `AcceptPublication` | `session=sfu_session_open; endpoint={endpoint_admitted, endpoint_degraded}; publication=publication_requested` | `publication_active` | `sfu_session_not_accepting`, `target_unavailable`, `publication_not_allowed`, `publication_quality_not_allowed` |
| `RejectPublication` | `publication=publication_requested` | `publication_rejected` | `publication_not_allowed`, `publication_quality_not_allowed` |
| `SuppressPublication` | `session={sfu_session_open, sfu_session_draining}; publication=publication_active` | `publication_suppressed` | `sfu_session_not_accepting`, `publication_quality_not_allowed` |
| `ClosePublication` | `session={sfu_session_open, sfu_session_draining}; publication={publication_active, publication_suppressed}` | `publication_closed` | `sfu_session_not_accepting`, `publication_not_allowed` |
| `StartSubscription` | `session=sfu_session_open; endpoint=endpoint_admitted; subscription=subscription_absent` | `subscription_requested` | `sfu_session_not_accepting`, `subscription_not_allowed`, `subscription_quality_not_allowed` |
| `AcceptSubscription` | `session=sfu_session_open; endpoint={endpoint_admitted, endpoint_degraded}; subscription=subscription_requested` | `subscription_active` | `sfu_session_not_accepting`, `target_unavailable`, `subscription_not_allowed`, `subscription_quality_not_allowed` |
| `RejectSubscription` | `subscription=subscription_requested` | `subscription_rejected` | `subscription_not_allowed`, `subscription_quality_not_allowed` |
| `SuppressSubscription` | `session={sfu_session_open, sfu_session_draining}; subscription=subscription_active` | `subscription_suppressed` | `sfu_session_not_accepting`, `subscription_backpressure_suppressed`, `subscription_quality_not_allowed` |
| `CloseSubscription` | `session={sfu_session_open, sfu_session_draining}; subscription={subscription_active, subscription_suppressed}` | `subscription_closed` | `sfu_session_not_accepting`, `subscription_not_allowed` |
| `BuildRouteCandidate` | `session=sfu_session_open; endpoint={endpoint_admitted, endpoint_degraded}; publication=publication_active; subscription=subscription_active` | `route_candidate` | `sfu_session_not_accepting`, `target_unavailable`, `route_conflict`, `stream_not_found`, `route_candidate_bound_exceeded` |
| `SelectRoute` | `session=sfu_session_open; endpoint={endpoint_admitted, endpoint_degraded}; publication=publication_active; subscription=subscription_active; route=route_candidate` | `route_selected` | `sfu_session_not_accepting`, `target_unavailable`, `route_conflict`, `stream_not_found`, `route_candidate_bound_exceeded` |
| `DelayActionByBackpressure` | `session={sfu_session_open, sfu_session_draining}; route={route_candidate, route_selected}` | `route_delayed` | `sfu_session_not_accepting`, `action_delayed_by_backpressure` |
| `ApplyBackpressure` | `session={sfu_session_open, sfu_session_draining}; route=route_selected` | `route_suppressed_by_backpressure` | `sfu_session_not_accepting`, `route_suppressed_by_backpressure` |
| `DegradeRouteByBackpressure` | `session={sfu_session_open, sfu_session_draining}; route={route_selected, route_delayed}` | `route_degraded` | `sfu_session_not_accepting`, `route_degraded_by_backpressure` |
| `CloseEndpointByBackpressure` | `session={sfu_session_open, sfu_session_draining}; endpoint={endpoint_admitted, endpoint_degraded}` | `endpoint_draining` | `sfu_session_not_accepting`, `endpoint_closed_by_backpressure` |
| `ApplyQualitySuppression` | `session={sfu_session_open, sfu_session_draining}; route=route_selected` | `route_suppressed_by_quality` | `sfu_session_not_accepting`, `route_suppressed_by_quality` |
| `RecoverRoute` | `session={sfu_session_open, sfu_session_draining}; route=route_suppressed_by_quality` | `route_selected` | `sfu_session_not_accepting`, `quality_recovery_not_allowed` |
| `RecoverRouteFromBackpressure` | `session={sfu_session_open, sfu_session_draining}; route={route_delayed, route_degraded, route_suppressed_by_backpressure}` | `route_selected` | `sfu_session_not_accepting`, `backpressure_recovery_not_allowed` |
| `DropRoute` | `session={sfu_session_open, sfu_session_draining}; route={route_candidate, route_selected, route_delayed, route_suppressed_by_backpressure, route_suppressed_by_quality, route_degraded}` | `route_dropped` | `sfu_session_not_accepting`, `route_conflict`, `target_unavailable` |
| `CloseRoute` | `session={sfu_session_open, sfu_session_draining}; route={route_selected, route_delayed, route_suppressed_by_backpressure, route_suppressed_by_quality, route_degraded, route_dropped}` | `route_closed` | `sfu_session_not_accepting`, `target_unavailable` |
| `BeginSessionDrain` | `session=sfu_session_open` | `sfu_session_draining` | `sfu_session_not_accepting` |
| `CloseSession` | `session={sfu_session_open, sfu_session_draining}` | `sfu_session_closed` | `sfu_session_not_accepting` |

### 3.8 Implicit transition rules (draining / closed / endpoint-unavailable coverage)

- Transitions without an explicit `session=sfu_session_draining` allowed pre-state are rejected in `sfu_session_draining` with `sfu_session_not_accepting`.
- Transitions without an explicit `session=sfu_session_closed` allowed pre-state are rejected in `sfu_session_closed` with `sfu_session_not_accepting`.
- Transitions that require an admitted/degraded endpoint reject `endpoint_draining`, `endpoint_removed`, and `endpoint_rejected` with `target_unavailable`.

### 3.9 Packet Relation

- packet buffer lifecycle is not SFU state.
- core may decide route, suppression, and drop for packet identity.
- driver owns packet bytes, cache, queue, retransmission, and release.
- Pacing queue, retransmission cache, and NACK-adjacent concrete feedback remain driver-owned execution detail (MUST).

### 3.10 collapse conditions (state machine)

- worker runtime owns route state.
- driver owns publication / subscription decision.
- packet buffer lifecycle is treated as core SFU state.
- pacing/retransmission cache is treated as core SFU state.
- concurrent SFU route or endpoint conflict is resolved by driver/runtime order.
- quality exporter owns suppression / recovery decision.
- regulated-specific priority changes generic SFU state transition.

---

## 4. Congestion / Pacing / Retransmission (policy/execution boundary)

This section fixes the responsibility boundary of the SFU's congestion observation, pacing, retransmission, and NACK-adjacent behavior. This section does not claim an implemented congestion controller, a concrete pacing algorithm, or packet loss recovery success.

### 4.1 Boundary (owner)

| Concern | Owner | Rule |
|---|---|---|
| congestion/backpressure policy | core/sfu | semantics of suppress/drop/degrade/delay/close/recovery rejection |
| quality decision | core/quality + core/sfu | quality metric model and decision semantics |
| raw queue/cache/socket measurement | driver | converts concrete observation into core-owned metric type |
| packet cache bytes | driver | bounded retention, eviction, lease release |
| pacing queue/timer | driver | scheduling execution detail |
| retransmission execution | driver | sends from cached packet lease |
| route/target selection | core/sfu | forwarding target set and routing decision |
| send operation | driver | concrete I/O and transport backend |

core does not own packet cache, pacing queue, timer wheel, NACK buffer, socket send queue, or raw bytes (prohibited). driver does not own the authority of congestion/backpressure policy (prohibited).

### 4.2 Congestion Observation Rule

driver may observe: transmit queue depth; send latency; packet cache pressure; packet loss signal or NACK-like event; transport/backend send failure; memory pressure; receive buffer pressure.

driver MUST map these observations to core-owned metric/input types before core decision. driver-local observation MUST NOT directly mutate route/subscription/endpoint/quality state.

### 4.3 Pacing Rule

- pacing is execution, not policy.
- core may return delay/degrade/suppress/drop intent with cataloged reason.
- driver may implement pacing queue and timers to execute accepted forwarding decisions.
- Pacing queue MUST be bounded per resource bounds. When enqueue or wait bound is exceeded, driver MUST end the packet lifecycle with `dropped_by_transmit_queue_bound` and reason `sfu_transmit_queue_bound_exceeded`.

### 4.4 Retransmission Rule

- retransmission is allowed only from driver-owned bounded packet cache (MUST).
- core may decide whether retransmission is semantically allowed for a packet/route/target.
- driver executes retransmission by resolving `PacketId` to a still-valid `BufferLease` or cached packet representation.
- If cache entry is absent/expired/evicted/bound-exceeded, retransmission MUST fail closed with the matching release/audit reason. driver MUST NOT reconstruct packet bytes in core or ask core to persist packet payload.

### 4.5 NACK-Adjacent Rule

- NACK-like external feedback is driver input until converted to a core-owned feedback reference.
- core may interpret the converted reference as retransmission intent, suppression input, or quality/backpressure signal.
- Concrete feedback packet parsing, protocol-specific ACK/NACK grammar, and backend event objects are driver-owned (MUST).

### 4.6 Failure Mapping (congestion/pacing/retransmission)

| Failure | Required reason or release code |
|---|---|
| packet cache cannot retain packet | `packet_cache_bound_exceeded` |
| packet cache retention duration exceeded | `retention_duration_exceeded` |
| transmit queue bound exceeded | `sfu_transmit_queue_bound_exceeded` |
| backpressure suppresses packet forwarding | `packet_suppressed_by_backpressure` |
| backpressure drops packet forwarding | `packet_dropped_by_backpressure` |
| route degraded by backpressure | `route_degraded_by_backpressure` |
| endpoint closed by backpressure | `endpoint_closed_by_backpressure` |
| recovery from suppressed/degraded/delayed route rejected | `backpressure_recovery_not_allowed` |
| target unavailable | `target_unavailable` |
| concrete send failed | `network_send_failed` |
| driver shutdown | `driver_shutdown` |

For packet lifecycle closure, the release code table of the packet buffer lifecycle (Chapter 07 §7) remains authoritative.

### 4.7 Audit Rule

- Each congestion/backpressure decision that changes forwarding/route/subscription/endpoint/packet lifecycle MUST be recorded using the event type mapping of the resource bounds and audit event rules.
- Driver pacing execution logs are diagnostics only and do not replace `backpressure_decision`, `resource_bound_decision`, `sfu_forwarding_decision`, or `quality_violation_decision` audit events (MUST).

### 4.8 Prohibitions (congestion/pacing/retransmission)

- core owns pacing queue or retransmission cache.
- driver changes route/subscription/endpoint state by congestion policy without core decision.
- retransmission cache is unbounded.
- NACK parser concrete type enters core.
- packet payload is persisted for retransmission by core.
- retransmission transform executes without packet rewrite / media transform admission (Chapter 07 §5).
- queue overflow is treated as successful forwarding.

### 4.9 collapse conditions (congestion/pacing/retransmission)

- pacing/retransmission data structure becomes core-owned.
- driver congestion observation mutates SFU state without core decision.
- retransmission uses packet bytes after driver lease lifetime ended.
- queue/cache overflow has no cataloged reason and audit event.
- diagnostic pacing logs are used as forwarding success evidence.
- retransmission path bypasses rewrite/transform class and copy bound evidence.
