# core-command-and-reason

Status: SSOT consolidated edition
Date: 2026-06-28 JST

## Purpose

This chapter internalizes, at a re-implementable level of detail, the arcRTC v0.2 Kernel core result shapes for command / decision / domain event / port intent / external response, the command idempotency / replay / correlation rules, the closed reason catalog (enumerating every reason code), and the error mapping between internal reasons and external representations. This chapter does not claim Rust struct / enum implementation, public API implementation, or runtime behavior.

Dependency notation `A <- B` reads as "B depends on A". The layering is `core <- drivers <- entrypoints`.

## 1. Result Shape Ownership Boundary

| Surface | Owner | Rule |
|---|---|---|
| inbound command semantic shape | core | core-owned command after driver conversion |
| use case decision | core | authoritative outcome such as accept / reject / suppress / drop / fail |
| domain event | core | state transition or decision observation |
| port command / intent | core | driver execution request; bytes or concrete I/O not included |
| driver execution observation | driver converted to core-owned observation | concrete send/write/read result converted to a cataloged reason |
| audit event | core model, driver sink | closed projection of decision/event |
| external response | driver/sdk | maps core reason/outcome to external form without loss |

A driver MUST NOT rewrite a core decision into success. entrypoints MUST NOT define use case result shape per binary.

## 2. Result Shape Classes (closed set)

| Shape class | Owner | Meaning |
|---|---|---|
| `CommandEnvelope` | core | validated correlation, version, command type, subject references |
| `UseCaseDecision` | core | authoritative outcome for a command |
| `DomainEvent` | core | state transition or decision fact |
| `PortIntent` | core | execution intent requested of a driver |
| `DriverObservation` | driver-to-core mapping | concrete execution result converted to core-owned observation |
| `AuditProjection` | core | evidence item projected into the audit event model |
| `ExternalResponseModel` | driver/sdk | external response shape; authoritative reason preserved |

A new result shape class requires a specification update.

## 3. Required UseCaseDecision Fields

Every `UseCaseDecision` MUST carry:

- correlation ID;
- command or event type;
- target surface;
- outcome;
- reason presence;
- reason category/code when non-success;
- state transition summary when state changes;
- port intents when driver execution is required;
- audit projection requirement;
- close-not-claimed evidence class when the decision is not runtime proof.

A success outcome MUST NOT carry a fake reason. A non-success outcome MUST carry a cataloged reason.

## 4. Construction Input Rule

A result shape such as `UseCaseDecision` MUST NOT receive correlation, command type, target surface, outcome, reason, state transition, port intent, audit projection, and evidence class as a naked multi-argument constructor. Implementations bundle the unchecked materials into a named input type such as `UseCaseDecisionInput` and pass that input type into fail-closed checks. This rule is the boundary that prevents field-order swaps, fake reason injection, and missing audit/evidence class.

## 5. Outcome Rule

The allowed authoritative outcomes are limited to the audit-compatible outcome set; use case results MUST NOT invent a second outcome vocabulary.

| Outcome family | Meaning | Reason rule |
|---|---|---|
| success observation | accepted, allowed, forwarded, selected, released, closed_success, idempotent_observed, within_bound_observed | reason absent |
| rejection / denial | rejected, denied, protocol_violation | cataloged reason required |
| suppression / drop / expiry / revocation | suppressed, dropped, expired, shed, revoked | cataloged reason required |
| degradation / delay | delayed, degraded | cataloged reason required |
| failure / conversion | failed, converted_failure | cataloged reason required |
| policy closure | closed_by_policy | cataloged reason required |

Partial success is prohibited unless the specification defines per-step outcomes and a rollback/compensation rule.

## 6. Event and Port Intent Rule

A core decision MAY produce domain events and port intents. A port intent is not driver execution success. Example separation:

```text
UseCaseDecision accepted
  -> DomainEvent emitted
  -> PortIntent send response / forward packet / persist audit
  -> DriverObservation success or cataloged failure
```

A driver failure after an accepted decision MUST be represented as a driver observation or follow-up decision evidence. It MUST NOT retroactively erase the original domain decision unless the state machine specification explicitly defines a compensating transition. When a compensating transition is allowed, the result shape MUST preserve both the original decision and the compensation decision.

## 7. Audit Projection Rule

Every decision that requires audit MUST define: event type code; decision outcome; reason presence; required references; resource owner tuple when resource-bound; sensitive data redaction rule.

## 8. External Response Rule

An external response is a projection, not the authoritative decision. HTTP status, WebSocket close code, STUN/TURN error code, SDK exception class, and CLI exit code MUST preserve traceability to the core outcome and reason.

## 9. Command Idempotency / Replay / Correlation

### 9.1 Ownership Boundary

| Concern | Owner | Rule |
|---|---|---|
| correlation tracing | core reference accepted after driver validation | tracking of request/event chain, not duplicate determination itself |
| command identity | core | tuple of command type, target reference, idempotency key, semantic payload digest |
| idempotency policy | core | decides accepted duplicate / rejected duplicate / conflicting replay |
| replay window policy | core | lifetime, scope, payload comparison rule |
| idempotency storage / response cache | driver persistence or in-memory driver detail | bounded implementation, not decision owner |
| SDK pending command replay | sdk | client-local retry only; does not create server acceptance |
| audit evidence | core event model and driver sink | idempotency decision and correlation references |

`CorrelationId` is a trace identity; the same `CorrelationId` alone is not treated as an idempotent duplicate. Idempotency requires an explicit command identity rule.

### 9.2 Idempotency Classes (closed set)

| Class | Meaning | Rule |
|---|---|---|
| `non_idempotent_command` | duplicate must be rejected or re-evaluated by the state machine | response replay not allowed |
| `idempotent_same_payload` | same command identity and same canonical payload digest | prior accepted result may be observed |
| `idempotent_conflict` | same idempotency scope but different payload digest or target | reject with cataloged reason |
| `response_replay_candidate` | prior response may be replayed as external projection | only within replay window and same semantic result |
| `sdk_pending_command_replay` | SDK resends client-local pending command after reconnect | server must still evaluate the command |
| `server_event_replay` | missed event replay from server side | prohibited unless a separate replay policy exists |

A new idempotency class requires a specification update.

### 9.3 Command Identity Rule

