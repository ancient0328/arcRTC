# regulated: 独立境界・enrichment lifecycle・out-of-scope feature admission

状態: public summary projection
日付: 2026-07-06 JST

## 目的

本章は arcRTC v0.2 Kernel における regulated 境界を、他文書・実コードを参照せずに完全自己完結で規定します。対象は次の3領域です: regulated 独立境界（optional domain support）、core event / audit pointer / non-sensitive tag を参照する場合の enrichment lifecycle の全状態、out-of-scope feature の admission / exclusion 機構（admission 判定規則と全 non-goal の扱い・将来採用条件）。本章単独で再現実装が可能な粒度を与えます。

arcRTC v0.2 は generic WebRTC communication foundation です。regulated は optional domain support であり、generic communication foundation の一部ではありません。regulated support（audit enrichment、HIPAA/GDPR helper 等）を generic communication core に混入してはなりません。chat、recording、screen share、DataChannel application semantics、UI/end-user workflow、media capture workflow、regulated workflow が Signaling / SFU / SDK / entrypoints に黙示的に混入しないよう、排除条件と将来採用条件を固定します。

本章は依存方向の表記を `A <- B`（B が A に依存）とします。許可し得る方向は `regulated -> core` のみであり、これも opaque communication event、audit pointer、non-sensitive tag への参照に限ります。`core -> regulated`、`drivers -> regulated`、`entrypoints -> regulated`、`sdk -> regulated`、`regulated -> sdk`、`regulated -> drivers`、`regulated -> entrypoints` を禁止します。

---

## A 部 Regulated 独立境界

### A-1 Allowed Dependency（許可される唯一の方向）

条件付きで許可し得る依存方向は次のみです。

```text
regulated -> core
```

この依存は、opaque communication event、audit pointer、non-sensitive tag への参照に限定します。regulated は generic audit event base schema や quality metric model の正を所有しません（禁止）。

### A-2 Prohibited Dependencies（禁止される方向）

次を禁止します。

```text
core -> regulated
drivers -> regulated
entrypoints -> regulated
sdk -> regulated
regulated -> sdk
regulated -> drivers
regulated -> entrypoints
```

### A-3 Regulated May Own（所有し得る関心事）

regulated は次を所有し得ます（許可）。

- domain-specific enrichment
- compliance mapping helper
- audit pointer mapping
- non-sensitive tag classification
- external regulatory integration helper
- domain-specific tests
- regulatory pointer mapping
- regulated-domain test fixtures

### A-4 Regulated Must Not Own（所有してはならない関心事）

regulated は次を所有してはなりません（禁止）。

- Signaling protocol semantics
- TURN protocol semantics
- SFU routing semantics
- SDK Signaling contract
- generic audit event base schema
- transport identity model
- token issuance

### A-5 Generic Core Responsibilities（generic core が所有する関心事）

generic core は次だけを所有します。

- communication event model
- opaque reference
- audit event base schema
- token verification boundary
- quality metrics model

### A-6 Data Boundary

generic core は sensitive domain payload を持ちません。regulated は core event を opaque reference として参照し、必要な domain mapping を regulated 側で行います。core は regulated type を import しません（禁止）。

### A-7 A 部 Collapse Conditions

次が成立するとき本 A 部の判断は崩れます。

- core が regulated type を import する（`core -> regulated` が発生する）。
- SDK が regulated に依存する（`sdk -> regulated` が発生する）。
- regulated が SDK に依存する（`regulated -> sdk` が発生する）。
- regulated が drivers / entrypoints に依存する（`regulated -> drivers` / `regulated -> entrypoints` が発生する）。
- generic audit event に domain payload が必須化される。
- regulated helper が communication protocol decision の正を持つ。
- core が HIPAA/GDPR rule を所有する。
- Signaling / TURN / SFU protocol に medical workflow semantics が入る。

---

## B 部 Enrichment Lifecycle の全状態

### B-1 境界

| Surface | Owner | Rule |
|---|---|---|
| communication event semantics | core | Signaling / SFU / TURN / audit semantics |
| opaque communication event reference | core | regulated は参照のみ |
| audit pointer mapping | regulated | pointer のみ、core schema rewrite 不可 |
| non-sensitive tag classification | regulated | core required field になってはならない |
| domain-specific payload | regulated | generic core が必須化しない |
| compliance/export helper | regulated | optional support のみ |

