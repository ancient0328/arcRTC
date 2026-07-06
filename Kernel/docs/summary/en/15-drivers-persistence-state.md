# drivers-persistence-state

Status: public summary projection
Date: 2026-07-06 JST

## Purpose

This chapter internalizes the current complete specification of the persistence / state driver family of arcRTC v0.2 Kernel, at a granularity sufficient for re-implementation from this chapter alone. This chapter internalizes the persistence boundary (PostgreSQL/Redis/S3/filesystem/in-memory), the state persistence policy (the division between state class and checkpoint/audit-only/driver-local), the durable recovery / restore / replay full recovery classes / state family policy / restore preconditions / replay rule / failure mapping, the schema / migration lifecycle, the export / backup artifact boundary, and the distributed state / replication / failover policy, down to owners, closed-set vocabulary, state machines, failure mapping, prohibitions/permissions, and fail-closed conditions, omitting none.

Dependency direction notation: `A <- B` means "B depends on A". Persistence is separated into a core-owned port contract and a driver-owned storage implementation. The driver implements a core-owned port, and external storage types stop at the driver boundary. `drivers -> entrypoints` and `drivers -> regulated` are prohibited. The v0.2 initial architecture has no implicit durable domain source-of-truth for room / SFU route / TURN allocation state. This chapter does not claim implemented recovery behavior, runtime readiness, production readiness, a replication engine, consensus, leader election, or automatic failover.

---

## 1. Persistence Boundary (`drivers/persistence`)

### 1.1 Ownership (ownership assignment, closed set)

| Area | Owner | Rule |
|---|---|---|
| PersistencePort contract | core | state / checkpoint / audit persistence intent |
| persistence consistency requirement | core | idempotency, ordering, retention policy intent |
| concrete DB / object store / filesystem | driver | PostgreSQL, Redis, S3, filesystem, memory |
| schema / migration file | driver | storage implementation detail |
| retry store execution | driver | bounded by core/driver policy mapping |
| entrypoint selection of store | entrypoints | typed configuration and wiring only |

### 1.2 Persistence Surfaces (closed set)

The surfaces treated as persistence in the v0.2 initial architecture are limited to the following. A new persistence surface is out of the v0.2 initial scope.

| Surface | Core contract | Driver implementation |
|---|---|---|
| domain state checkpoint | checkpoint intent and version | DB / memory / file |
| audit event persistence | ordered audit record intent | file / HTTP / syslog / DB / S3 |
| audit hash-chain record | chain scope and sequence semantics | storage and export |
| retry store | retry policy intent and closed bounds | queue / durable store |

### 1.3 Atomicity Relation

The persistence driver MAY provide a concrete transaction / durability mechanism. Core command atomicity, commit boundary, compensation, and the point at which a result is accepted as evidence are defined outside the storage implementation. When a command path requires both a core decision and a persistence/audit side effect, per-step outcomes MUST be reported according to the atomicity rule.

### 1.4 Storage Shape Rule

Core MUST NOT expose SQL row shape, table name, S3 key layout, filesystem path, Redis key, or migration file as a domain API. The driver MAY define these only as implementation detail. If a persisted representation must become a public contract / shared artifact, it requires the relevant protocol rule or the export / backup artifact rule (Section 5). Schema migration success does not prove domain restore success. Domain restore/replay evidence is evaluated separately under the durable recovery rule (Section 3). Replication / consensus / leader election / failover claims require the evidence of the distributed state rule (Section 6); storage availability / transaction success does not prove them.

### 1.5 Failure and Retry Rule (closed set)

Persistence retry bounds and actions follow the resource bounds / backpressure rule. Persistence driver failure MUST NOT be silently downgraded to successful domain decision evidence. Persistence transaction success MUST NOT erase a failed audit / external response / compensation step.

| Failure | Required reason |
|---|---|
| concrete persistence unavailable | `persistence_unavailable` |
| retry entry count / byte / retry-count bound reached | `persistence_retry_bound_exceeded` |
| retry duration exceeded | `persistence_retry_duration_exceeded` |
| driver shutdown | `driver_shutdown` |

### 1.6 Audit Relation

