# core-runtime-time-concurrency

状態: SSOT 統合版
日付: 2026-06-28 JST

## 目的

本章は arcRTC v0.2 Kernel の core が所有する runtime / clock / randomness 抽象境界、runtime task / worker lifecycle、retry / timeout / cancellation、concurrency / ordering / lock ownership、atomicity / transaction / compensation、unit / measurement / time normalization、time synchronization / clock skew / timestamp trust、resource bounds / backpressure の現行完全仕様を、本章のみで再現実装可能な粒度で内在化することを目的とします。

依存方向の表記: `A <- B` は「B が A に依存」を意味します。core は外部 I/O 非依存です。time、entropy、spawn、timer、cancellation は core policy（決定）と driver execution（実行）を分離します。driver は時刻観測と物理実行を所有し、core は expiry / deadline / ordering / bound policy を所有します。entrypoints は selected implementation を wiring するのみであり、time policy、entropy semantics、runtime retry semantics、bound policy、enforcement を所有しません。本章は各 lifecycle の全状態遷移と閉集合語彙、fail-closed 条件、failure mapping を内在化します。

---

## 1. Runtime / Clock / Randomness 境界

### 1.1 所有境界（owner）

core は `ClockPort`、`RandomPort`、`RuntimePort` の境界を所有します。time、entropy、spawn、timer、cancellation を core policy と driver execution に分離します。

| Surface | core 所有 | driver 所有 |
|---|---|---|
| ClockPort | current time abstraction、monotonic comparison input、deadline semantics | system clock / test clock implementation |
| RandomPort | nonce / opaque ID / challenge entropy contract | OS RNG / deterministic test RNG implementation |
| RuntimePort | timer / spawn / cancellation contract | tokio や他の runtime execution |

entrypoints は selected implementation を wiring します。entrypoints は time policy、entropy semantics、runtime retry semantics を定義してはなりません（禁止）。

### 1.2 Time Authority 規則

core は expiry と deadline policy を所有します（必須）。driver は time observation を供給します。cross-node timestamp trust と skew-bounded comparison は本章 7 節に従います。

TURN allocation expiry、permission expiry、channel bind expiry、Signaling room lifecycle、resource retention、configuration startup timeout は、core policy と cataloged reason を通して評価されなければなりません（必須）。duration、timestamp、rate window、deadline の単位は policy 比較前に正規化されなければなりません（必須、本章 6 節）。

driver timer delay は late execution を起こし得ますが、core policy が item を expired とみなすか否かを再定義してはなりません（禁止）。

### 1.3 Randomness 規則

driver は entropy を供給します。core は nonce、opaque ID、challenge、reference stability の意味を所有します。

`RandomPort` output は core 所有 constructor への opaque input として扱われなければなりません（必須）。driver は participant role、tenant、regulated subject、application user identity を生成 randomness に encode してはなりません（禁止）。deterministic test RNG は testing scope に限り許可され、production / runtime evidence として使用してはなりません（禁止）。

### 1.4 Runtime 規則

`RuntimePort` は timer、spawn、cancellation、shutdown observation を抽象化します。runtime implementation は execution を schedule してよいが、domain policy を所有してはなりません（禁止）。`RuntimePort` の task / worker lifecycle、detached task 禁止、supervision scope、join/cancel evidence、task panic handling は本章 2 節が所有します。

runtime が required driver execution を schedule できない場合、path は cataloged configuration / resource / driver / shutdown reason を通して fail しなければなりません（必須、fail-closed）。runtime task cancellation は、relevant core state machine がその遷移を accept しない限り、accepted domain lifecycle transition として扱ってはなりません（禁止）。runtime panic / crash / restart observation は process lifecycle evidence であり、それ自体で successful domain transition ではありません。

### 1.5 Failure Mapping（runtime / clock）

| Failure | 必須 reason |
|---|---|
| required runtime configuration missing | `runtime_config_missing` |
| runtime が selected driver/entrypoint を initialize できない | `runtime_config_invalid` |
| runtime task class / owner / supervision / spawn / join / cancel / panic failure | 本章 2 節の runtime task reason |
| driver runtime is shutting down | `driver_shutdown` |
| timer/queue resource bound exceeded | 一致する resource-bound reason（本章 8 節） |
| memory pressure bound exceeded | `memory_pressure_exceeded` |

### 1.6 禁止

- core が concrete runtime handle を import する。
- driver が local timer delay から expiry semantics を決定する。
- driver が normalized unit policy なしに raw platform time/measurement を比較する。
- random generator implementation が identity semantics を所有する。
- entrypoints が required configuration missing 後に system clock / RNG / runtime default を silent に代替する。
- deterministic test clock/RNG を production-readiness evidence として使用する。
- runtime worker state が SFU route state、TURN allocation state、Signaling room state を所有する。
- detached task / worker supervision が implicit である。
- wall-clock timestamp を skew trust class なしに cross-node causal order として扱う。

### 1.7 Collapse 条件（runtime / clock）

- driver timer implementation が domain expiry を定義する。
- runtime worker が domain state transition を所有する。
- runtime task lifecycle failure が generic runtime availability に隠れる。
- entropy source が external identity を core ID に encode する。
- core が tokio / browser / native runtime types を import する。
- runtime failure が無視されたまま close / complete / ready が主張される。
- process crash/restart が classification と recovery evidence なしに recovery success として扱われる。
- clock synchronization evidence が local ClockPort availability から推論される。

---

## 2. Runtime Task / Worker Lifecycle

### 2.1 境界（owner）

本節は `RuntimePort` のうち task / worker lifecycle が domain state、driver queue、entrypoints supervisor、shutdown、crash recovery と混同されないよう所有者と証跡粒度を固定します。本節は runtime 実装、tokio task 実装、worker pool 実装、supervisor integration、runtime readiness を主張しません。

| 関心事 | 所有者 | 規則 |
|---|---|---|
| domain state transition | core | task completion / cancellation / panic observation だけでは変化しない |
| RuntimePort task contract | core | spawn/cancel/join observation の core 所有 contract |
| concrete task handle / join handle | driver/entrypoints runtime | core public API / domain state へ出さない |
| driver I/O worker | driver | socket / packet / sink / persistence / exporter execution のみ |
| entrypoints supervision task | entrypoints | process/component lifecycle observation のみ |
| task queue / mailbox | physical execution の owner | bounded resource。それ自体は ordering authority ではない |
| task panic observation | entrypoints/driver runtime | process または task lifecycle evidence。domain recovery ではない |
| task cancellation | driver/entrypoints observation、command が既に core に入っていれば core decision | retry/timeout/cancellation 規則（本章 3 節）に従う |

