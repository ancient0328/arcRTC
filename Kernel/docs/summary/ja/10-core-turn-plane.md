# core-turn-plane

状態: public summary projection
日付: 2026-07-06 JST

## 目的

本章は arcRTC v0.2 Kernel の `core/turn` が所有する TURN contract と TURN lifecycle の現行完全仕様を、本章のみで再現実装可能な粒度で内在化することを目的とします。本章は TURN allocation / permission / channel bind / relay semantics を core 所有とし、UDP/TCP/socket I/O、tokio、SIMD backend、HMAC/crypto concrete implementation などの wire driver execution を driver 所有として分離します。本章は allocation / permission / channel bind の全 state と全遷移、guard、reject/deny 条件、expiry/refresh/release 規則、credential verification boundary、fail-closed 条件を内在化します。

依存方向の表記: `A <- B` は「B が A に依存」を意味します。TURN の pure semantics は core が所有し、I/O と分離されます。v0.2 Kernel は TURN product system を所有せず、reference distro / product distro は Kernel 外 distro に置きます。arcRTC は credential issuance を所有せず、externally issued credential / token の verification boundary のみを持ちます。

---

## 1. TURN Contract

### 1.1 境界（owner）

| 領域 | 所有者 | 規則 |
|---|---|---|
| TURN semantics | `core/turn` | STUN/TURN message semantic model、allocation/permission/channel bind/relay decision、credential verification boundary、fail-closed rule |
| UDP / TCP / socket I/O | `drivers/network` | concrete network I/O |
| executable contract / composition evidence surface | `entrypoints/turn-server` | server executable / configuration / listener setup / dependency wiring / shutdown signal handling |

STUN/TURN wire decode/encode driver 境界は TURN wire driver 規則に従います。TURN credential shared secret / generation / overlap / revocation lifecycle は secret rotation lifecycle 規則に従います。cross-plane identity / session binding は cross-plane binding 規則に従います。

### 1.2 Core Model（core 所有 model、閉集合）

core は次を model として所有します（必須）。

- transaction ID
- request / response / indication
- allocation
- permission
- channel binding
- channel binding reference
- core-owned address type としての peer address
- relay decision
- credential verification outcome
- lifetime / expiry
- closed error reason

### 1.3 Core Decisions（core 所有 decision、閉集合）

core は次の decision を所有します（必須）。

- allocation accepted / rejected
- allocation released / expired
- refresh accepted / rejected / expired
- permission accepted / rejected / revoked
- permission expired
- channel bind accepted / rejected
- channel bind expired
- relay allowed / denied
- malformed message classification
- expired credential classification
- unauthorized request classification

### 1.4 Driver Responsibilities

driver は次を core に持ち込んではなりません（禁止）。UDP socket、TCP listener/stream、tokio task、raw external byte buffer ownership、SIMD backend type、concrete HMAC library type、OS network interface type。

### 1.5 Fail-Closed 規則（decision handling と reason code 閉集合）

次の場合は handling 列に従って fail-closed とし、cataloged reason code に接続しなければなりません（必須）。

| Failure | Decision handling | Reason code |
|---|---|---|
| malformed STUN/TURN message | rejected | `malformed_turn_message` |
| invalid transaction ID | rejected | `malformed_turn_message` |
| missing credential proof | allocation/refresh/permission/channel bind command は rejected、relay data は denied | `credential_missing` |
| invalid credential proof | allocation/refresh/permission/channel bind command は rejected、relay data は denied | `credential_invalid` |
| expired credential | allocation/refresh/permission/channel bind command は rejected、relay data は denied | `credential_expired` |
| credential generation が active rotation policy で非 accept | allocation/refresh/permission/channel bind command は rejected、relay data は denied | `secret_generation_not_accepted` |
| credential key または generation が revoked | allocation/refresh/permission/channel bind command は rejected、relay data は denied | `secret_key_revoked` |
| prior-generation overlap window 期限切れ | allocation/refresh/permission/channel bind command は rejected、relay data は denied | `secret_overlap_window_expired` |
| rotation state source 利用不可 | allocation/refresh/permission/channel bind command は rejected、relay data は denied | `secret_rotation_state_unavailable` |
| allocation capacity 超過 | rejected | `allocation_capacity_exceeded` |
| permission capacity 超過 | rejected | `permission_capacity_exceeded` |
| unauthorized peer | rejected | `peer_not_allowed` |
| peer policy により permission revoked | revoked | `peer_not_allowed` |
| active permission 必須箇所で missing/非 active permission | active permission を要求する command（channel bind 含む）は rejected、relay data は denied | `permission_not_found` |
| relay denied | denied | `relay_denied` |
| allocation missing または非 active | refresh/permission/channel bind/release command は rejected、relay data は denied | `allocation_not_found` |
| unsupported method | rejected | `unsupported_turn_method` |
| unsupported TURN contract version | rejected | `unsupported_turn_contract_version` |
| lifetime violation | rejected | `turn_lifetime_violation` |
| allocation lifetime cap 超過 | expired | `allocation_lifetime_exceeded` |
| permission lifetime 期限切れ | expired | `permission_lifetime_exceeded` |
| channel bind lifetime 期限切れ | expired | `channel_bind_lifetime_exceeded` |
| refresh count または cumulative refresh cap 超過 | expired | `refresh_limit_exceeded` |

