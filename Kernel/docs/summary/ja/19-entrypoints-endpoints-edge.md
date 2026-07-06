# entrypoints の public endpoint と edge/proxy trust 境界

状態: public summary projection
日付: 2026-07-06 JST

## 目的

本章は arcRTC v0.2 Kernel の public endpoint surface と connection lifecycle の全状態と遷移、ならびに edge / reverse proxy / load balancer / gateway / trusted header / origin-host trust の信頼境界を、他文書・実コードを参照せずに完全自己完結で規定します。本章単独で再現実装が可能な粒度を与えます。

public endpoint は外部公開面として許可される endpoint class、公開/内部の分離、connection lifecycle の fail-closed 条件を固定します。Signaling / SFU / TURN の domain semantics は各 core contract に従い、public endpoint はそれらを直接所有しません。edge/proxy 由来の metadata と trusted header policy は本章後半が固定します。

## 第1節 Public Endpoint 境界

public endpoint は、外部 client または peer から到達可能な listener / route / socket / WebSocket / HTTP / UDP/TCP surface です。internal control-plane、admin/maintenance、service-to-service route、debug-only route は public endpoint ではありません。これらを外部公開するには、対象の core 契約で明示的に許可しなければなりません（必須）。

| 関心事 | Owner | Rule |
|---|---|---|
| endpoint class admission | architecture / entrypoint 規範 | public / internal / admin / test-only を明示する |
| concrete listener, socket, HTTP route, WebSocket upgrade | driver / entrypoints | network binding と protocol mechanics のみ |
| semantic admission, room/session/allocation/route transition | core | accepted/rejected decision と reason catalog |
| public client contract | Signaling / SDK 規範 | SDK は Signaling-only public contract に限定する |
| TLS / DTLS / transport security profile | security / driver 規範 | protected path requirement 単独では endpoint admission ではない |
| internal service route exposure | internal control-plane 規範 | public endpoint として扱わない |
| service discovery / endpoint resolution | entrypoints/drivers | resolved endpoint は public admission を証明しない |

## 第2節 Endpoint Classes（閉集合）

v0.2 initial architecture の endpoint class は次に限定します。新 endpoint class は v0.2 初期 scope 外です。

| Endpoint class | 許可される surface | Semantic owner |
|---|---|---|
| `signaling_public` | WebSocket / HTTP signaling ingress | Signaling contract 規範 |
| `turn_public_relay` | STUN/TURN UDP/TCP/TLS listener | TURN contract と TURN wire driver 規範 |
| `sfu_media_public` | WebRTC media transport ingress/egress | SFU contract と secure media 規範 |
| `health_public_readonly` | 明示的に admit された health/readiness observation | health/readiness 規範 |
| `admin_private` | publicly に expose されない operator/admin route | operator/admin 規範 |
| `internal_control_private` | publicly に expose されない service-to-service route | internal control-plane 規範 |
| `test_only_endpoint` | local test/fake surface のみ | test double 規範 |

## 第3節 Endpoint Admission Rule

すべての endpoint declaration は次を記録しなければなりません（必須）。

- endpoint class;
- endpoint が resolved の場合、service discovery source と resolved endpoint scope;
- concrete protocol と listener owner;
- proxy 由来 metadata が endpoint に影響しうる場合、edge/proxy trust policy;
- target core contract または明示的な非 domain operational contract;
- authentication / authorization requirement;
- local test scope 外で exposed の場合、transport security profile;
- rate, quota, resource, connection bound policy;
- correlation propagation rule;
- audit event type;
- public error mapping;
- endpoint metadata の redaction rule。

required field のいずれかが absent の場合、その endpoint を admitted public surface として扱ってはなりません。

## 第4節 Connection Lifecycle Rule（全状態）

connection lifecycle states は次の vocabulary に閉じられます（閉集合）。

