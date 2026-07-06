# core-security-and-audit

Status: public summary projection
Date: 2026-07-06 JST

## Purpose

This chapter internalizes the current complete specification of the security token verification boundary, authorization context / communication policy, audit event model, and audit hash-chain contract owned by the core of arcRTC v0.2 Kernel, at a granularity sufficient for re-implementation from this chapter alone.

Dependency direction notation: `A <- B` means "B depends on A". The core is free of external I/O dependencies. arcRTC is not an authentication platform; it is a communication infrastructure that holds a verification boundary for externally issued tokens. arcRTC does not issue tokens and does not own user accounts. This chapter makes identity-neutral primitives, verification request / result / decision semantics, authorization context mapping, all fields / types / closed sets of audit events, and hash-chain structure / verification rules / tamper detection core-owned, and separates JWT / JWK / cryptographic library / key fetch / cache / concrete sink as driver-owned. Token verification success, communication authorization success, and operator/admin authorization success are distinct and MUST NOT be conflated.

---

## 1. Security Token Verification

### 1.1 Boundary (owner)

The core owns token verification request / result / decision semantics. The driver owns JWT / JWK / cryptographic library / key fetch / cache implementation. entrypoints wire the verifier implementation.

### 1.2 Core Responsibilities (core-owned, MUST)

The core owns: verification request type; verification result type; closed rejection reason; required claims vocabulary; token expiry decision; audience / issuer policy as abstract rule; participant transport identity mapping rule; authorization context input only as typed communication policy, not entrypoint role semantics.

### 1.3 Driver Responsibilities (driver-owned)

The driver owns: JWT parser; signature verification library; JWK fetch / cache; typed key material / key source config received through entrypoints wiring; HTTP client; cryptographic backend; external error conversion.

### 1.4 Fail-Closed Reasons (closed set)

Token verification has the following closed reason codes. The reason code MUST be a code from the core reason catalog, and category-only reason values are prohibited.

| Failure | Reason code |
|---|---|
| token missing | `token_missing` |
| token malformed | `token_malformed` |
| signature invalid | `token_signature_invalid` |
| key unavailable | `token_key_unavailable` |
| issuer mismatch | `token_issuer_mismatch` |
| audience mismatch | `token_audience_mismatch` |
| expired | `token_expired` |
| not yet valid | `token_not_yet_valid` |
| required claim missing | `token_required_claim_missing` |
| unsupported algorithm | `token_unsupported_algorithm` |

### 1.5 Audit / Wrapper Rule

Rejected token verification MUST emit `token_verification_decision` with the concrete `token_*` reason code. Signaling MAY expose `token_verification_failed` as a public wrapper rejection reason only when the same correlation chain records the concrete `token_*` reason in `token_verification_decision`. `token_verification_failed` MUST NOT replace or erase the concrete token verification reason in audit. Authorization context mapping MUST be recorded separately when a verified token is used as communication policy input.

### 1.6 Prohibitions / Collapse Conditions (token verification)

Prohibitions: arcRTC issues a token; arcRTC owns a user account; core depends on a concrete API such as `jsonwebtoken`; core performs HTTP key fetch; Signaling participant ID is equated with authenticated user ID; entrypoint-specific role from token claims is brought into protocol decision; token verification success is treated as communication authorization success.

Collapse: token issuance is made an arcRTC responsibility; core depends on a JWT concrete library; the driver's external error becomes a core decision as an open-ended string; the driver reads key configuration directly from env / file / process args; domain role authorization is mixed into the generic communication protocol; authorization policy is inferred from token verification without authorization context mapping.

---

## 2. Authorization Context / Communication Policy

### 2.1 Boundary (owner)

This section fixes the boundary that maps a verified credential to generic communication policy input without conflating token verification, core identity, admission, and publication/subscription/TURN permission decisions. This section does not assert user account, role management, tenant billing, or regulated authorization implementation.

| Concern | Owner | Rule |
|---|---|---|
| token verification result | core/security semantics + driver verifier | section 1 of this chapter |
| external auth claims / domain roles | external application or regulated | not generic core identity |
| authorization context mapping | entrypoints/drivers before core policy input | convert to typed opaque context |
| edge/proxy metadata | entrypoints/drivers before authorization mapping | may be used only after edge trust policy admits it |
| communication authorization policy | core typed policy | used in join/publication/subscription/TURN/admission decision |
| regulated authorization | regulated | not a required generic core schema |
| operator/admin authorization | entrypoints/drivers + admin policy | not communication participant authorization |
| internal service identity / trust | entrypoints/drivers + service trust policy | internal control authorization input only |
| audit evidence | core event model and driver sink | authorization context decision |

Token verification success does not automatically authorize join, publication, subscription, TURN relay, or quota admission.

### 2.2 Authorization Context Classes (closed set)

| Class | Meaning | Rule |
|---|---|---|
| `verified_credential_context` | token/key verification result exists | not domain authorization by itself |
| `participant_join_context` | join policy input for room membership | maps to Signaling join decision |
| `publication_policy_context` | SFU publication authorization input | maps to SFU publication decision |
| `subscription_policy_context` | SFU subscription authorization input | maps to SFU subscription decision |
| `turn_relay_policy_context` | TURN allocation/permission/relay authorization input | maps to TURN decisions |
| `admission_policy_context` | rate/quota/admission grouping input | follows rate limit rule (chapter 13) |
| `regulated_authorization_context` | regulated-side policy only | must not become generic core requirement |

