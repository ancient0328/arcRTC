# テストと証跡体系

状態: public summary projection
日付: 2026-07-06 JST

## 目的

本章は arcRTC v0.2 Kernel の testing / evidence 境界を規定します。対象は、test tier、source-shape evidence 規則、code coverage の閾値と分母 scope 定義、evidence record の field、test double / fake driver 境界、test fixture / scenario data 境界、Native SDK command evidence 規則です。test は設計・実装・runtime の各層の証拠を分けて扱い、correlation ID・閉集合 reason・再現手順を必須とします。

本章で規範語として用いる語の意味は次のとおりです。**必須**＝満たさなければならない条件、**禁止**＝行ってはならない事項、**許可**＝行ってよい事項、**fail-closed**＝必須条件が満たせない・不明な場合に失敗側へ倒すことです。`UNKNOWN`、free-text-only reason、unclassified skip、diagnostic-only output は採用に数えてはなりません。test の pass はそれ単独で readiness を成立させません。readiness は独自の証跡を持つ別の claim です。

## Test tier

v0.2 testing tier は次に限定します。

| Tier | Scope | Evidence class |
|---|---|---|
| architecture static test | dependency direction, forbidden imports, package boundary | source-shape evidence |
| core unit test | domain model, state machine, reason catalog, use case | source/runtime-in-test evidence |
| driver contract test | conversion, port implementation, resource bounds | driver behavior evidence |
| entrypoint composition test | startup wiring, typed config, fail-closed startup | composition evidence |
| SDK contract test | Signaling-only public client behavior | public contract evidence |
| regulated optional test | regulated boundary and enrichment only | optional support evidence |
| integration test | selected end-to-end path under controlled environment | runtime-in-test evidence |
| code coverage gate | production source line coverage for core and overall source set | code coverage quality evidence |

## Source-Shape Evidence Rule

source-shape evidence は対象 package/module の effective source set を検査することが必須です。Rust crate root が source shard を使う場合、test は関連する `src` surface 配下の全 `.rs` file を走査し、generated/build output を除外することが必須です。SDK target が複数 platform source file に分かれる場合、その platform の target source set を走査することが必須です。

core source-shape evidence は semantic modular monolith boundary を test することが必須です。core における forbidden driver/entrypoint/framework/runtime/socket/DB/browser/native/cloud/concrete transport import、hidden semantic owner transfer、source shard scope drift、source-set scan gap を reject することが必須です。

監査性のため、600 lines は source comprehension の review signal です。test は file size を記録・検査してよいですが、source file が 600 lines を超えたことだけを理由に fail してはなりません。line count は、runtime behavior、production readiness、live readiness、benchmark acceptance をそれ単独で証明も反証もしません。

## Code coverage 規則（閾値・分母）

code coverage evidence は production source 実行到達性のための quality gate です。readiness / live / native command success の単独証明ではありません。内部正式名としての `enterprise-grade` は採用 line coverage threshold 群のみを指し、enterprise-ready / production-ready / live-ready / native-runtime-ready を意味しません。外部向けまたは短縮表現では、この gate を `multi-language code coverage quality gate` と記述してよいです。

### 用語定義

| Term | Definition |
|---|---|
| core line coverage | `Kernel/core/` 配下の Rust production source に対する line coverage |
| overall line coverage | Rust `core/`, `drivers/`, `entrypoints/`, `regulated/` と SDK TypeScript / Swift / Android production source について、各 language/tool が報告する covered / instrumented production lines を合算した weighted aggregate |
| project-level coverage indicator | `overall line coverage` の性質。project-level の実行到達性指標であり、各 language/tool の coverable line 定義が意味論的に完全同一であることを主張しない |
| core threshold | core line coverage `>= 90.00%` |
| overall threshold | overall line coverage `>= 85.00%` |
| core crate floor | 各 core crate の line coverage `>= 80.00%` |
| assertion-bearing coverage | test が production behavior / boundary / reject reason / state transition / value semantics を assert して到達した coverage |
| diagnostic-only coverage | import-only, smoke-only, text-inspection-only, generated-output-only, or command reachability coverage |

### 採用閾値

| Target | Threshold |
|---|---:|
| core line coverage | `>= 90.00%` |
| overall line coverage | `>= 85.00%` |
| each core crate line coverage floor | `>= 80.00%` |

