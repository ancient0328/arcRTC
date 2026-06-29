# core-sfu-plane

状態: SSOT 統合版
日付: 2026-06-28 JST

## 目的

本章は arcRTC v0.2 Kernel の `core/sfu` が所有する SFU contract、SFU state machine、packet semantic view の field boundary、congestion/pacing/retransmission の policy/execution 境界の現行完全仕様を、本章のみで再現実装可能な粒度で内在化することを目的とします。本章は routing / quality decision / backpressure semantics を core 所有とし、RTP/RTCP byte I/O、str0m、socket、runtime worker、metrics exporter などの execution を driver 所有として分離します。packet bytes の所有権・寿命・copy 許可（packet buffer lifecycle 全状態）は第07章 §7 が定義します。本章は packet semantic view の field boundary と SFU 固有の routing/state/congestion semantics を内在化します。

依存方向の表記: `A <- B` は「B が A に依存」を意味します。SFU の pure semantics は core が所有し、I/O と分離されます。v0.2 Kernel は SFU product system を所有せず、reference implementation / product implementation は Kernel 外 implementations に置きます。

---

## 1. SFU Contract

### 1.1 境界（owner）

| 領域 | 所有者 | 規則 |
|---|---|---|
| routing, forwarding intent, quality decision, backpressure policy | `core/sfu` | SFU domain semantics の正 |
| transport I/O, str0m, RTP byte parsing backend, runtime worker, metrics exporter | drivers | execution detail |
| executable contract / composition evidence surface と wiring | entrypoints | server executable / dependency wiring / configuration / driver selection / process lifecycle |

RTP/RTCP packet bytes の所有権、borrowed view、cache、queue、copy 許可条件は packet buffer lifecycle（第07章 §7）に従います。congestion/pacing/retransmission の policy/execution 境界は本章 §4 に従います。media codec/track/layer negotiation は第07章 §4 に従います。secure media session lifecycle は第07章 §8 に従います。cross-plane identity / session binding、out-of-scope feature admission はそれぞれの規則体系に従います。

### 1.2 Core Model（core 所有 model、閉集合）

core は次を model として所有します（必須）。

- SFU session
- participant endpoint
- media stream
- accepted negotiation 後の media codec/track/layer reference
- publication
- subscription
- forwarding intent
- route candidate
- borrowed packet abstract view
- core-owned type としての quality observation
- backpressure state
- admission decision
- rejection reason

### 1.3 Core Decisions（core 所有 decision、閉集合）

core は次の decision を所有します（必須）。

- participant admission / rejection
- publication accepted / rejected
- subscription accepted / rejected
- route selected / not selected
- forwarding allowed / suppressed
- degradation / recovery decision
- backpressure action
- quality violation classification

### 1.4 Driver Responsibilities

driver は次を所有します（必須）。RTP/RTCP byte I/O、raw packet bytes、buffer lease、packet cache、transmit queue、pacing queue、retransmission cache、str0m event conversion、UDP/TCP/socket I/O、runtime worker execution、SIMD backend、concrete metrics exporter、external logging sink、retain される場合の legacy interop conversion。

### 1.5 禁止 semantics（SFU core に含めてはならない）

medical workflow priority、application-specific room policy、UI state、recording policy、chat semantics、screen share workflow、DataChannel application semantics、UI / end-user workflow、regulated data classification、concrete worker thread strategy、packet bytes ownership、packet cache、transmit queue、codec implementation / transcoding backend、SFU reference implementation / product implementation。

### 1.6 Fail-Closed 規則（decision handling と reason code 閉集合）

次の場合は handling 列に従って fail-closed とし、cataloged reason code に接続しなければなりません（必須）。

