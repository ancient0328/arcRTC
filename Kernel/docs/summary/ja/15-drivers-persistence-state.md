# drivers-persistence-state

状態: SSOT 統合版
日付: 2026-06-28 JST

## 目的

本章は arcRTC v0.2 Kernel の persistence / state driver family の現行完全仕様を、本章のみで再現実装可能な粒度で内在化することを目的とします。本章は persistence 境界（PostgreSQL/Redis/S3/filesystem/in-memory）、state persistence policy（state class と checkpoint/audit-only/driver-local の切り分け）、durable recovery / restore / replay の全 recovery class・state family policy・restore precondition・replay rule・failure mapping、schema / migration lifecycle、export / backup artifact 境界、distributed state / replication / failover policy を所有者・閉集合語彙・状態機械・failure mapping・禁止/許可・fail-closed 条件まで落とさずに内在化します。

依存方向の表記: `A <- B` は「B が A に依存」を意味します。persistence は core が所有する port contract と driver が所有する storage implementation に分離します。driver は core-owned port を実装し、external storage 型は driver 境界で停止します。`drivers -> entrypoints`、`drivers -> regulated` は禁止です。v0.2 initial architecture は room / SFU route / TURN allocation state に対する implicit durable domain source-of-truth を持ちません。本章は実装済み recovery behavior、runtime readiness、production readiness、replication engine、consensus、leader election、automatic failover の成立を主張しません。

---

## 1. Persistence Boundary（`drivers/persistence`）

### 1.1 Ownership（所有割当、閉集合）

| 領域 | 所有者 | 規則 |
|---|---|---|
| PersistencePort contract | core | state / checkpoint / audit persistence intent |
| persistence consistency requirement | core | idempotency, ordering, retention policy intent |
| concrete DB / object store / filesystem | driver | PostgreSQL, Redis, S3, filesystem, memory |
| schema / migration file | driver | storage implementation detail |
| retry store execution | driver | bounded by core/driver policy mapping |
| entrypoint selection of store | entrypoints | typed configuration and wiring only |

### 1.2 Persistence Surfaces（閉集合）

v0.2 initial architecture で persistence として扱う surface は次に限定します。新規 persistence surface は v0.2 初期 scope 外です。

| Surface | Core contract | Driver implementation |
|---|---|---|
| domain state checkpoint | checkpoint intent and version | DB / memory / file |
| audit event persistence | ordered audit record intent | file / HTTP / syslog / DB / S3 |
| audit hash-chain record | chain scope and sequence semantics | storage and export |
| retry store | retry policy intent and closed bounds | queue / durable store |

### 1.3 Atomicity Relation（atomicity 関係）

persistence driver は concrete transaction / durability mechanism を提供してよい。core command atomicity、commit boundary、compensation、結果が evidence として受理される時点は storage implementation の外で定義します。command path が core decision と persistence/audit side effect の両方を要する場合、per-step outcome は atomicity 規則に従って報告します。

### 1.4 Storage Shape Rule（storage shape 規則）

core は SQL row shape、table name、S3 key layout、filesystem path、Redis key、migration file を domain API として露出してはなりません（禁止）。driver はこれらを implementation detail としてのみ定義してよい。persisted representation が public contract / shared artifact になる場合は、当該 protocol 規則または export / backup artifact 規則（第5節）を要します。schema migration success は domain restore success を証明しません。domain restore/replay evidence は durable recovery 規則（第3節）の下で別個に評価します。replication / consensus / leader election / failover の主張は distributed state 規則（第6節）の evidence を要し、storage availability / transaction success はそれを証明しません。

### 1.5 Failure and Retry Rule（閉集合）

persistence retry bounds と action は resource bounds / backpressure 規則に従います。persistence driver failure は successful domain decision evidence へ silently に downgrade してはなりません（禁止）。persistence transaction success は failed audit / external response / compensation step を消去してはなりません。

| Failure | Required reason |
|---|---|
| concrete persistence unavailable | `persistence_unavailable` |
| retry entry count / byte / retry-count bound reached | `persistence_retry_bound_exceeded` |
| retry duration exceeded | `persistence_retry_duration_exceeded` |
| driver shutdown | `driver_shutdown` |

### 1.6 Audit Relation（audit 関係）