この gate の pass は、production readiness、live readiness、native command success を単独では証明しません。この threshold set は測定後の数値へ後付けしたものとして扱ってはなりません。threshold policy を変更する場合は、measurement rerun 前に変更前後の rule を記録することが必須です。

### 現在の検証済みステータス

最終検証: 2026-06-29（JST）。固定された採用閾値に対する最新の実測ステータスであり、再測定のたびに上書き更新し、本章に定義された coverage コマンドで再導出できます（各測定は correlation ID・command・再現手順を記録します）。これは code coverage quality gate のステータスのみで、production / live / native readiness を主張しません。

| 指標 | 実測 | 閾値 | 判定 |
|---|---:|---:|:--|
| core line coverage | 97.50%（3548/3639） | `>= 90.00%` | PASS |
| overall line coverage（Rust ＋ TypeScript / Swift / Android SDK） | 94.49%（10660/11282） | `>= 85.00%` | PASS |
| 各 core crate line coverage floor | 最低 86.44%（core/quality） | `>= 80.00%` | PASS（21/21） |

言語別 line coverage: Rust core 97.50% / drivers 97.55% / entrypoints 89.29% / regulated 90.91%、TypeScript SDK 91.85%、Swift SDK 97.64%、Android SDK 98.43%。diagnostic-only の entrypoint `main.rs` 起動分（19.69%）は除外、除外後 overall は 97.61%。

### Denominator Rule（分母 scope 定義）

denominator に含める対象:

- Rust production source under `core/`, `drivers/`, `entrypoints/`, `regulated/`
- TypeScript SDK production source
- Swift SDK production source
- Android / Kotlin SDK production source after coverage task configuration

denominator から除外する対象:

- Rust `tests/` and `integration-tests/` packages
- SDK test directories
- benchmark runner and benchmark support code
- fixtures
- generated build output
- `target/`, `.build/`, Gradle build output, V8 raw output

denominator を変更する場合、coverage report は changed denominator と previous denominator の差分を記録することが必須です。overall line coverage は project-level indicator であり、各 tool が instrumented / coverable line として報告した covered / total を合算して算出します。この weighted aggregate は、言語間の statement model、branch model、macro / generated code handling、method/function instrumentation が同一であることを証明しません。

### Coverage report の field

coverage report は次を記録することが必須です。

| Field | Required content |
|---|---|
| correlation ID | coverage run と report を結び付ける ID |
| command | language / target 別の coverage command |
| working directory | command 実行ディレクトリ |
| toolchain | coverage tool and compiler/runtime version |
| denominator scope | included / excluded production source paths |
| exclusion reason | closed reason |
| core line coverage | covered / instrumented / percentage |
| overall line coverage | covered / instrumented / percentage |
| core crate floor table | each core crate covered / instrumented / percentage |
| threshold judgement | pass/fail per threshold |
| assertion-bearing classification | coverage が assertion-bearing であることの分類 |
| diagnostic-only exclusion | import-only / smoke-only / text-inspection-only coverage を採用しない根拠 |
| rerun condition | 同一条件で再実行できる手順 |

overall coverage を先に満たしても、core line coverage と core crate floor を満たさない限り採用しません。

### Assertion-Bearing Test Rule

coverage improvement test は、少なくとも次のいずれかを assert することが必須です。

- accepted / admitted / allocated / approved / converted の正常系
- reject / deny / violation / collapse reason の異常系
- driver concrete type が core signature に入らない境界
- core-owned reference だけが cross-plane に通る境界
- reason / command / lifecycle / evidence class の closed vocabulary
- before / during / after / rollback / recovery の state transition
- Signaling / SFU / TURN / transport binding の不整合拒否

production source を import しただけの coverage、CLI や Kernel executable entrypoint を起動しただけの coverage、source text を検査しただけの coverage は diagnostic-only とし、coverage threshold の達成根拠として採用してはなりません。

### Non-Claim Boundary

この gate が成立しても、branch coverage sufficiency、path coverage sufficiency、function / method coverage sufficiency、mutation score sufficiency、fuzzing coverage sufficiency、property-test completeness、protocol conformance completeness、security readiness、runtime readiness、production readiness、live readiness、native SDK runtime readiness、native application command success は成立しません。Swift / Android command result は coverage gate の入力証跡としてのみ採用でき、native SDK runtime readiness、native entrypoint readiness、一般的な native command success claim へ転用してはなりません。

### Fail-Closed Conditions

次のいずれかに該当する場合、code coverage gate は fail-closed です。