Audit event meaning is owned by the audit event rule, and audit hash-chain semantics by the audit hash-chain rule. The persistence driver MAY store / export / retry audit records but does not own audit event meaning or chain validity semantics. Exported / backed-up persistence artifacts require an artifact class / redaction class / retention class / integrity class under the export / backup artifact rule. If the selected audit persistence cannot record a required failure and no bootstrap audit record path is available, the affected startup / decision path MUST NOT be used as close / complete / ready evidence.

### 1.7 Prohibitions (prohibition, closed set)

- Core imports the concrete API of `sqlx`, PostgreSQL, Redis, S3, or filesystem.
- A driver schema decides a domain invariant.
- The persistence driver accepts / rejects a Signaling / SFU / TURN command.
- A persistence failure hides behind a successful closeout claim.
- Storage transaction success is reported as domain commit success without a core decision and atomicity evidence.
- The retry store is unbounded.
- A migration file is treated as a domain model source of truth.
- A backup/export artifact is used as restore success evidence without restore/replay qualification.
- A storage replication feature is treated as domain replication / failover success without the distributed state rule.

### 1.8 Persistence Boundary Collapse Conditions

A storage schema becomes a core domain API. Driver persistence owns state transition semantics. The persistence retry queue is unbounded or unaudited. Audit hash-chain meaning is delegated to the storage implementation. Closeout evidence relies on persistence output whose write failure was not recorded or bounded. The persistence implementation becomes the owner of compensation / aggregate commit semantics. A persisted artifact leaves the storage boundary without export/backup artifact classification. Persistence backend replication is used as a domain state ownership proof.

---

## 2. State Persistence Policy

### 2.1 State Classes (closed set)

The state classes of the v0.2 initial architecture are limited to the following. A new state class is out of the v0.2 initial scope.

| Class | Meaning | Persistence rule |
|---|---|---|
| `ephemeral-core-state` | domain state in runtime memory | persistence is not required |
| `checkpoint-eligible-state` | core-owned snapshot intent that may be saved for restart/recovery | saved only via PersistencePort |
| `audit-only-state` | left in audit event / hash-chain as decision/evidence | via AuditSinkPort / hash-chain record |
| `driver-local-state` | socket, buffer, retry queue, external client/session detail | driver-owned, not a core source-of-truth |
| `configuration-scope-state` | startup/wiring/config validation scope | configuration decision and startup references |
| `sdk-local-state` | SDK connection/client state | SDK-owned, not server/core state |

### 2.2 Aggregate State Policy (state class per aggregate, closed set)

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

The initial v0.2 has no implicit durable domain source-of-truth for room / SFU route / TURN allocation state. Durable recovery / restore / replay behavior is governed by the durable recovery rule (Section 3). Persistence classification does not admit replication / consensus / failover. A future scope expansion that extends durable recovery semantics beyond that rule MUST define the following: persisted state owner, snapshot shape, restore preconditions, replay/audit relation, conflict resolution, distributed state owner/scope when multiple nodes are involved, failure reason mapping, and verification evidence.

### 2.4 Checkpoint Rule

A checkpoint is an optimization / recovery support and is not automatically the domain source of truth. Core owns checkpoint intent and version. The driver owns concrete schema / table / key / object / file layout. Driver DB transaction success MAY support driver-local durability but does not define aggregate commit semantics unless the atomicity rule and state machine permit it. Checkpoint restore MUST NOT create room / participant / route / allocation / permission / channel bind state unless a restore policy exists. Checkpoint persistence MUST NOT be used as replication / failover proof unless the distributed state rule admits that state class.

### 2.5 Audit Relation

The audit event / hash-chain record is evidence and tamper-evidence material and is not a mutable domain state store. Audit replay MAY be used for verification only when a separate replay/restore rule defines ordering / gap handling / conflict rules. The initial replay/restore owner boundary is the durable recovery rule; absent state-family permission there, replay remains verification-only.

### 2.6 State Policy Failure Rule (closed set)

Persistence failure MUST NOT be hidden while claiming state recovery / closeout evidence / runtime readiness.

