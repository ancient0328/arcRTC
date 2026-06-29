# 第04章 core-command-and-reason

状態: SSOT 統合版
日付: 2026-06-28 JST

## 目的

本章は arcRTC v0.2 Kernel の core における command / decision / domain event / port intent / external response の結果形状、command idempotency / replay / correlation 規則、閉集合 reason catalog（全 reason code を網羅）、および internal reason と external 表現の error mapping を、再現実装可能な粒度で内在化します。本章は Rust struct / enum 実装、public API 実装、runtime behavior を主張しません。

依存方向の記法は `A <- B` を「B が A に依存」と読みます。層構造は `core <- drivers <- entrypoints` です。

## 1. 結果形状の所有境界

| Surface | Owner | 規則 |
|---|---|---|
| inbound command semantic shape | core | driver conversion 後の core-owned command |
| use case decision | core | accept / reject / suppress / drop / fail 等の authoritative outcome |
| domain event | core | state transition または decision observation |
| port command / intent | core | driver execution request。bytes や concrete I/O は含まない |
| driver execution observation | driver が core-owned observation へ変換 | concrete send/write/read result を cataloged reason へ変換 |
| audit event | core model、driver sink | decision/event の closed projection |
| external response | driver/sdk | core reason/outcome を失わず外部形式へ写す |

driver は core decision を成功へ書き換えてはなりません。entrypoints は use case result shape を binary ごとに分岐定義してはなりません。

## 2. result shape class（閉集合）

| Shape class | Owner | 意味 |
|---|---|---|
| `CommandEnvelope` | core | validated correlation、version、command type、subject references |
| `UseCaseDecision` | core | command に対する authoritative outcome |
| `DomainEvent` | core | state transition または decision fact |
| `PortIntent` | core | driver に依頼する execution intent |
| `DriverObservation` | driver-to-core mapping | concrete execution result を core-owned observation へ変換 |
| `AuditProjection` | core | audit event model に射影した evidence item |
| `ExternalResponseModel` | driver/sdk | external response shape。authoritative reason は保持 |

新規 result shape class は仕様の更新を要します。

## 3. UseCaseDecision の必須 field

すべての `UseCaseDecision` は次を持たなければなりません。

- correlation ID
- command または event type
- target surface
- outcome
- reason presence
- 非成功時の reason category/code
- state 変化時の state transition summary
- driver execution が必要なときの port intents
- audit projection requirement
- decision が runtime proof でないときの close-not-claimed evidence class

success outcome は fake reason を持ってはなりません。非成功 outcome は cataloged reason を持たなければなりません。

## 4. construction input 規則

`UseCaseDecision` 等の結果形状は、correlation、command type、target surface、outcome、reason、state transition、port intent、audit projection、evidence class を裸の多引数 constructor として受け取ってはなりません。実装は未検査材料を `UseCaseDecisionInput` のような名前付き入力型に束ね、その入力型を fail-closed 検査へ渡します。この規則は field order の取り違え、fake reason の混入、audit/evidence class の抜け落ちを防ぐ境界です。

## 5. outcome 規則

許可される authoritative outcome は audit 互換の outcome set に限定し、use case result は第二の outcome vocabulary を発明してはなりません。

| Outcome family | 意味 | reason 規則 |
|---|---|---|
| success observation | accepted, allowed, forwarded, selected, released, closed_success, idempotent_observed, within_bound_observed | reason 不在 |
| rejection / denial | rejected, denied, protocol_violation | cataloged reason 必須 |
| suppression / drop / expiry / revocation | suppressed, dropped, expired, shed, revoked | cataloged reason 必須 |
| degradation / delay | delayed, degraded | cataloged reason 必須 |
| failure / conversion | failed, converted_failure | cataloged reason 必須 |
| policy closure | closed_by_policy | cataloged reason 必須 |

partial success は、仕様が per-step outcome と rollback/compensation rule を定義しない限り禁止します。

## 6. event と port intent 規則

core decision は domain event と port intent を生成できます。port intent は driver execution success ではありません。分離の例:

```text
UseCaseDecision accepted
  -> DomainEvent emitted
  -> PortIntent send response / forward packet / persist audit
  -> DriverObservation success または cataloged failure
```

accepted decision 後の driver failure は driver observation または follow-up decision evidence として表現します。state machine 仕様が compensating transition を明示しない限り、元の domain decision を遡って消去してはなりません。compensating transition が許可される場合、result shape は original decision と compensation decision の両方を保持しなければなりません。

## 7. audit projection 規則

audit を要する decision は次を定義しなければなりません。event type code / decision outcome / reason presence / required references / resource-bound 時の resource owner tuple / sensitive data redaction rule。

## 8. external response 規則

external response は projection であり authoritative decision ではありません。HTTP status、WebSocket close code、STUN/TURN error code、SDK exception class、CLI exit code は core outcome と reason への traceability を保持しなければなりません。

## 9. command idempotency / replay / correlation

### 9.1 所有境界

| Concern | Owner | 規則 |
|---|---|---|
| correlation tracing | driver validation 後に受理した core reference | request/event chain の追跡であり duplicate 判定そのものではない |
| command identity | core | command type、target reference、idempotency key、semantic payload digest の組 |
| idempotency policy | core | accepted duplicate / rejected duplicate / conflicting replay を判断 |
| replay window policy | core | lifetime、scope、payload comparison rule |
| idempotency storage / response cache | driver persistence または in-memory driver detail | bounded implementation。decision owner ではない |
| SDK pending command replay | sdk | client-local retry のみ。server acceptance を作らない |
| audit evidence | core event model + driver sink | idempotency decision と correlation references |

`CorrelationId` は trace identity であり、同一 `CorrelationId` だけで idempotent duplicate とは扱いません。idempotency は明示的な command identity rule を要します。

### 9.2 idempotency class（閉集合）

| Class | 意味 | 規則 |
|---|---|---|
| `non_idempotent_command` | duplicate は state machine が reject または再評価 | response replay 不可 |
| `idempotent_same_payload` | 同一 command identity かつ同一 canonical payload digest | prior accepted result を観測してよい |
| `idempotent_conflict` | 同一 idempotency scope だが payload digest または target が異なる | cataloged reason で reject |
| `response_replay_candidate` | prior response を external projection として replay 可 | replay window 内かつ同一 semantic result のみ |
| `sdk_pending_command_replay` | SDK が reconnect 後に client-local pending command を再送 | server は依然 command を評価 |
| `server_event_replay` | server 側からの missed event replay | 別 replay policy が存在しない限り禁止 |