Runtime task は domain aggregate owner ではありません。Runtime task completion は accepted domain decision の代替ではありません。

### 2.2 Task Classes（閉集合）

v0.2 initial architecture の task class は次に限定します。

| Class | 意味 | 規則 |
|---|---|---|
| `no_runtime_task` | target path が runtime task を要しない | task evidence claim なし |
| `driver_io_worker` | network/socket/protocol driver worker | domain decision を所有不可 |
| `driver_packet_worker` | packet queue/cache/rewrite/forward execution worker | packet byte ownership は driver-local のまま |
| `driver_sink_worker` | audit/metrics/persistence/export sink worker | audit/event meaning を代替不可 |
| `entrypoints_supervision_task` | entrypoints-level process/component supervisor task | それ自体で readiness/recovery proof ではない |
| `runtime_timer_task` | timer/deadline callback execution | timer execution は expiry policy ではない |
| `test_runtime_task` | test 用 deterministic / fake task execution | test evidence のみ |
| `detached_task_requested` | admitted parent/supervision scope を持たない task | 将来の仕様が exact class を admit しない限り reject |

新しい task class は仕様の更新を要します。

### 2.3 Ownership 規則

claim に影響する全 runtime task は次を宣言しなければなりません（必須）。task class、parent component または supervision scope、owning layer、allowed input reference types、allowed output observation、cancellation propagation rule、join/wait bound、適用時は queue/mailbox bound、panic/failure mapping、audit event relation。

core は `RuntimePort` を通して opaque な schedule/cancellation/result observation を受け取ってよい。core は concrete task handle、runtime handle、join handle、worker queue object、task-local cache、task-local lock を domain state として受け取ってはなりません（禁止）。driver は concrete task handle を implementation detail としてのみ所有してよい。entrypoints は task を wire し observe してよいが、task completion を domain acceptance に変えてはなりません（禁止）。

### 2.4 Supervision 規則

Detached task execution は v0.2 initial architecture で禁止します。全 task は次のいずれかの下になければなりません（必須）。entrypoint startup/run supervision、driver component supervision、bounded test harness supervision、explicit RuntimePort schedule/cancel scope。

parent scope が終了する場合、task は次のいずれかをしなければなりません（必須）。bounded drain/join window 内で complete、cataloged reason で cancel、fail して cataloged task lifecycle evidence を record。

task または process の restart は domain recovery を証明しません。

### 2.5 Cancellation / Shutdown 関係

Task cancellation は cancellation の発生した境界を保存しなければなりません（必須）。

| Cancellation surface | 必須 relation |
|---|---|
| driver/core conversion 前 | driver-local cancellation。domain mutation なし |
| command が core に入った後 | 先行 core decision evidence が authoritative のまま |
| shutdown/drain 中 | cross-plane shutdown/drain 規則に従う |
| driver queue/cache execution 中 | driver failure/shutdown/resource reason。domain decision を書き換えない |
| test harness timeout 中 | testing evidence が timeout/cancel class を record |

Task cancellation は、relevant な仕様がその domain result を admit しない限り、accepted leave、room close、endpoint removal、allocation release、audit flush success、packet forwarding success として報告してはなりません（禁止）。

### 2.6 Resource Bound 規則

task queue、worker mailbox、join wait、cancellation wait、supervision restart attempt は bounded でなければなりません（必須）。bound が resource evidence に参加する場合は本章 8 節に従います。task class が本章 8 節に無い新 bounded resource を導入する場合、実装前に resource bound 規則と reason catalog を update しなければなりません。

### 2.7 Failure Mapping（runtime task）

| Failure | 必須 reason |
|---|---|
| task class is not admitted | `runtime_task_class_not_admitted` |
| task owner/supervision scope is invalid | `runtime_task_owner_violation` |
| task has no admitted supervision scope | `runtime_task_supervision_missing` |
| detached task is requested | `runtime_task_detached_not_allowed` |
| runtime cannot spawn required task | `runtime_task_spawn_failed` |
| task join/wait observation failed | `runtime_task_join_failed` |
| task cancellation failed or could not be observed | `runtime_task_cancel_failed` |
| task panic was observed | `runtime_task_panic_detected` |
| task queue/mailbox/join bound exceeded | `runtime_task_queue_bound_exceeded` |
| driver/runtime is shutting down | `driver_shutdown` |

### 2.8 Audit 規則（runtime task）

Runtime task / worker lifecycle decision は audit event type `runtime_task_lifecycle_decision` を使用します。event は `StartupRunId`、task class、parent component/supervision scope、owning layer、materialized 時の task reference、適用時の cancellation/join bound、command-scoped 時の `CorrelationId` を carry しなければなりません（必須）。

### 2.9 Evidence 規則（runtime task）

Runtime task / worker lifecycle evidence は次を record しなければなりません（必須）。task class、owning layer、parent component または supervision scope、core が task を observe する場合の RuntimePort relation、command/procedure、working directory、command-scoped 時の correlation ID、process-scoped 時の startup run ID、spawn/cancel/join/panic outcome、bounded wait/queue/restart policy、non-success の cataloged reason、close-not-claimed scope。task execution log は、上記 field を持たない限り diagnostic のみです。

### 2.10 禁止（runtime task）

- concrete runtime task handle が core public API または domain state に現れる。
- admitted supervision scope なしに detached task が spawn される。
- worker completion が domain decision として扱われる。
- task cancellation が先行 accepted/rejected domain decision を書き換える。
- task panic が graceful shutdown または recovery success として扱われる。
- unbounded task queue / mailbox / join wait / restart loop が許可される。
- entrypoints supervisor restart が runtime readiness または domain restore evidence として使用される。
- driver worker が Signaling room、SFU route、TURN allocation、audit chain semantics を所有する。

---

## 3. Retry / Timeout / Cancellation

### 3.1 境界（owner）

本節は command の再試行可否、driver operation retry、deadline 超過、client/entrypoint/runtime cancellation が state、audit、external response を fail-open させないための owner と結果形状を固定します。本節は retry 実装、scheduler 実装、runtime cancellation 成功を主張しません。

