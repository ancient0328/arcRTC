# entrypoints の topology と control-plane: deployment・internal control・service identity・discovery

状態: public summary projection
日付: 2026-07-06 JST

## 目的

本章は arcRTC v0.2 Kernel における deployment topology と control-plane の境界を、他文書・実コードを参照せずに完全自己完結で規定します。対象は次の4領域です: deployment topology / service boundary、internal control-plane / service-to-service contract、internal service identity / trust、service discovery / endpoint resolution。本章単独で再現実装が可能な粒度を与えます。

single process、split service、multi-node、service discovery、node affinity、node-local state、service-to-service identity が core semantics を暗黙に変えないよう、owner と evidence 境界を固定します。topology selection は Signaling / SFU / TURN / SDK / audit semantics を silently 変えてはなりません（禁止）。

---

## A 部 Deployment Topology / Service Boundary

### A-1 境界

| 関心事 | Owner | Rule |
|---|---|---|
| domain semantics | core | topology によって変化しない |
| service composition | entrypoints | selected topology と driver wiring |
| service discovery / endpoint resolution | driver/entrypoints | concrete discovery backend と runtime lookup; cache/fallback/staleness は専用規範（D 部） |
| internal service identity / trust | internal control authorization 前の entrypoints/drivers | endpoint, peer proof, credential trust を explicit にする |
| edge/proxy ingress metadata | entrypoints/drivers/deployment | edge trust policy なしに信用しない |
| internal control-plane contract | core-owned contract + entrypoints/driver wiring | split-plane message semantics を explicit にする |
| node-local SFU/TURN/runtime state | driver/entrypoints runtime | durable/global source-of-truth ではない; replication/failover admission は distributed state 規範 |
| topology policy input | entrypoints typed configuration + semantic な場合 core policy | unsupported 時 fail-closed |
| operational evidence | reports | topology class と node scope が必須 |

### A-2 Topology Classes（閉集合）

| Class | 意味 | Rule |
|---|---|---|
| `single_process_local` | 1 executable が selected planes を compose | local evidence only |
| `split_plane_same_host` | 同一 host 上の Signaling/SFU/TURN 別 process | explicit endpoint wiring が必須 |
| `split_plane_networked` | planes が network 越しに通信 | service endpoint と failure mapping が必須 |
| `multi_node_experimental` | 1 plane に複数 node | evidence なしに production-ready ではない |
| `external_managed_dependency` | external service が dependency を供給 | driver contract と health/readiness evidence が必須 |

新 topology class は v0.2 初期 scope 外です。

### A-3 Node State Rule

Room、SFU endpoint/route、TURN allocation/permission、packet cache、runtime queue、SDK local state は automatically cluster-global ではありません。command または packet path が node affinity または sticky routing を要する場合、topology policy は次を declare しなければなりません: affinity key; owning node scope; failover behavior; unavailable-node reason; recovery/replay relation; evidence class。distributed state 規範下の explicit distributed state policy が無い限り、node-local state は wrong node からアクセスされたとき fail-closed しなければなりません。

### A-4 Service Discovery Rule（topology 視点）

service discovery は driver/entrypoints infrastructure であり、D 部に従います。endpoint を resolve してよいが、次を所有しません: domain accept/reject decision; protocol version semantics; authorization policy; 別 plane の readiness success; failover 後の recovery success。service discovery failure を unverified endpoint への fallback の背後に隠してはなりません。fallback、TTL/cache、stale endpoint、endpoint scope conflict の扱いは D 部が所有します。topology が networked internal service calls を用いる場合、resolved endpoint evidence は internal service identity/trust evidence または explicit close-not-claimed scope と pair しなければなりません。

### A-5 Topology Failure Mapping

