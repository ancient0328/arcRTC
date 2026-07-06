# 第06章 core-protocol-and-serialization

状態: public summary projection
日付: 2026-07-06 JST

## 目的

本章は arcRTC v0.2 Kernel の deterministic / canonical serialization 規則、protocol / contract versioning 体系、compatibility / deprecation 規則（v0.1 を自動互換にしない点を含む）、external wire envelope の構造とフィールド、および feature flag / capability / experimental surface lifecycle を、再現実装可能な粒度で内在化します。本章は具体的 serializer 実装、wire API 実装、release plan、production enablement を主張しません。

依存方向の記法は `A <- B` を「B が A に依存」と読みます。層構造は `core <- drivers <- entrypoints` です。canonical serialization（evidence/hash 入力）と external wire encoding は別物として分離します。

## 1. canonical serialization / deterministic encoding

audit hash-chain、evidence digest、compatibility test、golden fixture が driver/platform/JSON formatting 差によって別意味にならないよう、canonical input と external encoding を分離します。external wire format は自動的に canonical encoding になりません。

### 1.1 所有境界

| Concern | Owner | 規則 |
|---|---|---|
| canonical semantic field set | core | hash/evidence/compatibility 対象 |
| canonical encoding rule | core | deterministic ordering と normalization |
| external wire encoding | driver/sdk | HTTP/JSON/binary/STUN/TURN 表現 |
| storage row/object format | driver | persistence detail。canonical semantic input ではない |
| golden fixture serialization | testing docs/support | canonical/external class を宣言 |
| hash-chain input | core/audit | canonical encoding のみを使用 |

### 1.2 canonical field 規則

canonical encoding は次を定義しなければなりません。

- field set
- field order
- absent/null handling
- enum representation
- integer representation
- decimal/ratio precision
- string normalization
- binary digest representation
- timestamp representation
- unknown field handling
- version field inclusion
- redaction before digest

いずれかが未定義なら、その encoding を hash-chain または golden compatibility evidence に用いることはできません。

### 1.3 初期 encoding policy

v0.2 initial architecture は policy のみを固定し、concrete library は固定しません。concrete encoding format は実装前に仕様で選択しなければなりません。

| Data class | canonical 規則 |
|---|---|
| audit event hash input | deterministic field order と normalized primitive values が必須 |
| core reason | category/code の textual value。localized text は不可 |
| core reference | opaque reference の canonical string または bytes（representation を宣言） |
| timestamp for evidence | 含める場合は UTC epoch milliseconds |
| duration | unit/time normalization 規範による normalized milliseconds |
| binary payload | raw sensitive payload を除外。admitted な場合のみ digest/reference |
| unknown field | compatibility 規則が許す場合のみ ignore。さもなくば reject |

### 1.4 canonical serialization failure

| Failure | 必須 reason |
|---|---|
| canonical encoding を生成できない | `canonical_serialization_failed` |
| canonical verification mismatch | `canonical_serialization_mismatch` |
| external payload を core type へ decode できない | `external_decode_failed` |
| external response encoding 失敗 | `external_encode_failed` |
| required canonical field 欠落 | `missing_required_wire_field` |
| unsupported canonical version | 該当 unsupported version reason |

### 1.5 canonical evidence 規則

digest、hash-chain、golden fixture、compatibility snapshot を用いる evidence は次を記録しなければなりません。canonical format/version / field set / redaction statement / 該当時の raw external source class / digest/hash value / verification command/procedure / close-not-claimed scope。external JSON pretty-print、DB row order、log text は canonical evidence ではありません。

## 2. protocol / contract versioning

### 2.1 version を持つ surface

| Surface | Owner | 例 |
|---|---|---|
| Signaling contract | core | command / event schema |
| TURN contract | core | request / response semantic model |
| SFU contract | core | endpoint / stream / route / decision model |
| SDK public contract | sdk | Signaling contract に map した platform client API |
| Audit schema | core | audit event schema |
| Driver wire encoding | drivers | JSON、binary frame、WebSocket、HTTP |

### 2.2 version ownership