### B-2 Enrichment Lifecycle（全 step）

v0.2 initial regulated enrichment lifecycle は次のとおりです。

1. core が opaque reference 付きで communication event / audit event を emit する。
2. regulated が許可された opaque pointer を receive または read する。
3. regulated が pointer を generic core の外で domain-specific context に map する。
4. regulated は regulated-local enrichment record を emit してよい。
5. regulated-local record は core audit pointer を参照してよいが、core audit event を mutate してはならない（禁止）。

regulated enrichment は post-core かつ optional です。core の accept/reject decision に participate してはなりません（禁止）。

### B-3 Allowed Reference Shape（参照可能な形）

regulated は次を参照してよい（許可）。

- `CorrelationId`
- `RoomId`（opaque communication room reference としてのみ）
- `ParticipantId`（opaque participant reference としてのみ）
- `AuditEventId` / audit pointer
- core 契約が admit した non-sensitive communication tag
- hash-chain record pointer

regulated は次を必須化してはなりません（禁止）。

- raw media payload
- raw token / credential
- SDK platform object
- driver buffer/packet lease
- entrypoint-specific user account object
- core event 内の medical または regulated domain payload

### B-4 Tag Rule（non-sensitive tag の規則）

non-sensitive tag は次でなければなりません（必須）。

- regulated-local vocabulary 内で closed
- generic core に required でない
- core が Signaling / SFU / TURN behavior を decide するために使用しない
- regulated validation 後の regulated-local context でのみ expose して安全

ある tag が generic communication semantics に required になった場合、それはもはや regulated enrichment ではなく、core 契約更新を要します。

### B-5 Audit Relation

regulated は regulated-local audit/enrichment record を作成してよい。それらの record は generic audit event を置換せず、audit hash chain semantics を変えません。core audit pointer は regulated の視点から immutable のままです。

### B-6 Prohibitions（enrichment lifecycle）

- core が regulated type を import する。
- regulated が Signaling / SFU / TURN accept/reject decision を所有する。
- regulated enrichment が core audit event を mutate する。
- regulated が generic communication event に domain payload を必須化する。
- SDK が regulated enrichment に依存する。
- drivers/entrypoints が generic communication path で regulated helper を call する。
- regulated tag が core の hidden authorization policy になる。

### B-7 B 部 Collapse Conditions

- regulated enrichment が generic core operation に required になる。
- core audit event schema に domain payload が追加される。
- regulated pointer が core event meaning を変える。
- regulated helper が entrypoints/drivers generic path に wire される。
- SDK platform API が regulated enrichment model に依存する。

---

## C 部 Out-of-Scope Feature Admission / Exclusion

### C-1 境界

| 関心事 | Owner | Rule |
|---|---|---|
| v0.2 initial feature scope | core 契約 | generic communication infrastructure のみ |
| out-of-scope feature request | 境界の entrypoints/sdk/drivers | cataloged reason で reject または close-not-claimed |
| future feature admission | core 契約更新 | owner, boundary, reason, evidence, migration が必須 |
| SDK public surface | sdk | out-of-scope feature を supported server behavior として公開しない |
| demo/local helper | entrypoints/demo | production policy または scope expansion になってはならない |
| regulated workflow | regulated | optional support のみ、generic core ではない |

v0.2 initial scope からの不在は、それ自体では実装の欠落（gap）ではありません。out-of-scope feature が core 契約の admission なしに v0.2 capability として exposed / tested / claimed されたときに限り境界違反となります。

### C-2 Excluded Feature Classes（閉集合・全 non-goal）

v0.2 initial architecture の excluded feature class は次に限定します。

| Class | 意味 | Rule |
|---|---|---|
| `chat_application_semantics` | user/application chat content or workflow | generic communication core から除外 |
| `recording_workflow` | media recording, storage, retrieval, retention workflow | 別の scope 拡張が無い限り除外 |
| `screen_share_workflow` | media routing を超える capture/share workflow | admit されない限り除外 |
| `datachannel_application_semantics` | SCTP/DataChannel entrypoint protocol semantics | Signaling-only SDK と core から除外 |
| `ui_end_user_workflow` | user-facing UI workflow | v0.2 infrastructure scope から除外 |
| `media_capture_workflow` | camera/microphone/screen capture permission UX | browser/native driver は許可された platform boundary を observe のみ |
| `regulated_domain_workflow` | medical/regulated domain workflow | regulated optional support のみ |

