# Entrypoints Operations Lifecycle: health/admin, shutdown/drain, crash, authorization

Status: SSOT consolidated edition
Date: 2026-06-28 JST

## Purpose

This chapter specifies, in a fully self-contained form (understandable without opening any other file, source dev-doc, or implementation code), the operations lifecycle boundaries of the arcRTC v0.2 Kernel. It covers four areas: health / readiness / liveness / admin / maintenance; cross-plane shutdown / drain (separation of process lifecycle and domain lifecycle); crash / panic / supervisor restart classification (all classes and restart rules); and operator / admin authorization. The granularity is sufficient for reimplementation from this chapter alone.

It fixes the owner and evidence-acceptance conditions so that operator-facing endpoint / CLI / probe / maintenance action is not confused with domain readiness, runtime readiness, or closeout evidence. Process lifecycle and domain lifecycle are separated.

---

## Part A. Health / Readiness / Liveness / Admin / Maintenance

### A-1 Boundary

| Surface | Owner | Rule |
|---|---|---|
| liveness observation | entrypoints/drivers | observe whether process/runtime can respond |
| readiness decision | entrypoints composition plus core/driver dependency status | composes whether the selected path can accept, but is not a domain decision |
| domain acceptance | core | individual decisions such as join/route/allocation |
| admin command | entrypoints/cli | calls a core use case or driver operation boundary |
| maintenance mode | entrypoints initiates, core/drivers classify | separated from domain state transition |
| probe response encoding | driver/entrypoints | external status projection |
| evidence report | reports | probe result is evidence only with correlation/procedure/scope |

Probe success does not prove domain command success. Listener startup does not prove readiness. Public endpoint lifecycle evidence is separate from probe success.

### A-2 Probe Classes (closed set)

| Probe class | Meaning | Claim limit |
|---|---|---|
| `process_liveness` | process/runtime loop is observable | not dependency readiness |
| `driver_dependency_readiness` | selected driver dependency can initialize/respond | not domain acceptance |
| `core_policy_readiness` | required core policy bundle accepted | not runtime integration |
| `composition_readiness` | selected entrypoint wiring completed | not production readiness |
| `maintenance_status` | entrypoint is in normal/draining/maintenance mode | not domain state proof alone |
| `operator_action_result` | admin/CLI action result | not a substitute for core decision evidence |

A new probe class is out of the v0.2 initial scope.

### A-3 Readiness Composition Rule

Readiness MUST declare which classes it includes. A readiness response MUST NOT be a single unqualified boolean.

Required fields: startup run ID; entrypoint name; profile class when applicable; selected drivers; deployment topology class and node scope when relevant; service discovery source/resolution state when dependency resolution affects readiness; distributed state class/failover admission status when node-local state affects readiness; runtime task class and supervision state when worker execution affects readiness; internal service trust class when service-to-service identity affects readiness; probe class; included dependency checks; excluded checks; outcome; cataloged reason for non-ready or failed outcome; evidence class and close-not-claimed scope.

If any required readiness component is not evaluated, readiness is not satisfied for claims requiring that component.

### A-4 Admin and Maintenance Rule

Admin and maintenance commands MAY: request drain/shutdown through the cross-plane shutdown/drain authority (Part B); inspect redacted state references; trigger bounded verification commands; request driver dependency probes; request audit/hash-chain verification. Admin and maintenance commands requiring privilege MUST pass operator/admin authorization (Part D) before target action execution.

Admin and maintenance commands MUST NOT: mutate core aggregate state outside a core use case/state machine; bypass an authorization/security boundary; expose raw secret/token/packet/regulated payload; turn probe success into domain acceptance; mark closeout complete without an evidence report.

### A-5 Failure Mapping

