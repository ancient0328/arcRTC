# core-security-and-audit

状態: public summary projection
日付: 2026-07-06 JST

## 目的

本章は arcRTC v0.2 Kernel の core が所有する security token verification boundary、authorization context / communication policy、audit event model、audit hash-chain contract の現行完全仕様を、本章のみで再現実装可能な粒度で内在化することを目的とします。

依存方向の表記: `A <- B` は「B が A に依存」を意味します。core は外部 I/O 非依存です。arcRTC は認証基盤ではなく、外部発行 token の検証境界を持つ通信基盤です。arcRTC は token を発行せず、user account を所有しません。本章は identity-neutral primitives、verification request / result / decision semantics、authorization context mapping、audit event の全 field / type / 閉集合、hash-chain 構造・検証規則・改竄検知を core 所有とし、JWT / JWK / cryptographic library / key fetch / cache / concrete sink を driver 所有として分離します。token verification success と communication authorization success と operator/admin authorization success は別であり、混同してはなりません。

---

## 1. Security Token Verification

### 1.1 境界（owner）

core は token verification request / result / decision semantics を所有します。driver は JWT / JWK / cryptographic library / key fetch / cache implementation を所有します。entrypoints は verifier implementation を wiring します。

### 1.2 Core Responsibilities（core 所有、必須）

core は次を所有します。verification request type、verification result type、closed rejection reason、required claims vocabulary、token expiry decision、abstract rule としての audience / issuer policy、participant transport identity mapping rule、entrypoint role semantics ではなく typed communication policy としてのみの authorization context input。

### 1.3 Driver Responsibilities（driver 所有）

driver は次を所有します。JWT parser、signature verification library、JWK fetch / cache、entrypoints wiring 経由で受け取る typed key material / key source config、HTTP client、cryptographic backend、external error conversion。

### 1.4 Fail-Closed Reasons（閉集合）

token verification は次の closed reason code を持ちます。reason code は core reason catalog の code でなければならず、category-only reason value は禁止です（禁止）。

| Failure | reason code |
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

### 1.5 Audit / Wrapper 規則

rejected token verification は concrete な `token_*` reason code を持つ `token_verification_decision` を emit しなければなりません（必須）。Signaling は、同じ correlation chain が concrete な `token_*` reason を `token_verification_decision` に record する場合に限り、`token_verification_failed` を public wrapper rejection reason として露出してよい。`token_verification_failed` は concrete token verification reason を audit で代替・消去してはなりません（禁止）。verified token が communication policy input として使われる場合、authorization context mapping は別途 record されなければなりません（必須）。

### 1.6 禁止 / Collapse 条件（token verification）

禁止: arcRTC が token を発行する。arcRTC が user account を所有する。core が `jsonwebtoken` 等 concrete API に依存する。core が HTTP key fetch を行う。Signaling participant ID を authenticated user ID と同一視する。token claims から entrypoint-specific role を protocol decision に持ち込む。token verification success を communication authorization success として扱う。

Collapse: token issuance を arcRTC responsibility とする。core が JWT concrete library に依存する。driver の external error が open-ended string のまま core decision になる。driver が env / file / process args から key configuration を直接読む。domain role authorization を generic communication protocol に混入する。authorization policy が authorization context mapping なしに token verification から推論される。

---

## 2. Authorization Context / Communication Policy

### 2.1 境界（owner）

本節は token verification、core identity、admission、publication/subscription/TURN permission decision を混同せず、verified credential から generic communication policy input へ写す境界を固定します。本節は user account、role management、tenant billing、regulated authorization 実装を主張しません。

| 関心事 | 所有者 | 規則 |
|---|---|---|
| token verification result | core/security semantics + driver verifier | 本章 1 節 |
| external auth claims / domain roles | external application または regulated | generic core identity ではない |
| authorization context mapping | core policy input 前の entrypoints/drivers | typed opaque context へ変換 |
| edge/proxy metadata | authorization mapping 前の entrypoints/drivers | edge trust policy が admit した後のみ使用可 |
| communication authorization policy | core typed policy | join/publication/subscription/TURN/admission decision に使用 |
| regulated authorization | regulated | generic core の必須 schema ではない |
| operator/admin authorization | entrypoints/drivers + admin policy | communication participant authorization ではない |
| internal service identity / trust | entrypoints/drivers + service trust policy | internal control authorization input のみ |
| audit evidence | core event model and driver sink | authorization context decision |

Token verification success は join、publication、subscription、TURN relay、quota admission を自動的に authorize しません（禁止）。

### 2.2 Authorization Context Classes（閉集合）

