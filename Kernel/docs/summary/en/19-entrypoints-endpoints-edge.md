# Entrypoints Public Endpoint and Edge/Proxy Trust Boundary

Status: SSOT consolidated edition
Date: 2026-06-28 JST

## Purpose

This chapter specifies, in a fully self-contained form (understandable without opening any other file, source dev-doc, or implementation code), all states and transitions of the public endpoint surface and connection lifecycle of the arcRTC v0.2 Kernel, and the trust boundary of edge / reverse proxy / load balancer / gateway / trusted header / origin-host trust. The granularity is sufficient for reimplementation from this chapter alone.

The public endpoint boundary fixes the endpoint classes permitted as an externally exposed surface, the public/internal separation, and the fail-closed conditions of the connection lifecycle. The domain semantics of Signaling / SFU / TURN follow their respective core contracts, and the public endpoint does not own them directly. Edge/proxy-derived metadata and trusted header policy are fixed in the second half of this chapter.

## Section 1. Public Endpoint Boundary

A public endpoint is a listener / route / socket / WebSocket / HTTP / UDP/TCP surface reachable from an external client or peer. internal control-plane, admin/maintenance, service-to-service route, and debug-only route are not public endpoints. Exposing them externally MUST be explicitly permitted by the core contract.

| Concern | Owner | Rule |
|---|---|---|
| endpoint class admission | architecture / entrypoint authority | declare public / internal / admin / test-only |
| concrete listener, socket, HTTP route, WebSocket upgrade | driver / entrypoints | network binding and protocol mechanics only |
| semantic admission, room/session/allocation/route transition | core | accepted/rejected decision and reason catalog |
| public client contract | Signaling / SDK authority | SDK is limited to a Signaling-only public contract |
| TLS / DTLS / transport security profile | security / driver authority | protected path requirement is not endpoint admission by itself |
| internal service route exposure | internal control-plane authority | not treated as public endpoint |
| service discovery / endpoint resolution | entrypoints/drivers | resolved endpoint does not prove public admission |

## Section 2. Endpoint Classes (closed set)

The endpoint classes of the v0.2 initial architecture are limited to the following. A new endpoint class is out of the v0.2 initial scope.

| Endpoint class | Allowed surface | Semantic owner |
|---|---|---|
| `signaling_public` | WebSocket / HTTP signaling ingress | Signaling contract authority |
| `turn_public_relay` | STUN/TURN UDP/TCP/TLS listener | TURN contract and TURN wire driver authority |
| `sfu_media_public` | WebRTC media transport ingress/egress | SFU contract and secure media authority |
| `health_public_readonly` | explicitly admitted health/readiness observation | health/readiness authority |
| `admin_private` | operator/admin route not publicly exposed | operator/admin authority |
| `internal_control_private` | service-to-service route not publicly exposed | internal control-plane authority |
| `test_only_endpoint` | local test/fake surface only | test double authority |

## Section 3. Endpoint Admission Rule

Every endpoint declaration MUST record:

- endpoint class;
- service discovery source and resolved endpoint scope when endpoint is resolved;
- concrete protocol and listener owner;
- edge/proxy trust policy when proxy-derived metadata can affect the endpoint;
- target core contract or explicitly non-domain operational contract;
- authentication / authorization requirement;
- transport security profile when exposed outside local test scope;
- rate, quota, resource, and connection bound policy;
- correlation propagation rule;
- audit event type;
- public error mapping;
- redaction rule for endpoint metadata.

If any required field is absent, the endpoint MUST NOT be treated as an admitted public surface.

## Section 4. Connection Lifecycle Rule (all states)

connection lifecycle states are closed to the following vocabulary (closed set).

| State | Meaning |
|---|---|
| `pre_open` | listener accepted or received transport material before protocol admission |
| `protocol_checked` | external protocol/version/frame shape was checked by driver |
| `security_checked` | required transport/security/auth material was checked or rejected |
| `core_admitted` | core accepted the target semantic action |
| `active` | connection is allowed to exchange admitted traffic |
| `draining` | endpoint or entrypoint is rejecting new work and finishing allowed in-flight work |
| `idle_expired` | configured idle/consent/lifetime policy ended the connection |
| `closed_success` | normal close completed with required audit/reference material |
| `closed_by_policy` | policy closed the connection with cataloged reason |
| `failed` | driver/runtime failure ended the connection with cataloged reason |

Driver MAY observe concrete socket or protocol state, but MUST NOT convert that observation into domain admission without a core decision. Core MAY reject semantic admission even when the concrete connection is physically open. Physical open state MUST NOT be used as readiness, user admission, media security, or routing evidence by itself.

### State transitions (typical path)
The typical path is `pre_open -> protocol_checked -> security_checked -> core_admitted -> active`. After `active`, the connection transitions to one of `draining`, `idle_expired`, `closed_success`, `closed_by_policy`, or `failed`. From each pre-`active` state, a check failure MAY transition to `failed` or `closed_by_policy`. Physical open is only an observation that advances the transition; it does not substitute for `core_admitted`.