| 失敗 | 必須 reason |
|---|---|
| selected topology が v0.2 initial architecture で未対応 | `deployment_topology_unsupported` |
| service discovery が required endpoint を resolve できない | `service_discovery_unavailable` |
| discovery source または endpoint scope invalid | `service_discovery_source_not_admitted` または `service_endpoint_scope_conflict` |
| internal service identity/trust mapping invalid または absent | internal service identity reason |
| node affinity が required だが absent | `node_affinity_required` |
| required node-local state unavailable | `node_state_unavailable` |
| cross-node route または relay が policy により not allowed | `cross_node_route_not_allowed` |
| topology configuration missing | `runtime_config_missing` |
| topology configuration invalid | `runtime_config_invalid` |
| driver/network path unavailable | `network_send_failed` / `network_receive_failed` / `driver_shutdown` |

### A-6 Topology Evidence / Audit Rule

topology evidence は次を記録しなければなりません: topology class; ingress metadata が path に影響する時 edge/proxy class; process/entrypoint set; node scope; selected service endpoints または redacted references; 関連時 node affinity rule; discovery failure behavior; endpoint resolution が影響する時 discovery source/cache/fallback class; networked service identity が影響する時 internal service trust class; node-local state が move/reconstruct されうる時 distributed state class と failover admission status; health/readiness relation; close-not-claimed scope。single-node evidence を multi-node proof として用いてはなりません。

topology decisions は audit event type `deployment_topology_decision` を用います。event は topology class、entrypoint/service reference、startup run ID、command-scoped 時 `CorrelationId` を carry しなければなりません。

---

## B 部 Internal Control-Plane / Service-to-Service Contract

本部は split service、same-host plane split、networked plane split、multi-node experimental path で、Signaling / SFU / TURN / entrypoints 間の内部制御 message が domain semantics、external wire protocol、driver transport detail と混同されないよう固定します。internal control message は public Signaling contract ではありません。external client commands は SFU/TURN internal control surfaces を直接呼んではなりません。

### B-1 境界

| 関心事 | Owner | Rule |
|---|---|---|
| domain command/decision semantics | core | plane split によって変化しない |
| internal control contract shape | core-owned port/contract + entrypoints wiring | command, event, reason, correlation を閉じる |
| internal transport encoding | driver/network | HTTP, WebSocket, gRPC, Unix socket, in-memory channel は実装 detail |
| service endpoint wiring | entrypoints | selected topology と typed configuration |
| service discovery | driver/entrypoints | concrete lookup, semantic authority ではない; cache/fallback/stale は resolution 規範 |
| internal service identity / trust | internal authorization 前の entrypoints/drivers | endpoint と peer proof を admitted service identity context へ map |
| internal control authorization mapping | core policy input 前の entrypoints/drivers | operator/entrypoint/service credential を typed opaque context へ変換 |
| audit evidence | core event model と driver sink | internal control decision |

### B-2 Control Plane Classes（閉集合）

| Class | 意味 | Rule |
|---|---|---|
| `in_process_plane_call` | single process composition が core use case を直接呼ぶ | network evidence claim なし |
| `same_host_plane_call` | split process same-host internal control | endpoint と auth context が必須 |
| `networked_plane_call` | split process networked internal control | service discovery, version, auth, timeout が必須 |
| `node_affinity_plane_call` | command が state を所有する node に到達する必要 | affinity key と unavailable behavior が必須 |
| `admin_plane_call` | operator/admin が control path を invoke | operator/admin authorization 規範に従う |

新 control-plane class は v0.2 初期 scope 外です。

### B-3 Contract Rule

internal control contract は次を declare しなければなりません: control-plane class; source entrypoint/service; target entrypoint/service; command/event type; contract version; correlation ID; command replay 可能時 idempotency class; authorization context class; networked または same-host service identity が call に影響する時 internal service trust class; timeout/cancellation policy; failure reason mapping; audit event relation。driver-local status code、RPC exception、socket error、service mesh policy を authoritative core reason にしてはなりません。

### B-4 Control-Plane Failure Mapping