新規 idempotency class は仕様の更新を要します。

### 9.3 command identity 規則

idempotent command identity は次を定義しなければなりません。command type / idempotency scope / target reference / 自然に存在する actor/participant reference / payload が semantics に影響する場合の canonical payload digest / replay window / accepted response replay rule / conflict reason / audit event relation。payload digest を用いる場合は本章第10章の canonical serialization 規則（第06章で内在化）に従います。

### 9.4 replay 規則

replay は retry success ではありません。driver または SDK は再送できますが、core は command が duplicate / expired / conflicting / acceptable のいずれかを評価しなければなりません。

response replay は次の全条件を満たすときのみ許可します。original command result が replay window 内 / command identity と canonical payload digest が一致 / original result の required audit evidence が存在するか path が close-not-claimed scope を明示 / response projection が original outcome と reason を保持。いずれか欠ける場合、response replay は fail-closed します。

### 9.5 idempotency / replay failure mapping

| Failure | 必須 reason |
|---|---|
| core entry 前に correlation ID 欠落 | `missing_correlation_id` |
| correlation が expected response/event chain に不一致 | `correlation_mismatch` |
| idempotency rule により duplicate reject | `duplicate_command` |
| duplicate の payload または target が異なる | `idempotency_payload_mismatch` |
| replay window expire | `idempotency_window_expired` |
| command class で replay 不許可 | `replay_not_allowed` |
| response replay evidence/cache unavailable | `response_replay_not_available` |
| canonical payload digest を生成できない | `canonical_serialization_failed` |

### 9.6 idempotency audit / evidence

idempotency / replay decision は audit event type `command_idempotency_decision` を用います。event は `CorrelationId`、command type、idempotency class、idempotency scope、自然に materialize した target reference を持たなければなりません。response replay が prior result を用いる場合、event は sensitive payload を copy せず prior correlation または audit event reference を参照しなければなりません。SDK reconnect evidence は、replayed command の server-side decision/audit evidence を含まない限り server idempotency を証明しません。

## 10. closed reason catalog

reason は閉集合です。free-text は補足説明に限定し、decision、audit、SDK mapping の正として使いません。reason は次の構成を持ちます。

| Field | Owner | 規則 |
|---|---|---|
| `category` | core | closed set |
| `code` | core | category 内 closed set |
| `retryable` | core | true / false |
| `safe_to_expose` | core | true / false |
| `audit_required` | core | true / false |
| `details` | driver または entrypoint | optional、non-authoritative |

### 10.1 cross-cutting category（12 個・閉集合）

| Category | 意味 |
|---|---|
| `malformed_input` | syntax または shape が invalid |
| `unsupported_version` | protocol version が不受理 |
| `unauthorized` | verification 失敗または required authorization 不在 |
| `forbidden_state` | 現 state で command が invalid |
| `duplicate` | idempotency rule が duplicate を reject |
| `ordering_violation` | command/event order が invalid |
| `expired` | lifetime または deadline 超過 |
| `resource_exhausted` | bounded resource limit 到達 |
| `backpressure` | pressure policy が delay/suppress/drop/degrade/close/reject recovery |
| `quality_violation` | quality policy が reject/suppress/degrade/reject recovery |
| `driver_failure` | conversion 後の external implementation failure |
| `shutdown` | lifecycle shutdown または cancellation により action 不可 |

### 10.2 reason metadata（category default）

`retryable`、`safe_to_expose`、`audit_required` は category default と code override から決定します。default にも override にも存在しない metadata を実装都合で推測してはなりません。

| Category | retryable | safe_to_expose | audit_required |
|---|---|---|---|
| `malformed_input` | false | true | true |
| `unsupported_version` | false | true | true |
| `unauthorized` | false | false | true |
| `forbidden_state` | false | true | true |
| `duplicate` | false | true | false |
| `ordering_violation` | false | true | true |
| `expired` | false | true | true |
| `resource_exhausted` | true | true | true |
| `backpressure` | true | true | true |
| `quality_violation` | true | true | true |
| `driver_failure` | true | false | true |
| `shutdown` | false | true | true |

### 10.3 code-specific override（54 個）