Idempotent command identity MUST define: command type; idempotency scope; target reference; actor/participant reference when naturally present; canonical payload digest when payload affects semantics; replay window; accepted response replay rule; conflict reason; audit event relation. When used, the payload digest follows the canonical serialization rules (internalized in chapter 06).

### 9.4 Replay Rule

Replay is not retry success. A driver or SDK MAY resend, but core MUST evaluate whether the command is duplicate, expired, conflicting, or acceptable.

Response replay is allowed only when all hold: the original command result is within the replay window; command identity and canonical payload digest match; required audit evidence for the original result exists or the path explicitly states close-not-claimed scope; the response projection preserves the original outcome and reason. If any condition is absent, response replay MUST fail closed.

### 9.5 Idempotency / Replay Failure Mapping

| Failure | Required reason |
|---|---|
| correlation ID missing before core entry | `missing_correlation_id` |
| correlation does not match expected response/event chain | `correlation_mismatch` |
| duplicate rejected by idempotency rule | `duplicate_command` |
| duplicate has different payload or target | `idempotency_payload_mismatch` |
| replay window expired | `idempotency_window_expired` |
| replay is not allowed for command class | `replay_not_allowed` |
| response replay evidence/cache unavailable | `response_replay_not_available` |
| canonical payload digest cannot be produced | `canonical_serialization_failed` |

### 9.6 Idempotency Audit / Evidence

Idempotency / replay decisions use audit event type `command_idempotency_decision`. The event MUST carry `CorrelationId`, command type, idempotency class, idempotency scope, and target reference when naturally materialized. If response replay uses a prior result, the event MUST reference the prior correlation or audit event reference without copying sensitive payload. SDK reconnect evidence does not prove server idempotency unless it includes server-side decision/audit evidence for the replayed command.

## 10. Closed Reason Catalog

reason is a closed set. free-text is limited to supplementary explanation and MUST NOT be used as authority for decision, audit, or SDK mapping. A reason has the following structure.

| Field | Owner | Rule |
|---|---|---|
| `category` | core | closed set |
| `code` | core | closed set within category |
| `retryable` | core | true / false |
| `safe_to_expose` | core | true / false |
| `audit_required` | core | true / false |
| `details` | driver or entrypoint | optional, non-authoritative |

### 10.1 Cross-Cutting Categories (12, closed set)

| Category | Meaning |
|---|---|
| `malformed_input` | syntax or shape is invalid |
| `unsupported_version` | protocol version is not accepted |
| `unauthorized` | verification failed or required authorization absent |
| `forbidden_state` | command is invalid for current state |
| `duplicate` | idempotency rule rejects duplicate |
| `ordering_violation` | command/event order is invalid |
| `expired` | lifetime or deadline exceeded |
| `resource_exhausted` | bounded resource limit reached |
| `backpressure` | pressure policy delays, suppresses, drops, degrades, closes, or rejects recovery |
| `quality_violation` | quality policy rejects, suppresses, degrades, or rejects recovery |
| `driver_failure` | external implementation failure after conversion |
| `shutdown` | lifecycle shutdown or cancellation prevents action |

### 10.2 Reason Metadata (category default)

`retryable`, `safe_to_expose`, and `audit_required` are determined from category default and code override. Metadata that exists in neither the category default nor a code override MUST NOT be inferred for implementation convenience.

| Category | retryable | safe_to_expose | audit_required |
|---|---|---|---|
| `malformed_input` | false | true | true |
| `unsupported_version` | false | true | true |
| `unauthorized` | false | false | true |
| `forbidden_state` | false | true | true |
| `duplicate` | false | true | false |
| `ordering_violation` | false | true | true |
| `expired` | false | true | true |
| `resource_exhausted` | true | true | true |
| `backpressure` | true | true | true |
| `quality_violation` | true | true | true |
| `driver_failure` | true | false | true |
| `shutdown` | false | true | true |

### 10.3 Code-Specific Override (54)

