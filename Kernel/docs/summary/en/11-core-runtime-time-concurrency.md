# core-runtime-time-concurrency

Status: public summary projection
Date: 2026-07-06 JST

## Purpose

This chapter internalizes the current complete specification of the runtime / clock / randomness abstraction boundary, runtime task / worker lifecycle, retry / timeout / cancellation, concurrency / ordering / lock ownership, atomicity / transaction / compensation, unit / measurement / time normalization, time synchronization / clock skew / timestamp trust, and resource bounds / backpressure owned by the core of arcRTC v0.2 Kernel, at a granularity sufficient for re-implementation from this chapter alone.

Dependency direction notation: `A <- B` means "B depends on A". The core is free of external I/O dependencies. time, entropy, spawn, timer, and cancellation separate core policy (decision) from driver execution. The driver owns time observation and physical execution; the core owns expiry / deadline / ordering / bound policy. entrypoints only wire selected implementations and MUST NOT own time policy, entropy semantics, runtime retry semantics, bound policy, or enforcement. This chapter internalizes all state transitions and closed-set vocabulary of each lifecycle, fail-closed conditions, and failure mapping.

---

## 1. Runtime / Clock / Randomness Boundary

### 1.1 Ownership (owner)

The core owns the `ClockPort`, `RandomPort`, and `RuntimePort` boundaries. time, entropy, spawn, timer, and cancellation are separated into core policy and driver execution.

| Surface | core owns | driver owns |
|---|---|---|
| ClockPort | current time abstraction, monotonic comparison input, deadline semantics | system clock / test clock implementation |
| RandomPort | nonce / opaque ID / challenge entropy contract | OS RNG / deterministic test RNG implementation |
| RuntimePort | timer / spawn / cancellation contract | tokio or other runtime execution |

entrypoints wire selected implementations. entrypoints MUST NOT define time policy, entropy semantics, or runtime retry semantics.

### 1.2 Time Authority Rule

The core MUST own expiry and deadline policy. The driver supplies time observation. Cross-node timestamp trust and skew-bounded comparison follow section 7 of this chapter.

TURN allocation expiry, permission expiry, channel bind expiry, Signaling room lifecycle, resource retention, and configuration startup timeouts MUST be evaluated through core policy and cataloged reason. Duration, timestamp, rate window, and deadline units MUST be normalized before policy comparison (section 6 of this chapter).

driver timer delay MAY cause late execution, but it MUST NOT redefine whether core policy considers an item expired.

### 1.3 Randomness Rule

The driver supplies entropy. The core owns the meaning of nonce, opaque ID, challenge, and reference stability.

`RandomPort` output MUST be treated as opaque input to core-owned constructors. The driver MUST NOT encode participant role, tenant, regulated subject, or application user identity into generated randomness. Deterministic test RNG is allowed only in testing scope and MUST NOT be used as production / runtime evidence.

### 1.4 Runtime Rule

`RuntimePort` abstracts timer, spawn, cancellation, and shutdown observation. The runtime implementation MAY schedule execution but MUST NOT own domain policy. The `RuntimePort` task / worker lifecycle, detached task prohibition, supervision scope, join/cancel evidence, and task panic handling are owned by section 2 of this chapter.

If the runtime cannot schedule required driver execution, the path MUST fail through cataloged configuration / resource / driver / shutdown reason (fail-closed). Runtime task cancellation MUST NOT be treated as an accepted domain lifecycle transition unless the relevant core state machine accepted that transition. Runtime panic / crash / restart observation is process lifecycle evidence, not a successful domain transition by itself.

### 1.5 Failure Mapping (runtime / clock)

| Failure | Required reason |
|---|---|
| required runtime configuration missing | `runtime_config_missing` |
| runtime cannot initialize selected driver/entrypoint | `runtime_config_invalid` |
| runtime task class, owner, supervision, spawn, join, cancel, or panic failure | runtime task reason from section 2 of this chapter |
| driver runtime is shutting down | `driver_shutdown` |
| timer/queue resource bound exceeded | matching resource-bound reason (section 8 of this chapter) |
| memory pressure bound exceeded | `memory_pressure_exceeded` |

### 1.6 Prohibitions

- core imports concrete runtime handle.
- driver decides expiry semantics from local timer delay.
- driver compares raw platform time/measurement without normalized unit policy.
- random generator implementation owns identity semantics.
- entrypoints silently substitute system clock, RNG, or runtime default after required configuration is missing.
- deterministic test clock/RNG is used as production-readiness evidence.
- runtime worker state owns SFU route state, TURN allocation state, or Signaling room state.
- detached task or worker supervision is implicit.
- wall-clock timestamp is treated as cross-node causal order without skew trust class.

### 1.7 Collapse Conditions (runtime / clock)

- driver timer implementation defines domain expiry.
- runtime worker owns domain state transition.
- runtime task lifecycle failure is hidden behind generic runtime availability.
- entropy source encodes external identity into core ID.
- core imports tokio/browser/native runtime types.
- runtime failure is ignored while claiming close / complete / ready.
- process crash/restart is treated as recovery success without classification and recovery evidence.
- clock synchronization evidence is inferred from local ClockPort availability.

---

## 2. Runtime Task / Worker Lifecycle

### 2.1 Boundary (owner)

This section fixes owners and evidence granularity so that, within `RuntimePort`, the task / worker lifecycle is not confused with domain state, driver queue, entrypoints supervisor, shutdown, or crash recovery. This section does not assert runtime implementation, tokio task implementation, worker pool implementation, supervisor integration, or runtime readiness.

| Concern | Owner | Rule |
|---|---|---|
| domain state transition | core | does not change from task completion / cancellation / panic observation alone |
| RuntimePort task contract | core | core-owned contract for spawn/cancel/join observation |
| concrete task handle / join handle | driver/entrypoints runtime | does not surface into core public API / domain state |
| driver I/O worker | driver | socket / packet / sink / persistence / exporter execution only |
| entrypoints supervision task | entrypoints | process/component lifecycle observation only |
| task queue / mailbox | owner of physical execution | bounded resource; not ordering authority by itself |
| task panic observation | entrypoints/driver runtime | process or task lifecycle evidence; not domain recovery |
| task cancellation | driver/entrypoints observation, core decision when command already entered core | follows retry/timeout/cancellation rule (section 3 of this chapter) |