| Code | retryable | safe_to_expose | audit_required | 理由 |
|---|---|---|---|---|
| `token_key_unavailable` | true | false | true | key source は recover し得るが exposure で key source detail を露出しない |
| `external_type_leak_blocked` | false | false | true | boundary violation は client retry 条件でない |
| `external_encode_failed` | false | false | true | encoding failure は server/driver defect を示す |
| `buffer_release_failed` | false | false | true | release failure は audit を要し core で retry しない |
| `core_policy_config_invalid` | false | true | true | invalid policy は retry せず stop/reject |
| `runtime_config_missing` | false | true | true | 必須 runtime config 欠落は startup/path を停止 |
| `runtime_config_invalid` | false | true | true | invalid runtime config は startup/path を停止 |
| `secret_unavailable` | true | false | true | secret source は recover し得るが secret detail を露出しない |
| `operation_cancelled` | false | true | true | cancellation を transport failure のように retry しない |
| `capability_not_enabled` | false | true | true | disabled capability は fail open / silent fallback しない |
| `process_panic_detected` | false | true | true | panic 分類は normal transport failure として retry しない |
| `process_crash_detected` | false | true | true | unclean process failure は明示 evidence classification を要する |
| `canonical_serialization_failed` | false | false | true | serializer failure は internal representation detail を露出し得る |
| `canonical_serialization_mismatch` | false | true | true | deterministic verification mismatch は client retry 条件でない |
| `authorization_context_invalid` | false | false | true | invalid authorization context は verifier detail を露出し得る |
| `authorization_policy_denied` | false | false | true | policy denial は policy internals を露出せず audit |
| `authorization_scope_not_allowed` | false | false | true | scope denial は authorization policy detail を露出しない |
| `response_replay_not_available` | false | false | true | missing replay material は server-side cache strategy を露出し得る |
| `secret_rotation_state_unavailable` | true | false | true | rotation state source は recover し得るが secret state detail を露出しない |
| `secret_key_revoked` | false | false | true | revoked key/generation detail を露出しない |
| `dependency_policy_violation` | false | true | true | dependency policy failure は client retry 条件でない |
| `vulnerability_gate_failed` | false | false | true | vulnerability detail は controlled evidence exposure を要する |
| `dependency_missing` | false | true | true | missing dependency/tool は evidence を阻み runtime retry 不可 |
| `sdk_contract_drift_detected` | false | true | true | generated contract drift は client retry 条件でない |
| `internal_control_authorization_denied` | false | false | true | service-to-service denial は internal policy detail を露出しない |
| `ice_candidate_redaction_required` | false | false | true | candidate material は address/network detail を露出し得る |
| `secure_media_key_state_invalid` | false | false | true | invalid media key state は keying detail を露出しない |
| `operator_credential_invalid` | false | false | true | operator credential verification detail を露出しない |
| `operator_action_denied` | false | false | true | privileged action denial は policy internals を露出しない |
| `public_endpoint_auth_required` | false | false | true | public endpoint authentication requirement は verifier detail を露出しない |
| `export_redaction_required` | false | false | true | sensitive artifact material を redaction 前に露出しない |
| `backup_artifact_unavailable` | true | false | true | artifact source は recover し得るが storage/tool detail を露出しない |
| `artifact_integrity_mismatch` | false | true | true | integrity mismatch は client retry 条件でない |
| `release_artifact_provenance_missing` | false | true | true | missing provenance は release claim を阻み runtime retry 不可 |
| `release_artifact_integrity_failed` | false | true | true | release integrity failure は distribution claim を阻む |
| `time_source_untrusted` | false | true | true | untrusted time source は target claim を阻む |
| `time_sync_unavailable` | true | false | true | time sync source は recover し得るが source detail が sensitive な場合がある |
| `timestamp_order_untrusted` | false | true | true | timestamp order を transport failure として retry しない |
| `forwarded_header_untrusted` | false | false | true | forwarded metadata trust failure は trusted upstream detail を露出しない |
| `tls_termination_boundary_invalid` | false | false | true | transport boundary failure は infrastructure detail を露出し得る |
| `public_internal_route_confusion` | false | false | true | route confusion は internal routing detail を露出しない |
| `runtime_reconfiguration_rollback_failed` | true | false | true | rollback execution は recover し得るが failure detail が runtime state を露出し得る |
| `packet_rewrite_owner_violation` | false | false | true | rewrite owner violation は media/backend boundary detail を露出し得る |
| `payload_transform_failed` | true | false | true | driver transform failure は recover し得るが backend detail を露出しない |
| `service_endpoint_scope_conflict` | false | false | true | endpoint scope conflict は internal routing detail を露出しない |
| `state_owner_conflict` | false | false | true | distributed state owner conflict は topology internals を露出しない |
| `split_brain_risk_detected` | false | false | true | split-brain risk は client retry 条件でなく audit を要する |
| `runtime_task_owner_violation` | false | false | true | task ownership violation は runtime/component boundary detail を露出し得る |
| `runtime_task_panic_detected` | false | true | true | task panic は normal transport failure として retry しない |
| `internal_service_identity_invalid` | false | false | true | service identity verification detail を露出しない |
| `internal_service_identity_untrusted` | false | false | true | service trust failure は trust policy internals を露出しない |
| `internal_service_identity_scope_conflict` | false | false | true | service scope conflict は internal topology detail を露出しない |
| `cross_plane_binding_invalid` | false | false | true | binding failure は cross-plane state relation detail を露出し得る |
| `cross_plane_binding_scope_conflict` | false | false | true | cross-plane scope conflict は session topology detail を露出しない |

### 10.4 Signaling reasons

| Code | Category | 意味 |
|---|---|---|
| `missing_correlation_id` | `malformed_input` | command が correlation ID を欠く |
| `malformed_command` | `malformed_input` | command を core type へ decode できない |
| `unsupported_command_version` | `unsupported_version` | command version が不受理 |
| `token_verification_failed` | `unauthorized` | token verifier 結果が command を reject |
| `room_not_accepting_join` | `forbidden_state` | room state が join を受理しない |
| `room_capacity_exceeded` | `resource_exhausted` | room materialization または active room bound 到達 |
| `room_lifetime_exceeded` | `expired` | room lifecycle duration 超過 |
| `room_closed` | `shutdown` | room は既に closed |
| `room_draining` | `shutdown` | room が draining で、明示的 draining allowance なしに command を reject |
| `room_close_not_allowed` | `forbidden_state` | room close/drain transition が現 state で invalid |
| `participant_not_joined` | `forbidden_state` | participant action は joined state を要する |
| `participant_rejected` | `unauthorized` | participant verification または join policy が membership を reject |
| `duplicate_command` | `duplicate` | idempotency rule が duplicate を reject |
| `correlation_mismatch` | `malformed_input` | command correlation が required scope に不一致 |
| `idempotency_payload_mismatch` | `duplicate` | idempotency key が異なる command semantics で再利用 |
| `idempotency_window_expired` | `expired` | idempotency decision/replay window 失効 |
| `replay_not_allowed` | `forbidden_state` | この command class/scope で replay 不許可 |
| `response_replay_not_available` | `driver_failure` | accepted idempotency observation 後に required response replay material が unavailable |
| `command_order_violation` | `ordering_violation` | command が invalid order で到着 |
| `concurrency_conflict` | `ordering_violation` | 同一 serialization scope で concurrent candidates が衝突 |

### 10.5 Token Verification reasons

| Code | Category | 意味 |
|---|---|---|
| `token_missing` | `unauthorized` | required token 不在 |
| `token_malformed` | `malformed_input` | token を verification input へ decode できない |
| `token_signature_invalid` | `unauthorized` | token signature verification 失敗 |
| `token_key_unavailable` | `driver_failure` | bounded lookup 後に verification key source unavailable |
| `token_issuer_mismatch` | `unauthorized` | issuer が accepted policy に不一致 |
| `token_audience_mismatch` | `unauthorized` | audience が accepted policy に不一致 |
| `token_expired` | `expired` | token expiry time 経過 |
| `token_not_yet_valid` | `forbidden_state` | token が現時刻に valid でない |
| `token_required_claim_missing` | `malformed_input` | required claim 不在 |
| `token_unsupported_algorithm` | `unsupported_version` | token algorithm が不受理 |

### 10.6 Authorization Context reasons

| Code | Category | 意味 |
|---|---|---|
| `authorization_context_missing` | `unauthorized` | required verified authorization context 不在 |
| `authorization_context_invalid` | `unauthorized` | authorization context を target boundary が信頼できない |
| `authorization_context_expired` | `expired` | authorization context lifetime 経過 |
| `authorization_policy_denied` | `unauthorized` | communication policy が要求 target/action を deny |
| `authorization_scope_not_allowed` | `forbidden_state` | verified authorization context が required communication scope を欠く |