新 excluded/admitted feature class は v0.2 初期 scope 外です。

### C-3 Future Admission Rule（将来採用条件）

excluded feature の future admission は次を declare しなければなりません（必須）。

- feature class;
- owner package/layer;
- relation to generic communication core;
- public SDK/API surface（あれば）;
- driver/runtime dependency boundary;
- security/privacy/redaction boundary;
- reason catalog additions;
- audit event relation;
- evidence class;
- migration/deprecation relation。

これが存在するまで、当該 feature は fail-closed するか close-not-claimed のままでなければなりません（fail-closed）。

### C-4 Failure Mapping（閉集合 reason）

| 失敗 | 必須 reason |
|---|---|
| 要求された feature が v0.2 scope 外 | `feature_out_of_scope` |
| feature admission が core 契約に記載されていない | `feature_admission_not_documented` |
| chat semantics が要求された | `chat_not_supported` |
| recording workflow が要求された | `recording_not_supported` |
| screen share workflow が要求された | `screen_share_not_supported` |
| DataChannel application semantics が要求された | `datachannel_not_supported` |
| UI/end-user workflow が要求された | `ui_workflow_not_supported` |
| media capture workflow が core/server feature として要求された | `media_capture_not_supported` |
| regulated workflow が generic core feature として要求された | `regulated_workflow_not_supported` |

### C-5 Audit Rule / Evidence Rule

out-of-scope feature decision は audit event type `out_of_scope_feature_decision` を用います。当該 event は command-scoped のとき `CorrelationId`、feature class、requested surface、rejection reason を carry しなければなりません（必須）。

excluded feature に関わる evidence は次を record しなければなりません（必須）。

- feature class;
- requested surface;
- feature が rejected か、close-not-claimed として ignored か、別の admitted 契約でカバーされるか;
- command-scoped のときの correlation ID;
- expected outcome;
- actual outcome;
- rejection の cataloged reason;
- close-not-claimed scope。

demo、mock、test helper の behavior は feature を v0.2 scope に admit しません（禁止）。

### C-6 Prohibitions（out-of-scope feature）

- SDK が out-of-scope feature を supported server capability として公開する。
- demo/local helper が production scope を拡張する。
- chat/recording/screen share/DataChannel/UI semantics が generic core に入る。
- media capture permission state が core communication state になる。
- regulated workflow が required generic communication behavior になる。
- out-of-scope feature pass が v0.2 readiness evidence として用いられる。

### C-7 C 部 Collapse Conditions

- excluded feature が core 契約の admission なしに実装される。
- SDK public API が excluded feature support を主張する。
- demo または local helper が scope authority になる。
- out-of-scope feature rejection が cataloged reason を欠く。
- future admission が owner、security/privacy、audit、evidence boundary を omit する。

---

## D 部 不変条件（Invariants）の要約

- regulated は optional domain support であり generic communication foundation の一部ではない。許可される依存方向は `regulated -> core`（opaque communication event / audit pointer / non-sensitive tag への参照のみ）のみ。
- `core -> regulated`、`drivers/entrypoints/sdk -> regulated`、`regulated -> sdk/drivers/entrypoints` は全て禁止。
- generic core は sensitive domain payload を持たず、core は regulated type を import しない。
- regulated enrichment は post-core かつ optional であり、core の accept/reject decision に participate せず、core audit event を mutate せず、generic core operation に required にならない。
- non-sensitive tag は regulated-local vocabulary 内で closed であり、generic core に required になった時点で regulated enrichment ではなくなり core 契約更新を要する。
- v0.2 scope からの不在は実装欠落ではない。excluded feature class（chat/recording/screen share/DataChannel/UI/media capture/regulated workflow）は core 契約の admission なしの exposure/test/claim でのみ境界違反となり、要求時は cataloged reason で fail-closed。
- future admission は owner / core relation / public surface / dependency boundary / security-privacy-redaction / reason catalog / audit / evidence / migration を全て declare するまで fail-closed。