| 関心事 | 所有者 | 規則 |
|---|---|---|
| retryability metadata | core reason catalog | category default と code override |
| domain command idempotency | core | duplicate と replay semantics |
| driver operation retry execution | driver | bounded retry のみ |
| command deadline policy | core | typed policy と monotonic time observation |
| timer/scheduler execution | driver | RuntimePort を通した runtime detail |
| client cancellation observation | driver/sdk | external signal conversion |
| entrypoint shutdown cancellation | entrypoints が initiate、core/drivers が classify | domain lifecycle と driver shutdown は別 |

Retry/cancel/timeout は第二の reason vocabulary を生んではなりません（禁止）。

### 3.2 Retry Classes（閉集合）

| Retry class | 所有者 | 規則 |
|---|---|---|
| `no_retry` | core | non-idempotent または policy-rejected command は自動 retry 不可 |
| `idempotent_command_replay` | core | 同一 correlation/command identity と idempotency rule が必須 |
| `driver_transport_retry` | driver | send/receive operation retry、bounded、failure が decision に影響する場合は audited |
| `persistence_retry` | driver | bounded retry store、persistence/resource 規則に従う |
| `audit_sink_retry` | driver | bounded sink retry、audit evidence の代替ではない |
| `test_only_retry` | testing scope | runtime/prod evidence として使用不可 |

新しい retry class は仕様の更新を要します。

### 3.3 Timeout / Deadline 規則

Timeout は operation boundary、Deadline は policy boundary です。

| Timeout/deadline surface | 所有者 | failure reason |
|---|---|---|
| command deadline | core | `operation_deadline_exceeded` |
| driver send/receive timeout | driver が core reason に変換 | `network_send_failed` または `network_receive_failed` |
| persistence retry duration | driver bounded execution | `persistence_retry_duration_exceeded` |
| packet/cache retention duration | driver bounded execution | `retention_duration_exceeded` |
| scheduled action 中の runtime shutdown | driver/entrypoints observation | `driver_shutdown` |

driver timer delay は core expiry semantics を再定義しません。

### 3.4 Cancellation 規則

Cancellation は発生箇所で classify しなければなりません（必須）。

| Cancellation source | boundary | 必須 handling |
|---|---|---|
| client が driver/core boundary 前に cancel | driver/sdk | domain state mutation なし。external cancellation response は local でよい |
| client が command の core 進入後に cancel | core decision または follow-up event | 既に accepted/rejected の decision evidence を保存 |
| entrypoint が shutdown 開始 | entrypoints + core/drivers | cross-plane shutdown/drain 規則に従う |
| runtime task cancelled | driver | `operation_cancelled` または `driver_shutdown` に変換 |
| test harness cancels | testing scope | evidence report が timed-out/cancelled test outcome として classify |

Cancellation は、relevant command が core state machine で accept されない限り、successful leave、room close、endpoint removal、allocation release として報告してはなりません（禁止）。

### 3.5 Retry Preconditions

Retry は次の全条件が成立するときのみ許可されます（必須）。reason metadata が failure を retryable と marking、または specific な仕様が retry を許可。command が idempotent、または retry が domain acceptance 前の driver-local。retry bound が宣言済み。retry duration が宣言済み。correlation と original command identity が保存。audit/event evidence が first attempt と retry attempt を区別可能。retry が raw sensitive data で privacy/redaction boundary を越えない。いずれかが fail すれば retry は許可されません（禁止）。

### 3.6 Failure Mapping（retry / timeout / cancellation）

| Failure | 必須 reason |
|---|---|
| core command deadline exceeded | `operation_deadline_exceeded` |
| operation cancelled after boundary | `operation_cancelled` |
| retry store bound exceeded | `persistence_retry_bound_exceeded` |
| retry duration exceeded | `persistence_retry_duration_exceeded` |
| network send retry exhausted | `network_send_failed` |
| network receive retry exhausted | `network_receive_failed` |
| audit backlog bound reached | `audit_backlog_bound_exceeded` |
| runtime or driver shutdown | `driver_shutdown` |
| runtime task lifecycle failure | 本章 2 節の runtime task reason |

### 3.7 禁止 / Collapse 条件（retry / timeout / cancellation）

禁止: non-idempotent domain command の自動 retry。driver retry が domain decision を変える。core decision なき cancellation を accepted domain lifecycle transition として扱う。timeout を generic success または free-text failure として報告。unbounded retry loop / unbounded retry store。test-only retry behavior を runtime evidence として使用。retry が original correlation または first-attempt evidence を消す。runtime task cancellation を successful domain lifecycle transition として扱う。

Collapse: retryability が driver exception text から推論される。command deadline に cataloged reason が無い。core entry 後の cancellation が先行 decision evidence を失う。retry bound または retry duration が欠落。timeout/cancel behavior が仕様の更新なしに driver ごとに異なる。task cancel/join failure が cancellation evidence 主張下に隠れる。

---

## 4. Concurrency / Ordering / Lock Ownership

### 4.1 境界（owner）

本節は同一 aggregate / resource への並行 command が driver-local lock や runtime scheduling によって domain semantics を変えないよう、serialize scope と failure mapping を固定します。本節は lock 実装、actor 実装、runtime scheduler 実装を主張しません。

| 関心事 | 所有者 | 規則 |
|---|---|---|
| domain ordering validity | core | state machine と command semantics |
| aggregate serialization scope | core | どの command が同一 state で conflict するか |
| physical mutex / channel / actor mailbox | package layer に応じ driver または core implementation detail | domain source-of-truth ではない |
| runtime scheduling | driver | execution timing。ordering authority ではない |
| runtime task lifecycle | driver/entrypoints runtime | supervision/cancel/join evidence。ordering authority ではない |
| inbound wire ordering observation | driver | observation を保存、validity を決定しない |
| idempotency | core | duplicate と replay semantics |

Lock state は domain state ではありません。Runtime execution order は自動的に valid command order ではありません。

### 4.2 Serialization Scopes（閉集合）