### 10.7 Cross-Plane Identity / Session Binding reasons

| Code | Category | 意味 |
|---|---|---|
| `cross_plane_binding_not_admitted` | `forbidden_state` | cross-plane binding class が admitted でない |
| `cross_plane_binding_missing` | `forbidden_state` | required cross-plane binding 不在 |
| `cross_plane_binding_invalid` | `malformed_input` | binding material が宣言 references に map できない |
| `cross_plane_binding_scope_conflict` | `forbidden_state` | source/target plane scope の衝突 |
| `cross_plane_binding_lifecycle_conflict` | `forbidden_state` | source/target lifecycle state が binding に valid でない |
| `cross_plane_binding_expired` | `expired` | binding lifetime または source relation 失効 |
| `cross_plane_binding_replay_detected` | `duplicate` | binding replay/idempotency conflict 検出 |

### 10.8 Operator / Admin Authorization reasons

| Code | Category | 意味 |
|---|---|---|
| `operator_credential_missing` | `unauthorized` | required operator/admin credential 不在 |
| `operator_credential_invalid` | `unauthorized` | operator/admin credential verification 失敗 |
| `operator_authorization_context_missing` | `unauthorized` | required operator/admin authorization context 不在 |
| `operator_authorization_context_expired` | `expired` | operator/admin authorization context lifetime 経過 |
| `operator_action_denied` | `unauthorized` | operator/admin policy が要求 privileged action を deny |
| `operator_scope_not_allowed` | `forbidden_state` | 要求 operator/admin target scope が不許可 |

### 10.9 TURN reasons

| Code | Category | 意味 |
|---|---|---|
| `malformed_turn_message` | `malformed_input` | message を core model へ decode できない |
| `unsupported_turn_method` | `unsupported_version` | method が contract で未サポート |
| `unsupported_turn_contract_version` | `unsupported_version` | TURN contract version が不受理 |
| `credential_missing` | `unauthorized` | required credential proof 不在 |
| `credential_invalid` | `unauthorized` | credential verification 失敗 |
| `credential_expired` | `expired` | credential lifetime 超過 |
| `allocation_not_found` | `forbidden_state` | active allocation が必要だが不在/非 active |
| `permission_not_found` | `forbidden_state` | active permission が必要だが不在/非 active |
| `relay_denied` | `forbidden_state` | relay decision が packet を deny |
| `turn_lifetime_violation` | `forbidden_state` | 要求 lifetime が activation 前に policy 違反 |
| `allocation_capacity_exceeded` | `resource_exhausted` | allocation table bound 到達 |
| `allocation_lifetime_exceeded` | `expired` | allocation absolute lifetime cap 到達 |
| `permission_capacity_exceeded` | `resource_exhausted` | permission table bound 到達 |
| `permission_lifetime_exceeded` | `expired` | permission absolute lifetime cap 到達 |
| `channel_bind_lifetime_exceeded` | `expired` | channel binding absolute lifetime cap 到達 |
| `refresh_limit_exceeded` | `expired` | refresh count または cumulative refresh duration cap 到達 |
| `peer_not_allowed` | `forbidden_state` | peer permission policy が peer を reject |
| `secret_generation_not_accepted` | `unauthorized` | credential generation が active rotation policy で不受理 |
| `secret_key_revoked` | `unauthorized` | credential key または generation が revoke 済み |
| `secret_overlap_window_expired` | `expired` | prior generation の credential overlap window 失効 |
| `secret_rotation_state_unavailable` | `driver_failure` | verification boundary で secret rotation state を観測できない |

### 10.10 SFU reasons

| Code | Category | 意味 |
|---|---|---|
| `participant_not_admitted` | `forbidden_state` | endpoint が admitted でない |
| `sfu_session_not_accepting` | `shutdown` | SFU session が draining または closed で新規 decision 不可 |
| `endpoint_quality_not_allowed` | `quality_violation` | endpoint admission を quality policy が reject |
| `endpoint_degraded_by_quality` | `quality_violation` | admitted endpoint を quality policy が degrade |
| `publication_not_allowed` | `forbidden_state` | publication が reject |
| `publication_quality_not_allowed` | `quality_violation` | publication を quality policy が reject/suppress |
| `subscription_not_allowed` | `forbidden_state` | subscription が reject |
| `subscription_quality_not_allowed` | `quality_violation` | subscription を quality policy が reject/suppress |
| `subscription_backpressure_suppressed` | `backpressure` | subscription を backpressure policy が suppress |
| `action_delayed_by_backpressure` | `backpressure` | action を backpressure policy が delay |
| `route_degraded_by_backpressure` | `backpressure` | route を backpressure policy が degrade |
| `endpoint_closed_by_backpressure` | `backpressure` | endpoint を backpressure policy が close |
| `backpressure_recovery_not_allowed` | `backpressure` | backpressure-delayed/degraded/suppressed route state からの recovery を reject |
| `route_suppressed_by_backpressure` | `backpressure` | route state を backpressure policy が suppress |
| `stream_not_found` | `forbidden_state` | stream reference が非 active |
| `route_conflict` | `forbidden_state` | route decision が state と衝突 |
| `packet_suppressed_by_backpressure` | `backpressure` | packet forwarding を suppress |
| `packet_dropped_by_backpressure` | `backpressure` | packet forwarding を backpressure policy が drop |
| `packet_suppressed_by_quality` | `quality_violation` | packet forwarding を suppress |
| `route_suppressed_by_quality` | `quality_violation` | route state を quality policy が suppress |
| `packet_cache_bound_exceeded` | `resource_exhausted` | bounded cache が packet を retain できない |
| `target_unavailable` | `forbidden_state` | target endpoint が受信不可 |
| `endpoint_capacity_exceeded` | `resource_exhausted` | endpoint admission bound 到達 |
| `route_candidate_bound_exceeded` | `resource_exhausted` | route candidate bound 到達 |
| `quality_recovery_not_allowed` | `quality_violation` | recovery を quality policy が reject |
| `unsupported_media_contract_version` | `unsupported_version` | media-facing contract version が不受理 |
| `media_codec_not_supported` | `unsupported_version` | negotiated codec が core routing semantics で未サポート |
| `media_track_not_allowed` | `forbidden_state` | 要求 track direction/kind が不許可 |
| `media_layer_not_available` | `forbidden_state` | 要求 simulcast/SVC layer が forwarding に unavailable |
| `media_payload_mapping_invalid` | `malformed_input` | RTP payload type または header-extension mapping を信頼できない |
| `media_feedback_not_supported` | `unsupported_version` | RTCP feedback capability が contract で未サポート |
| `media_transcode_not_supported` | `unsupported_version` | 要求 operation が transcoding を要し core は所有しない |