A new authorization context class requires a specification update.

### 2.3 Mapping Rule

Authorization mapping MUST declare: source credential or external context class; allowed target decision surfaces; opaque core reference or typed policy input; lifetime and expiry; claim/field redaction rule; failure reason mapping; audit event relation.

Raw token claims, entrypoint user role, tenant role, billing account, facility role, medical role, or regulated subject MUST NOT become generic core identity. Client IP, forwarded header, host, origin, or SNI MUST NOT become authorization context unless an edge trust policy and authorization mapping both admit that use.

### 2.4 Decision Rule

Core MAY use authorization context only as typed communication policy input. Each target surface retains its own decision and reason: Signaling join uses Signaling join decision; SFU publication/subscription uses SFU decision; TURN permission/relay uses TURN decision; quota/admission uses resource/admission decision. Authorization context decision MAY explain why policy input is unavailable or invalid, but it does not replace the target domain decision event.

### 2.5 Failure Mapping (authorization context)

| Failure | Required reason |
|---|---|
| required authorization context missing | `authorization_context_missing` |
| authorization context cannot be mapped to typed policy input | `authorization_context_invalid` |
| authorization context expired | `authorization_context_expired` |
| policy denies requested communication action | `authorization_policy_denied` |
| requested authorization scope not allowed in generic core | `authorization_scope_not_allowed` |
| edge/proxy metadata is not trustworthy for authorization mapping | `forwarded_header_untrusted` or `client_address_untrusted` |
| token verification failed | concrete token reason or `token_verification_failed` wrapper as allowed |
| required authorization configuration missing | `runtime_config_missing` |
| required authorization configuration invalid | `runtime_config_invalid` |

### 2.6 Audit / Evidence Rule

Authorization context mapping decisions use audit event type `authorization_context_decision`. Target domain decisions still use their own audit event types. The same correlation chain MAY include both events, but one does not replace the other.

Evidence for authorization context or policy claims MUST include: source credential/context class; mapped authorization context class; target decision surface; lifetime/expiry observation or deterministic time fixture; redaction statement for raw claims and sensitive auth payload; audit event type `authorization_context_decision`; target domain decision event when the claim includes join, publication, subscription, TURN, or admission behavior; closed reason for missing, invalid, expired, denied, or scope-not-allowed outcomes. Token verification evidence alone does not prove communication authorization. Communication authorization evidence alone does not prove operator/admin authorization. Internal service trust evidence alone does not prove target internal control authorization.

### 2.7 Prohibitions / Collapse Conditions (authorization context)

Prohibitions: token verification success is treated as join/publication/subscription/TURN authorization success; entrypoint-specific role becomes generic core identity; regulated authorization schema becomes required generic core schema; SDK platform wrapper performs server authorization decision; authorization denial is recorded as free-text only; raw claims or sensitive auth payload enter audit/log/report; operator/admin action is authorized by communication participant policy; internal service call is authorized from endpoint reachability or peer verification alone; edge/proxy metadata becomes authorization context without edge trust policy.

Collapse: authorization context owner is implicit; generic core reads entrypoint/regulated role semantics directly; target domain decision is skipped after authorization mapping; authorization context can fail open when required input is absent; audit cannot distinguish token verification, authorization mapping, and target decision; audit cannot distinguish communication authorization and operator/admin authorization; audit cannot distinguish verified authorization context from proxy-derived metadata; audit cannot distinguish internal service trust from target internal control authorization.

---

## 3. Audit Event Model

### 3.1 Boundary (owner)

The core owns the audit event model, hash-chain contract, and audit sink port. drivers own concrete sinks such as file / HTTP / syslog / PostgreSQL / S3. regulated owns optional enrichment. Audit provides observability for the communication infrastructure but does not mix regulated payload into the generic core.

### 3.2 Core Audit Event Fields (required field set)

The core audit event has:

- event ID
- correlation ID
- timestamp
- component
- event type code
- subject transport reference when required by reference rule
- room/session reference when required by reference rule
- startup run ID when required by reference rule
- configuration scope reference when required by reference rule
- authorization context class/reference when required by reference rule
- media negotiation class/reference when required by reference rule
- service topology class/reference when required by reference rule
- observability signal class/reference when required by reference rule
- dependency/toolchain reference when required by reference rule
- SDK platform/projection reference when required by reference rule
- internal control-plane class/reference when required by reference rule
- ICE candidate/connectivity class/reference when required by reference rule
- secure media session class/reference when required by reference rule
- operator/admin authorization class/reference when required by reference rule
- out-of-scope feature class/reference when required by reference rule
- public endpoint and connection lifecycle class/reference when required by reference rule
- export/backup artifact class/reference when required by reference rule
- release artifact/provenance/distribution class/reference when required by reference rule
- time synchronization/clock skew class/reference when required by reference rule
- edge/proxy trust class/reference when required by reference rule
- runtime reconfiguration class/reference when required by reference rule
- packet rewrite/media transform class/reference when required by reference rule
- service discovery/endpoint resolution class/reference when required by reference rule
- distributed state/failover class/reference when required by reference rule
- runtime task/worker class/reference when required by reference rule
- internal service identity/trust class/reference when required by reference rule
- cross-plane identity/session binding class/reference when required by reference rule
- runtime reconfiguration class/generation/reference when required by reference rule
- decision outcome
- reason presence
- reason category when reason presence is `cataloged`
- reason code when reason presence is `cataloged`
- resource policy owner when the audit event is resource-bound related
- physical resource owner when the audit event is resource-bound related
- previous hash reference if hash-chain applies
- event hash
- non-sensitive tags