| Code | retryable | safe_to_expose | audit_required | Reason |
|---|---|---|---|---|
| `token_key_unavailable` | true | false | true | key source may recover, but exposure must not reveal key source detail |
| `external_type_leak_blocked` | false | false | true | boundary violation is not a client retry condition |
| `external_encode_failed` | false | false | true | encoding failure indicates server/driver defect |
| `buffer_release_failed` | false | false | true | release failure requires audit and is not retried by core |
| `core_policy_config_invalid` | false | true | true | invalid policy must stop or reject rather than retry |
| `runtime_config_missing` | false | true | true | missing required runtime config must stop startup/path |
| `runtime_config_invalid` | false | true | true | invalid runtime config must stop startup/path |
| `secret_unavailable` | true | false | true | secret source may recover, but secret detail is not exposed |
| `operation_cancelled` | false | true | true | cancellation must not be retried as if it were transport failure |
| `capability_not_enabled` | false | true | true | disabled capability must not fail open or fallback silently |
| `process_panic_detected` | false | true | true | panic classification is not retryable as normal transport failure |
| `process_crash_detected` | false | true | true | unclean process failure requires explicit evidence classification |
| `canonical_serialization_failed` | false | false | true | serializer failure may reveal internal representation detail |
| `canonical_serialization_mismatch` | false | true | true | deterministic verification mismatch is not a client retry condition |
| `authorization_context_invalid` | false | false | true | invalid authorization context may reveal verifier detail |
| `authorization_policy_denied` | false | false | true | policy denial must be audited without revealing policy internals |
| `authorization_scope_not_allowed` | false | false | true | scope denial must not expose authorization policy detail |
| `response_replay_not_available` | false | false | true | missing replay material may reveal server-side cache strategy |
| `secret_rotation_state_unavailable` | true | false | true | rotation state source may recover, but secret state detail is not exposed |
| `secret_key_revoked` | false | false | true | revoked key/generation detail must not be exposed |
| `dependency_policy_violation` | false | true | true | dependency policy failure is not a client retry condition |
| `vulnerability_gate_failed` | false | false | true | vulnerability details require controlled evidence exposure |
| `dependency_missing` | false | true | true | missing dependency/tool blocks evidence and is not runtime retryable |
| `sdk_contract_drift_detected` | false | true | true | generated contract drift is not a client retry condition |
| `internal_control_authorization_denied` | false | false | true | service-to-service denial must not reveal internal policy detail |
| `ice_candidate_redaction_required` | false | false | true | candidate material may reveal address or network detail |
| `secure_media_key_state_invalid` | false | false | true | invalid media key state must not expose keying detail |
| `operator_credential_invalid` | false | false | true | operator credential verification detail must not be exposed |
| `operator_action_denied` | false | false | true | privileged action denial must not reveal policy internals |
| `public_endpoint_auth_required` | false | false | true | public endpoint authentication requirement must not reveal verifier detail |
| `export_redaction_required` | false | false | true | sensitive artifact material must not be exposed before redaction |
| `backup_artifact_unavailable` | true | false | true | artifact source may recover, but storage/tool detail must not be exposed |
| `artifact_integrity_mismatch` | false | true | true | integrity mismatch is not a client retry condition |
| `release_artifact_provenance_missing` | false | true | true | missing provenance blocks release claim and is not runtime retryable |
| `release_artifact_integrity_failed` | false | true | true | release integrity failure blocks distribution claim |
| `time_source_untrusted` | false | true | true | untrusted time source blocks the target claim |
| `time_sync_unavailable` | true | false | true | time synchronization source may recover, but source detail may be sensitive |
| `timestamp_order_untrusted` | false | true | true | timestamp order cannot be retried as a transport failure |
| `forwarded_header_untrusted` | false | false | true | forwarded metadata trust failure must not expose trusted upstream detail |
| `tls_termination_boundary_invalid` | false | false | true | transport boundary failure may reveal infrastructure detail |
| `public_internal_route_confusion` | false | false | true | route confusion must not expose internal routing detail |
| `runtime_reconfiguration_rollback_failed` | true | false | true | rollback execution may recover, but failure detail may expose runtime state |
| `packet_rewrite_owner_violation` | false | false | true | rewrite owner violation may reveal media/backend boundary detail |
| `payload_transform_failed` | true | false | true | driver transform failure may recover, but backend detail must not be exposed |
| `service_endpoint_scope_conflict` | false | false | true | endpoint scope conflict must not reveal internal routing detail |
| `state_owner_conflict` | false | false | true | distributed state owner conflict must not expose topology internals |
| `split_brain_risk_detected` | false | false | true | split-brain risk is not a client retry condition and requires audit |
| `runtime_task_owner_violation` | false | false | true | task ownership violation may expose runtime/component boundary detail |
| `runtime_task_panic_detected` | false | true | true | task panic is not retryable as normal transport failure |
| `internal_service_identity_invalid` | false | false | true | service identity verification detail must not be exposed |
| `internal_service_identity_untrusted` | false | false | true | service trust failure must not reveal trust policy internals |
| `internal_service_identity_scope_conflict` | false | false | true | service scope conflict must not expose internal topology detail |
| `cross_plane_binding_invalid` | false | false | true | binding failure may reveal cross-plane state relation detail |
| `cross_plane_binding_scope_conflict` | false | false | true | cross-plane scope conflict must not expose session topology detail |

### 10.4 Signaling Reasons

| Code | Category | Meaning |
|---|---|---|
| `missing_correlation_id` | `malformed_input` | command lacks correlation ID |
| `malformed_command` | `malformed_input` | command cannot be decoded to core type |
| `unsupported_command_version` | `unsupported_version` | command version is not accepted |
| `token_verification_failed` | `unauthorized` | token verifier result rejects command |
| `room_not_accepting_join` | `forbidden_state` | room state does not accept join |
| `room_capacity_exceeded` | `resource_exhausted` | room materialization or active room bound is reached |
| `room_lifetime_exceeded` | `expired` | room lifecycle duration exceeded |
| `room_closed` | `shutdown` | room is already closed |
| `room_draining` | `shutdown` | room is draining and rejects commands without explicit draining allowance |
| `room_close_not_allowed` | `forbidden_state` | room close/drain transition is invalid for current state |
| `participant_not_joined` | `forbidden_state` | participant action requires joined state |
| `participant_rejected` | `unauthorized` | participant verification or join policy rejected membership |
| `duplicate_command` | `duplicate` | idempotency rule rejects duplicate |
| `correlation_mismatch` | `malformed_input` | command correlation does not match required scope |
| `idempotency_payload_mismatch` | `duplicate` | idempotency key was reused with different command semantics |
| `idempotency_window_expired` | `expired` | idempotency decision/replay window has expired |
| `replay_not_allowed` | `forbidden_state` | command replay is not allowed for this command class or scope |
| `response_replay_not_available` | `driver_failure` | required response replay material is unavailable after accepted idempotency observation |
| `command_order_violation` | `ordering_violation` | command arrives in invalid order |
| `concurrency_conflict` | `ordering_violation` | concurrent candidates conflict on the same serialization scope |

### 10.5 Token Verification Reasons

| Code | Category | Meaning |
|---|---|---|
| `token_missing` | `unauthorized` | required token is absent |
| `token_malformed` | `malformed_input` | token cannot be decoded into verification input |
| `token_signature_invalid` | `unauthorized` | token signature verification failed |
| `token_key_unavailable` | `driver_failure` | verification key source is unavailable after bounded lookup |
| `token_issuer_mismatch` | `unauthorized` | issuer does not match accepted policy |
| `token_audience_mismatch` | `unauthorized` | audience does not match accepted policy |
| `token_expired` | `expired` | token expiry time has passed |
| `token_not_yet_valid` | `forbidden_state` | token is not valid for current time |
| `token_required_claim_missing` | `malformed_input` | required claim is absent |
| `token_unsupported_algorithm` | `unsupported_version` | token algorithm is not accepted |

### 10.6 Authorization Context Reasons

| Code | Category | Meaning |
|---|---|---|
| `authorization_context_missing` | `unauthorized` | required verified authorization context is absent |
| `authorization_context_invalid` | `unauthorized` | authorization context cannot be trusted by the target boundary |
| `authorization_context_expired` | `expired` | authorization context lifetime has passed |
| `authorization_policy_denied` | `unauthorized` | communication policy denies the requested target/action |
| `authorization_scope_not_allowed` | `forbidden_state` | verified authorization context lacks the required communication scope |

### 10.7 Cross-Plane Identity / Session Binding Reasons