| Scope | 所有者 | 保護される semantics |
|---|---|---|
| `room_scope` | core/signaling | room state、participant membership、command idempotency |
| `participant_scope` | core/signaling | room 内 participant lifecycle |
| `sfu_session_scope` | core/sfu | session admission と route lifecycle |
| `sfu_endpoint_scope` | core/sfu | endpoint publication/subscription lifecycle |
| `packet_lifecycle_scope` | driver | packet buffer lease、queue、cache、release |
| `turn_allocation_scope` | core/turn | allocation lifecycle と refresh |
| `turn_permission_scope` | core/turn | permission と relay authorization |
| `audit_chain_scope` | core/audit | hash-chain ordering |
| `configuration_scope` | core/entrypoints boundary | startup/wiring validation sequence |

新しい serialization scope は仕様の更新を要します。

### 4.3 Ordering 規則

core は command が現 state に valid か否かの decision を所有します。driver は command を observed order で deliver してよいが、arrival order を唯一の domain validity rule として使ってはなりません（禁止）。

Ordering failure は次に map しなければなりません（必須）。protocol command order failure は `command_order_violation`。idempotency duplicate rejection は `duplicate_command`。並行 accepted candidate が同一 serialization scope で conflict する場合は `concurrency_conflict`。bounded serialization queue/lock admission 枯渇は `lock_contention_bound_exceeded`。

### 4.4 Lock Ownership 規則

| Lock/resource | 許可 owner | 規則 |
|---|---|---|
| aggregate mutation guard | core implementation detail | core state を保護、API に lock type を露出しない |
| driver queue/mutex | driver | external I/O/resource を保護、domain ordering を所有しない |
| entrypoint-level process lock | entrypoints | startup/process guard のみ |
| test harness synchronization | testing scope | production semantics ではない |

Lock handle、actor mailbox、runtime task handle、queue object は core public API または domain state に現れてはなりません（禁止）。Task class と supervision scope は serialization scope を再定義してはなりません（禁止）。

### 4.5 Bound 規則 / Audit 規則

unbounded growth risk を持つ全 serialization queue または lock wait は次を定義しなければなりません（必須）。scope、最大 pending command または wait duration、owner、failure reason、audit event mapping、cancellation behavior。bound 超過時は、より specific な resource reason が適用されない限り `lock_contention_bound_exceeded` を使用します。

command execution を reject/drop する concurrency/ordering failure は relevant plane event type または `resource_bound_decision` で audit/event evidence を emit しなければなりません（必須）。`lock_contention_bound_exceeded` は resource policy owner `core`、physical resource owner を implementation owner に一致させた `resource_bound_decision` を使用します。

### 4.6 禁止 / Collapse 条件（concurrency）

禁止: driver lock acquisition order が domain command order を定義。runtime scheduler order が state machine precondition を代替。unbounded aggregate command queue。lock object が domain model または public API になる。race conflict が core decision なき last-writer-wins で解決。test synchronization behavior を production semantics として扱う。task scheduling order を serialization authority として扱う。

Collapse: domain ordering validity が driver arrival order のみから推論。lock contention が無制限に成長可能。concurrent conflict に cataloged reason が無い。physical lock state が core semantic result に現れる。serialization scope が仕様の更新なしに entrypoint binary ごとに異なる。task lifecycle または worker queue ownership が domain ordering semantics を変える。

---

## 5. Atomicity / Transaction / Compensation

### 5.1 境界（owner）

本節は core decision、domain event、audit persistence、external response、driver execution の一部だけが成功した場合に、成功扱い・補償・証跡の扱いを誤らせないための owner と結果分類を固定します。本節は database transaction 実装、outbox 実装、compensation 実装を主張しません。

| 関心事 | 所有者 | 規則 |
|---|---|---|
| domain decision atomicity | core | aggregate transition と decision outcome |
| port intent emission | core | driver execution request。execution success ではない |
| concrete DB transaction | driver | storage implementation detail |
| audit persistence execution | driver | bounded sink/persistence detail |
| external response emission | driver/sdk | projection と send execution |
| compensation decision | domain state 変化時は core、external resource cleanup は driver | owner は explicit でなければならない |
| evidence の扱い | evidence | partial success は classification なしに close evidence にできない |

Atomicity boundary は実装前に named でなければなりません（必須）。

### 5.2 Atomicity Classes（閉集合）

| Class | 意味 | 規則 |
|---|---|---|
| `single_core_decision` | external side effect claim なき一つの core decision | core state のみで atomic |
| `core_decision_plus_audit_required` | domain decision が audit evidence を要する | audit failure は close evidence を block |
| `core_decision_plus_port_intent` | accepted decision に driver execution が続く | port intent は execution success ではない |
| `driver_local_transaction` | DB/file/network resource transaction | driver-owned。domain invariant を定義しない |
| `compensating_transition_required` | accepted state が follow-up compensation を要する | 仕様/state machine で定義必須 |
| `non_compensable_observation` | undo 不可な observation | rollback ではなく observation として報告 |

新しい atomicity class は仕様の更新を要します。

### 5.3 Commit Boundary 規則

全 command path は commit boundary を識別しなければなりません（必須）。before core entry、after core validation but before aggregate mutation、after aggregate transition、after audit projection、after driver persistence、after external response emission。command path が複数 boundary を越える場合、各 step は outcome と failure handling を要します。Partial success は per-step outcome と compensation が定義されない限り禁止です（禁止）。

### 5.4 Compensation 規則

Compensation は default では rollback ではありません。Compensation は次を宣言しなければなりません（必須）。triggering failure、affected state/event、owner、allowed compensating transition、audit event relation、external response rule、audit evidence relation。compensation rule が無い場合、system は original accepted decision を保存し、後発 failure を driver observation または evidence limitation として record しなければなりません（必須）。

### 5.5 Failure Mapping / Evidence 規則（atomicity）

| Failure | 必須 reason |
|---|---|
| atomic commit step failed | `atomic_commit_failed` |
| compensation required but not available | `compensation_required` |
| compensation execution failed | `compensation_failed` |
| persistence unavailable during commit | `persistence_unavailable` |
| audit backlog prevents required audit | `audit_backlog_bound_exceeded` |
| external response encoding failed | `external_encode_failed` |
| external response send failed | `network_send_failed` |
| driver shutdown during commit/compensation | `driver_shutdown` |

Closeout evidence は次を識別しなければなりません（必須）。atomicity class、commit boundary、per-step outcome、適用時の compensation status、audit/persistence/external response failure、close-not-claimed scope。partial success classification を欠く evidence は close / complete / ready claim に使えません（fail-closed）。