### 3.3 Construction Input Rule

Audit events and hash-chain records have many fields and MUST NOT be bare multi-argument constructors. The implementation MUST bundle unchecked material into named input types such as `AuditEventInput`, `HashChainRecordInput`, and perform fail-closed checks of reason presence, required reference, resource-bound owner, and hash-chain relation inside the constructor. The input type is not a substitute authority for the audit sink or external response; it is the boundary of unchecked material passed into the core audit model.

### 3.4 Prohibited Fields

The core audit event MUST NOT include: medical record payload; patient identity; entrypoint user profile; domain role as protocol identity; raw credential secret; raw media payload; regulated-specific required schema; fake room/session reference for startup or configuration event. Privacy, redaction, and retention constraints follow the privacy/redaction/retention rule. Canonical event serialization and deterministic digest material follow the canonical serialization rule.

### 3.5 Resource-Bound Owner Field Rule

`resource-bound related` means an audit event instance whose bounded resource, event type, outcome, and reason code tuple matches a row in chapter 11 Required Bound Closed Action Mapping. This includes `resource_bound_decision`, `driver_resource_bound_decision`, and TURN-specific decision event instances when those instances are the mapped bound audit event for TURN allocation, refresh cap, permission, or channel bind bound rows. Every resource-bound related audit event instance MUST carry resource policy owner and physical resource owner fields. Multi-purpose event types do not become resource-bound related for every instance. For example, `turn_allocation_decision` accepted/released events and credential rejection events are not resource-bound related unless their resource, outcome, and reason match a Required Bound Closed Action Mapping row. For TURN allocation / refresh cap / permission / channel bind bound events, resource policy owner is `core` and physical resource owner is `driver`. TURN relay queue bound is not carried by `turn_relay_decision`; it uses `resource_bound_decision` with resource policy owner `core` and physical resource owner `driver`. For inbound frame size bound events, resource policy owner is `driver` and physical resource owner is `driver`. Signaling / SFU domain decision events that also carry a bound reason are supplementary domain decisions. They do not replace the mapped resource-bound audit event and do not satisfy the owner field requirement unless they are the mapped bound audit event type in Required Bound Closed Action Mapping.

### 3.6 Event Type Codes (closed set)

The audit event type is treated as a closed-set code. A free-text category MUST NOT be used as the authority of the event type.