| Code | Category | Meaning |
|---|---|---|
| `cross_plane_binding_not_admitted` | `forbidden_state` | cross-plane binding class is not admitted |
| `cross_plane_binding_missing` | `forbidden_state` | required cross-plane binding is absent |
| `cross_plane_binding_invalid` | `malformed_input` | binding material cannot map to declared references |
| `cross_plane_binding_scope_conflict` | `forbidden_state` | source and target plane scopes conflict |
| `cross_plane_binding_lifecycle_conflict` | `forbidden_state` | source or target lifecycle state is not valid for binding |
| `cross_plane_binding_expired` | `expired` | binding lifetime or source relation has expired |
| `cross_plane_binding_replay_detected` | `duplicate` | binding replay or idempotency conflict was detected |

### 10.8 Operator / Admin Authorization Reasons

| Code | Category | Meaning |
|---|---|---|
| `operator_credential_missing` | `unauthorized` | required operator/admin credential is absent |
| `operator_credential_invalid` | `unauthorized` | operator/admin credential verification failed |
| `operator_authorization_context_missing` | `unauthorized` | required operator/admin authorization context is absent |
| `operator_authorization_context_expired` | `expired` | operator/admin authorization context lifetime has passed |
| `operator_action_denied` | `unauthorized` | operator/admin policy denies the requested privileged action |
| `operator_scope_not_allowed` | `forbidden_state` | requested operator/admin target scope is not allowed |

### 10.9 TURN Reasons

| Code | Category | Meaning |
|---|---|---|
| `malformed_turn_message` | `malformed_input` | message cannot be decoded to core model |
| `unsupported_turn_method` | `unsupported_version` | method is not supported by contract |
| `unsupported_turn_contract_version` | `unsupported_version` | TURN contract version is not accepted |
| `credential_missing` | `unauthorized` | required credential proof absent |
| `credential_invalid` | `unauthorized` | credential verification failed |
| `credential_expired` | `expired` | credential lifetime exceeded |
| `allocation_not_found` | `forbidden_state` | active allocation required but absent or not active |
| `permission_not_found` | `forbidden_state` | active permission required but absent or not active |
| `relay_denied` | `forbidden_state` | relay decision denies packet |
| `turn_lifetime_violation` | `forbidden_state` | requested lifetime violates policy before activation |
| `allocation_capacity_exceeded` | `resource_exhausted` | allocation table bound is reached |
| `allocation_lifetime_exceeded` | `expired` | allocation absolute lifetime cap is reached |
| `permission_capacity_exceeded` | `resource_exhausted` | permission table bound is reached |
| `permission_lifetime_exceeded` | `expired` | permission absolute lifetime cap is reached |
| `channel_bind_lifetime_exceeded` | `expired` | channel binding absolute lifetime cap is reached |
| `refresh_limit_exceeded` | `expired` | refresh count or cumulative refresh duration cap is reached |
| `peer_not_allowed` | `forbidden_state` | peer permission policy rejects the peer |
| `secret_generation_not_accepted` | `unauthorized` | credential generation is not accepted by active rotation policy |
| `secret_key_revoked` | `unauthorized` | credential key or generation has been revoked |
| `secret_overlap_window_expired` | `expired` | credential overlap window for prior generation has expired |
| `secret_rotation_state_unavailable` | `driver_failure` | secret rotation state cannot be observed at the verification boundary |

### 10.10 SFU Reasons

| Code | Category | Meaning |
|---|---|---|
| `participant_not_admitted` | `forbidden_state` | endpoint is not admitted |
| `sfu_session_not_accepting` | `shutdown` | SFU session is draining or closed for new decision |
| `endpoint_quality_not_allowed` | `quality_violation` | endpoint admission is rejected by quality policy |
| `endpoint_degraded_by_quality` | `quality_violation` | admitted endpoint is degraded by quality policy |
| `publication_not_allowed` | `forbidden_state` | publication is rejected |
| `publication_quality_not_allowed` | `quality_violation` | publication is rejected or suppressed by quality policy |
| `subscription_not_allowed` | `forbidden_state` | subscription is rejected |
| `subscription_quality_not_allowed` | `quality_violation` | subscription is rejected or suppressed by quality policy |
| `subscription_backpressure_suppressed` | `backpressure` | subscription is suppressed by backpressure policy |
| `action_delayed_by_backpressure` | `backpressure` | action is delayed by backpressure policy |
| `route_degraded_by_backpressure` | `backpressure` | route is degraded by backpressure policy |
| `endpoint_closed_by_backpressure` | `backpressure` | endpoint is closed by backpressure policy |
| `backpressure_recovery_not_allowed` | `backpressure` | recovery from backpressure-delayed, backpressure-degraded, or backpressure-suppressed route state is rejected |
| `route_suppressed_by_backpressure` | `backpressure` | route state is suppressed by backpressure policy |
| `stream_not_found` | `forbidden_state` | stream reference is not active |
| `route_conflict` | `forbidden_state` | route decision conflicts with state |
| `packet_suppressed_by_backpressure` | `backpressure` | packet forwarding suppressed |
| `packet_dropped_by_backpressure` | `backpressure` | packet forwarding dropped by backpressure policy |
| `packet_suppressed_by_quality` | `quality_violation` | packet forwarding suppressed |
| `route_suppressed_by_quality` | `quality_violation` | route state is suppressed by quality policy |
| `packet_cache_bound_exceeded` | `resource_exhausted` | bounded cache cannot retain packet |
| `target_unavailable` | `forbidden_state` | target endpoint cannot receive |
| `endpoint_capacity_exceeded` | `resource_exhausted` | endpoint admission bound is reached |
| `route_candidate_bound_exceeded` | `resource_exhausted` | route candidate bound is reached |
| `quality_recovery_not_allowed` | `quality_violation` | recovery is rejected by quality policy |
| `unsupported_media_contract_version` | `unsupported_version` | media-facing contract version is not accepted |
| `media_codec_not_supported` | `unsupported_version` | negotiated codec is not supported by core routing semantics |
| `media_track_not_allowed` | `forbidden_state` | requested track direction/kind is not allowed |
| `media_layer_not_available` | `forbidden_state` | requested simulcast/SVC layer is not available for forwarding |
| `media_payload_mapping_invalid` | `malformed_input` | RTP payload type or header-extension mapping cannot be trusted |
| `media_feedback_not_supported` | `unsupported_version` | RTCP feedback capability is not supported by the contract |
| `media_transcode_not_supported` | `unsupported_version` | requested operation requires transcoding, which core does not own |