| State | 意味 |
|---|---|
| `pre_open` | protocol admission 前に listener が transport material を accept/receive した |
| `protocol_checked` | external protocol/version/frame shape が driver により check された |
| `security_checked` | required transport/security/auth material が check または reject された |
| `core_admitted` | core が target semantic action を accept した |
| `active` | connection が admitted traffic の交換を許可される |
| `draining` | endpoint または entrypoint が new work を reject し、許可された in-flight work を完了させている |
| `idle_expired` | configured idle/consent/lifetime policy が connection を終了した |
| `closed_success` | required audit/reference material を伴う normal close が完了した |
| `closed_by_policy` | policy が cataloged reason で connection を close した |
| `failed` | driver/runtime failure が cataloged reason で connection を終了した |

driver は concrete socket または protocol state を observe してよいが、その observation を core decision なしに domain admission へ変換してはなりません。core は、concrete connection が physically open であっても semantic admission を reject してよいです。physical open state を、それ単独で readiness、user admission、media security、routing の evidence として用いてはなりません。

### 状態遷移（典型経路）
典型経路は `pre_open -> protocol_checked -> security_checked -> core_admitted -> active` です。`active` 以後は `draining`、`idle_expired`、`closed_success`、`closed_by_policy`、`failed` のいずれかへ遷移します。各 pre-`active` state からは check 失敗時に `failed` または `closed_by_policy` へ遷移しえます。physical open は遷移を進めるための観測に過ぎず、`core_admitted` を代替しません。

## 第5節 Public / Internal Separation Rule

`internal_control_private` と `admin_private` endpoint class は、default で public listener class に bind してはなりません（必須）。deployment topology がこれらを同一 process または network address の背後に置く場合でも、route-level admission はそれらを private として mark し、固有の authorization policy を要求しなければなりません。public endpoint admission は internal service trust から authorization を継承できません。

## 第6節 Public Endpoint Failure Mapping

| 失敗 | 必須 reason |
|---|---|
| endpoint class not admitted | `public_endpoint_not_allowed` |
| public endpoint version unsupported | `public_endpoint_version_unsupported` |
| required public endpoint authentication absent | `public_endpoint_auth_required` |
| core entry 前に protocol upgrade/handshake 失敗 | `public_endpoint_upgrade_failed` |
| connection lifecycle state transition invalid | `connection_lifecycle_violation` |
| connection idle/consent/public lifetime window expires | `connection_idle_timeout` |
| connection close policy が required audit/reference material を生成できない | `connection_close_policy_violation` |

pre-core decode failure は driver conversion 規範に従います。resource exhaustion は resource bounds / backpressure 規範に従います。transport security failure は transport security configuration 規範または secure media session lifecycle 規範に従います（当該 contract が failure を所有する場合）。edge/proxy trust failure は本章第8節以降に従います。service discovery / endpoint resolution failure は service discovery 規範に従います。

## 第7節 Public Endpoint Evidence / Audit Rule

public endpoint evidence は endpoint class、protocol、listener owner、target core contract、auth/security profile、bound policy、correlation rule、observed lifecycle state transition を記録しなければなりません。service discovery が endpoint に影響する場合、evidence は discovery source、endpoint scope、cache/staleness rule、fallback behavior も記録しなければなりません。listener startup 単独は source-shape または composition evidence に過ぎず、domain admission、media readiness、TURN relay correctness、SDK public contract、production readiness を証明しません。

public endpoint と connection lifecycle decisions は audit event type `public_endpoint_connection_decision` を用います。event は endpoint class、concrete protocol class、target contract reference、connection lifecycle state、command-scoped 時 `CorrelationId`、startup/listener-scoped 時 `StartupRunId` を carry しなければなりません。

## 第8節 Edge / Proxy Trust 境界

edge/proxy は外部 client と arcRTC entrypoint/driver listener の間に置かれる infrastructure component です。edge/proxy が生成または変換する header、source address、scheme、host、SNI、TLS termination status、request ID は driver/entrypoint observation であり、core identity、authorization、admission、audit source として自動採用してはなりません（禁止）。