| Failure | Decision handling | Reason code |
|---|---|---|
| unknown participant | rejected | `participant_not_admitted` |
| SFU session が decision を accept しない | rejected | `sfu_session_not_accepting` |
| endpoint admission capacity 超過 | rejected | `endpoint_capacity_exceeded` |
| quality policy が endpoint admission を reject | rejected | `endpoint_quality_not_allowed` |
| quality policy が endpoint を degrade | degraded | `endpoint_degraded_by_quality` |
| quality policy が endpoint/route recovery を reject | rejected | `quality_recovery_not_allowed` |
| endpoint または route target 利用不可 | lifecycle/routing selection は rejected、forwarding execution は failed | `target_unavailable` |
| invalid stream identity | rejected | `stream_not_found` |
| unauthorized publication | rejected | `publication_not_allowed` |
| quality policy が publication を reject/suppress | decision type により rejected/suppressed | `publication_quality_not_allowed` |
| unauthorized subscription | rejected | `subscription_not_allowed` |
| quality policy が subscription を reject/suppress | decision type により rejected/suppressed | `subscription_quality_not_allowed` |
| route conflict | route construction/selection は rejected、packet forwarding は suppressed | `route_conflict` |
| route candidate bound 超過 | rejected | `route_candidate_bound_exceeded` |
| quality policy が route を suppress | suppressed | `route_suppressed_by_quality` |
| backpressure policy が route を suppress | suppressed | `route_suppressed_by_backpressure` |
| quality policy violation | suppressed | `packet_suppressed_by_quality` |
| backpressure threshold violation | suppressed | `packet_suppressed_by_backpressure` |
| backpressure policy が packet を drop | dropped | `packet_dropped_by_backpressure` |
| backpressure policy が subscription を suppress | suppressed | `subscription_backpressure_suppressed` |
| backpressure policy が action を delay | delayed | `action_delayed_by_backpressure` |
| backpressure policy が route を degrade | degraded | `route_degraded_by_backpressure` |
| backpressure policy が endpoint を close | closed_by_policy | `endpoint_closed_by_backpressure` |
| backpressure-delayed/degraded/suppressed route state からの recovery が reject | rejected | `backpressure_recovery_not_allowed` |
| SFU transmit queue capacity または wait limit 超過 | dropped | `sfu_transmit_queue_bound_exceeded` |
| unsupported media contract version | rejected | `unsupported_media_contract_version` |
| unsupported codec/profile | rejected | `media_codec_not_supported` |
| media policy が track publication/subscription を不許可 | rejected | `media_track_not_allowed` |
| 要求 media layer 利用不可 | rejected または suppressed | `media_layer_not_available` |
| invalid payload/SSRC/RID/MID mapping | rejected | `media_payload_mapping_invalid` |

SFU endpoint admission と route candidate construction の rate limit/quota/admission decision は rate limit/quota/admission 規則に従います。SFU endpoint admission/publication/subscription/route/forwarding が Signaling participant/session relation に依存する場合、claim が plane を跨ぐ前に cross-plane binding evidence が存在しなければなりません（必須）。

### 1.7 collapse 条件（SFU contract）

- worker/runtime layer が routing decision の正を持つ。
- metrics exporter が quality decision の正を持つ。
- core が str0m / tokio / socket concrete type に依存する。
- core が packet bytes、buffer lease、packet cache、transmit queue を所有する。
- core が pacing queue または retransmission cache を所有する。
- regulated-specific priority が generic SFU core に入る。
- codec implementation or payload parser が SFU routing authority になる。
- out-of-scope feature が SFU routing/forwarding authority になる。
- Signaling participant、TURN allocation、ICE candidate、secure media observation が cross-plane binding なしに SFU endpoint/route authority として扱われる。

---

## 2. SFU Boundary（plane 分離決定）

### 2.1 Decision

SFU の中核意味論は `core/sfu` が所有します。RTP/RTCP packet I/O、str0m、UDP、tokio runtime、worker scheduling、metrics exporter、legacy bridge は drivers または entrypoints に置きます（必須）。

### 2.2 Core SFU Responsibilities

`core/sfu` は次を所有します（必須）。participant/endpoint model、media stream identity model、forwarding intent、routing decision、subscription/publication semantics、congestion/backpressure policy、quality threshold/degradation decision、admission/rejection semantics、fault classification、audit-relevant SFU event model。

### 2.3 Driver / Entrypoint Responsibilities

SFU driver: RTP/RTCP byte I/O、str0m event conversion、UDP/TCP/socket concrete transport、runtime worker execution、SIMD/CPU-specific packet processing、metrics exporter、tracing integration、retain される場合の legacy interop bridge。

SFU entrypoint: server executable / product-system wiring detail、dependency wiring、configuration loading、driver selection、process lifecycle。

### 2.4 Prohibited Placement / collapse 条件