audit event meaning は audit event 規則が、audit hash-chain semantics は audit hash-chain 規則が所有します。persistence driver は audit record を store / export / retry してよいが、audit event meaning や chain validity semantics を所有しません。export / backup された persistence artifact は export / backup artifact 規則の下で artifact class / redaction class / retention class / integrity class を要します。選択された audit persistence が required failure を記録できず bootstrap audit record path も無い場合、その startup / decision path は close / complete / ready evidence として使用してはなりません。

### 1.7 Prohibitions（禁止、閉集合）

- core が `sqlx`、PostgreSQL、Redis、S3、filesystem の concrete API を import する。
- driver schema が domain invariant を決める。
- persistence driver が Signaling / SFU / TURN command を accept / reject する。
- persistence failure が successful closeout claim の背後に隠れる。
- core decision と atomicity evidence なしに storage transaction success が domain commit success として報告される。
- retry store が unbounded である。
- migration file が domain model source of truth として扱われる。
- restore/replay qualification なしに backup/export artifact が restore success evidence として使用される。
- distributed state 規則なしに storage replication feature が domain replication / failover success として扱われる。

### 1.8 Persistence Boundary Collapse Conditions（崩壊条件）

storage schema が core domain API になる。driver persistence が state transition semantics を所有する。persistence retry queue が unbounded または unaudited である。audit hash-chain meaning が storage implementation へ委譲される。write failure が記録も bound もされなかった persistence output に closeout evidence が依存する。persistence implementation が compensation / aggregate commit semantics の owner になる。persisted artifact が export/backup artifact classification なしに storage boundary を離れる。persistence backend replication が domain state ownership proof として使用される。

---

## 2. State Persistence Policy

### 2.1 State Classes（閉集合）

v0.2 initial architecture の state class は次に限定します。新規 state class は v0.2 初期 scope 外です。

| Class | Meaning | Persistence rule |
|---|---|---|
| `ephemeral-core-state` | runtime memory 上の domain state | persistence required ではない |
| `checkpoint-eligible-state` | restart/recovery のため保存可能な core-owned snapshot intent | PersistencePort 経由でのみ保存 |
| `audit-only-state` | decision/evidence として audit event / hash-chain に残す | AuditSinkPort / hash-chain record 経由 |
| `driver-local-state` | socket, buffer, retry queue, external client/session detail | driver-owned, core source-of-truth ではない |
| `configuration-scope-state` | startup/wiring/config validation scope | configuration decision and startup references |
| `sdk-local-state` | SDK connection/client state | SDK-owned, server/core state ではない |

### 2.2 Aggregate State Policy（aggregate ごとの state class、閉集合）

| State family | Class | Persistence rule |
|---|---|---|
| Signaling room state | `ephemeral-core-state`; checkpoint only by explicit policy | no implicit durable room source-of-truth |
| Signaling participant state | `ephemeral-core-state`; checkpoint only by explicit policy | no driver-owned membership source-of-truth |
| Signaling idempotency state | `checkpoint-eligible-state` when configured | semantics core-owned, storage driver-owned |
| SFU session / endpoint / publication / subscription / route state | `ephemeral-core-state` | forwarding state is not durable source-of-truth in initial v0.2 |
| TURN allocation / permission / channel bind state | `ephemeral-core-state`; audit lifecycle events required | relay authorization is not silently restored without explicit policy |
| Audit event | `audit-only-state` | audit event canonical and hash-chain canonical apply |
| Audit hash-chain record | `audit-only-state` | chain scope and sequence semantics core-owned |
| Atomicity/compensation evidence | `audit-only-state` or `configuration-scope-state` per command path | evidence of per-step outcomes, not driver transaction authority |
| Resource bound counters | `ephemeral-core-state` or `driver-local-state` per owner mapping | persistence does not redefine bound policy |
| Driver retry store | `driver-local-state` | bounded by resource policy; not domain source-of-truth |
| Configuration decision | `configuration-scope-state` | startup evidence only, not runtime domain state |
| SDK connection state | `sdk-local-state` | not server/core source-of-truth |

### 2.3 Source-of-Truth Rule