TURN relay queue capacity または wait limit 超過は `turn_relay_decision` denial ではありません。これは resource-bound scheduling drop であり、`resource_bound_decision` を reason `turn_relay_queue_bound_exceeded`、active `AllocationId`、active `PermissionId`、resource policy owner `core`、physical resource owner `driver` で使用しなければなりません（必須）。`permission_lifetime_exceeded` と `channel_bind_lifetime_exceeded` は expiry reason です。invalid requested lifetime による permission/channel bind request rejection は `turn_lifetime_violation` を使用します。

### 1.6 Auth Boundary

- arcRTC は credential issuance を所有しません（必須）。
- arcRTC は externally issued credential / token の verification boundary を持ちます。
- secret generation acceptance、overlap、revocation、rotation state observation は verification inputs であり credential issuance ではありません。
- TURN core は credential generation に対して reject を decide してよいが、raw secrets を fetch/mint/persist/rotate してはなりません（禁止）。
- TURN credential delivery / allocation / permission / channel bind success は、admitted cross-plane binding と target-plane decision なしに Signaling/SFU/secure media success にはなりません（必須）。

### 1.7 collapse 条件（TURN contract）

- socket loop が TURN decision の正を持つ。
- core が UDP/TCP concrete type を参照する。
- driver が permission rule を所有する。
- TURN credential issuance を core responsibility として扱う。
- TURN credential rotation failure が generic credential failure に潰される。
- raw secret / generation material が core model、audit、evidence に露出する。
- Signaling participant or SFU endpoint relation が cross-plane binding なしに TURN credential/allocation/permission/relay observation から推定される。

---

## 2. TURN Boundary（plane 分離決定）

### 2.1 Decision

TURN の中核意味論は `core/turn` が所有します。UDP / TCP / socket / tokio / SIMD backend / process lifecycle は drivers または entrypoints が所有します（必須）。

### 2.2 Core TURN Responsibilities

`core/turn` は次を所有します（必須）。STUN/TURN message semantic model、request/response/indication/error semantics、allocation lifecycle、refresh semantics、permission lifecycle、channel bind semantics、relay decision、credential verification boundary、abstract rule としての nonce/realm/timestamp policy、malformed/expired/unauthorized request の fail-closed rule。

### 2.3 Driver / Entrypoint Responsibilities

TURN driver: UDP socket I/O、TCP framing I/O、packet read/write、runtime task scheduling、HMAC/crypto library concrete implementation、SIMD/CPU feature optimization、external byte buffer conversion、metrics export。

TURN entrypoint: server executable / product-system wiring detail、configuration loading、network listener setup、dependency wiring、shutdown signal handling。

### 2.4 Prohibited Placement / collapse 条件

禁止: socket read loop に allocation decision の正を置く。TCP framing に permission rule の正を置く。core が `tokio::net` / `socket2` / SIMD backend concrete type に依存する。entrypoint が TURN protocol error classification を所有する。TURN credential issuance を arcRTC が所有する（arcRTC は検証境界を持つだけ）。

collapse: core が socket/runtime/SIMD concrete implementation を参照する。driver が allocation/permission/relay rule を所有する。entrypoint が protocol semantics を分岐実装する。TURN auth を token issuance と誤認する。

---

## 3. TURN Lifecycle

TURN lifecycle semantics の正は `core/turn` が所有します。

### 3.1 Lifecycle Owners

| Lifecycle | Owner | Driver role |
|---|---|---|
| Allocation lifecycle | core | socket / packet I/O |
| Permission lifecycle | core | address conversion |
| Channel bind lifecycle | core | byte framing conversion |
| Credential verification outcome | core | crypto / token verifier implementation |
| Timer execution | driver | core-owned expiry rule execution |

