# core-transport-and-media

状態: SSOT 統合版
日付: 2026-06-28 JST

## 目的

本章は arcRTC v0.2 Kernel の `core/transport` が所有する pure transport contract、ならびに SDP/ICE negotiation、ICE candidate policy / connectivity lifecycle、media codec/track/layer negotiation、packet rewrite / media transform boundary、secure media session lifecycle の現行完全仕様を、本章のみで再現実装可能な粒度で内在化することを目的とします。本章は Sans-IO semantics を core 所有とし、socket / runtime / str0m concrete type / browser・native WebRTC API などの I/O 実装を driver 所有として分離します。本章は concrete SDP grammar、ICE agent、DTLS/SRTP runtime、codec implementation、transcoding、media engine runtime を実装済みとして主張しません。

依存方向の表記: `A <- B` は「B が A に依存」を意味します。Signaling / SFU / TURN の pure semantics は core が所有し、I/O と分離されます。

---

## 1. core transport contract（Sans-IO transport contract）

### 1.1 境界（所有権）

`core/transport` は次を所有します（必須）。

- transport command / event の core-owned type
- WebRTC transport capability の抽象表現
- SDP / ICE-adjacent semantic fragment の抽象表現
- RTP / RTCP packet semantic view との接続点
- transport-level version / capability negotiation semantics
- transport failure reason への接続

driver は次を所有します（必須）。

- str0m concrete event / state / error
- UDP / TCP / HTTP / WebSocket socket I/O
- browser / native WebRTC API
- DTLS / SRTP / ICE stack implementation detail
- concrete SDP string encoding / parsing backend
- runtime task / worker execution

transport contract は Sans-IO semantics であり、socket、runtime、str0m concrete type、browser/native WebRTC API を所有してはなりません（禁止）。

### 1.2 Core Transport Types（閉集合）

| Type family | Owner | 規則 |
|---|---|---|
| `TransportCommand` | core | driver implementation に対する abstract command |
| `TransportEvent` | core | external transport event を core meaning へ変換したもの |
| `TransportCapability` | core | capability negotiation の意味論 |
| `SessionDescriptionRef` | core | SDP payload そのものではなく opaque/validated semantic reference |
| `IceCandidateRef` | core | candidate relay / negotiation intent の core-owned reference |
| `PacketSemanticView` | core | routing に必要な borrowed packet view |

`SessionDescriptionRef` と `IceCandidateRef` は、core が external concrete library type を parse / retain することを許可しません。これらは単独で cross-plane binding を生成しません。

### 1.3 SDP / ICE 規則（transport level）

- Signaling は offer / answer / ICE candidate intent を relay してよい（許可）。
- core transport はその intent の version、correlation、policy 境界を validate してよい（許可）。
- concrete SDP grammar parsing、ICE agent execution、connectivity checks、candidate gathering は driver 所有の implementation detail です（必須）。
- driver が external SDP / ICE material を core-owned reference shape へ map できない場合、core entry 前に fail-close しなければなりません（必須・driver conversion 境界）。

### 1.4 RTP / RTCP 規則（transport level）

- RTP / RTCP raw bytes は driver 所有です（必須）。
- core transport と core SFU は borrowed semantic view のみを observe してよい（許可、§4 packet buffer lifetime 規則準拠）。
- core transport は packet bytes、buffer lease、packet cache、transmit queue、parser crate object、str0m packet object を retain してはなりません（禁止）。

### 1.5 Version / Capability 規則

- transport-facing version / capability negotiation は protocol version 体系に接続しなければなりません（必須）。
- unsupported transport contract version は `unsupported_media_contract_version` または適用可能な protocol-specific reason を使用しなければなりません（必須）。
- transport version mismatch を retryable な free-text error として扱ってはなりません（禁止）。

### 1.6 Driver Mapping 規則

- `drivers/webrtc-str0m` が WebRTC transport port を実装します。
- `drivers/network` が concrete network I/O を供給します。
- いずれの driver も alternate transport semantics を定義してはなりません（禁止）。
- str0m が event を emit したとき、driver はそれを core-owned `TransportEvent` へ map するか、cataloged reason で conversion を fail させなければなりません（必須）。
- core が `TransportCommand` を emit したとき、driver はそれを str0m / browser / native / network action へ translate してよいが、domain decision を変更してはなりません（禁止）。

### 1.7 禁止事項（core transport contract）

- core が `str0m::*` を import する。
- core が socket / runtime / browser / native WebRTC concrete type を import する。
- driver が concrete SDP / ICE parser object を core API として exposure する。
- core が raw RTP / RTCP bytes を保存する。
- driver が transport implementation の都合で Signaling / SFU / TURN の accept / reject を決定する。
- transport reference や connectivity observation が単独で cross-plane binding を生成する。
- transport version mismatch を retryable free-text error として扱う。

### 1.8 fail-closed / collapse 条件（core transport contract）

次のいずれかが成立する場合、本契約の判断は崩れます。

- core transport が str0m wrapper になる。
- SDP / ICE concrete type が core に入る。
- ICE connectivity や secure media readiness が Signaling relay や listener startup から推定される。
- packet bytes や buffer lease が core 所有になる。
- driver transport event が core domain decision semantics を変更する。
- transport reference や event が cross-plane binding 規則を bypass する。
- unsupported transport version が cataloged reason を欠く。

---

## 2. SDP / ICE Negotiation 境界

本節は SDP offer/answer、ICE candidate、capability/version negotiation の owner と fail-closed 条件を細分化します。core は SDP string、ICE candidate string、str0m event、browser/native WebRTC concrete type を所有せず、`SessionDescriptionRef`、`IceCandidateRef`、capability/version/correlation の意味論だけを所有します。

### 2.1 境界（owner）