core は contract version semantics を所有します。driver は external encoding version を所有します。sdk は public API version を所有しますが、Signaling semantics を変更しません。SDK public API projection と contract generation は SDK contract generation 規範に従います。

### 2.3 compatibility 規則

| Change | Compatibility | 規則 |
|---|---|---|
| default behavior を持つ optional field の追加 | compatible | default が closed かつ explicit なら受理 |
| required field の追加 | breaking | 新 major contract version |
| field の削除 | breaking | 新 major contract version |
| reason code semantics の変更 | breaking | 新 version なしでは禁止 |
| reason code の追加 | conditional | catalog update を通じてのみ許可 |
| state transition の変更 | breaking | 新 contract version |
| driver encoding の追加 | compatible | core contract が不変なら |

### 2.4 negotiation 規則

version negotiation は protocol contract については core semantics、wire format については driver semantics です。negotiation result は次のいずれかでなければなりません。

- accepted exact version
- accepted compatible version
- rejected unsupported version

古い version への implicit fallback は禁止します。

### 2.5 capability 規則

capability は version の代替ではありません。capability は accepted version 内の optional behavior を記述します。capability は required state transition semantics を変更してはなりません。capability lifecycle と disabled-capability failure handling は本章 §4 に従います。

### 2.6 v0.1 関係

v0.1 protocol shape は v0.2 version 0 ではありません。v0.1 shape は v0.2 仕様が明示的に採用するときのみ参照できます。

## 3. protocol compatibility / deprecation

compatibility window、deprecation notice、removal、SDK parity、driver wire compatibility の fail-open を防ぎます。compatibility は implicit ではありません。受理される任意の古い/compatible version は surface と version range で列挙しなければなりません。

### 3.1 compatibility 境界

| Concern | Owner | 規則 |
|---|---|---|
| core contract compatibility | core | Signaling / SFU / TURN / audit semantic version |
| driver wire compatibility | driver | external encoding version と parser mapping |
| SDK public compatibility | sdk | Signaling semantics を保持した platform public API |
| deprecation decision | 仕様 | removal window と affected surface を記録 |
| release communication | entrypoints/project docs | domain semantics を変更しない |
| evidence report | reports | compatibility test result と unsupported version rejection |

### 3.2 compatibility surface matrix

| Surface | Compatibility owner | required evidence class |
|---|---|---|
| Signaling command/event contract | core | command/event compatibility test |
| SFU media-facing contract | core | 実装時の route/packet decision compatibility test |
| TURN contract | core | 実装時の allocation/permission/relay compatibility test |
| Audit event schema | core | event encode/decode と hash-chain compatibility test |
| Driver wire encoding | driver | external decode/encode compatibility test |
| SDK public API | sdk | platform parity と server reason preservation test |

この表に evidence class が存在することは、evidence が実行済みであることを意味しません。

### 3.3 deprecation lifecycle

deprecation lifecycle は次の順序に限定します。いずれかの step を飛ばすと deprecation/removal readiness を主張できません。

1. affected surface と version を特定する。
2. reason、owner、compatibility window を持つ仕様の更新を行う。
3. unsupported-version behavior と cataloged reason を定義する。
4. 影響箇所の SDK parity と driver mapping rules を更新する。
5. compatibility/negative test plan を追加する。
6. test 実行時に reports へ execution evidence を記録する。
7. documented removal condition 充足後にのみ support を削除する。

### 3.4 fail-closed compatibility 規則

unsupported version は cataloged reason で reject しなければなりません。古い version への silent fallback は禁止します。removed fields の best-effort decode は、compatibility rule が removed/optional field を明示的に map しない限り禁止します。

| Failure | 必須 reason |
|---|---|
| Signaling command/event version unsupported | `unsupported_command_version` |
| media-facing SFU/transport contract unsupported | `unsupported_media_contract_version` |
| TURN contract version unsupported | `unsupported_turn_contract_version` |
| driver wire encoding unsupported | `unsupported_driver_wire_version` |
| compatibility mapping 後に required wire field 欠落 | `missing_required_wire_field` |
| compatibility mapping 後に external enum を map できない | `external_enum_unmapped` |