| Class | 意味 | 規則 |
|---|---|---|
| `verified_credential_context` | token/key verification result が存在 | それ自体で domain authorization ではない |
| `participant_join_context` | room membership 用 join policy input | Signaling join decision に map |
| `publication_policy_context` | SFU publication authorization input | SFU publication decision に map |
| `subscription_policy_context` | SFU subscription authorization input | SFU subscription decision に map |
| `turn_relay_policy_context` | TURN allocation/permission/relay authorization input | TURN decision に map |
| `admission_policy_context` | rate/quota/admission grouping input | rate limit 規則（本章 13 章）に従う |
| `regulated_authorization_context` | regulated 側 policy のみ | generic core requirement になってはならない |

新しい authorization context class は仕様の更新を要します。

### 2.3 Mapping 規則

Authorization mapping は次を宣言しなければなりません（必須）。source credential または external context class、allowed target decision surfaces、opaque core reference または typed policy input、lifetime と expiry、claim/field redaction rule、failure reason mapping、audit event relation。

Raw token claims、entrypoint user role、tenant role、billing account、facility role、medical role、regulated subject は generic core identity になってはなりません（禁止）。Client IP、forwarded header、host、origin、SNI は、edge trust policy と authorization mapping の両方が admit しない限り authorization context になってはなりません（禁止）。

### 2.4 Decision 規則

Core は authorization context を typed communication policy input としてのみ使用してよい。各 target surface は自身の decision と reason を保持します。Signaling join は Signaling join decision、SFU publication/subscription は SFU decision、TURN permission/relay は TURN decision、quota/admission は resource/admission decision を使用します。Authorization context decision は policy input が unavailable または invalid な理由を説明してよいが、target domain decision event を代替しません（禁止）。

### 2.5 Failure Mapping（authorization context）

| Failure | 必須 reason |
|---|---|
| required authorization context missing | `authorization_context_missing` |
| authorization context cannot be mapped to typed policy input | `authorization_context_invalid` |
| authorization context expired | `authorization_context_expired` |
| policy denies requested communication action | `authorization_policy_denied` |
| requested authorization scope not allowed in generic core | `authorization_scope_not_allowed` |
| edge/proxy metadata is not trustworthy for authorization mapping | `forwarded_header_untrusted` または `client_address_untrusted` |
| token verification failed | concrete token reason または許可された `token_verification_failed` wrapper |
| required authorization configuration missing | `runtime_config_missing` |
| required authorization configuration invalid | `runtime_config_invalid` |

### 2.6 Audit / Evidence 規則

Authorization context mapping decision は audit event type `authorization_context_decision` を使用します。Target domain decision は自身の audit event type を使用します。同じ correlation chain は両方の event を含んでよいが、一方が他方を代替しません（禁止）。

authorization context または policy claim の evidence は次を含まなければなりません（必須）。source credential/context class、mapped authorization context class、target decision surface、lifetime/expiry observation または deterministic time fixture、raw claims と sensitive auth payload の redaction statement、audit event type `authorization_context_decision`、claim が join/publication/subscription/TURN/admission behavior を含む場合の target domain decision event、missing/invalid/expired/denied/scope-not-allowed outcome の closed reason。Token verification evidence 単独は communication authorization を証明しません。Communication authorization evidence 単独は operator/admin authorization を証明しません。Internal service trust evidence 単独は target internal control authorization を証明しません。

### 2.7 禁止 / Collapse 条件（authorization context）

禁止: token verification success を join/publication/subscription/TURN authorization success として扱う。entrypoint-specific role が generic core identity になる。regulated authorization schema が required generic core schema になる。SDK platform wrapper が server authorization decision を行う。authorization denial を free-text のみで record。raw claims または sensitive auth payload を audit/log/report に入れる。operator/admin action を communication participant policy で authorize。internal service call を endpoint reachability または peer verification 単独から authorize。edge/proxy metadata を edge trust policy なしに authorization context にする。

Collapse: authorization context owner が implicit。generic core が entrypoint/regulated role semantics を直接読む。authorization mapping 後に target domain decision を skip。required input absence 時に authorization context が fail open し得る。audit が token verification / authorization mapping / target decision を区別できない。audit が communication authorization と operator/admin authorization を区別できない。audit が verified authorization context と proxy-derived metadata を区別できない。audit が internal service trust と target internal control authorization を区別できない。

---

## 3. Audit Event Model

### 3.1 境界（owner）

core は audit event model、hash-chain contract、audit sink port を所有します。drivers は file / HTTP / syslog / PostgreSQL / S3 等の concrete sink を所有します。regulated は optional enrichment を所有します。audit は communication infrastructure の観測可能性を担いますが、regulated payload を generic core に混入しません（禁止）。

### 3.2 Core Audit Event Fields（必須 field 集合）

core audit event は次を持ちます。