## Section 5. Public / Internal Separation Rule

`internal_control_private` and `admin_private` endpoint classes MUST NOT be bound to public listener classes by default. If deployment topology places them behind the same process or network address, route-level admission MUST still mark them private and require their own authorization policy. Public endpoint admission cannot inherit authorization from internal service trust.

## Section 6. Public Endpoint Failure Mapping

| Failure | Required reason |
|---|---|
| endpoint class is not admitted | `public_endpoint_not_allowed` |
| public endpoint version is unsupported | `public_endpoint_version_unsupported` |
| required public endpoint authentication is absent | `public_endpoint_auth_required` |
| protocol upgrade or handshake fails before core entry | `public_endpoint_upgrade_failed` |
| connection lifecycle state transition is invalid | `connection_lifecycle_violation` |
| connection idle, consent, or public lifetime window expires | `connection_idle_timeout` |
| connection close policy cannot produce required audit/reference material | `connection_close_policy_violation` |

Pre-core decode failure follows the driver conversion authority. Resource exhaustion follows the resource bounds / backpressure authority. Transport security failure follows the transport security configuration authority or the secure media session lifecycle authority when those contracts own the failure. Edge/proxy trust failure follows Sections 8 onward. Service discovery / endpoint resolution failure follows the service discovery authority.

## Section 7. Public Endpoint Evidence / Audit Rule

Public endpoint evidence MUST record endpoint class, protocol, listener owner, target core contract, auth/security profile, bound policy, correlation rule, and observed lifecycle state transition. When service discovery affects the endpoint, evidence MUST also record discovery source, endpoint scope, cache/staleness rule, and fallback behavior. Listener startup alone is source-shape or composition evidence only; it does not prove domain admission, media readiness, TURN relay correctness, SDK public contract, or production readiness.

Public endpoint and connection lifecycle decisions use audit event type `public_endpoint_connection_decision`. The event MUST carry endpoint class, concrete protocol class, target contract reference, connection lifecycle state, `CorrelationId` when command-scoped, and `StartupRunId` when startup/listener-scoped.

## Section 8. Edge / Proxy Trust Boundary

An edge/proxy is an infrastructure component placed between the external client and the arcRTC entrypoint/driver listener. The header, source address, scheme, host, SNI, TLS termination status, and request ID that an edge/proxy generates or transforms are driver/entrypoint observations, and MUST NOT be automatically adopted as core identity, authorization, admission, or audit source.

| Concern | Owner | Rule |
|---|---|---|
| edge/proxy topology class | entrypoints/deployment authority | explicit topology and trust policy required |
| concrete proxy/load balancer behavior | driver/entrypoints/deployment | infrastructure observation only |
| forwarded header parsing | driver/network | pre-core conversion and validation |
| trusted metadata admission | core policy input where semantic | only after typed trust policy acceptance |
| public endpoint semantic admission | core/public endpoint authority | not owned by proxy |
| TLS termination proof | transport security / edge trust boundary | listener security and edge security are separate |
| source address for rate/quota/audit | driver observation plus core policy | never raw header by default |

## Section 9. Edge Classes (closed set)

The edge classes of the v0.2 initial architecture are limited to the following. A new edge class is out of the v0.2 initial scope.

| Edge class | Meaning | Rule |
|---|---|---|
| `direct_public_listener` | entrypoint/driver listener is directly exposed | no proxy header is trusted |
| `reverse_proxy_http_ws` | HTTP/WebSocket reverse proxy in front of entrypoint | trusted header allowlist required |
| `tcp_udp_load_balancer` | L4 load balancer before TURN/SFU/listener | source address policy required |
| `tls_terminating_edge` | TLS terminates before entrypoint listener | downstream security relation must be explicit |
| `service_mesh_ingress` | mesh sidecar/gateway supplies ingress metadata | mesh identity is not core identity by default |
| `test_edge_simulator` | local test/fake edge | test evidence only |

## Section 10. Trusted Metadata Classes (closed set)

Trusted edge metadata classes are closed to the following initial vocabulary. A new trusted metadata class is out of the v0.2 initial scope.

| Metadata class | Example | Default |
|---|---|---|
| `forwarded_for` | `Forwarded`, `X-Forwarded-For` | untrusted |
| `forwarded_proto` | `Forwarded`, `X-Forwarded-Proto` | untrusted |
| `forwarded_host` | `Forwarded`, `X-Forwarded-Host` | untrusted |
| `origin_header` | `Origin` | untrusted policy input |
| `host_header` | `Host` / `:authority` | untrusted policy input |
| `sni_host` | TLS SNI | untrusted until transport/security policy admits it |
| `client_address_observation` | socket peer address or proxy protocol address | driver observation |
| `edge_request_id` | proxy-generated request ID | diagnostic only unless mapped to correlation rule |

## Section 11. Trust Admission Rule

An edge/proxy trust policy MUST record:

- edge class;
- deployment topology class;
- trusted upstream identity or network scope;
- accepted header/metadata classes;
- header precedence and conflict rule;
- maximum hop count when a forwarded chain is accepted;
- TLS termination and downstream security relation;
- origin/host admission rule;
- client address use limit;
- rate/quota/admission relation;
- audit reference rule;
- redaction rule.

If any required field is absent, proxy-derived metadata MUST be treated as untrusted and MUST NOT influence authorization, rate/quota, public/internal endpoint separation, or audit source attribution.

## Section 12. Header / Source Address / TLS Termination Rule

Forwarded headers are never trusted by name alone. Driver MAY parse them only when the current edge class admits that metadata class and the immediate upstream is trusted by policy. When multiple sources conflict, the result MUST fail closed unless the policy defines deterministic precedence. Raw client IP, forwarded chain, host, origin, and SNI MUST NOT become core identity. They MAY become typed policy input only through an admitted trust policy.

TLS termination at edge does not prove entrypoint-to-edge or edge-to-backend security. When TLS terminates before the entrypoint listener, the policy MUST declare: edge termination class; backend transport security requirement; trusted edge identity; certificate/secret reference class; audit evidence relation; failure mapping for missing downstream protection. Secure media session evidence remains governed by the secure media authority and is not proven by edge TLS.

## Section 13. Edge Failure Mapping

| Failure | Required reason |
|---|---|
| edge/proxy class is not admitted | `edge_proxy_not_admitted` |
| forwarded/trusted header is not admitted or not trustworthy | `forwarded_header_untrusted` |
| forwarded header chain exceeds accepted hop count or conflicts | `forwarded_header_chain_invalid` |
| origin or host is not allowed by policy | `origin_host_not_allowed` |
| client address cannot be trusted for target decision | `client_address_untrusted` |
| TLS termination/downstream security relation is invalid | `tls_termination_boundary_invalid` |
| public/internal route is confused by edge/proxy mapping | `public_internal_route_confusion` |

Pre-core decode failure follows the driver conversion authority. Endpoint admission follows the public endpoint authority. Topology selection follows the deployment topology authority.

## Section 14. Edge Evidence / Audit Rule

Edge/proxy trust evidence MUST record edge class, topology class, trusted upstream scope, accepted metadata classes, header precedence, hop count, TLS termination relation, origin/host policy, client address use limit, command/procedure, working directory, and rerun condition. Proxy configuration snippets or cloud console screenshots alone are diagnostic unless an evidence report records the required fields.

Edge/proxy trust decisions use audit event type `edge_proxy_trust_decision`. The event MUST carry edge class, topology class, metadata class, trust policy reference, target endpoint class when available, `StartupRunId`, and `CorrelationId` when command-scoped.

## Section 15. Prohibitions

- public listener existence is treated as Signaling / SFU / TURN readiness.
- driver socket state is treated as domain state.
- internal control-plane or admin route is exposed as public endpoint by default.
- WebSocket upgrade success is treated as participant admission.
- UDP/TCP listener bind success is treated as TURN allocation or SFU route success.
- public endpoint error hides a cataloged core reason behind generic success.
- endpoint class is added by route naming convention out of the v0.2 initial scope.
- proxy route/header changes public/internal endpoint separation without edge trust admission.
- resolved endpoint or service discovery name changes public/internal endpoint separation without endpoint admission.
- forwarded header is trusted because it has a standard name.
- client IP, host, origin, or SNI becomes core identity.
- public/internal route separation depends only on proxy route naming.
- edge TLS termination is treated as backend secure transport or secure media proof.
- rate/quota/admission policy uses raw forwarded header without trust policy.
- proxy request ID replaces core `CorrelationId`.
- service mesh identity becomes application authorization context by default.

## Section 16. Collapse Conditions

- public/internal endpoint class is absent.
- physical connection state owns semantic admission.
- internal control/admin route becomes public by configuration accident.
- public endpoint evidence lacks correlation, endpoint class, or target contract.
- connection close/idle failure is not mapped to closed reason vocabulary.
- edge/proxy metadata changes endpoint admission without trusted metadata policy.
- service discovery or resolved endpoint changes public/internal admission without endpoint class policy.
- edge class or trusted metadata class is absent.
- forwarded header can influence policy without trusted upstream and hop rule.
- TLS termination boundary is implicit.
- public/internal endpoint separation can be changed by proxy configuration without core contract admission.
- audit source attribution relies on untrusted proxy metadata.

## Section 17. Invariants (summary)

- The endpoint class (public/internal/admin/test-only) is always explicit, and internal/admin routes are not bound to a public listener by default.
- Physical connection state (socket/protocol open) does not substitute for a core decision; only `core_admitted` grants semantic admission.
- Proxy-derived metadata becomes typed input only through an admitted trust policy, never core identity, authorization, or audit source.
- Missing required fields, conflict, and untrusted upstream always fail closed.
- Listener startup and TLS termination do not prove readiness, media security, or service trust.
