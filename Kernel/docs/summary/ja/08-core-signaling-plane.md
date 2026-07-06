# core-signaling-plane

状態: public summary projection
日付: 2026-07-06 JST

## 目的

本章は arcRTC v0.2 Kernel の `core/signaling` が所有する Signaling contract と Signaling state machine の現行完全仕様を、本章のみで再現実装可能な粒度で内在化することを目的とします。Signaling は SDK と server が共有する protocol contract であり、network framework の都合で定義されません。本章は room state、command/event semantics、accept/reject boundary、全 state と全遷移、guard、reject 条件、idempotency/ordering/correlation 規則、fail-closed 条件、ならびに driver/entrypoint/SDK との境界を内在化します。

依存方向の表記: `A <- B` は「B が A に依存」を意味します。Signaling の pure semantics は core が所有し、I/O（WebSocket/HTTP/encoding/runtime）と分離されます。v0.2 Kernel は Signaling product system を所有せず、reference distro / product distro は Kernel 外 distro に置きます。

---

## 1. Signaling Contract

### 1.1 境界（owner）

| 領域 | 所有者 | 規則 |
|---|---|---|
| Signaling semantics | `core/signaling` | room identity、participant transport identity、command/event schema 意味論、state transition、accept/reject、correlation、ordering/idempotency policy、TURN credential request/delivery abstract contract、protocol violation fail-closed rule |
| transport と encoding | `drivers/network` | WebSocket frame、HTTP request/response、external JSON/binary encoding、network timeout、connection close mapping、external error code mapping、driver-local logging/metrics |
| executable contract / composition evidence surface | `entrypoints/signaling-server` | binary entrypoint、configuration loading、dependency wiring、driver selection、process lifecycle |
| Signaling-only client boundary | `sdk/*` | 利用者へ Signaling-only contract を公開 |

command idempotency / replay / correlation、authorization context / communication policy、cross-plane identity / session binding、SDK public API projection、ICE candidate / connectivity lifecycle、out-of-scope feature admission は、それぞれの規則体系に従います（本章は Signaling contract 内在分のみを規定）。

### 1.2 Identity 規則

- Signaling は user identity を所有しません（必須）。
- client-provided ID を trusted identity として扱ってはなりません（禁止）。
- 外部露出する participant identity は server-generated opaque transport ID とします（必須）。
- Signaling join success は、admitted cross-plane binding と target-plane decision なしに SFU endpoint admission、TURN allocation、secure media authorization にはなりません（必須）。

### 1.3 Command Set（閉集合、public）

v0.2 initial Signaling contract の public command set は次に限定します。追加する場合は protocol versioning 規則に従い、compatible capability または new contract version として明示しなければなりません。

| Command | Core responsibility |
|---|---|
| `JoinRoom` | room membership request、accept/reject |
| `LeaveRoom` | membership termination |
| `SendOffer` | SDP offer relay intent |
| `SendAnswer` | SDP answer relay intent |
| `SendIceCandidate` | ICE candidate relay intent |
| `RequestTurnCredential` | TURN credential delivery request boundary |
| `AcknowledgeForward` | accepted forwarding acknowledgement |

### 1.4 Event Set（閉集合、public）

v0.2 initial Signaling contract の public event set は次に限定します。追加する場合は protocol versioning 規則に従い、compatible capability または new contract version として明示しなければなりません。

| Event | Core responsibility |
|---|---|
| `Joined` | accepted room membership |
| `Rejected` | closed reason を伴う fail-closed rejection |
| `ParticipantJoined` | room-visible participant lifecycle |
| `ParticipantLeft` | room-visible participant lifecycle |
| `OfferReceived` | offer relay event |
| `AnswerReceived` | answer relay event |
| `IceCandidateReceived` | ICE relay event |
| `TurnCredentialAvailable` | credential delivery event |
| `ProtocolViolation` | closed violation classification |

### 1.5 Fail-Closed 規則（reason code 閉集合）

次の場合は reject / violation として扱い、cataloged reason code に接続しなければなりません（必須）。

| Failure | Reason code |
|---|---|
| missing correlation ID | `missing_correlation_id` |
| malformed command | `malformed_command` |
| unauthorized token verification result | `token_verification_failed` |
| room policy violation | `room_not_accepting_join` |
| room materialization または active room capacity 超過 | `room_capacity_exceeded` |
| room participant admission capacity 超過 | `admission_capacity_exceeded` |
| room lifecycle duration 超過 | `room_lifetime_exceeded` |
| room が draining で新規 join/command を reject | `room_draining` |
| room が既に closed | `room_closed` |
| room close/drain transition が invalid | `room_close_not_allowed` |
| invalid participant state transition | `participant_not_joined` |
| participant verification または join policy が membership を reject | `participant_rejected` |
| idempotency rule 外の duplicate command | `duplicate_command` |
| idempotency replay conflict | `idempotency_payload_mismatch` |
| replay window 期限切れ | `idempotency_window_expired` |
| required authorization context が missing/invalid | `authorization_context_missing` または `authorization_context_invalid` |
| authorization policy が要求 communication action を deny | `authorization_policy_denied` |
| ordering violation | `command_order_violation` |
| signaling command queue capacity または wait limit 超過 | `signaling_command_queue_bound_exceeded` |
| unsupported command version | `unsupported_command_version` |
| cross-plane binding が required だが absent/invalid | cross-plane binding reason |