### 3.5 SDK compatibility 規則

SDK platform compatibility は wrapper で API shape を保持できますが、次はできません。Signaling command/event semantics の変更 / server reason category/code の隠蔽 / platform-only server state の作成 / SFU/TURN/media semantics を SDK Signaling contract として exposure / 全 supported platform の contract test なしの compatibility 主張。

### 3.6 v0.1 関係

v0.1 protocol shape は historical input のみです。v0.1 compatibility は surface と version による明示的な v0.2 採用を要します。明示的採用がなければ、v0.1 shape は accepted v0.2 compatible version ではありません。

## 4. feature flag / capability / experimental lifecycle

configuration flag、protocol capability、driver selection、experimental feature を混同し、core semantics が暗黙に分岐しないよう owner と lifecycle を固定します。feature flag は protocol version の代替ではありません。capability は required state transition semantics を変更する許可ではありません。

### 4.1 所有境界

| Surface | Owner | 規則 |
|---|---|---|
| core protocol capability | core | accepted version 内の optional behavior のみ |
| feature flag value | entrypoints/config | selected driver/exporter/profile gating |
| driver implementation selection | entrypoints | core semantics を変えない |
| SDK capability exposure | sdk | server capability semantics を保持 |
| experimental lifecycle decision | 仕様 | scope、evidence、removal/adoption condition |
| out-of-scope feature admission | 仕様 | excluded feature は flag 単独で enable 不可 |
| runtime enablement evidence | reports | profile と correlation が必須 |
| runtime flag/profile change | runtime reconfiguration 規範 | generation と apply scope が必須 |

### 4.2 flag class（閉集合）

| Flag class | 許可される用途 | 禁止される用途 |
|---|---|---|
| `driver_selection` | concrete implementation を選択 | core decision を変更 |
| `exporter_selection` | metrics/log/audit sink implementation を enable | audit meaning を変更 |
| `profile_selection` | config profile/bundle を選択 | required validation を bypass |
| `experimental_surface_gate` | 明示的に documented な experimental path を gate | stable contract を silent に変更 |
| `test_only_gate` | fake/deterministic support を enable | runtime/prod evidence |

新規 flag class は仕様の更新を要します。

### 4.3 capability 規則

capability は次を宣言しなければなりません。surface / owning layer / accepted contract version / optional behavior / capability 不在時の required fallback / capability が必須だが不在のときの reason / client-visible 時の SDK parity requirement / adoption 前の required evidence class。required capability が不在のとき、より具体的な unsupported-version reason が適用されない限り `capability_not_enabled` を用います。

### 4.4 experimental lifecycle stage

stage 名は documentation classification であり runtime success 主張ではありません。

| Stage | 意味 | promotion condition |
|---|---|---|
| `draft_documented` | draft 仕様が存在 | scope と owner が確定 |
| `gated_scaffold` | 明示的 gate の背後に scaffold が存在 | dependency direction evidence |
| `gated_implemented` | 明示的 gate の背後に実装が存在 | unit/contract evidence |
| `controlled_integration` | selected integration evidence が存在 | integration report |
| `adopted_contract` | 通常 contract に promote | 仕様の更新と compatibility rule |
| `removed` | support 削除 | compatibility/deprecation lifecycle 充足 |

### 4.5 feature / capability failure mapping

| Failure | 必須 reason |
|---|---|
| required capability 不在 | `capability_not_enabled` |
| protocol version unsupported | surface 固有 unsupported version reason |
| feature/profile configuration invalid | `runtime_config_invalid` または `core_policy_config_invalid` |
| gate なしの experimental surface 使用 | `capability_not_enabled` |
| admission 仕様なしの out-of-scope feature 使用 | `feature_admission_not_documented` |
| test evidence 外での test-only gate 使用 | evidence rejection（runtime success ではない） |
| runtime flag change が admitted でない | `runtime_reconfiguration_not_allowed` |

## 5. external wire protocol envelope

wire encoding は driver-owned ですが、core に渡る semantic envelope は core-owned command / event / reason / reference に変換されていなければなりません。canonical serialization（§1）は external wire encoding とは別であり、仕様が exact field set/order/normalization/version を明示的に admit しない限り、external wire 表現は canonical serialization になりません。