| 失敗 | 必須 reason |
|---|---|
| internal control message が contract に map できない | `internal_control_message_invalid` |
| internal control contract version unsupported | `internal_control_version_unsupported` |
| internal control authorization context missing | `internal_control_authorization_missing` |
| internal control authorization denied | `internal_control_authorization_denied` |
| internal service identity が missing/invalid/untrusted/expired/scope-conflicting | internal service identity reason |
| required service endpoint が resolve できない | `service_discovery_unavailable` |
| command が node affinity を要するが affinity が absent | `node_affinity_required` |
| target node-local state unavailable | `node_state_unavailable` |
| cross-node route not allowed | `cross_node_route_not_allowed` |
| internal control response timeout | `operation_deadline_exceeded` |
| resolved endpoint が stale または fallback が not admitted | `service_endpoint_stale` または `service_endpoint_fallback_not_allowed` |
| distributed owner/state conflict が検出 | `state_owner_conflict` または `split_brain_risk_detected` |
| network send failed | `network_send_failed` |
| network receive failed | `network_receive_failed` |
| driver shutdown | `driver_shutdown` |

### B-5 Control-Plane Audit / Evidence Rule

internal control decisions は audit event type `internal_control_plane_decision` を用います。event は `CorrelationId`、source service、target service、control-plane class、contract version、利用可能時 topology class を carry しなければなりません。

internal control-plane evidence は次を記録しなければなりません: topology class; control-plane class; source/target service; contract version; correlation ID; authorization context class; 該当時 internal service trust class と trust policy reference; 関連時 retry/idempotency relation; timeout/cancellation policy; networked control が discovery を用いる時 service discovery source と endpoint resolution state; command が node-local state を target する時 distributed state class と owner scope; expected outcome; actual outcome; non-success の cataloged reason; close-not-claimed scope。in-process evidence は same-host/networked/multi-node internal control behavior を証明しません。

---

## C 部 Internal Service Identity / Trust

本部は、解決済み endpoint、TLS peer verification、service credential、internal control authorization を同一視しないための trust 境界を固定します。resolved endpoint success は service identity success ではありません。TLS/mTLS session establishment は internal control authorization success ではありません。

### C-1 境界

| 関心事 | Owner | Rule |
|---|---|---|
| service endpoint resolution | entrypoints/drivers | endpoint observation のみ; service identity ではない |
| transport peer verification | driver/security backend | peer proof observation のみ; authorization ではない |
| service identity policy | core-owned typed policy + entrypoints configuration | accepted service identity class と target scope |
| credential/key loading | driver/security backend | concrete secret/certificate/token handling |
| service identity mapping | core policy input 前の entrypoints/drivers | raw credential/peer material を opaque service identity context へ |
| internal control authorization | internal control contract / authorization policy | service identity は input、replacement ではない |
| topology relation | deployment topology 規範 | identity scope は topology と service role に一致 |
| audit evidence | core event model と driver sink | trust decision は endpoint/control decision と別 |

### C-2 Trust Classes（閉集合）

| Class | 意味 | Rule |
|---|---|---|
| `service_identity_not_required` | network peer が無い in-process call | in-process composition evidence のみ |
| `static_configured_service_identity` | startup config が source/target service identity を bind | startup validation と scope が必須 |
| `mtls_peer_identity` | mTLS peer certificate または同等 peer proof を observe | trust anchor, peer scope, expiry が必須 |
| `signed_service_token_identity` | signed service credential を verify | issuer/audience/scope/lifetime が必須 |
| `mesh_asserted_service_identity` | service mesh/sidecar が service identity を assert | mesh trust policy と downstream relation が必須 |
| `test_service_identity` | test 用 deterministic fake identity | test evidence only |
| `unauthenticated_internal_service_requested` | internal service call が admitted identity proof を持たない | v0.2 初期 scope では rejected |

新 trust class は v0.2 初期 scope 外です。

### C-3 Identity Mapping Rule

internal service identity mapping は次を declare しなければなりません: trust class; source service; target service; topology class; credential または peer proof reference class; trust anchor または verifier reference; accepted scope; contract version relation; lifetime/expiry rule; credential-bearing 時 replay または freshness rule; service discovery endpoint scope との relation; internal control authorization context との relation; audit event relation。raw certificate、raw private key、raw service token、raw mesh assertion payload、raw secret は core state、audit body、logs、SDK surface、reports に入ってはなりません。opaque credential/reference material と redacted diagnostic summary のみが evidence に現れてよいです。