| Event type code | Meaning | Required reason connection |
|---|---|---|
| `signaling_join_decision` | Signaling join accepted/rejected | absent for accepted; required for rejected |
| `signaling_participant_lifecycle_decision` | Signaling participant leave/lifecycle termination accepted/rejected | absent for accepted; required for rejected |
| `signaling_room_lifecycle_decision` | Signaling room drain/close/closed-room observation accepted/rejected | absent for accepted/idempotent observation; required for rejected |
| `signaling_protocol_violation` | Signaling command violates protocol | reason category/code required |
| `signaling_relay_event` | Signaling relay event emitted | absent for forwarded; required for suppressed/rejected relay |
| `turn_allocation_decision` | TURN allocation accepted/rejected/released/expired | absent for accepted/released; required for rejected/expired |
| `turn_refresh_decision` | TURN refresh accepted/rejected/expired by refresh cap | absent for accepted; required for rejected/expired |
| `turn_permission_decision` | TURN permission accepted/rejected/revoked/expired | absent for accepted; required for rejected/revoked/expired |
| `turn_channel_bind_decision` | TURN channel bind accepted/rejected/expired | absent for accepted; required for rejected/expired |
| `turn_relay_decision` | TURN relay allowed/denied | absent for allowed; required for denied |
| `sfu_session_lifecycle_decision` | SFU session drain/close accepted/rejected | absent for accepted; required for rejected |
| `sfu_admission_decision` | SFU endpoint admission accepted/rejected | absent for accepted; required for rejected |
| `sfu_endpoint_lifecycle_decision` | SFU endpoint drain/removal accepted/rejected | absent for accepted; required for rejected |
| `sfu_publication_decision` | SFU publication accepted/rejected/suppressed/closed | absent for accepted/closed; required for rejected/suppressed |
| `sfu_subscription_decision` | SFU subscription accepted/rejected/suppressed/closed | absent for accepted/closed; required for rejected/suppressed |
| `sfu_forwarding_decision` | SFU forwarding selected/rejected/suppressed/dropped/closed/failed | absent for forwarded/selected/successful closed; required for rejected/suppressed/dropped/failed |
| `backpressure_decision` | backpressure accepted/delayed/suppressed/degraded/dropped/closed a target/rejected recovery | absent for accepted; required for the rest |
| `resource_bound_decision` | core-policy bounded resource accepted/rejected/dropped/shed/expired; physical owner is explicit field | absent for accepted/within-bound observation; required for rejected/dropped/shed/expired |
| `driver_resource_bound_decision` | explicitly listed driver-local resource bound decision | required for dropped |
| `configuration_decision` | typed configuration accepted/rejected/failed during startup/wiring | absent for accepted; required for rejected/failed |
| `quality_violation_decision` | quality policy violation or recovery rejection | reason category/code required |
| `token_verification_decision` | token verification accepted/rejected | absent for accepted; required for rejected |
| `driver_error_converted` | driver-local error converted to core reason | reason category/code required |
| `operational_probe_observation` | health/readiness/liveness/admin probe observed | absent for satisfied observation; required for non-satisfied/failed |
| `admin_maintenance_decision` | admin/maintenance action accepted/rejected/failed | absent for accepted; required for rejected/failed |
| `process_lifecycle_observation` | process crash/panic/unclean shutdown/supervisor restart observed | reason category/code required |
| `atomicity_compensation_decision` | commit/compensation path accepted/rejected/failed | absent for accepted; required for rejected/failed |
| `canonical_serialization_verification` | canonical serialization/digest verification observed | absent for matched observation; required for mismatch/failed |
| `sdk_reconnect_observation` | SDK reconnect/resumption observation | absent for accepted; required for rejected/failed/expired |
| `test_fixture_decision` | fixture/scenario data accepted/rejected/failed for evidence use | absent for accepted; required for rejected/failed |
| `command_idempotency_decision` | command idempotency/replay/correlation decision observed | absent for accepted/idempotent observation; required for rejected/expired/failed |
| `authorization_context_decision` | verified authorization context/communication policy accepted/rejected | absent for accepted; required for rejected/expired/failed |
| `deployment_topology_decision` | entrypoint/service deployment topology accepted/rejected/failed | absent for accepted; required for rejected/failed |
| `media_negotiation_decision` | codec/track/layer/payload mapping/feedback negotiation accepted/rejected/suppressed/failed | absent for accepted; required for rejected/suppressed/failed |
| `observability_signal_decision` | telemetry/audit/log/report signal accepted/rejected/dropped/failed | absent for accepted; required for rejected/dropped/failed |
| `secret_rotation_decision` | secret generation/overlap/revocation/rotation state accepted/rejected/expired/revoked/failed | absent for accepted; required for the rest |
| `supply_chain_decision` | dependency/license/vulnerability/lockfile/toolchain gate accepted/rejected/failed | absent for accepted; required for rejected/failed |
| `sdk_public_api_contract_decision` | generated SDK public API contract accepted/rejected/failed against canonical source | absent for accepted; required for rejected/failed |
| `internal_control_plane_decision` | internal service-to-service control accepted/rejected/expired/failed | absent for accepted; required for rejected/expired/failed |
| `ice_candidate_connectivity_decision` | ICE candidate policy/restart/connectivity/consent decision accepted/rejected/expired/failed | absent for accepted; required for rejected/expired/failed |
| `secure_media_session_decision` | DTLS/SRTP secure media session decision accepted/rejected/expired/failed | absent for accepted; required for rejected/expired/failed |
| `operator_admin_authorization_decision` | operator/admin authorization accepted/rejected/expired/failed | absent for accepted; required for rejected/expired/failed |
| `out_of_scope_feature_decision` | excluded/future-admitted feature request accepted/rejected/failed | absent for accepted; required for rejected/failed |
| `public_endpoint_connection_decision` | public endpoint admission/connection lifecycle accepted/rejected/expired/closed/failed | absent for accepted/closed_success; required for rejected/expired/closed_by_policy/failed |
| `export_backup_artifact_decision` | export/backup artifact accepted/rejected/failed | absent for accepted; required for rejected/failed |
| `release_artifact_distribution_decision` | release artifact/distribution accepted/rejected/failed | absent for accepted; required for rejected/failed |
| `time_synchronization_decision` | time synchronization/clock skew observation accepted/rejected/failed/expired | absent for accepted; required for rejected/failed/expired |
| `edge_proxy_trust_decision` | edge/proxy trust/trusted metadata decision accepted/rejected/failed | absent for accepted; required for rejected/failed |
| `runtime_reconfiguration_decision` | runtime reconfiguration generation accepted/rejected/failed; rollback-required expressed by `failed` with cataloged reason | absent for accepted; required for rejected/failed |
| `packet_rewrite_transform_decision` | packet rewrite/media transform intent/execution accepted/rejected/failed | absent for accepted; required for rejected/failed |
| `service_discovery_resolution_decision` | service discovery/endpoint resolution accepted/rejected/expired/failed | absent for accepted; required for rejected/expired/failed |
| `distributed_state_failover_decision` | distributed state/replication admission/owner conflict/failover decision accepted/rejected/failed | absent for accepted; required for rejected/failed |
| `runtime_task_lifecycle_decision` | runtime task/worker lifecycle accepted/rejected/expired/failed | absent for accepted; required for rejected/expired/failed |
| `internal_service_trust_decision` | internal service identity/trust mapping accepted/rejected/expired/failed | absent for accepted; required for rejected/expired/failed |
| `cross_plane_binding_decision` | cross-plane identity/session binding accepted/rejected/expired/failed | absent for accepted; required for rejected/expired/failed |

### 3.7 Decision Outcome Rule (closed set)

The decision outcome is treated as a closed-set code. The outcome per event type MUST be chosen from the Event Outcome Compatibility table in section 3.9. Free-text outcomes are prohibited.

| Decision outcome | Reason presence |
|---|---|
| `accepted` | `none` |
| `allowed` | `none` |
| `forwarded` | `none` |
| `selected` | `none` |
| `released` | `none` |
| `closed_success` | `none` |
| `idempotent_observed` | `none` |
| `within_bound_observed` | `none` |
| `rejected` | `cataloged` |
| `denied` | `cataloged` |
| `suppressed` | `cataloged` |
| `dropped` | `cataloged` |
| `expired` | `cataloged` |
| `revoked` | `cataloged` |
| `failed` | `cataloged` |
| `delayed` | `cataloged` |
| `degraded` | `cataloged` |
| `shed` | `cataloged` |
| `closed_by_policy` | `cataloged` |
| `protocol_violation` | `cataloged` |
| `converted_failure` | `cataloged` |