- event ID
- correlation ID
- timestamp
- component
- event type code
- subject transport reference（reference rule が要求する場合）
- room/session reference（reference rule が要求する場合）
- startup run ID（reference rule が要求する場合）
- configuration scope reference（reference rule が要求する場合）
- authorization context class/reference（reference rule が要求する場合）
- media negotiation class/reference（reference rule が要求する場合）
- service topology class/reference（reference rule が要求する場合）
- observability signal class/reference（reference rule が要求する場合）
- dependency/toolchain reference（reference rule が要求する場合）
- SDK platform/projection reference（reference rule が要求する場合）
- internal control-plane class/reference（reference rule が要求する場合）
- ICE candidate/connectivity class/reference（reference rule が要求する場合）
- secure media session class/reference（reference rule が要求する場合）
- operator/admin authorization class/reference（reference rule が要求する場合）
- out-of-scope feature class/reference（reference rule が要求する場合）
- public endpoint and connection lifecycle class/reference（reference rule が要求する場合）
- export/backup artifact class/reference（reference rule が要求する場合）
- release artifact/provenance/distribution class/reference（reference rule が要求する場合）
- time synchronization/clock skew class/reference（reference rule が要求する場合）
- edge/proxy trust class/reference（reference rule が要求する場合）
- runtime reconfiguration class/reference（reference rule が要求する場合）
- packet rewrite/media transform class/reference（reference rule が要求する場合）
- service discovery/endpoint resolution class/reference（reference rule が要求する場合）
- distributed state/failover class/reference（reference rule が要求する場合）
- runtime task/worker class/reference（reference rule が要求する場合）
- internal service identity/trust class/reference（reference rule が要求する場合）
- cross-plane identity/session binding class/reference（reference rule が要求する場合）
- runtime reconfiguration class/generation/reference（reference rule が要求する場合）
- decision outcome
- reason presence
- reason category（reason presence が `cataloged` のとき）
- reason code（reason presence が `cataloged` のとき）
- resource policy owner（audit event が resource-bound related のとき）
- physical resource owner（audit event が resource-bound related のとき）
- previous hash reference（hash-chain が適用される場合）
- event hash
- non-sensitive tags

### 3.3 Construction Input 規則

audit event と hash-chain record は field 数が多く、裸の多引数 constructor にしてはなりません（禁止）。実装は `AuditEventInput`、`HashChainRecordInput` のような名前付き入力型で未検査材料を束ね、reason presence、required reference、resource-bound owner、hash-chain relation の検査を constructor 内で fail-closed に行います（必須）。入力型は audit sink や external response の代替 authority ではなく、core audit model へ渡す未検査材料の境界です。

### 3.4 Prohibited Fields

core audit event に次を含めてはなりません（禁止）。medical record payload、patient identity、entrypoint user profile、protocol identity としての domain role、raw credential secret、raw media payload、regulated-specific required schema、startup または configuration event 用の fake room/session reference。Privacy、redaction、retention constraints は privacy/redaction/retention 規則に従います。Canonical event serialization と deterministic digest material は canonical serialization 規則に従います。

### 3.5 Resource-Bound Owner Field 規則

`resource-bound related` とは、bounded resource、event type、outcome、reason code の tuple が本章 11 章 Required Bound Closed Action Mapping の row に一致する audit event instance を意味します。これには `resource_bound_decision`、`driver_resource_bound_decision`、および TURN allocation / refresh cap / permission / channel bind bound row の mapped bound audit event であるときの TURN-specific decision event instance が含まれます。全 resource-bound related audit event instance は resource policy owner と physical resource owner field を carry しなければなりません（必須）。Multi-purpose event type は全 instance で resource-bound related になるわけではありません。例えば `turn_allocation_decision` の accepted/released event や credential rejection event は、resource/outcome/reason が Required Bound Closed Action Mapping row に一致しない限り resource-bound related ではありません。TURN allocation / refresh cap / permission / channel bind bound event では resource policy owner は `core`、physical resource owner は `driver` です。TURN relay queue bound は `turn_relay_decision` で carry されず、resource policy owner `core`、physical resource owner `driver` の `resource_bound_decision` を使用します。inbound frame size bound event では resource policy owner は `driver`、physical resource owner は `driver` です。Signaling / SFU domain decision event が bound reason も carry する場合は supplementary domain decision であり、mapped resource-bound audit event を代替せず、Required Bound Closed Action Mapping の mapped bound audit event type でない限り owner field requirement を満たしません。

### 3.6 Event Type Codes（閉集合）

audit event type は閉集合の code として扱います。free-text category を event type の正として使ってはなりません（禁止）。