| Concern | Owner | 規則 |
|---|---|---|
| offer / answer relay intent | core/signaling | room/participant/correlation/version 境界を評価する |
| ICE candidate relay intent | core/signaling | accepted participant と ordering 境界を評価する |
| transport capability semantics | core/transport | accepted version と capability relation を評価する |
| media codec/track/layer capability semantics | core/transport + core/sfu | accepted media contract と SFU intent |
| SDP grammar parse / serialize | driver | concrete parser/backend detail |
| ICE gathering / connectivity check | driver | concrete stack detail |
| ICE candidate exposure / consent observation | core policy + driver observation | detailed lifecycle は §3 に従う |
| DTLS / SRTP setup | driver | crypto/session implementation detail |
| SDK public API mapping | sdk | Signaling-only contract を platform API へ写す |
| executable wiring | entrypoints | selected driver と typed configuration の組み立て |

### 2.2 Negotiation Flow（inbound 順序、必須）

inbound negotiation material は次の順序を守らなければなりません。

1. driver または sdk boundary が external SDP/ICE material を receive する。
2. driver が wire shape、size、required field、driver wire version、concrete decode を enforce する。
3. driver が external material を core-owned reference shape へ map する。
4. core/signaling が room/participant/order/version/correlation を評価する。
5. core が accepted relay / rejected relay / protocol violation の decision を返す。
6. driver/sdk が core decision を external response/event へ map する（reason 変更禁止）。

- concrete ICE connectivity success や media transport readiness は Signaling relay acceptance によって証明されません（必須）。
- offer/answer/ICE relay acceptance は、admitted cross-plane binding なしに Signaling participant、SFU endpoint、TURN allocation、secure media session を bind しません（必須）。

### 2.3 Version / Capability 規則

protocol contract version と driver wire encoding version の分離を保たなければなりません（必須）。

| Version surface | Owner | Failure reason |
|---|---|---|
| Signaling command/event version | core | `unsupported_command_version` |
| media-facing transport contract version | core/transport | `unsupported_media_contract_version` |
| driver wire encoding version | driver | `unsupported_driver_wire_version` |
| SDK public API version | sdk | SDK-local mapping、server reason を保存 |

- capability は version を置換しません。
- capability は accepted version 内の optional behavior を選択してよいが、required Signaling state transitions や SFU routing semantics を変更してはなりません（禁止）。
- codec/track/layer capability を SDP string 単独から推定してはならず、media negotiation mapping を通過させなければなりません（必須）。

### 2.4 Ordering 規則

- offer/answer/ICE relay ordering は core Signaling semantics です。
- driver は receive order を保持してよいが、command ordering validity を決定してはなりません（禁止）。
- rejected ordering は `command_order_violation` を使用します。
- duplicate command は idempotency semantics が duplicate を reject した場合 `duplicate_command` を使用します。

### 2.5 Failure Mapping（SDP/ICE negotiation）

| Failure | Required reason |
|---|---|
| external SDP/ICE material が reference shape へ decode できない | `external_decode_failed` |
| candidate class または exposure policy が reject | `ice_candidate_policy_violation` |
| candidate connectivity または consent observation が失敗 | `ice_connectivity_check_failed` または `ice_consent_expired` |
| required wire field 欠落 | `missing_required_wire_field` |
| external enum/attribute class が core reference へ map できない | `external_enum_unmapped` |
| Signaling command version 非対応 | `unsupported_command_version` |
| media-facing contract version 非対応 | `unsupported_media_contract_version` |
| media codec/profile 非対応 | `media_codec_not_supported` |
| media payload/track/layer mapping 不正 | `media_payload_mapping_invalid` |
| participant 未 join | `participant_not_joined` |
| command が invalid order で到着 | `command_order_violation` |
| room が draining | `room_draining` |
| room が closed | `room_closed` |
| network send 失敗 | `network_send_failed` |
| network receive 失敗 | `network_receive_failed` |
| driver shutdown | `driver_shutdown` |
| required cross-plane binding が absent/invalid | cross-plane binding reason |

driver-local parser/backend error を free-text core reason にしてはなりません（禁止）。

### 2.6 SDK Parity 規則

TypeScript / Android / iOS SDK は platform idiom を変えてよいが、次を保存しなければなりません（必須）。

- offer / answer / ICE candidate relay の同一 semantic command/event set
- 同一 correlation propagation
- rejection の同一 server reason category/code
- 同一 version negotiation behavior
- 同一 Signaling-only boundary

SDK は PeerConnection / media readiness を server-side Signaling success として exposure してはなりません（禁止）。

### 2.7 禁止事項（SDP/ICE negotiation）

- core が concrete SDP string を parse する。
- core が ICE candidate concrete parser object を保存する。
- driver が core reject 後に local convenience で offer/answer/ICE ordering を accept する。
- Signaling relay acceptance を DTLS/SRTP/media readiness として報告する。
- SDP codec/payload field を単独で accepted SFU media negotiation として扱う。
- SDP/ICE relay success を単独で cross-plane binding として扱う。
- SDK platform 差が offer/answer/ICE command semantics を変更する。
- unsupported version が silent に fallback する。

### 2.8 collapse 条件（SDP/ICE negotiation）

- SDP/ICE concrete type が core に入る。
- driver wire version が core protocol version として扱われる。
- ICE connectivity result が Signaling relay event から推定される。
- ICE candidate exposure/restart/connectivity/consent lifecycle が §3 を bypass する。
- malformed negotiation material が concrete external type として core へ到達する。
- server rejection reason が SDK/platform mapping で消去される。
- media negotiation support が media negotiation 規則なしに暗黙化される。
- cross-plane binding が offer/answer/ICE correlation 単独から推定される。

---

## 3. ICE Candidate Policy / Connectivity Lifecycle

本節は SDP/ICE relay intent、candidate gathering、candidate exposure policy、relay-only policy、ICE restart、connectivity check、consent freshness が Signaling relay acceptance や driver implementation detail と混同されないよう固定します。Signaling による ICE candidate relay の acceptance は connectivity を証明しません（必須）。

### 3.1 境界（owner）

