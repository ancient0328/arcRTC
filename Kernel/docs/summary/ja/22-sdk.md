# sdk: Signaling-only 境界・platform parity・reconnect/resumption・public API contract generation・native command evidence

状態: public summary projection
日付: 2026-07-06 JST

## 目的

本章は arcRTC v0.2 Kernel における SDK 境界を、他文書・実コードを参照せずに完全自己完結で規定します。対象は次の5領域です: SDK Signaling-only 境界、TypeScript / Android / iOS の platform parity、reconnect / session resumption の全状態と手順、public API contract generation / projection、native SDK command evidence 境界。本章単独で再現実装が可能な粒度を与えます。

SDK は利用者が Signaling contract に接続するための client boundary であり、media framework でも regulated workflow SDK でもありません。SDK が media、auth issuance、regulated domain support を所有しないことを固定します。SDK local の挙動（reconnect、threading、lifecycle、error wrapper、public API 形状）が server-side semantics に黙示的に昇格しないよう、owner と failure mapping を全て内在化します。

本章は依存方向の表記を `A <- B`（B が A に依存）とします。SDK は独立した Signaling-only 境界であり、regulated を直接所有しません。`sdk -> regulated` と `regulated -> sdk` の双方を禁止します。

---

## A 部 SDK Signaling-only 境界

### A-1 SDK Scope（所有する関心事）

SDK は次を所有します（許可）。

- signaling endpoint connection
- signaling command send
- signaling event receive
- correlation ID propagation
- message codec
- typed error mapping
- reconnect / close behavior（client transport の関心事として）
- signaling session に必要な local SDK state
- Signaling contract のための version / capability value の send/receive、および negotiation result の handling

### A-2 SDK Out of Scope（所有しない関心事）

SDK は次を所有しません（禁止）。

- PeerConnection abstraction / PeerConnection lifecycle
- camera / microphone capture
- media track rendering / capture / render
- screen sharing
- recording
- chat
- DataChannel application semantics
- UI / end-user workflow
- auth issuance / token issuance
- user account management / role authorization
- regulated workflow
- medical data model

v0.2 initial scope からの不在は、それ自体では実装の欠落（gap）ではありません。out-of-scope feature が core 契約の admission なしに v0.2 capability として exposed / tested / claimed されたときに限り境界違反となります。

### A-3 Version Negotiation 境界

Signaling contract の version negotiation semantics は core が所有します。SDK は supported version / capability value を send し、accepted または rejected の result を handle してよい（許可）。SDK は version または capability について server-side の accept / reject semantics を decide してはなりません（禁止）。

### A-4 Reconnect / Resumption 境界（要約。詳細は C 部）

SDK reconnect は client-local transport behavior です。session resumption は、reconnect の契約（C 部）が許す場合に限り、client-side の correlation と pending command state を preserve してよい。new または accepted の Signaling contract event なしに、server-side の room membership、participant lifecycle、SFU route、TURN allocation、acceptance result を recreate してはなりません（禁止）。

### A-5 Regulated Rule

SDK は regulated support を直接所有しません（禁止）。regulated integration が必要な場合でも、regulated は SDK event ではなく core が所有する opaque communication event、audit pointer、non-sensitive tag を参照します。`regulated -> sdk` と `sdk -> regulated` を禁止します。

### A-6 SDK Error Boundary（閉集合 SDK-local error code）

SDK error は閉じた SDK-local code として次に分類します。

| SDK error code | Owner | 意味 | Server reason 必須 |
|---|---|---|---|
| `connection_failed` | sdk | client transport connection failed | no |
| `server_protocol_violation` | sdk wrapper, core semantics | server `ProtocolViolation` event を受信 | yes |
| `local_protocol_violation` | sdk | send 前に SDK が local protocol misuse を検出 | no |
| `rejected_by_server` | sdk | server が command または negotiation を reject | yes |
| `timeout` | sdk | client-side wait deadline を超過 | no |
| `local_unsupported_version` | sdk | SDK が configured Signaling version を send できない | no |
| `server_unsupported_version` | sdk wrapper, core semantics | server が Signaling version を reject | yes |
| `malformed_event` | sdk | event を SDK Signaling model に decode できない | no |
| `local_serialization_error` | sdk | SDK が local command を encode できない | no |
| `closed_by_remote` | sdk | remote が Signaling connection を close | catalog reason を伴う server-origin close では必須、catalog reason を伴わない local transport close では不在 |

不変条件（error boundary）:

- SDK は server-side rejection reason を別 semantic に変換してはなりません（禁止）。
- SDK-local error category は wrapper のみです。
- server-side catalog reason を carry してよいのは `rejected_by_server`、`server_protocol_violation`、`server_unsupported_version`、`closed_by_remote` のみです。
- これらの SDK error が server-side catalog reason を carry するとき、SDK は cataloged の `category` と `code` を preserve しなければなりません（必須）。
- SDK は server-side catalog reason を SDK-local wrapper text のみに collapse してはなりません（禁止）。

### A-7 A 部 Collapse Conditions

次が成立するとき本 A 部の判断は崩れます。

- SDK が PeerConnection / media を core contract として公開する。
- SDK が regulated API を直接所有する。
- regulated が SDK event / SDK API に依存する。
- platform ごとに Signaling command / event semantics が異なる。
- SDK が server-side accept / reject rule を再実装する。
- SDK が version / capability accept-reject semantics を所有する。
- SDK reconnect / session resumption が server-side state を暗黙に復元する。
- SDK が生成した / public な API artifact が server semantic authority になる。
- SDK が out-of-scope feature を supported server capability として公開する。

---

## B 部 Platform Parity（TypeScript / Android / iOS）

### B-1 境界

| Surface | Owner | Rule |
|---|---|---|
| Signaling command / event semantics | core | platform 差分なし |
| SDK public API shape | sdk | platform idiom は許可する |
| platform transport implementation | sdk または driver implementation layer | core semantics を変更しない |
| error mapping | sdk wrapper（core reason preservation 付き） | server reason category/code を保持 |
| threading / lifecycle integration | sdk | client-local concern のみ |
| regulated workflow | regulated | SDK は直接所有しない |

### B-2 Parity Dimensions（閉集合）

v0.2 initial SDK parity は次の dimension に限定します。

| Dimension | Required parity |
|---|---|
| command set | connect, close, join, leave, offer, answer, ice candidate, capability/version negotiation |
| event set | connected/closed, join result, participant lifecycle, offer/answer/ice relay, rejection/protocol violation |
| correlation | 生成 / 供給された correlation ID が command ごとに traceable |
| version negotiation | 同一 semantic の accept/reject behavior |
| compatibility / deprecation | supported platform ごとに同一の accepted/rejected version semantics |
| capability exposure | supported platform ごとに同一の server-defined capability semantics |
| server reason preservation | server reason が存在するとき同一の catalog category/code |
| local error classes | platform-specific wrapper は許可、ただし closed local category が必須 |
| lifecycle | local cancel/close は command なしに server-side state transition になってはならない |
| reconnect/session resumption | 同一の client-local retry/resumption class と同一の server reason preservation |
| public API projection | source contract から同一の semantic command/event/reason/correlation matrix |

新 parity dimension は v0.2 初期 scope 外です。

### B-3 Platform API Rule

platform idiom は許可します。例:

- TypeScript は Promise / callback / event emitter style を公開してよい。
- Android は suspend / callback / Flow style を公開してよい。
- iOS は async / delegate / Combine-like wrapper style を公開してよい。

これらの API style 差分は次を変更してはなりません（禁止）。

- command semantics
- event semantics
- server reason category/code
- correlation rule
- version negotiation semantics
- SDK out-of-scope の media/regulated 境界
- reconnect/session resumption semantics

### B-4 Error Mapping Rule

SDK-local error は wrapper error です。server-origin rejection は server catalog category/code を preserve しなければなりません（必須）。

| Error origin | SDK behavior |
|---|---|
| local serialization failure | SDK-local closed error |
| local transport connection failure | SDK-local closed error |
| local timeout | SDK-local closed error |
| server rejection | SDK wrapper ＋ preserved server reason |
| server protocol violation | SDK wrapper ＋ preserved server reason |
| malformed server event | SDK-local closed error、server reason を捏造しない |

SDK は server の `token_verification_failed`、`room_closed`、`unsupported_command_version`、その他 cataloged reason を platform-only free text に変換してはなりません（禁止）。

### B-5 Threading / Lifecycle Rule

threading、coroutine、event loop、lifecycle callback、cancellation primitive は platform-local です。これらは server room state、participant state、SFU route state、TURN allocation state を定義しません。

local SDK close は client transport を close してよい。ただし Signaling leave command が送出され contract を通じて accepted/rejected されない限り、server-side leave として報告してはなりません（禁止）。SDK reconnect/resumption は client transport を retry し、または許可された client-local pending command state を replay してよいが、それは reconnect 契約（C 部）が定義する範囲に限ります。server-side の participant、room、SFU、TURN state を silently に recreate してはなりません（禁止）。

