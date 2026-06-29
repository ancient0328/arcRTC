# entrypoints の運用ライフサイクル: health/admin・shutdown/drain・crash・authorization

状態: SSOT 統合版
日付: 2026-06-28 JST

## 目的

本章は arcRTC v0.2 Kernel における運用ライフサイクルの境界を、他文書・実コードを参照せずに完全自己完結で規定します。対象は次の4領域です: health / readiness / liveness / admin / maintenance、cross-plane shutdown / drain（process lifecycle と domain lifecycle の分離）、crash / panic / supervisor restart classification（全分類と再起動規則）、operator / admin authorization。本章単独で再現実装が可能な粒度を与えます。

operator-facing endpoint / CLI / probe / maintenance action が domain readiness、runtime readiness、closeout evidence と混同されないよう、owner と evidence 受理条件を固定します。process lifecycle と domain lifecycle は分離します。

---

## A 部 Health / Readiness / Liveness / Admin / Maintenance

### A-1 境界

| Surface | Owner | Rule |
|---|---|---|
| liveness observation | entrypoints/drivers | process/runtime が応答可能かの観測 |
| readiness decision | entrypoints composition + core/driver dependency status | selected path が受付可能かを合成するが domain decision ではない |
| domain acceptance | core | join/route/allocation 等の個別 decision |
| admin command | entrypoints/cli | core use case または driver operation boundary を呼ぶ |
| maintenance mode | entrypoints が initiate、core/drivers が classify | domain state transition とは分離 |
| probe response encoding | driver/entrypoints | external status projection |
| evidence report | reports | probe result は correlation/procedure/scope を伴う evidence のみ |

probe success は domain command success を証明しません。listener startup は readiness を証明しません。public endpoint lifecycle evidence は probe success と別です。

### A-2 Probe Classes（閉集合）

| Probe class | 意味 | Claim limit |
|---|---|---|
| `process_liveness` | process/runtime loop が observable | dependency readiness ではない |
| `driver_dependency_readiness` | selected driver dependency が initialize/respond 可能 | domain acceptance ではない |
| `core_policy_readiness` | required core policy bundle が accepted | runtime integration ではない |
| `composition_readiness` | selected entrypoint wiring が completed | production readiness ではない |
| `maintenance_status` | entrypoint が normal/draining/maintenance mode のいずれか | 単独では domain state proof ではない |
| `operator_action_result` | admin/CLI action result | core decision evidence の代替ではない |

新 probe class は v0.2 初期 scope 外です。

### A-3 Readiness Composition Rule

readiness は含む class を declare しなければなりません。readiness response は単一の unqualified boolean であってはなりません（必須）。

必須 field: startup run ID; entrypoint name; 該当時 profile class; selected drivers; 関連時 deployment topology class と node scope; dependency resolution が readiness に影響する時 service discovery source/resolution state; node-local state が影響する時 distributed state class/failover admission status; worker execution が影響する時 runtime task class と supervision state; service-to-service identity が影響する時 internal service trust class; probe class; included dependency checks; excluded checks; outcome; non-ready/failed outcome の cataloged reason; evidence class と close-not-claimed scope。

required readiness component が evaluate されない場合、その component を要する claim に対して readiness は満たされません。

### A-4 Admin and Maintenance Rule

admin と maintenance commands は次を行ってよいです（許可）: drain/shutdown を cross-plane shutdown/drain 規範（本章 B 部）経由で request する; redacted state references を inspect する; bounded verification commands を trigger する; driver dependency probes を request する; audit/hash-chain verification を request する。privilege を要する admin/maintenance commands は、target action 実行前に operator/admin authorization（本章 D 部）を pass しなければなりません。

admin と maintenance commands は次を行ってはなりません（禁止）: core aggregate state を core use case/state machine 外で mutate する; authorization/security boundary を bypass する; raw secret/token/packet/regulated payload を expose する; probe success を domain acceptance に変える; evidence report なしに closeout complete を mark する。

### A-5 Failure Mapping

