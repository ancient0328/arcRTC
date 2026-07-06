# drivers-observability-privacy

状態: public summary projection
日付: 2026-07-06 JST

## 目的

本章は arcRTC v0.2 Kernel の observability / privacy driver family の現行完全仕様を、本章のみで再現実装可能な粒度で内在化することを目的とします。本章は observability 境界（tracing / metrics exporter / log sink）、observability signal taxonomy / cardinality / sampling の閉集合、privacy / redaction / retention 規則（raw secret / token / media / regulated payload を core / audit / log / report へ混入させないための owner と data class）を所有者・閉集合語彙・failure mapping・禁止/許可・fail-closed 条件まで落とさずに内在化します。

依存方向の表記: `A <- B` は「B が A に依存」を意味します。observability は core が所有する port を実装する driver family であり、audit event meaning / quality decision / domain decision の正を所有しません。observability signal は default で audit event ではありません。本章は exporter 実装、dashboard、alert runtime、production monitoring、法令適合、production privacy certification、実運用 retention 設定済みを主張しません。

---

## 1. Observability Boundary（`drivers/observability`）

### 1.1 Boundary（所有割当、閉集合）

observability は tracing、metrics exporter、log sink を扱いますが、audit event meaning、quality decision、domain decision の正を所有しません。

| Surface | Owner | Rule |
|---|---|---|
| audit event meaning | core | audit event 規則 |
| audit hash-chain semantics | core | audit hash-chain 規則 |
| quality metric model and decision | core | quality metrics 規則 |
| metrics export format | driver | Prometheus-style, tracing, vendor exporter |
| log / trace sink | driver | external sink and formatting |
| entrypoint-level exporter selection | entrypoints | typed configuration and wiring only |

signal taxonomy、cardinality、sampling、alert boundary は第2節の observability signal taxonomy に従います。

### 1.2 Metrics Rule（metric 規則）

core は metric が quality または resource-bound decision semantics に参加する場合のみ metric name を所有します。driver は追加の operational metric を export してよいが、driver-exported metric は authoritative domain state ではありません。metric が quality decision に参加する場合、その decision rule は quality metrics 規則で定義しなければなりません。exporter-side aggregation は quality decision semantics を変えてはなりません（禁止）。metric label class、cardinality、sampling は observability signal taxonomy（第2節）に従わなければなりません。

### 1.3 Logging Rule（log 規則）

log と trace は operational diagnostics です。それらは audit event、Closed Gate Report、reproducible test evidence を置換しません。log は raw secret、raw token、raw credential、packet payload bytes、regulated domain payload を運んではなりません（禁止）。log が reference を含む場合、core identity and reference 規則の core-owned opaque reference を使用しなければなりません。log / trace / metric label / report / raw token / raw packet payload / regulated payload の redaction と retention 規則は privacy / redaction / retention 規則（第3節）に従います。trace / span / log signal class は evidence として採用する前に declare しなければなりません。

### 1.4 Failure Mapping（閉集合）

observability failure は driver failure または required な場合 resource-bound decision として記録しなければなりません。それは domain decision を success に変えたり required rejection を隠したりしてはなりません（禁止）。

| Failure | Required reason |
|---|---|
| metrics export failed | `metrics_export_failed` |
| metrics backlog bound exceeded | `metrics_backlog_bound_exceeded` |
| signal taxonomy invalid | `observability_signal_invalid` |
| metric label cardinality exceeded | `metric_cardinality_exceeded` |
| required telemetry sampling policy absent | `telemetry_sampling_policy_missing` |
| driver shutdown | `driver_shutdown` |

### 1.5 Evidence Rule（evidence 規則）

observability output は investigation を支援してよいが、日付付き証跡 document が correlation ID、command、environment、reproducible procedure を記録しない限り close / complete / ready evidence として採用されません。time window、source、bound definition を欠く metrics sample は performance / stability claim の evidence ではありません。signal class、cardinality class、sampling policy を欠く observability sample は diagnostic only です。

### 1.6 Prohibitions（禁止、閉集合）

- metrics exporter が quality decision を所有する。
- log text が closed reason vocabulary になる。
- tracing span name が audit event type になる。
- observability driver が domain flow を変えるため network / persistence driver に直接依存する。
- raw token / credential / packet payload / regulated payload が log される。
- verification success を主張しながら missing metrics export が無視される。
- alert / metric / trace signal が domain decision を直接駆動する。
- metric label cardinality が unbounded である。

### 1.7 Observability Boundary Collapse Conditions（崩壊条件）

observability output が audit event semantics を置換する。exporter aggregation が core quality decision を変える。log / free-text が authoritative reason になる。metrics backlog が unbounded または unaudited である。operational log が reproducible report なしに closeout evidence として使用される。observability evidence を主張しながら signal taxonomy または sampling policy が省略される。

---

## 2. Observability Signal Taxonomy / Cardinality / Sampling

### 2.1 Boundary（所有割当、閉集合）

本節は metrics / logs / traces / alerts が audit、quality decision、runtime evidence、operator action と混同されないよう signal class と採用条件を固定します。observability signal は default で audit event ではありません。