| event type code | 意味 | required reason connection |
|---|---|---|
| `signaling_join_decision` | Signaling join accepted/rejected | accepted は absent、rejected は required |
| `signaling_participant_lifecycle_decision` | Signaling participant leave/lifecycle termination accepted/rejected | accepted は absent、rejected は required |
| `signaling_room_lifecycle_decision` | Signaling room drain/close/closed-room observation accepted/rejected | accepted/idempotent observation は absent、rejected は required |
| `signaling_protocol_violation` | Signaling command が protocol 違反 | reason category/code required |
| `signaling_relay_event` | Signaling relay event emitted | forwarded は absent、suppressed/rejected relay は required |
| `turn_allocation_decision` | TURN allocation accepted/rejected/released/expired | accepted/released は absent、rejected/expired は required |
| `turn_refresh_decision` | TURN refresh accepted/rejected/expired by refresh cap | accepted は absent、rejected/expired は required |
| `turn_permission_decision` | TURN permission accepted/rejected/revoked/expired | accepted は absent、rejected/revoked/expired は required |
| `turn_channel_bind_decision` | TURN channel bind accepted/rejected/expired | accepted は absent、rejected/expired は required |
| `turn_relay_decision` | TURN relay allowed/denied | allowed は absent、denied は required |
| `sfu_session_lifecycle_decision` | SFU session drain/close accepted/rejected | accepted は absent、rejected は required |
| `sfu_admission_decision` | SFU endpoint admission accepted/rejected | accepted は absent、rejected は required |
| `sfu_endpoint_lifecycle_decision` | SFU endpoint drain/removal accepted/rejected | accepted は absent、rejected は required |
| `sfu_publication_decision` | SFU publication accepted/rejected/suppressed/closed | accepted/closed は absent、rejected/suppressed は required |
| `sfu_subscription_decision` | SFU subscription accepted/rejected/suppressed/closed | accepted/closed は absent、rejected/suppressed は required |
| `sfu_forwarding_decision` | SFU forwarding selected/rejected/suppressed/dropped/closed/failed | forwarded/selected/successful closed は absent、rejected/suppressed/dropped/failed は required |
| `backpressure_decision` | backpressure accepted/delayed/suppressed/degraded/dropped/closed/rejected recovery | accepted は absent、その他は required |
| `resource_bound_decision` | core-policy bounded resource accepted/rejected/dropped/shed/expired、physical owner は explicit field | accepted/within-bound observation は absent、rejected/dropped/shed/expired は required |
| `driver_resource_bound_decision` | explicitly listed driver-local resource bound decision | dropped は required |
| `configuration_decision` | typed configuration accepted/rejected/failed during startup/wiring | accepted は absent、rejected/failed は required |
| `quality_violation_decision` | quality policy violation または recovery rejection | reason category/code required |
| `token_verification_decision` | token verification accepted/rejected | accepted は absent、rejected は required |
| `driver_error_converted` | driver-local error が core reason に変換 | reason category/code required |
| `operational_probe_observation` | health/readiness/liveness/admin probe observed | satisfied observation は absent、non-satisfied/failed は required |
| `admin_maintenance_decision` | admin/maintenance action accepted/rejected/failed | accepted は absent、rejected/failed は required |
| `process_lifecycle_observation` | process crash/panic/unclean shutdown/supervisor restart observed | reason category/code required |
| `atomicity_compensation_decision` | commit/compensation path accepted/rejected/failed | accepted は absent、rejected/failed は required |
| `canonical_serialization_verification` | canonical serialization/digest verification observed | matched observation は absent、mismatch/failed は required |
| `sdk_reconnect_observation` | SDK reconnect/resumption observation | accepted は absent、rejected/failed/expired は required |
| `test_fixture_decision` | fixture/scenario data accepted/rejected/failed for evidence use | accepted は absent、rejected/failed は required |
| `command_idempotency_decision` | command idempotency/replay/correlation decision observed | accepted/idempotent observation は absent、rejected/expired/failed は required |
| `authorization_context_decision` | verified authorization context/communication policy accepted/rejected | accepted は absent、rejected/expired/failed は required |
| `deployment_topology_decision` | entrypoint/service deployment topology accepted/rejected/failed | accepted は absent、rejected/failed は required |
| `media_negotiation_decision` | codec/track/layer/payload mapping/feedback negotiation accepted/rejected/suppressed/failed | accepted は absent、rejected/suppressed/failed は required |
| `observability_signal_decision` | telemetry/audit/log/report signal accepted/rejected/dropped/failed | accepted は absent、rejected/dropped/failed は required |
| `secret_rotation_decision` | secret generation/overlap/revocation/rotation state accepted/rejected/expired/revoked/failed | accepted は absent、その他は required |
| `supply_chain_decision` | dependency/license/vulnerability/lockfile/toolchain gate accepted/rejected/failed | accepted は absent、rejected/failed は required |
| `sdk_public_api_contract_decision` | generated SDK public API contract accepted/rejected/failed against canonical source | accepted は absent、rejected/failed は required |
| `internal_control_plane_decision` | internal service-to-service control accepted/rejected/expired/failed | accepted は absent、rejected/expired/failed は required |
| `ice_candidate_connectivity_decision` | ICE candidate policy/restart/connectivity/consent decision accepted/rejected/expired/failed | accepted は absent、rejected/expired/failed は required |
| `secure_media_session_decision` | DTLS/SRTP secure media session decision accepted/rejected/expired/failed | accepted は absent、rejected/expired/failed は required |
| `operator_admin_authorization_decision` | operator/admin authorization accepted/rejected/expired/failed | accepted は absent、rejected/expired/failed は required |
| `out_of_scope_feature_decision` | excluded/future-admitted feature request accepted/rejected/failed | accepted は absent、rejected/failed は required |
| `public_endpoint_connection_decision` | public endpoint admission/connection lifecycle accepted/rejected/expired/closed/failed | accepted/closed は absent、rejected/expired/closed_by_policy/failed は required |
| `export_backup_artifact_decision` | export/backup artifact accepted/rejected/failed | accepted は absent、rejected/failed は required |
| `release_artifact_distribution_decision` | release artifact/distribution accepted/rejected/failed | accepted は absent、rejected/failed は required |
| `time_synchronization_decision` | time synchronization/clock skew observation accepted/rejected/failed/expired | accepted は absent、rejected/failed/expired は required |
| `edge_proxy_trust_decision` | edge/proxy trust/trusted metadata decision accepted/rejected/failed | accepted は absent、rejected/failed は required |
| `runtime_reconfiguration_decision` | runtime reconfiguration generation accepted/rejected/failed、rollback-required は cataloged reason 付き `failed` で表現 | accepted は absent、rejected/failed は required |
| `packet_rewrite_transform_decision` | packet rewrite/media transform intent/execution accepted/rejected/failed | accepted は absent、rejected/failed は required |
| `service_discovery_resolution_decision` | service discovery/endpoint resolution accepted/rejected/expired/failed | accepted は absent、rejected/expired/failed は required |
| `distributed_state_failover_decision` | distributed state/replication admission/owner conflict/failover decision accepted/rejected/failed | accepted は absent、rejected/failed は required |
| `runtime_task_lifecycle_decision` | runtime task/worker lifecycle accepted/rejected/expired/failed | accepted は absent、rejected/expired/failed は required |
| `internal_service_trust_decision` | internal service identity/trust mapping accepted/rejected/expired/failed | accepted は absent、rejected/expired/failed は required |
| `cross_plane_binding_decision` | cross-plane identity/session binding accepted/rejected/expired/failed | accepted は absent、rejected/expired/failed は required |

