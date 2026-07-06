# Entrypoints Topology and Control-Plane: deployment, internal control, service identity, discovery

Status: public summary projection
Date: 2026-07-06 JST

## Purpose

This chapter specifies, in a fully self-contained form (understandable without opening any other file, source dev-doc, or the actual code), the deployment topology and control-plane boundaries of the arcRTC v0.2 Kernel. It covers four areas: deployment topology / service boundary; internal control-plane / service-to-service contract; internal service identity / trust; and service discovery / endpoint resolution. The granularity is sufficient for re-implementation from this chapter alone.

It fixes the owner and evidence boundaries so that single process, split service, multi-node, service discovery, node affinity, node-local state, and service-to-service identity do not implicitly change core semantics. Topology selection MUST NOT silently change Signaling / SFU / TURN / SDK / audit semantics.

---

## Part A. Deployment Topology / Service Boundary

### A-1 Boundary

| Concern | Owner | Rule |
|---|---|---|
| domain semantics | core | does not change by topology |
| service composition | entrypoints | selected topology and driver wiring |
| service discovery / endpoint resolution | driver/entrypoints | concrete discovery backend and runtime lookup; cache/fallback/staleness owned by the dedicated authority (Part D) |
| internal service identity / trust | entrypoints/drivers before internal control authorization | endpoint, peer proof, and credential trust are explicit |
| edge/proxy ingress metadata | entrypoints/drivers/deployment | not trusted without an edge trust policy |
| internal control-plane contract | core-owned contract plus entrypoints/driver wiring | split-plane message semantics are explicit |
| node-local SFU/TURN/runtime state | driver/entrypoints runtime | not a durable/global source-of-truth; replication/failover admission owned by the distributed state authority |
| topology policy input | entrypoints typed configuration plus core policy where semantic | fail-closed when unsupported |
| operational evidence | reports | topology class and node scope required |

### A-2 Topology Classes (closed set)

| Class | Meaning | Rule |
|---|---|---|
| `single_process_local` | one executable composes selected planes | local evidence only |
| `split_plane_same_host` | separate Signaling/SFU/TURN processes on same host | explicit endpoint wiring required |
| `split_plane_networked` | planes communicate over network | service endpoint and failure mapping required |
| `multi_node_experimental` | multiple nodes for one plane | not production-ready without evidence |
| `external_managed_dependency` | external service supplies dependency | driver contract and health/readiness evidence required |

A new topology class is out of the v0.2 initial scope.

### A-3 Node State Rule

Room, SFU endpoint/route, TURN allocation/permission, packet cache, runtime queue, and SDK local state are not automatically cluster-global. If a command or packet path requires node affinity or sticky routing, the topology policy MUST declare: affinity key; owning node scope; failover behavior; unavailable-node reason; recovery/replay relation; evidence class. Absent an explicit distributed state policy under the distributed state authority, node-local state MUST fail closed when accessed from the wrong node.

### A-4 Service Discovery Rule (topology view)

Service discovery is driver/entrypoints infrastructure and follows Part D. It MAY resolve endpoints, but it does not own: domain accept/reject decision; protocol version semantics; authorization policy; readiness success for another plane; recovery success after failover. Service discovery failure MUST NOT be hidden behind fallback to an unverified endpoint. Fallback, TTL/cache, stale endpoint, and endpoint scope conflict handling are owned by Part D. When topology uses networked internal service calls, resolved endpoint evidence MUST be paired with internal service identity/trust evidence or an explicit close-not-claimed scope.

### A-5 Topology Failure Mapping