### 10.11 Packet Rewrite / Media Transform Reasons

| Code | Category | Meaning |
|---|---|---|
| `packet_rewrite_class_not_admitted` | `forbidden_state` | packet rewrite or transform class is not admitted |
| `packet_rewrite_intent_invalid` | `malformed_input` | core rewrite intent is missing required fields or conflicts with route |
| `packet_rewrite_owner_violation` | `forbidden_state` | rewrite path tries to move routing/quality or byte ownership to the wrong layer |
| `payload_transform_not_admitted` | `forbidden_state` | payload transform was requested without admitted specification |
| `payload_transform_failed` | `driver_failure` | driver payload transform, rewrite, or encode step failed |
| `rewrite_copy_bound_exceeded` | `resource_exhausted` | rewrite path copy/allocation bound was exceeded |

### 10.12 ICE Candidate / Connectivity Reasons

| Code | Category | Meaning |
|---|---|---|
| `ice_candidate_policy_violation` | `forbidden_state` | ICE candidate class or exposure is not allowed by policy |
| `ice_candidate_mapping_invalid` | `malformed_input` | ICE candidate cannot map to core-owned reference |
| `ice_candidate_redaction_required` | `malformed_input` | ICE candidate material cannot be used until redacted or rejected |
| `ice_gathering_failed` | `driver_failure` | concrete ICE gathering failed before accepted evidence |
| `ice_connectivity_check_failed` | `driver_failure` | driver observed ICE connectivity check failure |
| `ice_consent_expired` | `expired` | ICE consent freshness expired or failed |
| `ice_restart_not_allowed` | `forbidden_state` | ICE restart is not allowed for current state or policy |

### 10.13 Secure Media Session Reasons

| Code | Category | Meaning |
|---|---|---|
| `secure_media_profile_not_supported` | `unsupported_version` | required secure media profile or version is not supported |
| `secure_media_handshake_failed` | `driver_failure` | DTLS/SRTP handshake failed in driver/backend |
| `secure_media_peer_verification_failed` | `unauthorized` | secure media peer verification failed |
| `secure_media_protection_not_active` | `forbidden_state` | protected media path is required but not active |
| `secure_media_key_state_invalid` | `unauthorized` | secure media key or generation state cannot be trusted |
| `secure_media_session_expired` | `expired` | secure media session lifetime has expired |
| `secure_media_rekey_required` | `forbidden_state` | secure media session requires rekey before continuing |

### 10.14 Public Endpoint / Connection Lifecycle Reasons

| Code | Category | Meaning |
|---|---|---|
| `public_endpoint_not_allowed` | `forbidden_state` | endpoint class is not admitted as public surface |
| `public_endpoint_version_unsupported` | `unsupported_version` | public endpoint protocol or contract version is not accepted |
| `public_endpoint_auth_required` | `unauthorized` | required public endpoint authentication is absent |
| `public_endpoint_upgrade_failed` | `driver_failure` | protocol upgrade or handshake failed before core entry |
| `connection_lifecycle_violation` | `forbidden_state` | connection state transition is invalid for the lifecycle policy |
| `connection_idle_timeout` | `expired` | connection idle, consent, or public lifetime window expired |
| `connection_close_policy_violation` | `forbidden_state` | close path cannot satisfy required policy, audit, or reference material |

### 10.15 Edge / Proxy Trust Reasons

| Code | Category | Meaning |
|---|---|---|
| `edge_proxy_not_admitted` | `forbidden_state` | edge or proxy class is not admitted |
| `forwarded_header_untrusted` | `malformed_input` | forwarded or trusted header cannot be trusted for the target decision |
| `forwarded_header_chain_invalid` | `malformed_input` | forwarded header chain exceeds hop policy or conflicts |
| `origin_host_not_allowed` | `forbidden_state` | origin, host, authority, or SNI value is not allowed by policy |
| `client_address_untrusted` | `malformed_input` | client address observation cannot be trusted for target decision |
| `tls_termination_boundary_invalid` | `forbidden_state` | TLS termination or downstream security relation is invalid |
| `public_internal_route_confusion` | `forbidden_state` | edge/proxy mapping confuses public and internal route classes |

### 10.16 Resource Bound Reasons

| Code | Category | Meaning |
|---|---|---|
| `signaling_command_queue_bound_exceeded` | `resource_exhausted` | signaling command queue capacity or wait limit is reached |
| `sfu_transmit_queue_bound_exceeded` | `resource_exhausted` | SFU transmit queue capacity or wait limit is reached |
| `turn_relay_queue_bound_exceeded` | `resource_exhausted` | TURN relay queue capacity or wait limit is reached |
| `frame_size_bound_exceeded` | `resource_exhausted` | inbound frame size bound is exceeded |
| `admission_capacity_exceeded` | `resource_exhausted` | admission capacity is reached |
| `retention_duration_exceeded` | `expired` | bounded retention duration is exceeded |
| `audit_backlog_bound_exceeded` | `resource_exhausted` | audit sink backlog bound is reached |
| `metrics_backlog_bound_exceeded` | `resource_exhausted` | metrics export backlog bound is reached |
| `metric_cardinality_exceeded` | `resource_exhausted` | metric label/cardinality policy bound is exceeded |
| `buffer_pool_bound_exceeded` | `resource_exhausted` | driver receive buffer pool bound is reached |
| `persistence_retry_bound_exceeded` | `resource_exhausted` | persistence retry store entry count, retry count, or bytes bound is reached |
| `persistence_retry_duration_exceeded` | `expired` | persistence retry duration bound is exceeded |
| `connection_concurrency_exceeded` | `resource_exhausted` | connection concurrency bound is reached |
| `memory_pressure_exceeded` | `resource_exhausted` | memory pressure bound is reached |
| `lock_contention_bound_exceeded` | `resource_exhausted` | bounded serialization queue or lock wait is exhausted |

### 10.17 Driver Conversion Reasons