| Failure | Required reason |
|---|---|
| persistence unavailable | `persistence_unavailable` |
| retry store bound exceeded | `persistence_retry_bound_exceeded` |
| retry duration exceeded | `persistence_retry_duration_exceeded` |
| audit backlog exceeded | `audit_backlog_bound_exceeded` |
| driver shutdown | `driver_shutdown` |

### 2.7 Prohibitions (prohibition, closed set)

- A driver persistence schema becomes a domain source-of-truth.
- Checkpoint restore creates domain state without a restore policy.
- SFU route state is durable by default.
- TURN allocation is silently restored from driver storage.
- The audit log is treated as a mutable state store.
- A driver DB transaction is treated as aggregate commit authority.
- SDK local connection state is treated as server-side participant state.
- The driver retry queue is unbounded or used as domain state.
- Persisted state is treated as replicated or failover-ready without a distributed state policy.

### 2.8 State Persistence Policy Collapse Conditions

A persisted schema defines an aggregate invariant. Restart/recovery behavior is claimed without a restore policy. Audit-only state is mutated as domain state. Driver-local state is used as a core source-of-truth. A persistence state class is used to bypass distributed state / failover admission. v0.2 closeout relies on persistence recovery that has not been specified and verified. Persistence transaction success is used as a domain atomicity proof without commit boundary evidence.

---

## 3. Durable Recovery / Restore / Replay

### 3.1 Boundary (ownership assignment, closed set)

| Concern | Owner | Rule |
|---|---|---|
| restore eligibility policy | core | only those explicit per state family become restore candidates |
| restore precondition | core | checks version, ordering, conflict, bound |
| snapshot / checkpoint intent | core | schema-independent checkpoint version and semantic shape |
| concrete snapshot schema | driver | DB row, object key, file layout, migration detail |
| audit replay verification | core | audit event ordering and hash-chain verification semantics |
| replay execution storage read | driver | bounded read, pagination, I/O failure handling |
| recovery mode selection | entrypoints | typed configuration and selected driver wiring only |
| crash/restart trigger classification | entrypoints/driver observation converted to core-owned evidence class | does not auto-establish restore eligibility |
| distributed failover claim | distributed state policy + recovery evidence | restore success and failover success are separate |

Data saved by the driver is not, by itself, a domain source-of-truth. Even if entrypoints selects a recovery mode, state that does not satisfy the core restore precondition MUST NOT be restored.

### 3.2 Recovery Classes (closed set)

The recovery classes of the v0.2 initial architecture are limited to the following. A new recovery class is out of the v0.2 initial scope.

| Recovery class | Meaning | Permitted use |
|---|---|---|
| `no_restore` | does not restore state after restart | initial default for SFU route, TURN active allocation, driver-local state |
| `checkpoint_restore_candidate` | can be a restore candidate from a core-owned checkpoint intent | state with an explicit policy, such as idempotency state |
| `audit_replay_verification_only` | replay is limited to verification and does not mutate domain state | audit event / hash-chain consistency verification |
| `driver_retry_recovery` | resumes a driver-owned retry queue boundedly | persistence retry, audit sink retry, etc. |

### 3.3 State Family Policy (closed set)

| State family | Recovery class | Restore rule |
|---|---|---|
| Signaling room state | `no_restore` by default | does not recreate a room after restart unless an explicit restore policy exists |
| Signaling participant state | `no_restore` by default | does not mark a participant joined without a connection observation |
| Signaling idempotency state | `checkpoint_restore_candidate` when configured | candidate only when checkpoint version, command identity, and ordering window match |
| SFU endpoint / route / publication / subscription state | `no_restore` | does not make packet route / subscription state a durable source-of-truth |
| TURN allocation / permission / channel bind state | `no_restore` | does not silently restore socket path / credential / lifetime |
| Audit event / hash-chain | `audit_replay_verification_only` | replay is verification, not domain state mutation |
| Persistence retry store | `driver_retry_recovery` | resumes only within the retry bound and duration bound |
| Metrics backlog | `driver_retry_recovery` when configured | metrics export retry is not domain evidence |
| SDK local state | `no_restore` for server/core | not treated as server-side participant state |