- core line coverage が `90.00%` 未満。
- overall line coverage が `85.00%` 未満。
- いずれかの core crate が `80.00%` 未満。
- denominator scope が記録されていない。
- excluded paths が closed reason で分類されていない。
- import-only / smoke-only / text-inspection-only coverage を assertion-bearing coverage として採用している。
- line coverage pass を branch / path / function / mutation / fuzzing / property-test / protocol-conformance sufficiency として扱っている。
- Swift / Android coverage-related command result を native readiness / native command success として採用している。
- command、working directory、toolchain、rerun condition がない。

## Evidence record の field（test evidence）

test result を採用できるのは、record が次を含む場合に限ります。

- correlation ID
- test command
- working directory
- target package/module
- source set scope when source-shape evidence is used
- input fixture or scenario
- fixture/scenario class and source
- canonical serialization format/version when golden or digest evidence is used
- SDK projection class when SDK public API contract is tested
- media negotiation class when codec/track/layer behavior is tested
- internal control-plane class when service-to-service behavior is tested
- ICE candidate/connectivity class when candidate policy, restart, connectivity, or consent is tested
- secure media session class when DTLS/SRTP behavior is tested
- operator/admin authorization class when privileged action is tested
- out-of-scope feature class when excluded/admitted feature behavior is tested
- public endpoint/connection lifecycle class when endpoint admission or lifecycle behavior is tested
- export/backup artifact class, redaction, retention, and integrity class when artifact behavior is tested
- release artifact class, provenance class, and distribution channel when release behavior is tested
- time synchronization trust class, node scope, skew policy, precision, and measurement window when timestamp trust is tested
- edge/proxy class, trusted metadata class, trusted upstream scope, and header precedence when ingress trust is tested
- runtime reconfiguration class, target surface, generation references, apply scope, and rollback/drain class when reconfiguration is tested
- packet rewrite/media transform class, copy allowance, execution owner, and transform admission status when rewrite/transform behavior is tested
- service discovery source, endpoint scope, resolution state, staleness, and fallback class when endpoint resolution is tested
- distributed state class, state family, owner node/scope, affinity key, conflict rule, and failover class when distributed state behavior is tested
- runtime task/worker class, supervision scope, owning layer, join/cancel bound, and task outcome when task lifecycle behavior is tested
- internal service identity/trust class, source/target service, credential/peer proof class, scope, and trust policy reference when service identity behavior is tested
- cross-plane binding class, source/target plane, source/target references, lifecycle state, and authorization context relation when binding behavior is tested
- coverage threshold judgement, denominator scope, and core crate floor table when code coverage is tested
- expected outcome
- actual outcome
- reason category/code for failure where applicable
- environment/toolchain sufficient for rerun

これらの field を欠く test output は diagnostic output のみです。

## Native SDK Command Evidence Rule

Android native SDK command evidence は source marker test ではありません。`sdk/android` から Android SDK Gradle wrapper command を実行し、同一の記録 command surface に次の task を含めることが必須です。

- `:sdk:testDebugUnitTest`
- `:sdk:assembleDebug`
- `:sdk:lintDebug`
- `--warning-mode all`

record は、正確な `Command:` line、正確な `Working directory:`、`BUILD SUCCESSFUL`、`Warning/deprecation output: none observed` を記録することが必須です。Android lint report に warning / error が含まれてはなりません。

iOS native SDK command evidence は `sdk/ios` から `swift test` を実行することが必須であり、source marker assertion で代替してはなりません。TypeScript SDK test は `sdk/typescript` 内に留めることが必須です。Cross-platform Android/iOS projection check は native test または cross-platform governance test が所有し、TypeScript package test 内に重複させてはなりません。coverage 収集中に取得した Swift / Android command success は native command success と等価ではありません。native command success には専用 command evidence field が必須です。

## Mock / Fake Rule と Test Double 境界

mock、fake、deterministic clock、deterministic RNG、in-memory persistence、simulated network は focused test で許可されます。test double として label することが必須で、production / runtime evidence として使ってはなりません。fake driver を使う場合、test はその fake で core contract または composition path を証明し、concrete external implementation を証明しません。

### Test Double 境界

| Test double surface | Owner | Rule |
|---|---|---|
| core state/domain semantics | core | fake が変更してはならない |
| core port contract | core | fake は contract を満たす実装に限定 |
| fake driver behavior | testing support | deterministic execution only |
| deterministic clock/RNG | testing support | test evidence class only |
| captured observation | testing support | assertions for contract behavior |
| production driver | drivers | fake behavior を継承しない |

