# 第05章 core-ports

状態: public summary projection
日付: 2026-07-06 JST

## 目的

本章は arcRTC v0.2 Kernel の core が所有する port の規範です。core が所有する port family（clock / random / token verifier / network / WebRTC transport / packet view / persistence / audit sink / metrics sink / runtime）と各責務、port contract の標準 shape（method 形状・input/output 型・error 型・所有規則・call shape）、および port ownership 方針を、再現実装可能な粒度で内在化します。

依存方向の記法は `A <- B` を「B が A に依存」と読みます。層構造は `core <- drivers <- entrypoints` であり、port は core-owned interface であって driver-owned interface ではありません。

## 1. port ownership 方針

arcRTC v0.2 は DDD / ヘキサゴナルアーキテクチャを基礎とします。この構造では、外部技術との接点を driver 側に置くだけでは不十分です。port の所有者を core に固定しなければ、driver が domain rule を所有し、entrypoints が composition root を超えて判断を持つ危険があります。

したがって v0.2 では、すべての primary / secondary port を core が所有します。

```text
core owns port
drivers implement port
entrypoints compose port implementations
```

- core は port の型、input、output、error vocabulary、contract を所有します。
- driver は core-owned port を実装します。
- entrypoints は port implementation を wiring します。

## 2. core が所有する port family

core が所有する port は次に限定します。

| Port | Core-owned contract（責務） | Driver implementation 例 |
|---|---|---|
| ClockPort | now、deadline comparison、monotonic time（時刻取得・期限判定） | system clock、test clock |
| RandomPort | nonce、opaque ID、challenge entropy | OS RNG、deterministic test RNG |
| TokenVerifierPort | external token verification result（外部発行 token 検証） | JWT RS256 verifier |
| NetworkPort | abstract datagram/stream send/receive（UDP/TCP/HTTP/WebSocket 境界） | tokio UDP/TCP、WebSocket、HTTP |
| WebRtcTransportPort | transport event / command exchange（str0m 等 transport 境界） | str0m |
| PacketViewPort | borrowed packet abstract view / routing input（driver-owned RTP/RTCP buffer lease から core-owned borrowed packet view を構成） | driver-owned RTP / RTCP buffer lease |
| PersistencePort | state / audit persistence boundary（room state、audit、checkpoint） | PostgreSQL、Redis、filesystem、memory |
| AuditSinkPort | audit event export（audit event の外部出力） | file、HTTP、syslog、S3 |
| MetricsSinkPort | metrics export（metrics / telemetry 出力） | tracing、Prometheus-style exporter |
| RuntimePort | timer / spawn / cancellation abstraction（runtime 非依存の scheduling 境界） | tokio runtime driver |

これらの port の test double / fake driver 実装は test-double boundary 規範に従います。

## 3. port contract 規則（共通）

すべての core-owned port は次を持たなければなりません。

- core-owned input type
- core-owned output type
- closed error classification
- fail-closed behavior
- correlation ID propagation rule
- signature に external concrete type を含まない
- driver packet bytes の ownership transfer を行わない
- concrete runtime task / join handle の core semantic state への transfer を行わない

## 4. port shape の field class（一般形）

すべての core-owned port は次の field class を明示します。

| Field class | 規則 |
|---|---|
| input type | core-owned type のみ |
| output type | core-owned type のみ |
| error type | closed reason category/code、または catalog 接続を持つ port-specific closed wrapper |
| correlation | command/request が user/protocol correlated なときは必須 |
| ownership | external concrete type なし、driver buffer ownership transfer なし |
| call shape | command、query、sink-submit、stream-observation、scheduler のいずれか |
| backpressure / bound | bounded なときは resource bound 規範に接続 |
| retry / timeout / cancellation | operation が retry/timeout/cancel され得るときは retry/timeout/cancellation 規範に接続 |

port contract は async runtime、socket、DB、cloud SDK、browser/native SDK、str0m concrete type に依存してはなりません。

## 5. 各 port の必須 shape

| Port | Call shape | Input | Output | Error / reason |
|---|---|---|---|---|
| ClockPort | query | clock request / monotonic comparison input | core time observation | driver が初期化できないときのみ `runtime_config_invalid`。domain expiry reason は core-owned のまま |
| RandomPort | query | entropy request purpose | opaque entropy bytes / generated reference material | `runtime_config_invalid`、`driver_shutdown` |
| TokenVerifierPort | command/query | credential/token verification request | verification result | token reason codes または `token_key_unavailable` |
| NetworkPort | command / stream-observation | core-owned outbound envelope または network observation request | delivery observation / converted inbound observation | `network_send_failed`、`network_receive_failed`、`driver_shutdown` |
| WebRtcTransportPort | command / stream-observation | transport command | transport event / command result | transport conversion または driver failure reason |
| PacketViewPort | query / borrowed view | driver 提供の borrowed packet semantic source | packet semantic view | `external_decode_failed`、`frame_size_bound_exceeded`、`buffer_release_failed` |
| PersistencePort | command / query | checkpoint、audit、retry、lookup intent | persistence acknowledgement / loaded core-owned state | `persistence_unavailable`、retry bound reasons |
| AuditSinkPort | sink-submit | audit event / hash-chain record intent | sink acknowledgement | audit backlog / persistence / driver failure reason |
| MetricsSinkPort | sink-submit | metric export intent | export acknowledgement | `metrics_export_failed`、`metrics_backlog_bound_exceeded` |
| RuntimePort | scheduler | timer / spawn / cancellation request | scheduled handle reference / cancellation observation | `runtime_config_invalid`、`driver_shutdown`、該当 resource bound reason |