### 3.4 Restore Preconditions (restore preconditions, all MUST)

A restore attempt becomes a candidate for core state mutation only when it satisfies all of the following. If even one is unmet, the restore is rejected fail-closed.

- triggering lifecycle observation is classified when restore follows crash, panic, unclean shutdown, or supervisor restart;
- the restore policy permits the state family;
- the snapshot/checkpoint version is evaluated as an accepted version;
- correlation chain, state owner, and aggregate reference are verified as core-owned references;
- for state requiring an audit hash-chain relation, there is no gap;
- checkpoint and replay result do not conflict;
- resource bound and lifetime bound are satisfied even at restore time;
- driver read failure, schema mismatch, and migration failure are connected to a cataloged reason;
- for state families requiring distributed state/failover policy, owner node, replacement owner, and conflict rule are recorded.

### 3.5 Replay Rule

Audit replay is audit integrity verification and is not the default route for domain mutation. To restore domain state via replay, the restore policy per state family MUST define the following: replay target event type, replay ordering, gap handling, duplicate handling, conflict resolution, lifetime / resource bound revalidation, replay failure reason mapping, and the condition under which a replay result may be adopted as closeout evidence. If these definitions do not exist, the audit replay result is limited to a verification observation.

### 3.6 Failure Mapping (closed set)

Restore failure is recorded through audit/event evidence when the selected audit path is available. If audit recording itself is unavailable, the affected restore attempt MUST NOT be used as close / complete / ready evidence.

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

### 3.7 Prohibitions (prohibition, closed set)

- A DB row / object key / file path / migration file becomes a domain source-of-truth.
- Audit replay mutates domain state without an explicit restore policy.
- SFU route state is silently restored.
- TURN allocation / permission / channel bind is silently restored.
- SDK local reconnect state is treated as server participant state.
- A restore conflict is resolved by driver-local preference.
- Restore success is claimed without checkpoint/replay evidence and a correlation ID.
- A crash/restart observation is treated as restore success.
- A service discovery fallback or replacement process start is treated as failover success.
- Replicated/consensus state is inferred from restore or checkpoint behavior.

### 3.8 Durable Recovery Collapse Conditions

A state family without a restore policy is restored. A driver schema decides an aggregate invariant or conflict resolution. Audit replay is used as a mutable domain state store. Restart behavior is treated as verified without restore report evidence. Resource/lifetime bounds are not rechecked during restore. Process lifecycle classification is missing for a crash/restart-driven recovery claim. Distributed owner/scope/conflict rule is absent for a failover recovery claim.

---

## 4. Schema / Migration Lifecycle

### 4.1 Boundary (ownership assignment, closed set)

| Concern | Owner | Rule |
|---|---|---|
| PersistencePort contract | core | schema-independent intent and version |
| persisted state semantic version | core | accepted version of checkpoint/retry/audit intent |
| concrete schema/table/key/file layout | driver | storage implementation detail |
| migration file | driver | concrete storage evolution detail |
| migration execution selection | entrypoints | startup/wiring configuration only |
| migration evidence report | dated evidence record | command/result/correlation/reproducible procedure |

A driver schema does not own aggregate invariant / state transition / reason semantics. Core does not depend on SQL table / object key / filesystem path / migration file name. When canonical serialization of a persisted representation is needed for digest / compatibility evidence, it follows the canonical serialization rule; migration snapshots / exported schema support artifacts follow the export / backup artifact rule (Section 5).

### 4.2 Migration Classes (closed set)

The migration classes of the v0.2 initial architecture are limited to the following. A new migration class is out of the v0.2 initial scope.

| Migration class | Meaning | Rule |
|---|---|---|
| `driver_schema_init` | new store initialization | startup evidence required |
| `driver_schema_forward` | forward change of concrete storage shape | compatibility / rollback plan required |
| `driver_schema_rollback` | reversal of concrete storage shape | data loss condition stated explicitly |
| `driver_encoding_compat` | persisted representation decode compatibility | accepted version and failure reason stated explicitly |
| `no_migration_required` | schema unchanged | report may state skipped with reason |

