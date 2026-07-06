# core-quality-and-admission

Status: public summary projection
Date: 2026-07-06 JST

## Purpose

This chapter internalizes the current complete specification of the quality metrics model and quality decision, and the identity-neutral resource policy boundary of rate limit / quota / admission, owned by the core of arcRTC v0.2 Kernel, at a granularity sufficient for re-implementation from this chapter alone.

Dependency direction notation: `A <- B` means "B depends on A". The core is free of external I/O dependencies. Quality is a state judgment of the communication infrastructure, not a regulated workflow priority. Admission is a bound decision over active room, participant, endpoint, allocation, connection, and similar, and MUST NOT mix tenant/user/application identity into the generic core. The core owns metric model, threshold, decision, closed reason, and admission policy semantics; the driver owns measurement source, exporter, runtime observation, external metric format, and enforcement execution. This chapter internalizes all quality decisions and admission decisions, closed-set reasons, and fail-closed conditions.

---

## 1. Quality Metrics Model / Quality Decision

### 1.1 Boundary (owner)

The core owns metric model, threshold, decision, and closed reason. drivers own measurement source, exporter, runtime observation, and external metric format. Unit, measurement, time window, precision, and comparison normalization follow the unit normalization rule (chapter 11 section 6). Observability signal taxonomy for exported metrics follows the observability signal taxonomy rule.

### 1.2 Core Metrics (core-owned, base set)

The metrics handled by the core have the following base set: RTT; packet loss; jitter; bitrate; frame rate if a media-facing contract requires it; relay latency; queue depth; backpressure state; route health; MOS-like score if explicitly defined by a core formula.

### 1.3 Core Decisions (core-owned)

The core decides: quality normal; quality degraded; quality violation; route suppression; backpressure action; recovery allowed; admission rejected by quality policy.

### 1.4 Driver Responsibilities (driver-owned)

The driver owns: raw measurement collection; platform-specific statistics source; str0m / RTP / RTCP concrete stats conversion; metrics exporter; dashboard / log format. Raw driver statistics MUST be converted to normalized units before core quality policy evaluation.

### 1.5 Prohibited Semantics

The quality core MUST NOT include: medical severity; patient priority; business SLA name; UI alert copy; platform-specific metric object; exporter-specific label schema as core contract; raw platform measurement unit or hidden sampling window; unbounded or sensitive observability metric label.

### 1.6 Closed Reason Requirement (closed set)

The quality decision reason is a closed set. A free-text only reason is not used for core decisions. The reason code MUST be a code from the core reason catalog, and category-only reason values are prohibited.

| Decision | Reason code |
|---|---|
| packet forwarding suppressed by quality | `packet_suppressed_by_quality` |
| route suppressed by quality policy | `route_suppressed_by_quality` |
| route recovery rejected by quality policy | `quality_recovery_not_allowed` |
| endpoint recovery rejected by quality policy | `quality_recovery_not_allowed` |
| endpoint admission rejected by quality policy | `endpoint_quality_not_allowed` |
| admitted endpoint degraded by quality policy | `endpoint_degraded_by_quality` |
| publication admission or suppression by quality policy | `publication_quality_not_allowed` |
| subscription admission or suppression by quality policy | `subscription_quality_not_allowed` |

### 1.7 Audit Mapping Rule (closed set)

The quality decision audit mapping is fixed as follows. Even when an SFU forwarding audit is emitted with the same correlation ID, it MUST NOT substitute for the quality violation audit. The quality target reference is closed to `EndpointId`, `RouteId`, `StreamId`, or `PacketId`. Packet forwarding suppressed by quality MUST carry `PacketId` when packet identity has been materialized.

| Quality decision | Audit event type | Decision outcome | Reason code |
|---|---|---|---|
| packet forwarding suppressed by quality | `quality_violation_decision` | `suppressed` | `packet_suppressed_by_quality` |
| route suppressed by quality policy | `quality_violation_decision` | `suppressed` | `route_suppressed_by_quality` |
| route recovery rejected by quality policy | `quality_violation_decision` | `rejected` | `quality_recovery_not_allowed` |
| endpoint recovery rejected by quality policy | `quality_violation_decision` | `rejected` | `quality_recovery_not_allowed` |
| endpoint admission rejected by quality policy | `quality_violation_decision` | `rejected` | `endpoint_quality_not_allowed` |
| admitted endpoint degraded by quality policy | `quality_violation_decision` | `degraded` | `endpoint_degraded_by_quality` |
| publication admission rejected by quality policy | `quality_violation_decision` | `rejected` | `publication_quality_not_allowed` |
| publication suppressed by quality policy | `quality_violation_decision` | `suppressed` | `publication_quality_not_allowed` |
| subscription admission rejected by quality policy | `quality_violation_decision` | `rejected` | `subscription_quality_not_allowed` |
| subscription suppressed by quality policy | `quality_violation_decision` | `suppressed` | `subscription_quality_not_allowed` |

The allowed outcomes of `quality_violation_decision` are `rejected`, `suppressed`, and `degraded` (consistent with chapter 12 section 3.9).

### 1.8 Collapse Conditions (quality)

The metrics exporter holds the authority of the quality decision; a platform-specific stats object becomes a core type; regulated priority is mixed into a generic quality decision; the threshold owner becomes other than core; the quality decision compares unnormalized units or hidden measurement windows; observability exporter label/sampling changes quality decision semantics.

---

## 2. Rate Limit / Quota / Admission

### 2.1 Boundary (owner)