## 6. port 固有の narrowing 規則

### 6.1 NetworkPort narrowing 規則

NetworkPort は raw external wire object を core に入れる許可ではありません。inbound external bytes は、domain use case entry の前に driver が semantic envelope へ decode しなければなりません（wire envelope の構造は第06章で内在化）。NetworkPort を receiving observation に用いる場合、その output は core-owned observation type でなければなりません。HTTP request、WebSocket frame、UDP socket address object、STUN/TURN parser object を返してはなりません。

### 6.2 PacketViewPort ownership 規則

PacketViewPort は borrowed semantic header view のみを公開できます。driver `BufferLease`、packet cache、transmit queue、parser crate object、raw bytes ownership を core に公開してはなりません。view lifetime は SFU packet buffer lifecycle が定義する driver-owned lease より長くてはなりません。

### 6.3 PersistencePort shape 規則

PersistencePort は次を分離しなければなりません。state checkpoint intent / audit persistence intent / hash-chain record persistence intent / retry store intent。各 intent は、source-of-truth、checkpoint、audit-only、driver retry data のいずれであるかを state persistence policy に従って宣言しなければなりません。

### 6.4 RuntimePort shape 規則

RuntimePort は runtime task handle を domain state として公開してはなりません。core は opaque schedule/cancellation references と execution observations のみを受け取れます。runtime task class、supervision scope、join/cancel bound、panic outcome は runtime task / worker lifecycle 規範に従います。timer execution delay は core expiry semantics を再定義しません。

## 7. error shape 規則

port errors は closed でなければなりません。driver-local error は non-authoritative detail として運べますが、decision、audit、SDK mapping は cataloged category/code を用いなければなりません。port errors の external exposure は external error mapping 規範（第04章で内在化）に従います。

## 8. forbidden signature（禁止 import 型）

core-owned port signature に次を含めてはなりません。

- `tokio::net::*`
- `axum::*`
- `sqlx::*`
- `aws_sdk_s3::*`
- `reqwest::*`
- `str0m::*`
- driver `BufferLease`
- driver `PacketCache`
- driver `TxQueue`
- concrete runtime task handle / join handle
- browser-native Web API 型
- Android / iOS platform SDK 型

外部型は driver 境界で core-owned type に変換します。禁止例として、core が `axum` request/response、`tokio::net::UdpSocket`、`str0m` concrete event、driver `BufferLease`/packet cache/transmit queue、`sqlx` row/pool、browser `WebSocket`/native SDK 型を受け取ることを禁止します。

## 9. port naming 規則

port 名は role を表します。technology 名を port 名に入れてはなりません。

- 許可例: `WebRtcTransportPort`、`AuditSinkPort`、`TokenVerifierPort`
- 禁止例: `Str0mPort`、`PostgresAuditPort`、`S3BackupPort`、`AxumSignalingPort`

## 10. prohibited ownership / 禁止事項

- drivers が port trait を定義する。
- entrypoints が port trait を定義する。
- drivers が Signaling / SFU / TURN の accept / reject rule を所有する。
- entrypoints が domain decision を所有する。
- core が driver concrete type を import する。
- core が framework / runtime / OS / browser / DB / cloud SDK に依存する。
- port input または output が external concrete type を含む。
- port が open-ended string を唯一の error として返す。
- NetworkPort が driver conversion を bypass する。
- PacketViewPort が byte ownership を core に transfer する。
- RuntimePort が concrete task handle を domain state として公開する。
- RuntimePort が、task execution が claim に影響する箇所で task class または supervision scope を省略する。
- PersistencePort が schema/table/key layout を core API として用いる。
- entrypoints が binary ごとに port variants を定義する。
- core-owned port が packet bytes、buffer lease、packet cache、transmit queue の所有権を受け取る。

## 11. 帰結（consequences）

- `core/src/transport/str0m_transport*` 相当は v0.2 core に直接残しません。
- PostgreSQL / S3 / HTTP / file / syslog sink は drivers 側に分離します。
- JWT 検証ライブラリ依存は driver に置き、core は verification result と decision rule を所有します。
- entrypoints は Kernel executable contract / CLI / demo / dependency wiring に限定します。

## 12. fail-closed / 不変条件

- すべての port は fail-closed behavior を持ちます。
- port error は closed（reason catalog 接続）。open-ended string 単独は不可。
- core-owned port は driver packet bytes、buffer lease、packet cache、transmit queue の ownership を受け取りません。
- RuntimePort は concrete task handle を domain state として core へ渡しません。
- port signature に external concrete type を含めません。
- correlation が必要な command/request では correlation ID propagation を必須とします。

## 13. 判断が崩れる条件（collapse conditions）

- driver が port を定義する。
- port signature が external concrete type を含む。
- core-owned port が packet bytes、buffer lease、packet cache、transmit queue の所有権を受け取る。
- RuntimePort が concrete task handle を domain state として core へ渡す。
- entrypoints が port contract を分岐定義する。
- port error が open-ended string だけになる。
- port が closed error mapping を欠く。
- port call shape が unbounded queue または hidden retry を許す。
- port retry/timeout/cancellation behavior が implicit になる。
- port behavior が worker execution に依存する箇所で runtime task lifecycle evidence が implicit になる。
- inbound wire object が NetworkPort を通じて core に入る。
- PersistencePort の state class が宣言されない。
- entrypoints が protocol semantics や accept / reject rule を持つ。
- port を実装都合で driver 側に移す。