### 3.8 Reason Presence Rule (closed set)

Audit event reason presence is a closed field.

| reason presence | Meaning | Reason category/code |
|---|---|---|
| `none` | success outcome does not carry a rejection/suppression/failure reason | must be absent |
| `cataloged` | non-success outcome carries a core reason | must be present and match the core reason catalog |

accepted, allowed, forwarded, selected, released, closed_success, idempotent_observed, within_bound_observed outcomes use `reason_presence = none`. rejected, denied, suppressed, dropped, expired, revoked, failed, delayed, degraded, shed, closed_by_policy, protocol_violation, converted_failure outcomes use `reason_presence = cataloged`.

### 3.9 Event Outcome Compatibility Rule (closed set)

The allowed outcomes per event type are limited to the following.

| Event type code | Allowed decision outcomes |
|---|---|
| `signaling_join_decision` | `accepted`, `rejected` |
| `signaling_participant_lifecycle_decision` | `accepted`, `rejected` |
| `signaling_room_lifecycle_decision` | `accepted`, `idempotent_observed`, `rejected` |
| `signaling_protocol_violation` | `protocol_violation` |
| `signaling_relay_event` | `forwarded`, `suppressed`, `rejected` |
| `turn_allocation_decision` | `accepted`, `released`, `rejected`, `expired` |
| `turn_refresh_decision` | `accepted`, `rejected`, `expired` |
| `turn_permission_decision` | `accepted`, `rejected`, `revoked`, `expired` |
| `turn_channel_bind_decision` | `accepted`, `rejected`, `expired` |
| `turn_relay_decision` | `allowed`, `denied` |
| `sfu_session_lifecycle_decision` | `accepted`, `rejected` |
| `sfu_admission_decision` | `accepted`, `rejected` |
| `sfu_endpoint_lifecycle_decision` | `accepted`, `rejected` |
| `sfu_publication_decision` | `accepted`, `rejected`, `suppressed`, `closed_success` |
| `sfu_subscription_decision` | `accepted`, `rejected`, `suppressed`, `closed_success` |
| `sfu_forwarding_decision` | `forwarded`, `selected`, `rejected`, `suppressed`, `dropped`, `closed_success`, `failed` |
| `backpressure_decision` | `accepted`, `delayed`, `suppressed`, `degraded`, `dropped`, `closed_by_policy`, `rejected` |
| `resource_bound_decision` | `accepted`, `within_bound_observed`, `rejected`, `dropped`, `shed`, `expired` |
| `driver_resource_bound_decision` | `dropped` |
| `configuration_decision` | `accepted`, `rejected`, `failed` |
| `quality_violation_decision` | `rejected`, `suppressed`, `degraded` |
| `token_verification_decision` | `accepted`, `rejected` |
| `driver_error_converted` | `converted_failure` |
| `operational_probe_observation` | `within_bound_observed`, `rejected`, `failed` |
| `admin_maintenance_decision` | `accepted`, `rejected`, `failed` |
| `process_lifecycle_observation` | `within_bound_observed`, `failed` |
| `atomicity_compensation_decision` | `accepted`, `rejected`, `failed` |
| `canonical_serialization_verification` | `within_bound_observed`, `failed` |
| `sdk_reconnect_observation` | `accepted`, `rejected`, `failed`, `expired` |
| `test_fixture_decision` | `accepted`, `rejected`, `failed` |
| `command_idempotency_decision` | `accepted`, `idempotent_observed`, `rejected`, `expired`, `failed` |
| `authorization_context_decision` | `accepted`, `rejected`, `expired`, `failed` |
| `deployment_topology_decision` | `accepted`, `rejected`, `failed` |
| `media_negotiation_decision` | `accepted`, `rejected`, `suppressed`, `failed` |
| `observability_signal_decision` | `accepted`, `rejected`, `dropped`, `failed` |
| `secret_rotation_decision` | `accepted`, `rejected`, `expired`, `revoked`, `failed` |
| `supply_chain_decision` | `accepted`, `rejected`, `failed` |
| `sdk_public_api_contract_decision` | `accepted`, `rejected`, `failed` |
| `internal_control_plane_decision` | `accepted`, `rejected`, `expired`, `failed` |
| `ice_candidate_connectivity_decision` | `accepted`, `rejected`, `expired`, `failed` |
| `secure_media_session_decision` | `accepted`, `rejected`, `expired`, `failed` |
| `operator_admin_authorization_decision` | `accepted`, `rejected`, `expired`, `failed` |
| `out_of_scope_feature_decision` | `accepted`, `rejected`, `failed` |
| `public_endpoint_connection_decision` | `accepted`, `rejected`, `expired`, `closed_success`, `closed_by_policy`, `failed` |
| `export_backup_artifact_decision` | `accepted`, `rejected`, `failed` |
| `release_artifact_distribution_decision` | `accepted`, `rejected`, `failed` |
| `time_synchronization_decision` | `accepted`, `rejected`, `expired`, `failed` |
| `edge_proxy_trust_decision` | `accepted`, `rejected`, `failed` |
| `runtime_reconfiguration_decision` | `accepted`, `rejected`, `failed` |
| `packet_rewrite_transform_decision` | `accepted`, `rejected`, `failed` |
| `service_discovery_resolution_decision` | `accepted`, `rejected`, `expired`, `failed` |
| `distributed_state_failover_decision` | `accepted`, `rejected`, `failed` |
| `runtime_task_lifecycle_decision` | `accepted`, `rejected`, `expired`, `failed` |
| `internal_service_trust_decision` | `accepted`, `rejected`, `expired`, `failed` |
| `cross_plane_binding_decision` | `accepted`, `rejected`, `expired`, `failed` |

