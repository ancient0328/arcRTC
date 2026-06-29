# Troubleshooting (cross-cutting)
Status: SSOT consolidated edition
Date: 2026-06-28 JST

## Purpose

This chapter internalizes, self-contained, the cross-cutting troubleshooting of the arcRTC v0.2 Kernel as practical tables of "symptom -> suspected boundary/cause -> fail-closed default behavior -> remediation." From this chapter alone, you can understand the detection and remediation of representative boundary violations (dependency-direction violation, out-of-closed-vocabulary, driver/entrypoint defining semantics, claim repurposing), the pitfalls of external error mapping, the handling per crash/panic classification, and the fail-closed default behavior of the CI quality gate.

The dependency-direction notation `A <- B` means "B depends on A (B may reference A)." Permitted directions are `core <- drivers`, `core <- entrypoints`, `drivers <- entrypoints`, conditionally `core <- regulated`.

**Fail-closed principle.** Ambiguous, out-of-closed-set, or unknown items are not adopted; fall to the rejecting side (No / reject / not-ready). `UNKNOWN` is not a success state. close / complete / ready is not claimed for an unperformed verification scope. reason is a closed set; out-of-closed-set items (free-text or invented reasons) are not adopted as the authority of decision, audit, or SDK mapping.

## 1. Boundary violations (dependency direction / ownership)

| Symptom | Suspected boundary / cause | Fail-closed default behavior | Remediation |
|---|---|---|---|
| core imports `axum` / `tokio::net` / `str0m` concrete / `sqlx` / browser·native SDK types | `core -> drivers` dependency-direction violation; core references concrete I/O·runtime·DB·cloud·browser·native types | architecture dependency gate fails; close/complete/ready is not claimable | Convert external types to core-owned types at the driver boundary. From core, reference only verification result·decision rule·borrowed view. Restore the form where core owns the port and the driver implements it |
| a driver crate defines a port trait | port ownership violation (driver owns a port interface) | dependency boundary gate fails (entrypoints/drivers must not define port traits) | Move the port trait back to core. Limit the driver to implementing the core-owned port |
| the authority of accept·reject·routing·allocation decision is inside a WebSocket handler / worker loop / socket read loop | driver/entrypoint owns core semantics (leakage of Signaling state transition·SFU routing·TURN allocation authority) | architecture dependency gate and source-shape test fail | Move that decision authority back to `core/signaling`·`core/sfu`·`core/turn`. Limit the driver to byte I/O·conversion·export and the entrypoint to wiring·lifecycle |
| a `drivers -> entrypoints` reference occurs | drivers depend on entrypoints (reverse of the permitted direction) | dependency boundary gate fails | Restore the form where entrypoints compose drivers. Do not reference entrypoints from drivers |
| `core/drivers/entrypoints/sdk -> regulated` occurs, or `regulated -> sdk/drivers/entrypoints` | regulated independent boundary violation | dependency boundary gate fails (sdk must not depend on regulated; regulated must not depend on drivers·entrypoints·sdk) | Restore regulated as an independent boundary of optional domain support. The only permitted direction is `regulated -> core` (limited to references to opaque communication events·audit pointers·non-sensitive tags) |
| the SDK API exposes media / auth issuance / regulated workflow / PeerConnection | SDK Signaling-only boundary violation | contract test gate (SDK public contract) fails | Restore the SDK to Signaling-only (command/event/connection lifecycle). Remove media·auth issuance·regulated·PeerConnection from the SDK |
| an entrypoint claims domain rule / protocol semantics / product readiness | entrypoint reclassification violation (entrypoint misread as a product server) | product / readiness claims by an entrypoint are rejected; an entrypoint is an executable contract / composition evidence surface only | Restore the entrypoint to an executable contract / composition evidence surface. Remove product readiness claims |
| the source-shape test fails because of a 600-line excess | misapplication of source shard / semantic modular monolith (line count made a hard gate above the semantic boundary) | 600 lines is a review signal, not a hard gate; only collapse of semantic owner·dependency·forbidden import is a fail condition | Do not fail on line count alone. Treat it as a source-shape failure only when semantic owner transfer·dependency inversion·forbidden import·hidden ownership accompanies it |

## 2. Deviation from the closed reason vocabulary

reason is a closed set. reason consists of `category` (core-owned, closed set), `code` (closed set within category), `retryable`/`safe_to_expose`/`audit_required` (core-owned), and `details` (driver/entrypoint, optional·non-authoritative). The cross-cutting categories are `malformed_input`, `unsupported_version`, `unauthorized`, `forbidden_state`, `duplicate`, `ordering_violation`, `expired`, `resource_exhausted`, `backpressure`, `quality_violation`, `driver_failure`, `shutdown`.

