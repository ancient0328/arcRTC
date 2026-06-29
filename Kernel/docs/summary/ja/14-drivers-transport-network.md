# drivers-transport-network

状態: SSOT 統合版
日付: 2026-06-28 JST

## 目的

本章は arcRTC v0.2 Kernel の transport / network driver family（`drivers/network`、`drivers/webrtc-str0m`、`drivers/browser`、`drivers/native`、TURN/STUN wire driver）の現行完全仕様を、本章のみで再現実装可能な粒度で内在化することを目的とします。本章は driver conversion（external 型と core-owned 型の変換規則）、network I/O 境界（tokio UDP/TCP/HTTP/WebSocket/socket）、transport driver（str0m を含む WebRTC transport port implementation）、browser/native driver 境界、TURN wire driver（STUN/TURN decode/encode 境界）を所有者・規則・failure mapping・閉集合語彙・禁止/許可・fail-closed 条件まで落とさずに内在化します。

依存方向の表記: `A <- B` は「B が A に依存」を意味します。driver は core が所有する port を実装し、external 型は driver 境界で core-owned 型へ変換します。`drivers -> entrypoints`、`drivers -> regulated`、`driver -> driver`（直接依存）は禁止です。driver は syntax / transport / framework 変換のみを行い、domain decision を行いません。cross-driver composition は entrypoints が core-owned port を通じて行います。

---

## 1. Driver Conversion（external 型と core-owned 型の変換規則）

### 1.1 変換方向（必須）

driver conversion は syntax / transport / framework 変換であり、domain decision ではありません。変換方向は次に固定します。

```text
external input
  -> driver decode / validate syntax
  -> core-owned command / event / observation / packet view
  -> core decision
  -> driver encode / execute
```

external error projection（変換失敗後の外部投影）は本章第1.5節の failure mapping に従います。

### 1.2 Driver Required Validation（core 呼び出し前の必須検査、閉集合）

driver は core-owned 型を生成する前に次を必ず検査します。検査できない場合、core を呼ばずに closed conversion failure とします（fail-closed）。

- external payload is decodable
- frame size is within driver bound
- required wire field exists before mapping
- byte buffer shape is parseable
- external enum value maps to core enum
- transport connection is readable / writable

### 1.3 Driver Must Not Decide（driver が判断してはならない事項、閉集合・禁止）

driver は次を判断してはなりません。

- Signaling join accepted / rejected
- TURN allocation accepted / rejected
- TURN refresh accepted / rejected / expired
- TURN permission accepted / rejected / revoked
- TURN channel bind accepted / rejected / expired
- SFU route selected / suppressed
- quality violation semantics
- backpressure policy
- protocol / contract version accept-reject semantics
- TURN method support semantics
- regulated domain meaning
- SDK public contract semantics

### 1.4 Semantic Delegation Rule（意味委譲規則）

driver は core entry の前に external encoding と syntactic shape のみを検証してよい（許可）。driver は core が必要とする protocol / contract version フィールドと method identifier を必ず保存します（必須）。unsupported command version、unsupported TURN method、unsupported TURN contract version、unsupported SFU/media contract version は core decision であり、driver conversion failure として扱ってはなりません（禁止）。

### 1.5 Conversion Failure Rule（変換失敗 → closed reason、閉集合）

conversion failure は必ず closed reason へ map します。`Reason` 列は必ず catalog 済み reason code を名指しし、category-only な reason 値は driver/core 境界で禁止です。

| Failure | Reason |
|---|---|
| external payload cannot decode | `external_decode_failed` |
| frame size exceeds driver bound | `frame_size_bound_exceeded` |
| unsupported driver wire version | `unsupported_driver_wire_version` |
| correlation ID field missing | `missing_correlation_id` |
| required wire field missing | `missing_required_wire_field` |
| byte buffer shape is not parseable | `external_decode_failed` |
| external enum has no core mapping | `external_enum_unmapped` |
| transport connection is not readable | `network_receive_failed` |
| transport connection is not writable | `network_send_failed` |
| external type would leak into core | `external_type_leak_blocked` |
| core event cannot encode | `external_encode_failed` |

### 1.6 Conversion Failure Audit Rule（変換失敗の監査規則）

- `frame_size_bound_exceeded` は driver-local resource-bound drop であり、outcome `dropped` を伴う `driver_resource_bound_decision` を emit します。
- 上表のそれ以外の conversion failure は outcome `converted_failure` と当該 catalog reason を伴う `driver_error_converted` を emit します。
- `missing_correlation_id` は outcome `converted_failure` の `driver_error_converted` として監査され、当該 command は core state machine に入りません。
- client 提供の `CorrelationId` が失敗前に decode / validate できない場合、audit reference の有無は audit event の pre-materialization 規則に従います。