| Concern | Owner | 規則 |
|---|---|---|
| candidate relay intent | core/signaling | room, participant, ordering, correlation を評価 |
| candidate exposure policy | core/transport policy + entrypoints configuration | host/srflx/relay/mDNS/redaction rule を固定 |
| candidate gathering/execution | driver/WebRTC backend | concrete ICE agent detail |
| connectivity check observation | driver | runtime observation, not Signaling success |
| consent freshness observation | driver | observation を core-owned event/reason へ変換 |
| ICE restart authorization | core/signaling + core/transport | restart intent と allowed state |
| evidence | reports | connectivity class と scope が必須 |

### 3.2 Candidate Classes（閉集合）

v0.2 initial architecture の candidate class は次に限定します。新規 class は仕様の更新が必須です。

| Class | 意味 | 規則 |
|---|---|---|
| `host_candidate_ref` | policy 適用後の host candidate reference | raw address exposure には明示 policy が必須 |
| `srflx_candidate_ref` | server-reflexive candidate reference | identity proof ではない |
| `relay_candidate_ref` | TURN relay candidate reference | claim 時は TURN allocation/permission relation が必須 |
| `mdns_candidate_ref` | mDNS-obfuscated candidate reference | raw local address は hidden を保つ |
| `trickle_candidate_ref` | incremental candidate relay | ordering/correlation が必須 |
| `ice_restart_intent` | restart negotiation intent | prior state と restart policy が必須 |
| `connectivity_observation` | driver observed connectivity result | diagnostic/evidence class、relay acceptance ではない |
| `consent_freshness_observation` | driver observed consent state | 期限切れ/失敗時は closed reason |

### 3.3 Policy 規則

ICE policy は次を宣言しなければなりません（必須）。

- accepted candidate classes
- relay-only または host/srflx allowance
- mDNS / raw address exposure rule
- candidate size と field validation
- trickle ordering rule
- ICE restart allowance
- connectivity/consent observation mapping
- audit event relation

required な箇所で candidate policy が absent の場合、candidate handling は fail-close しなければなりません（必須）。

### 3.4 Failure Mapping（ICE candidate/connectivity）

| Failure | Required reason |
|---|---|
| candidate class が policy で不許可 | `ice_candidate_policy_violation` |
| candidate が core reference へ map できない | `ice_candidate_mapping_invalid` |
| candidate が redaction 必須の address/material を exposure | `ice_candidate_redaction_required` |
| relay evidence 前に candidate gathering 失敗 | `ice_gathering_failed` |
| connectivity check 失敗 | `ice_connectivity_check_failed` |
| consent freshness 期限切れ/失敗 | `ice_consent_expired` |
| 現 state で ICE restart 不許可 | `ice_restart_not_allowed` |
| external ICE material が decode できない | `external_decode_failed` |
| command ordering 不正 | `command_order_violation` |
| network receive 失敗 | `network_receive_failed` |
| network send 失敗 | `network_send_failed` |

### 3.5 Audit / Evidence 規則

- ICE candidate / connectivity decision は audit event type `ice_candidate_connectivity_decision` を使用します。event は `CorrelationId`、candidate/connectivity class、policy reference、materialize 時の participant/room reference、driver connectivity evidence 使用時は observation class を carry しなければなりません（必須）。
- ICE evidence は次を記録しなければなりません（必須）: candidate/connectivity class、accepted policy class、correlation ID、materialize 時の participant/room reference、relay-only/exposure policy result、関連時の trickle/restart relation、claim 時の connectivity/consent observation class、expected outcome、actual outcome、non-success の cataloged reason、close-not-claimed scope。
- candidate relay evidence は connectivity や consent freshness を証明しません。relay candidate evidence は、admitted cross-plane binding と target-plane evidence なしに TURN allocation/permission や SFU endpoint binding を証明しません（必須）。

### 3.6 禁止事項（ICE candidate/connectivity）

- policy なしで raw host address exposure を accept する。
- Signaling relay acceptance から ICE connectivity success を推定する。
- driver ICE backend が room/participant ordering を決定する。
- ICE restart が silent に room/participant/SFU/TURN state を再生成する。
- binding class なしに candidate/connectivity observation を cross-plane binding として扱う。
- mDNS candidate を audit/log/report で raw local address に展開する。
- evidence scope なしに connectivity observation を production readiness として使う。

### 3.7 collapse 条件（ICE candidate/connectivity）

- candidate exposure policy が暗黙化される。
- ICE restart に authorization/state rule がない。
- consent freshness 失敗が generic network failure のみで記録される。
- candidate relay と connectivity evidence が混同される。
- driver ICE object が core state に入る。
- ICE evidence が cross-plane binding 規則なしに TURN/SFU/secure media binding evidence として使われる。

---

## 4. Media Codec / Track / Layer Negotiation

本節は SDP、RTP payload type、SSRC、RID/MID、simulcast/SVC、RTX/FEC、RTCP feedback が SFU routing semantics や driver parser detail と混同されないよう、owner と fail-closed 条件を固定します。codec implementation、transcoding、media engine runtime、browser/native media readiness は主張しません。

### 4.1 境界（owner）

| Concern | Owner | 規則 |
|---|---|---|
| media-facing contract semantics | core/transport + core/sfu | accepted capability と routing intent |
| codec implementation | driver/browser/native/webrtc backend | concrete encoder/decoder detail |
| SDP/RTP payload mapping | driver conversion + core accepted reference | parser detail は単独で semantic authority になれない |
| publication/subscription track policy | core/sfu | accepted track/layer decision |
| simulcast/SVC layer selection | core intent, driver execution | silent な driver-only subscription semantics 禁止 |
| RTX/FEC/RTCP feedback parsing | driver conversion | core は admitted feedback/reference のみを見る |
| transcode/media transform execution | driver | §5 が admit しない限り禁止 |

payload type、SSRC、RID、MID、codec string は、accepted core-owned reference へ map されるまで authoritative core identity になりません（必須）。track publication/subscription mapping は、cross-plane binding 規則が admit しない限り Signaling participant を SFU endpoint へ bind しません（必須）。

