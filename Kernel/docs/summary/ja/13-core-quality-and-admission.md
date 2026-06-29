# core-quality-and-admission

状態: SSOT 統合版
日付: 2026-06-28 JST

## 目的

本章は arcRTC v0.2 Kernel の core が所有する quality metrics model と quality decision、および rate limit / quota / admission の identity-neutral resource policy 境界の現行完全仕様を、本章のみで再現実装可能な粒度で内在化することを目的とします。

依存方向の表記: `A <- B` は「B が A に依存」を意味します。core は外部 I/O 非依存です。quality は communication infrastructure の状態判断であり、regulated workflow priority ではありません。admission は active room、participant、endpoint、allocation、connection 等の bound decision であり、tenant/user/application identity を generic core に混入させてはなりません。core は metric model、threshold、decision、closed reason、admission policy semantics を所有し、driver は measurement source、exporter、runtime observation、external metric format、enforcement execution を所有します。本章は全 quality decision と admission decision、閉集合 reason、fail-closed 条件を内在化します。

---

## 1. Quality Metrics Model / Quality Decision

### 1.1 境界（owner）

core は metric model、threshold、decision、closed reason を所有します。drivers は measurement source、exporter、runtime observation、external metric format を所有します。Unit、measurement、time window、precision、comparison normalization は unit normalization 規則（第 11 章 6 節）に従います。exported metrics の observability signal taxonomy は observability signal taxonomy 規則に従います。

### 1.2 Core Metrics（core 所有、基本集合）

core が扱う metric は次を基本集合とします。RTT、packet loss、jitter、bitrate、media-facing contract が要求する場合の frame rate、relay latency、queue depth、backpressure state、route health、core formula が明示定義する場合の MOS-like score。

### 1.3 Core Decisions（core 所有）

core は次を判断します。quality normal、quality degraded、quality violation、route suppression、backpressure action、recovery allowed、admission rejected by quality policy。

### 1.4 Driver Responsibilities（driver 所有）

driver は次を所有します。raw measurement collection、platform-specific statistics source、str0m / RTP / RTCP concrete stats conversion、metrics exporter、dashboard / log format。Raw driver statistics は core quality policy evaluation 前に normalized unit へ converted されなければなりません（必須）。

### 1.5 Prohibited Semantics

quality core に次を含めてはなりません（禁止）。medical severity、patient priority、business SLA name、UI alert copy、platform-specific metric object、core contract としての exporter-specific label schema、raw platform measurement unit または hidden sampling window、unbounded または sensitive observability metric label。

### 1.6 Closed Reason Requirement（閉集合）

quality decision reason は閉集合にします。free-text only reason は core decision に使いません（禁止）。reason code は core reason catalog の code でなければならず、category-only reason value は禁止です（禁止）。

| Decision | reason code |
|---|---|
| packet forwarding suppressed by quality | `packet_suppressed_by_quality` |
| route suppressed by quality policy | `route_suppressed_by_quality` |
| route recovery rejected by quality policy | `quality_recovery_not_allowed` |
| endpoint recovery rejected by quality policy | `quality_recovery_not_allowed` |
| endpoint admission rejected by quality policy | `endpoint_quality_not_allowed` |
| admitted endpoint degraded by quality policy | `endpoint_degraded_by_quality` |
| publication admission or suppression by quality policy | `publication_quality_not_allowed` |
| subscription admission or suppression by quality policy | `subscription_quality_not_allowed` |

### 1.7 Audit Mapping 規則（閉集合）

quality decision audit mapping は次に固定します。SFU forwarding audit が同じ correlation ID で出る場合でも、quality violation audit の代替にしてはなりません（禁止）。quality target reference は `EndpointId`、`RouteId`、`StreamId`、`PacketId` に閉じます。packet forwarding suppressed by quality は packet identity が materialize されている場合 `PacketId` を carry しなければなりません（必須）。