initial v0.2 は room / SFU route / TURN allocation state に対する implicit durable domain source-of-truth を持ちません。durable recovery / restore / replay behavior は durable recovery 規則（第3節）が支配します。persistence classification は replication / consensus / failover を admit しません。durable recovery semantics をその規則を超えて拡張する将来の scope 拡張は、次を定義しなければなりません（必須）。persisted state owner、snapshot shape、restore preconditions、replay/audit relation、conflict resolution、複数 node 関与時の distributed state owner/scope、failure reason mapping、verification evidence。

### 2.4 Checkpoint Rule

checkpoint は optimization / recovery support であり自動的に domain source of truth ではありません。core は checkpoint intent と version を所有します。driver は concrete schema / table / key / object / file layout を所有します。driver DB transaction success は driver-local durability を支えてよいが、atomicity 規則と state machine が許す場合を除き aggregate commit semantics を定義しません。checkpoint restore は restore policy が無い限り room / participant / route / allocation / permission / channel bind state を作成してはなりません（禁止）。checkpoint persistence は distributed state 規則が当該 state class を admit しない限り replication / failover proof として使用してはなりません。

### 2.5 Audit Relation

audit event / hash-chain record は evidence かつ tamper-evidence material であり、mutable domain state store ではありません。audit replay は、別個の replay/restore 規則が ordering / gap handling / conflict rule を定義する場合のみ verification に使用してよい。初期 replay/restore owner boundary は durable recovery 規則であり、そこに state-family permission が無い限り replay は verification-only に留まります。

### 2.6 State Policy Failure Rule（閉集合）

persistence failure は state recovery / closeout evidence / runtime readiness を主張しながら隠してはなりません（禁止）。

| Failure | Required reason |
|---|---|
| persistence unavailable | `persistence_unavailable` |
| retry store bound exceeded | `persistence_retry_bound_exceeded` |
| retry duration exceeded | `persistence_retry_duration_exceeded` |
| audit backlog exceeded | `audit_backlog_bound_exceeded` |
| driver shutdown | `driver_shutdown` |

### 2.7 Prohibitions（禁止、閉集合）

- driver persistence schema が domain source-of-truth になる。
- checkpoint restore が restore policy なしに domain state を作成する。
- SFU route state が default で durable である。
- TURN allocation が driver storage から silently に restore される。
- audit log が mutable state store として扱われる。
- driver DB transaction が aggregate commit authority として扱われる。
- SDK local connection state が server-side participant state として扱われる。
- driver retry queue が unbounded または domain state として使用される。
- persisted state が distributed state policy なしに replicated / failover-ready として扱われる。

### 2.8 State Persistence Policy Collapse Conditions（崩壊条件）

persisted schema が aggregate invariant を定義する。restart/recovery behavior が restore policy なしに主張される。audit-only state が domain state として mutate される。driver-local state が core source-of-truth として使用される。persistence state class が distributed state / failover admission を迂回するために使用される。v0.2 closeout が未仕様・未検証の persistence recovery に依存する。commit boundary evidence なしに persistence transaction success が domain atomicity proof として使用される。

---

## 3. Durable Recovery / Restore / Replay

### 3.1 Boundary（所有割当、閉集合）

| Concern | Owner | Rule |
|---|---|---|
| restore eligibility policy | core | state family ごとに明示されたものだけ復元候補になる |
| restore precondition | core | version、ordering、conflict、bound を検査する |
| snapshot / checkpoint intent | core | schema-independent checkpoint version と semantic shape |
| concrete snapshot schema | driver | DB row、object key、file layout、migration detail |
| audit replay verification | core | audit event ordering と hash-chain verification semantics |
| replay execution storage read | driver | bounded read、pagination、I/O failure handling |
| recovery mode selection | entrypoints | typed configuration と selected driver wiring only |
| crash/restart trigger classification | entrypoints/driver observation converted to core-owned evidence class | restore eligibility を自動成立させない |
| distributed failover claim | distributed state policy + recovery evidence | restore success and failover success are separate |

driver が保存した data はそれだけでは domain source-of-truth ではありません。entrypoints が recovery mode を選んでも、core restore precondition を満たさない state は復元してはなりません（禁止）。

### 3.2 Recovery Classes（閉集合）

v0.2 initial architecture の recovery class は次に限定します。新規 recovery class は v0.2 初期 scope 外です。