### 5.6 禁止 / Collapse 条件（atomicity）

禁止: driver DB transaction が domain invariant を定義。port intent を driver execution success として扱う。external response success を audit persistence success として扱う。compensation が core/state-machine rule なしに state を mutate。partial success を final success の裏に隠す。failed audit/persistence step を close evidence として使用。

Collapse: command path が commit boundary なしに複数 side effect を持つ。accepted domain decision が compensating transition なき driver failure で消える。compensation owner が implicit。evidence が per-step outcome を欠く。database transaction を core aggregate authority として扱う。

---

## 6. Unit / Measurement / Time Normalization

### 6.1 境界（owner）

本節は duration、deadline、rate window、bytes、packet count、quality metrics、benchmark values、clock source を比較可能な core 所有 representation に揃え、driver/platform 差や表記ゆれが policy decision を変えないよう固定します。本節は具体的閾値、benchmark result、runtime clock accuracy を主張しません。

| 関心事 | 所有者 | 規則 |
|---|---|---|
| normalized unit type | core | policy/decision に使う単位と丸め規則 |
| raw platform measurement | driver | OS/runtime/library specific observation |
| benchmark reporting unit | benchmark docs/reports | normalized unit と raw observation を区別 |
| wall-clock timestamp | driver/entrypoints supplied、core が observation として validate | ordering authority ではない |
| monotonic duration/deadline | ClockPort observation 経由の core policy | expiry/deadline decision |
| metric export format | driver | external labels/format は policy ではない |

driver は raw value を collect してよいが、core policy evaluation 前に convert しなければなりません（必須）。

### 6.2 Normalized Units（閉集合）

| Quantity | normalized unit | 規則 |
|---|---|---|
| duration | integer の milliseconds | policy/deadline 用 monotonic duration |
| timestamp | evidence 用 UTC epoch milliseconds | それ自体で domain ordering authority ではない |
| bytes | integer の bytes | binary size / buffer / frame bounds |
| packet count | integer count | packet/cache/queue bounds |
| rate | explicit numerator を持つ units per second | implicit time window 不可 |
| ratio | declared precision の rational または fixed decimal | hidden float comparison 不可 |
| bitrate | bits per second | bytes per second と区別 |
| jitter / RTT | integer milliseconds または declared precision | source と window が必須 |

新しい unit は仕様の更新を要します。

### 6.3 Time Source 規則

expiry、deadline、timeout、retention、rate window を伴う core policy decision は `ClockPort` 経由の monotonic time observation を使用しなければなりません（必須）。Wall-clock timestamp は audit/evidence display には許可されますが、それ自体で ordering には使用しません（禁止）。

Clock skew/drift handling は次を定義しなければなりません（必須）。source、適用時の maximum accepted skew、wall-clock が evidence-only か policy input か、time observation が invalid 時の failure reason、evidence reporting rule。Cross-node skew admission と time trust class は本章 7 節が定義します。

### 6.4 Rounding / Comparison 規則

Measurement comparison は次を定義しなければなりません（必須）。normalized unit、precision、rounding direction、comparison operator、inclusive/exclusive boundary、sampling window、raw measurement の owner、policy decision の owner。Driver/exporter formatting は comparison result を変えてはなりません（禁止）。

### 6.5 Failure Mapping / Evidence 規則（unit）

| Failure | 必須 reason |
|---|---|
| measurement cannot be normalized | `measurement_normalization_failed` |
| required time observation unavailable | `time_observation_unavailable` |
| operation deadline exceeded | `operation_deadline_exceeded` |
| retention duration exceeded | `retention_duration_exceeded` |
| time/measurement source の runtime configuration invalid | `runtime_config_invalid` |
| driver shutdown during measurement | `driver_shutdown` |

Report は useful 時の raw source と claim 使用時の normalized value の両方を record しなければなりません（必須）。Benchmark と quality report は unit、window、precision、aggregation method を述べなければなりません（必須）。unit/window を欠く evidence は performance、capacity、quality claim を支持できません（fail-closed）。

### 6.6 禁止 / Collapse 条件（unit）

禁止: driver-exported metric label が policy unit になる。wall-clock order が monotonic deadline/expiry policy を代替。float comparison が hidden precision を使う。bitrate と byte-rate を混同。benchmark report が unit/window/aggregation を省略。raw platform-specific stats object が core policy に入る。timezone conversion を synchronization evidence として扱う。

Collapse: policy threshold が normalized unit を欠く。driver/platform measurement format が core decision を変える。clock source が implicit。evidence が異なる unit/window で比較。wall-clock timestamp が唯一の ordering authority。cross-node timestamp comparison が time trust class または skew policy を欠く。

---

## 7. Time Synchronization / Clock Skew / Timestamp Trust

### 7.1 境界（owner）

本節は複数 node、複数 process、外部観測、audit/report timestamp を比較する際の時刻信頼境界を固定します。unit normalization は本章 6 節に従います。time source は driver/runtime が観測します。core は expiry、deadline、ordering tolerance、skew allowance、timestamp trust policy を所有します。entrypoints は selected clock/time-source implementation を wiring しますが、時刻信頼 policy を所有しません。

| 関心事 | 所有者 | 規則 |
|---|---|---|
| local monotonic duration | ClockPort / core policy | deadline と elapsed duration は可能時に monotonic comparison を使用 |
| wall-clock timestamp | driver observation / audit model | report/audit timestamp。default で causal authority ではない |
| cross-node skew policy | core / topology 仕様 | max skew と trust class が必須 |
| external time source | driver / deployment | NTP/PTP/cloud metadata 等は observation source のみ |
| timestamp normalization | unit/time 規則（本章 6 節） | precision、window、timezone、unit が必須 |
| evidence timestamp claim | evidence 規則 | command time と observation time を分離 |

### 7.2 Time Trust Classes（閉集合）

| Time trust class | 意味 | 規則 |
|---|---|---|
| `single_process_monotonic` | 一 process の monotonic elapsed comparison | local duration/deadline のみ証明可 |
| `single_node_wall_clock` | 一 node の wall-clock timestamp | observation time を label 可、cross-node causal order は不可 |
| `multi_node_bounded_skew` | node 間 skew が policy 内で measured | bounded cross-node comparison を支持可 |
| `external_trusted_time_source` | accepted external time source observation | source class と failure mode を record 必須 |
| `test_deterministic_clock` | deterministic test clock | test evidence のみ、production/runtime evidence ではない |
| `time_untrusted` | target claim に time source を信頼不可 | target claim は fail または reduce 必須 |