禁止: worker loop に routing decision の正を置く。metrics exporter に quality decision の正を置く。legacy bridge を core の protocol model に混入する。core が str0m concrete type/tokio runtime/socket/metrics exporter に依存する。regulated-specific quality semantics を generic SFU core に入れる。

collapse: `server/sfu` を v0.2 に subtree copy する。routing/quality/backpressure decision が drivers/entrypoints に分散する。runtime tuning を core semantics と混同する。legacy bridge が current architecture の中心になる。

---

## 3. SFU State Machine

SFU state transition の正は `core/sfu` が所有します。

### 3.1 State Owners

| State family | Owner | Driver role |
|---|---|---|
| SFU session state | core | lifecycle observation |
| Endpoint state | core | transport observation conversion |
| Publication state | core | media source observation conversion |
| Subscription state | core | target readiness observation conversion |
| Route state | core | forwarding execution |
| Backpressure action state | core | delayed execution observation |
| Packet buffer lifecycle | driver | packet buffer lifecycle（第07章 §7）が governing |

process lifecycle または driver worker shutdown は、以下の遷移なしに SFU state を直接 mutate することを authorize しません（必須）。

### 3.2 Session States（閉集合）

| State | 意味 |
|---|---|
| `sfu_session_open` | endpoints と route decisions を admit |
| `sfu_session_draining` | 新規 admission を suppress し、既存 route を drain |
| `sfu_session_closed` | 全 新規 SFU decision を reject |

### 3.3 Endpoint States（閉集合）

| State | 意味 |
|---|---|
| `endpoint_observed` | driver observed endpoint |
| `endpoint_admission_pending` | admission decision 進行中 |
| `endpoint_admitted` | endpoint が publish/subscribe 可 |
| `endpoint_degraded` | degraded quality state で admitted を維持 |
| `endpoint_draining` | endpoint 退出中 |
| `endpoint_removed` | endpoint が routable でない |
| `endpoint_rejected` | endpoint admission rejected |

### 3.4 Publication States（閉集合）

| State | 意味 |
|---|---|
| `publication_absent` | active publication なし |
| `publication_requested` | publication 評価中 |
| `publication_active` | stream が routable |
| `publication_suppressed` | stream は存在するが routing suppressed |
| `publication_closed` | publication 終了 |
| `publication_rejected` | publication rejected |

### 3.5 Subscription States（閉集合）

| State | 意味 |
|---|---|
| `subscription_absent` | subscription なし |
| `subscription_requested` | subscription 評価中 |
| `subscription_active` | target が route を receive 可 |
| `subscription_suppressed` | target subscription は存在するが forwarding suppressed |
| `subscription_closed` | subscription 終了 |
| `subscription_rejected` | subscription rejected |

### 3.6 Route States（閉集合）

| State | 意味 |
|---|---|
| `route_candidate` | route が selectable |
| `route_selected` | route が core により selected |
| `route_delayed` | backpressure policy により route action が delayed |
| `route_suppressed_by_backpressure` | backpressure policy により route suppressed |
| `route_suppressed_by_quality` | quality policy により route suppressed |
| `route_degraded` | degraded form でのみ usable |
| `route_dropped` | route 使用不可 |
| `route_closed` | route 終了 |

### 3.7 Transition Rules（全遷移表）

pre-state tuple 表記:

- `family={a, b}` は同 family のいずれか 1 state が必須。
- `family=x; other_family=y` は両 family state が同時に必須。
- 省略された family は precondition なし。