### 10.11 Packet Rewrite / Media Transform reasons

| Code | Category | 意味 |
|---|---|---|
| `packet_rewrite_class_not_admitted` | `forbidden_state` | packet rewrite または transform class が admitted でない |
| `packet_rewrite_intent_invalid` | `malformed_input` | core rewrite intent が required field を欠くか route と衝突 |
| `packet_rewrite_owner_violation` | `forbidden_state` | rewrite path が routing/quality または byte ownership を誤った layer へ移そうとする |
| `payload_transform_not_admitted` | `forbidden_state` | admitted 仕様なしに payload transform を要求 |
| `payload_transform_failed` | `driver_failure` | driver payload transform/rewrite/encode step 失敗 |
| `rewrite_copy_bound_exceeded` | `resource_exhausted` | rewrite path copy/allocation bound 超過 |

### 10.12 ICE Candidate / Connectivity reasons

| Code | Category | 意味 |
|---|---|---|
| `ice_candidate_policy_violation` | `forbidden_state` | ICE candidate class または exposure が policy で不許可 |
| `ice_candidate_mapping_invalid` | `malformed_input` | ICE candidate を core-owned reference へ map できない |
| `ice_candidate_redaction_required` | `malformed_input` | ICE candidate material を redaction まで使用不可 |
| `ice_gathering_failed` | `driver_failure` | accepted evidence 前に concrete ICE gathering 失敗 |
| `ice_connectivity_check_failed` | `driver_failure` | driver が ICE connectivity check failure を観測 |
| `ice_consent_expired` | `expired` | ICE consent freshness 失効または失敗 |
| `ice_restart_not_allowed` | `forbidden_state` | 現 state/policy で ICE restart 不許可 |

### 10.13 Secure Media Session reasons

| Code | Category | 意味 |
|---|---|---|
| `secure_media_profile_not_supported` | `unsupported_version` | required secure media profile/version が未サポート |
| `secure_media_handshake_failed` | `driver_failure` | DTLS/SRTP handshake が driver/backend で失敗 |
| `secure_media_peer_verification_failed` | `unauthorized` | secure media peer verification 失敗 |
| `secure_media_protection_not_active` | `forbidden_state` | protected media path が必要だが非 active |
| `secure_media_key_state_invalid` | `unauthorized` | secure media key または generation state を信頼できない |
| `secure_media_session_expired` | `expired` | secure media session lifetime 失効 |
| `secure_media_rekey_required` | `forbidden_state` | 継続前に secure media session の rekey が必要 |

### 10.14 Public Endpoint / Connection Lifecycle reasons

| Code | Category | 意味 |
|---|---|---|
| `public_endpoint_not_allowed` | `forbidden_state` | endpoint class が public surface として admitted でない |
| `public_endpoint_version_unsupported` | `unsupported_version` | public endpoint protocol/contract version が不受理 |
| `public_endpoint_auth_required` | `unauthorized` | required public endpoint authentication 不在 |
| `public_endpoint_upgrade_failed` | `driver_failure` | core entry 前に protocol upgrade/handshake 失敗 |
| `connection_lifecycle_violation` | `forbidden_state` | connection state transition が lifecycle policy で invalid |
| `connection_idle_timeout` | `expired` | connection idle/consent/public lifetime window 失効 |
| `connection_close_policy_violation` | `forbidden_state` | close path が required policy/audit/reference material を満たせない |

### 10.15 Edge / Proxy Trust reasons

| Code | Category | 意味 |
|---|---|---|
| `edge_proxy_not_admitted` | `forbidden_state` | edge または proxy class が admitted でない |
| `forwarded_header_untrusted` | `malformed_input` | forwarded/trusted header を target decision に信頼できない |
| `forwarded_header_chain_invalid` | `malformed_input` | forwarded header chain が hop policy 超過または矛盾 |
| `origin_host_not_allowed` | `forbidden_state` | origin/host/authority/SNI 値が policy で不許可 |
| `client_address_untrusted` | `malformed_input` | client address observation を target decision に信頼できない |
| `tls_termination_boundary_invalid` | `forbidden_state` | TLS termination または downstream security relation が invalid |
| `public_internal_route_confusion` | `forbidden_state` | edge/proxy mapping が public と internal route class を混同 |

### 10.16 Resource Bound reasons

| Code | Category | 意味 |
|---|---|---|
| `signaling_command_queue_bound_exceeded` | `resource_exhausted` | signaling command queue capacity/wait limit 到達 |
| `sfu_transmit_queue_bound_exceeded` | `resource_exhausted` | SFU transmit queue capacity/wait limit 到達 |
| `turn_relay_queue_bound_exceeded` | `resource_exhausted` | TURN relay queue capacity/wait limit 到達 |
| `frame_size_bound_exceeded` | `resource_exhausted` | inbound frame size bound 超過 |
| `admission_capacity_exceeded` | `resource_exhausted` | admission capacity 到達 |
| `retention_duration_exceeded` | `expired` | bounded retention duration 超過 |
| `audit_backlog_bound_exceeded` | `resource_exhausted` | audit sink backlog bound 到達 |
| `metrics_backlog_bound_exceeded` | `resource_exhausted` | metrics export backlog bound 到達 |
| `metric_cardinality_exceeded` | `resource_exhausted` | metric label/cardinality policy bound 超過 |
| `buffer_pool_bound_exceeded` | `resource_exhausted` | driver receive buffer pool bound 到達 |
| `persistence_retry_bound_exceeded` | `resource_exhausted` | persistence retry store entry count/retry count/bytes bound 到達 |
| `persistence_retry_duration_exceeded` | `expired` | persistence retry duration bound 超過 |
| `connection_concurrency_exceeded` | `resource_exhausted` | connection concurrency bound 到達 |
| `memory_pressure_exceeded` | `resource_exhausted` | memory pressure bound 到達 |
| `lock_contention_bound_exceeded` | `resource_exhausted` | bounded serialization queue または lock wait 枯渇 |

### 10.17 Driver Conversion reasons