| Code | Category | Meaning |
|---|---|---|
| `external_decode_failed` | `malformed_input` | external payload cannot map to core type |
| `unsupported_driver_wire_version` | `unsupported_version` | driver wire encoding version is not accepted |
| `missing_required_wire_field` | `malformed_input` | required external field is absent before mapping |
| `external_enum_unmapped` | `malformed_input` | external enum value has no core mapping |
| `external_type_leak_blocked` | `driver_failure` | boundary detected concrete external type leakage before core entry |
| `external_encode_failed` | `driver_failure` | core event cannot be encoded externally |
| `network_send_failed` | `driver_failure` | concrete send failed |
| `network_receive_failed` | `driver_failure` | concrete receive failed |
| `persistence_unavailable` | `driver_failure` | concrete persistence unavailable |
| `metrics_export_failed` | `driver_failure` | concrete metrics export failed |
| `buffer_release_failed` | `driver_failure` | driver buffer release failed |
| `driver_shutdown` | `shutdown` | driver shutdown ended the operation |

### 10.18 Configuration Reasons

| Code | Category | Meaning |
|---|---|---|
| `core_policy_config_invalid` | `malformed_input` | typed core policy configuration is invalid |
| `runtime_config_missing` | `malformed_input` | required runtime configuration is absent |
| `runtime_config_invalid` | `malformed_input` | runtime configuration cannot initialize selected driver/entrypoint |
| `secret_unavailable` | `driver_failure` | required secret source is unavailable without exposing the secret |
| `secret_rotation_required` | `forbidden_state` | active configuration requires a newer secret generation before use |
| `deployment_topology_unsupported` | `unsupported_version` | selected deployment topology is not supported by this entrypoint/service boundary |
| `service_discovery_unavailable` | `driver_failure` | selected service discovery source is unavailable |
| `service_discovery_source_not_admitted` | `forbidden_state` | selected service discovery source class is not admitted |
| `service_endpoint_resolution_failed` | `driver_failure` | endpoint cannot be resolved for target service |
| `service_endpoint_stale` | `expired` | cached or generation-scoped service endpoint is stale |
| `service_endpoint_fallback_not_allowed` | `forbidden_state` | fallback endpoint is not admitted by policy |
| `service_endpoint_scope_conflict` | `forbidden_state` | resolved endpoint does not match expected scope |
| `service_endpoint_contract_mismatch` | `unsupported_version` | resolved endpoint contract or version is not accepted |
| `node_affinity_required` | `forbidden_state` | command requires an affinity or node-local owner that is absent |
| `node_state_unavailable` | `driver_failure` | required node-local state cannot be observed |
| `cross_node_route_not_allowed` | `forbidden_state` | route would cross a topology boundary not allowed by contract |
| `distributed_state_not_admitted` | `forbidden_state` | distributed state class is not admitted |
| `state_replication_not_admitted` | `forbidden_state` | state replication was requested without admitted policy |
| `consensus_not_admitted` | `forbidden_state` | consensus or leader election was requested without admitted policy |
| `failover_not_proven` | `forbidden_state` | failover was requested or claimed without admitted policy/evidence |
| `state_owner_conflict` | `ordering_violation` | multiple owners conflict for the same state scope |
| `split_brain_risk_detected` | `ordering_violation` | split-brain risk is detected for owner-scoped state |
| `replication_lag_bound_exceeded` | `expired` | replication lag or handoff window exceeded accepted bound |
| `runtime_reconfiguration_not_allowed` | `forbidden_state` | runtime reconfiguration class is not admitted |
| `configuration_generation_missing` | `malformed_input` | required configuration generation reference is absent |
| `configuration_generation_conflict` | `ordering_violation` | proposed configuration generation conflicts with active generation or order |
| `runtime_reconfiguration_validation_failed` | `malformed_input` | proposed runtime generation failed validation |
| `runtime_reconfiguration_apply_not_allowed` | `forbidden_state` | runtime reconfiguration apply is not allowed for target surface or active scope |
| `runtime_reconfiguration_drain_required` | `forbidden_state` | drain or restart is required before runtime reconfiguration can apply |
| `runtime_reconfiguration_rollback_required` | `forbidden_state` | rollback is required before the change can be treated as evidence |
| `runtime_reconfiguration_rollback_failed` | `driver_failure` | runtime reconfiguration rollback execution failed |

### 10.19 Internal Control Plane Reasons

| Code | Category | Meaning |
|---|---|---|
| `internal_control_message_invalid` | `malformed_input` | internal control message cannot map to contract |
| `internal_control_version_unsupported` | `unsupported_version` | internal control contract version is not accepted |
| `internal_control_authorization_missing` | `unauthorized` | required internal control authorization context is absent |
| `internal_control_authorization_denied` | `unauthorized` | internal control authorization policy denied the request |

### 10.20 Internal Service Identity / Trust Reasons

| Code | Category | Meaning |
|---|---|---|
| `internal_service_identity_source_not_admitted` | `forbidden_state` | internal service trust class or identity source is not admitted |
| `internal_service_identity_missing` | `unauthorized` | required internal service identity is absent |
| `internal_service_identity_invalid` | `unauthorized` | service identity material cannot be mapped or verified |
| `internal_service_identity_untrusted` | `unauthorized` | service identity cannot be trusted for the target path |
| `internal_service_identity_scope_conflict` | `forbidden_state` | service identity scope does not match target service, topology, or contract |
| `internal_service_peer_verification_failed` | `unauthorized` | internal service peer verification failed |
| `internal_service_credential_expired` | `expired` | service credential or peer proof lifetime has expired |
| `internal_service_trust_policy_missing` | `malformed_input` | required service trust policy is absent |

### 10.21 Operation Lifecycle Reasons

| Code | Category | Meaning |
|---|---|---|
| `operation_deadline_exceeded` | `expired` | command or operation deadline was exceeded |
| `operation_cancelled` | `shutdown` | operation was cancelled after the relevant boundary |
| `capability_not_enabled` | `forbidden_state` | required capability or gated surface is not enabled |

### 10.22 Runtime Task / Worker Lifecycle Reasons

