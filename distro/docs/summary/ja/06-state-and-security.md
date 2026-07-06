# State と Security

状態: SSOT 統合版
日付: 2026-06-28 JST

## 目的

本章は、arcRTC v0.2 distro 領域（Kernel 外実装領域）における state / persistence の境界と、identity / auth / security の境界を、再現実装が可能な粒度で完全に固定します。本章は完全に自己完結しており、本仕様書内の他章番号参照のみを許可し、外部ファイル・Kernel 文書・実コードを開かずに理解および再現実装できます。

本章の中心命題は次の 2 点です。

1. distro は何を永続化し、何を永続化しないか。state の source-of-truth 規則は何か。
2. distro は identity / auth / security について何を所有し、何を所有しないか。fail-closed の境界は何か。

依存規則は `distro -> Kernel`（contract / SDK / command surface のみ）です。Kernel は communication semantics と contract を所有し、distro 側 state は Kernel contract を利用した runtime state / product state / evidence state です。distro は Kernel semantic authority ではありません。

---

## 1. State / Persistence 境界

本節は、reference state、product persistence topology、benchmark fixture state、evidence artifact の境界を固定します。

### 1.1 文脈と原則

SFU / TURN / Signaling distro は session、participant、allocation、route、relay、subscription、command evidence を扱います。state / persistence boundary を未固定にすると、reference state、product database、benchmark fixture、evidence artifact が混線し、reference success が product persistence readiness に転用されます。

Kernel は communication semantics と contract を所有します。distro 側 state は Kernel contract を利用した runtime state / product state / evidence state であり、Kernel semantic authority ではありません。

### 1.2 State Class 所有マトリクス

| State class | Owner | Persistence rule | Claim boundary |
|---|---|---|---|
| reference runtime state | reference distro | in-memory only | reference behavior に限定 |
| benchmark fixture state | benchmark harness | fixture file / in-memory | benchmark scenario execution に限定 |
| command evidence state | evidence report / command output | report artifact | evidence scope に限定 |
| product runtime state | product distro | product-owned persistence admission record が必要 | product scope に限定 |
| production database state | product distro / operations | production readiness admission record が必要 | reference scope では不採用 |

reference distro は production database を持ってはなりません（MUST NOT）。product distro が database / queue / external storage を採用する場合、まず product persistence admission decision を記録しなければなりません（MUST）。

### 1.3 Reference State Rule

reference distro の state は in-memory deterministic state のみに固定します。reference distro は database、queue、external storage、cloud persistence を持ってはなりません（MUST NOT）。

#### Required Reference State Files

| Package | File | 所有対象 |
|---|---|---|
| `arcrtc-reference-signaling` | `src/state.rs` | room / session / participant projection |
| `arcrtc-reference-turn` | `src/state.rs` | allocation / permission / relay projection |
| `arcrtc-reference-sfu` | `src/state.rs` | route / subscription / forwarding projection |
| `arcrtc-reference-composition` | `src/composition_state.rs` | cross-plane correlation projection |

各 state file は次を満たさなければなりません（MUST）。

- deterministic constructor を持つ。
- wall-clock を直接読まない。
- random value を直接生成しない。
- Kernel state type を所有しない。
- product persistence schema を含めない。

### 1.4 Product Persistence Rule

product persistence は `product-distro/persistence-topology/` が所有します。初期 product persistence は topology scaffold に限定し、具体 database provider は未採用とします。

| File | Role |
|---|---|
| `product-distro/persistence-topology/src/lib.rs` | product persistence topology export |
| `product-distro/persistence-topology/src/topology.rs` | persistence topology model |
| `product-distro/persistence-topology/src/mapper.rs` | Kernel / product projection mapping |
| `product-distro/persistence-topology/src/error.rs` | persistence evidence reason bridge |

provider-specific database item shape は、記録された product persistence provider admission decision なしに追加してはなりません（MUST NOT）。

### 1.5 State Ownership Rule

state module が所有してよいもの（MAY）は次です。

- session lifecycle projection
- participant registry projection
- relay allocation projection
- local routing projection
- benchmark fixture projection
- evidence correlation projection

state module が所有してはならないもの（MUST NOT）は次です。

- Kernel port definition
- Kernel reason catalog
- Kernel semantic decision
- product policy authority
- production readiness claim

### 1.6 Evidence State Rule（source-of-truth 規則）

evidence artifact は runtime state の source of truth ではありません。evidence artifact は report としてのみ採用します（MAY）。evidence artifact を runtime state として読み込んではなりません（MUST NOT）。

### 1.7 Persistence Prohibition

次は許可しません（MUST NOT）。

- reference in-memory state を production persistence として扱う。
- product persistence schema を Kernel contract として扱う。
- database item shape を Kernel semantic authority にする。
- benchmark fixture を production readiness の根拠にする。
- evidence artifact を runtime source of truth にする。

### 1.8 State Persistence 非主張（Non-claims）

state / persistence 境界は次を主張しません。

- product persistence design completion
- production database readiness
- durability guarantee
- backup / restore readiness
- operational readiness