| Event | Allowed pre-state | Success state | Decision reason code |
|---|---|---|---|
| `ObserveEndpoint` | `session=sfu_session_open` | `endpoint_observed` | `sfu_session_not_accepting` |
| `BeginEndpointAdmission` | `session=sfu_session_open; endpoint=endpoint_observed` | `endpoint_admission_pending` | `sfu_session_not_accepting`, `endpoint_capacity_exceeded` |
| `AdmitEndpoint` | `endpoint=endpoint_admission_pending` | `endpoint_admitted` | `participant_not_admitted`, `endpoint_capacity_exceeded`, `endpoint_quality_not_allowed` |
| `RejectEndpoint` | `endpoint=endpoint_admission_pending` | `endpoint_rejected` | `participant_not_admitted`, `endpoint_capacity_exceeded`, `endpoint_quality_not_allowed` |
| `DegradeEndpoint` | `endpoint=endpoint_admitted` | `endpoint_degraded` | `endpoint_degraded_by_quality` |
| `RecoverEndpoint` | `endpoint=endpoint_degraded` | `endpoint_admitted` | `quality_recovery_not_allowed` |
| `DrainEndpoint` | `session={sfu_session_open, sfu_session_draining}; endpoint={endpoint_admitted, endpoint_degraded}` | `endpoint_draining` | `sfu_session_not_accepting`, `target_unavailable` |
| `RemoveEndpoint` | `session={sfu_session_open, sfu_session_draining}; endpoint=endpoint_draining` | `endpoint_removed` | `sfu_session_not_accepting`, `target_unavailable` |
| `StartPublication` | `session=sfu_session_open; endpoint=endpoint_admitted; publication=publication_absent` | `publication_requested` | `sfu_session_not_accepting`, `publication_not_allowed`, `publication_quality_not_allowed` |
| `AcceptPublication` | `session=sfu_session_open; endpoint={endpoint_admitted, endpoint_degraded}; publication=publication_requested` | `publication_active` | `sfu_session_not_accepting`, `target_unavailable`, `publication_not_allowed`, `publication_quality_not_allowed` |
| `RejectPublication` | `publication=publication_requested` | `publication_rejected` | `publication_not_allowed`, `publication_quality_not_allowed` |
| `SuppressPublication` | `session={sfu_session_open, sfu_session_draining}; publication=publication_active` | `publication_suppressed` | `sfu_session_not_accepting`, `publication_quality_not_allowed` |
| `ClosePublication` | `session={sfu_session_open, sfu_session_draining}; publication={publication_active, publication_suppressed}` | `publication_closed` | `sfu_session_not_accepting`, `publication_not_allowed` |
| `StartSubscription` | `session=sfu_session_open; endpoint=endpoint_admitted; subscription=subscription_absent` | `subscription_requested` | `sfu_session_not_accepting`, `subscription_not_allowed`, `subscription_quality_not_allowed` |
| `AcceptSubscription` | `session=sfu_session_open; endpoint={endpoint_admitted, endpoint_degraded}; subscription=subscription_requested` | `subscription_active` | `sfu_session_not_accepting`, `target_unavailable`, `subscription_not_allowed`, `subscription_quality_not_allowed` |
| `RejectSubscription` | `subscription=subscription_requested` | `subscription_rejected` | `subscription_not_allowed`, `subscription_quality_not_allowed` |
| `SuppressSubscription` | `session={sfu_session_open, sfu_session_draining}; subscription=subscription_active` | `subscription_suppressed` | `sfu_session_not_accepting`, `subscription_backpressure_suppressed`, `subscription_quality_not_allowed` |
| `CloseSubscription` | `session={sfu_session_open, sfu_session_draining}; subscription={subscription_active, subscription_suppressed}` | `subscription_closed` | `sfu_session_not_accepting`, `subscription_not_allowed` |
| `BuildRouteCandidate` | `session=sfu_session_open; endpoint={endpoint_admitted, endpoint_degraded}; publication=publication_active; subscription=subscription_active` | `route_candidate` | `sfu_session_not_accepting`, `target_unavailable`, `route_conflict`, `stream_not_found`, `route_candidate_bound_exceeded` |
| `SelectRoute` | `session=sfu_session_open; endpoint={endpoint_admitted, endpoint_degraded}; publication=publication_active; subscription=subscription_active; route=route_candidate` | `route_selected` | `sfu_session_not_accepting`, `target_unavailable`, `route_conflict`, `stream_not_found`, `route_candidate_bound_exceeded` |
| `DelayActionByBackpressure` | `session={sfu_session_open, sfu_session_draining}; route={route_candidate, route_selected}` | `route_delayed` | `sfu_session_not_accepting`, `action_delayed_by_backpressure` |
| `ApplyBackpressure` | `session={sfu_session_open, sfu_session_draining}; route=route_selected` | `route_suppressed_by_backpressure` | `sfu_session_not_accepting`, `route_suppressed_by_backpressure` |
| `DegradeRouteByBackpressure` | `session={sfu_session_open, sfu_session_draining}; route={route_selected, route_delayed}` | `route_degraded` | `sfu_session_not_accepting`, `route_degraded_by_backpressure` |
| `CloseEndpointByBackpressure` | `session={sfu_session_open, sfu_session_draining}; endpoint={endpoint_admitted, endpoint_degraded}` | `endpoint_draining` | `sfu_session_not_accepting`, `endpoint_closed_by_backpressure` |
| `ApplyQualitySuppression` | `session={sfu_session_open, sfu_session_draining}; route=route_selected` | `route_suppressed_by_quality` | `sfu_session_not_accepting`, `route_suppressed_by_quality` |
| `RecoverRoute` | `session={sfu_session_open, sfu_session_draining}; route=route_suppressed_by_quality` | `route_selected` | `sfu_session_not_accepting`, `quality_recovery_not_allowed` |
| `RecoverRouteFromBackpressure` | `session={sfu_session_open, sfu_session_draining}; route={route_delayed, route_degraded, route_suppressed_by_backpressure}` | `route_selected` | `sfu_session_not_accepting`, `backpressure_recovery_not_allowed` |
| `DropRoute` | `session={sfu_session_open, sfu_session_draining}; route={route_candidate, route_selected, route_delayed, route_suppressed_by_backpressure, route_suppressed_by_quality, route_degraded}` | `route_dropped` | `sfu_session_not_accepting`, `route_conflict`, `target_unavailable` |
| `CloseRoute` | `session={sfu_session_open, sfu_session_draining}; route={route_selected, route_delayed, route_suppressed_by_backpressure, route_suppressed_by_quality, route_degraded, route_dropped}` | `route_closed` | `sfu_session_not_accepting`, `target_unavailable` |
| `BeginSessionDrain` | `session=sfu_session_open` | `sfu_session_draining` | `sfu_session_not_accepting` |
| `CloseSession` | `session={sfu_session_open, sfu_session_draining}` | `sfu_session_closed` | `sfu_session_not_accepting` |