| Recovery class | Meaning | Permitted use |
|---|---|---|
| `no_restore` | restart 後に state を復元しない | SFU route、TURN active allocation、driver-local state の初期既定 |
| `checkpoint_restore_candidate` | core-owned checkpoint intent から復元候補にできる | idempotency state など、明示 policy がある state |
| `audit_replay_verification_only` | replay は検証に限定し、domain state を mutation しない | audit event / hash-chain consistency verification |
| `driver_retry_recovery` | driver-owned retry queue を bounded に再開する | persistence retry、audit sink retry など |

### 3.3 State Family Policy（閉集合）

| State family | Recovery class | Restore rule |
|---|---|---|
| Signaling room state | `no_restore` by default | explicit restore policy がない限り restart 後に room を再作成しない |
| Signaling participant state | `no_restore` by default | connection observation なしに participant を joined としない |
| Signaling idempotency state | `checkpoint_restore_candidate` when configured | checkpoint version、command identity、ordering window が一致する場合だけ候補 |
| SFU endpoint / route / publication / subscription state | `no_restore` | packet route / subscription state を durable source-of-truth にしない |
| TURN allocation / permission / channel bind state | `no_restore` | socket path / credential / lifetime を silent restore しない |
| Audit event / hash-chain | `audit_replay_verification_only` | replay は verification であり domain state mutation ではない |
| Persistence retry store | `driver_retry_recovery` | retry bound と duration bound の範囲内だけ再開 |
| Metrics backlog | `driver_retry_recovery` when configured | metrics export retry は domain evidence ではない |
| SDK local state | `no_restore` for server/core | server-side participant state として扱わない |

### 3.4 Restore Preconditions（復元前提、すべて必須）

restore attempt は次をすべて満たす場合だけ core state mutation の候補になります。1つでも満たさない場合、restore は fail-closed で拒否されます。

- triggering lifecycle observation is classified when restore follows crash, panic, unclean shutdown, or supervisor restart;
- restore policy が state family を許可している。
- snapshot/checkpoint version が accepted version として評価されている。
- correlation chain、state owner、aggregate reference が core-owned reference として検証されている。
- audit hash-chain relation が必要な state では gap がない。
- checkpoint と replay result が conflict しない。
- resource bound と lifetime bound が restore 時点でも満たされている。
- driver read failure、schema mismatch、migration failure が cataloged reason へ接続されている。
- distributed state/failover policy が必要な state family では owner node、replacement owner、conflict rule が記録されている。

### 3.5 Replay Rule（replay 規則）

audit replay は audit integrity verification であり、domain mutation の既定経路ではありません。replay で domain state を復元するには、state family ごとの restore policy が次を定義している必要があります（必須）。replay 対象 event type、replay ordering、gap handling、duplicate handling、conflict resolution、lifetime / resource bound revalidation、replay failure reason mapping、replay result を closeout evidence に採用できる条件。これらの定義が存在しない場合、audit replay result は verification observation に限定します。

### 3.6 Failure Mapping（閉集合）

restore failure は、選択された audit path が利用可能なとき audit/event evidence を通じて記録します。audit recording 自体が利用不能な場合、当該 restore attempt は close / complete / ready evidence として使用してはなりません。

| Failure | Required reason |
|---|---|
| persistence read unavailable | `persistence_unavailable` |
| persisted representation cannot map to core type | `external_decode_failed` |
| persisted schema or version unsupported by driver encoding | `unsupported_driver_wire_version` |
| required persisted field absent | `missing_required_wire_field` |
| persisted enum has no core mapping | `external_enum_unmapped` |
| restore target state already conflicts with current state | `command_order_violation` or state-specific forbidden reason |
| retry store bound exceeded during recovery | `persistence_retry_bound_exceeded` |
| retry duration exceeded during recovery | `persistence_retry_duration_exceeded` |
| driver shutdown during restore/replay | `driver_shutdown` |
| restart/crash observation indicates unclean source state | `unclean_shutdown_detected` or process lifecycle reason |
| failover is claimed without distributed state policy/evidence | `failover_not_proven` |
| state owner conflict is detected during recovery | `state_owner_conflict` |

### 3.7 Prohibitions（禁止、閉集合）