| Signal surface | Owner | Rule |
|---|---|---|
| audit event | core model | audit event 規則 |
| quality metric used for decision | core quality policy | quality metrics 規則 |
| operational metric export | driver observability | diagnostic unless adopted by report |
| trace/span/log formatting | driver observability | no domain meaning authority |
| alert rule | entrypoints/operations policy | cannot define core reason |
| signal sampling/cardinality policy | observability policy or profile | must be explicit before the evidence is accepted |

### 2.2 Signal Classes（閉集合）

v0.2 initial architecture の signal class は次に限定します。新規 signal class は v0.2 初期 scope 外です。

| Class | Meaning | Evidence rule |
|---|---|---|
| `audit_signal` | canonical audit event or hash-chain record | the audit policy owns meaning |
| `quality_decision_metric` | metric used by quality decision | the quality policy and unit normalization required |
| `resource_bound_metric` | metric used by resource bound decision | resource bound owner tuple required |
| `operational_metric` | diagnostic/exported metric | report required for evidence |
| `trace_span` | execution trace diagnostic | not decision authority |
| `structured_log` | redacted log diagnostic | not reason vocabulary |
| `alert_signal` | derived operator notification | not domain decision |
| `profiling_signal` | CPU/memory/runtime diagnostic | not correctness proof |

### 2.3 Cardinality and Label Rule（必須宣言項目）

metric label、log field、trace attribute は次を declare しなければなりません。signal class、owner、allowed reference types、sensitive data classification、maximum cardinality class、sampling が適用される場合の sampling rule、retention/redaction rule、evidence 受理規則。user identity、token claims、raw packet data、SDP/ICE material、regulated payload、external free-text に由来する unbounded label は禁止です。

### 2.4 Sampling Rule（sampling 規則）

sampling は diagnostic volume を減らしてよいが、次を変えてはなりません（禁止）。audit event completeness、resource-bound decision evidence、quality policy が明示的に許可しない限り quality decision input、Closed Gate evidence。sampling が required evidence を隠す場合、その evidence claim は fail closed しなければなりません。

### 2.5 Failure Mapping（閉集合）

| Failure | Required reason |
|---|---|
| signal does not match allowed taxonomy | `observability_signal_invalid` |
| metric label cardinality bound exceeded | `metric_cardinality_exceeded` |
| required sampling policy absent | `telemetry_sampling_policy_missing` |
| alert signal attempts to drive domain decision | `alert_signal_not_allowed` |
| observability export is not allowed by privacy/retention policy | `observability_export_not_allowed` |
| metrics export failed | `metrics_export_failed` |
| metrics backlog bound exceeded | `metrics_backlog_bound_exceeded` |

### 2.6 Evidence Rule（evidence 規則）

observability evidence は signal class、sampling、cardinality、time window、source、redaction、report rerun condition を記録しなければなりません。これらの field を欠く operational metric / log output は diagnostic only に留まります。

### 2.7 Audit Rule

observability signal decision は audit event type `observability_signal_decision` を使用します。当該 event は、rejection/drop/failure が cardinality または export-bound のとき、signal class、signal reference、resource owner field を持たなければなりません（必須）。

### 2.8 Prohibitions と Collapse Conditions

禁止（閉集合）: trace span name が audit event type になる。alert status が domain decision になる。sampled operational metric が complete audit evidence として使用される。metric label が raw identity / token / packet payload / regulated payload を含む。dashboard state が runtime correctness proof として扱われる。exporter aggregation が quality decision semantics を変える。崩壊条件: signal class が省略される。unbounded cardinality が許可される。sampling policy が required evidence を隠す。alert / metric / log が audit または reason catalog を置換する。observability signal taxonomy が core 契約更新なしに driver ごとに異なる。

---

## 3. Privacy / Redaction / Retention

### 3.1 Boundary（所有割当、閉集合）

本節は generic communication core、drivers、entrypoints、sdk、regulated、observability、audit、reports の間で raw secret、raw token、raw media、regulated payload を混入させないための owner と retention class を固定します。observability signal taxonomy と metric/log label class は第2節に、secret rotation evidence は secret rotation lifecycle 規則に、edge/proxy metadata redaction は edge/proxy trust boundary 規則に従います。

| Surface | Owner | Rule |
|---|---|---|
| core reason/reference/audit model | core | opaque reference と non-sensitive tag だけを所有 |
| raw credential/token/key material | driver/entrypoints | core/audit/log/report へ出さない |
| raw RTP/RTCP/media payload | driver | packet lifecycle 内に閉じる |
| regulated payload | regulated | generic core に混入しない |
| log/trace/metric formatting | driver | redaction 後の external sink 表現 |
| report evidence | dated evidence reports | raw sensitive data を含めず correlation/reference を記録 |
| SDK client local data | sdk | server/core source-of-truth ではない |

### 3.2 Data Classes（閉集合）

v0.2 initial architecture の data class は次に限定します。新規 data class は v0.2 初期 scope 外です。

