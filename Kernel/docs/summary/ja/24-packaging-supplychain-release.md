# packaging / supply-chain / release: dependency・license・toolchain gate と release artifact・distribution・provenance

状態: public summary projection
日付: 2026-07-06 JST

## 目的

本章は arcRTC v0.2 Kernel における supply chain と release の境界を、他文書・実コードを参照せずに完全自己完結で規定します。対象は次の2領域です: supply chain / dependency / license / toolchain gate、release artifact / distribution / provenance。本章単独で再現実装が可能な粒度を与えます。

crate / package 境界の詳細は第02章が所有します。本章はそれを前提に supply chain と release を内在化します。crate / npm / Gradle / SwiftPM / toolchain dependency が architecture boundary、security posture、evidence claim を破らないよう、また生成された binary / crate / package / image / SDK package / docs bundle が配布可能成果物として扱われる条件を、owner と fail-closed 条件として固定します。

本章は dependency install、SBOM 生成、license audit、vulnerability scan、release 実行、publish 成功を主張しません。これらは evidence command の class を明示した上で別の report が扱います。

---

## A 部 Supply Chain / Dependency / License / Toolchain Gate

### A-1 境界

| 関心事 | Owner | Rule |
|---|---|---|
| architecture dependency direction | 第02章 | core/drivers/entrypoints/sdk/regulated boundary |
| package dependency admission | package/scaffold governance | dependency class と owner layer が必須 |
| third-party implementation dependency | package owner | layer dependency を逆転させてはならない |
| license policy | project governance | allow/deny/review 分類 |
| vulnerability policy | CI/security governance | adoption claim 前に scan evidence が必須 |
| toolchain version | scaffold/CI governance | pinned または明示的 range |
| generated artifacts | build tooling | admit されない限り source authority ではない |

workspace membership または package manager success は architecture permission を含意しません。

### A-2 Dependency Classes（閉集合）

v0.2 initial architecture の dependency class は次に限定します。

| Class | 意味 | Rule |
|---|---|---|
| `core_semantic_dependency` | core semantic package が使う dependency | driver/runtime concrete dependency なし |
| `driver_implementation_dependency` | concrete I/O/runtime/parser/backend dependency | driver package のみ |
| `entrypoint_composition_dependency` | executable wiring/config dependency | entrypoints package のみ |
| `sdk_public_dependency` | SDK platform public または implementation dependency | regulated/core internals の leakage なし |
| `regulated_optional_dependency` | regulated support dependency | generic core requirement になってはならない |
| `test_tool_dependency` | test/build/fixture support | production dependency ではない |
| `build_tool_dependency` | formatter/codegen/build helper | evidence class が必須 |

新 dependency class は v0.2 初期 scope 外です。

### A-3 Admission Rule（dependency 採用時の必須記録）

dependency admission は次を record しなければなりません（必須）。

- package manager と package name;
- owner layer;
- dependency class;
- direct/transitive status;
- license class;
- security-sensitive のときの vulnerability scan status;
- lockfile impact;
- architecture dependency impact;
- experimental のときの replacement/removal condition。

dependency は、その boundary と owner が record されない限り、implementation evidence として採用してはなりません（禁止）。

### A-4 License and Vulnerability Rule

license と vulnerability evidence は build/test evidence と separate です。

- build success は license acceptance を証明しません。
- license acceptance は vulnerability clearance を証明しません。
- vulnerability scan pass は runtime security を証明しません。

license または vulnerability state が adoption を block する場合、影響を受ける dependency path は release/readiness claim に対し fail-closed しなければなりません。dependency gate success は release artifact provenance への input であって、それ自体は release artifact ではありません。

### A-5 Failure Mapping（閉集合 reason）

| 失敗 | 必須 reason |
|---|---|
| dependency が architecture policy に違反 | `dependency_policy_violation` |
| dependency license が accepted されていない | `license_policy_violation` |
| vulnerability gate が adoption を block | `vulnerability_gate_failed` |
| toolchain version が policy に不一致 | `toolchain_version_mismatch` |
| lockfile drift を検出 | `lockfile_drift_detected` |
| evidence command に required dependency/tool が欠落 | report reason として `dependency_missing` |

### A-6 Evidence Rule / Audit Rule（supply chain）

supply-chain evidence は次を record しなければなりません（必須）: package manager、command、working directory、lockfile state、dependency class、license result、用いた場合の vulnerability scan class、close-not-claimed scope。これらの field を欠く package install output は diagnostic のみです。

supply-chain decision は audit event type `supply_chain_decision` を用います。当該 event は package/toolchain reference、dependency class、および evidence または gate run の `CorrelationId` を carry しなければなりません（必須）。

### A-7 Prohibitions（supply chain）

- core package が feature flag を通じて driver/runtime concrete dependency を import する。
- package manager install success が dependency policy acceptance として扱われる。
- generated/cache/local artifact が source authority になる。
- release/readiness claim に対し license/vulnerability result が omit される。
- project policy が pnpm を要求する JS/TS evidence で npm/yarn が使われる。
- test helper が hidden production dependency になる。
- dependency gate output が artifact class、source ref、command、digest なしに release artifact provenance として扱われる。

### A-8 A 部 Collapse Conditions

- dependency class が absent。
- license または vulnerability gate が silently bypass され得る。
- reproducibility を主張しながら lockfile drift が ignore される。
- package dependency direction が architecture boundary に違反。
- toolchain evidence が version と working directory を欠く。
- release/distribution claim が artifact provenance classification を bypass する。

---

## B 部 Release Artifact / Distribution / Provenance

### B-1 境界

release artifact は source tree から build / package / sign / verify / publish される成果物です。release artifact は source authority ではなく、生成元 source ref、toolchain、dependency gate、test/evidence、provenance に接続された派生物です。

