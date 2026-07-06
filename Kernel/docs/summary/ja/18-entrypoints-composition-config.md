# entrypoints の composition root と configuration 境界

状態: public summary projection
日付: 2026-07-06 JST

## 目的

本章は arcRTC v0.2 Kernel の `entrypoints/` における composition root（dependency injection / wiring）境界、core が所有する configuration boundary 型、configuration profile / policy bundle、runtime reconfiguration / policy hot-swap の全手順と禁止事項を、他文書・実コードを参照せずに完全自己完結で規定します。本章単独で再現実装が可能な粒度を与えます。

entrypoints は Kernel 内の executable contract、CLI、demo、dependency wiring、composition evidence surface を所有しますが、domain rule、protocol semantics、port contract、SFU / TURN / Signaling の product distro を所有しません。

## 第1節 Entrypoint Set（閉集合）

v0.2 initial architecture の Kernel entrypoint set は次に限定します。これ以外の新 entrypoint は v0.2 初期 scope 外です。

| Entrypoint | 責務 |
|---|---|
| `entrypoints/signaling-server` | Signaling executable contract / composition evidence surface |
| `entrypoints/sfu-server` | SFU executable contract / composition evidence surface |
| `entrypoints/turn-server` | TURN executable contract / composition evidence surface |
| `entrypoints/cli` | operator / developer 向け CLI |
| `entrypoints/demo` | demo composition |
| `entrypoints/configuration` | configuration profile、policy bundle、feature/capability wiring |
| `entrypoints/endpoints` | public endpoint lifecycle と edge/proxy trust wiring |
| `entrypoints/topology` | deployment topology、service discovery、endpoint resolution wiring |
| `entrypoints/internal-control` | internal service identity と control-plane wiring |
| `entrypoints/admin` | health/readiness/liveness/admin/operator wiring |

## 第2節 Composition Rule

entrypoints は次の行為を行うことが許可されます。

- environment variables、files、process args、deployment settings の parse。
- typed configuration の construct。
- driver implementation の選択。
- core use case を driver implementation へ wire する。
- executable process の start / stop。
- 許可された境界を通じて core use case を呼ぶ CLI/demo command の expose。

entrypoints は、代替の domain decision、reason vocabulary、port trait、state transition を定義することが禁止です。
entrypoints は、reference distro、product distro、production readiness、live readiness の evidence として扱うことが禁止です。

## 第3節 Wiring Rule（依存方向）

entrypoints は次の方向にのみ dependency を wire します。依存方向の記法 `A <- B` は「B が A に依存」を意味します。

```text
entrypoints -> core
entrypoints -> drivers
drivers -> core
```

すなわち `core <- drivers`、`core <- entrypoints`、`drivers <- entrypoints` です。

- entrypoints は、core から entrypoints への依存、または drivers から entrypoints への依存を導入することが禁止です。
- entrypoints は、regulated boundary interaction が明示的に許可されない限り、`regulated` を generic communication path に wire することが禁止です。
- entrypoints は core と drivers に依存してよいが、domain rule を所有しません。SFU / TURN / Signaling product system を所有しません。`entrypoints -> regulated`（generic path）は禁止です。

## 第4節 Startup Fail-Closed Rule

startup / wiring は、required configuration、driver implementation、secret source、clock、RNG、runtime、persistence、audit sink、metrics sink のいずれかが initialize できないとき、fail-closed しなければなりません（必須）。

| 失敗 | 必須 reason |
|---|---|
| required runtime configuration missing | `runtime_config_missing` |
| runtime configuration が selected driver/entrypoint を initialize できない | `runtime_config_invalid` |
| required secret source unavailable | `secret_unavailable` |
| selected driver が bounded initialization 後も unavailable | 対応する driver failure reason |
| selected deployment topology unsupported | `deployment_topology_unsupported` |
| required dependency に対する service discovery unavailable | `service_discovery_unavailable` |
| service discovery fallback または endpoint scope が admit されない | `service_endpoint_fallback_not_allowed` または `service_endpoint_scope_conflict` |
| distributed state / replication / failover が admit されない | `distributed_state_not_admitted` / `state_replication_not_admitted` / `failover_not_proven` |
| runtime task supervision/spawn/join/cancel failure | runtime task reason |
| internal service identity/trust mapping failure | internal service identity reason |
| selected entrypoint/profile に対し public endpoint class が admit されない | `public_endpoint_not_allowed` |
| selected entrypoint/profile に対し runtime reconfiguration が admit されない | `runtime_reconfiguration_not_allowed` |