### 4.3 Lifecycle Rule (fixed order)

The schema migration lifecycle is handled in the following order.

1. the core contract defines persistence surface and owner.
2. driver defines concrete schema/migration implementation.
3. entrypoints wires migration mode and selected driver.
4. migration dry-run or validation command is executed where available.
5. migration execution result is recorded as a dated evidence record.
6. restore/replay/checkpoint behavior is evaluated by its own policy, not by schema existence.

Schema migration success does not prove domain restore success. Domain restore success requires the durable recovery rule (Section 3).

### 4.4 Compatibility Rule (compatibility rule, required items)

Persisted representation compatibility MUST define the following: accepted representation versions, rejected representation versions, the canonical format/version when the representation is used for digest / compatibility evidence, required fields, default behavior for newly optional fields, the explicit rule for unknown field handling, missing field reason mapping, rollback condition, and data loss condition. An unsupported or unmappable persisted representation MUST fail closed with a cataloged driver conversion / persistence reason.

### 4.5 Failure Mapping (closed set)

A migration failure MUST NOT be hidden behind startup success / closeout evidence.

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

### 4.6 Report Rule (required items for the migration report)

A migration report MUST include the following: correlation ID or startup run ID, target driver and store class, migration class, source schema/encoding version, target schema/encoding version, canonical format/version when digest / compatibility evidence is claimed, command or procedure, result, cataloged reason for non-success, rollback status or rollback non-applicability reason, and sensitive-data redaction statement. A report lacking a reproducible procedure or correlation/startup reference is not migration evidence.

### 4.7 Prohibitions (prohibition, closed set)

- A migration file defines a domain invariant.
- Schema existence is treated as restore/replay success.
- Core imports a DB migration library.
- A driver migration failure is silently skipped.
- An incompatible persisted representation is best-effort decoded.
- A storage row/object layout is treated as canonical serialization without an explicit rule.
- A rollback plan is omitted for a forward migration that can affect durable data.
- A migration snapshot is adopted as backup/restore evidence without artifact classification.

### 4.8 Schema / Migration Collapse Conditions

A schema/table/key layout becomes a core API. Migration success is used as a domain recovery proof. Incompatible stored data is accepted without a cataloged reason. A migration report lacks a correlation/startup reference or a reproducible procedure. Migration mode is selected by the driver without entrypoints typed configuration. Migration compatibility evidence lacks a canonical representation rule. An exported migration artifact lacks redaction / retention / integrity classification.

---

## 5. Export / Backup Artifact Boundary

### 5.1 Boundary (ownership assignment, closed set)

An export / backup artifact is a shareable or storable deliverable generated from storage / audit / configuration / evidence / checkpoint / schema support. The artifact's concrete format / path / bucket key / database dump shape is driver / tooling detail and is not a core semantic authority. Release/distribution artifacts follow the release artifact distribution provenance rule.

| Concern | Owner | Rule |
|---|---|---|
| artifact semantic class | core contract | closed class required |
| artifact generation execution | driver / tooling / entrypoints | concrete file, object, dump, archive |
| redaction and retention policy | privacy policy | raw sensitive material is not admitted |
| hash/digest/integrity | canonical serialization / audit hash-chain policy | concrete export format is not automatically canonical |
| restore/replay semantics | durable recovery / schema migration policy | backup presence does not prove restore success |
| public sharing decision | governance / target policy | artifact must not become public API by accident |

### 5.2 Artifact Classes (closed set)

The export / backup artifact classes of the v0.2 initial architecture are limited to the following. A new artifact class is out of the v0.2 initial scope.

| Artifact class | Meaning | Owner |
|---|---|---|
| `audit_event_export` | redacted audit event export or chain segment | audit event rule and audit hash-chain rule |
| `state_checkpoint_backup` | domain checkpoint backup for recovery qualification | state persistence policy rule and durable recovery rule |
| `persistence_snapshot` | driver-local DB/object/filesystem snapshot | persistence boundary rule |
| `configuration_export` | typed configuration/profile/policy bundle export | configuration rule |
| `evidence_bundle` | rerunnable report-support material | evidence report rule |
| `schema_migration_snapshot` | schema/migration support snapshot | schema migration rule |