### 5.1 wire envelope 境界

| Surface | Owner | 規則 |
|---|---|---|
| JSON / binary / HTTP / WebSocket / UDP / TCP frame | driver | external wire encoding |
| STUN / TURN wire message bytes | driver | decode/encode implementation |
| SDK platform message codec | sdk | Signaling public contract wrapper |
| semantic command/event | core | core-owned type のみ |
| reason category/code | core | 本章 §4 と第04章の reason catalog |
| protocol version と capability semantics | core | 本章 §2・§4 |

external envelope shape は core API になりません。core semantic envelope は driver concrete frame/request/response object を含みません。

### 5.2 semantic envelope field

driver/core 境界を越える command/event は、次の field class を持つ core-owned semantic envelope で表現しなければなりません。

| Field class | Required | Owner | 規則 |
|---|---|---|---|
| `surface` | yes | core | closed set: signaling, sfu, turn, transport, driver |
| `contract_version` | yes | core | version semantics は §2 に従う |
| `correlation_id` | 境界後 yes | core | pre-core の missing/invalid は driver conversion rule に従う |
| `command_or_event_type` | yes | core | contract 仕様ごとの closed set |
| `subject_reference` | conditional | core | 自然に materialize したときの RoomId、ParticipantId、AllocationId 等 |
| `payload_model` | conditional | core | core-owned payload type。wire object ではない |
| `reason` | conditional | core | 非成功 outcome では必須 |

driver はこの envelope の外に external metadata を運べますが、仕様が admit しない限り non-authoritative です。

### 5.3 external envelope 規則

external wire envelope は transport 固有 field（HTTP method/path/status、WebSocket message type/close code、JSON object field names、binary frame tag、UDP peer address、STUN/TURN method・attribute encoding）を含み得ます。これらは driver-owned であり、core entry 前に semantic envelope へ map するか reject しなければなりません。

### 5.4 error response 規則

core rejection / failure は semantic reason を変えずに external へ exposure しなければなりません。

| Core outcome | external mapping 規則 |
|---|---|
| accepted / allowed / forwarded / selected | driver は success response に map し得る |
| rejected / denied / suppressed / dropped / expired / failed | response を emit するとき driver は traceable external response に cataloged reason category/code を保持しなければならない |
| protocol violation before core entry | driver は conversion/resource failure audit を emit し external error response に map し得る |

HTTP status、WebSocket close code、SDK error wrapper、STUN/TURN error code は authoritative reason ではありません。authoritative reason は core catalog category/code、または core entry 前に記録した driver conversion reason です。詳細な external error projection は第04章 §11 に従います。

### 5.5 correlation 規則

driver は許可された correlation boundary のみを validate または create します。

- external command が valid client correlation ID を運ぶ場合、driver は core `CorrelationId` へ map する。
- external command が required correlation ID を欠く場合、driver は core entry 前に `missing_correlation_id` で fail する。
- driver は失敗した client command に対し fake client correlation ID を合成してはならない。
- startup / configuration path では `StartupRunId` と `ConfigurationScopeRef` を用いる（第03章の identity 規則）。

command が driver/core 境界を越えた後、`CorrelationId` は必須です。

### 5.6 version 規則

external wire version と core contract version は明示的に map しなければなりません。version field の decode に成功している場合、version mismatch を generic decode failure として扱ってはなりません。

| Failure | 必須 reason |
|---|---|
| unsupported driver wire version | `unsupported_driver_wire_version` |
| unsupported Signaling command version | `unsupported_command_version` |
| unsupported TURN contract version | `unsupported_turn_contract_version` |
| unsupported media-facing contract version | `unsupported_media_contract_version` |

### 5.7 wire envelope failure mapping

| Failure | 必須 reason |
|---|---|
| external payload を core type へ decode できない | `external_decode_failed` |
| required wire field 欠落 | `missing_required_wire_field` |
| external enum に core mapping なし | `external_enum_unmapped` |
| external type leakage 検出 | `external_type_leak_blocked` |
| external response encoding 失敗 | `external_encode_failed` |
| frame size bound 超過 | `frame_size_bound_exceeded` |