startup failure が通常の audit sink で記録できない場合は、bootstrap audit record path（予約された証跡経路）を適用します。いずれの path でも failure を記録できない場合、その startup attempt を closeout evidence として用いてはなりません（禁止）。

## 第5節 Configuration Ownership（core が所有する境界型）

configuration は core policy input と driver/entrypoint runtime input に分離します。

| Configuration kind | Owner | 例 |
|---|---|---|
| core policy configuration | core | thresholds, bounds, accepted versions |
| driver runtime configuration | drivers | socket address, TLS files, DB DSN, S3 bucket |
| entrypoint composition configuration | entrypoints | selected drivers, process options |
| SDK client configuration | sdk | signaling endpoint URL, reconnect behavior |
| regulated configuration | regulated | domain-specific mapping and enrichment |

### Core Configuration Rule
core は configuration を core が所有する typed policy としてのみ受け取ります。core は environment variables、files、process args、OS settings、cloud metadata を読むことが禁止です。

### Driver Configuration Rule
driver は external configuration を、entrypoints が wire した typed runtime configuration としてのみ受け取ります。driver は runtime settings から core policy を導出することが禁止です。driver-local runtime configuration は、external implementation initialization と I/O execution に限定します。driver-local initialization は environment variables、files、process args、deployment metadata を直接読むことが禁止です。

### Entrypoints Configuration Rule
entrypoints は environment variables、files、process args、deployment settings を読むことが許可されます。entrypoints は domain decision を所有しません。entrypoints は configuration を typed input として core/drivers に wire します。entrypoints は required policy または runtime input に対し silent な default 代入を行うことが禁止です。

### Policy Validation Rule
core は core policy configuration の解釈と validation を所有します。entrypoints は external input を typed configuration structure に parse してよいが、policy の acceptance / rejection は core に属します。driver は external implementation を initialize するための driver-local runtime settings を validate してよいが、invalid な core policy を accepted behavior に変えてはなりません。

## 第6節 Configuration Failure Rule（fail-closed）

configuration failure は fail-closed しなければなりません（必須）。

| 失敗 | reason code |
|---|---|
| typed core policy configuration が invalid | `core_policy_config_invalid` |
| required runtime configuration が missing | `runtime_config_missing` |
| runtime configuration が selected driver/entrypoint を initialize できない | `runtime_config_invalid` |
| required secret source が unavailable | `secret_unavailable` |

missing または invalid な required configuration は、startup を停止するか、影響を受ける command path を上記 reason で reject しなければなりません。implicit fallback、silent defaulting、reason 無き partial enablement は禁止です。configuration failure は、startup path 停止または影響 command path reject の前に、cataloged reason を伴う `configuration_decision` audit event を emit しなければなりません。failed configuration が selected audit sink の initialization を妨げる場合は、bootstrap audit record path で記録します。通常 sink も bootstrap path も記録できない場合、startup は停止し、その startup attempt を closeout evidence として用いてはなりません。startup configuration の acceptance は runtime reconfiguration を admit しません。

## 第7節 Feature Flag Rule

feature flag は core semantics を暗黙に変えることが禁止です。

許可（Allowed）:
- driver implementation の選択。
- optional exporter の enable。
- external encoding の選択。

禁止（Prohibited）:
- Signaling state machine の silent な変更。
- TURN lifecycle の silent な変更。
- SFU routing semantics の silent な変更。
- security verification の bypass。
- audit requirement の bypass。

## 第8節 Secret Rule

secrets は driver/entrypoint の関心事です。core は raw secrets を所有することが禁止です。core は opaque credential reference または verification result を所有してよいです。

## 第9節 Configuration Profile / Policy Bundle

configuration は複数の core policy、driver runtime config、entrypoint composition config を bundle として検証します。bundle composition は policy ownership を移動させてはなりません。

