# drivers-security-secrets

状態: SSOT 統合版
日付: 2026-06-28 JST

## 目的

本章は arcRTC v0.2 Kernel の security / secrets driver family の現行完全仕様を、本章のみで再現実装可能な粒度で内在化することを目的とします。本章は security key source / verifier driver 境界（key fetch / cache / refresh / cryptographic backend / failure mapping）、transport security configuration（TLS/mTLS、DTLS/SRTP backend、certificate/key source、allowed cipher/profile、secret handling、startup failure）、secret rotation lifecycle の全 state と全手順を、所有者・閉集合語彙・状態機械・failure mapping・禁止/許可・fail-closed 条件まで落とさずに内在化します。

依存方向の表記: `A <- B` は「B が A に依存」を意味します。token verification semantics と issuer/audience/claim policy は core が所有し、key fetch / cache / refresh / concrete crypto backend は driver が所有します。entrypoints は typed configuration / typed reference を渡し、driver はそれを実装します。arcRTC は token issuance / credential issuance を所有せず、externally issued token / credential の verification boundary のみを持ちます。本章は secret manager 実装、certificate deployment、cryptographic certification、runtime rotation / security verification 成功を主張しません。required secure mode が失敗した場合の insecure fallback はありません（fail-closed）。

---

## 1. Security Key Source / Verifier Driver 境界

### 1.1 Boundary（所有割当、閉集合）

token verification semantics は token verification 規則が定義し、本節は key fetch、cache、refresh、cryptographic backend、failure mapping の driver 境界を固定します。secret rotation lifecycle は第3節に従います。

| Surface | Owner | Rule |
|---|---|---|
| verification request/result semantics | core | token verification decision and reason |
| issuer/audience/claim policy | core | abstract policy input |
| key source configuration | entrypoints | typed configuration only |
| key fetch/cache/refresh | driver | bounded external implementation |
| JWT/JWK/parser/crypto library | driver | concrete implementation only |
| token issuance | external system | arcRTC does not own |

### 1.2 Key Source Types（閉集合）

v0.2 initial key source type は次のとおりです。新規 key source type は v0.2 初期 scope 外です。

| Key source | Owner | Rule |
|---|---|---|
| static typed key material | entrypoints supplies typed config; driver verifies shape | no raw secret in core |
| JWKS endpoint | driver fetch/cache | bounded HTTP and cache policy required |
| file/secret manager reference | entrypoints supplies typed reference; driver loads | core sees opaque credential/key reference only |

### 1.3 Cache / Refresh Rule（必須 bound）

key cache は driver 所有であり bounded でなければなりません。必須 bound: maximum keys、maximum key material bytes、maximum cache age、maximum failed refresh attempts、maximum refresh wait time。cache miss または stale key は driver refresh を trigger してよいが、refresh failure は token を silently に accept してはなりません（禁止）。bound 内に key を取得できない場合、verification は `token_key_unavailable` で fail closed します。generation overlap、revocation、stale key acceptance は secret rotation 規則（第3節）が支配します。

### 1.4 Verification Flow（順序固定）

1. entrypoints wires typed verifier configuration and selected driver.
2. driver loads/fetches bounded key material.
3. driver parses token and performs concrete crypto verification.
4. driver maps crypto/key errors to core-owned verification result.
5. core applies issuer/audience/claim/time policy and emits decision.

driver は entrypoint-specific domain role authorization を適用してはなりません（禁止）。core は HTTP key fetch または concrete crypto library call を行ってはなりません（禁止）。

### 1.5 Failure Mapping（閉集合）

driver-local crypto/library error は次の cataloged reason のいずれかへ変換しなければなりません。

| Failure | Required reason |
|---|---|
| token absent | `token_missing` |
| token cannot be decoded | `token_malformed` |
| signature invalid | `token_signature_invalid` |
| key source unavailable / bounded lookup failed | `token_key_unavailable` |
| issuer mismatch | `token_issuer_mismatch` |
| audience mismatch | `token_audience_mismatch` |
| token expired | `token_expired` |
| token not yet valid | `token_not_yet_valid` |
| required claim absent | `token_required_claim_missing` |
| unsupported algorithm | `token_unsupported_algorithm` |
| required secret/key source config missing | `secret_unavailable` or `runtime_config_missing` by startup phase |
| key generation revoked or outside overlap | secret rotation reason from 第3節 |