### C-4 Authorization Relation（必須 sequence）

service identity verification は、networked internal control path がそれを要求する場合の prerequisite input です。target internal control authorization decision を置き換えません。networked internal control の必須 sequence:

1. service discovery が declared scope 内で endpoint を resolve する。
2. trust class が要求する場合、transport security が peer proof を observe する。
3. service identity が raw peer/credential material を admitted opaque identity context へ map する。
4. internal control-plane contract が version、correlation、timeout、idempotency、authorization context を validate する。
5. command が core semantics に入る場合、target core use case が domain decision を行う。

earlier step での失敗は fail-closed し、later-step success として reclassify してはなりません。

### C-5 Identity Failure Mapping

| 失敗 | 必須 reason |
|---|---|
| trust class not admitted | `internal_service_identity_source_not_admitted` |
| required service identity absent | `internal_service_identity_missing` |
| service identity material が map できない | `internal_service_identity_invalid` |
| service identity が target path に trusted でない | `internal_service_identity_untrusted` |
| identity scope が target service/topology/contract に不一致 | `internal_service_identity_scope_conflict` |
| internal service trust の peer verification 失敗 | `internal_service_peer_verification_failed` |
| service credential または peer proof expired | `internal_service_credential_expired` |
| required service trust policy absent | `internal_service_trust_policy_missing` |
| trust mapping 前に endpoint resolution scope conflict | `service_endpoint_scope_conflict` |
| trust mapping 後に internal control authorization context absent | `internal_control_authorization_missing` |
| internal control authorization が call を deny | `internal_control_authorization_denied` |

### C-6 Identity Audit / Evidence Rule

internal service identity/trust decisions は audit event type `internal_service_trust_decision` を用います。event は `StartupRunId`、trust class、source service、target service、topology class、credential/peer proof reference class、trust policy reference、endpoint resolution が関わる時 endpoint scope、command-scoped 時 `CorrelationId` を carry しなければなりません。

internal service identity/trust evidence は次を記録しなければなりません: trust class; source/target service; topology class; networked 時 endpoint resolution state; 該当時 transport security/peer verification class; credential/peer proof reference class; trust policy reference; accepted scope と contract version relation; lifetime/expiry/freshness rule; internal control authorization relation; command/procedure; working directory; expected outcome; actual outcome; non-success の cataloged reason; close-not-claimed scope。endpoint resolution logs、TLS handshake logs、mesh route status は、上記 fields を伴う evidence が採用しない限り diagnostic only です。

---

## D 部 Service Discovery / Endpoint Resolution

本部は service discovery、endpoint resolution、service registry、DNS/mesh lookup、fallback endpoint の境界です。concrete endpoint lookup が domain semantics、public endpoint admission、readiness、authorization、failover success を所有しないよう固定します。解決結果は network target observation であり、これらを自動成立させません。解決結果は internal service identity を自動成立させません。

### D-1 境界

| 関心事 | Owner | Rule |
|---|---|---|
| topology class | deployment topology 規範 | discovery source は topology に一致 |
| service discovery source | entrypoints/config + driver | concrete lookup backend |
| endpoint resolution execution | driver/entrypoints | DNS, registry, mesh, static config, local process |
| endpoint semantic admission | core/public/internal contract | resolver は所有しない |
| internal control contract | core-owned contract + entrypoints wiring | resolved endpoint は contract を bypass しない |
| internal service identity / trust | service identity 規範 | resolved endpoint は trust policy を通じた input のみ |
| readiness relation | health/readiness 規範 | resolution success は readiness success ではない |
| evidence/reporting | reports | source, cache, TTL, fallback, stale behavior が必須 |

### D-2 Discovery Source Classes（閉集合）

| Class | 意味 | Rule |
|---|---|---|
| `static_config_endpoint` | typed startup configuration が endpoint を供給 | startup validation が必須 |
| `local_process_registry` | entrypoints composition が in-process/same-host service を resolve | local evidence only |
| `dns_resolution` | DNS name が service endpoint を resolve | TTL/cache behavior が必須 |
| `service_registry_lookup` | registry または control-plane store が endpoint を resolve | registry contract が必須 |
| `service_mesh_resolution` | mesh/sidecar が route を resolve | mesh policy は core authorization ではない |
| `test_resolver` | test 用 deterministic fake resolver | test evidence only |