| Bundle | Owner | Rule |
|---|---|---|
| core policy bundle | core | thresholds, accepted versions, bounds, security requirements |
| driver runtime bundle | driver | socket, DB, exporter, TLS/key source references |
| entrypoint composition bundle | entrypoints | selected drivers and startup mode |
| deployment topology bundle | entrypoints | entrypoint/service topology class, service discovery, node affinity |
| service discovery bundle | entrypoints/drivers | discovery source, endpoint scope, TTL/cache, fallback |
| internal service trust bundle | entrypoints/drivers | trust class, source/target service, credential/peer proof reference, scope |
| distributed state bundle | entrypoints/deployment | state class, owner scope, affinity, failover/replication admission |
| runtime task bundle | entrypoints/drivers | task class, supervision scope, join/cancel bound |
| secret rotation bundle | driver | secret source references, accepted generations, overlap/revocation policy |
| supply-chain bundle | CI/release scope | dependency, license, vulnerability, lockfile, toolchain evidence |
| SDK client bundle | sdk | Signaling endpoint and client-local behavior |
| regulated mapping bundle | regulated | optional domain support, not generic core policy |
| test profile bundle | testing scope | deterministic and fake settings only for evidence class |

### Profile Classes（閉集合）

| Profile class | 意味 | Adoption rule |
|---|---|---|
| `development_local` | local manual run profile | not production evidence |
| `test_deterministic` | deterministic clock/RNG/fake driver profile | test evidence only |
| `integration_controlled` | controlled integration profile | integration evidence only |
| `benchmark_controlled` | benchmark profile | benchmark evidence only |
| `production_candidate` | candidate production-like profile | runtime 主張前に明示的 evidence report が必須 |

新 profile class は v0.2 初期 scope 外です。

### Bundle Validation Rule（順序）

startup/wiring は bundle を次の順序で validate しなければなりません（必須）。

1. entrypoint composition bundle が parseable かつ complete である。
2. selected driver runtime bundles が present かつ internally valid である。
3. core policy bundle が present かつ core により accepted である。
4. cross-bundle references が raw secret leakage なく resolve する。
5. feature/capability settings が feature lifecycle 規範により allowed である。
6. deployment topology class が explicit かつ accepted である。
7. discovery が configured のとき、service discovery source、endpoint scope、fallback policy が explicit である。
8. topology が node-local state を node 間で touch しうるとき、distributed state class と owner scope が explicit である。
9. service-to-service identity が profile に影響するとき、internal service trust class と accepted scope が explicit である。
10. worker execution が profile に影響するとき、runtime task class と supervision scope が explicit である。
11. 任意の credential verifier/key source が configured のとき、secret rotation policy source が present である。
12. build/release claims のとき、supply-chain/toolchain evidence class が declared である。
13. profile の evidence class が declared である。

partial bundle acceptance は、degraded mode とその close-not-claimed scope が定義されない限り禁止です。startup での profile validation は、reconfiguration class が admit しない限り runtime profile swap を authorize しません。

### Bundle Failure Mapping

| 失敗 | 必須 reason |
|---|---|
| required entrypoint/runtime bundle missing | `runtime_config_missing` |
| entrypoint/runtime bundle が selected driver/entrypoint を initialize できない | `runtime_config_invalid` |
| core policy bundle invalid | `core_policy_config_invalid` |
| required secret source unavailable | `secret_unavailable` |
| 必要箇所で secret rotation policy/state unavailable | `secret_rotation_state_unavailable` |
| selected deployment topology unsupported | `deployment_topology_unsupported` |
| service discovery source unavailable | `service_discovery_unavailable` |
| discovery source / endpoint scope / fallback policy invalid | `service_discovery_source_not_admitted` / `service_endpoint_scope_conflict` / `service_endpoint_fallback_not_allowed` |
| distributed state / replication / consensus / failover policy invalid | `distributed_state_not_admitted` / `state_replication_not_admitted` / `consensus_not_admitted` / `failover_not_proven` |
| internal service trust profile invalid または not admitted | internal service identity reason |
| runtime task profile invalid または not admitted | runtime task reason |
| toolchain または lockfile が accepted profile に不一致 | `toolchain_version_mismatch` または `lockfile_drift_detected` |
| dependency/license/vulnerability policy rejected | `dependency_policy_violation` / `license_policy_violation` / `vulnerability_gate_failed` |
| selected feature/capability disabled | `capability_not_enabled` |
| test-only profile を non-test evidence に使用 | evidence report rejection（runtime success ではない） |
| runtime profile swap not admitted | `runtime_reconfiguration_not_allowed` |

### Evidence Rule
evidence report は profile class と bundle sources を raw secret material なしで declare しなければなりません。`test_deterministic` 下の evidence を runtime/production evidence に用いてはなりません。ある profile class の evidence を、新 report なしで別の evidence class に promote してはなりません。