| Failure | Required reason |
|---|---|
| readiness component not satisfied | `readiness_not_satisfied` |
| health probe cannot execute due to driver/entrypoint failure | `health_probe_unavailable` |
| action blocked by maintenance mode | `maintenance_mode_active` |
| admin action not allowed by boundary/policy | `admin_action_not_allowed` |
| operator/admin authorization denied | `operator_action_denied` |
| operator/admin authorization context missing | `operator_authorization_context_missing` |
| required runtime configuration missing | `runtime_config_missing` |
| runtime configuration invalid | `runtime_config_invalid` |
| driver shutdown during probe/action | `driver_shutdown` |
| service discovery unavailable | `service_discovery_unavailable` |
| endpoint resolution stale or fallback not admitted | `service_endpoint_stale` or `service_endpoint_fallback_not_allowed` |
| node-local state unavailable | `node_state_unavailable` |
| failover state not proven | `failover_not_proven` |
| runtime task supervision/spawn/join/cancel failure affects probe | runtime task reason |
| internal service identity/trust failure affects probe | internal service identity reason |

### A-6 Evidence Rule

Health/readiness/liveness evidence MUST record: command or probe endpoint; working directory or target entrypoint; startup run ID; correlation ID when command-scoped; probe class; included/excluded checks; topology class and node scope when relevant; service discovery source/resolution state when it affects the probe; distributed state class and failover status when it affects the probe; runtime task class/supervision state when it affects the probe; internal service trust class when it affects the probe; expected outcome; actual outcome; cataloged reason for non-success; close-not-claimed scope. Probe output without these fields is diagnostic only.

---

## Part B. Cross-Plane Shutdown / Drain

This part is the shutdown and drain boundary spanning Signaling / SFU / TURN / drivers / entrypoints. It fixes owner and ordering so that process lifecycle, room drain, SFU session drain, TURN relay stop, driver I/O stop, and audit/persistence flush are not confused. Crash / panic / supervisor restart classification follows Part C and is treated as a separate evidence class from graceful drain.

### B-1 Boundary

| Concern | Owner | Rule |
|---|---|---|
| process signal observation | entrypoints | receive OS/process/runtime signal |
| split-service shutdown control | internal control-plane contract | service-to-service drain command/event boundary |
| shutdown mode selection | entrypoints | typed shutdown request and wiring to selected drivers |
| room drain/close semantics | core/signaling | room state transition and rejection reason |
| SFU session/endpoint lifecycle semantics | core/sfu | session drain, endpoint close, forwarding stop decision |
| TURN allocation/permission relay semantics | core/turn | lifecycle event and relay authorization semantics |
| socket/listener stop | driver | concrete I/O stop and receive loop closure |
| runtime task/worker stop | driver/entrypoints runtime | bounded join/cancel through the task lifecycle authority |
| packet/buffer release | driver | lease/queue/cache release |
| audit/persistence/metrics flush execution | driver | bounded flush/retry and failure mapping |
| unclean process termination observation | entrypoints/driver observation | does not convert into graceful drain success |

entrypoints MAY initiate the shutdown order but do not own the authority of domain state transition. driver MAY stop I/O but does not own the domain meaning of accepted/rejected/drained.

### B-2 Drain Sequence (principle order)

1. entrypoints creates shutdown correlation and typed shutdown request.
2. entrypoints stops new external admission at the listener/connection boundary.
3. core/signaling begins room drain or rejects unavailable room commands with a cataloged reason.
4. core/sfu drains sessions/endpoints and returns forwarding closure/suppression decisions.
5. core/turn stops new allocation/permission paths through existing TURN lifecycle decisions or driver shutdown conversion.
6. driver stops receive loops and prevents new buffer leases.
7. driver/entrypoints cancel or join runtime tasks inside admitted supervision scopes.
8. driver drains bounded transmit/relay/audit/persistence queues according to resource bounds.
9. driver releases packet buffers, packet cache, TURN relay resources, and network sockets.
10. entrypoints stops runtime after the mandatory audit/bootstrap evidence path has been attempted.

If a later step fails, earlier accepted domain decisions MUST NOT be reclassified as success for the failed step.

### B-3 Plane-Specific Rules