| Code | Category | 意味 |
|---|---|---|
| `external_decode_failed` | `malformed_input` | external payload を core type へ map できない |
| `unsupported_driver_wire_version` | `unsupported_version` | driver wire encoding version が不受理 |
| `missing_required_wire_field` | `malformed_input` | mapping 前に required external field 不在 |
| `external_enum_unmapped` | `malformed_input` | external enum 値に core mapping なし |
| `external_type_leak_blocked` | `driver_failure` | core entry 前に concrete external type leakage を検出 |
| `external_encode_failed` | `driver_failure` | core event を external に encode できない |
| `network_send_failed` | `driver_failure` | concrete send 失敗 |
| `network_receive_failed` | `driver_failure` | concrete receive 失敗 |
| `persistence_unavailable` | `driver_failure` | concrete persistence unavailable |
| `metrics_export_failed` | `driver_failure` | concrete metrics export 失敗 |
| `buffer_release_failed` | `driver_failure` | driver buffer release 失敗 |
| `driver_shutdown` | `shutdown` | driver shutdown が operation を終了 |

### 10.18 Configuration reasons

| Code | Category | 意味 |
|---|---|---|
| `core_policy_config_invalid` | `malformed_input` | typed core policy configuration が invalid |
| `runtime_config_missing` | `malformed_input` | required runtime configuration 不在 |
| `runtime_config_invalid` | `malformed_input` | runtime configuration が selected driver/entrypoint を初期化できない |
| `secret_unavailable` | `driver_failure` | required secret source が secret 露出なしに unavailable |
| `secret_rotation_required` | `forbidden_state` | active configuration が使用前に newer secret generation を要求 |
| `deployment_topology_unsupported` | `unsupported_version` | selected deployment topology が当該 entrypoint/service boundary で未サポート |
| `service_discovery_unavailable` | `driver_failure` | selected service discovery source unavailable |
| `service_discovery_source_not_admitted` | `forbidden_state` | selected service discovery source class が admitted でない |
| `service_endpoint_resolution_failed` | `driver_failure` | target service の endpoint を解決できない |
| `service_endpoint_stale` | `expired` | cached/generation-scoped service endpoint が stale |
| `service_endpoint_fallback_not_allowed` | `forbidden_state` | fallback endpoint が policy で admitted でない |
| `service_endpoint_scope_conflict` | `forbidden_state` | 解決 endpoint が expected scope に不一致 |
| `service_endpoint_contract_mismatch` | `unsupported_version` | 解決 endpoint contract/version が不受理 |
| `node_affinity_required` | `forbidden_state` | command が不在の affinity/node-local owner を要する |
| `node_state_unavailable` | `driver_failure` | required node-local state を観測できない |
| `cross_node_route_not_allowed` | `forbidden_state` | route が contract 不許可の topology boundary を越える |
| `distributed_state_not_admitted` | `forbidden_state` | distributed state class が admitted でない |
| `state_replication_not_admitted` | `forbidden_state` | admitted policy なしに state replication を要求 |
| `consensus_not_admitted` | `forbidden_state` | admitted policy なしに consensus/leader election を要求 |
| `failover_not_proven` | `forbidden_state` | admitted policy/evidence なしに failover を要求/主張 |
| `state_owner_conflict` | `ordering_violation` | 同一 state scope で複数 owner が衝突 |
| `split_brain_risk_detected` | `ordering_violation` | owner-scoped state で split-brain risk 検出 |
| `replication_lag_bound_exceeded` | `expired` | replication lag/handoff window が accepted bound 超過 |
| `runtime_reconfiguration_not_allowed` | `forbidden_state` | runtime reconfiguration class が admitted でない |
| `configuration_generation_missing` | `malformed_input` | required configuration generation reference 不在 |
| `configuration_generation_conflict` | `ordering_violation` | 提案 configuration generation が active generation/order と衝突 |
| `runtime_reconfiguration_validation_failed` | `malformed_input` | 提案 runtime generation が validation 失敗 |
| `runtime_reconfiguration_apply_not_allowed` | `forbidden_state` | target surface/active scope で runtime reconfiguration apply 不許可 |
| `runtime_reconfiguration_drain_required` | `forbidden_state` | runtime reconfiguration apply 前に drain/restart が必要 |
| `runtime_reconfiguration_rollback_required` | `forbidden_state` | evidence として扱う前に rollback が必要 |
| `runtime_reconfiguration_rollback_failed` | `driver_failure` | runtime reconfiguration rollback execution 失敗 |

### 10.19 Internal Control Plane reasons

| Code | Category | 意味 |
|---|---|---|
| `internal_control_message_invalid` | `malformed_input` | internal control message を contract へ map できない |
| `internal_control_version_unsupported` | `unsupported_version` | internal control contract version が不受理 |
| `internal_control_authorization_missing` | `unauthorized` | required internal control authorization context 不在 |
| `internal_control_authorization_denied` | `unauthorized` | internal control authorization policy が request を deny |

### 10.20 Internal Service Identity / Trust reasons

| Code | Category | 意味 |
|---|---|---|
| `internal_service_identity_source_not_admitted` | `forbidden_state` | internal service trust class または identity source が admitted でない |
| `internal_service_identity_missing` | `unauthorized` | required internal service identity 不在 |
| `internal_service_identity_invalid` | `unauthorized` | service identity material を map/verify できない |
| `internal_service_identity_untrusted` | `unauthorized` | service identity を target path に信頼できない |
| `internal_service_identity_scope_conflict` | `forbidden_state` | service identity scope が target service/topology/contract に不一致 |
| `internal_service_peer_verification_failed` | `unauthorized` | internal service peer verification 失敗 |
| `internal_service_credential_expired` | `expired` | service credential または peer proof lifetime 失効 |
| `internal_service_trust_policy_missing` | `malformed_input` | required service trust policy 不在 |

### 10.21 Operation Lifecycle reasons

| Code | Category | 意味 |
|---|---|---|
| `operation_deadline_exceeded` | `expired` | command/operation deadline 超過 |
| `operation_cancelled` | `shutdown` | 該当 boundary 後に operation を cancel |
| `capability_not_enabled` | `forbidden_state` | required capability/gated surface が enable でない |

### 10.22 Runtime Task / Worker Lifecycle reasons