Runtime task is not a domain aggregate owner. Runtime task completion is not a substitute for an accepted domain decision.

### 2.2 Task Classes (closed set)

The v0.2 initial architecture task classes are limited to the following.

| Class | Meaning | Rule |
|---|---|---|
| `no_runtime_task` | target path does not require runtime task | no task evidence claim |
| `driver_io_worker` | network/socket/protocol driver worker | cannot own domain decision |
| `driver_packet_worker` | packet queue/cache/rewrite/forward execution worker | packet byte ownership remains driver-local |
| `driver_sink_worker` | audit/metrics/persistence/export sink worker | cannot replace audit/event meaning |
| `entrypoints_supervision_task` | entrypoints-level process/component supervisor task | not readiness/recovery proof by itself |
| `runtime_timer_task` | timer/deadline callback execution | timer execution is not expiry policy |
| `test_runtime_task` | deterministic or fake task execution for tests | test evidence only |
| `detached_task_requested` | task has no admitted parent/supervision scope | rejected unless a future specification admits exact class |

A new task class requires a specification update.

### 2.3 Ownership Rule

Every runtime task that affects a claim MUST declare: task class; parent component or supervision scope; owning layer; allowed input reference types; allowed output observation; cancellation propagation rule; join/wait bound; queue/mailbox bound when applicable; panic/failure mapping; audit event relation.

Core MAY receive opaque schedule/cancellation/result observations through `RuntimePort`. Core MUST NOT receive concrete task handles, runtime handles, join handles, worker queue objects, task-local caches, or task-local locks as domain state. Drivers MAY own concrete task handles only as implementation detail. Entrypoints MAY wire and observe tasks, but entrypoints MUST NOT turn task completion into domain acceptance.

### 2.4 Supervision Rule

Detached task execution is prohibited in v0.2 initial architecture. Every task MUST be under one of: entrypoint startup/run supervision; driver component supervision; bounded test harness supervision; explicit RuntimePort schedule/cancel scope.

If the parent scope ends, the task MUST either: complete inside a bounded drain/join window; cancel with cataloged reason; or fail and record cataloged task lifecycle evidence.

Restarting a task or process does not prove domain recovery.

### 2.5 Cancellation / Shutdown Relation

Task cancellation MUST preserve the boundary where cancellation occurs.

| Cancellation surface | Required relation |
|---|---|
| before driver/core conversion | driver-local cancellation; no domain mutation |
| after command entered core | prior core decision evidence remains authoritative |
| during shutdown/drain | follows cross-plane shutdown/drain rule |
| during driver queue/cache execution | driver failure/shutdown/resource reason; domain decision is not rewritten |
| during test harness timeout | testing evidence records timeout/cancel class |

Task cancellation MUST NOT be reported as accepted leave, room close, endpoint removal, allocation release, audit flush success, or packet forwarding success unless the relevant specification admits that domain result.

### 2.6 Resource Bound Rule

Task queue, worker mailbox, join wait, cancellation wait, and supervision restart attempts MUST be bounded. When the bound participates in resource evidence, it follows section 8 of this chapter. If a task class introduces a new bounded resource not covered in section 8, the resource bound rule and reason catalog MUST be updated before implementation.

### 2.7 Failure Mapping (runtime task)

| Failure | Required reason |
|---|---|
| task class is not admitted | `runtime_task_class_not_admitted` |
| task owner/supervision scope is invalid | `runtime_task_owner_violation` |
| task has no admitted supervision scope | `runtime_task_supervision_missing` |
| detached task is requested | `runtime_task_detached_not_allowed` |
| runtime cannot spawn required task | `runtime_task_spawn_failed` |
| task join/wait observation failed | `runtime_task_join_failed` |
| task cancellation failed or could not be observed | `runtime_task_cancel_failed` |
| task panic was observed | `runtime_task_panic_detected` |
| task queue/mailbox/join bound exceeded | `runtime_task_queue_bound_exceeded` |
| driver/runtime is shutting down | `driver_shutdown` |

### 2.8 Audit Rule (runtime task)

Runtime task / worker lifecycle decisions use audit event type `runtime_task_lifecycle_decision`. The event MUST carry `StartupRunId`, task class, parent component/supervision scope, owning layer, task reference when materialized, cancellation/join bound when applicable, and `CorrelationId` when command-scoped.

### 2.9 Evidence Rule (runtime task)

Runtime task / worker lifecycle evidence MUST record: task class; owning layer; parent component or supervision scope; RuntimePort relation when core observes the task; command/procedure; working directory; correlation ID when command-scoped; startup run ID when process-scoped; spawn/cancel/join/panic outcome; bounded wait/queue/restart policy; cataloged reason for non-success; close-not-claimed scope. Task execution logs are diagnostic only unless they carry the fields above.

### 2.10 Prohibitions (runtime task)

- concrete runtime task handle appears in core public API or domain state.
- detached task is spawned without admitted supervision scope.
- worker completion is treated as domain decision.
- task cancellation rewrites prior accepted/rejected domain decision.
- task panic is treated as graceful shutdown or recovery success.
- unbounded task queue, mailbox, join wait, or restart loop is allowed.
- entrypoints supervisor restart is used as runtime readiness or domain restore evidence.
- driver worker owns Signaling room, SFU route, TURN allocation, or audit chain semantics.

---

## 3. Retry / Timeout / Cancellation

### 3.1 Boundary (owner)

This section fixes owners and result shapes so that command retryability, driver operation retry, deadline exceedance, and client/entrypoint/runtime cancellation do not let state, audit, or external response fail open. This section does not assert retry implementation, scheduler implementation, or runtime cancellation success.