### 3.10 Reference Presence Rule (closed set)

Audit reference presence is closed per event type. `absent_not_applicable` is the only allowed absence marker. If a required reference type has not yet been materialized at the point of rejection/failure, that reference field MUST be `absent_not_applicable`; the event MUST still carry the cataloged reason explaining the pre-materialization rejection/failure. For pre-core driver conversion failure or pre-core driver resource-bound failure where client-supplied `CorrelationId` cannot be decoded or validated, the `CorrelationId` reference field follows this pre-materialization rule and MUST be `absent_not_applicable`. The driver MUST NOT synthesize a fake client `CorrelationId`. After a command crosses the driver/core boundary, `CorrelationId` is mandatory and MUST NOT be `absent_not_applicable`.

| Event type code | Required references | References that must be `absent_not_applicable` when not naturally present |
|---|---|---|
| `configuration_decision` | `StartupRunId`, `ConfigurationScopeRef`, `CorrelationId` | subject transport reference, room/session reference |
| `signaling_join_decision` | `CorrelationId`, `RoomId`, `ParticipantId` when participant is materialized | startup run ID, configuration scope reference |
| `signaling_participant_lifecycle_decision` | `CorrelationId`, `RoomId`, `ParticipantId` | startup run ID, configuration scope reference |
| `signaling_room_lifecycle_decision` | `CorrelationId`, `RoomId` | startup run ID, configuration scope reference |
| `signaling_protocol_violation` | `CorrelationId`, subject transport reference when command reached transport boundary | startup run ID, configuration scope reference |
| `signaling_relay_event` | `CorrelationId`, `RoomId`, `ParticipantId` | startup run ID, configuration scope reference |
| `turn_allocation_decision` | `CorrelationId`, `AllocationId` when materialized, `CredentialRef` when credential was present, resource policy owner and physical resource owner when reason is allocation capacity or allocation lifetime bound | room/session reference unless the TURN path is room-scoped by Signaling contract |
| `turn_refresh_decision` | `CorrelationId`, `AllocationId`, `CredentialRef` when credential was present, owner tuple when reason is refresh bound | room/session reference unless room-scoped |
| `turn_permission_decision` | `CorrelationId`, `AllocationId`, `PermissionId` when materialized, owner tuple when reason is permission capacity or permission lifetime bound | room/session reference unless room-scoped |
| `turn_channel_bind_decision` | `CorrelationId`, `AllocationId`, `PermissionId`, `ChannelBindId` when materialized, owner tuple when reason is channel bind lifetime bound | room/session reference unless room-scoped |
| `turn_relay_decision` | `CorrelationId`, `AllocationId`, `PermissionId` when materialized | room/session reference unless room-scoped |
| `sfu_session_lifecycle_decision` | `CorrelationId`, `SessionId` | startup run ID, configuration scope reference |
| `sfu_admission_decision` | `CorrelationId`, `SessionId`, `EndpointId` when materialized | startup run ID, configuration scope reference |
| `sfu_endpoint_lifecycle_decision` | `CorrelationId`, `SessionId`, `EndpointId` | startup run ID, configuration scope reference |
| `sfu_publication_decision` | `CorrelationId`, `SessionId`, `EndpointId`, `StreamId` when materialized | startup run ID, configuration scope reference |
| `sfu_subscription_decision` | `CorrelationId`, `SessionId`, `EndpointId`, `StreamId` when materialized | startup run ID, configuration scope reference |
| `sfu_forwarding_decision` | `CorrelationId`, `SessionId`, `EndpointId` when target-scoped, `RouteId` when materialized, `PacketId` when packet-scoped | startup run ID, configuration scope reference |
| `backpressure_decision` | `CorrelationId`, affected resource or route reference, `EndpointId` when endpoint-scoped, `PacketId` when packet-scoped | startup run ID, configuration scope reference |
| `resource_bound_decision` | `CorrelationId`, resource name, resource policy owner, physical resource owner, `PacketId` when packet-scoped, `AllocationId` and `PermissionId` when resource is TURN relay queue | startup run ID and configuration scope reference unless the resource is configuration-scoped |
| `driver_resource_bound_decision` | `CorrelationId`, driver resource reference, resource policy owner, physical resource owner | room/session reference is `absent_not_applicable` unless a validated `RoomId` or `SessionId` already exists for the same correlation chain |
| `quality_violation_decision` | `CorrelationId`, quality target reference, `PacketId` when packet-scoped | startup run ID, configuration scope reference |
| `token_verification_decision` | `CorrelationId`, `CredentialRef` when credential was present | startup run ID, configuration scope reference |
| `driver_error_converted` | `CorrelationId`, driver error source reference, `PacketId` when packet lifecycle failure is converted | room/session reference is `absent_not_applicable` unless a validated `RoomId` or `SessionId` already exists for the same correlation chain |
| `operational_probe_observation` | `StartupRunId`, probe class, entrypoint reference, `CorrelationId` when command-scoped | room/session reference unless probe is explicitly domain-scoped |
| `admin_maintenance_decision` | `StartupRunId`, admin action reference, `CorrelationId` | room/session reference unless action is explicitly domain-scoped |
| `process_lifecycle_observation` | `StartupRunId`, process reference, failure class | room/session reference |
| `atomicity_compensation_decision` | `CorrelationId`, atomicity class, commit boundary reference | startup run ID unless startup-scoped |
| `canonical_serialization_verification` | `CorrelationId`, canonical format/version, digest/hash reference | room/session reference unless serialized object is domain-scoped |
| `sdk_reconnect_observation` | `CorrelationId`, SDK platform, reconnect class | startup run ID, configuration scope reference |
| `test_fixture_decision` | `CorrelationId`, fixture class, fixture reference | room/session reference unless fixture is domain-scoped |
| `command_idempotency_decision` | `CorrelationId`, command type, idempotency class, idempotency scope, target reference when materialized | startup run ID, configuration scope reference |
| `authorization_context_decision` | `CorrelationId`, authorization context class, target surface, `CredentialRef` when credential was present | startup run ID, configuration scope reference |
| `deployment_topology_decision` | `StartupRunId`, topology class, entrypoint/service reference, `CorrelationId` when command-scoped | room/session reference unless topology decision is domain-scoped |
| `media_negotiation_decision` | `CorrelationId`, media negotiation class, `EndpointId` when materialized, `StreamId` when materialized | startup run ID, configuration scope reference |
| `observability_signal_decision` | `CorrelationId`, signal class, signal reference, resource policy owner and physical resource owner when reason is cardinality or export bound | room/session reference unless signal is domain-scoped |
| `secret_rotation_decision` | `StartupRunId`, secret class, opaque generation reference, `CorrelationId` when command-scoped | room/session reference unless rotation decision is domain-scoped |
| `supply_chain_decision` | `CorrelationId`, package/toolchain reference, dependency class | room/session reference |
| `sdk_public_api_contract_decision` | `CorrelationId`, SDK platform, projection class, source contract version | room/session reference |
| `internal_control_plane_decision` | `CorrelationId`, source service, target service, control-plane class, contract version, topology class when available | room/session reference unless control path is domain-scoped |
| `ice_candidate_connectivity_decision` | `CorrelationId`, candidate/connectivity class, ICE policy reference, `RoomId` and `ParticipantId` when materialized | startup run ID, configuration scope reference |
| `secure_media_session_decision` | `CorrelationId`, secure media session class, media/security contract version, `SessionId` and `EndpointId` when materialized, redacted key/certificate reference when applicable | startup run ID, configuration scope reference |
| `operator_admin_authorization_decision` | `CorrelationId`, operator/admin class, action class, target scope, `StartupRunId` when startup/entrypoint-scoped | room/session reference unless action is explicitly domain-scoped |
| `out_of_scope_feature_decision` | `CorrelationId` when command-scoped, feature class, requested surface | room/session reference unless request is domain-scoped |
| `public_endpoint_connection_decision` | endpoint class, protocol class, target contract reference, connection lifecycle state, `CorrelationId` when command-scoped, `StartupRunId` when startup/listener-scoped | room/session reference unless endpoint decision is domain-scoped |
| `export_backup_artifact_decision` | `CorrelationId`, artifact class, source scope, redaction class, retention class, storage/tool owner, integrity reference when present | room/session reference unless artifact is domain-scoped |
| `release_artifact_distribution_decision` | `CorrelationId`, artifact class, source ref, package/module reference, artifact digest reference, distribution channel, provenance class | room/session reference |
| `time_synchronization_decision` | `StartupRunId`, node scope, time trust class, time source class, skew policy reference, observed skew class, `CorrelationId` when command-scoped | room/session reference unless time decision is domain-scoped |
| `edge_proxy_trust_decision` | `StartupRunId`, edge class, topology class, metadata class, trust policy reference, target endpoint class when available, `CorrelationId` when command-scoped | room/session reference unless edge decision is domain-scoped |
| `runtime_reconfiguration_decision` | `StartupRunId`, reconfiguration class, target surface, current generation reference, proposed generation reference, apply scope, drain/restart class, rollback class when applicable, `CorrelationId` when command-scoped | room/session reference unless reconfiguration target is domain-scoped |
| `packet_rewrite_transform_decision` | `CorrelationId`, `PacketId` when materialized, route/target reference when materialized, rewrite/transform class, copy allowance class, execution owner | room/session reference unless packet decision is domain-scoped |
| `service_discovery_resolution_decision` | `StartupRunId`, discovery source class, topology class, target service, endpoint scope, resolution state, fallback class when applicable, `CorrelationId` when command-scoped | room/session reference unless resolution decision is domain-scoped |
| `distributed_state_failover_decision` | `StartupRunId`, distributed state class, state family, owner node/scope, affinity key when applicable, failover class, replacement owner when applicable, `CorrelationId` when command-scoped | room/session reference unless state decision is domain-scoped |
| `runtime_task_lifecycle_decision` | `StartupRunId`, task class, parent component/supervision scope, owning layer, task reference when materialized, cancellation/join bound when applicable, `CorrelationId` when command-scoped | room/session reference unless task decision is domain-scoped |
| `internal_service_trust_decision` | `StartupRunId`, trust class, source service, target service, topology class, credential/peer proof reference class, trust policy reference, endpoint scope when endpoint resolution is involved, `CorrelationId` when command-scoped | room/session reference unless trust decision is domain-scoped |
| `cross_plane_binding_decision` | `CorrelationId`, binding class, source plane, target plane, source reference, target reference when materialized, authorization context class when applicable, lifecycle state class | startup run ID unless binding is startup-scoped |