| Code | Category | 意味 |
|---|---|---|
| `runtime_task_class_not_admitted` | `forbidden_state` | runtime task/worker class が admitted でない |
| `runtime_task_owner_violation` | `forbidden_state` | task owner/supervision scope が boundary 違反 |
| `runtime_task_supervision_missing` | `malformed_input` | required task supervision scope 不在 |
| `runtime_task_detached_not_allowed` | `forbidden_state` | detached task execution が admitted でない |
| `runtime_task_spawn_failed` | `driver_failure` | runtime が required task を spawn できない |
| `runtime_task_join_failed` | `driver_failure` | task join/wait observation 失敗 |
| `runtime_task_cancel_failed` | `driver_failure` | task cancellation 失敗または観測不可 |
| `runtime_task_panic_detected` | `shutdown` | runtime task/worker panic を観測 |
| `runtime_task_queue_bound_exceeded` | `resource_exhausted` | runtime task queue/worker mailbox/join wait/cancellation wait bound 枯渇 |

### 10.23 Operational / Evidence Boundary reasons

| Code | Category | 意味 |
|---|---|---|
| `readiness_not_satisfied` | `forbidden_state` | claim に必要な readiness component が未充足 |
| `health_probe_unavailable` | `driver_failure` | entrypoint/driver failure で health/readiness probe 実行不可 |
| `maintenance_mode_active` | `shutdown` | maintenance mode が要求 action を阻む |
| `admin_action_not_allowed` | `forbidden_state` | operator/admin action が boundary/policy で不許可 |
| `measurement_normalization_failed` | `malformed_input` | raw measurement を required unit へ正規化できない |
| `time_observation_unavailable` | `driver_failure` | required time observation unavailable |
| `atomic_commit_failed` | `driver_failure` | core decision 後の commit step 失敗 |
| `compensation_required` | `driver_failure` | compensation が必要だが successful compensation がまだ存在しない |
| `compensation_failed` | `driver_failure` | compensation execution 失敗 |
| `canonical_serialization_failed` | `driver_failure` | canonical encoding を生成できない |
| `canonical_serialization_mismatch` | `malformed_input` | deterministic canonical verification が expected digest/hash に不一致 |
| `process_panic_detected` | `shutdown` | process panic を観測 |
| `process_crash_detected` | `shutdown` | process crash を観測 |
| `unclean_shutdown_detected` | `shutdown` | 前回 shutdown が graceful drain/audit completion evidence を欠く |
| `supervisor_restart_observed` | `shutdown` | supervisor restart を観測し readiness を含意してはならない |
| `session_resumption_not_allowed` | `forbidden_state` | SDK/server session resumption が contract で不許可 |
| `sdk_reconnect_exhausted` | `expired` | SDK local reconnect attempts/window 枯渇 |
| `fixture_invalid` | `malformed_input` | fixture/scenario data shape が test に invalid |
| `fixture_redaction_required` | `malformed_input` | fixture/scenario data を redaction まで使用不可 |
| `observability_signal_invalid` | `malformed_input` | telemetry/audit/log signal が closed taxonomy に不一致 |
| `telemetry_sampling_policy_missing` | `malformed_input` | signal class に required sampling policy が不在 |
| `alert_signal_not_allowed` | `forbidden_state` | alerting signal が evidence/runtime class に不許可 |
| `observability_export_not_allowed` | `forbidden_state` | export sink/signal class が policy で不許可 |
| `dependency_policy_violation` | `forbidden_state` | dependency が allowlist/ownership policy 不満足 |
| `dependency_missing` | `malformed_input` | evidence/build gate に required dependency/tool 不在 |
| `license_policy_violation` | `forbidden_state` | dependency license が policy で不受理 |
| `vulnerability_gate_failed` | `forbidden_state` | vulnerability policy gate が dependency set を reject |
| `toolchain_version_mismatch` | `malformed_input` | toolchain version が accepted profile に不一致 |
| `lockfile_drift_detected` | `malformed_input` | lockfile/generated dependency material が accepted state と相違 |
| `sdk_contract_generation_failed` | `driver_failure` | evidence として扱う前に SDK public API contract generation 失敗 |
| `sdk_contract_drift_detected` | `malformed_input` | generated SDK public API artifact が source contract と相違 |
| `sdk_public_api_unmapped` | `malformed_input` | public SDK API surface に source contract mapping 不在 |
| `sdk_golden_mismatch` | `malformed_input` | SDK generated fixture/golden が canonical source と相違 |
| `sdk_platform_projection_invalid` | `malformed_input` | platform projection が canonical contract semantics を保持できない |
| `feature_out_of_scope` | `forbidden_state` | 要求 feature が v0.2 initial scope 外 |
| `feature_admission_not_documented` | `forbidden_state` | 要求 feature が required な仕様 admission を欠く |
| `chat_not_supported` | `forbidden_state` | chat semantics が v0.2 initial scope で未サポート |
| `recording_not_supported` | `forbidden_state` | recording workflow が v0.2 initial scope で未サポート |
| `screen_share_not_supported` | `forbidden_state` | screen share workflow が v0.2 initial scope で未サポート |
| `datachannel_not_supported` | `forbidden_state` | DataChannel application semantics が v0.2 initial scope で未サポート |
| `ui_workflow_not_supported` | `forbidden_state` | UI/end-user workflow が v0.2 initial scope で未サポート |
| `media_capture_not_supported` | `forbidden_state` | media capture workflow が core/server feature として未サポート |
| `regulated_workflow_not_supported` | `forbidden_state` | regulated workflow が generic core feature として未サポート |

### 10.24 Export / Backup Artifact reasons

| Code | Category | 意味 |
|---|---|---|
| `export_surface_not_allowed` | `forbidden_state` | export/backup surface が admitted でない |
| `export_redaction_required` | `malformed_input` | artifact を redaction まで採用不可 |
| `backup_artifact_unavailable` | `driver_failure` | artifact を driver/tool から生成/取得できない |
| `artifact_integrity_mismatch` | `malformed_input` | artifact integrity check 失敗 |
| `artifact_restore_not_allowed` | `forbidden_state` | artifact を admission なしに restore/import input として使用 |

### 10.25 Release Artifact / Distribution / Provenance reasons

| Code | Category | 意味 |
|---|---|---|
| `release_artifact_not_built` | `malformed_input` | 主張 class の release artifact が未 build |
| `release_artifact_provenance_missing` | `malformed_input` | release artifact provenance が required field を欠く |
| `release_artifact_integrity_failed` | `malformed_input` | artifact digest/signature verification 失敗 |
| `release_version_mismatch` | `malformed_input` | source/package/version が主張 release に不一致 |
| `distribution_channel_not_allowed` | `forbidden_state` | distribution channel が admitted でない |

### 10.26 Time Synchronization / Clock Skew reasons