- DB row / object key / file path / migration file が domain source-of-truth になる。
- audit replay が explicit restore policy なしに domain state を mutate する。
- SFU route state が silently に restore される。
- TURN allocation / permission / channel bind が silently に restore される。
- SDK local reconnect state が server participant state として扱われる。
- restore conflict が driver-local preference で解決される。
- checkpoint/replay evidence と correlation ID なしに restore success が主張される。
- crash/restart observation が restore success として扱われる。
- service discovery fallback または replacement process start が failover success として扱われる。
- replicated/consensus state が restore / checkpoint behavior から推測される。

### 3.8 Durable Recovery Collapse Conditions（崩壊条件）

restore policy の無い state family が restore される。driver schema が aggregate invariant または conflict resolution を決める。audit replay が mutable domain state store として使用される。restart behavior が restore report evidence なしに検証済みとして扱われる。restore 時に resource/lifetime bounds が再検査されない。crash/restart 駆動の recovery claim に process lifecycle classification が欠ける。failover recovery claim に distributed owner/scope/conflict rule が欠ける。

---

## 4. Schema / Migration Lifecycle

### 4.1 Boundary（所有割当、閉集合）

| Concern | Owner | Rule |
|---|---|---|
| PersistencePort contract | core | schema-independent intent と version |
| persisted state semantic version | core | checkpoint/retry/audit intent の accepted version |
| concrete schema/table/key/file layout | driver | storage implementation detail |
| migration file | driver | concrete storage evolution detail |
| migration execution selection | entrypoints | startup/wiring configuration only |
| migration evidence report | dated evidence record | command/result/correlation/reproducible procedure |

driver schema は aggregate invariant / state transition / reason semantics を所有しません。core は SQL table / object key / filesystem path / migration file name に依存しません。persisted representation の canonical serialization が digest / compatibility evidence に必要な場合は canonical serialization 規則に、migration snapshot / exported schema support artifact は export / backup artifact 規則（第5節）に従います。

### 4.2 Migration Classes（閉集合）

v0.2 initial architecture の migration class は次に限定します。新規 migration class は v0.2 初期 scope 外です。

| Migration class | Meaning | Rule |
|---|---|---|
| `driver_schema_init` | new store initialization | startup evidence が必要 |
| `driver_schema_forward` | concrete storage shape の前進変更 | compatibility / rollback plan が必要 |
| `driver_schema_rollback` | concrete storage shape の戻し | data loss condition を明示 |
| `driver_encoding_compat` | persisted representation decode compatibility | accepted version と failure reason を明示 |
| `no_migration_required` | schema unchanged | report may state skipped with reason |

### 4.3 Lifecycle Rule（順序固定）

schema migration lifecycle は次の順序で扱います。

1. the core contract defines persistence surface and owner.
2. driver defines concrete schema/migration implementation.
3. entrypoints wires migration mode and selected driver.
4. migration dry-run or validation command is executed where available.
5. migration execution result is recorded as a dated evidence record.
6. restore/replay/checkpoint behavior is evaluated by its own policy, not by schema existence.

schema migration success は domain restore success を証明しません。domain restore success は durable recovery 規則（第3節）を要します。

### 4.4 Compatibility Rule（互換性規則、必須項目）

persisted representation compatibility は次を定義しなければなりません。accepted representation versions、rejected representation versions、representation が digest / compatibility evidence に使われる場合の canonical format/version、required fields、newly optional field の default behavior、unknown field handling の explicit rule、missing field reason mapping、rollback condition、data loss condition。unsupported または unmappable な persisted representation は catalog 済み driver conversion / persistence reason で fail closed しなければなりません。

### 4.5 Failure Mapping（閉集合）

migration failure は startup success / closeout evidence の背後に隠してはなりません（禁止）。

| Failure | Required reason |
|---|---|
| persistence store unavailable during migration | `persistence_unavailable` |
| runtime migration configuration missing | `runtime_config_missing` |
| runtime migration configuration invalid | `runtime_config_invalid` |
| persisted representation cannot decode | `external_decode_failed` |
| persisted representation version unsupported | `unsupported_driver_wire_version` |
| required persisted field absent | `missing_required_wire_field` |
| persisted enum cannot map to core reference | `external_enum_unmapped` |
| retry bound exceeded during migration/retry | `persistence_retry_bound_exceeded` |
| retry duration exceeded | `persistence_retry_duration_exceeded` |
| driver shutdown | `driver_shutdown` |