pre-core failure は domain state machine に入りません。

## 6. 禁止事項

- driver wire encoding を既定で canonical encoding として扱う。
- free-text/log formatting が hash input に影響する。
- compatibility rule なしに unknown field を受理する。
- raw sensitive payload を canonical digest に含める。
- precision rule なしに floating point 値を encode する。
- hash-chain record が non-deterministic field order を用いる。
- driver wire version を core protocol version として扱う。
- SDK platform version が Signaling semantics を変更する。
- unsupported version が silent に fallback する。
- 新 contract version なしに breaking change を導入する。
- v0.1 message shape を仕様採用なしに v0.2 contract として受理する。
- deprecation notice に affected surface/version がない。
- 仕様の更新なしに removal する。
- SDK が server semantics を変えて old behavior を保持する。
- driver が compatibility rule なしに removed fields を authoritative core semantics として受理する。
- compatibility 主張が test/evidence なしに release notes のみに依拠する。
- feature flag が仕様の更新なしに core state machine を変更する。
- capability が accepted version 内の required behavior を変更する。
- experimental surface が default で enable される。
- out-of-scope feature が仕様 admission なしに flag で enable される。
- SDK が server contract 未定義の capability を exposure する。
- test-only gate が runtime/prod evidence として使われる。
- removal が protocol compatibility/deprecation lifecycle を bypass する。
- JSON / HTTP / WebSocket / STUN / TURN wire object を core command type として用いる。
- external status code が core reason を置き換える。
- driver が rejected core decision を successful external semantic response に map する。
- version mismatch を generic driver failure として隠す。
- missing correlation ID を fake client ID で修復する。
- SDK platform envelope が server-side semantics を定義する。
- external error mapping が unsafe reason detail を露出する、または cataloged reason traceability を失う。
- external wire encoding を deterministic serialization rule なしに canonical evidence encoding として扱う。

## 7. fail-closed / 不変条件

- unsupported version は cataloged reason で reject（silent fallback 不可）。
- canonical field set のいずれか未定義なら hash-chain/golden evidence に使えない。
- canonical encoding を生成できない場合 `canonical_serialization_failed` で fail-closed。
- core 境界後の `CorrelationId` は必須。pre-core 欠落は `missing_correlation_id` で fail。
- pre-core conversion failure は domain state machine に入らない。
- required capability 不在は silent fallback せず `capability_not_enabled`。
- experimental surface は default で enable しない。

## 8. 判断が崩れる条件（collapse conditions）

- 同一 semantic event が driver ごとに異なる canonical hash を生成し得る。
- canonical field set が implicit になる。
- external storage/wire shape が仕様の更新なしに canonical semantic input になる。
- digest evidence が format/version を欠く。
- canonical mismatch を無視して evidence validity を主張する。
- driver wire version を core protocol version として扱う。
- SDK platform version が Signaling semantics を変更する。
- unsupported version が silent に fallback する。
- 新 contract version なしに breaking change が導入される。
- v0.1 message shape が仕様採用なしに v0.2 contract として受理される。
- accepted compatible versions が列挙されない。
- unsupported version が cataloged reason で reject されない。
- SDK/platform compatibility が core semantics を変更する。
- removal が documented compatibility window と evidence plan なしに発生する。
- SDK public API drift が compatibility/deprecation rule なしに受理される。
- flag value が hidden domain policy になる。
- absent capability が silent に fallback する。
- experimental stage が runtime readiness として扱われる。
- SDK/platform capability が server semantics から乖離する。
- feature removal が compatibility/deprecation evidence なしに発生する。
- feature flag が excluded feature の admission authority として使われる。
- runtime feature/capability switch が reconfiguration generation と apply scope を欠く。
- wire frame shape が core API になる。
- core reason category/code が external error mapping で失われる。
- pre-core conversion failure が domain state machine に入る。
- correlation rule が仕様の更新なしに driver ごとに相違する。
- protocol version semantics が driver に所有される。
- canonical evidence hash が driver wire formatting に依存する。