### 4.2 Negotiation Classes（閉集合）

v0.2 initial architecture の media negotiation class は次に限定します。新規 class は仕様の更新が必須です。

| Class | 意味 | 規則 |
|---|---|---|
| `codec_capability_offer` | offered codec/profile capability | core は semantic capability を accept/reject 可 |
| `track_publication_offer` | endpoint が media track を publish したい | SFU publication decision へ map |
| `track_subscription_request` | endpoint が media track/layer を subscribe したい | SFU subscription decision へ map |
| `payload_type_mapping` | external RTP payload type を accepted media reference へ | driver が map、core が validate |
| `simulcast_layer_selection` | RID/layer selection intent | core selection、driver execution |
| `svc_layer_selection` | SVC dependency/layer selection intent | 明示 support 必須 |
| `rtx_fec_feedback_support` | retransmission/recovery support | driver execution と core policy relation |

### 4.3 Mapping 規則

codec/track/layer mapping は次を定義しなければなりません（必須）。

- media contract version
- accepted codec/profile set
- payload type mapping rule
- SSRC/RID/MID mapping rule
- track と stream reference relation
- track/stream claim が Signaling/SFU/TURN/secure media plane を跨ぐ場合の cross-plane binding relation
- layer selection rule
- unsupported feature reason
- audit/event relation

external SDP/RTP field は map・accept されるまで untrusted のままです（必須）。

### 4.4 Transcode 規則

- initial v0.2 は transcoding を core behavior として定義しません。
- transcoding / payload transform / codec rewrite / media normalization が必要な場合、§5 が admission boundary を定義し、別個の future な仕様が exact owner、resource bounds、quality relation、privacy boundary、evidence rule を定義しなければなりません（必須）。
- header rewrite intent は core が routing/forwarding intent としてのみ返してよい。actual RTP/RTCP byte rewrite は driver 所有のままです（必須）。

### 4.5 Failure Mapping（media negotiation）

| Failure | Required reason |
|---|---|
| codec/profile 非対応 | `media_codec_not_supported` |
| track publication/subscription 不許可 | `media_track_not_allowed` |
| 要求 media layer 利用不可 | `media_layer_not_available` |
| payload type / SSRC / RID / MID mapping 不正 | `media_payload_mapping_invalid` |
| RTCP feedback / RTX / FEC support 不可 | `media_feedback_not_supported` |
| transcoding/media transform 要求だが非対応 | `media_transcode_not_supported` |
| media-facing contract version 非対応 | `unsupported_media_contract_version` |
| external negotiation material が decode できない | `external_decode_failed` |

### 4.6 Audit / Evidence 規則

- media negotiation decision は SFU publication/subscription/forwarding/protocol violation 用に既定の target event type を使用します。target SFU decision 前の media negotiation mapping に特化した decision の場合、audit event type `media_negotiation_decision` を使用します。
- media negotiation claim の evidence は次を含まなければなりません（必須）: media contract version、negotiation class、accepted codec/profile または rejected capability、関連時の payload type/SSRC/RID/MID mapping result、materialize 時の track/layer reference、SFU target decision event または `media_negotiation_decision`、unsupported codec/disallowed track/unavailable layer/invalid mapping/unsupported feedback/unsupported transcode の closed reason。
- browser/native media readiness evidence は、core media negotiation decision に接続されない限り SFU routing semantics を証明しません。media negotiation evidence は、cross-plane binding evidence なしに Signaling participant、TURN permission、ICE connectivity、secure media binding を証明しません（必須）。

### 4.7 禁止事項（media negotiation）

- payload type 単独が codec authority になる。
- driver parser が publication/subscription authorization を決定する。
- SDK が codec/media readiness を Signaling success として exposure する。
- unsupported simulcast/SVC layer が reason なしに silent downgrade される。
- routing decision によって transcoding が暗黙化される。
- RTP/RTCP parser object が core domain state に入る。
- media negotiation success を cross-plane participant/endpoint/session binding として扱う。

### 4.8 collapse 条件（media negotiation）

- SDP/RTP field value が mapping なしに core identity になる。
- codec support が仕様の更新なしに driver ごとに異なる。
- media layer selection が driver execution 内に隠れる。
- transcode/media transform が owner/resource/evidence rule なしに現れる。
- SFU route decision が concrete codec implementation object に依存する。
- packet rewrite/media transform が §5 なしに実行される。
- cross-plane binding が codec/track/layer negotiation 単独から推定される。

---

## 5. Packet Rewrite / Media Transform Boundary

本節は packet rewrite、header rewrite、SSRC/sequence mapping、payload transform、media transform の境界を固定します。core が返せる rewrite/transform intent と driver が実行する byte operation の境界を定義し、transcoding 実装、media engine runtime、payload transform backend、production media transform readiness は主張しません。

### 5.1 境界（owner）

packet rewrite / media transform は、SFU routing/forwarding intent の後または実行時に、driver が packet bytes を送信可能な形へ変換する処理です。core は rewrite/transform の意味論的 intent を返せますが、packet bytes、payload bytes、codec engine、encryption/framing backend、buffer ownership を持ちません（必須）。

| Concern | Owner | 規則 |
|---|---|---|
| route/target selection | core/sfu | forwarding target と suppression decision |
| rewrite intent | core/sfu（semantic 時） | closed intent class のみ |
| header byte rewrite | driver | concrete RTP/RTCP byte mutation |
| payload transform execution | driver | class が admit されない限り禁止 |
| codec transcode execution | driver/media backend | 仕様が admit しない限り initial v0.2 では out |
| copy/allocation strategy | driver | copied buffer を core へ leak させない |
| evidence/reporting | reports | intent/execution/copy class と failure reason 必須 |

### 5.2 Rewrite / Transform Classes（閉集合）

v0.2 initial architecture の rewrite/transform class は次に限定します。新規 class は仕様の更新が必須です。