Fake driver はテスト用 driver 実装であり、別系統の domain authority ではありません。

### Test Double class

v0.2 initial architecture の test double class は次に限定します。

| Class | Allowed use | Prohibited use |
|---|---|---|
| `fake_port_driver` | contract tests for core port usage | production runtime |
| `stub_external_service` | deterministic external response in tests | domain policy definition |
| `spy_sink` | capture audit/metrics calls | audit meaning ownership |
| `deterministic_clock` | expiry/deadline tests | production evidence |
| `deterministic_rng` | ID/correlation reproducibility tests | security/runtime evidence |
| `fault_injection_driver` | failure mapping tests | default production behavior |

### Test Double の Contract / Evidence 規則

すべての fake driver は次を declare することが必須です。port implemented、covered call shapes、supported success outcomes、supported failure reasons、bounded behavior、deterministic behavior source、unsupported behavior handling、evidence class where it may be used。unsupported behavior は test 内で contract violation として fail closed することが必須であり、silently succeed してはなりません。

fake driver を使う test evidence は次を記録することが必須です。fake driver class、deterministic clock/RNG use、port contract covered、real driver behavior not claimed。fake-based unit/contract test は runtime proof ではありません。fault injection success は real failure recovery proof ではありません。

## Fixture / Scenario Data 境界

input fixture、generated scenario、captured packet metadata、benchmark data、golden sample は、synthetic / captured-redacted / generated / derived from v0.1 / manually authored のいずれかを declare することが必須です。fixture evidence は raw secret、raw token、raw packet payload、unredacted SDP/ICE material、regulated payload、personal data を含んではなりません。

### Fixture 境界

| Data surface | Owner | Rule |
|---|---|---|
| core semantic fixture | testing support | core-owned type expectations only |
| driver wire fixture | driver test support | external encoding sample, not core API |
| golden file | testing support | canonical format/version required |
| captured packet/sample | driver/testing support | raw sensitive payload prohibited unless redacted/synthetic |
| regulated scenario data | regulated testing support | generic core fixture に混入しない |

fixture は test input であり、それ単独では production evidence ではありません。

### Fixture class

v0.2 initial architecture の fixture class は次に限定します。

| Class | Meaning | Rule |
|---|---|---|
| `synthetic_core_fixture` | hand-authored core semantic input | safe for unit/contract tests |
| `driver_wire_fixture` | external encoding sample | driver conversion tests only |
| `golden_canonical_fixture` | deterministic canonical encoding fixture | must cite canonical serialization version |
| `fault_injection_fixture` | failure scenario input | must map expected reason |
| `redacted_capture_fixture` | captured material after redaction | raw source not stored in repo |
| `regulated_fixture` | regulated support scenario | not generic core fixture |

### Redaction / Ownership 規則と Golden Fixture

fixture は次を declare することが必須です。fixture class、owner layer、source、synthetic/captured/redacted status、sensitive data handling、canonical serialization version when applicable、expected reason/outcome、allowed evidence class。raw token、raw credential、raw key、raw RTP/RTCP/media payload、patient/user profile、regulated payload を generic fixture として保存してはなりません。

golden fixture が valid となるのは次の場合のみです。canonical format/version is declared、field set and ordering are deterministic、unknown field handling is explicit、redaction status is recorded、update procedure is documented、test command and expected digest/outcome are recorded。golden fixture drift は、compatibility 規則で評価されるまでは test/evidence issue であり、自動的な protocol breakage ではありません。

### Fixture Failure Mapping

| Failure | Required reason |
|---|---|
| fixture shape invalid | `fixture_invalid` |
| fixture requires redaction before use | `fixture_redaction_required` |
| canonical fixture mismatch | `canonical_serialization_mismatch` |
| driver wire fixture cannot decode | `external_decode_failed` |
| expected reason absent | `missing_required_wire_field` or fixture validation failure |

fixture/scenario data を使う evidence は次を記録することが必須です。fixture class、fixture path or source reference、redaction status、expected outcome/reason、actual outcome/reason、update policy。redaction statement のない captured data は evidence として採用できません。

## Negative Test Rule

