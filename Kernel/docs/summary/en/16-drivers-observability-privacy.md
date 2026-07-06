# drivers-observability-privacy

Status: public summary projection
Date: 2026-07-06 JST

## Purpose

This chapter internalizes the current complete specification of the observability / privacy driver family of arcRTC v0.2 Kernel, at a granularity sufficient for re-implementation from this chapter alone. This chapter internalizes the observability boundary (tracing / metrics exporter / log sink), the closed set of observability signal taxonomy / cardinality / sampling, and the privacy / redaction / retention rules (the owner and data classes that prevent raw secret / token / media / regulated payload from mixing into core / audit / log / report), down to owners, closed-set vocabulary, failure mapping, prohibitions/permissions, and fail-closed conditions, omitting none.

Dependency direction notation: `A <- B` means "B depends on A". Observability is a driver family that implements core-owned ports and does not own the authority of audit event meaning / quality decision / domain decision. An observability signal is not an audit event by default. This chapter does not claim exporter implementation, dashboard, alert runtime, production monitoring, legal compliance, production privacy certification, or that operational retention is already configured.

---

## 1. Observability Boundary (`drivers/observability`)

### 1.1 Boundary (ownership assignment, closed set)

Observability handles tracing, metrics exporter, and log sink but does not own the authority of audit event meaning, quality decision, or domain decision.

| Surface | Owner | Rule |
|---|---|---|
| audit event meaning | core | audit event rule |
| audit hash-chain semantics | core | audit hash-chain rule |
| quality metric model and decision | core | quality metrics rule |
| metrics export format | driver | Prometheus-style, tracing, vendor exporter |
| log / trace sink | driver | external sink and formatting |
| entrypoint-level exporter selection | entrypoints | typed configuration and wiring only |

Signal taxonomy, cardinality, sampling, and alert boundary follow the observability signal taxonomy in Section 2.

### 1.2 Metrics Rule

Core owns metric names only when the metric participates in quality or resource-bound decision semantics. The driver MAY export additional operational metrics, but driver-exported metrics are not authoritative domain state. If a metric participates in a quality decision, its decision rule MUST be defined in the quality metrics rule. Exporter-side aggregation MUST NOT change quality decision semantics. Metric label class, cardinality, and sampling MUST follow the observability signal taxonomy (Section 2).

### 1.3 Logging Rule

Logs and traces are operational diagnostics. They do not replace audit events, Closed Gate Reports, or reproducible test evidence. Logs MUST NOT carry raw secrets, raw tokens, raw credentials, packet payload bytes, or regulated domain payload. If logs include references, they MUST use core-owned opaque references from the core identity and reference rule. Redaction and retention rules for logs / traces / metric labels / reports / raw tokens / raw packet payload / regulated payload follow the privacy / redaction / retention rule (Section 3). Trace / span / log signal class MUST be declared before adoption as evidence.

### 1.4 Failure Mapping (closed set)

Observability failure MUST be recorded as a driver failure or a resource-bound decision where required. It MUST NOT change a domain decision to success or hide a required rejection.

| Failure | Required reason |
|---|---|
| metrics export failed | `metrics_export_failed` |
| metrics backlog bound exceeded | `metrics_backlog_bound_exceeded` |
| signal taxonomy invalid | `observability_signal_invalid` |
| metric label cardinality exceeded | `metric_cardinality_exceeded` |
| required telemetry sampling policy absent | `telemetry_sampling_policy_missing` |
| driver shutdown | `driver_shutdown` |

### 1.5 Evidence Rule

Observability output can support investigation but is not accepted as close / complete / ready evidence unless a dated evidence record records correlation ID, command, environment, and reproducible procedure. Metrics samples without time window, source, and bound definition are not evidence for performance / stability claims. Observability samples without signal class, cardinality class, and sampling policy are diagnostic only.

### 1.6 Prohibitions (prohibition, closed set)

- A metrics exporter owns a quality decision.
- Log text becomes closed reason vocabulary.
- A tracing span name becomes an audit event type.
- The observability driver depends directly on the network / persistence driver to alter domain flow.
- A raw token / credential / packet payload / regulated payload is logged.
- A missing metrics export is ignored while claiming verification success.
- An alert / metric / trace signal drives a domain decision directly.
- Metric label cardinality is unbounded.

### 1.7 Observability Boundary Collapse Conditions

Observability output replaces audit event semantics. Exporter aggregation changes the core quality decision. Log / free-text becomes the authoritative reason. The metrics backlog is unbounded or unaudited. Operational logs are used as closeout evidence without a reproducible report. Signal taxonomy or sampling policy is omitted while claiming observability evidence.

---

## 2. Observability Signal Taxonomy / Cardinality / Sampling

### 2.1 Boundary (ownership assignment, closed set)

This section fixes signal class and adoption conditions so that metrics / logs / traces / alerts are not confused with audit, quality decision, runtime evidence, or operator action. An observability signal is not an audit event by default.