| 関心事 | Owner | Rule |
|---|---|---|
| source authority | repository / core 契約 | release artifact は source を置換しない |
| package boundary | crate/package 契約（第02章） | package ownership と layer direction が必須 |
| dependency / license / vulnerability | supply-chain（A 部） | release claim は gate を bypass できない |
| build and test evidence | CI/testing policy | command evidence class を明示すること |
| artifact provenance | release governance | source ref, toolchain, command, digest が必須 |
| distribution channel | release governance / entrypoints / SDK owner | channel は admit されること |

### B-2 Release Artifact Classes（閉集合）

v0.2 initial architecture の release artifact class は次に限定します。

| Artifact class | 意味 |
|---|---|
| `rust_crate_package` | internal または external distribution 向け Rust crate package |
| `kernel_contract_entrypoint_binary` | Kernel executable contract / composition evidence entrypoint artifact |
| `container_image` | containerized Kernel entrypoint artifact |
| `typescript_sdk_package` | pnpm evidence で build される pnpm/npm-distributable SDK package |
| `android_sdk_package` | Android SDK package artifact |
| `ios_sdk_package` | iOS/Swift package artifact |
| `documentation_bundle` | release distribution 向け docs bundle |
| `ci_evidence_bundle` | release-support evidence bundle |

新 release artifact class は v0.2 初期 scope 外です。

### B-3 Provenance Rule（release claim ごとの必須記録）

すべての release artifact claim は次を record しなければなりません（必須）。

- artifact class;
- source ref または immutable source snapshot;
- package/module name;
- build command と working directory;
- toolchain version;
- dependency/lockfile state;
- 該当する場合の license と vulnerability gate state;
- claim に採用した test/evidence reports;
- artifact digest;
- signature または明示的 unsigned class;
- distribution channel;
- rollback/removal condition;
- correlation ID。

source ref と command を欠く artifact digest は provenance ではありません。dependency/license/vulnerability state を欠く build success は release readiness ではありません。

### B-4 Distribution Channel Rule（閉集合 channel）

distribution channel は次の initial vocabulary に closed です。

| Channel | Rule |
|---|---|
| `local_artifact_only` | local build output、public release ではない |
| `internal_registry` | private registry または artifact store |
| `public_registry` | public package/container registry |
| `github_release` | GitHub release asset |
| `documentation_site` | published docs site または docs artifact host |

新 channel は v0.2 初期 scope 外です。public distribution は explicit public channel admission、provenance、redaction review、supply-chain gate evidence を要します。

### B-5 Release Claim Rule

release artifact evidence は、その report が述べる scope のみを support し得ます。別 evidence が無い限り、production readiness、live operational readiness、security certification、protocol compatibility、SDK parity、regulated workflow support を含意してはなりません（禁止）。

### B-6 Failure Mapping（閉集合 reason）

| 失敗 | 必須 reason |
|---|---|
| claim した class の release artifact が build されていない | `release_artifact_not_built` |
| provenance field が欠落 | `release_artifact_provenance_missing` |
| artifact digest/signature verification が fail | `release_artifact_integrity_failed` |
| source/package/version が claim した release に不一致 | `release_version_mismatch` |
| distribution channel が admit されていない | `distribution_channel_not_allowed` |

### B-7 Evidence Rule / Audit Rule（release）

release evidence は次を record しなければなりません（必須）: artifact class、source ref、package/module、build command、working directory、toolchain、dependency/license/vulnerability state、digest/signature class、distribution channel、採用した tests/reports、rerun condition。package manager または registry output 単独では diagnostic のみです。

release artifact decision は audit event type `release_artifact_distribution_decision` を用います。当該 event は artifact class、source ref、package/module reference、artifact digest reference、distribution channel、provenance class、`CorrelationId` を carry しなければなりません（必須）。

### B-8 Prohibitions（release）

- generated release artifact が source authority になる。
- build output が dependency/license/vulnerability clearance として扱われる。
- local artifact が public distribution として記述される。
- admitted distribution channel なしに public registry publish が試みられる。
- unsigned artifact が signed として記述される。
- release artifact evidence が runtime/live readiness proof として用いられる。
- SDK package release が server semantic correctness の主張に用いられる。

### B-9 B 部 Collapse Conditions

- artifact class が absent。
- source ref、command、toolchain、digest のいずれかが provenance から absent。
- distribution channel が open-ended。
- release claim に対し supply-chain gate が bypass され得る。
- generated artifact が canonical source として扱われる。

---

## C 部 不変条件（Invariants）の要約

- crate / package 境界の詳細は第02章が所有する。本章は supply chain と release を内在化する。
- workspace membership または package manager / build / install success は architecture permission でも dependency policy acceptance でもない。
- dependency は dependency class と owner layer が record されない限り implementation evidence として採用されず、architecture dependency direction を逆転させない。
- license、vulnerability、build/test evidence は相互に独立であり、いずれかの pass が他を証明しない。block 時は影響 path が release/readiness claim に対し fail-closed。
- release artifact は source authority ではなく派生物であり、artifact class / source ref / build command / working directory / toolchain / dependency-license-vulnerability state / digest / signature-or-unsigned-class / distribution channel / correlation ID を欠く provenance は不成立。
- distribution channel は閉集合（`local_artifact_only` / `internal_registry` / `public_registry` / `github_release` / `documentation_site`）であり、public distribution は explicit admission と provenance と redaction review と supply-chain gate evidence を要する。
- release/supply-chain evidence は report の scope のみを support し、production / live / security certification / protocol compatibility / SDK parity / regulated workflow を別 evidence なしに含意しない。