| Failure | Required reason |
|---|---|
| selected topology not supported by the v0.2 initial architecture | `deployment_topology_unsupported` |
| service discovery cannot resolve required endpoint | `service_discovery_unavailable` |
| discovery source or endpoint scope is invalid | `service_discovery_source_not_admitted` or `service_endpoint_scope_conflict` |
| internal service identity/trust mapping is invalid or absent | internal service identity reason |
| node affinity is required but absent | `node_affinity_required` |
| required node-local state unavailable | `node_state_unavailable` |
| cross-node route or relay not allowed by policy | `cross_node_route_not_allowed` |
| topology configuration missing | `runtime_config_missing` |
| topology configuration invalid | `runtime_config_invalid` |
| driver/network path unavailable | `network_send_failed`, `network_receive_failed`, or `driver_shutdown` |

### A-6 Topology Evidence / Audit Rule

Topology evidence MUST record: topology class; edge/proxy class when ingress metadata affects the path; process/entrypoint set; node scope; selected service endpoints or redacted references; node affinity rule when relevant; discovery failure behavior; discovery source/cache/fallback class when endpoint resolution affects the topology evidence; internal service trust class when networked service identity affects the topology evidence; distributed state class and failover admission status when node-local state can move or be reconstructed; health/readiness relation; close-not-claimed scope. Single-node evidence MUST NOT be used as multi-node proof.

Topology decisions use audit event type `deployment_topology_decision`. The event MUST carry topology class, entrypoint/service reference, startup run ID, and `CorrelationId` when the decision is command-scoped.

---

## Part B. Internal Control-Plane / Service-to-Service Contract

This part fixes that, in split service, same-host plane split, networked plane split, and multi-node experimental paths, the internal control message between Signaling / SFU / TURN / entrypoints is not confused with domain semantics, external wire protocol, or driver transport detail. An internal control message is not the public Signaling contract. External client commands MUST NOT directly call SFU/TURN internal control surfaces.

### B-1 Boundary

| Concern | Owner | Rule |
|---|---|---|
| domain command/decision semantics | core | does not change by plane split |
| internal control contract shape | core-owned port/contract plus entrypoints wiring | closes command, event, reason, correlation |
| internal transport encoding | driver/network | HTTP, WebSocket, gRPC, Unix socket, in-memory channel are implementation detail |
| service endpoint wiring | entrypoints | selected topology and typed configuration |
| service discovery | driver/entrypoints | concrete lookup, not semantic authority; cache/fallback/stale owned by the resolution authority |
| internal service identity / trust | entrypoints/drivers before internal authorization | endpoint and peer proof mapped to an admitted service identity context |
| internal control authorization mapping | entrypoints/drivers before core policy input | convert operator/entrypoint/service credential into a typed opaque context |
| audit evidence | core event model and driver sink | internal control decision |

### B-2 Control Plane Classes (closed set)

| Class | Meaning | Rule |
|---|---|---|
| `in_process_plane_call` | single process composition calls a core use case directly | no network evidence claim |
| `same_host_plane_call` | split process same-host internal control | endpoint and auth context required |
| `networked_plane_call` | split process networked internal control | service discovery, version, auth, timeout required |
| `node_affinity_plane_call` | command must reach the node owning state | affinity key and unavailable behavior required |
| `admin_plane_call` | operator/admin invokes a control path | follows the operator/admin authorization authority |

A new control-plane class is out of the v0.2 initial scope.

### B-3 Contract Rule

An internal control contract MUST declare: control-plane class; source entrypoint/service; target entrypoint/service; command/event type; contract version; correlation ID; idempotency class when command replay is possible; authorization context class; internal service trust class when networked or same-host service identity affects the call; timeout/cancellation policy; failure reason mapping; audit event relation. A driver-local status code, RPC exception, socket error, or service mesh policy MUST NOT become the authoritative core reason.

### B-4 Control-Plane Failure Mapping