| Concern | Owner | Rule |
|---|---|---|
| retryability metadata | core reason catalog | category default and code override |
| domain command idempotency | core | duplicate and replay semantics |
| driver operation retry execution | driver | bounded retry only |
| command deadline policy | core | typed policy and monotonic time observation |
| timer/scheduler execution | driver | runtime detail through RuntimePort |
| client cancellation observation | driver/sdk | external signal conversion |
| entrypoint shutdown cancellation | entrypoints initiates, core/drivers classify | domain lifecycle and driver shutdown remain separate |

Retry/cancel/timeout MUST NOT create a second reason vocabulary.

### 3.2 Retry Classes (closed set)

| Retry class | Owner | Rule |
|---|---|---|
| `no_retry` | core | non-idempotent or policy-rejected command must not retry automatically |
| `idempotent_command_replay` | core | same correlation/command identity and idempotency rule required |
| `driver_transport_retry` | driver | send/receive operation retry, bounded and audited when failure affects decision |
| `persistence_retry` | driver | bounded retry store, governed by persistence/resource rule |
| `audit_sink_retry` | driver | bounded sink retry, not a substitute for audit evidence |
| `test_only_retry` | testing scope | cannot be used as runtime/prod evidence |

A new retry class requires a specification update.

### 3.3 Timeout / Deadline Rule

Timeout is an operation boundary. Deadline is a policy boundary.

| Timeout/deadline surface | Owner | Failure reason |
|---|---|---|
| command deadline | core | `operation_deadline_exceeded` |
| driver send/receive timeout | driver converted to core reason | `network_send_failed` or `network_receive_failed` |
| persistence retry duration | driver bounded execution | `persistence_retry_duration_exceeded` |
| packet/cache retention duration | driver bounded execution | `retention_duration_exceeded` |
| runtime shutdown during scheduled action | driver/entrypoints observation | `driver_shutdown` |

Driver timer delay does not redefine core expiry semantics.

### 3.4 Cancellation Rule

Cancellation MUST be classified by where it happens.

| Cancellation source | Boundary | Required handling |
|---|---|---|
| client cancels before driver/core boundary | driver/sdk | no domain state mutation; external cancellation response may be local |
| client cancels after command entered core | core decision or follow-up event | must preserve already accepted/rejected decision evidence |
| entrypoint starts shutdown | entrypoints + core/drivers | follows cross-plane shutdown/drain rule |
| runtime task cancelled | driver | converted to `operation_cancelled` or `driver_shutdown` |
| test harness cancels | testing scope | evidence report must classify as timed-out/cancelled test outcome |

Cancellation MUST NOT be reported as successful leave, room close, endpoint removal, or allocation release unless the relevant command was accepted by the core state machine.

### 3.5 Retry Preconditions

Retry is allowed only when all conditions hold: reason metadata marks the failure retryable or the specification allows retry; command is idempotent or retry is driver-local before domain acceptance; retry bound is declared; retry duration is declared; correlation and original command identity are preserved; audit/event evidence can distinguish first attempt and retry attempt; retry does not cross privacy/redaction boundary with raw sensitive data. If any condition fails, retry is not allowed.

### 3.6 Failure Mapping (retry / timeout / cancellation)

| Failure | Required reason |
|---|---|
| core command deadline exceeded | `operation_deadline_exceeded` |
| operation cancelled after boundary | `operation_cancelled` |
| retry store bound exceeded | `persistence_retry_bound_exceeded` |
| retry duration exceeded | `persistence_retry_duration_exceeded` |
| network send retry exhausted | `network_send_failed` |
| network receive retry exhausted | `network_receive_failed` |
| audit backlog bound reached | `audit_backlog_bound_exceeded` |
| runtime or driver shutdown | `driver_shutdown` |
| runtime task lifecycle failure | runtime task reason from section 2 of this chapter |

### 3.7 Prohibitions / Collapse Conditions (retry / timeout / cancellation)

Prohibitions: automatic retry of non-idempotent domain command; driver retry changes domain decision; cancellation treated as accepted domain lifecycle transition without core decision; timeout reported as generic success or free-text failure; unbounded retry loop or unbounded retry store; test-only retry behavior used as runtime evidence; retry erases original correlation or first-attempt evidence; runtime task cancellation treated as successful domain lifecycle transition.

Collapse: retryability is inferred from driver exception text; command deadline has no cataloged reason; cancellation after core entry loses prior decision evidence; retry bound or retry duration is absent; timeout/cancel behavior differs per driver without a specification update; task cancel/join failure is hidden while claiming cancellation evidence.

---

## 4. Concurrency / Ordering / Lock Ownership

### 4.1 Boundary (owner)

This section fixes serialize scope and failure mapping so that concurrent commands to the same aggregate / resource do not change domain semantics through driver-local lock or runtime scheduling. This section does not assert lock implementation, actor implementation, or runtime scheduler implementation.

| Concern | Owner | Rule |
|---|---|---|
| domain ordering validity | core | state machine and command semantics |
| aggregate serialization scope | core | which commands conflict on same state |
| physical mutex / channel / actor mailbox | driver or core implementation detail according to package layer | not domain source-of-truth |
| runtime scheduling | driver | execution timing, not ordering authority |
| runtime task lifecycle | driver/entrypoints runtime | supervision/cancel/join evidence, not ordering authority |
| inbound wire ordering observation | driver | preserves observations, does not decide validity |
| idempotency | core | duplicate and replay semantics |

Lock state is not domain state. Runtime execution order is not automatically valid command order.

### 4.2 Serialization Scopes (closed set)