### 5.3 Artifact Admission Rule (admission condition, required items)

An export / backup artifact may be adopted as evidence only when it records the following: artifact class, generation command or procedure, working directory or driver/tool owner, source scope and time phase, correlation ID, redaction class, retention class, integrity digest or explicit non-digest reason, schema/format version when applicable, restore/replay applicability, owner of any concrete storage location, and rerun or regeneration condition. If the artifact is not intended for restore/replay, the report MUST state that explicitly. Backup existence does not imply restore readiness.

### 5.4 Redaction and Sensitive Data Rule

Export / backup artifacts MUST NOT contain raw secret / raw token / raw packet payload / unredacted SDP/ICE material / regulated payload / personal data unless a specific regulated/private evidence policy admits the class. Redaction failure rejects artifact adoption. An encrypted artifact is still sensitive and MUST carry its artifact class / redaction class / retention class.

### 5.5 Integrity Rule

Digest, hash-chain, canonical serialization, and storage checksum are different evidence classes. Storage checksum proves only the concrete artifact byte stability for that storage operation. It does not prove audit hash-chain validity, deterministic canonical serialization, domain restore correctness, or schema migration correctness.

### 5.6 Restore / Import Rule

Restore / import / replay / migration from an artifact are separate operations. They each require their owning policy and an evidence report. An artifact may be valid as backup evidence while still not being admitted as restore input.

### 5.7 Failure Mapping (closed set)

| Failure | Required reason |
|---|---|
| export or backup surface is not admitted | `export_surface_not_allowed` |
| artifact requires redaction before adoption | `export_redaction_required` |
| artifact cannot be generated or fetched from driver/tool | `backup_artifact_unavailable` |
| artifact integrity check fails | `artifact_integrity_mismatch` |
| artifact is used as restore/import input without admission | `artifact_restore_not_allowed` |

### 5.8 Audit Rule

Export / backup artifact decisions use audit event type `export_backup_artifact_decision`. The event MUST carry artifact class, source scope, redaction class, integrity reference when present, retention class, `CorrelationId`, and storage/tool owner.

### 5.9 Prohibitions and Collapse Conditions

Prohibitions (closed set): a database dump shape becomes a public API; backup file existence is treated as restore success; storage checksum is treated as an audit hash-chain proof; raw secret / token / packet payload / SDP/ICE material / regulated payload / personal data is exported into general evidence; an artifact path / bucket key becomes core domain identity; an export format is treated as canonical serialization without an explicit policy connection; release/distribution artifact evidence hides under backup evidence. Collapse conditions: artifact class is absent; redaction or retention class is absent for a shared artifact; backup/import/restore semantics are conflated; integrity evidence class is unclear; artifact generation can succeed while the failure reason remains free-text only.

---

## 6. Distributed State / Replication / Failover

### 6.1 Boundary (ownership assignment, closed set)

Distributed state is a state in which multiple nodes / processes / service instances may participate in the same domain state family. In the v0.2 initial architecture, unless an explicit policy exists, Signaling room, SFU route, TURN allocation/permission/channel bind, packet cache, and runtime queue are treated as node-local. This section does not claim a replication engine, consensus protocol, leader election, automatic failover, or multi-node production readiness.

| Concern | Owner | Rule |
|---|---|---|
| domain state ownership | core | state family and aggregate invariant |
| node-local runtime state | entrypoints/drivers runtime | not cluster-global by default |
| node affinity / sticky routing | deployment topology + entrypoints | required when state is node-local |
| replication execution | driver/entrypoints/deployment | prohibited unless admitted |
| consensus / leader election | a future scope expansion | not present in initial v0.2 |
| failover recovery | recovery/restore policy + this chapter | success requires evidence |
| evidence/reporting | reports | state class, owner, node scope, and conflict rule required |

### 6.2 Distributed State Classes (closed set)

The distributed state classes of the v0.2 initial architecture are limited to the following. A new distributed state class is out of the v0.2 initial scope.