| Failure | Required reason |
|---|---|
| internal control message cannot map to contract | `internal_control_message_invalid` |
| internal control contract version unsupported | `internal_control_version_unsupported` |
| internal control authorization context missing | `internal_control_authorization_missing` |
| internal control authorization denied | `internal_control_authorization_denied` |
| internal service identity is missing, invalid, untrusted, expired, or scope-conflicting | internal service identity reason |
| required service endpoint cannot be resolved | `service_discovery_unavailable` |
| command requires node affinity but affinity is absent | `node_affinity_required` |
| target node-local state unavailable | `node_state_unavailable` |
| cross-node route not allowed | `cross_node_route_not_allowed` |
| internal control response timeout | `operation_deadline_exceeded` |
| resolved endpoint is stale or fallback is not admitted | `service_endpoint_stale` or `service_endpoint_fallback_not_allowed` |
| distributed owner/state conflict is detected | `state_owner_conflict` or `split_brain_risk_detected` |
| network send failed | `network_send_failed` |
| network receive failed | `network_receive_failed` |
| driver shutdown | `driver_shutdown` |

### B-5 Control-Plane Audit / Evidence Rule

Internal control decisions use audit event type `internal_control_plane_decision`. The event MUST carry `CorrelationId`, source service, target service, control-plane class, contract version, and topology class when available.

Internal control-plane evidence MUST record: topology class; control-plane class; source/target service; contract version; correlation ID; authorization context class; internal service trust class and trust policy reference when applicable; retry/idempotency relation when relevant; timeout/cancellation policy; service discovery source and endpoint resolution state when networked control uses discovery; distributed state class and owner scope when command targets node-local state; expected outcome; actual outcome; cataloged reason for non-success; close-not-claimed scope. In-process evidence does not prove same-host, networked, or multi-node internal control behavior.

---

## Part C. Internal Service Identity / Trust

This part fixes the trust boundary so that a resolved endpoint, TLS peer verification, service credential, and internal control authorization are not equated. Resolved endpoint success is not service identity success. TLS/mTLS session establishment is not internal control authorization success.

### C-1 Boundary

| Concern | Owner | Rule |
|---|---|---|
| service endpoint resolution | entrypoints/drivers | endpoint observation only; not service identity |
| transport peer verification | driver/security backend | peer proof observation only; not authorization |
| service identity policy | core-owned typed policy plus entrypoints configuration | accepted service identity class and target scope |
| credential/key loading | driver/security backend | concrete secret/certificate/token handling |
| service identity mapping | entrypoints/drivers before core policy input | raw credential/peer material to opaque service identity context |
| internal control authorization | internal control contract / authorization policy | service identity is input, not replacement |
| topology relation | deployment topology authority | identity scope must match topology and service role |
| audit evidence | core event model and driver sink | trust decision is distinct from endpoint and control decision |

### C-2 Trust Classes (closed set)

| Class | Meaning | Rule |
|---|---|---|
| `service_identity_not_required` | in-process call where no network peer exists | only for in-process composition evidence |
| `static_configured_service_identity` | startup config binds source/target service identity | startup validation and scope required |
| `mtls_peer_identity` | mTLS peer certificate or equivalent peer proof is observed | trust anchor, peer scope, and expiry required |
| `signed_service_token_identity` | signed service credential is verified | issuer/audience/scope/lifetime required |
| `mesh_asserted_service_identity` | service mesh or sidecar asserts service identity | mesh trust policy and downstream relation required |
| `test_service_identity` | deterministic fake identity for tests | test evidence only |
| `unauthenticated_internal_service_requested` | internal service call has no admitted identity proof | rejected in the v0.2 initial scope |

A new trust class is out of the v0.2 initial scope.

### C-3 Identity Mapping Rule

Internal service identity mapping MUST declare: trust class; source service; target service; topology class; credential or peer proof reference class; trust anchor or verifier reference; accepted scope; contract version relation; lifetime/expiry rule; replay or freshness rule when credential-bearing; relation to service discovery endpoint scope; relation to internal control authorization context; audit event relation. Raw certificate, raw private key, raw service token, raw mesh assertion payload, or raw secret MUST NOT enter core state, audit body, logs, SDK surface, or reports. Only opaque credential/reference material and a redacted diagnostic summary MAY appear in evidence.