| 失敗 | 必須 reason |
|---|---|
| readiness component not satisfied | `readiness_not_satisfied` |
| driver/entrypoint failure により health probe が execute 不可 | `health_probe_unavailable` |
| maintenance mode により action がブロック | `maintenance_mode_active` |
| admin action が boundary/policy により not allowed | `admin_action_not_allowed` |
| operator/admin authorization denied | `operator_action_denied` |
| operator/admin authorization context missing | `operator_authorization_context_missing` |
| required runtime configuration missing | `runtime_config_missing` |
| runtime configuration invalid | `runtime_config_invalid` |
| probe/action 中の driver shutdown | `driver_shutdown` |
| service discovery unavailable | `service_discovery_unavailable` |
| endpoint resolution stale または fallback not admitted | `service_endpoint_stale` または `service_endpoint_fallback_not_allowed` |
| node-local state unavailable | `node_state_unavailable` |
| failover state not proven | `failover_not_proven` |
| worker execution が probe に影響する task supervision/spawn/join/cancel failure | runtime task reason |
| service-to-service identity が probe に影響する internal service identity/trust failure | internal service identity reason |

### A-6 Evidence Rule

health/readiness/liveness evidence は次を記録しなければなりません: command または probe endpoint; working directory または target entrypoint; startup run ID; command-scoped 時 correlation ID; probe class; included/excluded checks; 関連時 topology class と node scope; 影響時 service discovery source/resolution state; 影響時 distributed state class と failover status; 影響時 runtime task class/supervision state; 影響時 internal service trust class; expected outcome; actual outcome; non-success の cataloged reason; close-not-claimed scope。これら field を欠く probe output は diagnostic only です。

---

## B 部 Cross-Plane Shutdown / Drain

本部は Signaling / SFU / TURN / drivers / entrypoints をまたぐ shutdown and drain 境界です。process lifecycle、room drain、SFU session drain、TURN relay stop、driver I/O stop、audit/persistence flush を混同しないため owner と順序を固定します。crash / panic / supervisor restart classification は本章 C 部に従い、graceful drain とは別の evidence class として扱います。

### B-1 境界

| 関心事 | Owner | Rule |
|---|---|---|
| process signal observation | entrypoints | OS/process/runtime signal を受ける |
| split-service shutdown control | internal control-plane contract | service-to-service drain command/event boundary |
| shutdown mode selection | entrypoints | typed shutdown request と selected drivers への wiring |
| room drain/close semantics | core/signaling | room state transition と rejection reason |
| SFU session/endpoint lifecycle semantics | core/sfu | session drain、endpoint close、forwarding stop decision |
| TURN allocation/permission relay semantics | core/turn | lifecycle event と relay authorization semantics |
| socket/listener stop | driver | concrete I/O stop と receive loop closure |
| runtime task/worker stop | driver/entrypoints runtime | task lifecycle 規範経由の bounded join/cancel |
| packet/buffer release | driver | lease/queue/cache release |
| audit/persistence/metrics flush execution | driver | bounded flush/retry と failure mapping |
| unclean process termination observation | entrypoints/driver observation | graceful drain success へ変換しない |

entrypoints は shutdown order を開始できるが、domain state transition の正を所有しません。driver は I/O を止められるが、accepted/rejected/drained の domain meaning を所有しません。

### B-2 Drain Sequence（原則順序）

1. entrypoints が shutdown correlation と typed shutdown request を作成する。
2. entrypoints が listener/connection boundary で new external admission を停止する。
3. core/signaling が room drain を開始するか、unavailable room commands を cataloged reason で reject する。
4. core/sfu が sessions/endpoints を drain し、forwarding closure/suppression decisions を返す。
5. core/turn が、既存 TURN lifecycle decisions または driver shutdown conversion により new allocation/permission paths を停止する。
6. driver が receive loops を停止し、new buffer leases を防ぐ。
7. driver/entrypoints が admitted supervision scopes 内で runtime tasks を cancel または join する。
8. driver が bounded transmit/relay/audit/persistence queues を resource bounds に従い drain する。
9. driver が packet buffers、packet cache、TURN relay resources、network sockets を release する。
10. entrypoints が、mandatory audit/bootstrap evidence path を試行した後に runtime を停止する。

later step が失敗した場合、earlier accepted domain decisions を failed step の success として reclassify してはなりません。

### B-3 Plane-Specific Rules