| Signal surface | Owner | Rule |
|---|---|---|
| audit event | core model | audit event rule |
| quality metric used for decision | core quality policy | quality metrics rule |
| operational metric export | driver observability | diagnostic unless adopted by report |
| trace/span/log formatting | driver observability | no domain meaning authority |
| alert rule | entrypoints/operations policy | cannot define core reason |
| signal sampling/cardinality policy | observability policy or profile | must be explicit before the evidence is accepted |

### 2.2 Signal Classes (closed set)

The signal classes of the v0.2 initial architecture are limited to the following. A new signal class is out of the v0.2 initial scope.

| Class | Meaning | Evidence rule |
|---|---|---|
| `audit_signal` | canonical audit event or hash-chain record | the audit policy owns meaning |
| `quality_decision_metric` | metric used by quality decision | the quality policy and unit normalization required |
| `resource_bound_metric` | metric used by resource bound decision | resource bound owner tuple required |
| `operational_metric` | diagnostic/exported metric | report required for evidence |
| `trace_span` | execution trace diagnostic | not decision authority |
| `structured_log` | redacted log diagnostic | not reason vocabulary |
| `alert_signal` | derived operator notification | not domain decision |
| `profiling_signal` | CPU/memory/runtime diagnostic | not correctness proof |

### 2.3 Cardinality and Label Rule (required declared items)

Metric labels, log fields, and trace attributes MUST declare the following: signal class, owner, allowed reference types, sensitive data classification, maximum cardinality class, sampling rule when sampling applies, retention/redaction rule, and evidence acceptance rule. Unbounded labels derived from user identity, token claims, raw packet data, SDP/ICE material, regulated payload, or external free-text are prohibited.

### 2.4 Sampling Rule

Sampling MAY reduce diagnostic volume but MUST NOT alter the following: audit event completeness, resource-bound decision evidence, quality decision input unless the quality policy explicitly allows sampling, and Closed Gate evidence. If sampling hides required evidence, the evidence claim MUST fail closed.

### 2.5 Failure Mapping (closed set)

| Failure | Required reason |
|---|---|
| signal does not match allowed taxonomy | `observability_signal_invalid` |
| metric label cardinality bound exceeded | `metric_cardinality_exceeded` |
| required sampling policy absent | `telemetry_sampling_policy_missing` |
| alert signal attempts to drive domain decision | `alert_signal_not_allowed` |
| observability export is not allowed by privacy/retention policy | `observability_export_not_allowed` |
| metrics export failed | `metrics_export_failed` |
| metrics backlog bound exceeded | `metrics_backlog_bound_exceeded` |

### 2.6 Evidence Rule

Observability evidence MUST record signal class, sampling, cardinality, time window, source, redaction, and report rerun condition. Operational metric or log output without these fields remains diagnostic only.

### 2.7 Audit Rule

Observability signal decisions use audit event type `observability_signal_decision`. The event MUST carry signal class, signal reference, and resource owner fields when the rejection/drop/failure is cardinality or export-bound.

### 2.8 Prohibitions and Collapse Conditions

Prohibitions (closed set): a trace span name becomes an audit event type; an alert status becomes a domain decision; a sampled operational metric is used as complete audit evidence; a metric label contains raw identity / token / packet payload / regulated payload; dashboard state is treated as a runtime correctness proof; exporter aggregation changes quality decision semantics. Collapse conditions: signal class is omitted; unbounded cardinality is allowed; sampling policy hides required evidence; alert / metric / log replaces audit or the reason catalog; observability signal taxonomy differs by driver without a core contract update.

---

## 3. Privacy / Redaction / Retention

### 3.1 Boundary (ownership assignment, closed set)

This section fixes the owner and retention class that prevent raw secret, raw token, raw media, and regulated payload from mixing among generic communication core, drivers, entrypoints, sdk, regulated, observability, audit, and reports. Observability signal taxonomy and metric/log label class follow Section 2; secret rotation evidence follows the secret rotation lifecycle rule; edge/proxy metadata redaction follows the edge/proxy trust boundary rule.

| Surface | Owner | Rule |
|---|---|---|
| core reason/reference/audit model | core | owns only opaque references and non-sensitive tags |
| raw credential/token/key material | driver/entrypoints | does not emit to core/audit/log/report |
| raw RTP/RTCP/media payload | driver | confined within the packet lifecycle |
| regulated payload | regulated | does not mix into generic core |
| log/trace/metric formatting | driver | external sink representation after redaction |
| report evidence | dated evidence reports | records correlation/reference without raw sensitive data |
| SDK client local data | sdk | not a server/core source-of-truth |

### 3.2 Data Classes (closed set)

The data classes of the v0.2 initial architecture are limited to the following. A new data class is out of the v0.2 initial scope.