A fake `RoomId` / `SessionId` MUST NOT be generated for event types whose reference rule marks room/session as `absent_not_applicable`.

### 3.11 Sink Rule / Regulated Enrichment Rule

The sink implementation is a driver. PostgreSQL, S3, HTTP, file, syslog, and in-memory are not core. A `configuration_decision` emitted before normal audit sink initialization uses a reserved bootstrap audit record path. The bootstrap audit record path is bounded, non-recursive, and only valid for startup/wiring failure before the selected audit sink is available. If the bootstrap audit record path cannot record the event, the startup/wiring attempt MUST stop and MUST NOT be used as closeout evidence (fail-closed).

Regulated enrichment does not change the core event. regulated treats opaque reference and non-sensitive tag as additional information. Regulated enrichment is not required for the core event to be valid.

### 3.12 Collapse Conditions (audit event)

The audit event requires regulated payload; PostgreSQL / S3 / HTTP sink becomes core implementation; audit reason becomes open-ended string only; audit event type becomes open-ended string only; a fake reason code is required on a success event; hash-chain semantics and persistence implementation are equated; audit sink/export format is treated as canonical serialization by default; audit/log/report path exposes raw secret, raw token, raw packet payload, or regulated payload; any of the decisions (command idempotency, authorization context, deployment topology, media negotiation, observability signal, secret rotation, supply-chain, SDK API contract, internal control-plane, ICE candidate/connectivity, secure media session, operator/admin authorization, out-of-scope feature, public endpoint, export/backup artifact, release artifact, time synchronization, edge/proxy trust, runtime reconfiguration, packet rewrite/media transform, service discovery, distributed state/failover, runtime task/worker lifecycle, internal service identity/trust, cross-plane identity/session binding) is recorded with an open-ended event type or missing reference rule.