### 3.7 Decision Outcome 規則（閉集合）

decision outcome は閉集合の code として扱います。event type ごとの outcome は本節 3.9 の Event Outcome Compatibility 表から選ばなければなりません。free-text outcome は禁止です（禁止）。

| decision outcome | reason presence |
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

### 3.8 Reason Presence 規則（閉集合）

audit event reason presence は閉 field です。

| reason presence | 意味 | reason category/code |
|---|---|---|
| `none` | success outcome は rejection/suppression/failure reason を持たない | 必ず absent |
| `cataloged` | non-success outcome は core reason を持つ | 必ず present、core reason catalog に一致 |

accepted、allowed、forwarded、selected、released、closed_success、idempotent_observed、within_bound_observed outcome は `reason_presence = none` を使用します。rejected、denied、suppressed、dropped、expired、revoked、failed、delayed、degraded、shed、closed_by_policy、protocol_violation、converted_failure outcome は `reason_presence = cataloged` を使用します。

### 3.9 Event Outcome Compatibility 規則（閉集合）

event type ごとの allowed outcome は次に限定します。

| event type code | allowed decision outcomes |
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

### 3.10 Reference Presence 規則（閉集合）

audit reference presence は event type ごとに閉です。`absent_not_applicable` が唯一許可される absence marker です。rejection/failure 時点で required reference type が未だ materialize されていない場合、その reference field は `absent_not_applicable` とし、event は pre-materialization rejection/failure を説明する cataloged reason を依然 carry しなければなりません（必須）。pre-core driver conversion failure または pre-core driver resource-bound failure で client 供給の `CorrelationId` が decode/validate できない場合、`CorrelationId` reference field はこの pre-materialization rule に従い `absent_not_applicable` でなければなりません。Driver は fake client `CorrelationId` を synthesize してはなりません（禁止）。command が driver/core boundary を越えた後、`CorrelationId` は mandatory であり `absent_not_applicable` であってはなりません。