各 boundary に対し、実行可能な範囲で negative case を含めるべきです。例: external concrete type cannot enter core signature、driver conversion failure does not call core、unsupported version fails closed、missing correlation ID fails closed、resource bound maps to closed reason、startup config failure is not treated as successful wiring、invalid fixture/scenario data is rejected before being accepted as evidence、canonical serialization mismatch is rejected as evidence、SDK public API projection drift is detected、media negotiation rejects unsupported mapping、internal control-plane rejects missing version/correlation/authorization、ICE candidate policy rejects disallowed exposure or restart、secure media session rejects missing peer verification、operator/admin action rejects missing authorization、out-of-scope feature request is rejected with cataloged reason、public endpoint rejects unadmitted class、export artifact rejects missing redaction、release artifact rejects missing provenance、time synchronization rejects skew exceeding policy、edge/proxy trust rejects unadmitted forwarded headers、runtime reconfiguration rejects unadmitted hot-swap、packet rewrite/media transform rejects unadmitted transform、service discovery rejects stale endpoint、distributed state/failover rejects split-brain risk、runtime task lifecycle rejects detached task、internal service trust rejects untrusted peer proof、cross-plane binding rejects implicit/missing binding。

## v0.1 Evidence Rule

v0.1 tests、integration tests、benches は evidence-only historical input です。v0.2 command / boundary / report で rerun または requalify されるまで、v0.2 behavior を証明しません。

## Evidence record template の全 field

evidence record は次の全 field を含むことが必須です。

- title
- report date/time in JST
- correlation ID
- target scope
- target classification
- time phase
- command or procedure
- working directory
- input files/packages/modules
- environment/toolchain when relevant
- expected outcome
- actual outcome
- closed reason classification for failure or skip
- evidence class
- sensitive data handling / redaction statement when source material can contain secrets, packet payload, SDP/ICE material, regulated payload, or personal data
- profile class when configuration profile affects the run
- test double class when fake/stub/mock/deterministic support affects the run
- fixture/scenario class when fixture or generated scenario data affects the run
- canonical serialization format/version when digest, hash-chain, golden, or compatibility evidence is used
- normalized unit, window, precision, and aggregation when measurement evidence is used
- atomicity class and commit boundary when partial success or compensation can affect the claim
- health/readiness/liveness/admin/maintenance class when operational probe or admin evidence is used
- process lifecycle class when crash, panic, unclean shutdown, or supervisor restart affects the claim
- SDK reconnect/session resumption class when client reconnect evidence is used
- deployment topology class and node scope when topology affects the run
- media negotiation class when codec/track/layer behavior affects the claim
- observability signal class, cardinality, and sampling when telemetry affects the claim
- supply-chain dependency class, license class, lockfile, vulnerability, and toolchain state when dependency evidence is used
- SDK public API projection class when SDK contract evidence is used
- secret rotation state when rotated credential or key material affects the claim
- internal control-plane class when service-to-service control affects the claim
- ICE candidate/connectivity class when candidate policy, restart, connectivity, or consent affects the claim
- secure media session class when DTLS/SRTP protected media path affects the claim
- operator/admin authorization class when privileged action affects the claim
- out-of-scope feature class when excluded or future-admitted feature behavior affects the claim
- public endpoint/connection lifecycle class when endpoint exposure or connection state affects the claim
- export/backup artifact class, redaction class, retention class, and integrity class when artifact evidence is used
- release artifact class, provenance class, digest/signature class, and distribution channel when release/distribution evidence is used
- time synchronization trust class, node scope, skew policy, precision, and measurement window when timestamp trust affects the claim
- edge/proxy class, trusted metadata class, trusted upstream scope, header precedence, and TLS termination relation when ingress trust affects the claim
- runtime reconfiguration class, target surface, current/proposed generation, apply scope, drain/restart class, and rollback class when runtime configuration change affects the claim
- packet rewrite/media transform class, copy allowance, execution owner, and transform admission status when packet byte mutation affects the claim
- service discovery source, endpoint scope, resolution state, TTL/cache/staleness, and fallback class when endpoint resolution affects the claim
- distributed state class, state family, owner node/scope, affinity key, replication/consensus admission, and failover class when multi-node or failover behavior affects the claim
- runtime task/worker class, supervision scope, owning layer, join/cancel bound, and task outcome when worker execution affects the claim
- internal service identity/trust class, source/target service, credential/peer proof class, accepted scope, and trust policy reference when service-to-service identity affects the claim
- cross-plane binding class, source/target plane, source/target references, lifecycle state, and authorization context relation when a claim crosses Signaling/SFU/TURN/ICE/secure media planes
- rerun condition