| 関心事 | Owner | Rule |
|---|---|---|
| edge/proxy topology class | entrypoints/deployment 規範 | explicit topology と trust policy が必須 |
| concrete proxy/load balancer behavior | driver/entrypoints/deployment | infrastructure observation のみ |
| forwarded header parsing | driver/network | pre-core conversion と validation |
| trusted metadata admission | semantic な場合は core policy input | typed trust policy acceptance 後のみ |
| public endpoint semantic admission | core/public endpoint 規範 | proxy は所有しない |
| TLS termination proof | transport security / edge trust 境界 | listener security と edge security は別 |
| rate/quota/audit 用 source address | driver observation + core policy | default で raw header を用いない |

## 第9節 Edge Classes（閉集合）

v0.2 initial architecture の edge class は次に限定します。新 edge class は v0.2 初期 scope 外です。

| Edge class | 意味 | Rule |
|---|---|---|
| `direct_public_listener` | entrypoint/driver listener が直接公開される | proxy header を信用しない |
| `reverse_proxy_http_ws` | entrypoint 前段の HTTP/WebSocket reverse proxy | trusted header allowlist が必須 |
| `tcp_udp_load_balancer` | TURN/SFU/listener 前段の L4 load balancer | source address policy が必須 |
| `tls_terminating_edge` | entrypoint listener 前で TLS termination | downstream security relation を explicit にする |
| `service_mesh_ingress` | mesh sidecar/gateway が ingress metadata を供給 | mesh identity は default で core identity ではない |
| `test_edge_simulator` | local test/fake edge | test evidence only |

## 第10節 Trusted Metadata Classes（閉集合）

trusted edge metadata class は次の initial vocabulary に閉じられます。新 trusted metadata class は v0.2 初期 scope 外です。

| Metadata class | 例 | Default |
|---|---|---|
| `forwarded_for` | `Forwarded`, `X-Forwarded-For` | untrusted |
| `forwarded_proto` | `Forwarded`, `X-Forwarded-Proto` | untrusted |
| `forwarded_host` | `Forwarded`, `X-Forwarded-Host` | untrusted |
| `origin_header` | `Origin` | untrusted policy input |
| `host_header` | `Host` / `:authority` | untrusted policy input |
| `sni_host` | TLS SNI | transport/security policy が admit するまで untrusted |
| `client_address_observation` | socket peer address または proxy protocol address | driver observation |
| `edge_request_id` | proxy 生成の request ID | correlation rule に map されない限り diagnostic only |

## 第11節 Trust Admission Rule

edge/proxy trust policy は次を記録しなければなりません（必須）。

- edge class;
- deployment topology class;
- trusted upstream identity または network scope;
- accepted header/metadata classes;
- header precedence と conflict rule;
- forwarded chain を accept する場合の maximum hop count;
- TLS termination と downstream security relation;
- origin/host admission rule;
- client address use limit;
- rate/quota/admission relation;
- audit reference rule;
- redaction rule。

required field のいずれかが absent の場合、proxy 由来 metadata は untrusted として扱い、authorization、rate/quota、public/internal endpoint separation、audit source attribution に影響を与えてはなりません。

## 第12節 Header / Source Address / TLS Termination Rule

forwarded header は名称のみでは決して信用されません。driver は、現在の edge class がその metadata class を admit し、かつ immediate upstream が policy により trusted である場合にのみ parse してよいです。複数 source が conflict する場合、policy が deterministic precedence を定義しない限り、結果は fail-closed しなければなりません。raw client IP、forwarded chain、host、origin、SNI は core identity になってはなりません。これらは admitted trust policy を通じてのみ typed policy input になりえます。

edge での TLS termination は、entrypoint-to-edge または edge-to-backend security を証明しません。TLS が entrypoint listener の前で terminate する場合、policy は次を declare しなければなりません: edge termination class; backend transport security requirement; trusted edge identity; certificate/secret reference class; audit evidence relation; missing downstream protection の failure mapping。secure media session evidence は secure media 規範下に留まり、edge TLS では証明されません。