| event type code | required references | naturally present でないとき `absent_not_applicable` にする references |
|---|---|---|
| `configuration_decision` | `StartupRunId`, `ConfigurationScopeRef`, `CorrelationId` | subject transport reference, room/session reference |
| `signaling_join_decision` | `CorrelationId`, `RoomId`, participant materialize 時 `ParticipantId` | startup run ID, configuration scope reference |
| `signaling_participant_lifecycle_decision` | `CorrelationId`, `RoomId`, `ParticipantId` | startup run ID, configuration scope reference |
| `signaling_room_lifecycle_decision` | `CorrelationId`, `RoomId` | startup run ID, configuration scope reference |
| `signaling_protocol_violation` | `CorrelationId`, command が transport boundary に達した場合 subject transport reference | startup run ID, configuration scope reference |
| `signaling_relay_event` | `CorrelationId`, `RoomId`, `ParticipantId` | startup run ID, configuration scope reference |
| `turn_allocation_decision` | `CorrelationId`, materialize 時 `AllocationId`, credential 存在時 `CredentialRef`, allocation capacity/lifetime bound reason 時 resource policy owner と physical resource owner | TURN path が Signaling contract で room-scoped でない限り room/session reference |
| `turn_refresh_decision` | `CorrelationId`, `AllocationId`, credential 存在時 `CredentialRef`, refresh bound reason 時 owner tuple | TURN path が room-scoped でない限り room/session reference |
| `turn_permission_decision` | `CorrelationId`, `AllocationId`, materialize 時 `PermissionId`, permission capacity/lifetime bound reason 時 owner tuple | TURN path が room-scoped でない限り room/session reference |
| `turn_channel_bind_decision` | `CorrelationId`, `AllocationId`, `PermissionId`, materialize 時 `ChannelBindId`, channel bind lifetime bound reason 時 owner tuple | TURN path が room-scoped でない限り room/session reference |
| `turn_relay_decision` | `CorrelationId`, `AllocationId`, materialize 時 `PermissionId` | TURN path が room-scoped でない限り room/session reference |
| `sfu_session_lifecycle_decision` | `CorrelationId`, `SessionId` | startup run ID, configuration scope reference |
| `sfu_admission_decision` | `CorrelationId`, `SessionId`, materialize 時 `EndpointId` | startup run ID, configuration scope reference |
| `sfu_endpoint_lifecycle_decision` | `CorrelationId`, `SessionId`, `EndpointId` | startup run ID, configuration scope reference |
| `sfu_publication_decision` | `CorrelationId`, `SessionId`, `EndpointId`, materialize 時 `StreamId` | startup run ID, configuration scope reference |
| `sfu_subscription_decision` | `CorrelationId`, `SessionId`, `EndpointId`, materialize 時 `StreamId` | startup run ID, configuration scope reference |
| `sfu_forwarding_decision` | `CorrelationId`, `SessionId`, target-scoped 時 `EndpointId`, materialize 時 `RouteId`, packet-scoped 時 `PacketId` | startup run ID, configuration scope reference |
| `backpressure_decision` | `CorrelationId`, affected resource または route reference, endpoint-scoped 時 `EndpointId`, packet-scoped 時 `PacketId` | startup run ID, configuration scope reference |
| `resource_bound_decision` | `CorrelationId`, resource name, resource policy owner, physical resource owner, packet-scoped 時 `PacketId`, resource が TURN relay queue のとき `AllocationId` と `PermissionId` | resource が configuration-scoped でない限り startup run ID と configuration scope reference |
| `driver_resource_bound_decision` | `CorrelationId`, driver resource reference, resource policy owner, physical resource owner | 同一 correlation chain に validated `RoomId`/`SessionId` が既存でない限り room/session reference は `absent_not_applicable` |
| `quality_violation_decision` | `CorrelationId`, quality target reference, packet-scoped 時 `PacketId` | startup run ID, configuration scope reference |
| `token_verification_decision` | `CorrelationId`, credential 存在時 `CredentialRef` | startup run ID, configuration scope reference |
| `driver_error_converted` | `CorrelationId`, driver error source reference, packet lifecycle failure 変換時 `PacketId` | 同一 correlation chain に validated `RoomId`/`SessionId` が既存でない限り room/session reference は `absent_not_applicable` |
| `operational_probe_observation` | `StartupRunId`, probe class, entrypoint reference, command-scoped 時 `CorrelationId` | probe が明示 domain-scoped でない限り room/session reference |
| `admin_maintenance_decision` | `StartupRunId`, admin action reference, `CorrelationId` | action が明示 domain-scoped でない限り room/session reference |
| `process_lifecycle_observation` | `StartupRunId`, process reference, failure class | room/session reference |
| `atomicity_compensation_decision` | `CorrelationId`, atomicity class, commit boundary reference | startup-scoped でない限り startup run ID |
| `canonical_serialization_verification` | `CorrelationId`, canonical format/version, digest/hash reference | serialized object が domain-scoped でない限り room/session reference |
| `sdk_reconnect_observation` | `CorrelationId`, SDK platform, reconnect class | startup run ID, configuration scope reference |
| `test_fixture_decision` | `CorrelationId`, fixture class, fixture reference | fixture が domain-scoped でない限り room/session reference |
| `command_idempotency_decision` | `CorrelationId`, command type, idempotency class, idempotency scope, materialize 時 target reference | startup run ID, configuration scope reference |
| `authorization_context_decision` | `CorrelationId`, authorization context class, target surface, credential 存在時 `CredentialRef` | startup run ID, configuration scope reference |
| `deployment_topology_decision` | `StartupRunId`, topology class, entrypoint/service reference, command-scoped 時 `CorrelationId` | topology decision が domain-scoped でない限り room/session reference |
| `media_negotiation_decision` | `CorrelationId`, media negotiation class, materialize 時 `EndpointId`, materialize 時 `StreamId` | startup run ID, configuration scope reference |
| `observability_signal_decision` | `CorrelationId`, signal class, signal reference, cardinality/export bound reason 時 owner tuple | signal が domain-scoped でない限り room/session reference |
| `secret_rotation_decision` | `StartupRunId`, secret class, opaque generation reference, command-scoped 時 `CorrelationId` | rotation decision が domain-scoped でない限り room/session reference |
| `supply_chain_decision` | `CorrelationId`, package/toolchain reference, dependency class | room/session reference |
| `sdk_public_api_contract_decision` | `CorrelationId`, SDK platform, projection class, source contract version | room/session reference |
| `internal_control_plane_decision` | `CorrelationId`, source service, target service, control-plane class, contract version, available 時 topology class | control path が domain-scoped でない限り room/session reference |
| `ice_candidate_connectivity_decision` | `CorrelationId`, candidate/connectivity class, ICE policy reference, materialize 時 `RoomId` と `ParticipantId` | startup run ID, configuration scope reference |
| `secure_media_session_decision` | `CorrelationId`, secure media session class, media/security contract version, materialize 時 `SessionId` と `EndpointId`, 適用時 redacted key/certificate reference | startup run ID, configuration scope reference |
| `operator_admin_authorization_decision` | `CorrelationId`, operator/admin class, action class, target scope, startup/entrypoint-scoped 時 `StartupRunId` | action が明示 domain-scoped でない限り room/session reference |
| `out_of_scope_feature_decision` | command-scoped 時 `CorrelationId`, feature class, requested surface | request が domain-scoped でない限り room/session reference |
| `public_endpoint_connection_decision` | endpoint class, protocol class, target contract reference, connection lifecycle state, command-scoped 時 `CorrelationId`, startup/listener-scoped 時 `StartupRunId` | endpoint decision が domain-scoped でない限り room/session reference |
| `export_backup_artifact_decision` | `CorrelationId`, artifact class, source scope, redaction class, retention class, storage/tool owner, present 時 integrity reference | artifact が domain-scoped でない限り room/session reference |
| `release_artifact_distribution_decision` | `CorrelationId`, artifact class, source ref, package/module reference, artifact digest reference, distribution channel, provenance class | room/session reference |
| `time_synchronization_decision` | `StartupRunId`, node scope, time trust class, time source class, skew policy reference, observed skew class, command-scoped 時 `CorrelationId` | time decision が domain-scoped でない限り room/session reference |
| `edge_proxy_trust_decision` | `StartupRunId`, edge class, topology class, metadata class, trust policy reference, available 時 target endpoint class, command-scoped 時 `CorrelationId` | edge decision が domain-scoped でない限り room/session reference |
| `runtime_reconfiguration_decision` | `StartupRunId`, reconfiguration class, target surface, current generation reference, proposed generation reference, apply scope, drain/restart class, 適用時 rollback class, command-scoped 時 `CorrelationId` | reconfiguration target が domain-scoped でない限り room/session reference |
| `packet_rewrite_transform_decision` | `CorrelationId`, materialize 時 `PacketId`, materialize 時 route/target reference, rewrite/transform class, copy allowance class, execution owner | packet decision が domain-scoped でない限り room/session reference |
| `service_discovery_resolution_decision` | `StartupRunId`, discovery source class, topology class, target service, endpoint scope, resolution state, 適用時 fallback class, command-scoped 時 `CorrelationId` | resolution decision が domain-scoped でない限り room/session reference |
| `distributed_state_failover_decision` | `StartupRunId`, distributed state class, state family, owner node/scope, 適用時 affinity key, failover class, 適用時 replacement owner, command-scoped 時 `CorrelationId` | state decision が domain-scoped でない限り room/session reference |
| `runtime_task_lifecycle_decision` | `StartupRunId`, task class, parent component/supervision scope, owning layer, materialize 時 task reference, 適用時 cancellation/join bound, command-scoped 時 `CorrelationId` | task decision が domain-scoped でない限り room/session reference |
| `internal_service_trust_decision` | `StartupRunId`, trust class, source service, target service, topology class, credential/peer proof reference class, trust policy reference, endpoint resolution 関与時 endpoint scope, command-scoped 時 `CorrelationId` | trust decision が domain-scoped でない限り room/session reference |
| `cross_plane_binding_decision` | `CorrelationId`, binding class, source plane, target plane, source reference, materialize 時 target reference, 適用時 authorization context class, lifecycle state class | binding が startup-scoped でない限り startup run ID |