| Data class | Allowed owner | Retention rule |
|---|---|---|
| `core_reference` | core | may be retained as an opaque ID/reference |
| `catalog_reason` | core | may be retained as a closed reason category/code |
| `non_sensitive_tag` | core/driver/regulated | only tags in the allowlist |
| `raw_secret` | entrypoints/driver | does not emit to audit/log/report/core state |
| `raw_token` | driver boundary | handled boundedly as verification input, not retained |
| `raw_key_material` | driver | bounded cache only; log/report prohibited |
| `raw_packet_payload` | driver | packet lifecycle retention only |
| `regulated_payload` | regulated | does not mix into the generic core/drivers/sdk/entrypoints path |
| `diagnostic_detail` | driver | non-authoritative; redaction mandatory |
| `edge_proxy_metadata` | driver/entrypoints | redacted evidence only after trust class is declared |

### 3.3 Redaction Rule (allowlist and prohibited fields, closed set)

Redaction is not merely a post-hoc step at sink formatting; it is performed by an allowlist before crossing the boundary.

Allowed evidence/log fields (permission, closed set):

- correlation ID;
- startup run ID;
- opaque room/session/participant/endpoint/packet/allocation references;
- event type code;
- decision outcome;
- reason category/code;
- bounded resource name and owner tuple;
- non-sensitive tag explicitly allowed by the core contract;
- edge/proxy class and trusted metadata class when redacted and admitted.

Prohibited evidence/log fields (prohibition, closed set):

- raw token;
- raw credential;
- raw key material;
- SDP body or ICE string when it contains sensitive network detail and no redacted reference has been defined;
- raw RTP/RTCP/media payload bytes;
- regulated payload;
- patient/user profile or entrypoint role as protocol identity;
- raw forwarded chain, raw client IP, host, origin, or SNI when no redacted reference has been defined.

### 3.4 Retention Rule (closed set)

Retention duration/bytes/count bounds MUST connect to the resource bounds / backpressure rule or a specialized retention policy.

| Retention target | Rule |
|---|---|
| audit event | retains closed event fields and hash-chain data only |
| audit hash-chain | retains tamper-evidence material, not mutable domain state |
| logs/traces | operational diagnostics; no raw sensitive data |
| metrics | labels must not contain raw identifiers or sensitive payload |
| packet cache | bounded driver-local retention only |
| key cache | bounded driver-local retention only |
| evidence report | correlation/reference and reproducible procedure only |

### 3.5 Report Rule

A Closed Gate Report, verification report, benchmark report, and analysis report MUST NOT paste raw sensitive material as evidence. When necessary, convert it into a redacted excerpt, opaque reference, hash/reference, or reproducible command/result summary. If a report contains raw sensitive data, that report MUST NOT be adopted as close / complete / ready evidence.

### 3.6 Failure Rule (closed set)

| Failure | Required handling |
|---|---|
| redaction cannot be applied | stop affected logging/reporting path; do not emit raw data |
| secret source unavailable | `secret_unavailable` |
| token/key verification detail cannot be safely exposed | preserve cataloged non-sensitive reason only |
| packet payload requested for report | reject the report as evidence unless redacted reference exists |
| metrics label would contain sensitive data | drop or rewrite label before export |
| observability signal label cardinality would expose sensitive identity | drop or rewrite signal before export |
| rotation evidence would expose raw secret material | reject the report as evidence |
| edge/proxy metadata would expose sensitive network identity | redact, drop, or reject the report as evidence |

### 3.7 Prohibitions (prohibition, closed set)

- A raw secret/token/key is included in core state, audit event, log, metric label, SDK public error, or report.
- Packet payload bytes are used as closeout evidence.
- Regulated payload is added to a generic core audit event.
- Redaction is left to external sink default behavior.
- Free-text diagnostic detail becomes the authoritative reason.
- A report preserves sensitive source material to improve reproducibility.
- Raw proxy/header/source-address material is logged or reported as authoritative identity.

### 3.8 Privacy / Redaction / Retention Collapse Conditions

Raw sensitive data crosses into core/audit/report as authoritative data. Retention has no bound or owner. Redaction failure emits raw data. Metric/log labels contain sensitive identity or payload. Report evidence cannot be shared without exposing sensitive material. Secret rotation or observability evidence exposes raw sensitive material. Edge/proxy metadata evidence lacks redaction and trust classification.

---

## 4. Chapter-wide Fail-closed Invariants

The fail-closed invariants common to all observability / privacy rules in this chapter are as follows (MUST). Observability failure does not change a domain decision to success or hide a required rejection. Every observability failure names a cataloged reason code. Metric label cardinality is bounded, and signal class / sampling policy is made explicit before the evidence is accepted. Redaction is performed by an allowlist before crossing the boundary; when it cannot be applied, the affected logging/reporting path is stopped and raw data is not emitted. Raw secret / token / key material / packet payload / regulated payload does not enter any of core state / audit event / log / metric label / SDK public error / report. If a report contains raw sensitive data, that report MUST NOT be adopted as close / complete / ready evidence.