### 1.7 External Type Prohibition（core 境界が露出してはならない external 型、閉集合・禁止）

core boundary は次を露出してはなりません。

- framework request / response
- WebSocket frame type
- tokio socket type
- str0m event type
- sqlx row / pool
- AWS SDK type
- browser API type
- Android / iOS platform type
- driver buffer handle

### 1.8 Lossless Mapping Rule（無損失 mapping 規則）

driver conversion は core semantics が必要とする情報を必ず保存します。external execution だけが必要とする情報は driver-local に留めます。external syntax / encoding が無効で core-owned input を構築できない場合、driver は conversion failure を返します。conversion が protocol / contract / policy の問いに到達した場合、driver は未決フィールドを保存し decision を core に委譲します。

### 1.9 Driver Conversion Collapse Conditions（崩壊条件）

次のいずれかが成立すると本規則の判断は崩れます。driver conversion が domain decision を行う。external concrete 型が core を通過する。conversion failure が success として扱われる。driver が core 未定義の semantic field を追加する。driver が core-required 情報を silently に落とす。driver が driver-local semantics で protocol / contract version または TURN method を拒否する。conversion failure が catalog reason traceability を保たずに外部露出される。

---

## 2. Network I/O Boundary（`drivers/network`）

### 2.1 Purpose と所有原則

network driver は external network I/O と wire framing を所有しますが、Signaling / SFU / TURN の domain semantics を所有しません。external wire envelope と semantic envelope の変換境界は wire protocol envelope 規則に、external error mapping は external error mapping 規則に、public endpoint admission と connection lifecycle は public endpoint connection lifecycle 規則に、edge/proxy trusted metadata は edge/proxy trust boundary 規則に、service discovery / endpoint resolution は service discovery endpoint resolution 規則に従います。

### 2.2 Ownership（所有割当、閉集合）

| 責務 | 所有者 | 規則 |
|---|---|---|
| UDP / TCP socket | driver | concrete listener, send, receive |
| HTTP / WebSocket endpoint | driver | external protocol binding and frame boundary |
| frame size / connection concurrency local bound | driver（core policy bound が列挙される場合を除く） | resource bounds / backpressure 規則に接続 |
| external wire decode / encode | driver | core-owned 型へ変換する |
| core command / event semantics | core | driver は変更しない |
| process entrypoint | entrypoints | listener selection and wiring only |
| network target resolution | entrypoints/drivers | concrete endpoint lookup, not semantic authority |

### 2.3 Inbound Flow（順序固定、必須）

network inbound flow は次の順序に限定します。

1. driver receives external bytes / frame / request.
2. driver enforces driver-local shape and frame bounds.
3. driver converts to core-owned command or packet view.
4. core evaluates domain / protocol semantics.
5. driver maps core response / command to external output.

driver は domain aggregate internals を直接呼び出してはなりません。driver は application use case または core-owned port boundary を通じて core に入ります。

### 2.4 Failure Mapping（閉集合）

| Failure | Required reason |
|---|---|
| external payload cannot decode to core type | `external_decode_failed` |
| external wire version unsupported | `unsupported_driver_wire_version` |
| required wire field absent | `missing_required_wire_field` |
| external enum has no mapping | `external_enum_unmapped` |
| inbound frame size bound exceeded | `frame_size_bound_exceeded` |
| connection concurrency bound exceeded | `connection_concurrency_exceeded` |
| concrete receive failed | `network_receive_failed` |
| concrete send failed | `network_send_failed` |
| driver is shutting down | `driver_shutdown` |

pre-core conversion failure は第1節の driver conversion 規則に、resource-bound failure は resource bounds / backpressure 規則に、endpoint-class / lifecycle failure は public endpoint connection lifecycle 規則に、proxy/header/source-address trust failure は edge/proxy trust boundary 規則に、service discovery / endpoint resolution failure は service discovery endpoint resolution 規則に従います。

### 2.5 Encoding Rule（符号化規則）

external JSON、HTTP status、WebSocket close code、binary frame、UDP datagram layout、TCP framing は driver 所有です。これらの mapping は core reason の category/code と、存在する場合の correlation reference を必ず保存します。driver は external representation を選んでよいが、新しい semantic reason を発明してはならず、catalog 済み rejection を generic success の背後に隠してはなりません（禁止）。

### 2.6 Direct Dependency Rule（直接依存規則）