| Plane | Drain behavior | Required failure relation |
|---|---|---|
| Signaling | new join rejection 前に room が `room_draining` に入る | `room_draining`, `room_closed`, `room_close_not_allowed` |
| SFU | session/endpoint stop は core decision、driver が queue/cache release を実行 | `sfu_session_not_accepting`, `endpoint_closed_by_backpressure`, 該当時 `driver_shutdown` |
| TURN | active relay path は lifecycle/release または driver shutdown conversion で停止 | `driver_shutdown`、既存 TURN forbidden/expired/resource reasons |
| Network driver | listener/read/write loops を停止 | `driver_shutdown`, `network_receive_failed`, `network_send_failed` |
| Persistence/audit/metrics driver | bounded flush/retry のみ | `persistence_unavailable`, `audit_backlog_bound_exceeded`, `metrics_export_failed`, `driver_shutdown` |

新 plane-specific shutdown reason は core reason catalog の update が必須です。

### B-4 Admission Stop Rule

admission stop は driver process death とは異なります。entrypoints が drain を開始したとき、driver は configured shutdown mode が要求する限り速やかに new external sessions の accept を停止しなければなりません。既に materialized な core state は、core state が存在する場合 core が所有する state machine を通じて close しなければなりません。admission stop 後の pre-core external inputs は `driver_shutdown` または connection-level close mapping で失敗しえます。

### B-5 Unclean Termination Rule

panic、crash、forced kill、supervisor restart は graceful shutdown sequence の後続 step として扱いません。その観測は process lifecycle observation として分類し、既存の accepted domain decisions を retroactive success / failure に書き換えてはなりません。unclean termination 後の restore / replay eligibility は durable recovery 規範に従います。

### B-6 Evidence Rule

shutdown/drain evidence は次を記録しなければなりません: shutdown correlation ID; 利用可能時 startup run ID; drain mode; runtime reconfiguration 起因時 reconfiguration generation; affected plane; accepted/rejected/failed decision; non-success の cataloged reason; bounded flush/retry result; worker execution が drain に影響する時 runtime task/worker join or cancellation result; あれば unreleased resource count。shutdown evidence の欠落は graceful shutdown の proof として用いてはなりません。

---

## C 部 Crash / Panic / Supervisor Restart Classification

本部は crash、panic、unclean shutdown、supervisor restart、process restart の境界です。graceful shutdown/drain、durable recovery、liveness/readiness と unclean process failure を混同しないため、分類と証跡受理条件を固定します。unclean restart は graceful drain ではありません。supervisor restart は recovery success ではありません。

### C-1 境界

| 関心事 | Owner | Rule |
|---|---|---|
| process crash observation | entrypoints/supervisor integration | process lifecycle observation のみ |
| panic classification | entrypoints/driver runtime boundary | domain decision ではない |
| task panic classification | runtime task lifecycle boundary | process crash と同一視しない |
| domain state after crash | restore policy が適用される場合のみ core | silent restore なし |
| driver resource cleanup after crash | driver/entrypoint runtime | best effort; evidence required |
| supervisor restart | external supervisor/entrypoints observation | readiness proof ではない |
| recovery/restore | 定義に従い core/drivers/entrypoints | durable recovery 規範に従う |

### C-2 Failure Classes（閉集合）

| Class | 意味 | Claim limit |
|---|---|---|
| `panic_observed` | runtime/application panic が観測 | graceful shutdown ではない |
| `task_panic_observed` | runtime task/worker panic が観測 | domain transition または process recovery ではない |
| `process_crash_observed` | process が予期せず exit | domain close ではない |
| `unclean_shutdown_detected` | shutdown が drain/audit completion evidence を欠く | graceful drain ではない |
| `supervisor_restart_observed` | external supervisor が process を restart | readiness または restore success ではない |
| `startup_after_unclean_exit` | prior unclean exit 後に entrypoint が start | state claims 前に restore policy が必須 |
| `crash_recovery_evidence` | controlled crash/restart test report | evidence class は tested scope に限定 |

新 process failure class は v0.2 初期 scope 外です。

### C-3 Classification Rule

report または observation は次を classify しなければなりません: 利用可能時 restart 前後の startup run ID; process identity; failure class; prior drain status; audit persistence status; restore/replay policy の適用有無; restart 後の readiness class; close-not-claimed scope。prior drain status または audit status が absent の場合、その restart を closeout claims に対し unclean として扱わなければなりません。