### 1.6 External Encoding 規則

- JSON、MessagePack、binary frame、WebSocket close code、HTTP status は driver 境界で扱います（必須）。
- core は encoded payload ではなく core-owned command / event type を扱います（必須）。
- `token_verification_failed` は Signaling public wrapper reason です。concrete token verification failure は `token_verification_decision` audit event に保存しなければなりません（必須）。

### 1.7 禁止 semantics（Signaling contract に含めてはならない）

medical role、application workflow state、chat semantics、recording semantics、screen share semantics、DataChannel application semantics、UI / end-user workflow、user authentication issuance、regulated payload。

### 1.8 collapse 条件（Signaling contract）

- WebSocket handler が Signaling state transition の正を持つ。
- SDK と server が別々の Signaling command semantics を持つ。
- Signaling payload が domain identity に依存する。
- protocol violation が open-ended string だけで扱われる。
- Signaling duplicate/replay behavior が SDK や network driver で定義される。
- token verification success が authorization context policy なしに Signaling authorization success として扱われる。
- Signaling participant success が cross-plane binding なしに SFU/TURN/secure media success として扱われる。
- out-of-scope feature が仕様 admission なしに Signaling command/event set に追加される。

---

## 2. Signaling Boundary（plane 分離決定）

### 2.1 Decision

Signaling の中核意味論は `core/signaling` が所有します。WebSocket、HTTP、Axum、tokio task、socket、request parsing、response encoding は drivers または entrypoints が所有します（必須）。

### 2.2 Core Signaling Responsibilities

`core/signaling` は次を所有します（必須）。

- room identity の扱い
- participant transport identity の扱い
- command / event schema の意味論
- join / leave / offer / answer / ice candidate / forward accepted の state transition
- accept / reject boundary
- correlation ID requirement
- ordering / idempotency / duplicate handling の policy
- TURN credential request / delivery の abstract contract
- protocol violation の fail-closed rule

### 2.3 Driver Responsibilities

Signaling driver は次を所有します（必須）。WebSocket frame 受信、HTTP request/response、external JSON/binary encoding、network timeout、connection close mapping、external error code mapping、driver-local logging/metrics export。

### 2.4 Entrypoint Responsibilities

Signaling entrypoint は次を所有します（必須）。binary entrypoint、configuration loading、dependency wiring、driver selection、process lifecycle。

### 2.5 SDK Relation

SDK は Signaling-only contract を利用者に公開します。SDK は browser/native PeerConnection、media capture、screen share、recording、regulated support を所有してはなりません（禁止）。

### 2.6 Prohibited Placement

- WebSocket handler 内に room state transition の正を置く。
- entrypoints に join accept / reject の rule を置く。
- SDK に server-side authorization decision を置く。
- Signaling message に entrypoint-specific user role / medical role / domain meaning を持たせる。
- client-provided participant ID を trusted identity として扱う。

### 2.7 collapse 条件（boundary）

- `core/signaling` 以外が Signaling state transition の正を持つ。
- WebSocket / HTTP 型が core に入る。
- SDK が media / regulated / auth issuance を所有する。
- Signaling が opaque transport ID ではなく domain identity に依存する。

---

## 3. Signaling State Machine

Signaling state transition の正は `core/signaling` が所有します。

### 3.1 State Owners

| State family | Owner | Driver role |
|---|---|---|
| Room state | core | external event conversion |
| Participant state | core | connection observation |
| Command idempotency state | core | core-owned idempotency semantics 用の optional storage implementation |
| Wire connection state | driver | 関連時は core event へ変換 |
| Process lifecycle | entrypoints | protocol semantics なし |

process lifecycle は、以下の遷移なしに entrypoints/driver が Signaling state を mutate することを authorize しません（必須）。

### 3.2 Room States（閉集合）

| State | 意味 |
|---|---|
| `room_absent` | room が存在しない、または未 materialize |
| `room_open` | room が join と relay command を accept |
| `room_draining` | room が新規 join を reject し、controlled leave/close を許可 |
| `room_closed` | room が idempotent close observation 以外の全 protocol command を reject |