| Code | Category | Meaning |
|---|---|---|
| `runtime_task_class_not_admitted` | `forbidden_state` | runtime task or worker class is not admitted |
| `runtime_task_owner_violation` | `forbidden_state` | task owner or supervision scope violates boundary |
| `runtime_task_supervision_missing` | `malformed_input` | required task supervision scope is absent |
| `runtime_task_detached_not_allowed` | `forbidden_state` | detached task execution is not admitted |
| `runtime_task_spawn_failed` | `driver_failure` | runtime could not spawn required task |
| `runtime_task_join_failed` | `driver_failure` | task join or wait observation failed |
| `runtime_task_cancel_failed` | `driver_failure` | task cancellation failed or could not be observed |
| `runtime_task_panic_detected` | `shutdown` | runtime task or worker panic was observed |
| `runtime_task_queue_bound_exceeded` | `resource_exhausted` | runtime task queue, worker mailbox, join wait, or cancellation wait bound is exhausted |

### 10.23 Operational / Evidence Boundary Reasons

| Code | Category | Meaning |
|---|---|---|
| `readiness_not_satisfied` | `forbidden_state` | readiness component required for the claim is not satisfied |
| `health_probe_unavailable` | `driver_failure` | health/readiness probe cannot execute due entrypoint/driver failure |
| `maintenance_mode_active` | `shutdown` | maintenance mode blocks the requested action |
| `admin_action_not_allowed` | `forbidden_state` | operator/admin action is not allowed by boundary or policy |
| `measurement_normalization_failed` | `malformed_input` | raw measurement cannot be normalized to the required unit |
| `time_observation_unavailable` | `driver_failure` | required time observation is unavailable |
| `atomic_commit_failed` | `driver_failure` | commit step after core decision failed |
| `compensation_required` | `driver_failure` | compensation is required but no successful compensation exists yet |
| `compensation_failed` | `driver_failure` | compensation execution failed |
| `canonical_serialization_failed` | `driver_failure` | canonical encoding could not be produced |
| `canonical_serialization_mismatch` | `malformed_input` | deterministic canonical verification did not match expected digest/hash |
| `process_panic_detected` | `shutdown` | process panic was observed |
| `process_crash_detected` | `shutdown` | process crash was observed |
| `unclean_shutdown_detected` | `shutdown` | prior shutdown lacks graceful drain/audit completion evidence |
| `supervisor_restart_observed` | `shutdown` | supervisor restart was observed and must not imply readiness |
| `session_resumption_not_allowed` | `forbidden_state` | SDK/server session resumption is not allowed by contract |
| `sdk_reconnect_exhausted` | `expired` | SDK local reconnect attempts or window exhausted |
| `fixture_invalid` | `malformed_input` | fixture/scenario data shape is invalid for the test |
| `fixture_redaction_required` | `malformed_input` | fixture/scenario data cannot be used until redacted |
| `observability_signal_invalid` | `malformed_input` | telemetry/audit/log signal does not match the closed taxonomy |
| `telemetry_sampling_policy_missing` | `malformed_input` | required sampling policy is absent for the signal class |
| `alert_signal_not_allowed` | `forbidden_state` | alerting signal is not allowed for the evidence or runtime class |
| `observability_export_not_allowed` | `forbidden_state` | export sink or signal class is not allowed by policy |
| `dependency_policy_violation` | `forbidden_state` | dependency does not satisfy allowlist or ownership policy |
| `dependency_missing` | `malformed_input` | required dependency or tool is absent for an evidence or build gate |
| `license_policy_violation` | `forbidden_state` | dependency license is not accepted by policy |
| `vulnerability_gate_failed` | `forbidden_state` | vulnerability policy gate rejected the dependency set |
| `toolchain_version_mismatch` | `malformed_input` | toolchain version does not match the accepted profile |
| `lockfile_drift_detected` | `malformed_input` | lockfile or generated dependency material differs from accepted state |
| `sdk_contract_generation_failed` | `driver_failure` | SDK public API contract generation failed before it could be treated as evidence |
| `sdk_contract_drift_detected` | `malformed_input` | generated SDK public API artifact differs from source contract |
| `sdk_public_api_unmapped` | `malformed_input` | public SDK API surface lacks a source contract mapping |
| `sdk_golden_mismatch` | `malformed_input` | SDK generated fixture/golden differs from canonical source |
| `sdk_platform_projection_invalid` | `malformed_input` | platform projection cannot preserve the canonical contract semantics |
| `feature_out_of_scope` | `forbidden_state` | requested feature is outside v0.2 initial scope |
| `feature_admission_not_documented` | `forbidden_state` | requested feature lacks required specification admission |
| `chat_not_supported` | `forbidden_state` | chat semantics are not supported in v0.2 initial scope |
| `recording_not_supported` | `forbidden_state` | recording workflow is not supported in v0.2 initial scope |
| `screen_share_not_supported` | `forbidden_state` | screen share workflow is not supported in v0.2 initial scope |
| `datachannel_not_supported` | `forbidden_state` | DataChannel application semantics are not supported in v0.2 initial scope |
| `ui_workflow_not_supported` | `forbidden_state` | UI/end-user workflow is not supported in v0.2 initial scope |
| `media_capture_not_supported` | `forbidden_state` | media capture workflow is not supported as core/server feature |
| `regulated_workflow_not_supported` | `forbidden_state` | regulated workflow is not supported as generic core feature |

### 10.24 Export / Backup Artifact Reasons

| Code | Category | Meaning |
|---|---|---|
| `export_surface_not_allowed` | `forbidden_state` | export or backup surface is not admitted |
| `export_redaction_required` | `malformed_input` | artifact cannot be adopted until redacted or rejected |
| `backup_artifact_unavailable` | `driver_failure` | artifact cannot be generated or fetched from driver/tool |
| `artifact_integrity_mismatch` | `malformed_input` | artifact integrity check failed |
| `artifact_restore_not_allowed` | `forbidden_state` | artifact is used as restore/import input without admission |

### 10.25 Release Artifact / Distribution / Provenance Reasons

| Code | Category | Meaning |
|---|---|---|
| `release_artifact_not_built` | `malformed_input` | release artifact for the claimed class was not built |
| `release_artifact_provenance_missing` | `malformed_input` | release artifact provenance lacks required fields |
| `release_artifact_integrity_failed` | `malformed_input` | artifact digest or signature verification failed |
| `release_version_mismatch` | `malformed_input` | source, package, or version does not match claimed release |
| `distribution_channel_not_allowed` | `forbidden_state` | distribution channel is not admitted |

### 10.26 Time Synchronization / Clock Skew Reasons