新しい time trust class は仕様の更新を要します。

### 7.3 Skew Policy 規則

cross-node または cross-process timestamp comparison は次を宣言しなければなりません（必須）。node scope、time trust class、max accepted skew、observation precision、measurement window、skew observation の source、skew を measure 不可時の failure reason、expiry/ordering/audit/evidence claim への impact。bounded skew が必須かつ observe 不可なら、claim は採用してはなりません（fail-closed）。

### 7.4 Ordering / Expiry / Deadline 規則

Wall-clock timestamp 単独は node 間の causal order を証明しません。Core ordering decision は correlation、sequence、idempotency、aggregate version、protocol state、または explicit bounded-skew policy を使用しなければなりません（必須）。Audit/report ordering は、hash-chain sequence、event sequence、bounded-skew evidence が record されない限り、presentation のみに timestamp を使用してよい。

Expiry/deadline policy は core 所有のままです。Driver time observation は late、unavailable、skewed であり得ますが、policy を再定義してはなりません（禁止）。Token expiry、TURN lifetime、ICE consent、session resumption、retention、retry timeout、public connection idle timeout は、comparison が monotonic duration、wall-clock timestamp、bounded-skew class のどれを使ったか record しなければなりません（必須）。

### 7.5 Failure Mapping（time synchronization）

| Failure | 必須 reason |
|---|---|
| measured skew exceeds policy | `clock_skew_exceeded` |
| time source is not trusted for the target claim | `time_source_untrusted` |
| required time synchronization observation is unavailable | `time_sync_unavailable` |
| timestamp order cannot be trusted for target comparison | `timestamp_order_untrusted` |

既存の local time observation failure は、synchronization/skew が target boundary でない場合は `time_observation_unavailable` を使用してよい。

### 7.6 Evidence / Audit 規則（time synchronization）

Time synchronization evidence は node scope、time trust class、max skew、observed skew、precision、measurement window、time source class、command time、observation time、rerun condition を record しなければなりません（必須）。JST report timestamp 単独は report metadata であり runtime clock synchronization を証明しません。

Time synchronization decision は audit event type `time_synchronization_decision` を使用します。event は node scope、time trust class、time source class、skew policy reference、observed skew class、`StartupRunId`、command-scoped 時の `CorrelationId` を carry しなければなりません（必須）。

### 7.7 禁止 / Collapse 条件（time synchronization）

禁止: wall-clock timestamp を bounded-skew evidence なしに cross-node causal order として使用。report creation time を runtime observation time として扱う。deterministic test clock を production time synchronization evidence として使用。driver-local NTP status が core expiry/deadline policy を再定義。timezone conversion を synchronization proof として扱う。missing skew observation を within-bound observation として受容。

Collapse: time trust class が absent。skew policy が cross-node claim に対し open-ended または unmeasured。wall-clock order が追加 evidence なしに causal order として扱われる。expiry/deadline owner が core policy から driver timer に移る。evidence が node scope、precision、measurement window を隠す。

---

## 8. Resource Bounds / Backpressure

### 8.1 Principle / Ownership

unbounded resource は禁止です（禁止）。queue、cache、buffer pool、retry store、audit sink backlog、packet retention、connection admission、serialization queue / lock wait は必ず bound と closed action を持たなければなりません（必須）。Runtime task queue / worker mailbox / join wait bound は本章 2 節と、resource-bound evidence として採用される場合は本節に従います。

| 関心事 | policy owner | physical resource owner | measurement owner | execution owner |
|---|---|---|---|---|
| admission bound | core | driver | driver | core decision, driver execution |
| packet cache bound | core | driver | driver | driver |
| transmit queue bound | core | driver | driver | driver |
| audit backlog bound | core | driver | driver | driver |
| persistence retry bound | core | driver | driver | driver |
| metrics export backlog | core | driver | driver | driver |
| driver receive buffer pool | core | driver | driver | driver |
| connection concurrency bound | core | driver | driver | driver |
| memory pressure bound | core | driver | driver | core decision, driver execution |
| serialization queue / lock wait bound | core | core または driver implementation detail | physical queue/lock の owner | core decision, physical owner execution |
| runtime task queue / worker mailbox / join wait | task owner に応じ core または driver | driver/entrypoints runtime | driver/entrypoints runtime | task lifecycle decision, physical owner execution |
| inbound frame size bound | driver | driver | driver | driver conversion |

entrypoints は bound policy や enforcement を所有しません。entrypoints は selected driver と typed configuration を wiring するのみです。

### 8.2 Bound Shape

各 bounded resource は次を定義します（必須）。resource name、owner、maximum units、適用時 maximum bytes、適用時 maximum duration、pressure observation type、reason catalog からの decision reason、drop / suppress / reject / retry / shed action、audit requirement。

### 8.3 Backpressure Decisions（閉集合）

core が所有する backpressure decision は次に限定し、各 decision は audit outcome と reason に接続します。

| Backpressure decision | audit event type | outcome | reason code |
|---|---|---|---|
| accept | `backpressure_decision` | `accepted` | success に不要 |
| delay action | `backpressure_decision` | `delayed` | `action_delayed_by_backpressure` |
| suppress forwarding | `backpressure_decision` | `suppressed` | `packet_suppressed_by_backpressure` |
| suppress subscription | `sfu_subscription_decision` | `suppressed` | `subscription_backpressure_suppressed` |
| suppress route state | `backpressure_decision` | `suppressed` | `route_suppressed_by_backpressure` |
| drop packet | `backpressure_decision` | `dropped` | `packet_dropped_by_backpressure` |
| pressure policy で packet retention 停止 | `backpressure_decision` | `dropped` | `packet_dropped_by_backpressure` |
| degrade route | `backpressure_decision` | `degraded` | `route_degraded_by_backpressure` |
| close endpoint | `backpressure_decision` | `closed_by_policy` | `endpoint_closed_by_backpressure` |
| backpressure-delayed/degraded/suppressed route state からの recovery を reject | `backpressure_decision` | `rejected` | `backpressure_recovery_not_allowed` |