| Scope | Owner | Protected semantics |
|---|---|---|
| `room_scope` | core/signaling | room state, participant membership, command idempotency |
| `participant_scope` | core/signaling | participant lifecycle within room |
| `sfu_session_scope` | core/sfu | session admission and route lifecycle |
| `sfu_endpoint_scope` | core/sfu | endpoint publication/subscription lifecycle |
| `packet_lifecycle_scope` | driver | packet buffer lease, queue, cache, release |
| `turn_allocation_scope` | core/turn | allocation lifecycle and refresh |
| `turn_permission_scope` | core/turn | permission and relay authorization |
| `audit_chain_scope` | core/audit | hash-chain ordering |
| `configuration_scope` | core/entrypoints boundary | startup/wiring validation sequence |

A new serialization scope requires a specification update.

### 4.3 Ordering Rule

The core owns the decision whether a command is valid for current state. The driver MAY deliver commands in observed order, but MUST NOT use arrival order as the only domain validity rule.

Ordering failures MUST map to: `command_order_violation` for protocol command order failure; `duplicate_command` for idempotency duplicate rejection; `concurrency_conflict` when concurrent accepted candidates conflict on the same serialization scope; `lock_contention_bound_exceeded` when bounded serialization queue/lock admission is exhausted.

### 4.4 Lock Ownership Rule

| Lock/resource | Allowed owner | Rule |
|---|---|---|
| aggregate mutation guard | core implementation detail | protects core state, does not expose lock type in API |
| driver queue/mutex | driver | protects external I/O/resource, does not own domain ordering |
| entrypoint-level process lock | entrypoints | startup/process guard only |
| test harness synchronization | testing scope | not production semantics |

Lock handle, actor mailbox, runtime task handle, or queue object MUST NOT appear in core public API or domain state. Task class and supervision scope MUST NOT redefine serialization scope.

### 4.5 Bound Rule / Audit Rule

Every serialization queue or lock wait with unbounded growth risk MUST define: scope; maximum pending commands or wait duration; owner; failure reason; audit event mapping; cancellation behavior. When the bound is exceeded, use `lock_contention_bound_exceeded` unless a more specific resource reason applies.

Concurrency/ordering failures that reject or drop command execution MUST emit audit/event evidence using the relevant plane event type or `resource_bound_decision`. `lock_contention_bound_exceeded` uses `resource_bound_decision` with resource policy owner `core` and physical resource owner matching the implementation owner.

### 4.6 Prohibitions / Collapse Conditions (concurrency)

Prohibitions: driver lock acquisition order defines domain command order; runtime scheduler order replaces state machine precondition; unbounded aggregate command queue; lock object becomes domain model or public API; race conflict resolved by last-writer-wins without core decision; test synchronization behavior treated as production semantics; task scheduling order treated as serialization authority.

Collapse: domain ordering validity inferred from driver arrival order only; lock contention can grow without bound; concurrent conflict has no cataloged reason; physical lock state appears in core semantic result; serialization scope differs by entrypoint binary without a specification update; task lifecycle or worker queue ownership changes domain ordering semantics.

---

## 5. Atomicity / Transaction / Compensation

### 5.1 Boundary (owner)

This section fixes owners and result classification so that, when only part of core decision, domain event, audit persistence, external response, or driver execution succeeds, success treatment, compensation, and evidence treatment are not misjudged. This section does not assert database transaction implementation, outbox implementation, or compensation implementation.

| Concern | Owner | Rule |
|---|---|---|
| domain decision atomicity | core | aggregate transition and decision outcome |
| port intent emission | core | driver execution request, not execution success |
| concrete DB transaction | driver | storage implementation detail |
| audit persistence execution | driver | bounded sink/persistence detail |
| external response emission | driver/sdk | projection and send execution |
| compensation decision | core when domain state changes; driver for external resource cleanup | owner must be explicit |
| evidence treatment | evidence | partial success cannot be close evidence without classification |

Atomicity boundary MUST be named before implementation.

### 5.2 Atomicity Classes (closed set)

| Class | Meaning | Rule |
|---|---|---|
| `single_core_decision` | one core decision without external side effect claim | decision is atomic in core state only |
| `core_decision_plus_audit_required` | domain decision requires audit evidence | audit failure blocks close evidence |
| `core_decision_plus_port_intent` | driver execution follows accepted decision | port intent is not execution success |
| `driver_local_transaction` | DB/file/network resource transaction | driver-owned; does not define domain invariant |
| `compensating_transition_required` | accepted state needs follow-up compensation | must be defined by the specification/state machine |
| `non_compensable_observation` | observation cannot be undone | must be reported as observation, not rollback |

A new atomicity class requires a specification update.

### 5.3 Commit Boundary Rule

Every command path MUST identify the commit boundary: before core entry; after core validation but before aggregate mutation; after aggregate transition; after audit projection; after driver persistence; after external response emission. If the command path crosses more than one boundary, each step needs outcome and failure handling. Partial success is prohibited unless per-step outcomes and compensation are defined.

### 5.4 Compensation Rule

Compensation is not rollback by default. Compensation MUST declare: triggering failure; affected state/event; owner; allowed compensating transition; audit event relation; external response rule; audit evidence relation. If no compensation rule exists, the system MUST preserve the original accepted decision and record the later failure as driver observation or evidence limitation.

### 5.5 Failure Mapping / Evidence Rule (atomicity)

| Failure | Required reason |
|---|---|
| atomic commit step failed | `atomic_commit_failed` |
| compensation required but not available | `compensation_required` |
| compensation execution failed | `compensation_failed` |
| persistence unavailable during commit | `persistence_unavailable` |
| audit backlog prevents required audit | `audit_backlog_bound_exceeded` |
| external response encoding failed | `external_encode_failed` |
| external response send failed | `network_send_failed` |
| driver shutdown during commit/compensation | `driver_shutdown` |

Closeout evidence MUST identify: atomicity class; commit boundary; per-step outcomes; compensation status when applicable; audit/persistence/external response failures; close-not-claimed scope. Evidence that omits partial success classification cannot be used for close / complete / ready claims (fail-closed).

### 5.6 Prohibitions / Collapse Conditions (atomicity)