### Evidence class

evidence class は省略してはなりません。

| Evidence class | Meaning |
|---|---|
| source-shape | file structure, dependency direction, static scan |
| build | build command result only |
| test | unit/contract/integration test result |
| runtime-in-test | controlled runtime test result |
| benchmark | measured performance/capacity result |
| live/operational | real environment observation |

### Skip / Failure Reason 規則

skipped / failed / timed-out / unavailable evidence は closed reason classification を用いることが必須です。`UNKNOWN` は許可された report reason ではありません。

| Reason class | Meaning |
|---|---|
| `not_in_scope` | outside current target |
| `not_yet_scaffolded` | package/command target does not exist yet |
| `dependency_missing` | required local dependency/tool unavailable |
| `command_failed` | command ran and failed |
| `command_timed_out` | command exceeded bounded time |
| `environment_unavailable` | required runtime/live environment unavailable |
| `evidence_incomplete` | output lacks required fields |
| `superseded_by_newer_report` | replaced by newer report |

build/test/runtime が未実行の場合、evidence record はそれを明記することが必須です。unverified evidence を verified claim へ変換してはなりません。

## Readiness Claim 境界

code coverage、benchmark value、CI pass、implementation checklist は、それ単独では readiness proof になりません。production readiness と live readiness は別の claim kind であり、それぞれ独自の field-complete evidence を要します。

- `production_readiness` は field-complete release provenance evidence、field-complete supply-chain gate evidence、field-complete production profile evidence、field-complete operational probe evidence を要します。
- `live_readiness` は production-readiness evidence のすべてに加え、field-complete live endpoint evidence、field-complete public network traversal evidence、field-complete monitoring / rollback / shutdown-drain evidence を要します。
- `native_android_command_success` は observed success marker を持つ rerunnable Android native command report を要します。source-level SDK contract evidence や CI pass で代替してはなりません。
- `native_ios_command_success` は observed success marker を持つ rerunnable iOS native command report を要します。source-level SDK contract evidence や CI pass で代替してはなりません。

field は集合として採用され、いずれか 1 つでも欠ける場合 readiness claim は fail-closed です。release provenance report と supply-chain report は同一 claim evidence に接続されていることが必須で、package/module、lockfile、toolchain、correlation のいずれかが一致しない場合 production readiness claim は fail-closed です。native command success report は success marker だけでは採用できず、correlation ID、command、working directory、evidence class、closed reason none、rerun condition、source-level replacement marker absence を確認できない場合 fail-closed です。

## 禁止事項（章全体の集約）

- passing unit tests を integration/runtime readiness claim にする。
- integration smoke を dependency boundary compliance proof にする。
- mocks を real driver behavior として記述する。
- v0.1 test success を v0.2 evidence として継承する。
- test failure reason を free-text のみで記録する。
- unrerunnable local output を evidence として採用する。
- fixture/scenario class を隠す。
- golden fixture formatting を explicit rule なく canonical serialization として扱う。
- fake driver が domain acceptance/rejection semantics を所有する。
- fake driver が unsupported operation に対し success を返す。
- deterministic RNG/clock を production security evidence として使う。
- test pass with fake を live/runtime behavior として報告する。
- raw sensitive production data を generic fixture として commit する。
- driver wire fixture を core API にする。
- import-only / smoke-only / text-inspection-only coverage を assertion-bearing coverage として採用する。
- line coverage を branch/path/function sufficiency、mutation score、fuzzing coverage、property-test completeness、protocol conformance completeness、security/runtime/production/live readiness、native application command success の証明に使う。
- benchmark value を correctness / runtime readiness / production readiness proof にする。
- `pnpm` rule を bypass した JS/TS evidence を採用する。

## Collapse Conditions（章全体の集約）

evidence class が明示されない、mock/fake boundary が隠される、fake driver behavior が concrete driver proof として扱われる、failure reason が closed vocabulary に mapping できない、runtime behavior が static source evidence から claim される、test record が correlation ID または rerun command を欠く、fixture/scenario data が allowed class と redaction rule に traceable でない、code coverage evidence が denominator scope / threshold judgement / assertion-bearing classification / core crate floor を欠く、test evidence が implementation source condition に混入する、production readiness / live readiness / native command success を field-complete evidence が欠落したまま採用する。これらのいずれかが生じた場合、本章の判断は崩れます。