### B-6 Test Parity Rule

各 platform SDK は次を cover する contract test を持たなければなりません（必須）。

- 同一 semantic command set の command encoding
- 同一 semantic event set の event decoding
- server reason preservation
- local error wrapper classification
- version negotiation handling
- surface が supported / removed と列挙されたときの compatibility/deprecation behavior
- capability disabled behavior と server reason preservation
- correlation propagation
- reconnect/session resumption class、retry bound、server reason preservation

test 結果は採用された場合に限り evidence です（テスト evidence の採用は testing 規範が所有）。SDK public API projection evidence は D 部に従います。

### B-7 Prohibitions（platform parity）

- ある platform が Signaling contract に無い追加の server-side semantics を公開する。
- platform-specific lifecycle event が implicit に server-side command になる。
- SDK error mapping が server reason category/code を消去する。
- SDK が regulated package を import し、または regulated workflow を公開する。
- SDK が PeerConnection/media を v0.2 Signaling contract として公開する。
- platform SDK が core internals または driver concrete types を public API として依存する。
- SDK reconnect/resumption が accepted Signaling command なしに server-side state を捏造する。
- 生成された SDK artifact が server semantics の source-of-truth になる。

### B-8 B 部 Collapse Conditions

- TypeScript / Android / iOS の command または event semantics が分岐する。
- platform wrapper が server-side reason を隠す。
- lifecycle/threading behavior が core protocol state を変える。
- SDK public API が regulated または media responsibility に依存する。
- contract test が共通の semantic command/event matrix を識別できない。
- reconnect/session resumption behavior が platform ごとに分岐し、または server reason を隠す。
- SDK public API projection が source Signaling contract reference を欠く。

---

## C 部 Reconnect / Session Resumption の全状態と手順

### C-1 境界

| 関心事 | Owner | Rule |
|---|---|---|
| SDK local reconnect loop | sdk | client-local behavior |
| reconnect backoff timer | sdk | server state を定義しない |
| Signaling reconnect command/event | contract が admit する場合の core | explicit command semantics が必須 |
| server participant state | core/signaling | SDK local connection が単独で mutate しない |
| session resumption policy | 契約が定義する場合の core のみ | implicit restore なし |
| transport reconnection I/O | sdk/driver | external implementation detail |
| durable restore | server core/drivers/entrypoints | SDK reconnect とは別 |

SDK local reconnect は server-side resumption ではありません。command idempotency、replay window、response replay、correlation rule は command idempotency 規範が所有します。

### C-2 Reconnect Classes（閉集合）

v0.2 initial architecture の reconnect class は次に限定します。

| Class | 意味 | Claim limit |
|---|---|---|
| `local_transport_reconnect` | SDK が underlying transport を reconnect | server participant restoration ではない |
| `signaling_rejoin_command` | SDK が explicit join/rejoin command を送出 | server decision が必須 |
| `event_stream_resubscribe` | SDK が client event stream を resubscribe | missed-event proof ではない |
| `session_resumption_requested` | contract が存在する場合の explicit resumption request | core contract が admit しない限り denied |
| `reconnect_exhausted` | SDK local retry/backoff limit に到達 | local SDK failure であり server state ではない |

新 reconnect class は v0.2 初期 scope 外です。

### C-3 Resumption Rule（採用前提条件）

session resumption は、Signaling contract が次を定義しない限り禁止します（fail-closed）。

- resumption command;
- identity/correlation requirements;
- accepted prior state references;
- replay/missed-event behavior;
- command idempotency と response replay behavior;
- expiration window;
- failure reasons;
- SDK parity behavior;
- evidence requirements。

その契約が無い場合、SDK は通常の join/leave/close semantics を使い、server rejection reason を preserve しなければなりません（必須）。

### C-4 Failure Mapping（閉集合 reason）

| 失敗 | 必須 reason |
|---|---|
| session resumption が許可されない | `session_resumption_not_allowed` |
| SDK reconnect attempts が exhausted | `sdk_reconnect_exhausted` |
| SDK replay が server idempotency policy と conflict | `idempotency_payload_mismatch` または `replay_not_allowed` |
| server が explicit join/rejoin を reject | server cataloged reason |
| local transport send/receive failed | SDK-local closed error または mapped network failure |
| operation cancelled | `operation_cancelled` または SDK-local cancellation class |
| operation deadline exceeded | `operation_deadline_exceeded` |

SDK-local error は server-side reason を捏造してはなりません（禁止）。

### C-5 Evidence Rule（reconnect）

SDK reconnect evidence は次を record しなければなりません（必須）。