## 第10節 Runtime Reconfiguration / Policy Hot-Swap

第5〜9節は startup/wiring 時の configuration validation を扱い、本節は起動後に設定や policy を変えることを許可する条件、または禁止を固定します。runtime reconfiguration とは、process 起動後に core policy、driver runtime configuration、entrypoint composition、feature/capability、endpoint exposure、security material、topology、observability export などの有効設定を変更しようとする操作です。v0.2 では、明示的に許可された reconfiguration class 以外は startup-only として扱います。

### Reconfiguration Classes（閉集合）

| Reconfiguration class | 意味 | Rule |
|---|---|---|
| `startup_only` | change は restart と startup validation を要する | 全 settings の default |
| `secret_rotation_reload` | rotation 規範下で secret/key material を reload | general policy change ではない |
| `observability_export_reload` | taxonomy bounds 内で exporter sink/level を変更 | domain decisions に影響してはならない |
| `maintenance_mode_switch` | maintenance または drain mode の enter/exit | health/admin と shutdown 規範が適用 |
| `test_profile_swap` | deterministic/test-only profile swap | test evidence only |
| `runtime_policy_hotswap` | restart 無しで core policy generation を変更 | v0.2 初期 scope では禁止 |

新 reconfiguration class は v0.2 初期 scope 外です。

### Generation State Rule（閉集合）

runtime reconfiguration は次の closed generation states を用いなければなりません。

| State | 意味 |
|---|---|
| `current_generation` | declared scope に対し active かつ accepted |
| `pending_generation` | parsed だが not accepted |
| `validating_generation` | validation 中で not active |
| `rejected_generation` | validation 失敗、apply してはならない |
| `rollback_generation` | rollback に用いる prior accepted generation |
| `retired_generation` | もはや active でなく、新 decision に accept されない |

generation names は evidence reference であり、raw secret values や unredacted config payload ではありません。

### Admission Rule
runtime reconfiguration request は次を記録しなければなりません: reconfiguration class、target configuration surface、target owner、current generation reference、proposed generation reference、apply scope、affected active sessions/connections/allocations/routes、drain/restart requirement、rollback behavior、audit event type、evidence class、close-not-claimed scope。これらを供給できない request は apply 前に reject しなければなりません。

### Apply Rule
runtime reconfiguration は、既に accepted な domain decision を retroactively に mutate してはなりません。既存の Signaling rooms、SFU sessions/routes、TURN allocations/permissions、packet caches、idempotency windows、audit hash-chain scopes は、target 契約が migration / re-evaluation を明示しない限り、decision が accept された generation を保持します。setting が public endpoint exposure、security mode、topology、authorization、rate/quota、protocol compatibility に影響する場合、その exact class の hot-swap が v0.2 初期 scope で admit されない限り、startup/restart または controlled drain が必須です。

### Rollback Rule
rollback は automatic success ではありません。rollback は次を定義しなければなりません: rollback generation、rollback trigger、rollback apply scope、rollback evidence、affected in-flight operation handling、rollback failure 時の reason。rollback が required だが証明できない場合、target path を readiness または closeout evidence として用いてはなりません。

### Reconfiguration Failure Mapping

| 失敗 | 必須 reason |
|---|---|
| runtime reconfiguration class not admitted | `runtime_reconfiguration_not_allowed` |
| required configuration generation reference missing | `configuration_generation_missing` |
| proposed generation が active generation/order と conflict | `configuration_generation_conflict` |
| proposed generation が validation 失敗 | `runtime_reconfiguration_validation_failed` |
| target surface または active scope に対し apply not allowed | `runtime_reconfiguration_apply_not_allowed` |
| apply 前に drain/restart 必須 | `runtime_reconfiguration_drain_required` |
| evidence が受理される前に rollback 必須 | `runtime_reconfiguration_rollback_required` |
| rollback execution 失敗 | `runtime_reconfiguration_rollback_failed` |

### Audit Rule
runtime reconfiguration decisions は audit event type `runtime_reconfiguration_decision` を用います。event は reconfiguration class、target surface、current generation、proposed generation、apply scope、drain/restart class、該当時 rollback class、`StartupRunId`、command-scoped 時 `CorrelationId` を carry しなければなりません。

## 第11節 CLI / Demo Rule