| Symptom | Suspected boundary / cause | Fail-closed default behavior | Remediation |
|---|---|---|---|
| decision/audit/SDK mapping uses a free-text reason as authority | out-of-closed-set reason adoption (free-text is supplementary only) | out-of-closed-set is not adopted; not made the authority of decision | Map the reason to a closed category/code. Limit free-text to non-authoritative `details` |
| metadata (retryable/safe_to_expose/audit_required) is guessed for convenience | guessing metadata absent from the category default or code override | guessed metadata is not adopted | Determine metadata from the category default and code override. If absent, update the reason rules before adopting |
| a retry hint is exposed for a non-retryable reason | reason metadata violation (e.g., `unauthorized`·`forbidden_state`·`expired` are `retryable=false`) | a retry hint must not be exposed for non-retryable | Follow `retryable` in the reason metadata. Allow retry hints only for `retryable=true` such as `resource_exhausted`·`backpressure`·`quality_violation`·`driver_failure` |
| the detail of a `safe_to_expose=false` reason is exposed externally | exposure-control violation (e.g., `unauthorized`·`driver_failure`·`token_key_unavailable`·`secret_unavailable` are `safe_to_expose=false`) | fall to a generic external failure class + opaque reference | Per the safe exposure rule, when `safe_to_expose=false` do not expose category/code and return only an opaque reference. Do not leak secret/key/token/backend detail |
| audit event relation is not recorded on an `audit_required=true` path | missing audit relation | without an audit relation, that path cannot be used as close evidence | Record the audit event relation. Do not adopt as close evidence a path where it cannot be recorded |
| a new reason / new process failure class outside the closed set is added | unauthorized expansion of the closed set | out-of-closed-set is not adopted | Adopt a new reason category/code·new process failure class only after an explicit rule update |

## 3. Pitfalls of external error mapping

Owners of external error mapping: authoritative reason category/code is core; reason exposure metadata (retryable·safe_to_expose·audit_required) is core; external protocol status/wrapper is driver/sdk/cli (a projection that does not lose the reason); raw driver error detail is driver (non-authoritative·redacted); audit event is core model + driver sink (not a substitute for the external response). External code is not authoritative; the authoritative failure remains the cataloged reason category/code or the pre-core driver conversion reason.

| Symptom | Suspected boundary / cause | Fail-closed default behavior | Remediation |
|---|---|---|---|
| an HTTP status / WebSocket close code replaces the core reason | external code mistaken as the authoritative reason | external code is not authoritative | Fix to a projection that preserves the core reason category/code or opaque error reference. HTTP and WebSocket: network driver; STUN/TURN: TURN wire driver; SDK: sdk; CLI: entrypoints/cli are the mapping owners |
| the external response claims success after a core non-success | failure mapping violation | the external response must not claim success after a core non-success | Make the core decision authoritative and record response emission failure separately as a driver observation. Use `external_encode_failed` on encode failure, `network_send_failed` on network send failure, `driver_shutdown` on driver shutdown |
| the SDK invents a server reason for a server-event decode failure | SDK invents server reason | the SDK must not use an invented server reason for a local decode failure | Return an SDK-local closed error and do not create a non-server-originated reason. Preserve only server-originated reasons |
| logs/traces are used as the external error authority | misuse of redacted diagnostics | logs/traces are not the external error authority | Keep logs/traces as redacted diagnostics only. Return the authoritative reason to the core catalog |
| field-order mistakes cause unsafe reason exposure or missing audit relation | use of a naked multi-argument constructor | a naked multi-argument constructor is not an admitted boundary | Pass external surface·status/wrapper class·correlation reference·exposed/redacted reason·audit relation·retry hint via a named input type such as `ExternalErrorProjectionInput` |

## 4. crash / panic / supervisor restart classification

The process failure class is a closed set: `panic_observed` (not graceful shutdown), `task_panic_observed` (neither domain transition nor process recovery), `process_crash_observed` (not domain close), `unclean_shutdown_detected` (not graceful drain), `supervisor_restart_observed` (not readiness/restore success), `startup_after_unclean_exit` (restore policy required before a state claim), `crash_recovery_evidence` (limited to the tested scope). Unclean restart is not graceful drain, and supervisor restart is not recovery success.

| Symptom | Suspected boundary / cause | Fail-closed default behavior | Remediation |
|---|---|---|---|
| an unclean crash is reported as graceful shutdown | crash classification violation | unclean shutdown is not graceful drain | Classify the failure class correctly (`unclean_shutdown_detected`/`process_crash_observed`). If prior drain status or audit status is absent, treat the restart as unclean for closeout claims |
| a supervisor restart is reported as readiness | restart=recovery misconception | supervisor restart is not readiness/restore success | Record the post-restart readiness class as separate evidence. Do not use process uptime as proof of restored domain state |
| previous domain state is reused after restart without a restore policy | silent restore | domain state after crash is handled by core only when a restore policy applies; silent restore is not allowed | At `startup_after_unclean_exit`, apply a restore/replay policy before a state claim (per the durable recovery rules). If inapplicable, use a restore-specific reason |
| a task panic is reported only as process readiness or generic driver failure | confusion of task panic with process crash | `task_panic_observed` is not equated with process crash | Classify task panic at the runtime task lifecycle boundary and use `runtime_task_panic_detected` (`process_crash_detected` on process crash observation, `process_panic_detected` on process panic observation) |
| panic/crash log text is adopted as the authoritative reason | misattributing authority to log text | panic log text is not the authoritative reason | Take the reason from the closed catalog. Keep logs as diagnostic support only |
| crash evidence lacks prior audit/drain status or the startup run boundary | missing evidence | crash evidence with omissions is not adopted | Record in the crash/restart evidence the exact command/procedure, supervisor/process runner, expected/observed failure class, startup run IDs, and audit/restore/readiness status (separate evidence classes) |