- platform;
- reconnect class;
- local retry/backoff policy;
- 送出した場合の explicit server command;
- server reason preservation;
- resumption が scope 内であったか;
- close-not-claimed scope。

fake server または fake driver を用いる reconnect test は test double 規範に従います。

### C-6 Prohibitions（reconnect）

- SDK local reconnect が server acceptance なしに participant を joined とマークする。
- reconnect success が media readiness として報告される。
- SDK が server resumption contract を捏造する。
- replay policy なしに missed events が replayed と仮定される。
- platform-specific reconnect behavior が Signaling semantics を変える。
- reconnect exhaustion が normal close として隠される。

### C-7 C 部 Collapse Conditions

- SDK reconnect が command なしに server-side state を mutate する。
- Signaling contract なしに resumption が accepted される。
- SDK platform wrapper が異なる server semantics を公開する。
- reconnect evidence が server command/reason relation を omit する。
- local reconnect が durable recovery proof として用いられる。

---

## D 部 Public API Contract Generation / Projection

### D-1 境界

| 関心事 | Owner | Rule |
|---|---|---|
| Signaling command/event semantics | core/signaling contract | SDK projection の source-of-truth |
| SDK public API shape | sdk | platform idiom 許可、semantics 不変 |
| contract projection rule | sdk ＋ docs governance | mapping table と golden evidence が必須 |
| generated code/artifact | build tooling/sdk | semantic authority ではない |
| SDK compatibility snapshot | testing/docs support | report に採用されたとき evidence |
| migration guide | docs/sdk | 単独では compatibility proof ではない |

SDK public API は platform idiom により異なってよいが、semantic な command/event/reason/correlation behavior は分岐してはなりません（禁止）。

### D-2 Contract Artifact Classes（閉集合）

| Class | 意味 | Rule |
|---|---|---|
| `semantic_contract_model` | core が所有する Signaling command/event/reason matrix | source-of-truth |
| `platform_api_projection` | ある platform の SDK API mapping | semantic contract model を cite すること |
| `generated_sdk_type` | generated または mechanical type artifact | source-of-truth ではない |
| `sdk_golden_snapshot` | deterministic な SDK contract fixture | canonical serialization / fixture rule が適用される |
| `sdk_compatibility_matrix` | supported platform/version behavior | test evidence が必須 |
| `sdk_migration_guide` | user-facing migration document | test なしでは proof ではない |

新 artifact class は v0.2 初期 scope 外です。

### D-3 Projection Rule（projection ごとの必須宣言）

各 SDK public API projection は次を declare しなければなりません（必須）。

- source Signaling command/event;
- platform API symbol;
- request/response/callback/event shape;
- correlation propagation rule;
- server reason preservation rule;
- local SDK error wrapper;
- version/capability behavior;
- reconnect/resumption relation;
- unsupported surface behavior;
- out-of-scope feature の rejection/projection rule。

generated code は source contract に無い server-side semantics を追加してはなりません（禁止）。

### D-4 Drift Rule

次のいずれかが成立するとき SDK drift が存在します。

- 任意の platform が supported semantic command/event を欠く;
- server reason preservation を変える;
- correlation rule を変える;
- local lifecycle を server state として扱う;
- media/regulated/auth issuance を Signaling SDK responsibility として公開する;
- out-of-scope feature を supported server capability として公開する;
- version/capability を source contract と異なる仕方で accept する。

drift は、解消されるか compatibility/deprecation rule で明示的に removed されるまで、parity/readiness claim を block します（fail-closed）。

### D-5 Failure Mapping（閉集合 reason）

| 失敗 | 必須 reason |
|---|---|
| SDK contract generation/projection failed | `sdk_contract_generation_failed` |
| SDK platform contract drift detected | `sdk_contract_drift_detected` |
| Signaling surface に SDK public mapping が無い | `sdk_public_api_unmapped` |
| SDK golden snapshot mismatch | `sdk_golden_mismatch` |
| platform projection が Signaling semantics に違反 | `sdk_platform_projection_invalid` |
| SDK projection が admission なしに out-of-scope feature を公開 | `feature_admission_not_documented` |
| canonical golden encoding を生成できない | `canonical_serialization_failed` |
| fixture/golden data invalid | `fixture_invalid` |

### D-6 Evidence Rule / Audit Rule（projection）

SDK contract evidence は次を record しなければなりません（必須）: platform、source contract version、projection artifact class、command/event matrix、reason preservation test、correlation test、用いた場合の golden snapshot class、close-not-claimed scope。SDK build success 単独では semantic parity を証明しません。