room/session を `absent_not_applicable` と marking する reference rule の event type には fake `RoomId` / `SessionId` を生成してはなりません（禁止）。

### 3.11 Sink 規則 / Regulated Enrichment 規則

sink implementation は driver です。PostgreSQL、S3、HTTP、file、syslog、in-memory は core ではありません。normal audit sink initialization 前に emit される `configuration_decision` は reserved bootstrap audit record path を使用します。bootstrap audit record path は bounded、non-recursive であり、selected audit sink が利用可能になる前の startup/wiring failure にのみ valid です。bootstrap audit record path が event を record できない場合、startup/wiring attempt は停止しなければならず、closeout evidence として使用してはなりません（fail-closed）。

regulated enrichment は core event を変更しません。regulated は opaque reference と non-sensitive tag を追加情報として扱います。core event の成立に regulated enrichment は必須ではありません。

### 3.12 Collapse 条件（audit event）

audit event が regulated payload を必須にする。PostgreSQL / S3 / HTTP sink が core implementation になる。audit reason が open-ended string のみになる。audit event type が open-ended string のみになる。success event に fake reason code を要求する。hash-chain semantics と persistence implementation を同一視する。audit sink/export format が default で canonical serialization として扱われる。audit/log/report path が raw secret、raw token、raw packet payload、regulated payload を露出する。各種 decision（command idempotency、authorization context、deployment topology、media negotiation、observability signal、secret rotation、supply-chain、SDK API contract、internal control-plane、ICE candidate/connectivity、secure media session、operator/admin authorization、out-of-scope feature、public endpoint、export/backup artifact、release artifact、time synchronization、edge/proxy trust、runtime reconfiguration、packet rewrite/media transform、service discovery、distributed state/failover、runtime task/worker lifecycle、internal service identity/trust、cross-plane identity/session binding）が open-ended event type または missing reference rule で record される。