## 第13節 Edge Failure Mapping

| 失敗 | 必須 reason |
|---|---|
| edge/proxy class not admitted | `edge_proxy_not_admitted` |
| forwarded/trusted header が not admitted または not trustworthy | `forwarded_header_untrusted` |
| forwarded header chain が accepted hop count 超過または conflict | `forwarded_header_chain_invalid` |
| origin または host が policy により not allowed | `origin_host_not_allowed` |
| client address が target decision に trusted でない | `client_address_untrusted` |
| TLS termination/downstream security relation invalid | `tls_termination_boundary_invalid` |
| edge/proxy mapping により public/internal route が混同 | `public_internal_route_confusion` |

pre-core decode failure は driver conversion 規範に従います。endpoint admission は public endpoint 規範に従います。topology selection は deployment topology 規範に従います。

## 第14節 Edge Evidence / Audit Rule

edge/proxy trust evidence は edge class、topology class、trusted upstream scope、accepted metadata classes、header precedence、hop count、TLS termination relation、origin/host policy、client address use limit、command/procedure、working directory、rerun condition を記録しなければなりません。proxy configuration snippet や cloud console screenshot 単独は、evidence report が required fields を記録しない限り diagnostic です。

edge/proxy trust decisions は audit event type `edge_proxy_trust_decision` を用います。event は edge class、topology class、metadata class、trust policy reference、利用可能時 target endpoint class、`StartupRunId`、command-scoped 時 `CorrelationId` を carry しなければなりません。

## 第15節 禁止事項（Prohibitions）

- public listener existence を Signaling / SFU / TURN readiness として扱う。
- driver socket state を domain state として扱う。
- internal control-plane または admin route を default で public endpoint として expose する。
- WebSocket upgrade success を participant admission として扱う。
- UDP/TCP listener bind success を TURN allocation または SFU route success として扱う。
- public endpoint error が cataloged core reason を generic success の背後に隠す。
- v0.2 初期 scope 外で route naming convention により endpoint class を追加する。
- proxy route/header が edge trust admission なしに public/internal endpoint separation を変える。
- resolved endpoint または service discovery name が endpoint admission なしに public/internal endpoint separation を変える。
- forwarded header を標準名であることを理由に信用する。
- client IP / host / origin / SNI が core identity になる。
- public/internal route separation が proxy route naming のみに依存する。
- edge TLS termination を backend secure transport または secure media proof として扱う。
- rate/quota/admission policy が trust policy なしに raw forwarded header を用いる。
- proxy request ID が core `CorrelationId` を置き換える。
- service mesh identity が default で application authorization context になる。

## 第16節 Collapse Conditions（判断が崩れる条件）

- public/internal endpoint class が absent。
- physical connection state が semantic admission を所有する。
- internal control/admin route が configuration accident で public になる。
- public endpoint evidence が correlation、endpoint class、target contract を欠く。
- connection close/idle failure が closed reason vocabulary に map されない。
- edge/proxy metadata が trusted metadata policy なしに endpoint admission を変える。
- service discovery または resolved endpoint が endpoint class policy なしに public/internal admission を変える。
- edge class または trusted metadata class が absent。
- forwarded header が trusted upstream と hop rule なしに policy に影響しうる。
- TLS termination boundary が implicit。
- public/internal endpoint separation が core contract admission なしに proxy configuration で変えられる。
- audit source attribution が untrusted proxy metadata に依存する。

## 第17節 不変条件（Invariants）の要約

- endpoint class（public/internal/admin/test-only）は常に明示され、内部・admin route は default で public listener に bind されない。
- physical connection state（socket/protocol open）は core decision を代替せず、`core_admitted` のみが semantic admission を与える。
- proxy 由来 metadata は admitted trust policy を通じてのみ typed input になり、core identity・authorization・audit source にならない。
- required field 欠落、conflict、untrusted upstream は常に fail-closed。
- listener startup や TLS termination は readiness・media security・service trust を証明しない。