---

## 4. Audit Hash-Chain Contract

### 4.1 Boundary (owner)

Audit event meaning is owned by section 3 of this chapter, and this section owns audit record ordering and tamper-evidence semantics.

| Responsibility | Owner |
|---|---|
| audit event field semantics | core |
| hash-chain scope / sequence / previous hash semantics | core |
| hash algorithm code set | core |
| canonical record serialization contract | core |
| storage, export, retry, backup | driver |
| verifier binary / CLI composition | entrypoints |

The storage driver does not own chain validity semantics. entrypoints do not own hash-chain semantics.

### 4.2 Chain Scope (closed set)

The hash-chain scope MUST be explicit. The allowed initial chain scopes are:

| Scope | Rule |
|---|---|
| startup chain | records startup / configuration / wiring decisions |
| signaling chain | records Signaling decisions and protocol violations |
| sfu chain | records SFU decisions |
| turn chain | records TURN decisions |
| driver chain | records driver conversion/resource/failure events |

A new chain scope requires a specification update.

### 4.3 Record Shape

Each hash-chain record MUST include: chain scope; sequence number; previous record hash or genesis marker; audit event type; audit event outcome; correlation/reference fields from section 3 of this chapter; reason category/code when required; canonical event payload digest; hash algorithm code; record hash. The initial v0.2 hash algorithm code set is:

| Code | Meaning |
|---|---|
| `sha256` | SHA-256 hash over canonical record input |

Adding an algorithm requires a specification update.

### 4.4 Canonicalization Rule

Hash input MUST be deterministic for the same audit event. Free-text details, log formatting, exporter timestamp formatting, DB row order, S3 key layout, and tracing span metadata MUST NOT change hash semantics. Canonical record input MUST declare the canonical format/version and normalized field set before it can be used as evidence. If the driver adds operational metadata, that metadata is outside the core hash input unless the specification explicitly admits it.

### 4.5 Failure / Evidence Rule / Persistence Relation

Hash-chain verification is an evidence validation process. If a chain gap, sequence mismatch, previous hash mismatch, unknown algorithm, canonical serialization failure, or canonicalization mismatch is observed, the affected record set MUST NOT be used as close / complete / ready evidence (fail-closed). Initial v0.2 does not use hash-chain verification failure as a runtime domain decision reason. Runtime domain decisions MUST continue to use the core reason catalog.

The persistence driver stores or exports hash-chain records. It MUST preserve sequence, previous hash, and record hash fields, and MUST NOT reorder records and present them as the same chain. Retry and export bounds follow the resource bound rule (chapter 11) and the persistence boundary rule.

### 4.6 Prohibitions / Collapse Conditions (hash-chain)

Prohibitions: storage backend defines hash-chain semantics; logs/traces replace hash-chain record; free-text details affect record hash; unknown hash algorithm is accepted; canonical format/version is implicit; chain gap is ignored while using the chain as evidence; audit event meaning is inferred from storage row shape.

Collapse: hash-chain validity is owned by persistence driver; record ordering is not deterministic; unknown algorithm is fail-open accepted; canonical serialization mismatch is ignored; failed verification is still used as closeout evidence; audit event canonical fields and hash-chain record fields diverge.