---

## 4. Audit Hash-Chain Contract

### 4.1 境界（owner）

audit event meaning は本章 3 節が所有し、本節は audit record ordering と tamper-evidence semantics を所有します。

| Responsibility | Owner |
|---|---|
| audit event field semantics | core |
| hash-chain scope / sequence / previous hash semantics | core |
| hash algorithm code set | core |
| canonical record serialization contract | core |
| storage, export, retry, backup | driver |
| verifier binary / CLI composition | entrypoints |

storage driver は chain validity semantics を所有しません。entrypoints は hash-chain semantics を所有しません。

### 4.2 Chain Scope（閉集合）

hash-chain scope は explicit でなければなりません（必須）。許可される initial chain scope は次のとおりです。

| Scope | 規則 |
|---|---|
| startup chain | startup / configuration / wiring decision を record |
| signaling chain | Signaling decision と protocol violation を record |
| sfu chain | SFU decision を record |
| turn chain | TURN decision を record |
| driver chain | driver conversion/resource/failure event を record |

新しい chain scope は仕様の更新を要します。

### 4.3 Record Shape

各 hash-chain record は次を含まなければなりません（必須）。chain scope、sequence number、previous record hash または genesis marker、audit event type、audit event outcome、本章 3 節の correlation/reference field、必要時 reason category/code、canonical event payload digest、hash algorithm code、record hash。initial v0.2 hash algorithm code set は次のとおりです。

| Code | 意味 |
|---|---|
| `sha256` | canonical record input に対する SHA-256 hash |

algorithm の追加は仕様の更新を要します。

### 4.4 Canonicalization 規則

Hash input は同一 audit event に対し deterministic でなければなりません（必須）。Free-text details、log formatting、exporter timestamp formatting、DB row order、S3 key layout、tracing span metadata は hash semantics を変えてはなりません（禁止）。Canonical record input は evidence として使う前に canonical format/version と normalized field set を宣言しなければなりません（必須）。driver が operational metadata を追加する場合、仕様が明示的に admit しない限りその metadata は core hash input の外です。

### 4.5 Failure / Evidence 規則 / Persistence 関係

Hash-chain verification は evidence validation process です。chain gap、sequence mismatch、previous hash mismatch、unknown algorithm、canonical serialization failure、canonicalization mismatch が観測された場合、影響を受けた record set は close / complete / ready evidence として使用してはなりません（fail-closed）。Initial v0.2 は hash-chain verification failure を runtime domain decision reason として使用しません。Runtime domain decision は core reason catalog を使い続けなければなりません（必須）。

Persistence driver は hash-chain record を store または export します。sequence、previous hash、record hash field を保存しなければならず（必須）、record を reorder して同一 chain として提示してはなりません（禁止）。Retry と export bound は resource bound 規則（本章 11 章）と persistence boundary 規則に従います。

### 4.6 禁止 / Collapse 条件（hash-chain）

禁止: storage backend が hash-chain semantics を定義。logs/traces が hash-chain record を代替。free-text details が record hash に影響。unknown hash algorithm を accept。canonical format/version が implicit。chain を evidence として使いつつ chain gap を無視。audit event meaning を storage row shape から推論。

Collapse: hash-chain validity が persistence driver に所有される。record ordering が deterministic でない。unknown algorithm が fail-open accept される。canonical serialization mismatch が無視される。failed verification が closeout evidence として依然使用される。audit event canonical field と hash-chain record field が乖離する。