capacity、endpoint capacity、connection concurrency による admission rejection は v0.2 initial canonical では resource-bound decision であり distinct backpressure decision ではありません。Memory pressure は v0.2 initial canonical では resource-bound shedding のみを使用し、separate admission rejection path を定義しません。backpressure による subscription suppression は `sfu_subscription_decision` を使用し、packet/route backpressure action は `backpressure_decision` を使用します。bound による packet cache eviction または packet retention expiry は resource-bound decision であり「pressure policy で packet retention 停止」ではありません。

driver は decision を実行します。driver は policy の正を所有しません。

### 8.4 Required Bounds（閉集合）

| Resource | required bound | bound exceeded reason code |
|---|---|---|
| active room set | maximum active rooms / window あたり maximum materializations | `room_capacity_exceeded` |
| room lifecycle | maximum room lifetime / maximum idle duration | `room_lifetime_exceeded` |
| signaling command queue | maximum commands / maximum wait duration | `signaling_command_queue_bound_exceeded` |
| room participant set | maximum participants | `admission_capacity_exceeded` |
| SFU endpoint admission | maximum admitted endpoints / maximum endpoint admission window | `endpoint_capacity_exceeded` |
| SFU packet cache | maximum packets / bytes / retention duration | `packet_cache_bound_exceeded`, `retention_duration_exceeded` |
| SFU transmit queue | maximum packets / bytes / wait duration | `sfu_transmit_queue_bound_exceeded` |
| SFU route candidates | decision あたり maximum candidates | `route_candidate_bound_exceeded` |
| TURN allocation table | maximum allocations | `allocation_capacity_exceeded` |
| TURN allocation lifetime | maximum lifetime | `allocation_lifetime_exceeded` |
| TURN refresh cap | maximum refresh count / maximum cumulative refresh duration | `refresh_limit_exceeded` |
| TURN permission table | allocation あたり maximum permissions | `permission_capacity_exceeded` |
| TURN permission lifetime | maximum permission lifetime | `permission_lifetime_exceeded` |
| TURN channel bind lifetime | maximum channel binding lifetime | `channel_bind_lifetime_exceeded` |
| TURN relay queue | maximum packets / bytes / wait duration | `turn_relay_queue_bound_exceeded` |
| audit sink backlog | maximum events / bytes / retry duration | `audit_backlog_bound_exceeded`, `retention_duration_exceeded` |
| persistence retry store | maximum retry entries / bytes / retry count / retry duration | `persistence_retry_bound_exceeded`, `persistence_retry_duration_exceeded` |
| metrics export backlog | maximum events / bytes | `metrics_backlog_bound_exceeded` |
| driver receive buffer pool | maximum leases / bytes | `buffer_pool_bound_exceeded` |
| inbound frame size | frame / message あたり maximum bytes | `frame_size_bound_exceeded` |
| connection concurrency | maximum concurrent connections / admission window | `connection_concurrency_exceeded` |
| memory pressure | maximum memory pressure state / pressure duration | `memory_pressure_exceeded` |
| aggregate serialization queue / lock wait | serialization scope あたり maximum pending commands / maximum wait duration | `lock_contention_bound_exceeded` |
| runtime task queue / worker mailbox / join wait | maximum pending tasks / maximum wait duration / maximum cancellation wait | `runtime_task_queue_bound_exceeded` |

各 required bound の owner tuple は本節 8.1 表と同一の (policy owner=core, physical=driver) を基本とし、serialization queue は physical owner が physical queue/lock の owner、runtime task queue は physical owner が driver/entrypoints runtime、inbound frame size は policy owner=driver / physical=driver です。

### 8.5 Fail-Closed 規則

bound exceeded は closed reason に map しなければなりません（必須、fail-closed）。silent unbounded growth は禁止です（禁止）。reason category と code の対応は次のとおりです（抜粋、各 bound は本節 8.4 の bound-specific code を使用）。room capacity / signaling queue / transmit queue / relay queue / admission capacity / endpoint capacity / route candidate / audit backlog / persistence retry store / metrics backlog / buffer pool / frame size / connection concurrency / memory pressure / lock contention / runtime task queue は reason category `resource_exhausted`。room lifetime / retention duration / allocation lifetime / permission lifetime / channel bind lifetime / refresh limit / persistence retry duration は reason category `expired`。backpressure 各 action は reason category `backpressure`。Generic capacity failure を reason code として emit してはなりません（禁止）。各 bounded resource は本節 8.4 の bound-specific reason code を使用しなければなりません（必須）。

### 8.6 Audit Event Mapping 規則

bound-related audit event type は次に固定します。Signaling / SFU / TURN decision event が同じ correlation ID で追加される場合でも、bound audit event の代替にしてはなりません（禁止）。同じ bound reason を carry する non-mapped Signaling / SFU domain decision event は supplementary であり、mapped bound audit event の owner field requirement を満たしません。

| Bound family | audit event type code |
|---|---|
| backpressure delay / packet suppression / packet drop / packet retention stop / route suppression / route degradation / endpoint close / recovery rejection | `backpressure_decision` |
| subscription suppression by backpressure | `sfu_subscription_decision` |
| room materialization / room lifecycle / signaling command queue / participant admission capacity | `resource_bound_decision` |
| SFU packet cache / SFU transmit queue / SFU route candidate bound / memory pressure | `resource_bound_decision` |
| TURN allocation capacity / allocation lifetime | `turn_allocation_decision` |
| TURN refresh cap | `turn_refresh_decision` |
| TURN permission capacity / permission lifetime | `turn_permission_decision` |
| TURN channel bind lifetime | `turn_channel_bind_decision` |
| TURN relay queue | `resource_bound_decision` |
| audit sink backlog / persistence retry store / metrics export backlog | `resource_bound_decision` |
| aggregate serialization queue / lock wait | `resource_bound_decision` |
| runtime task queue / worker mailbox / join wait | `runtime_task_lifecycle_decision` |
| driver receive buffer pool / connection concurrency | `resource_bound_decision` |
| inbound frame size | `driver_resource_bound_decision` |
| explicitly listed driver-local resource bound | `driver_resource_bound_decision` |