| quality decision | audit event type | decision outcome | reason code |
|---|---|---|---|
| packet forwarding suppressed by quality | `quality_violation_decision` | `suppressed` | `packet_suppressed_by_quality` |
| route suppressed by quality policy | `quality_violation_decision` | `suppressed` | `route_suppressed_by_quality` |
| route recovery rejected by quality policy | `quality_violation_decision` | `rejected` | `quality_recovery_not_allowed` |
| endpoint recovery rejected by quality policy | `quality_violation_decision` | `rejected` | `quality_recovery_not_allowed` |
| endpoint admission rejected by quality policy | `quality_violation_decision` | `rejected` | `endpoint_quality_not_allowed` |
| admitted endpoint degraded by quality policy | `quality_violation_decision` | `degraded` | `endpoint_degraded_by_quality` |
| publication admission rejected by quality policy | `quality_violation_decision` | `rejected` | `publication_quality_not_allowed` |
| publication suppressed by quality policy | `quality_violation_decision` | `suppressed` | `publication_quality_not_allowed` |
| subscription admission rejected by quality policy | `quality_violation_decision` | `rejected` | `subscription_quality_not_allowed` |
| subscription suppressed by quality policy | `quality_violation_decision` | `suppressed` | `subscription_quality_not_allowed` |

`quality_violation_decision` の allowed outcome は `rejected`、`suppressed`、`degraded` です（第 12 章 3.9 節と一致）。

### 1.8 Collapse 条件（quality）

metrics exporter が quality decision の正を持つ。platform-specific stats object が core type になる。regulated priority が generic quality decision に混入する。threshold の owner が core 以外になる。quality decision が unnormalized unit または hidden measurement window で比較する。observability exporter label/sampling が quality decision semantics を変える。

---

## 2. Rate Limit / Quota / Admission

### 2.1 境界（owner）

本節は tenant/user/application identity を generic core に混入させず、admission policy を fail-closed にする粒度を固定します。bounded resource と reason は resource bounds / backpressure 規則（第 11 章 8 節）が定義します。本節は具体的な rate 数値、quota 値、multi-tenant billing policy、production enforcement を主張しません。admission policy input として使う authorization context mapping は authorization context 規則（第 12 章 2 節）に従い、admission input として使う edge/proxy source metadata は edge/proxy trust 規則に従います。

| 関心事 | 所有者 | 規則 |
|---|---|---|
| admission policy semantics | core | active room、participant、endpoint、allocation、connection 等の bound decision |
| quota/rate policy input | core typed policy | external tenant/user semantics を直接読まない |
| external identity/auth context mapping | core policy input 前の entrypoints/drivers | opaque reference または accepted policy scope へ変換 |
| edge/proxy source metadata mapping | core policy input 前の entrypoints/drivers | admitted trust policy が必須 |
| raw measurement | driver | connection count、queue depth、time window observation |
| enforcement execution | driver | connection close、queue reject、packet drop、send suppression |
| audit evidence | core event model and driver sink | owner tuple 付き resource-bound decision |

core は application tenant、billing account、user profile、regulated role を generic protocol identity として所有しません（禁止）。必要な grouping は core 所有 opaque admission scope reference と typed policy input で表します。

### 2.2 Admission Scope 規則（閉集合）

admission scope は次のいずれかの core 所有 reference に限定します。

| Scope | 意味 | 例となる bound |
|---|---|---|
| `global` | process/runtime wide policy scope | connection concurrency |
| `room` | room-scoped policy | room participant set |
| `session` | SFU session-scoped policy | endpoint admission |
| `allocation` | TURN allocation-scoped policy | permission count |
| `credential_ref` | opaque credential/key verification result scope | 明示 configure 時の credential-bound admission |
| `driver_connection_ref` | pre-core connection scope | frame size と connection concurrency |

Tenant/user/entrypoint role は v0.2 initial architecture の built-in admission scope ではありません。そのような scope の追加は generic communication boundary を保存する仕様の更新を要します。authorization context は authorization 規則を通してのみ external auth material を admitted opaque scope に map してよい。Forwarded header、client IP、host、origin、SNI は default で admission scope ではありません。