---

## 2. Identity / Auth / Security 境界

本節は、reference fixture identity、local auth、TURN credential、product auth policy、secret handling の境界を固定します。

### 2.1 文脈と原則

SFU / TURN / Signaling distro は identity、credential、authorization、transport security、TURN credential、session admission を扱います。identity / auth / security boundary を未固定にすると、reference fixture credential、product authentication、live endpoint security、Kernel reason が混線します。

Kernel は auth provider、tenant identity provider、production credential issuer を所有しません。distro 側も、reference scope では production auth を発行しません。

### 2.2 Surface Decision マトリクス

| Surface | Decision | Claim boundary |
|---|---|---|
| reference identity | local fixture identity | reference behavior に限定 |
| reference signaling auth | deterministic local token / command fixture | production auth を主張しない |
| reference TURN credential | local fixture credential | production TURN security を主張しない |
| product identity provider | product distro admission record が必要 | product scope に限定 |
| production security | production readiness admission record が必要 | reference scope では不採用 |

secret、credential、token、private key を reports、source fixture に実秘密として保存してはなりません（MUST NOT）。fixture credential は明示的に fixture と分かる値に限定しなければなりません（MUST）。

### 2.3 Reference Fixture Identity

reference distro は deterministic fixture identity のみを使用しなければなりません（MUST）。

| Fixture field | Format | Owner |
|---|---|---|
| `identity_id` | `fixture-identity-{n}` | reference signaling / composition |
| `session_id` | `fixture-session-{n}` | reference signaling |
| `room_id` | `fixture-room-{n}` | reference signaling |
| `turn_credential_id` | `fixture-turn-credential-{n}` | reference TURN |
| `sfu_route_id` | `fixture-sfu-route-{n}` | reference SFU |

fixture value は production credential として扱ってはなりません（MUST NOT）。secret、private key、real token を repository / report に保存してはなりません（MUST NOT）。

### 2.4 Required Security Files

| Package | File | Role |
|---|---|---|
| `arcrtc-reference-signaling` | `src/fixture_identity.rs` | local identity / session fixture |
| `arcrtc-reference-signaling` | `src/local_auth.rs` | deterministic local auth check |
| `arcrtc-reference-turn` | `src/fixture_credential.rs` | local TURN credential fixture |
| `arcrtc-reference-sfu` | `src/fixture_route_auth.rs` | local route admission fixture |
| `arcrtc-product-policy` | `src/auth_policy.rs` | product auth policy boundary |
| `arcrtc-product-policy` | `src/security_reason.rs` | product security evidence reason bridge |

### 2.5 Security Boundary Rule

security module が所有してよいもの（MAY）は次です。

- fixture identity parsing
- fixture credential verification
- local command admission check
- product auth policy interface
- distro evidence reason mapping

security module が所有してはならないもの（MUST NOT）は次です。

- Kernel reason catalog
- Kernel port definition
- production identity provider authority
- production secret lifecycle
- live readiness claim

### 2.6 Product Auth Rule / Admission

product auth provider は初期状態では未採用です。product distro が auth provider を採用する場合、次を先に固定しなければなりません（MUST）。

- provider owner
- credential source
- token validation rule
- secret storage boundary
- failure reason closed set
- audit evidence boundary
- non-claim scope

### 2.7 Forbidden Pattern

次は許可しません（MUST NOT）。

- fixture token を production auth として扱う。
- product auth failure reason を Kernel reason catalog に追加する。
- secret を report に記録する。
- auth command success を live readiness として扱う。

### 2.8 Identity / Auth / Security 非主張（Non-claims）

identity / auth / security 境界は次を主張しません。

- production auth readiness
- security readiness
- TURN production credential readiness
- public endpoint security
- identity provider integration completion

---

## 3. 不変条件と fail-closed 条件（Collapse Conditions）

本章の正典は、次のいずれかが起きた場合に崩れます。これらは fail-closed の境界であり、いずれも禁止です（MUST NOT）。

### 3.1 State / Persistence 不変条件

- reference distro が database / queue / external storage を直接持つ。
- reference distro が production database を直接採用する。
- product persistence schema を Kernel contract として扱う。
- product persistence schema が Kernel semantics を上書きする。
- database item shape を Kernel semantic authority にする。
- evidence artifact を runtime state として読み込む。
- evidence artifact を runtime source of truth にする。
- benchmark fixture state を product state として継承する。
- state persistence success を live readiness として扱う。
- persistence owner が記録された admission decision なしに source へ追加される。

### 3.2 Identity / Auth / Security 不変条件

- reference distro が real credential issuer を持つ。
- reference distro が production credential issuer を持つ。
- fixture identity が product identity source of truth になる。
- fixture token / fixture credential を production auth として扱う。
- secret を reports / source fixture に保存する。
- product auth provider を記録された admission decision なしに導入する。
- product auth failure reason を Kernel reason catalog に混入する。
- auth command success を live readiness として扱う。
- security evidence が production security readiness を自動主張する。
- identity provider choice を記録された admission decision なしに source へ導入する。