### 3.8 暗黙遷移規則（draining / closed / endpoint 不可の網羅）

- 明示的な `session=sfu_session_draining` allowed pre-state を持たない遷移は、`sfu_session_draining` で `sfu_session_not_accepting` により reject されます。
- 明示的な `session=sfu_session_closed` allowed pre-state を持たない遷移は、`sfu_session_closed` で `sfu_session_not_accepting` により reject されます。
- admitted/degraded endpoint を要求する遷移は、`endpoint_draining`、`endpoint_removed`、`endpoint_rejected` を `target_unavailable` により reject します。

### 3.9 Packet Relation

- packet buffer lifecycle は SFU state ではありません。
- core は packet identity に対して route、suppression、drop を決定してよい。
- driver は packet bytes、cache、queue、retransmission、release を所有します。
- pacing queue、retransmission cache、NACK-adjacent concrete feedback は driver-owned execution detail のままです（必須）。

### 3.10 collapse 条件（state machine）

- worker runtime が route state を所有する。
- driver が publication/subscription decision を所有する。
- packet buffer lifecycle が core SFU state として扱われる。
- pacing/retransmission cache が core SFU state として扱われる。
- concurrent SFU route/endpoint conflict が driver/runtime order で解決される。
- quality exporter が suppression/recovery decision を所有する。
- regulated-specific priority が generic SFU state transition を変更する。

---

## 4. Congestion / Pacing / Retransmission（policy/execution 境界）

本節は SFU の congestion observation、pacing、retransmission、NACK-adjacent behavior の責務境界を固定します。本節は実装済み congestion controller、具体的 pacing algorithm、packet loss recovery 成功を主張しません。

### 4.1 境界（owner）

| Concern | Owner | 規則 |
|---|---|---|
| congestion/backpressure policy | core/sfu | suppress/drop/degrade/delay/close/recovery rejection の意味論 |
| quality decision | core/quality + core/sfu | quality metric model と decision semantics |
| raw queue/cache/socket measurement | driver | concrete observation を core-owned metric type へ変換 |
| packet cache bytes | driver | bounded retention, eviction, lease release |
| pacing queue/timer | driver | scheduling execution detail |
| retransmission execution | driver | cached packet lease から送信 |
| route/target selection | core/sfu | forwarding target set と routing decision |
| send operation | driver | concrete I/O and transport backend |

core は packet cache、pacing queue、timer wheel、NACK buffer、socket send queue、raw bytes を所有しません（禁止）。driver は congestion/backpressure policy の正を所有しません（禁止）。