新 discovery source class は v0.2 初期 scope 外です。

### D-3 Resolution State Rule（閉集合）

endpoint resolution は次の closed states を用いなければなりません。

| State | 意味 |
|---|---|
| `resolution_not_required` | selected topology が endpoint lookup を要さない |
| `resolution_pending` | lookup が accepted endpoint を未生成 |
| `resolution_accepted` | endpoint reference が declared scope に accept された |
| `resolution_rejected` | lookup result が policy により reject された |
| `resolution_stale` | cached endpoint が TTL/generation/window を超過 |
| `resolution_failed` | lookup 失敗または driver unavailable |

resolved endpoint references は evidence references であり、public reachability または readiness の proof ではありません。

### D-4 Admission Rule

endpoint resolution は、resolution policy が次を記録する場合にのみ採用してよいです: discovery source class; topology class; target service/plane; expected endpoint scope; public/internal endpoint relation; 該当時 endpoint version または contract reference; TTL/cache/staleness rule; fallback behavior; authorization/context relation; endpoint が internal service target の時 service identity/trust relation; readiness relation; audit event relation。required field のいずれかが absent の場合、その endpoint を closeout evidence または semantic success に用いてはなりません。

### D-5 Fallback Rule

fallback endpoint use は、policy が次を明示的に記録しない限り禁止です: fallback source class; accepted target scope; stale endpoint rejection rule; public/internal separation rule; primary failure の audit reason; evidence limitation。unverified endpoint への fallback は fail-closed しなければなりません。

### D-6 Discovery Failure Mapping

| 失敗 | 必須 reason |
|---|---|
| discovery source class not admitted | `service_discovery_source_not_admitted` |
| selected discovery source unavailable | `service_discovery_unavailable` |
| target service の endpoint が resolve できない | `service_endpoint_resolution_failed` |
| cached endpoint が stale または generation-invalid | `service_endpoint_stale` |
| fallback endpoint not admitted | `service_endpoint_fallback_not_allowed` |
| resolved endpoint が expected scope に不一致 | `service_endpoint_scope_conflict` |
| resolved endpoint の contract/version が not accepted | `service_endpoint_contract_mismatch` |
| resolver が public/internal endpoint を誤 map | `public_internal_route_confusion` |
| resolver output が admitted trust policy なしに service identity として用いられる | `internal_service_identity_untrusted` |

network send/receive failures は network I/O boundary に従います。internal control-plane contract failures は B 部に従います。internal service identity/trust failures は C 部に従います。public endpoint admission は public endpoint 規範に従います。

### D-7 Discovery Audit / Evidence Rule

service discovery/endpoint resolution evidence は discovery source class、topology class、target service/plane、endpoint scope、endpoint reference または redacted endpoint、TTL/cache rule、staleness state、fallback behavior、contract/version reference、command/procedure、working directory、rerun condition を記録しなければなりません。endpoint resolution が internal service control を target する場合、evidence は service identity/trust relation または close-not-claimed scope も記録しなければなりません。DNS/registry/mesh output 単独は、required fields を伴う evidence report が採用しない限り diagnostic です。

service discovery/endpoint resolution decisions は audit event type `service_discovery_resolution_decision` を用います。event は `StartupRunId`、command-scoped 時 `CorrelationId`、discovery source class、topology class、target service、endpoint scope、resolution state、該当時 fallback class、rejected/failed outcome の cataloged reason を carry しなければなりません。

---

## E 部 横断 禁止事項（Prohibitions）