### C-4 Authorization Relation (required sequence)

Service identity verification is a prerequisite input when a networked internal control path requires it. It does not replace the target internal control authorization decision. The required sequence for networked internal control is:

1. service discovery resolves endpoint within the declared scope;
2. transport security observes peer proof when the trust class requires it;
3. service identity maps raw peer/credential material into an admitted opaque identity context;
4. internal control-plane contract validates version, correlation, timeout, idempotency, and authorization context;
5. target core use case makes the domain decision if the command crosses into core semantics.

Failure at any earlier step MUST fail closed and MUST NOT be reclassified as a later-step success.

### C-5 Identity Failure Mapping

| Failure | Required reason |
|---|---|
| trust class is not admitted | `internal_service_identity_source_not_admitted` |
| required service identity is absent | `internal_service_identity_missing` |
| service identity material cannot be mapped | `internal_service_identity_invalid` |
| service identity cannot be trusted for target path | `internal_service_identity_untrusted` |
| identity scope does not match target service/topology/contract | `internal_service_identity_scope_conflict` |
| peer verification failed for internal service trust | `internal_service_peer_verification_failed` |
| service credential or peer proof expired | `internal_service_credential_expired` |
| required service trust policy is absent | `internal_service_trust_policy_missing` |
| endpoint resolution scope conflicts before trust mapping | `service_endpoint_scope_conflict` |
| internal control authorization context is absent after trust mapping | `internal_control_authorization_missing` |
| internal control authorization denies the call | `internal_control_authorization_denied` |

### C-6 Identity Audit / Evidence Rule

Internal service identity/trust decisions use audit event type `internal_service_trust_decision`. The event MUST carry `StartupRunId`, trust class, source service, target service, topology class, credential/peer proof reference class, trust policy reference, endpoint scope when endpoint resolution is involved, and `CorrelationId` when command-scoped.

Internal service identity/trust evidence MUST record: trust class; source/target service; topology class; endpoint resolution state when networked; transport security/peer verification class when applicable; credential/peer proof reference class; trust policy reference; accepted scope and contract version relation; lifetime/expiry/freshness rule; internal control authorization relation; command/procedure; working directory; expected outcome; actual outcome; cataloged reason for non-success; close-not-claimed scope. Endpoint resolution logs, TLS handshake logs, or mesh route status are diagnostic only unless adopted by evidence with the fields above.

---

## Part D. Service Discovery / Endpoint Resolution

This part is the boundary for service discovery, endpoint resolution, service registry, DNS/mesh lookup, and fallback endpoint. It fixes that concrete endpoint lookup does not own domain semantics, public endpoint admission, readiness, authorization, or failover success. A resolution result is a network target observation and does not automatically establish those. A resolution result does not automatically establish internal service identity.

### D-1 Boundary

| Concern | Owner | Rule |
|---|---|---|
| topology class | deployment topology authority | discovery source must match topology |
| service discovery source | entrypoints/config plus driver | concrete lookup backend |
| endpoint resolution execution | driver/entrypoints | DNS, registry, mesh, static config, local process |
| endpoint semantic admission | core/public/internal contract | not owned by the resolver |
| internal control contract | core-owned contract plus entrypoints wiring | resolved endpoint must not bypass the contract |
| internal service identity / trust | service identity authority | resolved endpoint is input only when mapped through a trust policy |
| readiness relation | health/readiness authority | resolution success is not readiness success |
| evidence/reporting | reports | source, cache, TTL, fallback, and stale behavior required |

### D-2 Discovery Source Classes (closed set)