### 2.3 Required Admission Decisions（閉集合）

| admission target | 所有者 | failure reason |
|---|---|---|
| room materialization / active room | core decision, driver execution | `room_capacity_exceeded` |
| participant join | core decision, driver execution | `admission_capacity_exceeded` |
| signaling command queue admission | core policy, driver queue execution | `signaling_command_queue_bound_exceeded` |
| SFU endpoint admission | core decision, driver execution | `endpoint_capacity_exceeded` |
| SFU route candidate construction | core routing decision | `route_candidate_bound_exceeded` |
| TURN allocation | core decision, driver execution | `allocation_capacity_exceeded` |
| TURN permission | core decision, driver execution | `permission_capacity_exceeded` |
| connection concurrency | core policy, driver execution | `connection_concurrency_exceeded` |
| inbound frame | driver-local pre-core bound | `frame_size_bound_exceeded` |

Required decision は resource bounds / backpressure 規則（第 11 章 8 節）の required bound table と owner tuple を再利用します。

### 2.4 Rate Window 規則

Rate limit window は policy であり storage implementation ではありません。Core は accepted typed policy shape と decision semantics を所有します。Driver は time/window counter を measure してよいが、decision が core 所有である箇所では observation を core 所有 input type を通して提示しなければなりません（必須）。

許可される window attribute: scope、maximum count、適用時 maximum bytes、maximum duration/window、reset semantics、reason code、audit event type、owner tuple。Window counter は bounded でなければならず（必須）、persistence/restore 仕様が明示許可しない限り durable domain state になってはなりません（禁止）。

### 2.5 Fail-Closed 規則

core 所有 admission decision に必要な admission measurement、quota lookup、rate window state が unavailable のとき、影響を受けた admission path は cataloged reason で reject または stop しなければなりません（必須、fail-closed）。default で accept してはなりません（禁止）。

| Failure | 必須 reason |
|---|---|
| bounded admission capacity reached | target-specific resource reason |
| driver failure により driver measurement を取得できない | `driver_shutdown` または applicable driver failure reason |
| runtime quota configuration missing | `runtime_config_missing` |
| runtime quota configuration invalid | `runtime_config_invalid` |
| 必要時 persistence-backed quota source unavailable | `persistence_unavailable` |
| required authorization context missing | `authorization_context_missing` |
| authorization scope not allowed | `authorization_scope_not_allowed` |
| proxy/header/client address を admission に信頼不可 | `client_address_untrusted` または `forwarded_header_untrusted` |
| frame size exceeded before core entry | `frame_size_bound_exceeded` |

### 2.6 Audit 規則

全 rejected/dropped/shed/expired admission または quota decision は audit event model（第 12 章 3 節）に接続しなければなりません（必須）。Resource-bound related event は resource policy owner と physical resource owner を carry しなければなりません（必須）。audit sink が unavailable で、必要箇所で decision を record する bootstrap/overflow path が無い場合、影響を受けた accepted path は closeout evidence として使用してはなりません（fail-closed）。

### 2.7 禁止 / Collapse 条件（admission）

禁止: tenant/user/billing identity が仕様の更新なしに generic core protocol identity になる。driver が required quota/rate state unavailable 後に admission を accept する。unbounded rate counter または quota cache。rate limit rejection が free-text reason を使う。SDK platform wrapper が server admission reason を変える。entrypoint config が required admission bound を silent に無効化する。authorization context が identity-neutral admission scope を bypass する。raw proxy metadata が edge trust policy なしに admission scope になる。

Collapse: quota/rate state unavailable 時に admission が fail open し得る。generic core 内で identity scope が regulated/entrypoint user model から推論される。rate/quota counter が unbounded または unaudited。resource-bound audit owner tuple が欠落。concrete rate value が evidence なしに production policy として主張される。external auth role が仕様の更新なしに admission scope になる。client IP または forwarded header が trusted metadata admission なしに quota identity として使用される。