Inbound frame size bound exceeded は audit event type `driver_resource_bound_decision`、outcome `dropped`、reason `frame_size_bound_exceeded`、resource policy owner `driver`、physical resource owner `driver` を使用します。v0.2 initial canonical では `driver_resource_bound_decision` は inbound frame size に限定されます（別の driver-local resource bound が明示追加されない限り）。SFU transmit queue bound が packet lifecycle を drop する場合は audit event type `resource_bound_decision`、outcome `dropped`、reason `sfu_transmit_queue_bound_exceeded`、packet release reason `dropped_by_transmit_queue_bound` を使用します。TURN allocation / refresh cap / permission / channel bind bound row は TURN-specific decision event type を使用しますが resource-bound related audit event のままであり、resource policy owner `core`、physical resource owner `driver` を carry しなければなりません。TURN relay queue bound は `resource_bound_decision` を使用し、resource name、resource policy owner、physical resource owner に加え relay path の active `AllocationId` と `PermissionId` を carry しなければなりません。

### 8.7 Required Bound Closed Action Mapping（閉集合）

| Resource | closed action | audit event type | outcome | reason code |
|---|---|---|---|---|
| active room set | reject room materialization | `resource_bound_decision` | `rejected` | `room_capacity_exceeded` |
| room lifecycle | expire or close room lifecycle | `resource_bound_decision` | `expired` | `room_lifetime_exceeded` |
| signaling command queue | reject queued command admission | `resource_bound_decision` | `rejected` | `signaling_command_queue_bound_exceeded` |
| room participant set | reject participant admission | `resource_bound_decision` | `rejected` | `admission_capacity_exceeded` |
| SFU endpoint admission | reject endpoint admission | `resource_bound_decision` | `rejected` | `endpoint_capacity_exceeded` |
| SFU packet cache | drop or evict packet retention | `resource_bound_decision` | `dropped` | `packet_cache_bound_exceeded` |
| SFU packet cache | expire packet retention | `resource_bound_decision` | `expired` | `retention_duration_exceeded` |
| SFU transmit queue | drop packet enqueue / scheduling | `resource_bound_decision` | `dropped` | `sfu_transmit_queue_bound_exceeded` |
| SFU route candidates | reject route candidate construction | `resource_bound_decision` | `rejected` | `route_candidate_bound_exceeded` |
| TURN allocation table | reject allocation | `turn_allocation_decision` | `rejected` | `allocation_capacity_exceeded` |
| TURN allocation lifetime | expire allocation | `turn_allocation_decision` | `expired` | `allocation_lifetime_exceeded` |
| TURN refresh cap | expire allocation refresh path | `turn_refresh_decision` | `expired` | `refresh_limit_exceeded` |
| TURN permission table | reject permission | `turn_permission_decision` | `rejected` | `permission_capacity_exceeded` |
| TURN permission lifetime | expire permission | `turn_permission_decision` | `expired` | `permission_lifetime_exceeded` |
| TURN channel bind lifetime | expire channel bind | `turn_channel_bind_decision` | `expired` | `channel_bind_lifetime_exceeded` |
| TURN relay queue | drop relay data scheduling | `resource_bound_decision` | `dropped` | `turn_relay_queue_bound_exceeded` |
| audit sink backlog | shed audit backlog and reject new audit-required path | `resource_bound_decision` | `shed` | `audit_backlog_bound_exceeded` |
| audit sink backlog | expire audit retry entry and reject new audit-required path | `resource_bound_decision` | `expired` | `retention_duration_exceeded` |
| persistence retry store | shed retry entry | `resource_bound_decision` | `shed` | `persistence_retry_bound_exceeded` |
| persistence retry store | expire retry entry | `resource_bound_decision` | `expired` | `persistence_retry_duration_exceeded` |
| metrics export backlog | shed metrics export item | `resource_bound_decision` | `shed` | `metrics_backlog_bound_exceeded` |
| driver receive buffer pool | reject receive buffer lease | `resource_bound_decision` | `rejected` | `buffer_pool_bound_exceeded` |
| inbound frame size | drop inbound frame before core entry | `driver_resource_bound_decision` | `dropped` | `frame_size_bound_exceeded` |
| connection concurrency | reject connection admission | `resource_bound_decision` | `rejected` | `connection_concurrency_exceeded` |
| memory pressure | shed lower-priority resource action | `resource_bound_decision` | `shed` | `memory_pressure_exceeded` |
| aggregate serialization queue / lock wait | reject command admission to serialization scope | `resource_bound_decision` | `rejected` | `lock_contention_bound_exceeded` |
| runtime task queue / worker mailbox / join wait | reject spawn/schedule/join continuation | `runtime_task_lifecycle_decision` | `rejected` | `runtime_task_queue_bound_exceeded` |

### 8.8 Audit Backlog Overflow 規則

audit sink backlog exceeded は自らの overflow event を saturated normal audit backlog へ recursively enqueue してはなりません（禁止）。audit sink backlog が bound に達したとき: (1) reserved non-recursive overflow record path を通じ outcome `shed`、reason `audit_backlog_bound_exceeded`、resource policy owner `core`、physical resource owner `driver` の `resource_bound_decision` を一つ emit する。(2) normal audit capacity が利用可能になるまで new audit-required command path を `audit_backlog_bound_exceeded` で reject/stop する。(3) required audit event を normal backlog または reserved overflow record path で record できない場合、audit-required decision を accepted として扱わない。audit sink backlog retry duration が expire したとき: (1) 同じ reserved non-recursive overflow record path を通じ outcome `expired`、reason `retention_duration_exceeded`、owner tuple 同上の `resource_bound_decision` を一つ emit する。(2) 影響を受けた previously accepted audit-required decision を closeout 目的で audit-failed と mark する。(3) その affected decision を close / complete / ready evidence として使用することを禁止する。

### 8.9 禁止 / Collapse 条件（resource bounds）

禁止: unbounded queue / cache / retry / packet retention。reason なき best-effort fallback。core reason mapping なき driver-local drop。entrypoints が core bound policy を override。task queue または worker mailbox bound が runtime implementation の裏に隠れる。

Collapse: bound の無い resource を導入する。worker execution が claim に影響するのに runtime task queue / worker mailbox / join wait bound を省略。driver が backpressure policy を所有。pressure reason が reason catalog に接続しない。capacity exceeded を success として扱う。unbounded retry または cache を許可。