Prohibitions: driver DB transaction defines domain invariant; port intent treated as driver execution success; external response success treated as audit persistence success; compensation mutates state without core/state-machine rule; partial success hidden behind final success; failed audit/persistence step used as close evidence.

Collapse: command path has multiple side effects without commit boundary; accepted domain decision is erased by driver failure without compensating transition; compensation owner is implicit; evidence lacks per-step outcome; database transaction is treated as core aggregate authority.

---

## 6. Unit / Measurement / Time Normalization

### 6.1 Boundary (owner)

This section aligns duration, deadline, rate window, bytes, packet count, quality metrics, benchmark values, and clock source into a comparable core-owned representation so that driver/platform differences and notation variance do not change policy decisions. This section does not assert concrete thresholds, benchmark results, or runtime clock accuracy.

| Concern | Owner | Rule |
|---|---|---|
| normalized unit type | core | unit and rounding rule used for policy/decision |
| raw platform measurement | driver | OS/runtime/library specific observation |
| benchmark reporting unit | benchmark docs/reports | distinguishes normalized unit and raw observation |
| wall-clock timestamp | driver/entrypoints supplied, core validated as observation | not ordering authority |
| monotonic duration/deadline | core policy via ClockPort observation | expiry/deadline decision |
| metric export format | driver | external labels/format are not policy |

Drivers MAY collect raw values but MUST convert them before core policy evaluation.

### 6.2 Normalized Units (closed set)

| Quantity | Normalized unit | Rule |
|---|---|---|
| duration | milliseconds as integer | monotonic duration for policy/deadline |
| timestamp | UTC epoch milliseconds for evidence only | not domain ordering authority by itself |
| bytes | bytes as integer | binary size / buffer / frame bounds |
| packet count | integer count | packet/cache/queue bounds |
| rate | units per second with explicit numerator | no implicit time window |
| ratio | rational or fixed decimal with declared precision | no hidden float comparison |
| bitrate | bits per second | distinguish from bytes per second |
| jitter / RTT | milliseconds as integer or declared precision | source and window required |

A new unit requires a specification update.

### 6.3 Time Source Rule

Core policy decisions involving expiry, deadline, timeout, retention, and rate window MUST use monotonic time observation through `ClockPort`. Wall-clock timestamp is allowed for audit/evidence display, not for ordering by itself.

Clock skew/drift handling MUST define: source; maximum accepted skew when applicable; whether wall-clock is evidence-only or policy input; failure reason when time observation is invalid; evidence reporting rule. Cross-node skew admission and time trust class are defined by section 7 of this chapter.

### 6.4 Rounding / Comparison Rule

Measurement comparison MUST define: normalized unit; precision; rounding direction; comparison operator; inclusive/exclusive boundary; sampling window; owner of raw measurement; owner of policy decision. Driver/exporter formatting MUST NOT change comparison result.

### 6.5 Failure Mapping / Evidence Rule (unit)

| Failure | Required reason |
|---|---|
| measurement cannot be normalized | `measurement_normalization_failed` |
| required time observation unavailable | `time_observation_unavailable` |
| operation deadline exceeded | `operation_deadline_exceeded` |
| retention duration exceeded | `retention_duration_exceeded` |
| runtime configuration invalid for time/measurement source | `runtime_config_invalid` |
| driver shutdown during measurement | `driver_shutdown` |

Reports MUST record both raw source when useful and normalized value when used for claim. Benchmark and quality reports MUST state unit, window, precision, and aggregation method. Evidence without unit/window cannot support performance, capacity, or quality claims (fail-closed).

### 6.6 Prohibitions / Collapse Conditions (unit)

Prohibitions: driver-exported metric label becomes policy unit; wall-clock order replaces monotonic deadline/expiry policy; float comparison uses hidden precision; bitrate and byte-rate are conflated; benchmark report omits unit/window/aggregation; raw platform-specific stats object enters core policy; timezone conversion is treated as synchronization evidence.

Collapse: policy threshold lacks normalized unit; driver/platform measurement format changes core decision; clock source is implicit; evidence compares values with different unit/window; wall-clock timestamp is used as sole ordering authority; cross-node timestamp comparison lacks time trust class or skew policy.

---

## 7. Time Synchronization / Clock Skew / Timestamp Trust

### 7.1 Boundary (owner)

This section fixes the time trust boundary when comparing multiple nodes, multiple processes, external observations, and audit/report timestamps. Unit normalization follows section 6 of this chapter. time source is observed by the driver/runtime. The core owns expiry, deadline, ordering tolerance, skew allowance, and timestamp trust policy. entrypoints wire the selected clock/time-source implementation but do not own time trust policy.

| Concern | Owner | Rule |
|---|---|---|
| local monotonic duration | ClockPort / core policy | deadline and elapsed duration use monotonic comparison when possible |
| wall-clock timestamp | driver observation / audit model | report/audit timestamp, not causal authority by default |
| cross-node skew policy | core / topology specification | max skew and trust class required |
| external time source | driver / deployment | NTP/PTP/cloud metadata etc. are observation sources only |
| timestamp normalization | unit/time rule (section 6 of this chapter) | precision, window, timezone, unit required |
| evidence timestamp claim | evidence rule | command time and observation time must be separated |

### 7.2 Time Trust Classes (closed set)

| Time trust class | Meaning | Rule |
|---|---|---|
| `single_process_monotonic` | one process monotonic elapsed comparison | can prove local duration/deadline only |
| `single_node_wall_clock` | one node wall-clock timestamp | can label observation time but not cross-node causal order |
| `multi_node_bounded_skew` | nodes have measured skew within policy | can support bounded cross-node comparison |
| `external_trusted_time_source` | accepted external time source observation | must record source class and failure mode |
| `test_deterministic_clock` | deterministic test clock | test evidence only, not production/runtime evidence |
| `time_untrusted` | time source cannot be trusted for target claim | target claim must fail or be reduced |

A new time trust class requires a specification update.

### 7.3 Skew Policy Rule