| Plane | Drain behavior | Required failure relation |
|---|---|---|
| Signaling | room enters `room_draining` before new join rejection | `room_draining`, `room_closed`, `room_close_not_allowed` |
| SFU | session/endpoint stop is core decision, driver executes queue/cache release | `sfu_session_not_accepting`, `endpoint_closed_by_backpressure`, `driver_shutdown` as applicable |
| TURN | active relay path stops by lifecycle/release or driver shutdown conversion | `driver_shutdown`, existing TURN forbidden/expired/resource reasons |
| Network driver | stop listener/read/write loops | `driver_shutdown`, `network_receive_failed`, `network_send_failed` |
| Persistence/audit/metrics driver | bounded flush/retry only | `persistence_unavailable`, `audit_backlog_bound_exceeded`, `metrics_export_failed`, `driver_shutdown` |

A new plane-specific shutdown reason requires a core reason catalog update.

### B-4 Admission Stop Rule

Admission stop is not the same as driver process death. When entrypoints starts drain, driver MUST stop accepting new external sessions as soon as the configured shutdown mode requires it. Already materialized core state MUST close through core-owned state machines when a core state exists. Pre-core external inputs after admission stop MAY fail with `driver_shutdown` or connection-level close mapping.

### B-5 Unclean Termination Rule

panic, crash, forced kill, and supervisor restart are not treated as a subsequent step of the graceful shutdown sequence. Their observation is classified as process lifecycle observation and MUST NOT rewrite existing accepted domain decisions into retroactive success/failure. Restore/replay eligibility after unclean termination follows the durable recovery authority.

### B-6 Evidence Rule

Shutdown/drain evidence MUST record: shutdown correlation ID; startup run ID when available; drain mode; reconfiguration generation when drain is caused by runtime reconfiguration; affected plane; accepted/rejected/failed decision; cataloged reason for non-success; bounded flush/retry result; runtime task/worker join or cancellation result when worker execution affects the drain; unreleased resource count if any. Missing shutdown evidence MUST NOT be used as proof of graceful shutdown.

---

## Part C. Crash / Panic / Supervisor Restart Classification

This part is the boundary for crash, panic, unclean shutdown, supervisor restart, and process restart. It fixes classification and evidence-acceptance conditions so that graceful shutdown/drain, durable recovery, and liveness/readiness are not confused with unclean process failure. Unclean restart is not graceful drain. Supervisor restart is not recovery success.

### C-1 Boundary

| Concern | Owner | Rule |
|---|---|---|
| process crash observation | entrypoints/supervisor integration | process lifecycle observation only |
| panic classification | entrypoints/driver runtime boundary | not a domain decision |
| task panic classification | runtime task lifecycle boundary | not equated with process crash |
| domain state after crash | core only if restore policy applies | no silent restore |
| driver resource cleanup after crash | driver/entrypoint runtime | best effort; evidence required |
| supervisor restart | external supervisor/entrypoints observation | not readiness proof |
| recovery/restore | core/drivers/entrypoints as defined | follows the durable recovery authority |

### C-2 Failure Classes (closed set)

| Class | Meaning | Claim limit |
|---|---|---|
| `panic_observed` | runtime/application panic observed | not graceful shutdown |
| `task_panic_observed` | runtime task/worker panic observed | not domain transition or process recovery |
| `process_crash_observed` | process exits unexpectedly | not domain close |
| `unclean_shutdown_detected` | shutdown lacks drain/audit completion evidence | not graceful drain |
| `supervisor_restart_observed` | external supervisor restarted process | not readiness or restore success |
| `startup_after_unclean_exit` | entrypoint starts after a prior unclean exit | restore policy required before state claims |
| `crash_recovery_evidence` | controlled crash/restart test report | evidence class limited to tested scope |

A new process failure class is out of the v0.2 initial scope.

### C-3 Classification Rule

A report or observation MUST classify: startup run ID before and after restart when available; process identity; failure class; prior drain status; audit persistence status; restore/replay policy applied or not applied; readiness class after restart; close-not-claimed scope. If prior drain status or audit status is absent, the restart MUST be treated as unclean for closeout claims.