| Class | 意味 | 規則 |
|---|---|---|
| `no_rewrite_forward` | 元の packet lease を unchanged で forward 可 | default で copy 禁止 |
| `header_rewrite_only` | driver が payload transform なしに RTP/RTCP header field を rewrite | header-only allocation または scatter/gather を優先 |
| `sequence_number_mapping` | driver が core-owned sequence mapping intent を適用 | mapping intent は closed かつ target-scoped 必須 |
| `ssrc_rewrite` | driver が accepted route/stream reference に基づき SSRC を rewrite | stream identity semantics を変更してはならない |
| `rtcp_feedback_rewrite` | driver が admitted RTCP feedback reference を map | parser detail は driver 所有のまま |
| `framing_security_backend_copy` | encryption/framing backend が driver-local copy を要求 | backend class を evidence 記録しない限り diagnostic |
| `payload_transform_requested` | payload bytes 変更が必要 | future な仕様が exact class を admit しない限り reject |
| `codec_transcode_requested` | codec decode/encode/transcode が必要 | initial v0.2 では reject |

### 5.3 Intent 規則

core rewrite intent は次を記録しなければなりません（必須）。

- route または forwarding decision reference
- materialize 時の target endpoint/route reference
- rewrite/transform class
- source stream/packet semantic reference
- target stream/packet semantic reference
- sequence/SSRC mapping 使用時の mapping table reference
- copy allowance class
- failure reason mapping
- audit event relation

core intent は raw bytes、mutable packet slice、codec engine object、driver buffer lease、OS buffer type、backend-specific frame object を含んではなりません（禁止）。

### 5.4 Driver Execution 規則

driver は現 forwarding decision に紐づく admitted class のみを実行してよい（必須）。driver execution は次を保たなければなりません。

- original buffer lease ownership
- target-specific rewrite isolation
- payload non-copy default
- copy が必要な場合の bounded allocation/copy budget
- release reason mapping
- diagnostics の privacy/redaction rule

driver が admitted class を exactly に実行できない場合、fail-close しなければならず、別の transform path へ silent downgrade してはなりません（禁止）。

### 5.5 Transcode / Payload Transform 規則

initial v0.2 は codec transcoding や payload transform を generic SFU behavior として admit しません。future feature が payload transform、codec rewrite、transcoding、media normalization、insertable-stream style transform、E2EE media transform を要求する場合、future な仕様が次を定義しなければなりません（必須）: transform class、owner と dependency boundary、resource と copy bounds、quality/latency relation、privacy/redaction rule、secure media relation、failure reason mapping、evidence と audit rule。それまで、これらの要求は not admitted として reject されます。

### 5.6 Failure Mapping（packet rewrite / media transform）

| Failure | Required reason |
|---|---|
| rewrite/transform class が not admitted | `packet_rewrite_class_not_admitted` |
| core rewrite intent の required field 欠落 / route と conflict | `packet_rewrite_intent_invalid` |
| driver/backend が rewrite で routing/quality semantics を所有しようとする | `packet_rewrite_owner_violation` |
| admitted 仕様なしに payload transform 要求 | `payload_transform_not_admitted` |
| initial v0.2 で codec transcode 要求 | `media_transcode_not_supported` |
| driver transform execution 失敗 | `payload_transform_failed` |
| rewrite path の copy/allocation bound 超過 | `rewrite_copy_bound_exceeded` |
| core が concrete packet bytes を要求 | `packet_rewrite_owner_violation` |

packet buffer release は §6 の SFU packet buffer lifecycle が governing します。

### 5.7 Audit / Evidence 規則

- packet rewrite / media transform decision は audit event type `packet_rewrite_transform_decision` を使用します。event は `CorrelationId`、materialize 時の `PacketId`、materialize 時の route/target reference、rewrite/transform class、copy allowance class、execution owner、rejected/failed outcome の cataloged reason を carry しなければなりません（必須）。
- evidence は rewrite/transform class、source forwarding decision、target route/endpoint、copy allowance class、driver execution class、resource/copy bound、payload transform admission status、command/procedure、working directory、rerun condition を記録しなければなりません（必須）。packet forwarding evidence は、この class evidence が記録されない限り transform readiness を証明しません。class と closed reason のない backend diagnostic log は adopted evidence になりません。

### 5.8 禁止事項（packet rewrite / media transform）

- core が packet bytes または payload bytes を mutate する。
- core が driver buffer lease、codec engine、OS buffer、backend frame object を所有する。
- driver rewrite path が routing/quality semantics を変更する。
- driver capability によって payload transform が silent に enable される。
- route や media negotiation success によって codec transcode が暗黙化される。
- zero-copy/bounded-copy 依存の claim で packet copy が evidence から隠れる。
- transform failure が generic network send failure のみで記録される。

### 5.9 collapse 条件（packet rewrite / media transform）

- rewrite/transform class が open-ended になる。
- core intent が concrete packet bytes や driver buffer handle を含む。
- payload transform が仕様 admission なしに実行できる。
- copy 可能な rewrite path に copy/allocation bound が absent。
- driver transform backend が SFU routing authority になる。

---

## 6. Packet Semantic View（field boundary、core 読み取り可能範囲）

本節は SFU が core へ渡す packet semantic view の field boundary を固定します。§7（packet buffer lifecycle）は packet bytes ownership / lifetime を定義し、本節は core が読んでよい semantic field と forbidden field を固定します。

### 6.1 境界（owner）

| Surface | Owner | 規則 |
|---|---|---|
| raw RTP / RTCP bytes | driver | core 所有不可 |
| parser concrete object | driver | core exposure 不可 |
| semantic packet view type | core | borrowed、minimal field set |
| routing decision | core | semantic field のみ使用 |
| header rewrite execution | driver | §5 に従う |
| packet release | driver | closed release reason |

### 6.2 Required Semantic Fields（閉集合）

core packet semantic view は次の initial field class のみを含んでよい。新規 field は仕様の更新が必須です。