This section fixes the granularity that keeps the admission policy fail-closed without mixing tenant/user/application identity into the generic core. Bounded resource and reason are defined by the resource bounds / backpressure rule (chapter 11 section 8). This section does not assert concrete rate values, quota values, multi-tenant billing policy, or production enforcement. Authorization context mapping, when used as admission policy input, follows the authorization context rule (chapter 12 section 2), and edge/proxy source metadata used as admission input follows the edge/proxy trust rule.

| Concern | Owner | Rule |
|---|---|---|
| admission policy semantics | core | bound decision over active room, participant, endpoint, allocation, connection, etc. |
| quota/rate policy input | core typed policy | does not directly read external tenant/user semantics |
| external identity/auth context mapping | entrypoints/drivers before core policy input | convert to opaque reference or accepted policy scope |
| edge/proxy source metadata mapping | entrypoints/drivers before core policy input | admitted trust policy required |
| raw measurement | driver | connection count, queue depth, time window observation |
| enforcement execution | driver | connection close, queue reject, packet drop, send suppression |
| audit evidence | core event model and driver sink | resource-bound decision with owner tuple |

The core does not own application tenant, billing account, user profile, or regulated role as a generic protocol identity. Necessary grouping is expressed by a core-owned opaque admission scope reference and typed policy input.

### 2.2 Admission Scope Rule (closed set)

The admission scope is limited to one of the following core-owned references.

| Scope | Meaning | Example bound |
|---|---|---|
| `global` | process/runtime wide policy scope | connection concurrency |
| `room` | room-scoped policy | room participant set |
| `session` | SFU session-scoped policy | endpoint admission |
| `allocation` | TURN allocation-scoped policy | permission count |
| `credential_ref` | opaque credential/key verification result scope | credential-bound admission when explicitly configured |
| `driver_connection_ref` | pre-core connection scope | frame size and connection concurrency |

Tenant/user/entrypoint role is not a built-in admission scope in v0.2 initial architecture. Adding such a scope requires a specification update that preserves the generic communication boundary. Authorization context MAY map external auth material to an admitted opaque scope only through the authorization rule. Forwarded header, client IP, host, origin, or SNI is not an admission scope by default.

### 2.3 Required Admission Decisions (closed set)

| Admission target | Owner | Failure reason |
|---|---|---|
| room materialization / active room | core decision, driver execution | `room_capacity_exceeded` |
| participant join | core decision, driver execution | `admission_capacity_exceeded` |
| signaling command queue admission | core policy, driver queue execution | `signaling_command_queue_bound_exceeded` |
| SFU endpoint admission | core decision, driver execution | `endpoint_capacity_exceeded` |
| SFU route candidate construction | core routing decision | `route_candidate_bound_exceeded` |
| TURN allocation | core decision, driver execution | `allocation_capacity_exceeded` |
| TURN permission | core decision, driver execution | `permission_capacity_exceeded` |
| connection concurrency | core policy, driver execution | `connection_concurrency_exceeded` |
| inbound frame | driver-local pre-core bound | `frame_size_bound_exceeded` |

Required decisions reuse the required bound table and owner tuple of the resource bounds / backpressure rule (chapter 11 section 8).

### 2.4 Rate Window Rule

The rate limit window is policy, not storage implementation. The core owns the accepted typed policy shape and decision semantics. The driver MAY measure time/window counters, but MUST present observations through core-owned input types where the decision is core-owned.

Allowed window attributes: scope; maximum count; maximum bytes when applicable; maximum duration/window; reset semantics; reason code; audit event type; owner tuple. Window counters MUST be bounded and MUST NOT become durable domain state unless a persistence/restore specification explicitly allows it.

### 2.5 Fail-Closed Rule

When admission measurement, quota lookup, or rate window state required for a core-owned admission decision is unavailable, the affected admission path MUST reject or stop with a cataloged reason (fail-closed). It MUST NOT accept by default.

| Failure | Required reason |
|---|---|
| bounded admission capacity reached | target-specific resource reason |
| driver measurement cannot be obtained due to driver failure | `driver_shutdown` or applicable driver failure reason |
| runtime quota configuration missing | `runtime_config_missing` |
| runtime quota configuration invalid | `runtime_config_invalid` |
| persistence-backed quota source unavailable when required | `persistence_unavailable` |
| required authorization context missing | `authorization_context_missing` |
| authorization scope not allowed | `authorization_scope_not_allowed` |
| proxy/header/client address cannot be trusted for admission | `client_address_untrusted` or `forwarded_header_untrusted` |
| frame size exceeded before core entry | `frame_size_bound_exceeded` |

### 2.6 Audit Rule

Every rejected/dropped/shed/expired admission or quota decision MUST connect to the audit event model (chapter 12 section 3). Resource-bound related events MUST carry resource policy owner and physical resource owner. If the audit sink is unavailable and no bootstrap/overflow path can record the decision where required, the affected accepted path MUST NOT be used as closeout evidence (fail-closed).

### 2.7 Prohibitions / Collapse Conditions (admission)

Prohibitions: tenant/user/billing identity becomes generic core protocol identity without a specification update; driver accepts admission after required quota/rate state is unavailable; unbounded rate counter or quota cache; rate limit rejection uses free-text reason; SDK platform wrapper changes server admission reason; entrypoint config silently disables required admission bound; authorization context bypasses identity-neutral admission scope; raw proxy metadata becomes admission scope without edge trust policy.

Collapse: admission can fail open when quota/rate state is unavailable; identity scope is inferred from regulated/entrypoint user model inside generic core; rate/quota counters are unbounded or unaudited; resource-bound audit owner tuple is missing; concrete rate values are claimed as production policy without evidence; external auth role becomes admission scope without a specification update; client IP or forwarded header is used as quota identity without trusted metadata admission.