driver socket shutdown は、allocation/permission/channel bind lifecycle の silent restoration、silent release、unaudited mutation を authorize しません（必須）。cross-plane shutdown/drain、command/decision/event result shape、concurrency/ordering/lock ownership、retry/timeout/cancellation、cross-plane identity/session binding はそれぞれの規則体系に従います。

### 3.2 Allocation States（閉集合）

| State | 意味 |
|---|---|
| `allocation_absent` | allocation が存在しない |
| `allocation_requested` | request 評価中 |
| `allocation_active` | permission が許可するとき relay 可 |
| `allocation_refreshing` | refresh 評価中 |
| `allocation_expired` | lifetime 終了 |
| `allocation_released` | allocation が意図的に release |
| `allocation_rejected` | request rejected |

### 3.3 Permission States（閉集合）

| State | 意味 |
|---|---|
| `permission_absent` | peer の permission なし |
| `permission_requested` | permission request 評価中 |
| `permission_active` | peer relay 許可 |
| `permission_expired` | permission lifetime 終了 |
| `permission_revoked` | permission が許可されなくなった |
| `permission_rejected` | request rejected |

### 3.4 Channel Bind States（閉集合）

| State | 意味 |
|---|---|
| `channel_unbound` | channel binding なし |
| `channel_bind_requested` | bind request 評価中 |
| `channel_bound` | channel 使用可 |
| `channel_expired` | binding lifetime 終了 |
| `channel_rejected` | bind request rejected |

### 3.5 Transition Rules（全遷移表）

pre-state tuple 表記:

- `family={a, b}` は同 family のいずれか 1 state が必須。
- `family=x; other_family=y` は両 family state が同時に必須。
- 省略された family は precondition なし。
- failure 列の `success-only` は failure branch を持たない行であり、invalid pre-state は別個の rejection 行で表現される。