| Class | Meaning | Rule |
|---|---|---|
| `node_local_state` | state exists only on owning node/process | default for SFU/TURN active runtime state |
| `affinity_required_state` | command/packet must reach owning node | affinity key and unavailable behavior required |
| `checkpoint_candidate_state` | restart recovery candidate under restore policy | not replication |
| `audit_verification_state` | audit/hash-chain can verify ordering/integrity | not mutable domain state |
| `replicated_state_requested` | state replication is requested | rejected in the v0.2 initial scope |
| `consensus_state_requested` | consensus/leader/quorum behavior is requested | rejected in the v0.2 initial scope |
| `automatic_failover_requested` | live failover without explicit restore proof is requested | rejected in initial v0.2 |

### 6.3 Ownership Rule (required record items)

Every state family that can be touched across node/process boundaries MUST record the following: state family, owning node/process scope, owner reference or affinity key, command routing rule, packet routing rule when packet-scoped, recovery/restore relation, conflict rule, failure reason mapping, and audit event relation. Absent this policy, cross-node access to node-local state MUST fail closed.

### 6.4 Replication and Consensus Rule

The initial v0.2 does not admit generic replication, consensus, leader election, quorum write, or eventually consistent mutation of core state. Replication-like behavior MAY appear only as: driver-local retry recovery under a bounded retry policy; a checkpoint candidate under the durable recovery rule; audit/hash-chain verification; or test fixture / fake driver behavior labelled as test-only. None of these becomes production distributed state semantics.

### 6.5 Failover Rule (failover rule, required record items)

Failover is not proven by process restart, endpoint resolution, service discovery fallback, or health probe success. Failover evidence MUST record the following: failed owner/node observation, replacement owner/node, affected state family, affinity/sticky routing update, restore/replay policy if state is reconstructed, conflict and duplicate handling, resource/lifetime revalidation, and audit continuity or close-not-claimed scope. If these fields are absent, failover MUST be treated as not proven.

### 6.6 Split-Brain Rule

If two nodes can accept commands for the same owner-scoped state without an admitted conflict/consensus rule, the path is invalid. The system MUST reject, drain, or mark close-not-claimed rather than presenting both outcomes as accepted.

### 6.7 Failure Mapping (closed set)

Recovery/restore failures remain governed by the durable recovery rule; service discovery failures by the service discovery rule; resource/lifetime bound failures by the resource bounds rule.

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

Distributed state / failover decisions use audit event type `distributed_state_failover_decision`. The event MUST carry `StartupRunId`, `CorrelationId` when command-scoped, distributed state class, state family, owner node/scope, affinity key when applicable, failover class, replacement owner when applicable, and the cataloged reason for rejected/failed outcomes.

### 6.9 Prohibitions and Collapse Conditions

Prohibitions (closed set): node-local state is treated as cluster-global by default; service discovery fallback is treated as failover success; health/readiness success is treated as state handoff success; audit replay mutates state without a restore policy; two nodes accept owner-scoped commands without a conflict/consensus rule; replication lag or handoff window is unbounded; test fake multi-node behavior is used as production distributed state evidence. Collapse conditions: state owner or node scope is absent; node-local state can be accessed cross-node without affinity or distributed policy; failover is claimed without restore/replay/conflict evidence; replication or consensus becomes an implicit implementation detail; split-brain can produce two accepted domain outcomes for the same owner-scoped state.

---

## 7. Chapter-wide Fail-closed Invariants

The fail-closed invariants common to all persistence / state drivers in this chapter are as follows (MUST). Persistence failure / restore failure / migration failure is not silently downgraded to a successful domain decision, closeout evidence, or runtime readiness. Every failure names a cataloged reason code. Room / SFU route / TURN allocation state is not a durable source-of-truth without an explicit policy. A restore is rejected fail-closed if it lacks even one of the restore preconditions in Section 3.4. Schema migration success / backup existence / storage checksum / storage replication does not, respectively, prove domain restore success / restore readiness / hash-chain proof / domain replication. Raw sensitive material is not included in an export/backup artifact without a redaction class. Distributed state / failover is not treated as proven without the evidence in Section 6.5. A path that does not satisfy these invariants MUST NOT be adopted as close / complete / ready evidence.