### 4.2 Congestion Observation 規則

driver は次を observe してよい（許可）: transmit queue depth、send latency、packet cache pressure、packet loss signal または NACK-like event、transport/backend send failure、memory pressure、receive buffer pressure。

driver はこれらの observation を core decision 前に core-owned metric/input type へ map しなければなりません（必須）。driver-local observation が route/subscription/endpoint/quality state を直接 mutate してはなりません（禁止）。

### 4.3 Pacing 規則

- pacing は execution であり policy ではありません。
- core は cataloged reason を伴う delay/degrade/suppress/drop intent を返してよい。
- driver は accepted forwarding decision を実行するために pacing queue と timer を実装してよい。
- pacing queue は resource bounds に従って bounded でなければなりません（必須）。enqueue または wait bound を超えた場合、driver は `dropped_by_transmit_queue_bound` と reason `sfu_transmit_queue_bound_exceeded` で packet lifecycle を終了しなければなりません（必須）。

### 4.4 Retransmission 規則

- retransmission は driver-owned bounded packet cache からのみ許可されます（必須）。
- core は packet/route/target に対して retransmission が semantically allowed かを決定してよい。
- driver は `PacketId` を依然 valid な `BufferLease` または cached packet representation に解決して retransmission を実行します。
- cache entry が absent/expired/evicted/bound-exceeded の場合、retransmission は対応する release/audit reason で fail-close しなければなりません（必須）。driver は core で packet bytes を再構築したり、core に packet payload を persist させたりしてはなりません（禁止）。

### 4.5 NACK-Adjacent 規則

- NACK-like external feedback は core-owned feedback reference へ変換されるまで driver input です。
- core は変換後 reference を retransmission intent、suppression input、quality/backpressure signal として解釈してよい。
- concrete feedback packet parsing、protocol-specific ACK/NACK grammar、backend event object は driver 所有です（必須）。

### 4.6 Failure Mapping（congestion/pacing/retransmission）

| Failure | Required reason または release code |
|---|---|
| packet cache が packet を retain できない | `packet_cache_bound_exceeded` |
| packet cache retention duration 超過 | `retention_duration_exceeded` |
| transmit queue bound 超過 | `sfu_transmit_queue_bound_exceeded` |
| backpressure が packet forwarding を suppress | `packet_suppressed_by_backpressure` |
| backpressure が packet forwarding を drop | `packet_dropped_by_backpressure` |
| backpressure が route を degrade | `route_degraded_by_backpressure` |
| backpressure が endpoint を close | `endpoint_closed_by_backpressure` |
| suppressed/degraded/delayed route からの recovery が reject | `backpressure_recovery_not_allowed` |
| target unavailable | `target_unavailable` |
| concrete send 失敗 | `network_send_failed` |
| driver shutdown | `driver_shutdown` |

packet lifecycle closure は packet buffer lifecycle（第07章 §7）の release code 表が authoritative のままです。

### 4.7 Audit 規則

- forwarding/route/subscription/endpoint/packet lifecycle を変更する各 congestion/backpressure decision は、resource bounds と audit event 規則の event type mapping を使用して記録しなければなりません（必須）。
- driver pacing execution log は diagnostics のみであり、`backpressure_decision`、`resource_bound_decision`、`sfu_forwarding_decision`、`quality_violation_decision` audit event を置換しません（必須）。

### 4.8 禁止事項（congestion/pacing/retransmission）

- core が pacing queue または retransmission cache を所有する。
- driver が core decision なしに congestion policy で route/subscription/endpoint state を変更する。
- retransmission cache が unbounded。
- NACK parser concrete type が core に入る。
- core が retransmission のために packet payload を persist する。
- retransmission transform が packet rewrite / media transform admission（第07章 §5）なしに実行される。
- queue overflow が successful forwarding として扱われる。

### 4.9 collapse 条件（congestion/pacing/retransmission）

- pacing/retransmission data structure が core-owned になる。
- driver congestion observation が core decision なしに SFU state を mutate する。
- retransmission が driver lease lifetime 終了後の packet bytes を使う。
- queue/cache overflow に cataloged reason と audit event がない。
- diagnostic pacing log が forwarding success evidence として使われる。
- retransmission path が rewrite/transform class と copy bound evidence を bypass する。