CLI と demo は便利な flow を expose してよいが、代替の domain authority ではありません。CLI/demo commands は core が所有する command/use case 境界に map しなければなりません。Demo defaults は production policy になってはなりません。Admin と maintenance commands は entrypoints が wire してよいが、その decision semantics、evidence requirements、failure mapping は health/admin 規範下に留まります。privileged admin または maintenance action 実行前に operator/admin authorization を満たさなければなりません。CLI/demo は、core contract admission を欠く out-of-scope feature request を reject または close-not-claimed として mark しなければなりません。CLI/demo surface は固有の fail-closed composition guard を expose し、server 全体の entrypoint composition test が CLI/demo 固有境界の marker-only な代替にならないようにしなければなりません。CLI/demo path が core use case を bypass することは禁止です。

## 第12節 禁止事項（Prohibitions）

- entrypoints が port traits を定義する。
- entrypoints が Signaling join acceptance、SFU route selection、TURN permission decision を所有する。
- entrypoints が SFU / TURN / Signaling reference distro または product distro を所有する。
- entrypoints が core/drivers 契約境界外で retry/backpressure/resource policy を実装する。
- entrypoints が required policy / runtime configuration の default を silently 代入する。
- entrypoints が admitted task supervision 外で detached workers を spawn する。
- entrypoints が endpoint resolution または TLS listener success を internal service trust として扱う。
- CLI/demo path が core use case を bypass する。
- entrypoints が generic communication path で regulated support に依存する。
- entrypoint health/readiness probe が required dependency observation なしに domain state / readiness success を捏造する。
- supervisor restart handling を recovery policy なしに domain recovery として扱う。
- deployment topology が domain semantics または dependency direction を変える。
- service discovery fallback が policy なしに endpoint または owner scope を変える。
- entrypoints が multi-node wiring から replication / consensus / failover を含意する。
- entrypoints が endpoint admission record なしに public/internal endpoint class を expose する。
- entrypoints が provenance なしに packaged binary を release/distribution evidence として扱う。
- entrypoints が edge trust policy なしに edge/proxy metadata を信用する。
- entrypoints が reconfiguration generation なしに policy/profile を hot-swap する。
- entrypoint profile が core semantics を silently 変える。
- driver runtime config が core policy を導出する。
- missing required bundle が default に fall back する。
- test profile が production profile になる。
- profile evidence が profile class を省略する。
- runtime profile swap が successful startup validation から推論される。

## 第13節 Collapse Conditions（判断が崩れる条件）

- executable entrypoint が domain rule を所有する。
- executable entrypoint が product distro として扱われる。
- composition root が第二の reason catalog を定義する。
- entrypoints が core が所有する ports を bypass し driver internals を domain authority として呼ぶ。
- startup failure が隠蔽されつつ close / complete / ready を主張する。
- demo configuration が暗黙の production policy になる。
- entrypoint profile または feature flag が core semantics を silently 変える。
- entrypoint が所有する admin/maintenance/health/process lifecycle path が domain authority になる。
- single-process または local topology evidence が split-service / multi-node proof として用いられる。
- operator/admin authorization が CLI access または localhost から推論される。
- CLI/demo が out-of-scope feature を production scope に admit する。
- entrypoint listener wiring が endpoint lifecycle evidence なしに public endpoint readiness として扱われる。
- entrypoint reconfiguration path が target 契約定義なしに core semantics を変える。
- bundle ownership が policy ownership を変える。
- partial config acceptance が missing required policy を隠す。
- raw secret material が profile evidence に現れる。
- runtime reconfiguration class が absent。
- active/pending/rejected generation state が implicit。
- apply scope が記録されない。
- hot-swap が target 契約定義なしに core semantics を変えうる。
- rollback が rollback generation と evidence なしに主張される。

## 第14節 不変条件（Invariants）の要約

- 依存方向は `entrypoints -> core`、`entrypoints -> drivers`、`drivers -> core` のみ。逆方向と `entrypoints -> regulated`（generic path）は不変的に禁止。
- core は外部環境（env/file/args/OS/cloud metadata）を読まず、typed policy のみを受ける。
- 必須 configuration / bundle / secret / driver の欠落・無効は常に fail-closed し、cataloged reason を伴う audit を残す。
- startup-only が default であり、明示的に admit された reconfiguration class 以外は runtime での設定変更を行わない。
- 既に accepted な domain decision は新 generation により遡及的に再解釈されない。
- raw secret は core/audit/log/report/evidence に現れない（opaque reference のみ）。