| Field | Required | 規則 |
|---|---|---|
| `packet_id` | yes | materialize 後の core-owned opaque packet reference |
| `correlation_id` | conditional | correlated flow に attach される packet で必須 |
| `source_endpoint_id` | yes | core-owned endpoint reference |
| `stream_id` | yes | core-owned stream reference |
| `media_kind` | conditional | routing/quality に必要なら closed value |
| `packet_kind` | yes | RTP または RTCP semantic class |
| `sequence_number` | conditional | routing/retransmission semantic のみ、raw parser type 不可 |
| `timestamp` | conditional | RTP timestamp semantic のみ |
| `ssrc_ref` | conditional | opaque/core-owned SSRC reference、raw parser object 不可 |
| `payload_type_ref` | conditional | payload type reference、codec negotiation authority 不可 |
| `marker` | conditional | routing/quality hint のみ |
| `packet_length` | yes | resource/quality decision 用 length |
| `arrival_time` | conditional | core time observation、driver clock object 不可 |

### 6.3 Forbidden Fields

core packet semantic view は次を含んではなりません（禁止）。

- raw packet bytes as owned buffer
- mutable payload slice
- driver `BufferLease`
- driver packet cache reference
- transmit queue reference
- parser crate object
- socket address object
- str0m packet/event object
- browser/native platform buffer type
- codec implementation object
- encryption key または SRTP backend detail
- regulated payload または application user identity

borrowed raw slice は §7 の lifetime rule の下でのみ存在してよく、core が store してはなりません（必須）。

### 6.4 Field Use 規則

semantic field は次にのみ使用してよい（許可）: route selection、target suppression/drop decision、quality/backpressure decision、packet cache intent、header rewrite intent、release/audit reference。

semantic field は次になってはなりません（禁止）: codec negotiation authority、regulated domain identity、SDK public API field、driver 外の driver cache key authority、durable persistence schema。

### 6.5 RTCP 規則

RTCP semantic class は routing/quality/feedback/retransmission intent に必要な field のみを exposure してよい。concrete RTCP packet parser object、compound packet layout、raw feedback payload は、仕様が specific semantic field を admit しない限り driver 所有です（必須）。

### 6.6 Failure Mapping（packet semantic view）

| Failure | Required reason |
|---|---|
| parser が required semantic view を生成できない | `external_decode_failed` |
| frame size bound 超過 | `frame_size_bound_exceeded` |
| media contract version 非対応 | `unsupported_media_contract_version` |
| buffer release 失敗 | `buffer_release_failed` |

packet identity materialize 前の failure は audit の absent reference rule に従います。

### 6.7 禁止事項 / collapse 条件（packet semantic view）

禁止: core が routing call を超えて borrowed packet view を store する。core が payload bytes を mutate する。packet semantic view が raw regulated payload を carry する。payload type field が codec implementation authority になる。negotiation mapping なしに payload field から media layer/codec selection を推定する。仕様の更新なしに packet view field を追加する。driver parser object が core 境界を跨ぐ。

collapse: semantic view が raw packet wrapper になる。field set が implementation convenience で拡大する。core が parser/cache/queue detail を所有する。packet view が SDK/regulated public data model になる。packet field が state persistence policy なしに domain source-of-truth として persist される。packet semantic view が media negotiation source-of-truth になる。packet semantic view が transform backend/rewrite execution detail を carry する。

---

## 7. SFU Packet Buffer Lifecycle（RTP/RTCP bytes 所有権・寿命・copy 許可）

本節は SFU における RTP / RTCP packet bytes の所有権、借用、寿命、copy 許可条件を固定します。本節は実装手順ではなく、特定 crate、pool size、lock-free data structure、SIMD backend を指定しません。

### 7.1 境界 summary

```text
driver owns raw bytes / buffer lease / queue / cache
core owns packet abstract view / routing semantics / forwarding decision
driver executes forwarding and releases buffers
```

### 7.2 Ownership 規則

| 対象 | 所有者 | 理由 |
|---|---|---|
| raw RTP / RTCP bytes | driver | socket / str0m / parser / buffer pool 由来の concrete data |
| receive buffer | driver | memory allocation と reuse strategy は infrastructure detail |
| buffer lease | driver | bytes の寿命、ref-count、loan-count は physical resource management |
| retransmission cache | driver | NACK / retransmission は packet retention strategy |
| transmit queue | driver | async scheduling と backpressure execution detail |
| packet abstract view type | core | routing semantics に必要な core-owned input |
| RTP header semantic view | core | routing / quality / stream identity に必要な抽象表現 |
| routing decision | core | SFU domain semantics |
| forwarding execution | driver | concrete I/O operation |

この表の driver ownership は physical bytes、lease、retention、allocation/reuse strategy の所有を意味します。

### 7.3 Core Packet View

core が受け取る packet は owned bytes ではなく borrowed abstract view です。概念構造は次のとおりです。

```text
SfuPacketView<'packet>
  packet_id
  stream identity
  source endpoint
  RTP / RTCP semantic header view
  borrowed raw packet slice
  borrowed payload slice
```

`SfuPacketView` は core-owned type ですが、その中の raw packet slice と payload slice は driver-owned buffer を借用します（必須）。

### 7.4 Lifetime 規則

core は borrowed packet view を routing call の間だけ読みます。core は packet bytes、payload slice、raw slice、driver buffer handle を保存してはなりません（禁止）。

許可: routing decision のための header/payload metadata 読み取り、quality/backpressure decision のための packet length/timestamp 読み取り、packet identity と stream identity を decision に含めること。

禁止: core が `Vec<u8>` を所有する、core が `Bytes` / `Arc<[u8]>` / driver buffer handle を所有する、core が `&[u8]` を queue/cache/async task に保存する、core が raw bytes を mutate する、core が driver-local reference count を操作する。

### 7.5 Routing Decision 境界