Any cross-node or cross-process timestamp comparison MUST declare: node scope; time trust class; max accepted skew; observation precision; measurement window; source of skew observation; failure reason when skew cannot be measured; impact on expiry, ordering, audit, and evidence claims. If bounded skew is required and cannot be observed, the claim MUST NOT be adopted (fail-closed).

### 7.4 Ordering / Expiry / Deadline Rule

Wall-clock timestamp alone does not prove causal order across nodes. Core ordering decisions MUST use correlation, sequence, idempotency, aggregate version, protocol state, or explicit bounded-skew policy. Audit/report ordering MAY use timestamp for presentation only unless hash-chain sequence, event sequence, or bounded-skew evidence is recorded.

Expiry/deadline policy remains core-owned. Driver time observation MAY be late, unavailable, or skewed, but MUST NOT redefine the policy. Token expiry, TURN lifetime, ICE consent, session resumption, retention, retry timeout, and public connection idle timeout MUST record whether the comparison used monotonic duration, wall-clock timestamp, or bounded-skew class.

### 7.5 Failure Mapping (time synchronization)

| Failure | Required reason |
|---|---|
| measured skew exceeds policy | `clock_skew_exceeded` |
| time source is not trusted for the target claim | `time_source_untrusted` |
| required time synchronization observation is unavailable | `time_sync_unavailable` |
| timestamp order cannot be trusted for target comparison | `timestamp_order_untrusted` |

Existing local time observation failure MAY still use `time_observation_unavailable` when synchronization/skew is not the target boundary.

### 7.6 Evidence / Audit Rule (time synchronization)

Time synchronization evidence MUST record node scope, time trust class, max skew, observed skew, precision, measurement window, time source class, command time, observation time, and rerun condition. JST report timestamp alone is report metadata and does not prove runtime clock synchronization.

Time synchronization decisions use audit event type `time_synchronization_decision`. The event MUST carry node scope, time trust class, time source class, skew policy reference, observed skew class, `StartupRunId`, and `CorrelationId` when command-scoped.

### 7.7 Prohibitions / Collapse Conditions (time synchronization)

Prohibitions: wall-clock timestamp used as cross-node causal order without bounded-skew evidence; report creation time treated as runtime observation time; deterministic test clock used as production time synchronization evidence; driver-local NTP status redefines core expiry/deadline policy; timezone conversion treated as synchronization proof; missing skew observation accepted as within-bound observation.

Collapse: time trust class is absent; skew policy is open-ended or unmeasured for cross-node claim; wall-clock order is treated as causal order without additional evidence; expiry/deadline owner moves from core policy to driver timer; evidence hides node scope, precision, or measurement window.

---

## 8. Resource Bounds / Backpressure

### 8.1 Principle / Ownership

unbounded resource is prohibited. queue, cache, buffer pool, retry store, audit sink backlog, packet retention, connection admission, and serialization queue / lock wait MUST have a bound and a closed action. Runtime task queue / worker mailbox / join wait bounds follow section 2 of this chapter and this section when adopted as resource-bound evidence.

| Concern | Policy owner | Physical resource owner | Measurement owner | Execution owner |
|---|---|---|---|---|
| admission bound | core | driver | driver | core decision, driver execution |
| packet cache bound | core | driver | driver | driver |
| transmit queue bound | core | driver | driver | driver |
| audit backlog bound | core | driver | driver | driver |
| persistence retry bound | core | driver | driver | driver |
| metrics export backlog | core | driver | driver | driver |
| driver receive buffer pool | core | driver | driver | driver |
| connection concurrency bound | core | driver | driver | driver |
| memory pressure bound | core | driver | driver | core decision, driver execution |
| serialization queue / lock wait bound | core | core or driver implementation detail | owner of physical queue/lock | core decision, physical owner execution |
| runtime task queue / worker mailbox / join wait | core or driver according to task owner | driver/entrypoints runtime | driver/entrypoints runtime | task lifecycle decision, physical owner execution |
| inbound frame size bound | driver | driver | driver | driver conversion |

entrypoints do not own bound policy or enforcement. entrypoints only wire the selected driver and typed configuration.

### 8.2 Bound Shape

Each bounded resource defines: resource name; owner; maximum units; maximum bytes if applicable; maximum duration if applicable; pressure observation type; decision reason from reason catalog; drop / suppress / reject / retry / shed action; audit requirement.

### 8.3 Backpressure Decisions (closed set)

The backpressure decisions owned by the core are limited to the following, and each decision connects to an audit outcome and reason.

| Backpressure decision | Audit event type | Outcome | Reason code |
|---|---|---|---|
| accept | `backpressure_decision` | `accepted` | not required for success |
| delay action | `backpressure_decision` | `delayed` | `action_delayed_by_backpressure` |
| suppress forwarding | `backpressure_decision` | `suppressed` | `packet_suppressed_by_backpressure` |
| suppress subscription | `sfu_subscription_decision` | `suppressed` | `subscription_backpressure_suppressed` |
| suppress route state | `backpressure_decision` | `suppressed` | `route_suppressed_by_backpressure` |
| drop packet | `backpressure_decision` | `dropped` | `packet_dropped_by_backpressure` |
| stop retaining packet because of pressure policy | `backpressure_decision` | `dropped` | `packet_dropped_by_backpressure` |
| degrade route | `backpressure_decision` | `degraded` | `route_degraded_by_backpressure` |
| close endpoint | `backpressure_decision` | `closed_by_policy` | `endpoint_closed_by_backpressure` |
| reject recovery from backpressure-delayed, backpressure-degraded, or backpressure-suppressed route state | `backpressure_decision` | `rejected` | `backpressure_recovery_not_allowed` |

Admission rejection caused by capacity, endpoint capacity, or connection concurrency is resource-bound decision, not a distinct backpressure decision in v0.2 initial canonical. Memory pressure uses resource-bound shedding only in v0.2 initial canonical and does not define a separate admission rejection path. Subscription suppression caused by backpressure uses `sfu_subscription_decision`; packet and route backpressure actions use `backpressure_decision`. Packet cache eviction or packet retention expiry caused by a bound is resource-bound decision, not "stop retaining packet because of pressure policy".