### 1.6 Secret Handling Rule（secret 取扱規則）

raw key material、raw token、raw secret、cryptographic backend error detail は audit event、log、metric label、SDK public error、core domain state に現れてはなりません（禁止）。privacy / redaction / retention handling は privacy / redaction / retention 規則に従います。transport certificate/private-key handling は、secret が transport security の一部のとき transport security configuration 規則（第2節）に従います。core は opaque credential/key reference と verification outcome のみを運んでよい。driver は operational detail を、canonical reason として露出しない non-authoritative diagnostic detail としてのみ運んでよい。

### 1.7 Prohibitions（禁止、閉集合）

- core が JWKS または file/secret material を fetch する。
- driver が key refresh 失敗時に token を accept する。
- driver が rotation policy なしに revoked または expired generation を accept する。
- key cache が unbounded である。
- raw token/key material が evidence として persist または log される。
- entrypoint-specific role authorization が generic token verification に混入する。
- token issuance が arcRTC 責務として扱われる。
- driver error が core decision で open-ended string のまま残る。

### 1.8 Key Source / Verifier Collapse Conditions（崩壊条件）

token verification が key source failure 時に fail open し得る。raw secret または token が authoritative data として core/audit/log に渡る。key source cache に bound または refresh failure reason が無い。key generation overlap/revocation behavior が implicit である。verifier driver が domain authorization を所有する。token issuance が v0.2 communication-core 責務になる。

---

## 2. Transport Security Configuration

### 2.1 Boundary（所有割当、閉集合）

本節は TLS/mTLS、DTLS/SRTP backend、certificate/key source、allowed cipher/profile、secret handling、startup failure の owner を固定します。secret rotation lifecycle は第3節に、secure media session lifecycle は secure media session lifecycle 規則に、edge TLS termination と downstream security trust は edge/proxy trust boundary 規則に、internal service identity / trust は internal service identity trust 規則に従います。

| Concern | Owner | Rule |
|---|---|---|
| abstract security requirement | core | transport must satisfy accepted policy before sensitive path |
| accepted protocol/profile policy | core | typed policy input として評価 |
| certificate/key source reference | entrypoints | typed runtime/config reference を渡す |
| raw certificate/private key loading | driver | concrete file/secret manager/backend handling |
| TLS/mTLS/DTLS/SRTP implementation | driver | concrete library and runtime detail |
| internal service peer identity | service identity policy | peer proof is mapped before internal control authorization |
| endpoint/listener selection | entrypoints | driver wiring only |
| edge TLS termination | edge/proxy trust boundary | not backend security proof by default |
| redaction/retention | observability/reporting boundary | raw secret material を出さない |

core は raw private key、raw certificate file、secret manager client、TLS library type、DTLS/SRTP session object を所有しません。

### 2.2 Configuration Rule（configuration 規則）

entrypoints は environment variable、file、process args、deployment settings、secret reference を読んでよい。entrypoints はそれらを typed runtime configuration と typed core policy input に変換しなければなりません。

core policy は次を含んでよい（許可）: required transport security mode、accepted protocol/profile version、abstract policy としての peer verification requirement、command path 受理前に必要な minimum verification result、accepted media/security contract version。

driver runtime configuration は次を含んでよい（許可）: listener bind setting、certificate/key source reference、trust anchor source reference、concrete backend selection、driver-local timeout/bounds、supported な場合の rotation source reference and reload bounds。

driver runtime configuration は core security policy を再定義してはなりません（禁止）。

### 2.3 Secret Handling Rule（secret 取扱規則）

raw private key、raw shared secret、raw token、raw certificate private material は次に入ってはなりません（禁止）: core state、audit event、log/trace、metric label、SDK public error、the evidence report body。report は opaque secret source reference、policy が許す場合の certificate fingerprint/hash、redacted diagnostic summary を含んでよい。