core は bytes を返しません。core は forwarding の意味論的判断だけを返します。Routing decision は次を表現しなければなりません（必須）: packet identity、forwarding targets、suppress/drop decision、rejection/suppression/drop reason、quality/backpressure related action、必要時の header rewrite intent、必要時の payload transform intent。

driver は routing decision と自分が保持する `packet_id -> BufferLease` 対応を使い、元の bytes を送信します。

### 7.6 Async / Queue 規則

async boundary、queue、worker handoff、pacing、retransmission をまたぐ場合、bytes の保持責任は常に driver に残ります（必須）。

許可: driver が `BufferLease` を queue に積む、driver が `PacketCache` に bounded retention として保持する、core が `packet_id` と target set を返す。

禁止: core が borrowed slice を async boundary の先へ持ち越す、core が packet cache を所有する、core が transmit queue を所有する、core が pacing queue を所有する。

### 7.7 Fan-Out 規則

SFU fan-out では target ごとに常時 packet copy してはなりません（禁止）。driver は同一 packet の raw bytes を shared lease として扱い、target ごとの送信完了で loan-count または reference count を減らします。

```text
one received packet
  -> one driver-owned BufferLease
  -> many target send attempts
  -> release after all loans are resolved
```

core は fan-out の target set を決め、driver は fan-out の memory retention と send execution を管理します。

### 7.8 Packet Cache 規則

NACK、retransmission、pacing、short-term loss recovery のための cache は driver 内部に置きます。Packet cache は必ず bounded でなければなりません（必須）。

必須 bound: maximum packets、maximum bytes、maximum retention duration、eviction reason。

禁止: unbounded packet cache、core-owned packet cache、eviction reason が open-ended string のみ、retransmission cache が routing semantics を所有すること。

### 7.9 Header Rewrite / Transform copy 許可（要約）

この節は ownership/copy の要約であり、rewrite/transform class、admission、audit/evidence は §5 に従います。header rewrite、sequence number mapping、SSRC rewrite、payload transform が必要な場合、core は intent を返し、実際の bytes 操作は driver が行います。

| 条件 | copy 方針 |
|---|---|
| rewrite 不要 | copy 禁止。shared lease を使う |
| header small rewrite | header-only allocation または scatter/gather を優先 |
| target-specific header rewrite | target-specific header buffer は許可。payload copy は避ける |
| payload transform 必須 | copy-on-write または new buffer allocation を許可 |
| encryption / framing backend が copy を要求 | driver-local copy を許可。core へ copy を露出しない |

copy が発生した場合も、copy の責任と所有権は driver に留まります（必須）。

### 7.10 Backpressure 規則

backpressure decision は core semantics です。queue length、buffer pressure、cache pressure の raw measurement は driver が収集し、core-owned metric type へ変換して core に渡します。driver は backpressure execution を行いますが、backpressure policy の正を所有しません（禁止）。

### 7.11 Failure / Release 規則（release reason 閉集合）

driver は packet lifecycle の終了理由を閉集合の release reason code で分類しなければなりません（必須）。unknown reason は使用しません。Release reason code は free text であってはなりません。

| Release reason code | 意味 | 非成功の catalog reason |
|---|---|---|
| `forwarded` | packet が selected target へ forward された | 成功には不要 |
| `suppressed_by_routing_decision` | core route decision が forwarding を suppress | `route_conflict` |
| `suppressed_by_quality_decision` | core quality decision が packet forwarding を suppress | `packet_suppressed_by_quality` |
| `suppressed_by_backpressure` | core backpressure decision が packet forwarding を suppress | `packet_suppressed_by_backpressure` |
| `dropped_by_backpressure` | core backpressure decision が packet を drop / retention 停止 | `packet_dropped_by_backpressure` |
| `dropped_by_transmit_queue_bound` | SFU transmit queue bound が enqueue/send scheduling を阻止 | `sfu_transmit_queue_bound_exceeded` |
| `target_unavailable` | target endpoint が packet を receive 不可 | `target_unavailable` |
| `send_failed` | driver send operation 失敗 | `network_send_failed` |
| `expired_from_packet_cache` | packet retention duration 終了 | `retention_duration_exceeded` |
| `evicted_by_cache_bound` | packet cache bound が eviction を強制 | `packet_cache_bound_exceeded` |
| `transform_failed` | driver transform/encode step 失敗 | `payload_transform_failed` |
| `release_failed` | driver が buffer lease release に失敗 | `buffer_release_failed` |
| `driver_shutdown` | driver shutdown が packet lifecycle を終了 | `driver_shutdown` |

Release reason audit mapping:

| Release reason code | Audit event type | Decision outcome | Catalog reason |
|---|---|---|---|
| `forwarded` | `sfu_forwarding_decision` | `forwarded` | 成功には不要 |
| `suppressed_by_routing_decision` | `sfu_forwarding_decision` | `suppressed` | `route_conflict` |
| `suppressed_by_quality_decision` | `quality_violation_decision` | `suppressed` | `packet_suppressed_by_quality` |
| `suppressed_by_backpressure` | `backpressure_decision` | `suppressed` | `packet_suppressed_by_backpressure` |
| `dropped_by_backpressure` | `backpressure_decision` | `dropped` | `packet_dropped_by_backpressure` |
| `dropped_by_transmit_queue_bound` | `resource_bound_decision` | `dropped` | `sfu_transmit_queue_bound_exceeded` |
| `target_unavailable` | `sfu_forwarding_decision` | `failed` | `target_unavailable` |
| `send_failed` | `driver_error_converted` | `converted_failure` | `network_send_failed` |
| `expired_from_packet_cache` | `resource_bound_decision` | `expired` | `retention_duration_exceeded` |
| `evicted_by_cache_bound` | `resource_bound_decision` | `dropped` | `packet_cache_bound_exceeded` |
| `transform_failed` | `packet_rewrite_transform_decision` | `failed` | `payload_transform_failed` |
| `release_failed` | `driver_error_converted` | `converted_failure` | `buffer_release_failed` |
| `driver_shutdown` | `driver_error_converted` | `converted_failure` | `driver_shutdown` |