| Code | Category | 意味 |
|---|---|---|
| `clock_skew_exceeded` | `malformed_input` | 測定 skew が accepted policy 超過 |
| `time_source_untrusted` | `malformed_input` | time source を target claim に信頼できない |
| `time_sync_unavailable` | `driver_failure` | required time synchronization observation unavailable |
| `timestamp_order_untrusted` | `malformed_input` | timestamp order を target comparison に信頼できない |

### 10.27 reason 総数

本 catalog の reason code は上記 10.4〜10.26 の各 category 別表で **合計 274 個（unique）** であり、全て閉集合です。12 個の cross-cutting category と 54 個の code-specific metadata override（いずれも 274 個の subset）を持ちます。catalog にない code を実装都合で追加してはなりません。

## 11. external error mapping

core reason category/code を HTTP、WebSocket、STUN/TURN、SDK error wrapper、CLI exit へ写す際、authoritative reason を失わないための owner と公開制御を固定します。external code は authoritative ではありません。authoritative failure は cataloged reason category/code または pre-core driver conversion reason のままです。

### 11.1 mapping 境界

| Surface | Owner | 規則 |
|---|---|---|
| authoritative reason category/code | core | 本章 §10 |
| reason exposure metadata | core | retryable、safe_to_expose、audit_required |
| external protocol status/wrapper | driver/sdk/cli | reason を失わない projection |
| raw driver error detail | driver | non-authoritative かつ redacted |
| audit event | core model + driver sink | external response の代替ではない |

### 11.2 mapping field

emit され得る external 非成功 response は次を定義しなければなりません。external surface / external status・wrapper class / safe かつ利用可能なときの correlation reference / `safe_to_expose = true` のときの exposed reason category/code / `safe_to_expose = false` のときの redacted opaque error reference / audit 必要時の audit event relation / reason metadata が retry を許すときのみ retry hint。実装はこれらを `ExternalErrorProjectionInput` のような名前付き入力型に通します。reason が safe to expose でない場合、external response は secret/key/token/backend detail を漏らしてはなりません。

### 11.3 surface mapping

| Surface | Mapping owner | 必須保持 |
|---|---|---|
| HTTP | network driver | category/code または opaque error reference |
| WebSocket | network driver | close/error event が traceability を保持 |
| STUN/TURN | TURN wire driver | TURN error 表現 + core reason relation |
| SDK TypeScript / Android / iOS | sdk | server-originated 時の server reason 保持 |
| CLI | entrypoints/cli | exit classification + correlation/report reference |
| logs/traces | observability driver | redacted diagnostics のみ |

concrete status number や wrapper class 名は、surface 固有 API 仕様が固定するまで driver/sdk implementation detail です。

### 11.4 safe exposure

| Metadata | external behavior |
|---|---|
| `safe_to_expose = true` | category/code を exposure 可 |
| `safe_to_expose = false` | generic external failure class + opaque reference を exposure |
| `retryable = true` | surface が対応するなら retry hint を exposure 可 |
| `audit_required = true` | audit event relation を記録、さもなくば path は close evidence に使えない |

driver は exception text から safe exposure を推測してはなりません。

### 11.5 external error mapping failure

| Failure | 必須 reason |
|---|---|
| core event を external に encode できない | `external_encode_failed` |
| error response emit 中の network send 失敗 | `network_send_failed` |
| SDK が malformed server event を decode できない | SDK-local closed error。server reason を発明しない |
| response emit 前の driver shutdown | `driver_shutdown` |

error response emission 失敗時、original reason は domain reason のまま残り、emission 失敗は driver observation です。

## 12. 禁止事項

- driver が rejected decision を success response に変える。
- entrypoints が binary ごとに別 result shape を定義する。
- port intent を driver execution success として扱う。
- driver observation が明示的 compensating transition なしに domain decision を上書きする。
- per-step outcome rule なしに partial success が現れる。
- compensation が original decision evidence を消去する。
- 非成功 outcome が cataloged reason を欠く。
- success outcome が fake reason を持つ。
- `CorrelationId` 単体を duplicate command identity として扱う。
- replay window expiry 後に response を replay する。
- driver cache hit を authoritative command acceptance として用いる。
- SDK reconnect が silent に server accepted result を作る。
- conflicting payload replay を idempotent observation として受理する。
- response replay が original reason または audit failure を隠す。
- HTTP status または WebSocket close code が core reason を置き換える。
- unsafe reason detail を、SDK/platform wrapper が text を期待するからと露出する。
- non-retryable reason に retry hint を露出する。
- core 非成功 decision 後に external response が success を主張する。
- SDK が local decode failure に対し server reason を発明する。
- logs/traces を external error authority として用いる。
- decision reason が free-text のみになる。
- driver-local error が core reason に変換されない。
- audit event が reason catalog に接続しない。
- reason catalog にない code を実装都合で追加する。

## 13. fail-closed / 不変条件

- 非成功 outcome は必ず cataloged reason を持つ（fail-closed）。
- canonical payload digest を生成できない場合 `canonical_serialization_failed` で fail-closed。
- response replay は全条件充足時のみ。欠ける場合 fail-closed。
- `safe_to_expose = false` の reason は external で secret/key/token/backend detail を漏らさない。
- `audit_required = true` の reason は audit relation を記録できないと close evidence に使えない。
- reason は閉集合。catalog 外 code は不可。

## 14. 判断が崩れる条件（collapse conditions）

- use case が driver の意味解釈する free-form result を返す。
- external status code が authoritative outcome になる。
- audit event と external response が異なる reason から導出される。
- accepted decision 後の driver execution failure が隠蔽される。
- result shape が仕様の更新なしに Signaling/SFU/TURN/SDK/CLI 間で相違する。
- compensation/transaction semantics が final external response のみから推測される。
- idempotency scope が implicit になる。
- duplicate handling が仕様の更新なしに driver/entrypoint で相違する。
- replay store が domain source-of-truth になる。
- SDK pending replay が server-side command evaluation を bypass する。
- response replay が prior result reference または canonical payload comparison を欠く。
- external surface が core reason への traceability を失う。
- unsafe reason が token/key/backend detail を露出する。
- driver exception text が public reason になる。
- external encode failure が original domain decision を隠す。
- SDK platform mapping が server-origin reason semantics を変える。
- decision reason が free-text のみになる。
- driver-local error が core reason に変換されない。
- SDK が server-side reason を別 semantics に変換する。
- audit event が reason catalog に接続しない。
- reason catalog にない code を実装都合で追加する。