### 4.6 Report Rule（migration report 必須項目）

migration report は次を含まなければなりません。correlation ID または startup run ID、target driver and store class、migration class、source schema/encoding version、target schema/encoding version、digest / compatibility evidence を主張する場合の canonical format/version、command or procedure、result、non-success の cataloged reason、rollback status または rollback 非適用理由、sensitive-data redaction statement。reproducible procedure または correlation/startup reference を欠く report は migration evidence ではありません。

### 4.7 Prohibitions（禁止、閉集合）

- migration file が domain invariant を定義する。
- schema existence が restore/replay success として扱われる。
- core が DB migration library を import する。
- driver migration failure が silently に skip される。
- incompatible persisted representation が best-effort で decode される。
- storage row/object layout が explicit rule なしに canonical serialization として扱われる。
- durable data に影響し得る forward migration で rollback plan が省略される。
- migration snapshot が artifact classification なしに backup/restore evidence として採用される。

### 4.8 Schema / Migration Collapse Conditions（崩壊条件）

schema/table/key layout が core API になる。migration success が domain recovery proof として使用される。incompatible stored data が cataloged reason なしに受理される。migration report が correlation/startup reference または reproducible procedure を欠く。migration mode が entrypoints typed configuration なしに driver によって選択される。migration compatibility evidence が canonical representation rule を欠く。exported migration artifact が redaction / retention / integrity classification を欠く。

---

## 5. Export / Backup Artifact 境界

### 5.1 Boundary（所有割当、閉集合）

export / backup artifact は storage / audit / configuration / evidence / checkpoint / schema support から生成される共有可能または保管可能な成果物です。artifact の concrete format / path / bucket key / database dump shape は driver / tooling detail であり core semantic authority ではありません。release/distribution 用 artifact は release artifact distribution provenance 規則に従います。

| Concern | Owner | Rule |
|---|---|---|
| artifact semantic class | core contract | closed class required |
| artifact generation execution | driver / tooling / entrypoints | concrete file, object, dump, archive |
| redaction and retention policy | privacy policy | raw sensitive material is not admitted |
| hash/digest/integrity | canonical serialization / audit hash-chain policy | concrete export format is not automatically canonical |
| restore/replay semantics | durable recovery / schema migration policy | backup presence does not prove restore success |
| public sharing decision | governance / target policy | artifact must not become public API by accident |

### 5.2 Artifact Classes（閉集合）

v0.2 initial architecture の export / backup artifact class は次に限定します。新規 artifact class は v0.2 初期 scope 外です。

| Artifact class | Meaning | 所有 |
|---|---|---|
| `audit_event_export` | redacted audit event export or chain segment | audit event 規則 and audit hash-chain 規則 |
| `state_checkpoint_backup` | domain checkpoint backup for recovery qualification | state persistence policy 規則 and durable recovery 規則 |
| `persistence_snapshot` | driver-local DB/object/filesystem snapshot | persistence boundary 規則 |
| `configuration_export` | typed configuration/profile/policy bundle export | configuration 規則 |
| `evidence_bundle` | rerunnable report-support material | evidence report 規則 |
| `schema_migration_snapshot` | schema/migration support snapshot | schema migration 規則 |

### 5.3 Artifact Admission Rule（採用条件、必須項目）

export / backup artifact が evidence として採用されるのは、次を記録する場合のみです。artifact class、generation command or procedure、working directory or driver/tool owner、source scope and time phase、correlation ID、redaction class、retention class、integrity digest または explicit non-digest reason、applicable な schema/format version、restore/replay applicability、concrete storage location の owner、rerun or regeneration condition。artifact が restore/replay 用でない場合、report はそれを明示しなければなりません。backup existence は restore readiness を含意しません。

### 5.4 Redaction and Sensitive Data Rule

export / backup artifact は、特定の regulated/private evidence policy が当該 class を admit しない限り、raw secret / raw token / raw packet payload / unredacted SDP/ICE material / regulated payload / personal data を含んではなりません（禁止）。redaction failure は artifact adoption を reject します。encrypted artifact も sensitive であり、artifact class / redaction class / retention class を持たなければなりません。