packet release audit event は、packet identity が materialize されている場合 `CorrelationId` と `PacketId` を carry しなければなりません。materialize されていない場合、`PacketId` は `absent_not_applicable` でなければなりません（必須）。

### 7.12 Prohibited Core Dependencies

core packet view は次に依存してはなりません（禁止）: `tokio::net::*`、`str0m::*`、socket concrete type、OS buffer type、browser/native platform buffer type、concrete RTP parser crate type、driver `BufferLease`、driver `PacketCache`、driver `TxQueue`。

### 7.13 collapse 条件（packet buffer lifecycle）

- core が packet bytes を所有する。
- core が borrowed packet slice を保存する。
- core が packet cache / transmit queue / pacing queue を所有する。
- driver が routing / quality / backpressure semantics を所有する。
- target ごとの常時 copy を標準経路にする。
- unbounded cache / queue / buffer retention を許可する。
- header rewrite の concrete bytes 処理を core responsibility にする。
- rewrite/transform class や copy allowance が §5 を bypass する。
- release reason が open-ended string のみになる。

---

## 8. Secure Media Session Lifecycle（DTLS/SRTP）

本節は DTLS/SRTP secure media session lifecycle の境界を固定します。transport security configuration、DTLS handshake、peer verification、SRTP protection state、media packet forwarding、secret rotation/rekey が混同されないよう、owner、failure mapping、evidence relation を固定します。本節は DTLS/SRTP runtime 実装、cryptographic certification、secure media readiness を主張しません。

### 8.1 境界（owner）

| Concern | Owner | 規則 |
|---|---|---|
| abstract secure media requirement | core/transport policy | required protection state と profile |
| media routing decision | core/sfu | secure state reference は policy input のみ |
| DTLS/SRTP implementation | driver/WebRTC backend | concrete handshake, keying, packet protection |
| certificate/key source | entrypoints/drivers | typed reference と secret handling |
| rekey/rotation state | driver observation + core policy | secret rotation 規則に従う |
| secure media evidence | reports | handshake, peer verification, protection state は別 class |

listener startup、SDP/ICE relay acceptance、DTLS handshake attempt は SRTP-protected forwarding を証明しません（必須）。

### 8.2 Secure Media Session Classes（閉集合）

v0.2 initial architecture の secure media session class は次に限定します。新規 class は仕様の更新が必須です。

| Class | 意味 | 規則 |
|---|---|---|
| `secure_media_required` | policy が protected media path を要求 | absence は fail-close |
| `dtls_handshake_observed` | driver observed handshake attempt/result | SRTP forwarding proof ではない |
| `peer_verification_observed` | driver observed peer verification result | domain authorization ではない |
| `srtp_protection_active` | driver observed packet protection active | evidence class は observed path に限定 |
| `secure_media_rekey_required` | key generation/rotation が rekey 要求 | secret rotation lifecycle に従う |
| `secure_media_session_closed` | secure media session 終了 | graceful domain drain を含意しない |

### 8.3 Lifecycle 規則

secure media session lifecycle は次を宣言しなければなりません（必須）: required media/security contract version、accepted DTLS/SRTP profile または abstract profile class、peer verification requirement、certificate/key source との relation、secret rotation/rekey との relation、packet forwarding claim 前に required な protection state、failure reason mapping、audit event relation。

core は DTLS/SRTP session object、raw keying material、cipher implementation object、protected packet bytes を retain してはなりません（禁止）。

### 8.4 Failure Mapping（secure media）

| Failure | Required reason |
|---|---|
| secure media profile/version 非対応 | `secure_media_profile_not_supported` |
| DTLS handshake 失敗 | `secure_media_handshake_failed` |
| peer verification 失敗 | `secure_media_peer_verification_failed` |
| required な箇所で SRTP protection 非 active | `secure_media_protection_not_active` |
| secure media key state 不正 | `secure_media_key_state_invalid` |
| secure media session 期限切れ | `secure_media_session_expired` |
| 継続前に rekey 必須 | `secure_media_rekey_required` |
| required secret source 利用不可 | `secret_unavailable` |
| rotation state 利用不可 | `secret_rotation_state_unavailable` |
| revoked secret/key generation | `secret_key_revoked` |

### 8.5 Audit / Evidence 規則

- secure media session decision は audit event type `secure_media_session_decision` を使用します。event は `CorrelationId`、secure media session class、media/security contract version、materialize 時の endpoint/session reference、該当時の redacted key/certificate reference を carry しなければなりません（必須）。
- secure media evidence は次を記録しなければなりません（必須）: secure media session class、media/security contract version、peer verification class、protection state class、許可時の redacted certificate/key reference、関連時の rotation/rekey relation、claim 時の packet forwarding claim scope、expected outcome、actual outcome、non-success の cataloged reason、close-not-claimed scope。
- handshake evidence は、同 scope の protection state evidence がない限り SRTP-protected packet forwarding を証明しません。secure media protection evidence は、admitted cross-plane binding と target-plane decision なしに Signaling participant authorization、SFU endpoint admission、TURN permission を証明しません（必須）。

### 8.6 禁止事項（secure media）

- listener startup を secure media readiness として扱う。
- DTLS handshake attempt を peer verification success として扱う。
- peer verification success を communication authorization success として扱う。
- cross-plane binding なしに secure media observation を Signaling/SFU/TURN binding として扱う。
- SRTP backend object が core に入る。
- raw keying material が audit/log/report に現れる。
- driver cache により stale/revoked media key を accept する。
- secure media required 後に unprotected forwarding を許可する。

### 8.7 collapse 条件（secure media）

- secure media requirement が暗黙化される。
- DTLS/SRTP concrete type が core 所有になる。
- handshake / peer verification / SRTP protection evidence が混同される。
- secure media readiness を claim しながら secret rotation state を省略する。
- secure media failure が generic network failure のみで記録される。
- secure media evidence が binding class と target decision なしに cross-plane binding evidence として使われる。