The driver executes the decision. The driver does not own policy authority.

### 8.4 Required Bounds (closed set)

| Resource | Required bound | Bound exceeded reason code |
|---|---|---|
| active room set | maximum active rooms / maximum materializations per window | `room_capacity_exceeded` |
| room lifecycle | maximum room lifetime / maximum idle duration | `room_lifetime_exceeded` |
| signaling command queue | maximum commands / maximum wait duration | `signaling_command_queue_bound_exceeded` |
| room participant set | maximum participants | `admission_capacity_exceeded` |
| SFU endpoint admission | maximum admitted endpoints / maximum endpoint admission window | `endpoint_capacity_exceeded` |
| SFU packet cache | maximum packets / bytes / retention duration | `packet_cache_bound_exceeded`, `retention_duration_exceeded` |
| SFU transmit queue | maximum packets / bytes / wait duration | `sfu_transmit_queue_bound_exceeded` |
| SFU route candidates | maximum candidates per decision | `route_candidate_bound_exceeded` |
| TURN allocation table | maximum allocations | `allocation_capacity_exceeded` |
| TURN allocation lifetime | maximum lifetime | `allocation_lifetime_exceeded` |
| TURN refresh cap | maximum refresh count / maximum cumulative refresh duration | `refresh_limit_exceeded` |
| TURN permission table | maximum permissions per allocation | `permission_capacity_exceeded` |
| TURN permission lifetime | maximum permission lifetime | `permission_lifetime_exceeded` |
| TURN channel bind lifetime | maximum channel binding lifetime | `channel_bind_lifetime_exceeded` |
| TURN relay queue | maximum packets / bytes / wait duration | `turn_relay_queue_bound_exceeded` |
| audit sink backlog | maximum events / bytes / retry duration | `audit_backlog_bound_exceeded`, `retention_duration_exceeded` |
| persistence retry store | maximum retry entries / bytes / retry count / retry duration | `persistence_retry_bound_exceeded`, `persistence_retry_duration_exceeded` |
| metrics export backlog | maximum events / bytes | `metrics_backlog_bound_exceeded` |
| driver receive buffer pool | maximum leases / bytes | `buffer_pool_bound_exceeded` |
| inbound frame size | maximum bytes per frame / message | `frame_size_bound_exceeded` |
| connection concurrency | maximum concurrent connections / admission window | `connection_concurrency_exceeded` |
| memory pressure | maximum memory pressure state / pressure duration | `memory_pressure_exceeded` |
| aggregate serialization queue / lock wait | maximum pending commands / maximum wait duration per serialization scope | `lock_contention_bound_exceeded` |
| runtime task queue / worker mailbox / join wait | maximum pending tasks / maximum wait duration / maximum cancellation wait | `runtime_task_queue_bound_exceeded` |

Each required bound's owner tuple is based on the same (policy owner=core, physical=driver) as the section 8.1 table, except that for serialization queue the physical owner is the owner of the physical queue/lock, for runtime task queue the physical owner is the driver/entrypoints runtime, and for inbound frame size the policy owner=driver / physical=driver.

### 8.5 Fail-Closed Rule

bound exceeded MUST map to a closed reason (fail-closed). silent unbounded growth is prohibited. The correspondence of reason category and code is as follows (each bound uses the bound-specific code in section 8.4): room capacity / signaling queue / transmit queue / relay queue / admission capacity / endpoint capacity / route candidate / audit backlog / persistence retry store / metrics backlog / buffer pool / frame size / connection concurrency / memory pressure / lock contention / runtime task queue use reason category `resource_exhausted`. room lifetime / retention duration / allocation lifetime / permission lifetime / channel bind lifetime / refresh limit / persistence retry duration use reason category `expired`. Each backpressure action uses reason category `backpressure`. Generic capacity failure MUST NOT be emitted as a reason code. Each bounded resource MUST use the bound-specific reason code in section 8.4.

### 8.6 Audit Event Mapping Rule

The bound-related audit event type is fixed as follows. Even when a Signaling / SFU / TURN decision event is added with the same correlation ID, it MUST NOT substitute for the bound audit event. A non-mapped Signaling / SFU domain decision event carrying the same bound reason is supplementary and does not satisfy the owner field requirement of the mapped bound audit event.

| Bound family | Audit event type code |
|---|---|
| backpressure delay / packet suppression / packet drop / packet retention stop / route suppression / route degradation / endpoint close / recovery rejection | `backpressure_decision` |
| subscription suppression by backpressure | `sfu_subscription_decision` |
| room materialization / room lifecycle / signaling command queue / participant admission capacity | `resource_bound_decision` |
| SFU packet cache / SFU transmit queue / SFU route candidate bound / memory pressure | `resource_bound_decision` |
| TURN allocation capacity / allocation lifetime | `turn_allocation_decision` |
| TURN refresh cap | `turn_refresh_decision` |
| TURN permission capacity / permission lifetime | `turn_permission_decision` |
| TURN channel bind lifetime | `turn_channel_bind_decision` |
| TURN relay queue | `resource_bound_decision` |
| audit sink backlog / persistence retry store / metrics export backlog | `resource_bound_decision` |
| aggregate serialization queue / lock wait | `resource_bound_decision` |
| runtime task queue / worker mailbox / join wait | `runtime_task_lifecycle_decision` |
| driver receive buffer pool / connection concurrency | `resource_bound_decision` |
| inbound frame size | `driver_resource_bound_decision` |
| explicitly listed driver-local resource bound | `driver_resource_bound_decision` |