### 5.5 Integrity Rule

digest、hash-chain、canonical serialization、storage checksum は異なる evidence class です。storage checksum は当該 storage operation の concrete artifact byte stability のみを証明します。それは audit hash-chain validity、deterministic canonical serialization、domain restore correctness、schema migration correctness を証明しません。

### 5.6 Restore / Import Rule

artifact からの restore / import / replay / migration は別個の operation です。それぞれ owning policy と evidence report を要します。artifact は backup evidence として有効でも restore input として未採用であり得ます。

### 5.7 Failure Mapping（閉集合）

| Failure | Required reason |
|---|---|
| export or backup surface is not admitted | `export_surface_not_allowed` |
| artifact requires redaction before adoption | `export_redaction_required` |
| artifact cannot be generated or fetched from driver/tool | `backup_artifact_unavailable` |
| artifact integrity check fails | `artifact_integrity_mismatch` |
| artifact is used as restore/import input without admission | `artifact_restore_not_allowed` |

### 5.8 Audit Rule

export / backup artifact decision は audit event type `export_backup_artifact_decision` を使用します。当該 event は artifact class、source scope、redaction class、存在する場合の integrity reference、retention class、`CorrelationId`、storage/tool owner を持たなければなりません（必須）。

### 5.9 Prohibitions と Collapse Conditions

禁止（閉集合）: database dump shape が public API になる。backup file existence が restore success として扱われる。storage checksum が audit hash-chain proof として扱われる。raw secret / token / packet payload / SDP/ICE material / regulated payload / personal data が general evidence へ export される。artifact path / bucket key が core domain identity になる。export format が explicit policy connection なしに canonical serialization として扱われる。release/distribution artifact evidence が backup evidence の下に隠れる。崩壊条件: artifact class が欠ける。shared artifact に redaction または retention class が欠ける。backup/import/restore semantics が混同される。integrity evidence class が不明確。failure reason が free-text のままで artifact generation が成功し得る。

---

## 6. Distributed State / Replication / Failover

### 6.1 Boundary（所有割当、閉集合）

distributed state は複数 node / process / service instance が同じ domain state family に関与し得る状態です。v0.2 initial architecture では explicit policy が無い限り、Signaling room、SFU route、TURN allocation/permission/channel bind、packet cache、runtime queue は node-local として扱います。本節は replication engine、consensus protocol、leader election、automatic failover、multi-node production readiness を主張しません。

| Concern | Owner | Rule |
|---|---|---|
| domain state ownership | core | state family and aggregate invariant |
| node-local runtime state | entrypoints/drivers runtime | not cluster-global by default |
| node affinity / sticky routing | deployment topology + entrypoints | required when state is node-local |
| replication execution | driver/entrypoints/deployment | prohibited unless admitted |
| consensus / leader election | a future scope expansion | not present in initial v0.2 |
| failover recovery | recovery/restore policy + this chapter | success requires evidence |
| evidence/reporting | reports | state class, owner, node scope, and conflict rule required |

### 6.2 Distributed State Classes（閉集合）

v0.2 initial architecture の distributed state class は次に限定します。新規 distributed state class は v0.2 初期 scope 外です。

| Class | Meaning | Rule |
|---|---|---|
| `node_local_state` | state exists only on owning node/process | default for SFU/TURN active runtime state |
| `affinity_required_state` | command/packet must reach owning node | affinity key and unavailable behavior required |
| `checkpoint_candidate_state` | restart recovery candidate under restore policy | not replication |
| `audit_verification_state` | audit/hash-chain can verify ordering/integrity | not mutable domain state |
| `replicated_state_requested` | state replication is requested | rejected in the v0.2 initial scope |
| `consensus_state_requested` | consensus/leader/quorum behavior is requested | rejected in the v0.2 initial scope |
| `automatic_failover_requested` | live failover without explicit restore proof is requested | rejected in initial v0.2 |

### 6.3 Ownership Rule（必須記録項目）

node/process boundary を跨ぎ得るすべての state family は次を記録しなければなりません。state family、owning node/process scope、owner reference or affinity key、command routing rule、packet-scoped 時の packet routing rule、recovery/restore relation、conflict rule、failure reason mapping、audit event relation。この policy が無い場合、node-local state への cross-node access は fail closed しなければなりません。