- topology selection が core semantics を silently 変える。
- service discovery が domain decision を所有する。
- service discovery または TLS listener startup が internal service trust として扱われる。
- node-local SFU/TURN state が default で cluster-global として扱われる。
- failover success が recovery/replay evidence なしに主張される。
- failover success が service discovery fallback 単独から主張される。
- replication/consensus behavior が distributed state 規範なしに multi-node topology から含意される。
- sticky routing requirement が隠される。
- local dev topology が production topology として扱われる。
- proxy/load-balancer metadata が edge trust policy なしに trusted topology evidence として扱われる。
- internal RPC status が core reason になる。
- endpoint resolution success が internal control success として扱われる。
- external client command が public Signaling contract を bypass して SFU/TURN internal surface に到達する。
- internal control authorization が network reachability から推論される。
- internal service identity が endpoint resolution / TLS listener startup / mesh route name から推論される。
- topology change が Signaling/SFU/TURN state semantics を変える。
- driver-to-driver internal call が domain authority になる。
- service discovery endpoint success が trusted service identity として扱われる。
- TLS/mTLS listener startup が peer trust success として扱われる。
- peer verification success が internal control authorization success として扱われる。
- mesh policy name または route name が core service identity になる。
- public endpoint credential が admitted trust class なしに internal service identity として reuse される。
- raw service token / certificate / private key / mesh assertion payload が core/audit/log/report に現れる。
- in-process service identity evidence が networked service trust evidence として reuse される。
- internal service trust failure が free-text のみで記録される。
- DNS/registry/mesh resolution success が readiness として扱われる。
- fallback endpoint が policy と audit reason なしに用いられる。
- stale cached endpoint が fresh evidence として用いられる。
- resolved internal endpoint が naming のみで public endpoint として expose される。
- mesh policy または resolver status が default で application authorization になる。
- in-process resolution evidence が networked discovery evidence として reuse される。

## F 部 横断 Collapse Conditions（判断が崩れる条件）

- node scope が topology evidence から absent。
- multi-node path が affinity または distributed policy なしに node-local state を用いる。
- fallback endpoint が typed topology policy なしに accept される。
- ある plane の readiness が別 plane の readiness として用いられる。
- split service wiring が direct driver-to-driver semantic dependency を作る。
- internal control-plane path が version/authorization/correlation/audit rule を bypass する。
- edge/proxy topology が explicit policy なしに source identity / endpoint class / trust boundary を変える。
- service discovery fallback が owner node または endpoint scope を dedicated evidence なしに変える。
- internal service identity/trust class が networked internal service calls を用いるとき隠される。
- topology が replication/consensus/failover を主張するとき distributed state policy が absent。
- internal control contract に version が無い。
- internal control path が correlation ID を欠く。
- internal control authorization owner が implicit。
- networked control-plane evidence が in-process execution から推論される。
- internal transport encoding が domain decision semantics を変える。
- service-to-service failure が free-text または external status のみで記録される。
- service discovery または distributed state class が control path に影響するとき隠される。
- trust class が implicit。
- endpoint resolution が identity 必須の箇所で service identity mapping を bypass する。
- transport peer verification が internal control authorization を bypass する。
- raw credential/peer proof material が core-owned になる。
- trust scope / topology class / contract relation が absent。
- service identity evidence が networked internal control がそれに依存するとき隠される。
- discovery source class が absent。
- caching が存在する箇所で endpoint TTL/cache/staleness rule が absent。
- fallback behavior が implicit。
- resolved endpoint が internal control contract または public endpoint admission を bypass する。
- resolver success が failover / readiness / authorization proof として用いられる。
- service identity/trust relation が endpoint resolution が internal control に feed するとき隠される。

## G 部 不変条件（Invariants）の要約

- domain semantics（Signaling/SFU/TURN/SDK/audit）は topology によって不変。topology は composition のみを変える。
- node-local state は default で cluster-global でなく、affinity または distributed state policy なしに wrong node アクセスは fail-closed。
- networked internal control の必須 sequence（discovery -> peer proof -> identity mapping -> control contract -> core decision）の各 step は独立であり、earlier-step 失敗は later-step success に昇格しない。
- resolved endpoint は service identity でも readiness でも authorization でも failover success でもない（observation のみ）。
- raw credential / peer proof / secret は core/audit/log/report に現れない（opaque reference のみ）。
- fallback、stale、scope conflict、unsupported topology、untrusted identity は常に fail-closed し cataloged reason を残す。