### 2.4 Failure Mapping（閉集合）

transport security startup/path failure は fail closed しなければなりません。non-sensitive development-only composition が明示的に定義され、その evidence が production readiness として使用されない場合を除き、insecure fallback は許可されません。

| Failure | Required reason |
|---|---|
| required runtime security configuration missing | `runtime_config_missing` |
| runtime security configuration invalid | `runtime_config_invalid` |
| required secret/key source unavailable | `secret_unavailable` |
| required secret rotation state unavailable | `secret_rotation_state_unavailable` |
| transport secret/key generation revoked | `secret_key_revoked` |
| transport/media contract version unsupported | `unsupported_media_contract_version` |
| network receive failed after secure transport setup attempt | `network_receive_failed` |
| network send failed after secure transport setup attempt | `network_send_failed` |
| driver shutdown | `driver_shutdown` |

### 2.5 Verification Evidence Rule（検証 evidence 区別、閉集合）

transport security evidence は次を区別しなければなりません。1 つの class の evidence は別の class を証明しません。

- configuration shape validation;
- secret source availability;
- listener startup;
- peer verification behavior;
- mTLS または service credential が internal control に使われる場合の internal service identity/trust mapping;
- DTLS/SRTP or TLS session establishment;
- negative test for invalid peer or missing secret;
- redaction of secret material in logs/reports;
- rotation が claim に影響する場合の rotation generation/overlap evidence;
- DTLS/SRTP protected media path が claim の一部のときの secure media session evidence;
- TLS が entrypoint listener より前で terminate する場合の edge termination and downstream protection evidence.

### 2.6 Prohibitions（禁止、閉集合）

- core が certificate/key file を読む。
- required secure mode 失敗後に driver が insecure fallback を accept する。
- TLS/mTLS/DTLS/SRTP concrete library object が core に渡る。
- raw key/token/secret が audit/log/report に現れる。
- entrypoint configuration branch が core security semantics を silently に変える。
- listener startup success が peer verification success として扱われる。
- peer verification success が internal service authorization success として扱われる。
- stale または revoked transport secret が rotation policy なしに accept される。
- edge TLS termination が edge trust policy なしに backend secure transport として扱われる。

### 2.7 Transport Security Configuration Collapse Conditions（崩壊条件）

raw transport secret が core-owned になる。required secure mode が insecure mode へ fail open し得る。driver runtime configuration が core security policy を override する。security evidence が listener startup を peer verification と混同する。report/log が secret material を露出する。secure transport readiness を主張しながら rotation state が省略される。transport configuration だけから secure media session readiness が主張される。secure transport readiness を主張しながら TLS termination boundary が implicit である。TLS/mTLS configuration だけから internal service trust が推測される。

---

## 3. Secret Rotation Lifecycle

### 3.1 Boundary（所有割当、閉集合）

本節は JWT/JWKS、TURN shared secret、TLS certificate/private key、DTLS/SRTP material、opaque secret reference の rotation、overlap、revocation、stale-key handling が fail-open にならないよう固定します。raw secret material は core、audit、logs、metrics、SDK public error、reports に入ってはなりません（禁止）。

| Concern | Owner | Rule |
|---|---|---|
| secret source reference | entrypoints typed configuration | raw secret ではなく reference |
| raw secret/key material load | driver | concrete file/secret-manager/backend detail |
| verification/acceptance policy | core typed policy where semantic | accepted generation, overlap, expiry |
| rotation execution | driver/entrypoints | reload/fetch/swap implementation |
| token/key verification | service identity policy | token result and key source failure mapping |
| transport security material | transport security policy | TLS/DTLS/SRTP boundary |
| evidence/reporting | reports | redacted rotation state only |

### 3.2 Secret Classes（閉集合）

v0.2 initial architecture の secret class は次に限定します。新規 secret class は v0.2 初期 scope 外です。