| Data class | Allowed owner | Retention rule |
|---|---|---|
| `core_reference` | core | opaque ID/reference として保持可 |
| `catalog_reason` | core | closed reason category/code として保持可 |
| `non_sensitive_tag` | core/driver/regulated | allowlist にある tag のみ |
| `raw_secret` | entrypoints/driver | audit/log/report/core state へ出さない |
| `raw_token` | driver boundary | verification input として bounded に扱い、保持しない |
| `raw_key_material` | driver | bounded cache only; log/report 禁止 |
| `raw_packet_payload` | driver | packet lifecycle retention only |
| `regulated_payload` | regulated | generic core/drivers/sdk/entrypoints path へ混入しない |
| `diagnostic_detail` | driver | non-authoritative; redaction mandatory |
| `edge_proxy_metadata` | driver/entrypoints | redacted evidence only after trust class is declared |

### 3.3 Redaction Rule（allowlist と禁止 field、閉集合）

redaction は sink formatting の後付けだけではなく、境界通過前の allowlist で行います。

許可される evidence/log field（許可、閉集合）:

- correlation ID;
- startup run ID;
- opaque room/session/participant/endpoint/packet/allocation references;
- event type code;
- decision outcome;
- reason category/code;
- bounded resource name and owner tuple;
- non-sensitive tag explicitly allowed by the core contract;
- edge/proxy class and trusted metadata class when redacted and admitted.

禁止される evidence/log field（禁止、閉集合）:

- raw token;
- raw credential;
- raw key material;
- SDP body or ICE string when it contains sensitive network detail and no redacted reference has been defined;
- raw RTP/RTCP/media payload bytes;
- regulated payload;
- patient/user profile or entrypoint role as protocol identity;
- raw forwarded chain, raw client IP, host, origin, or SNI when no redacted reference has been defined.

### 3.4 Retention Rule（閉集合）

retention duration/bytes/count bound は resource bounds / backpressure 規則または specialized retention policy に接続しなければなりません。

| Retention target | Rule |
|---|---|
| audit event | retains closed event fields and hash-chain data only |
| audit hash-chain | retains tamper-evidence material, not mutable domain state |
| logs/traces | operational diagnostics; no raw sensitive data |
| metrics | labels must not contain raw identifiers or sensitive payload |
| packet cache | bounded driver-local retention only |
| key cache | bounded driver-local retention only |
| evidence report | correlation/reference and reproducible procedure only |

### 3.5 Report Rule（report 規則）

Closed Gate Report、verification report、benchmark report、analysis report は raw sensitive material を根拠として貼り付けてはなりません（禁止）。必要な場合は redacted excerpt、opaque reference、hash/reference、reproducible command/result summary へ変換します。report が raw sensitive data を含む場合、その report は close / complete / ready evidence として採用してはなりません。

### 3.6 Failure Rule（閉集合）

| Failure | Required handling |
|---|---|
| redaction cannot be applied | stop affected logging/reporting path; do not emit raw data |
| secret source unavailable | `secret_unavailable` |
| token/key verification detail cannot be safely exposed | preserve cataloged non-sensitive reason only |
| packet payload requested for report | reject the report as evidence unless redacted reference exists |
| metrics label would contain sensitive data | drop or rewrite label before export |
| observability signal label cardinality would expose sensitive identity | drop or rewrite signal before export |
| rotation evidence would expose raw secret material | reject the report as evidence |
| edge/proxy metadata would expose sensitive network identity | redact, drop, or reject the report as evidence |

### 3.7 Prohibitions（禁止、閉集合）

- raw secret/token/key が core state、audit event、log、metric label、SDK public error、report に含まれる。
- packet payload bytes が closeout evidence として使用される。
- regulated payload が generic core audit event に追加される。
- redaction が external sink default behavior に委ねられる。
- free-text diagnostic detail が authoritative reason になる。
- report が reproducibility 向上のため sensitive source material を保持する。
- raw proxy/header/source-address material が authoritative identity として log / report される。

### 3.8 Privacy / Redaction / Retention Collapse Conditions（崩壊条件）

raw sensitive data が authoritative data として core/audit/report に渡る。retention に bound または owner が無い。redaction failure が raw data を emit する。metric/log label が sensitive identity または payload を含む。report evidence が sensitive material を露出せずに共有できない。secret rotation または observability evidence が raw sensitive material を露出する。edge/proxy metadata evidence が redaction と trust classification を欠く。

---

## 4. 章全体の fail-closed 不変条件

本章の全 observability / privacy 規則に共通する fail-closed 不変条件は次のとおりです（必須）。observability failure は domain decision を success に変えたり required rejection を隠したりしない。すべての observability failure は catalog 済み reason code を名指しする。metric label cardinality は bounded であり、signal class / sampling policy は evidence として採用される前に明示される。redaction は境界通過前の allowlist で行い、適用できない場合は当該 logging/reporting path を停止して raw data を emit しない。raw secret / token / key material / packet payload / regulated payload は core state / audit event / log / metric label / SDK public error / report のいずれにも入らない。report が raw sensitive data を含む場合、その report は close / complete / ready evidence として採用してはなりません。