| Class | Meaning | Rule |
|---|---|---|
| `static_config_endpoint` | endpoint is supplied by typed startup configuration | startup validation required |
| `local_process_registry` | entrypoints composition resolves in-process or same-host service | local evidence only |
| `dns_resolution` | DNS name resolves service endpoint | TTL/cache behavior required |
| `service_registry_lookup` | registry or control-plane store resolves endpoint | registry contract required |
| `service_mesh_resolution` | mesh/sidecar resolves route | mesh policy is not core authorization |
| `test_resolver` | deterministic fake resolver for tests | test evidence only |

A new discovery source class is out of the v0.2 initial scope.

### D-3 Resolution State Rule (closed set)

Endpoint resolution MUST use these closed states.

| State | Meaning |
|---|---|
| `resolution_not_required` | selected topology does not require endpoint lookup |
| `resolution_pending` | lookup has not produced an accepted endpoint |
| `resolution_accepted` | endpoint reference accepted for the declared scope |
| `resolution_rejected` | lookup result rejected by policy |
| `resolution_stale` | cached endpoint exceeded TTL/generation/window |
| `resolution_failed` | lookup failed or driver unavailable |

Resolved endpoint references are evidence references, not proof of public reachability or readiness.

### D-4 Admission Rule

Endpoint resolution MAY be adopted only when the resolution policy records: discovery source class; topology class; target service/plane; expected endpoint scope; public/internal endpoint relation; endpoint version or contract reference when applicable; TTL/cache/staleness rule; fallback behavior; authorization/context relation; service identity/trust relation when the endpoint is an internal service target; readiness relation; audit event relation. If any required field is absent, the endpoint MUST NOT be used for closeout evidence or semantic success.

### D-5 Fallback Rule

Fallback endpoint use is prohibited unless the policy explicitly records: fallback source class; accepted target scope; stale endpoint rejection rule; public/internal separation rule; audit reason for primary failure; evidence limitation. Fallback to an unverified endpoint MUST fail closed.

### D-6 Discovery Failure Mapping

| Failure | Required reason |
|---|---|
| discovery source class is not admitted | `service_discovery_source_not_admitted` |
| selected discovery source is unavailable | `service_discovery_unavailable` |
| endpoint cannot be resolved for target service | `service_endpoint_resolution_failed` |
| cached endpoint is stale or generation-invalid | `service_endpoint_stale` |
| fallback endpoint is not admitted | `service_endpoint_fallback_not_allowed` |
| resolved endpoint does not match expected scope | `service_endpoint_scope_conflict` |
| resolved endpoint contract/version is not accepted | `service_endpoint_contract_mismatch` |
| resolver maps public/internal endpoint incorrectly | `public_internal_route_confusion` |
| resolver output is used as service identity without admitted trust policy | `internal_service_identity_untrusted` |

Network send/receive failures remain governed by the network I/O boundary. Internal control-plane contract failures remain governed by Part B. Internal service identity/trust failures remain governed by Part C. Public endpoint admission remains governed by the public endpoint authority.

### D-7 Discovery Audit / Evidence Rule

Service discovery/endpoint resolution evidence MUST record discovery source class, topology class, target service/plane, endpoint scope, endpoint reference or redacted endpoint, TTL/cache rule, staleness state, fallback behavior, contract/version reference, command/procedure, working directory, and rerun condition. When endpoint resolution targets internal service control, evidence MUST also record service identity/trust relation or close-not-claimed scope. DNS/registry/mesh output alone is diagnostic unless adopted by an evidence report with the required fields.

Service discovery/endpoint resolution decisions use audit event type `service_discovery_resolution_decision`. The event MUST carry `StartupRunId`, `CorrelationId` when command-scoped, discovery source class, topology class, target service, endpoint scope, resolution state, fallback class when applicable, and cataloged reason for rejected/failed outcomes.

---

## Part E. Cross-cutting Prohibitions