Inbound frame size bound exceeded uses audit event type `driver_resource_bound_decision`, outcome `dropped`, reason `frame_size_bound_exceeded`, resource policy owner `driver`, and physical resource owner `driver`. In v0.2 initial canonical, `driver_resource_bound_decision` is limited to inbound frame size unless another driver-local resource bound is explicitly added. SFU transmit queue bound that drops packet lifecycle uses audit event type `resource_bound_decision`, outcome `dropped`, reason `sfu_transmit_queue_bound_exceeded`, and packet release reason `dropped_by_transmit_queue_bound`. TURN allocation / refresh cap / permission / channel bind bound rows use TURN-specific decision event types but remain resource-bound related audit events and MUST carry resource policy owner `core` and physical resource owner `driver`. TURN relay queue bound uses `resource_bound_decision` and MUST carry the active `AllocationId` and `PermissionId` for the relay path in addition to resource name, resource policy owner, and physical resource owner.

### 8.7 Required Bound Closed Action Mapping (closed set)

| Resource | Closed action | Audit event type | Outcome | Reason code |
|---|---|---|---|---|
| active room set | reject room materialization | `resource_bound_decision` | `rejected` | `room_capacity_exceeded` |
| room lifecycle | expire or close room lifecycle | `resource_bound_decision` | `expired` | `room_lifetime_exceeded` |
| signaling command queue | reject queued command admission | `resource_bound_decision` | `rejected` | `signaling_command_queue_bound_exceeded` |
| room participant set | reject participant admission | `resource_bound_decision` | `rejected` | `admission_capacity_exceeded` |
| SFU endpoint admission | reject endpoint admission | `resource_bound_decision` | `rejected` | `endpoint_capacity_exceeded` |
| SFU packet cache | drop or evict packet retention | `resource_bound_decision` | `dropped` | `packet_cache_bound_exceeded` |
| SFU packet cache | expire packet retention | `resource_bound_decision` | `expired` | `retention_duration_exceeded` |
| SFU transmit queue | drop packet enqueue / scheduling | `resource_bound_decision` | `dropped` | `sfu_transmit_queue_bound_exceeded` |
| SFU route candidates | reject route candidate construction | `resource_bound_decision` | `rejected` | `route_candidate_bound_exceeded` |
| TURN allocation table | reject allocation | `turn_allocation_decision` | `rejected` | `allocation_capacity_exceeded` |
| TURN allocation lifetime | expire allocation | `turn_allocation_decision` | `expired` | `allocation_lifetime_exceeded` |
| TURN refresh cap | expire allocation refresh path | `turn_refresh_decision` | `expired` | `refresh_limit_exceeded` |
| TURN permission table | reject permission | `turn_permission_decision` | `rejected` | `permission_capacity_exceeded` |
| TURN permission lifetime | expire permission | `turn_permission_decision` | `expired` | `permission_lifetime_exceeded` |
| TURN channel bind lifetime | expire channel bind | `turn_channel_bind_decision` | `expired` | `channel_bind_lifetime_exceeded` |
| TURN relay queue | drop relay data scheduling | `resource_bound_decision` | `dropped` | `turn_relay_queue_bound_exceeded` |
| audit sink backlog | shed audit backlog and reject new audit-required path | `resource_bound_decision` | `shed` | `audit_backlog_bound_exceeded` |
| audit sink backlog | expire audit retry entry and reject new audit-required path | `resource_bound_decision` | `expired` | `retention_duration_exceeded` |
| persistence retry store | shed retry entry | `resource_bound_decision` | `shed` | `persistence_retry_bound_exceeded` |
| persistence retry store | expire retry entry | `resource_bound_decision` | `expired` | `persistence_retry_duration_exceeded` |
| metrics export backlog | shed metrics export item | `resource_bound_decision` | `shed` | `metrics_backlog_bound_exceeded` |
| driver receive buffer pool | reject receive buffer lease | `resource_bound_decision` | `rejected` | `buffer_pool_bound_exceeded` |
| inbound frame size | drop inbound frame before core entry | `driver_resource_bound_decision` | `dropped` | `frame_size_bound_exceeded` |
| connection concurrency | reject connection admission | `resource_bound_decision` | `rejected` | `connection_concurrency_exceeded` |
| memory pressure | shed lower-priority resource action | `resource_bound_decision` | `shed` | `memory_pressure_exceeded` |
| aggregate serialization queue / lock wait | reject command admission to serialization scope | `resource_bound_decision` | `rejected` | `lock_contention_bound_exceeded` |
| runtime task queue / worker mailbox / join wait | reject spawn/schedule/join continuation | `runtime_task_lifecycle_decision` | `rejected` | `runtime_task_queue_bound_exceeded` |

### 8.8 Audit Backlog Overflow Rule

audit sink backlog exceeded MUST NOT recursively enqueue its own overflow event into the saturated normal audit backlog. When audit sink backlog reaches its bound: (1) emit one `resource_bound_decision` with outcome `shed`, reason `audit_backlog_bound_exceeded`, resource policy owner `core`, and physical resource owner `driver` through a reserved non-recursive overflow record path; (2) reject or stop new audit-required command paths with `audit_backlog_bound_exceeded` until normal audit capacity is available; (3) never treat an audit-required decision as accepted if its required audit event cannot be recorded through the normal backlog or reserved overflow record path. When audit sink backlog retry duration expires: (1) emit one `resource_bound_decision` with outcome `expired`, reason `retention_duration_exceeded`, and the same owner tuple through the same reserved non-recursive overflow record path; (2) mark the affected previously accepted audit-required decision as audit-failed for closeout purposes; (3) prohibit using that affected decision as close / complete / ready evidence.

### 8.9 Prohibitions / Collapse Conditions (resource bounds)

Prohibitions: unbounded queue / cache / retry / packet retention; fallback to best-effort without reason; driver-local drop without core reason mapping; entrypoints overriding core bound policy; task queue or worker mailbox bound hidden behind runtime implementation.

Collapse: introducing a resource without a bound; omitting the runtime task queue / worker mailbox / join wait bound when worker execution affects the claim; driver owns backpressure policy; pressure reason does not connect to reason catalog; capacity exceeded treated as success; allowing unbounded retry or cache.