| Event | Allowed pre-state | Success state | Failure state | Decision reason code |
|---|---|---|---|---|
| `Allocate` | `allocation=allocation_absent` | `allocation_requested` | `allocation_rejected` | `credential_missing`, `credential_invalid`, `credential_expired`, `allocation_capacity_exceeded` |
| `AcceptAllocation` | `allocation=allocation_requested` | `allocation_active` | `allocation_rejected` | `credential_invalid`, `credential_expired`, `allocation_capacity_exceeded` |
| `RejectAllocation` | `allocation=allocation_requested` | `allocation_rejected` | `allocation_rejected` | `credential_missing`, `credential_invalid`, `credential_expired`, `allocation_capacity_exceeded` |
| `Refresh` | `allocation=allocation_active` | `allocation_refreshing` | `allocation_active` | `credential_missing`, `credential_invalid`, `credential_expired` |
| `RejectRefreshLifetimeViolation` | `allocation=allocation_active` | `allocation_active` | `allocation_active` | `turn_lifetime_violation` |
| `AcceptRefresh` | `allocation=allocation_refreshing` | `allocation_active` | `allocation_active` | `credential_missing`, `credential_invalid`, `credential_expired` |
| `RejectRefresh` | `allocation=allocation_refreshing` | `allocation_active` | `allocation_active` | `credential_missing`, `credential_invalid`, `credential_expired` |
| `RejectRefreshAbsent` | `allocation=allocation_absent` | `allocation_absent` | `allocation_absent` | `allocation_not_found` |
| `RejectRefreshRequested` | `allocation=allocation_requested` | `allocation_requested` | `allocation_requested` | `allocation_not_found` |
| `RejectRefreshExpired` | `allocation=allocation_expired` | `allocation_expired` | `allocation_expired` | `allocation_not_found` |
| `RejectRefreshReleased` | `allocation=allocation_released` | `allocation_released` | `allocation_released` | `allocation_not_found` |
| `RejectRefreshRejected` | `allocation=allocation_rejected` | `allocation_rejected` | `allocation_rejected` | `allocation_not_found` |
| `Release` | `allocation={allocation_active, allocation_refreshing}` | `allocation_released` | success-only | success-only |
| `RejectReleaseAbsent` | `allocation=allocation_absent` | `allocation_absent` | `allocation_absent` | `allocation_not_found` |
| `RejectReleaseRequested` | `allocation=allocation_requested` | `allocation_requested` | `allocation_requested` | `allocation_not_found` |
| `RejectReleaseExpired` | `allocation=allocation_expired` | `allocation_expired` | `allocation_expired` | `allocation_not_found` |
| `RejectReleaseReleased` | `allocation=allocation_released` | `allocation_released` | `allocation_released` | `allocation_not_found` |
| `RejectReleaseRejected` | `allocation=allocation_rejected` | `allocation_rejected` | `allocation_rejected` | `allocation_not_found` |
| `ExpireAllocationLifetime` | `allocation={allocation_active, allocation_refreshing}` | `allocation_expired` | `allocation_expired` | `allocation_lifetime_exceeded` |
| `ExpireRefreshCap` | `allocation={allocation_active, allocation_refreshing}` | `allocation_expired` | `allocation_expired` | `refresh_limit_exceeded` |
| `CreatePermission` | `allocation=allocation_active; permission=permission_absent` | `permission_requested` | `permission_rejected` | `credential_missing`, `credential_invalid`, `credential_expired`, `allocation_not_found`, `peer_not_allowed`, `permission_capacity_exceeded`, `turn_lifetime_violation` |
| `AcceptPermission` | `allocation=allocation_active; permission=permission_requested` | `permission_active` | `permission_rejected` | `credential_missing`, `credential_invalid`, `credential_expired`, `allocation_not_found`, `peer_not_allowed`, `permission_capacity_exceeded`, `turn_lifetime_violation` |
| `RejectPermission` | `permission=permission_requested` | `permission_rejected` | `permission_rejected` | `credential_missing`, `credential_invalid`, `credential_expired`, `allocation_not_found`, `peer_not_allowed`, `permission_capacity_exceeded`, `turn_lifetime_violation` |
| `ExpirePermission` | `permission=permission_active` | `permission_expired` | `permission_expired` | `permission_lifetime_exceeded` |
| `RevokePermission` | `permission=permission_active` | `permission_revoked` | `permission_revoked` | `peer_not_allowed` |
| `ChannelBind` | `allocation=allocation_active; permission=permission_active; channel=channel_unbound` | `channel_bind_requested` | `channel_rejected` | `credential_missing`, `credential_invalid`, `credential_expired`, `allocation_not_found`, `permission_not_found`, `relay_denied`, `turn_lifetime_violation` |
| `AcceptChannelBind` | `allocation=allocation_active; permission=permission_active; channel=channel_bind_requested` | `channel_bound` | `channel_rejected` | `credential_missing`, `credential_invalid`, `credential_expired`, `allocation_not_found`, `permission_not_found`, `relay_denied`, `turn_lifetime_violation` |
| `RejectChannelBind` | `channel=channel_bind_requested` | `channel_rejected` | `channel_rejected` | `credential_missing`, `credential_invalid`, `credential_expired`, `allocation_not_found`, `permission_not_found`, `relay_denied`, `turn_lifetime_violation` |
| `ExpireChannelBind` | `channel=channel_bound` | `channel_expired` | `channel_expired` | `channel_bind_lifetime_exceeded` |
| `RelayData` | `allocation=allocation_active; permission=permission_active` | `allocation_active`, `permission_active` | `allocation_active`, `permission_active` | `credential_missing`, `credential_invalid`, `credential_expired`, `relay_denied` |
| `DenyRelayDataAllocationAbsent` | `allocation=allocation_absent` | `allocation_absent` | `allocation_absent` | `allocation_not_found` |
| `DenyRelayDataAllocationRequested` | `allocation=allocation_requested` | `allocation_requested` | `allocation_requested` | `allocation_not_found` |
| `DenyRelayDataAllocationExpired` | `allocation=allocation_expired` | `allocation_expired` | `allocation_expired` | `allocation_not_found` |
| `DenyRelayDataAllocationReleased` | `allocation=allocation_released` | `allocation_released` | `allocation_released` | `allocation_not_found` |
| `DenyRelayDataAllocationRejected` | `allocation=allocation_rejected` | `allocation_rejected` | `allocation_rejected` | `allocation_not_found` |
| `DenyRelayDataPermissionAbsent` | `allocation=allocation_active; permission=permission_absent` | `allocation_active`, `permission_absent` | `allocation_active`, `permission_absent` | `permission_not_found` |
| `DenyRelayDataPermissionRequested` | `allocation=allocation_active; permission=permission_requested` | `allocation_active`, `permission_requested` | `allocation_active`, `permission_requested` | `permission_not_found` |
| `DenyRelayDataPermissionExpired` | `allocation=allocation_active; permission=permission_expired` | `allocation_active`, `permission_expired` | `allocation_active`, `permission_expired` | `permission_not_found` |
| `DenyRelayDataPermissionRevoked` | `allocation=allocation_active; permission=permission_revoked` | `allocation_active`, `permission_revoked` | `allocation_active`, `permission_revoked` | `permission_not_found` |
| `DenyRelayDataPermissionRejected` | `allocation=allocation_active; permission=permission_rejected` | `allocation_active`, `permission_rejected` | `allocation_active`, `permission_rejected` | `permission_not_found` |