- topology selection changes core semantics silently.
- service discovery owns domain decision.
- service discovery or TLS listener startup is treated as internal service trust.
- node-local SFU/TURN state is treated as cluster-global by default.
- failover success is claimed without recovery/replay evidence.
- failover success is claimed from service discovery fallback alone.
- replication/consensus behavior is implied by multi-node topology without a distributed state authority.
- sticky routing requirement is hidden.
- local dev topology is treated as production topology.
- proxy/load-balancer metadata is treated as trusted topology evidence without an edge trust policy.
- internal RPC status becomes core reason.
- endpoint resolution success is treated as internal control success.
- external client command bypasses public Signaling contract to reach SFU/TURN internal surface.
- internal control authorization is inferred from network reachability.
- internal service identity is inferred from endpoint resolution, TLS listener startup, or mesh route name.
- topology change alters Signaling/SFU/TURN state semantics.
- driver-to-driver internal call becomes domain authority.
- service discovery endpoint success is treated as trusted service identity.
- TLS/mTLS listener startup is treated as peer trust success.
- peer verification success is treated as internal control authorization success.
- mesh policy name or route name becomes core service identity.
- public endpoint credential is reused as internal service identity without an admitted trust class.
- raw service token, certificate, private key, or mesh assertion payload appears in core/audit/log/report.
- in-process service identity evidence is reused as networked service trust evidence.
- internal service trust failure is recorded as free-text only.
- DNS/registry/mesh resolution success is treated as readiness.
- fallback endpoint is used without policy and audit reason.
- stale cached endpoint is used as fresh evidence.
- resolved internal endpoint is exposed as public endpoint by naming alone.
- mesh policy or resolver status becomes application authorization by default.
- in-process resolution evidence is reused as networked discovery evidence.

## Part F. Cross-cutting Collapse Conditions

- node scope is absent from topology evidence.
- multi-node path uses node-local state without affinity or distributed policy.
- fallback endpoint is accepted without typed topology policy.
- readiness of one plane is used as readiness of another plane.
- split service wiring creates a direct driver-to-driver semantic dependency.
- internal control-plane path bypasses version, authorization, correlation, or audit rule.
- edge/proxy topology changes source identity, endpoint class, or trust boundary without explicit policy.
- service discovery fallback changes owner node or endpoint scope without dedicated evidence.
- internal service identity/trust class is hidden when topology uses networked internal service calls.
- distributed state policy is absent when topology claims replication, consensus, or failover.
- internal control contract has no version.
- internal control path lacks correlation ID.
- internal control authorization owner is implicit.
- networked control-plane evidence is inferred from in-process execution.
- internal transport encoding changes domain decision semantics.
- service-to-service failure is recorded only as free-text or external status.
- service discovery or distributed state class is hidden when it affects the control path.
- trust class is implicit.
- endpoint resolution bypasses service identity mapping where identity is required.
- transport peer verification bypasses internal control authorization.
- raw credential/peer proof material becomes core-owned.
- trust scope, topology class, or contract relation is absent.
- service identity evidence is hidden when networked internal control depends on it.
- discovery source class is absent.
- endpoint TTL/cache/staleness rule is absent where caching exists.
- fallback behavior is implicit.
- resolved endpoint bypasses internal control contract or public endpoint admission.
- resolver success is used as failover, readiness, or authorization proof.
- service identity/trust relation is hidden when endpoint resolution feeds internal control.

## Part G. Invariants (summary)

- Domain semantics (Signaling/SFU/TURN/SDK/audit) are invariant under topology; topology changes only composition.
- Node-local state is not cluster-global by default, and wrong-node access fails closed without an affinity or distributed state policy.
- Each step of the required sequence for networked internal control (discovery -> peer proof -> identity mapping -> control contract -> core decision) is independent, and an earlier-step failure is not promoted to a later-step success.
- A resolved endpoint is neither service identity, readiness, authorization, nor failover success (observation only).
- raw credential / peer proof / secret does not appear in core/audit/log/report (only an opaque reference does).
- Fallback, stale, scope conflict, unsupported topology, and untrusted identity always fail closed and leave a cataloged reason.