### C-4 Failure Mapping

| Failure | Required reason |
|---|---|
| process panic observed | `process_panic_detected` |
| task panic observed without process crash | `runtime_task_panic_detected` |
| process crash observed | `process_crash_detected` |
| unclean shutdown detected | `unclean_shutdown_detected` |
| supervisor restart observed | `supervisor_restart_observed` |
| restore not allowed after restart | restore-specific reason from the durable recovery authority |
| driver shutdown during crash handling | `driver_shutdown` |

### C-5 Evidence Rule

Crash/restart evidence MUST NOT be adopted unless it records: exact command/procedure; supervisor or process runner; expected failure class; observed failure class; startup run IDs; logs only as diagnostic support; audit/restore/readiness status as separate evidence classes. Process uptime after restart is not proof of restored domain state.

---

## Part D. Operator / Admin Authorization

This part is the boundary for operator / admin authorization. It fixes that operator-facing CLI, admin endpoint, maintenance action, probe action, and evidence verification command are not confused with generic communication authorization, domain decision, or runtime readiness. Communication participant authorization does not authorize operator/admin action. Operator/admin authorization does not authorize participant join, publication, subscription, or TURN relay.

### D-1 Boundary

| Concern | Owner | Rule |
|---|---|---|
| operator credential source | external system or entrypoints/drivers | raw credential is not core identity |
| operator credential verification | entrypoints/drivers + security verifier boundary | typed result only |
| admin authorization context mapping | entrypoints/drivers before core/admin policy input | opaque operator/admin context |
| admin action policy | core/admin policy or entrypoint boundary policy depending on action class | explicit action/scope required |
| maintenance action execution | entrypoints invokes core use case or driver operation | no domain mutation outside allowed boundary |
| audit evidence | core event model and driver sink | operator/admin authorization decision |

### D-2 Operator Admin Classes (closed set)

| Class | Meaning | Rule |
|---|---|---|
| `operator_probe_context` | operator may request a diagnostic probe | not domain acceptance |
| `operator_admin_action_context` | operator may request an admin action | action/scope required |
| `maintenance_action_context` | operator may request drain/maintenance mode | follows health/admin and shutdown authorities |
| `evidence_verification_context` | operator may verify evidence/hash-chain/report | does not create evidence by itself |
| `developer_local_context` | local development helper context | not production/operator evidence |

A new operator/admin class is out of the v0.2 initial scope.

### D-3 Authorization Rule

Operator/admin authorization MUST declare: operator/admin class; credential/context source; allowed action class; allowed target scope; lifetime and expiry; redaction rule; target command boundary; audit event relation; failure reason mapping. An admin action MUST still call the target core use case or driver operation through the allowed boundary.

### D-4 Failure Mapping

| Failure | Required reason |
|---|---|
| operator credential missing | `operator_credential_missing` |
| operator credential invalid | `operator_credential_invalid` |
| operator authorization context missing | `operator_authorization_context_missing` |
| operator authorization context expired | `operator_authorization_context_expired` |
| operator action denied by policy | `operator_action_denied` |
| operator target scope not allowed | `operator_scope_not_allowed` |
| admin action not allowed by boundary/policy | `admin_action_not_allowed` |
| maintenance mode blocks action | `maintenance_mode_active` |
| runtime configuration missing | `runtime_config_missing` |
| runtime configuration invalid | `runtime_config_invalid` |

### D-5 Audit / Evidence Rule

Operator/admin authorization decisions use audit event type `operator_admin_authorization_decision`. Target admin/maintenance actions still use `admin_maintenance_decision` or the target domain event type. The same correlation chain MAY include both events, but one does not replace the other.

Operator/admin evidence MUST record: operator/admin class; action class; target scope; credential/context class without raw credential; lifetime/expiry observation when relevant; audit event type `operator_admin_authorization_decision`; target action event when the claim includes action execution; expected outcome; actual outcome; cataloged reason for non-success; close-not-claimed scope. CLI exit code alone is diagnostic only.