| Code | Category | Meaning |
|---|---|---|
| `clock_skew_exceeded` | `malformed_input` | measured skew exceeds accepted policy |
| `time_source_untrusted` | `malformed_input` | time source cannot be trusted for the target claim |
| `time_sync_unavailable` | `driver_failure` | required time synchronization observation is unavailable |
| `timestamp_order_untrusted` | `malformed_input` | timestamp order cannot be trusted for the target comparison |

### 10.27 Reason Total

The reason codes in this catalog total **274 unique codes** across the per-category tables in 10.4 through 10.26, all closed sets. There are 12 cross-cutting categories and 54 code-specific metadata overrides (all 54 are a subset of the 274). A code absent from the catalog MUST NOT be added for implementation convenience.

## 11. External Error Mapping

When mapping core reason category/code to HTTP, WebSocket, STUN/TURN, SDK error wrapper, or CLI exit, the owner and exposure control are fixed so the authoritative reason is not lost. The external code is not authoritative. The authoritative failure remains the cataloged reason category/code or the pre-core driver conversion reason.

### 11.1 Mapping Boundary

| Surface | Owner | Rule |
|---|---|---|
| authoritative reason category/code | core | this chapter section 10 |
| reason exposure metadata | core | retryable, safe_to_expose, audit_required |
| external protocol status/wrapper | driver/sdk/cli | projection without losing reason |
| raw driver error detail | driver | non-authoritative and redacted |
| audit event | core model + driver sink | not a substitute for the external response |

### 11.2 Mapping Fields

Every external non-success response that can be emitted MUST define: external surface; external status/wrapper class; correlation reference when safe and available; exposed reason category/code when `safe_to_expose = true`; redacted opaque error reference when `safe_to_expose = false`; audit event relation when audit is required; retry hint only if reason metadata allows retry. Implementation MUST pass these fields through a named input type such as `ExternalErrorProjectionInput`. If the reason is not safe to expose, the external response MUST NOT leak secret/key/token/backend detail.

### 11.3 Surface Mapping

| Surface | Mapping owner | Required preservation |
|---|---|---|
| HTTP | network driver | category/code or opaque error reference |
| WebSocket | network driver | close/error event must preserve traceability |
| STUN/TURN | TURN wire driver | TURN error representation plus core reason relation |
| SDK TypeScript / Android / iOS | sdk | server reason preservation when server-originated |
| CLI | entrypoints/cli | exit classification plus correlation/report reference |
| logs/traces | observability driver | redacted diagnostics only |

Concrete status numbers or wrapper class names are driver/sdk implementation detail until a surface-specific API specification fixes them.

### 11.4 Safe Exposure

| Metadata | External behavior |
|---|---|
| `safe_to_expose = true` | category/code may be exposed |
| `safe_to_expose = false` | expose generic external failure class plus opaque reference |
| `retryable = true` | retry hint may be exposed if surface supports it |
| `audit_required = true` | audit event relation must be recorded or the path cannot use close evidence |

A driver MUST NOT infer safe exposure from exception text.

### 11.5 External Error Mapping Failure

| Failure | Required reason |
|---|---|
| core event cannot be encoded externally | `external_encode_failed` |
| network send failed while emitting error response | `network_send_failed` |
| SDK cannot decode malformed server event | SDK-local closed error and no invented server reason |
| driver shutdown before response emission | `driver_shutdown` |

When error response emission fails, the original reason remains the domain reason and the emission failure is a driver observation.

## 12. Prohibitions

- a driver turns a rejected decision into a success response.
- entrypoints define an alternate result shape per binary.
- a port intent is treated as driver execution success.
- a driver observation overwrites a domain decision without an explicit compensating transition.
- partial success appears without a per-step outcome rule.
- compensation erases the original decision evidence.
- a non-success outcome lacks a cataloged reason.
- a success outcome carries a fake reason.
- `CorrelationId` alone is treated as duplicate command identity.
- a response is replayed after the replay window expiry.
- a driver cache hit is used as authoritative command acceptance.
- SDK reconnect silently creates a server accepted result.
- a conflicting payload replay is accepted as idempotent observation.
- a response replay hides the original reason or audit failure.
- HTTP status or WebSocket close code replaces the core reason.
- unsafe reason detail is exposed because an SDK/platform wrapper expects text.
- a retry hint is exposed for a non-retryable reason.
- an external response claims success after a core non-success decision.
- the SDK invents a server reason for a local decode failure.
- logs/traces are used as the external error authority.
- a decision reason becomes free-text only.
- a driver-local error is not converted into a core reason.
- an audit event does not connect to the reason catalog.
- a code absent from the reason catalog is added for implementation convenience.

## 13. Fail-Closed / Invariants

- A non-success outcome always carries a cataloged reason (fail-closed).
- When a canonical payload digest cannot be produced, fail closed with `canonical_serialization_failed`.
- Response replay only when all conditions hold; otherwise fail closed.
- A reason with `safe_to_expose = false` MUST NOT leak secret/key/token/backend detail externally.
- A reason with `audit_required = true` cannot be used for close evidence unless the audit relation is recorded.
- reason is a closed set; out-of-catalog codes are not allowed.

## 14. Collapse Conditions

- a use case returns a free-form result that the driver interprets semantically.
- an external status code becomes the authoritative outcome.
- the audit event and external response derive from different reasons.
- driver execution failure is hidden after an accepted decision.
- the result shape differs between Signaling, SFU, TURN, SDK, or CLI without a specification update.
- compensation or transaction semantics are inferred from the final external response only.
- the idempotency scope is implicit.
- duplicate handling differs by driver/entrypoint without a specification update.
- the replay store becomes the domain source-of-truth.
- SDK pending replay bypasses server-side command evaluation.
- response replay lacks a prior result reference or canonical payload comparison.
- an external surface loses traceability to the core reason.
- an unsafe reason exposes token/key/backend detail.
- driver exception text becomes the public reason.
- an external encode failure hides the original domain decision.
- SDK platform mapping changes server-origin reason semantics.
- a decision reason becomes free-text only.
- a driver-local error is not converted into a core reason.
- the SDK converts a server-side reason into different semantics.
- an audit event does not connect to the reason catalog.
- a code absent from the reason catalog is added for implementation convenience.