### C-4 Failure Mapping

| 失敗 | 必須 reason |
|---|---|
| process panic observed | `process_panic_detected` |
| process crash 無しの task panic observed | `runtime_task_panic_detected` |
| process crash observed | `process_crash_detected` |
| unclean shutdown detected | `unclean_shutdown_detected` |
| supervisor restart observed | `supervisor_restart_observed` |
| restart 後 restore not allowed | durable recovery 規範の restore-specific reason |
| crash handling 中の driver shutdown | `driver_shutdown` |

### C-5 Evidence Rule

crash/restart evidence は次を記録しない限り採用してはなりません: exact command/procedure; supervisor または process runner; expected failure class; observed failure class; startup run IDs; logs は diagnostic support のみ; audit/restore/readiness status は別 evidence class として。restart 後の process uptime は restored domain state の proof ではありません。

---

## D 部 Operator / Admin Authorization

本部は operator / admin authorization の境界です。operator-facing CLI、admin endpoint、maintenance action、probe action、evidence verification command が generic communication authorization、domain decision、runtime readiness と混同されないよう固定します。communication participant authorization は operator/admin action を authorize しません。operator/admin authorization は participant join/publication/subscription/TURN relay を authorize しません。

### D-1 境界

| 関心事 | Owner | Rule |
|---|---|---|
| operator credential source | external system または entrypoints/drivers | raw credential は core identity ではない |
| operator credential verification | entrypoints/drivers + security verifier boundary | typed result のみ |
| admin authorization context mapping | core/admin policy input 前の entrypoints/drivers | opaque operator/admin context |
| admin action policy | action class に応じ core/admin policy または entrypoint boundary policy | explicit action/scope required |
| maintenance action execution | entrypoints が core use case または driver operation を invoke | allowed boundary 外の domain mutation なし |
| audit evidence | core event model と driver sink | operator/admin authorization decision |

### D-2 Operator Admin Classes（閉集合）

| Class | 意味 | Rule |
|---|---|---|
| `operator_probe_context` | operator が diagnostic probe を request してよい | domain acceptance ではない |
| `operator_admin_action_context` | operator が admin action を request してよい | action/scope required |
| `maintenance_action_context` | operator が drain/maintenance mode を request してよい | health/admin と shutdown 規範に従う |
| `evidence_verification_context` | operator が evidence/hash-chain/report を verify してよい | 単独で evidence を作らない |
| `developer_local_context` | local development helper context | production/operator evidence ではない |

新 operator/admin class は v0.2 初期 scope 外です。

### D-3 Authorization Rule

operator/admin authorization は次を declare しなければなりません: operator/admin class; credential/context source; allowed action class; allowed target scope; lifetime と expiry; redaction rule; target command boundary; audit event relation; failure reason mapping。admin action は、なお allowed boundary を通じて target core use case または driver operation を呼ばなければなりません。

### D-4 Failure Mapping

| 失敗 | 必須 reason |
|---|---|
| operator credential missing | `operator_credential_missing` |
| operator credential invalid | `operator_credential_invalid` |
| operator authorization context missing | `operator_authorization_context_missing` |
| operator authorization context expired | `operator_authorization_context_expired` |
| operator action が policy により denied | `operator_action_denied` |
| operator target scope not allowed | `operator_scope_not_allowed` |
| admin action が boundary/policy により not allowed | `admin_action_not_allowed` |
| maintenance mode が action をブロック | `maintenance_mode_active` |
| runtime configuration missing | `runtime_config_missing` |
| runtime configuration invalid | `runtime_config_invalid` |

### D-5 Audit / Evidence Rule

operator/admin authorization decisions は audit event type `operator_admin_authorization_decision` を用います。target admin/maintenance actions は、なお `admin_maintenance_decision` または target domain event type を用います。同一 correlation chain は両 event を含んでよいが、一方が他方を置き換えません。

operator/admin evidence は次を記録しなければなりません: operator/admin class; action class; target scope; raw credential なしの credential/context class; 関連時 lifetime/expiry observation; audit event type `operator_admin_authorization_decision`; action execution を含む claim 時 target action event; expected outcome; actual outcome; non-success の cataloged reason; close-not-claimed scope。CLI exit code 単独は diagnostic only です。