### 3.3 Participant States（閉集合）

| State | 意味 |
|---|---|
| `participant_new` | transport は observed だが未 accept |
| `participant_verifying` | token/policy verification 進行中 |
| `participant_joined` | participant が room に accepted |
| `participant_leaving` | leave が accepted で cleanup 進行中 |
| `participant_left` | membership 終了 |
| `participant_rejected` | join または command が reject |

### 3.4 Command Transition Rules（全遷移表）

pre-state tuple 表記:

- `family={a, b}` は同 family のいずれか 1 state が必須を意味します。
- `family=x; other_family=y` は両 family state が同時に必須を意味します。
- 省略された family はその family の precondition がないことを意味します。

| Command / core event | Allowed pre-state | Success state | Reject reason code |
|---|---|---|---|
| `JoinRoom` | `room={room_absent, room_open}; participant=participant_new` | `participant_verifying` | `room_not_accepting_join`, `room_capacity_exceeded`, `admission_capacity_exceeded`, `room_lifetime_exceeded`, `token_verification_failed`, `room_draining`, `room_closed` |
| `AcceptJoinVerification` | `room={room_absent, room_open}; participant=participant_verifying` | `room_open`, `participant_joined` | `room_not_accepting_join`, `room_capacity_exceeded`, `admission_capacity_exceeded`, `room_lifetime_exceeded`, `room_draining`, `room_closed` |
| `RejectJoinVerification` | `participant=participant_verifying` | `participant_rejected` | `token_verification_failed`, `participant_rejected` |
| `LeaveRoom` | `room={room_open, room_draining}; participant=participant_joined` | `participant_leaving` | `participant_not_joined`, `room_closed` |
| `FinishLeave` | `room={room_open, room_draining}; participant=participant_leaving` | `participant_left` | `participant_not_joined`, `room_closed` |
| `BeginRoomDrain` | `room=room_open` | `room_draining` | `room_close_not_allowed` |
| `CloseRoom` | `room={room_open, room_draining, room_closed}` | `room_closed` | `room_close_not_allowed` |
| `ObserveClosedRoom` | `room=room_closed` | `room_closed` | `room_close_not_allowed` |
| `SendOffer` | `room=room_open; participant=participant_joined` | `room_open`, `participant_joined` | `participant_not_joined`, `command_order_violation`, `room_draining`, `room_closed` |
| `SendAnswer` | `room=room_open; participant=participant_joined` | `room_open`, `participant_joined` | `participant_not_joined`, `command_order_violation`, `room_draining`, `room_closed` |
| `SendIceCandidate` | `room=room_open; participant=participant_joined` | `room_open`, `participant_joined` | `participant_not_joined`, `command_order_violation`, `room_draining`, `room_closed` |
| `RequestTurnCredential` | `room=room_open; participant=participant_joined` | `room_open`, `participant_joined` | `participant_not_joined`, `token_verification_failed`, `room_draining`, `room_closed` |
| `AcknowledgeForward` | `room=room_open; participant=participant_joined` | `room_open`, `participant_joined` | `participant_not_joined`, `duplicate_command`, `room_draining`, `room_closed` |

### 3.5 暗黙遷移規則（draining / closed の網羅）

- 明示的な `room=room_closed` allowed pre-state を持たない全 command は、`room_closed` で `room_closed` により reject されます。
- 明示的な `room=room_draining` allowed pre-state を持たない command は、`room_draining` で `room_draining` により reject されます。
- `room_draining` での新規 join は `room_draining` で reject され、controlled leave と close transition は許可されます。
- `room_closed` での `CloseRoom` と `ObserveClosedRoom` は idempotent success であり、rejection ではありません。

### 3.6 Idempotency 規則

- idempotency は core semantics です。
- duplicate command handling は closed result を生成しなければなりません: 同一効果の accepted duplicate、または rejected duplicate。
- driver は idempotency semantics を決定してはなりません（禁止）。

### 3.7 Ordering 規則

- ordering rule は core semantics です。
- driver は wire order を保持/observe してよいが、command ordering validity は core に属します（必須）。

### 3.8 Correlation 規則

- driver/core 境界を跨ぐ全 command は `CorrelationId` を carry しなければなりません（必須）。
- missing correlation ID は driver conversion 境界で `missing_correlation_id` により fail-close され、command は state machine に入りません（必須）。

### 3.9 collapse 条件（state machine）

- WebSocket connection state が participant state として扱われる。
- driver が join / leave / relay state transition を決定する。
- duplicate command handling が driver-local になる。
- missing correlation ID が許容される。
- SDK が別の Signaling state machine を定義する。
- process shutdown が core transition なしに Signaling state authority として扱われる。
- concurrent Signaling command conflict が core ordering/idempotency rule の外で解決される。