`drivers/network` は `drivers/persistence`、`drivers/observability`、`drivers/webrtc-str0m`、`entrypoints/*`、`regulated` に直接依存してはなりません（禁止）。cross-driver composition は entrypoints が core-owned port を通じて行います。

### 2.7 Prohibitions（禁止、閉集合）

- network driver が自前 policy で join、route、allocation、permission、relay を accept / reject する。
- network driver が core entry 後に decode failure を domain rejection に変える。
- WebSocket close code が authoritative core reason になる。
- HTTP status が authoritative core reason になる。
- core が rejected / denied / failed decision を返したのに driver が success response を送る。
- driver 同士が直接依存し entrypoints composition root を迂回する。
- external response が catalog reason traceability を失う。
- network listener bind または WebSocket upgrade が public endpoint admission として扱われる。
- network driver が edge trust policy なしに forwarded header / proxy source metadata を信頼する。
- network driver が endpoint resolution を domain readiness / internal control success として扱う。

### 2.8 Network I/O Collapse Conditions（崩壊条件）

network I/O layer が protocol semantics を所有する。external wire format が core に漏れる。core reason が driver-local status text に置換される。resource bound event が required canonical mapping を通じて監査されない。network driver が persistence / observability implementation を直接 compose する。public/internal endpoint class が route naming だけから推測される。source address / host / origin / forwarded header が core identity になる。resolved network endpoint が semantic owner / authorization proof / readiness proof になる。

---

## 3. Transport Driver（str0m を含む WebRTC transport port implementation）

### 3.1 Purpose と原則

transport driver は core-owned port を実装するだけで protocol semantics を所有しません。browser / native driver の platform-specific boundary は第5節に従います。

### 3.2 Driver Set（閉集合）

transport driver は次に限定します。

| Driver | 対象 |
|---|---|
| `drivers/webrtc-str0m` | str0m concrete implementation |
| `drivers/network` | UDP / TCP / HTTP / WebSocket |
| `drivers/browser` | browser-facing boundary |
| `drivers/native` | native platform boundary |

`drivers/observability`、`drivers/persistence`、audit sink、metrics sink は transport driver ではありません。それらは core-owned port を実装する driver family ですが本節の対象外です。

### 3.3 Conversion Rule（変換規則）

driver は external 型を core-owned 型に変換してから core を呼び出します。core から返った command / decision / event を external 型に戻します。RTP / RTCP packet bytes を扱う場合、driver は raw bytes と buffer lifecycle を所有し、core には borrowed abstract view だけを渡します。

### 3.4 Driver May Own（driver が所有してよい、閉集合・許可）

driver は次を所有してよい。

- external library initialization
- network listener
- socket read/write
- byte buffer codec
- buffer pool
- buffer lease
- bounded packet cache
- transmit queue
- external error mapping
- retry transport detail
- serialization format
- TLS / platform-specific transport setting

本節の "may own" は physical implementation / execution detail の所有を意味します。buffer pool、packet cache、transmit queue、retry transport は unbounded resource や driver-owned policy を許可しません。bounded resource policy、closed action、audit owner tuple は resource bounds / backpressure 規則に従います。

### 3.5 Driver Must Not Own（driver が所有してはならない、閉集合・禁止）

driver は次を所有してはなりません。

- Signaling accept / reject rule
- TURN allocation / refresh / permission / channel bind / relay rule
- SFU routing / quality / backpressure rule
- SDK public contract semantics
- regulated domain semantics
- audit event meaning

### 3.6 str0m Rule

str0m は driver implementation です。str0m event、state、error、SDP/ICE concrete 型は core API として公開しません。core には WebRTC transport port と core-owned event / command 型だけを置きます。

### 3.7 Transport Driver Collapse Conditions（崩壊条件）

driver が core-owned rule を再実装する。external concrete 型が core boundary を通過する。driver-owned packet bytes、buffer lease、packet cache、transmit queue が core ownership へ移る。drivers 間で直接依存し entrypoints composition root を迂回する。driver-local error が core error classification へ変換されない。observability / persistence driver を transport driver として扱う。

---

## 4. TURN Wire Driver（STUN/TURN decode/encode 境界）

### 4.1 Purpose と原則

TURN / STUN wire decode/encode driver は core TURN semantics を所有しません。TURN contract 規則と TURN lifecycle 規則が core TURN semantics を所有し、本節は external bytes、method/code mapping、attribute mapping、socket failure mapping を driver-owned detail として固定します。

### 4.2 Boundary（所有割当、閉集合）