---

## E 部 横断 禁止事項（Prohibitions）

- listener bind success を full readiness として report する。
- liveness success を domain acceptance として report する。
- admin command が core use case 外で state を mutate する。
- admin command が operator/admin authorization evidence なしに execute する。
- maintenance mode が driver-local flag のみで表現される。
- probe response が failed dependency を隠す。
- readiness output が reports なしに production evidence として用いられる。
- single-node readiness を multi-node readiness として用いる。
- public endpoint connection success を target contract evidence なしに domain readiness として扱う。
- service discovery success を target dependency/probe evidence なしに readiness として扱う。
- failover が replacement endpoint reachability 単独で healthy として扱われる。
- worker task が spawn success 単独で healthy として扱われる。
- internal service trust が endpoint resolution または TLS listener startup 単独で healthy として扱われる。
- entrypoints が core decision なしに room/SFU/TURN state を closed に直接 mutate する。
- driver が socket close を successful domain leave/close として扱う。
- shutdown failure が process exit code のみの背後に隠される。
- crash / panic / supervisor restart observation が graceful drain success として report される。
- unbounded drain wait または unbounded flush queue が許可される。
- failed audit/persistence flush が closeout evidence として用いられる。
- ある plane の successful drain が別 plane の successful drain を含意する。
- split-service drain control が internal control-plane correlation/audit evidence を欠く。
- runtime reconfiguration が required drain/restart 前に apply される。
- detached または unjoined worker が graceful drain claim 中に残る。
- unclean crash が graceful shutdown として report される。
- supervisor restart が readiness として report される。
- crash recovery success が restore/replay evidence なしに推論される。
- panic log text が authoritative reason になる。
- restarted process が restore policy なしに previous domain state を reuse する。
- crash evidence が prior audit/drain status を省略する。
- communication participant token が default で operator/admin action を authorize する。
- operator/admin authorization が localhost / network reachability / deployment environment から推論される。
- raw operator credential が audit/log/report に現れる。
- admin UI または CLI response が domain decision evidence になる。
- developer-local context が production operator proof として用いられる。

## F 部 横断 Collapse Conditions（判断が崩れる条件）

- readiness が単一の unqualified boolean。
- operator action が core/drivers/entrypoints ownership を bypass する。
- operator action authorization が endpoint reachability から推論される。
- health probe が sensitive raw data を expose する。
- maintenance status が core transition なしに domain state を変える。
- probe success が evidence class 外で build/test/runtime proof として用いられる。
- topology class が operational readiness evidence で隠される。
- public endpoint lifecycle state が endpoint behavior を target claim とするとき隠される。
- process lifecycle が domain state authority として扱われる。
- driver I/O closure が core decision なしに accepted room/SFU/TURN transition として report される。
- drain waits または queues が unbounded。
- shutdown が correlation と plane-specific evidence を欠く。
- graceful shutdown success が process exit 単独から推論される。
- unclean termination evidence が graceful drain evidence と mix される。
- reconfiguration-driven drain が generation と apply-scope evidence を欠く。
- process failure class が absent。
- restart が domain recovery proof として扱われる。
- unclean shutdown が durable recovery policy を bypass する。
- panic/crash observation が core state を mutate する。
- task panic が process readiness または generic driver failure のみとして report される。
- crash report が startup run boundary を欠く。
- operator/admin action に explicit authorization context が無い。
- target action scope が implicit。
- operator authorization と communication authorization が同一 decision として扱われる。
- admin action execution が target boundary evidence を欠く。
- operator/admin denial が free-text のみで記録される。

## G 部 不変条件（Invariants）の要約

- readiness は閉集合の probe class と included/excluded checks を明示する合成であり、単一 boolean ではない。
- process lifecycle（liveness/shutdown/crash/restart）は domain lifecycle と分離され、互いを代替しない。
- domain state の close は core が所有する state machine を通じてのみ行い、driver I/O closure や process exit はそれを証明しない。
- drain と flush は常に bounded であり、unclean termination は graceful drain と別 evidence class。
- operator/admin authorization は communication authorization と独立であり、reachability や localhost から推論されない。
- raw credential / secret / sensitive data は audit/log/report/probe に現れない。