| Class | Meaning | Rule |
|---|---|---|
| `jwt_verification_key` | token verification key/JWKS material | key source driver owns fetch/cache |
| `turn_shared_secret` | TURN credential derivation or HMAC secret | rotation overlap required when active credentials exist |
| `transport_certificate_key` | TLS/mTLS/DTLS/SRTP private material | transport security policy applies |
| `opaque_secret_reference` | reference to external secret manager/file/env source | entrypoints may pass typed reference |
| `ephemeral_session_key_material` | backend session material | driver-owned, no report/log exposure |

### 3.3 Rotation State Rule（rotation 状態機械、閉集合）

rotation lifecycle は次の state を使用します。state name は policy/evidence classification であり raw key identifier ではありません。

| State | Meaning |
|---|---|
| `current_generation` | accepted for new verification/issuance use where applicable |
| `previous_generation_overlap` | accepted only for bounded verification overlap |
| `pending_generation` | loaded but not accepted for decision |
| `revoked_generation` | must not be accepted |
| `expired_generation` | lifetime ended |

### 3.4 Overlap and Revocation Rule（必須定義項目）

rotation policy は次を定義しなければなりません。secret class、generation reference format、maximum overlap window、revocation behavior、active credential/session relation、stale または revoked material の failure reason、audit/evidence relation、redaction rule。required な場所で rotation state を判定できない場合、affected secure path は fail closed しなければなりません。

### 3.5 Failure Mapping（閉集合）

| Failure | Required reason |
|---|---|
| required rotation has not occurred | `secret_rotation_required` |
| presented generation is not accepted | `secret_generation_not_accepted` |
| key/secret generation is revoked | `secret_key_revoked` |
| overlap window expired | `secret_overlap_window_expired` |
| rotation state unavailable | `secret_rotation_state_unavailable` |
| secret source unavailable | `secret_unavailable` |
| runtime security configuration missing | `runtime_config_missing` |
| runtime security configuration invalid | `runtime_config_invalid` |
| token key unavailable | `token_key_unavailable` |

### 3.6 Evidence Rule（evidence 規則）

rotation evidence は opaque secret reference、policy が許す場合の generation reference hash/fingerprint、rotation state、overlap window、redacted diagnostic summary を含んでよい。それは raw secret、raw token、raw private key、raw certificate private material、backend-specific secret payload を含んではなりません（禁止）。

### 3.7 Audit Rule

secret rotation decision は audit event type `secret_rotation_decision` を使用します。当該 event は secret class、opaque generation reference、startup run ID、command-scoped 時の `CorrelationId` を持たなければなりません（必須）。

### 3.8 Prohibitions と Collapse Conditions

禁止（閉集合）: stale generation が overlap policy なしに accept される。revoked key が cache convenience のため accept される。secret rotation failure が insecure mode へ fall back する。raw secret material が audit/log/report に書かれる。driver-local rotation state が core security policy を silently に変える。token issuance が arcRTC 責務として扱われる。崩壊条件: rotation state が implicit である。overlap window が unbounded である。revocation が fail open し得る。report が raw secret material を露出する。current/previous/pending generation semantics が core 契約更新なしに driver ごとに異なる。

---

## 4. 章全体の fail-closed 不変条件

本章の全 security / secrets driver に共通する fail-closed 不変条件は次のとおりです（必須）。token verification は key source failure / refresh failure / rotation state unavailable のとき fail open せず、`token_key_unavailable` または対応する cataloged reason で fail closed する。required secure mode が失敗した場合、insecure fallback は許可されず fail closed する。key cache / rotation overlap window は bounded であり、stale または revoked generation は overlap/rotation policy なしに accept しない。raw secret / token / key material / certificate private material / cryptographic backend error detail は core state / audit event / log / metric label / SDK public error / report のいずれにも入らない。arcRTC は token / credential issuance を所有せず verification boundary のみを持つ。listener startup success / peer verification success / TLS configuration はそれぞれ peer verification / internal service authorization / secure media session readiness を証明しない。これらの不変条件を満たさない path は close / complete / ready evidence として採用してはなりません。