### 6.4 Replication and Consensus Rule

initial v0.2 は generic replication、consensus、leader election、quorum write、core state の eventually consistent mutation を admit しません。replication-like behavior は次としてのみ現れてよい（許可）。bounded retry policy 下の driver-local retry recovery、durable recovery 規則下の checkpoint candidate、audit/hash-chain verification、test-only と明示された test fixture / fake driver behavior。これらのいずれも production distributed state semantics になりません。

### 6.5 Failover Rule（failover 規則、必須記録項目）

failover は process restart、endpoint resolution、service discovery fallback、health probe success では証明されません。failover evidence は次を記録しなければなりません。failed owner/node observation、replacement owner/node、affected state family、affinity/sticky routing update、state 再構成時の restore/replay policy、conflict and duplicate handling、resource/lifetime revalidation、audit continuity または close-not-claimed scope。これらが欠ける場合、failover は not proven として扱わなければなりません。

### 6.6 Split-Brain Rule

admitted conflict/consensus rule 無しに 2 node が同じ owner-scoped state の command を accept し得る場合、その path は invalid です。system は両 outcome を accepted として提示するのではなく、reject / drain / close-not-claimed mark のいずれかを行わなければなりません。

### 6.7 Failure Mapping（閉集合）

recovery/restore failure は durable recovery 規則が、service discovery failure は service discovery 規則が、resource/lifetime bound failure は resource bounds 規則が支配し続けます。

| Failure | Required reason |
|---|---|
| distributed state class is not admitted | `distributed_state_not_admitted` |
| state replication requested without admitted policy | `state_replication_not_admitted` |
| consensus or leader election requested without admitted policy | `consensus_not_admitted` |
| automatic failover requested without admitted policy/evidence | `failover_not_proven` |
| node affinity is required but absent | `node_affinity_required` |
| owner node/state is unavailable | `node_state_unavailable` |
| cross-node route or relay is not allowed | `cross_node_route_not_allowed` |
| two owners conflict for the same state scope | `state_owner_conflict` |
| split-brain risk is detected | `split_brain_risk_detected` |
| replication lag or handoff window exceeds bound | `replication_lag_bound_exceeded` |

### 6.8 Audit Rule

distributed state / failover decision は audit event type `distributed_state_failover_decision` を使用します。当該 event は `StartupRunId`、command-scoped 時の `CorrelationId`、distributed state class、state family、owner node/scope、applicable な affinity key、failover class、applicable な replacement owner、rejected/failed outcome の cataloged reason を持たなければなりません（必須）。

### 6.9 Prohibitions と Collapse Conditions

禁止（閉集合）: node-local state が default で cluster-global として扱われる。service discovery fallback が failover success として扱われる。health/readiness success が state handoff success として扱われる。audit replay が restore policy なしに state を mutate する。conflict/consensus rule 無しに 2 node が owner-scoped command を accept する。replication lag または handoff window が unbounded である。test fake multi-node behavior が production distributed state evidence として使用される。崩壊条件: state owner または node scope が欠ける。node-local state が affinity または distributed policy なしに cross-node access され得る。failover が restore/replay/conflict evidence なしに主張される。replication または consensus が implicit implementation detail になる。split-brain が同じ owner-scoped state に対し 2 つの accepted domain outcome を生み得る。

---

## 7. 章全体の fail-closed 不変条件

本章の全 persistence / state driver に共通する fail-closed 不変条件は次のとおりです（必須）。persistence failure / restore failure / migration failure は successful domain decision、closeout evidence、runtime readiness へ silently に downgrade しない。すべての failure は catalog 済み reason code を名指しする。room / SFU route / TURN allocation state は explicit policy なしに durable source-of-truth でない。restore は第3.4節の restore precondition を 1 つでも欠けば fail-closed で拒否される。schema migration success / backup existence / storage checksum / storage replication はそれぞれ domain restore success / restore readiness / hash-chain proof / domain replication を証明しない。raw sensitive material は export/backup artifact に redaction class なしに含めない。distributed state / failover は第6.5節の evidence なしに proven として扱わない。これらの不変条件を満たさない path は close / complete / ready evidence として採用してはなりません。