| Surface | Owner | Rule |
|---|---|---|
| raw STUN/TURN bytes | driver | socket buffer and parser input |
| STUN/TURN parser / encoder | driver | concrete library/detail |
| method / attribute wire code | driver maps | core sees semantic command/result |
| TURN allocation / permission / relay semantics | core | lifecycle and decision owner |
| credential verification outcome | core semantics, driver crypto/key implementation | no credential issuance |
| UDP/TCP socket | driver | I/O execution only |

### 4.3 Inbound Mapping（wire class → core semantic target、閉集合・必須）

driver は inbound wire message を core entry の前に core-owned TURN command へ map します。raw attribute object、parser error、socket address、byte buffer は core state になってはなりません（禁止）。

| Wire class | Core semantic target |
|---|---|
| Allocate request | `Allocate` command |
| Refresh request | `Refresh` command |
| CreatePermission request | `CreatePermission` command |
| ChannelBind request | `ChannelBind` command |
| Send/Data indication | `RelayData` intent |
| unsupported method | rejection before or at core boundary with `unsupported_turn_method` |
| malformed message | pre-core rejection with `malformed_turn_message` |

### 4.4 Outbound Mapping（core outcome → driver mapping、閉集合）

core decision は semantic outcome を変えずに external TURN response / indication へ map します。wire error code は authoritative reason ではありません。authoritative reason は catalog 済み core reason と audit event に残ります。

| Core outcome | Driver mapping rule |
|---|---|
| allocation accepted | success response with mapped allocation fields |
| refresh accepted | success response with mapped lifetime fields |
| permission/channel bind accepted | success response |
| relay allowed | outbound data forwarding |
| rejected / denied / expired / revoked | error response or drop/deny behavior preserving cataloged reason in audit |

### 4.5 Attribute Rule（属性 decode/encode 規則）

driver は binary attribute decode/encode を所有します。core は変換後の semantic validation を所有します。例: transaction ID は core-owned transaction reference へ、peer address は core-owned address 型へ、credential proof は verification request/result へ、lifetime field は core-owned requested lifetime へ、channel number は core-owned channel bind reference へ map します。必須 attribute の欠落は、より具体的な credential reason が適用される場合を除き `malformed_turn_message` へ map します。

### 4.6 Socket Failure Rule（閉集合）

socket failure は、core が実際に relay semantics を deny した場合を除き、TURN authorization denial として報告してはなりません（禁止）。

| Failure | Required reason |
|---|---|
| receive failure | `network_receive_failed` |
| send failure | `network_send_failed` |
| frame size bound exceeded | `frame_size_bound_exceeded` |
| relay queue bound exceeded | `turn_relay_queue_bound_exceeded` |
| driver shutdown | `driver_shutdown` |

### 4.7 Prohibitions（禁止、閉集合）

- driver parser が allocation/permission/channel state を所有する。
- socket loop が relay authorization を決める。
- wire error code が core catalog reason を置換する。
- raw STUN/TURN attribute object が core に渡る。
- malformed message が TURN lifecycle state machine に入る。
- relay queue bound が `turn_relay_decision` denial として emit される。
- driver が TURN credential を発行する。

### 4.8 TURN Wire Driver Collapse Conditions（崩壊条件）

wire parser state が core TURN state になる。socket failure が core relay denial と混同される。unsupported method または malformed message が catalog reason を欠く。TURN wire attribute shape が core API として露出する。driver credential issuance が導入される。

---

## 5. Browser / Native Driver 境界（`drivers/browser`、`drivers/native`）

### 5.1 Purpose と原則

browser / native driver は platform API と core-owned port の接続を担いますが、SDK public contract、domain rule、regulated workflow、media UI を所有しません。out-of-scope feature の admission / exclusion は out-of-scope feature admission 規則に従います。browser / native driver は core-owned port を実装してよいが、port を定義してはならず、platform concrete 型を core 経由で露出してはならず、Signaling / SFU / TURN semantics を再解釈してはなりません（禁止）。

### 5.2 Boundary（所有割当、閉集合）

| Surface | Owner | Rule |
|---|---|---|
| browser Web API / native platform API | driver | concrete platform interaction only |
| core port contract | core | type, input, output, error vocabulary, ownership rule |
| SDK public API | sdk | Signaling-only public client contract |
| entrypoint lifecycle wiring | entrypoints | selected driver wiring and process/entrypoint startup |
| regulated workflow | regulated | optional support only, not driver-owned |

### 5.3 Initial Scope（v0.2 初期 scope）

v0.2 initial browser / native driver scope は approved port が必要とする platform binding に限定します。

許可される初期責務（許可）:

- platform WebSocket / HTTP client binding when used as network implementation
- platform timer / clock / cancellation execution when used as RuntimePort / ClockPort implementation
- platform randomness source when used as RandomPort implementation
- platform logging / metrics sink when used as observability implementation
- platform storage only when selected as PersistencePort implementation
- platform-specific conversion between external API errors and cataloged core reasons

scope 外（禁止、閉集合）:

- camera / microphone capture
- media track rendering
- PeerConnection public abstraction in SDK
- screen sharing
- recording
- chat
- DataChannel application semantics
- UI / end-user workflow
- push notification workflow
- regulated workflow ownership
- user account or auth issuance

### 5.4 Type Conversion Rule（型変換規則）

browser / native concrete 型は driver boundary で止めます。core signature で禁止される型（閉集合・禁止）:

- browser `WebSocket`, `MessageEvent`, `Blob`, `ArrayBuffer`, `ReadableStream`
- DOM event types
- Android SDK / Kotlin platform types
- iOS Foundation / AVFoundation / Network framework concrete types
- platform permission result objects
- platform lifecycle callback objects

driver は core 呼び出し前に platform input を core-owned command / event / reference / port result へ変換します。driver は core decision を semantic category/code を変えずに platform output へ戻します。

### 5.5 Lifecycle Rule（lifecycle 観測規則、閉集合）

platform lifecycle event は、core contract が core-owned event へ map しない限り driver observation です。driver lifecycle observation は platform-local policy で room を close、route を drop、permission を revoke、command を reject してはなりません（禁止）。core state transition は core 所有のままです。

| Platform observation | Default owner | Core entry condition |
|---|---|---|
| browser tab/page visibility | driver | only if mapped to approved command/event |
| mobile entrypoint foreground/background | driver | only if mapped to approved command/event |
| network availability change | driver | only if mapped to network/transport port event |
| platform cancellation | driver/runtime | only through RuntimePort cancellation result |
| platform shutdown | driver/entrypoints | cataloged driver shutdown or startup failure |

### 5.6 Failure Mapping（閉集合）

新規の platform-specific failure reason は使用前に core reason catalog 更新を要します。

| Failure | Required reason |
|---|---|
| platform input cannot map to core type | `external_decode_failed` |
| platform output cannot be encoded | `external_encode_failed` |
| concrete network receive failed | `network_receive_failed` |
| concrete network send failed | `network_send_failed` |
| required runtime/platform configuration missing | `runtime_config_missing` |
| platform/runtime cannot initialize selected driver | `runtime_config_invalid` |
| driver shutdown ended operation | `driver_shutdown` |

### 5.7 SDK Relation（SDK 関係）

SDK は自身の platform implementation layer を通じてのみ browser / native driver を呼び出してよい。SDK は Signaling-only public client contract を露出し続けなければなりません。browser / native driver は SDK-only semantics を core に追加してはならず、SDK event shape を core event shape として露出してはなりません（禁止）。

### 5.8 Prohibitions（禁止、閉集合）

- browser / native driver が core port trait を定義する。
- platform API 型が core に渡る。
- platform lifecycle callback が domain transition を所有する。
- browser / native driver が SDK public contract semantics を所有する。
- browser / native driver が regulated workflow を所有する。
- platform permission または media device state が v0.2 初期 scope で generic communication core state として扱われる。
- platform-specific error text が authoritative reason になる。

### 5.9 Browser / Native Driver Collapse Conditions（崩壊条件）

core が browser/native concrete 型を import する。SDK public API と driver implementation boundary が崩れる。platform lifecycle event が Signaling / SFU / TURN state を直接 mutate する。browser/native driver が v0.2 初期 scope に media または regulated workflow を持ち込む。browser/native driver が v0.2 初期 scope 外で out-of-scope feature を admit する。platform-local free-text error が reason catalog を迂回する。

---

## 6. 章全体の fail-closed 不変条件

本章の全 driver に共通する fail-closed 不変条件は次のとおりです（必須）。conversion / validation が成立しない場合、driver は core を呼ばず closed reason を返す。すべての failure は catalog 済み reason code を名指しし、category-only 値を境界で許さない。external concrete 型は driver boundary で停止し core を通過しない。driver は domain decision（accept/reject/route/relay/permission/quality/backpressure/version 受理）を所有しない。resource-bound drop は resource bounds / backpressure 規則の audit mapping を通る。drivers は相互に直接依存せず entrypoints composition root を経由する。catalog reason traceability を失った external response、または required rejection を generic success の背後に隠した path は close / complete / ready evidence として採用してはなりません。