SDK public API contract decision は audit event type `sdk_public_api_contract_decision` を用います。当該 event は SDK platform、projection class、source contract version、および generation または evidence run の `CorrelationId` を carry しなければなりません（必須）。

### D-7 Prohibitions / Collapse（projection）

Prohibitions:

- generated SDK type が server semantic authority になる。
- platform idiom が command/event meaning を変える。
- SDK public API が media、regulated workflow、auth issuance、server authorization を所有する。
- SDK local error が server catalog reason を消去する。
- migration guide が compatibility proof として扱われる。
- SDK golden snapshot が semantic diff classification なしに変わる。

Collapse Conditions:

- SDK projection が source Signaling contract reference を欠く。
- platform API drift が compatibility/deprecation path なしに accepted される。
- generated artifact が semantics を silently 変える。
- SDK contract evidence が command/event/reason/correlation matrix を欠く。
- SDK public API が out-of-scope の media/regulated/auth responsibility を公開する。

---

## E 部 Native SDK Command Evidence 境界

### E-1 Context

arcRTC v0.2 は SDK public contract を TypeScript / Android / iOS で扱います。source marker test は SDK boundary の静的確認には使えますが、native command success の代替にはなりません（禁止）。

- Android では Gradle wrapper、Android SDK、Kotlin/Android Gradle Plugin、compileSdk、unit test、build、lint が同一 command evidence surface に含まれます。
- iOS では SwiftPM test command が native command evidence surface です。

### E-2 Android Native Command Evidence Rule

Android native SDK command evidence は `sdk/android` を working directory とし、Gradle wrapper command で次を同時に実行します（必須）。

- `:sdk:testDebugUnitTest`
- `:sdk:assembleDebug`
- `:sdk:lintDebug`
- `--warning-mode all`

Android report は次を持たなければなりません（必須）: exact `Command:`、exact `Working directory:`、`BUILD SUCCESSFUL`、`Warning/deprecation output: none observed`。Android lint report は warning / error を含んではなりません（禁止）。Android command は system Gradle ではなく project Gradle wrapper を使います。

### E-3 iOS Native Command Evidence Rule

iOS native SDK command evidence は `sdk/ios` を working directory とする `swift test` であり、source marker test で代替してはなりません（禁止）。

### E-4 TypeScript / Cross-platform Governance Rule

TypeScript SDK test は TypeScript package 内に閉じます。Android / iOS source projection の cross-platform governance は native tests または cross-platform governance tests が扱い、TypeScript package test に重複させてはなりません（禁止）。

### E-5 Consequences / Claim Scope

- 曖昧な `SDK Android tests` という gate label は使わず、`SDK Android unit/build/lint command` とします。
- Android build success、unit test success、lint no-issue は native command evidence であり、production readiness / live readiness / public distribution / server semantic correctness の証明ではありません。
- Native command report parser は substring marker だけで採用してはなりません（禁止）。

### E-6 E 部 Collapse Conditions

- source marker test を native command success の代替にする。
- Android command から `:sdk:testDebugUnitTest`、`:sdk:assembleDebug`、`:sdk:lintDebug`、`--warning-mode all` のいずれかを外す。
- Android lint warning / error を残したまま adopted evidence にする。
- system Gradle 実行と wrapper 実行の command surface を混在させる。
- TypeScript package test が Android / iOS source projection の重複検査を所有する。

---

## F 部 不変条件（Invariants）の要約

- SDK は Signaling-only 境界であり、media / auth issuance / regulated workflow を所有しない。これらは out-of-scope feature であり、core 契約の admission なしの公開は境界違反。
- Signaling semantics（command / event / reason / correlation / version negotiation）は platform によって不変。platform は API idiom のみを変える。
- server-side rejection reason（catalog category/code）は wrapper を通じて常に preserve され、`rejected_by_server` / `server_protocol_violation` / `server_unsupported_version` / `closed_by_remote` のみが carry してよい。
- SDK local の reconnect / threading / lifecycle / close は、accepted Signaling command なしに server-side state（room / participant / SFU / TURN / acceptance）へ昇格しない。
- session resumption は専用 Signaling contract が全項目を定義しない限り fail-closed。
- generated / public な SDK API artifact は source-of-truth ではなく、`semantic_contract_model` が source-of-truth。drift は parity/readiness claim を block。
- native command evidence（Android unit/build/lint、iOS swift test）は build/test の証明であって、production / live / distribution / server semantic correctness の証明ではない。
- `sdk -> regulated` と `regulated -> sdk` は双方禁止。