---

## Part E. Cross-cutting Prohibitions

- listener bind success is reported as full readiness.
- liveness success is reported as domain acceptance.
- admin command mutates state outside a core use case.
- admin command executes without operator/admin authorization evidence.
- maintenance mode is represented only by a driver-local flag.
- probe response hides a failed dependency.
- readiness output is used as production evidence without reports.
- single-node readiness is used as multi-node readiness.
- public endpoint connection success is treated as domain readiness without target contract evidence.
- service discovery success is treated as readiness without target dependency/probe evidence.
- failover is treated as healthy from replacement endpoint reachability alone.
- worker task is treated as healthy from spawn success alone.
- internal service trust is treated as healthy from endpoint resolution or TLS listener startup alone.
- entrypoints directly mutate room/SFU/TURN state to closed without a core decision.
- driver treats socket close as a successful domain leave/close.
- shutdown failure is hidden behind a process exit code only.
- crash / panic / supervisor restart observation is reported as graceful drain success.
- unbounded drain wait or unbounded flush queue is allowed.
- failed audit/persistence flush is used as closeout evidence.
- one plane's successful drain implies another plane's successful drain.
- split-service drain control lacks internal control-plane correlation/audit evidence.
- runtime reconfiguration applies before required drain/restart.
- detached or unjoined worker remains while graceful drain is claimed.
- unclean crash is reported as graceful shutdown.
- supervisor restart is reported as readiness.
- crash recovery success is inferred without restore/replay evidence.
- panic log text becomes authoritative reason.
- restarted process reuses previous domain state without restore policy.
- crash evidence omits prior audit/drain status.
- communication participant token authorizes operator/admin action by default.
- operator/admin authorization is inferred from localhost, network reachability, or deployment environment.
- raw operator credential appears in audit/log/report.
- admin UI or CLI response becomes domain decision evidence.
- developer-local context is used as production operator proof.

## Part F. Cross-cutting Collapse Conditions

- readiness is a single unqualified boolean.
- operator action bypasses core/drivers/entrypoints ownership.
- operator action authorization is inferred from endpoint reachability.
- health probe exposes sensitive raw data.
- maintenance status changes domain state without a core transition.
- probe success is used as build/test/runtime proof outside its evidence class.
- topology class is hidden in operational readiness evidence.
- public endpoint lifecycle state is hidden when endpoint behavior is the target claim.
- process lifecycle is treated as domain state authority.
- driver I/O closure is reported as an accepted room/SFU/TURN transition without a core decision.
- drain waits or queues are unbounded.
- shutdown lacks correlation and plane-specific evidence.
- graceful shutdown success is inferred from process exit alone.
- unclean termination evidence is mixed with graceful drain evidence.
- reconfiguration-driven drain lacks generation and apply-scope evidence.
- process failure class is absent.
- restart is treated as domain recovery proof.
- unclean shutdown bypasses durable recovery policy.
- panic/crash observation mutates core state.
- task panic is reported only as process readiness or generic driver failure.
- crash report lacks startup run boundary.
- operator/admin action has no explicit authorization context.
- target action scope is implicit.
- operator authorization and communication authorization are treated as the same decision.
- admin action execution lacks target boundary evidence.
- operator/admin denial is recorded only as free-text.

## Part G. Invariants (summary)

- Readiness is a composition that makes explicit the closed-set probe classes and included/excluded checks; it is not a single boolean.
- Process lifecycle (liveness/shutdown/crash/restart) is separated from domain lifecycle, and neither substitutes for the other.
- Closing a domain state is done only through a core-owned state machine; driver I/O closure or process exit does not prove it.
- Drain and flush are always bounded, and unclean termination is a separate evidence class from graceful drain.
- Operator/admin authorization is independent from communication authorization and is not inferred from reachability or localhost.
- raw credential / secret / sensitive data does not appear in audit/log/report/probe.