### 3.6 Expiry 規則

- expiry decision は core semantics、timer execution は driver implementation です。
- driver は port boundary を通じて core-owned time observations を提供しなければなりません（必須）。
- allocation lifetime、permission lifetime、channel bind lifetime、refresh count、cumulative refresh duration は bound を持たなければなりません（必須）。
- refresh は absolute lifetime cap を延長してはなりません（禁止）。
- `permission_lifetime_exceeded` と `channel_bind_lifetime_exceeded` は expiry outcome であり、permission/channel bind rejection outcome ではありません。
- activation 前の invalid requested lifetime は `turn_lifetime_violation` を使用します。
- `allocation_refreshing` に入る前の refresh lifetime violation は `allocation_active` を保存し、`expired` outcome を使用してはなりません。
- refresh credential rejection は `allocation_active` を保存します。
- `allocation_lifetime_exceeded` は allocation expiry outcome で `ExpireAllocationLifetime` を使用します。
- `refresh_limit_exceeded` は refresh cap expiry outcome で `ExpireRefreshCap` を使用します。

### 3.7 Refresh Audit 規則

- TURN refresh decision は `turn_refresh_decision` を使用し、initial `turn_allocation_decision` を使用しません。
- `Refresh` success は `allocation_refreshing` に入るのみで、pending transition であり final `turn_refresh_decision` outcome `accepted` を emit してはなりません。
- `AcceptRefresh` は outcome `accepted` で final `turn_refresh_decision` を emit します。
- `Refresh` immediate failure、`RejectRefresh`、`RejectRefreshLifetimeViolation`、非 active refresh rejection 行は outcome `rejected` で `turn_refresh_decision` を emit します。
- refresh cap expiry は outcome `expired` と reason `refresh_limit_exceeded` で `turn_refresh_decision` を使用します。
- allocation absolute lifetime expiry は outcome `expired` と reason `allocation_lifetime_exceeded` で `turn_allocation_decision` を使用します。

### 3.8 Relay Queue Bound 規則

- `turn_relay_decision` は relay authorization のみを所有します。
- TURN relay queue capacity または wait limit failure は scheduling resource-bound drop であり、relay authorization denial ではありません。
- `turn_relay_queue_bound_exceeded` は outcome `dropped`、resource policy owner `core`、physical resource owner `driver`、active `AllocationId`、active `PermissionId` で `resource_bound_decision` として audit しなければなりません（必須）。
- 非 active allocation または permission に対する RelayData は outcome `denied` で `turn_relay_decision` を emit し、現在の非 active state を保存し、`allocation_not_found` または `permission_not_found` を使用します。
- RelayData は allocation/permission expiry reason を使用してはなりません。time-based expiry は専用の expiry transition を使用します。

### 3.9 Release 規則

- `allocation_active` または `allocation_refreshing` からの `Release` は reasonless success であり audit outcome `released` を使用します。
- active/refreshing allocation のない release request は `allocation_not_found` で reject され、現在の非 active allocation state を保存し、audit outcome `released` を使用してはなりません（必須）。

### 3.10 Allocation Precondition 規則

- `allocation_active` または `allocation_refreshing` を要求する TURN command は、absent/requested/expired/released/rejected allocation state を `allocation_not_found` で reject しなければなりません（必須）。
- TURN pending accept transition は active/bound state へ移る前に parent allocation/permission state を revalidate しなければなりません（必須）。
- `permission_active` を要求する TURN command は、absent/requested/expired/revoked/rejected permission state を `permission_not_found` で reject しなければなりません（必須）。
- Signaling participant または SFU endpoint への cross-plane relation は TURN lifecycle preconditions を override してはなりません（禁止）。

### 3.11 Credential 規則

- arcRTC は externally issued credential を verify します。
- arcRTC は credential を issue しません（必須）。

### 3.12 collapse 条件（TURN lifecycle）

- socket loop が allocation state を所有する。
- driver が permission decision を所有する。
- timer implementation が expiry semantics を定義する。
- TURN credential issuance が core responsibility になる。
- channel bind state が TCP framing state として扱われる。
- driver shutdown が core/audit relation なしに successful TURN lifecycle transition として扱われる。
- concurrent TURN allocation/permission conflict が driver socket order で解決される。
- cross-plane relation が TURN allocation/permission/channel bind lifecycle precondition を silent に bypass する。