## 5. CI quality gate fail-closed default behavior

CI is a guardrail and does not by itself establish close / complete / ready. CI output is not sufficient evidence on its own; a passing CI run for an unperformed or unknown verification scope is not a close / complete / ready claim. CI logs are diagnostic output only.

| Symptom | Suspected boundary / cause | Fail-closed default behavior | Remediation |
|---|---|---|---|
| a required gate cannot run / times out / reports unknown state | gate execution failure | do not treat the target scope as close/complete/ready; `UNKNOWN` is not a success state | Make the gate runnable, and obtain a reproducible result before judging |
| CI success is claimed as production readiness | a close-like claim bypasses the verification scope | close / complete / ready is not claimed for an unperformed verification scope | CI success is a guardrail, not sole grounds for readiness. Production / live readiness is a separate implementations-side claim |
| enterprise code coverage passes with core line `<90%` / overall `<85%` / core crate floor `<80%` / missing denominator scope | coverage gate fail-open | fail-closed when core line<90% / overall<85% / core crate floor<80% / denominator missing / diagnostic-only coverage is adopted | Obtain threshold-meeting coverage. Do not adopt import-only·smoke-only·text-inspection-only·generated-output-only coverage as enterprise coverage |
| v0.2 completion is claimed from CI success alone | completion claimed for an unperformed verification scope | completion is not claimed for an unperformed verification scope | Limit CI success to its scope. Kernel completion is a separate claim and is not derived from CI success alone |
| JS/TS evidence is generated with npm/yarn | tooling violation | npm/yarn are not adopted as v0.2 CI evidence | Use `pnpm` for JS/TS commands |
| a source-only static gate proves runtime behavior, or a runtime smoke proves architecture compliance | confusion of evidence classes | do not mix evidence classes across source/build/runtime/live | Limit static gates to proving dependency direction·source-shape and runtime smoke to proving composition, recording class-specific evidence fields for each |
| an endpoint/artifact/release/time/edge·proxy/reconfiguration gate is claimed successful without class-specific evidence | missing class-specific fields | each gate must not be claimed successful without class-specific evidence fields | Record the required fields of each gate class (public endpoint lifecycle, export/backup artifact, release artifact provenance, time synchronization, edge/proxy trust, runtime reconfiguration, packet rewrite/media transform, service discovery, distributed state/failover, runtime task/worker lifecycle, internal service identity/trust, cross-plane identity/session binding, etc.) |

## 6. Detecting and remediating claim scope repurposing

Kernel evidence and implementations readiness are separate claims and MUST NOT substitute for one another. The following are representative symptoms of claim scope repurposing.

| Symptom | Suspected boundary / cause | Fail-closed default behavior | Remediation |
|---|---|---|---|
| Kernel evidence is repurposed into implementations production/live readiness | Kernel / implementations boundary violation | the presence/absence of implementations is not adopted into the Kernel completion claim | Keep Kernel evidence closed to the Kernel claim. Treat implementations readiness as separate evidence and a separate claim |
| benchmark·coverage·native command success is repurposed as proof of readiness | crossing of evidence scope | benchmark/coverage/native command success is Kernel evidence and not proof of production/live/native application readiness | Limit each evidence to its scope (measurement·diagnostic·command success). Claim readiness separately on the implementations side |
| an unperformed verification scope is treated as performed | scope repurposing | an unperformed verification scope is not treated as performed; `UNKNOWN` is not a success state | Perform the verification within its scope, or do not claim close / complete / ready for it |
| a present artifact is treated as a completion proof | confusion of presence with proof | artifact existence is not a completion proof | Do not use artifact existence as proof. An out-of-scope artifact is managed separately and is not adopted into the Kernel completion claim |

## 7. Final detection principle (fail-closed summary)

- Ambiguous, out-of-closed-set, unknown, diagnostic-only, mixed-scope, or readiness-smuggled inputs fall to the rejecting side. `UNKNOWN` is not a success state.
- reason, process failure class, and CI gate class are all closed sets. Expansion of a closed set is adopted only after an explicit rule update.
- close / complete / ready / freeze is not claimed for an unperformed verification scope.
- When a boundary violation or claim scope repurposing is suspected, remediation is always in the direction of "returning to the authoritative owner (core semantics / core reason / core port)" and does not move semantic authority to the driver·entrypoint·SDK·regulated·implementations side.
