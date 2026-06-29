# 第07章 reference implementation design

状態: SSOT 統合版
日付: 2026-06-28 JST

## 目的

本章は、arcRTC v0.2 implementations 領域における reference implementation（Kernel frozen contract を利用する最小実装層）の構造、責務境界、各 plane（Signaling / TURN / SFU）の implementation design、composition design、runtime topology、transport 境界、configuration profile 境界、composition runtime bridge を、再現実装可能な粒度で固定します。本章は完全自己完結であり、他文書・実コードを参照せずに理解できます。同一仕様書内の他章番号のみ参照します。

reference implementation は production readiness を主張しません。reference implementation は Kernel frozen contract（contract / SDK / command surface）を使う最小実装層であり、依存方向は `implementations -> Kernel`（contract のみ）です。

---

## 1. reference implementation の所有面

reference implementation は次の surface を所有します。

| Surface | 所有内容 |
|---|---|
| `reference-implementation/output/` | reference output outcome 型。product input subset は 3-type allow-list（`ReferenceSignalingOutcome` / `ReferenceTurnOutcome` / `ReferenceSfuOutcome`） |
| `reference-implementation/signaling/` | Signaling reference wiring |
| `reference-implementation/turn/` | TURN reference wiring |
| `reference-implementation/sfu/` | SFU reference wiring |
| `reference-implementation/composition/` | Signaling / TURN / SFU reference composition |
| `reference-implementation/deployment-profiles/` | local / benchmark 向け profile |
| `reference-implementation/ops/` | reference execution helper |

reference implementation は次を**所有しません（MUST NOT own）**。

- product policy
- production deployment
- public distribution
- production readiness
- live readiness
- Kernel semantic authority
- Kernel contract modification

### 1.1 package 形状（file contract）

reference implementation package は workspace members に従います。各 package は次の必須ファイルを持ちます。

| Package | 必須ファイル |
|---|---|
| `arcrtc-reference-output` | `src/lib.rs`, `src/signaling.rs`, `src/turn.rs`, `src/sfu.rs`, `src/composition.rs`, `src/error.rs` |
| `arcrtc-reference-signaling` | `src/lib.rs`, `src/kernel_contract.rs`, `src/state.rs`, `src/fixture_identity.rs`, `src/local_auth.rs`, `src/error.rs` |
| `arcrtc-reference-turn` | `src/lib.rs`, `src/kernel_contract.rs`, `src/state.rs`, `src/fixture_credential.rs`, `src/error.rs` |
| `arcrtc-reference-sfu` | `src/lib.rs`, `src/kernel_contract.rs`, `src/state.rs`, `src/fixture_route_auth.rs`, `src/error.rs` |
| `arcrtc-reference-composition` | `src/lib.rs`, `src/step_input.rs`, `src/composition_state.rs`, `src/runtime_bridge.rs`, `src/error.rs` |
| `arcrtc-reference-ops` | `src/lib.rs`, `src/runtime.rs`, `src/evidence.rs`, `src/reason.rs`, `src/error.rs` |
| `reference-implementation/deployment-profiles` | `local.toml`, `benchmark.toml` |

profile files は Rust workspace member ではありません。

### 1.2 構築順序（required direction）

reference implementation は次の順に構築すること（MUST）。

1. reference output package
2. Signaling reference implementation
3. TURN reference implementation
4. SFU reference implementation
5. reference composition
6. local / benchmark profile
7. execution helper

### 1.3 evidence 境界

reference implementation test が示せるものは reference behavior と composition behavior に限定します。reference implementation test は production readiness、live readiness、native application readiness、public distribution readiness を示しません（MUST NOT）。

### 1.4 崩壊条件（collapse conditions）

次のいずれかが起きた場合、本章の規範は崩れます（fail-closed の是正対象）。

- reference implementation を product implementation と同一 claim として扱う。
- reference test pass を production readiness として採用する。
- reference implementation が product policy、monitoring、rollback を所有する。
- reference implementation が Kernel contract を変更する。
- reference implementation が database / queue / external storage を持つ。
- fixture identity を production credential として扱う。
- reference output outcome 型を `arcrtc-reference-output` 外で重複定義する。

---

## 2. Signaling reference implementation design

Signaling reference implementation は reference composition の最初の plane です。session / room / participant / control-plane command の接続面を提供しますが、Kernel core semantics を所有しません。所有面は `reference-implementation/signaling/` です。

### 2.1 所有するもの

| Owned item | 意味 |
|---|---|
| reference signaling runtime wrapper | Signaling command を受ける reference runtime surface |
| reference room/session mapping | reference 実行 profile の room/session identifier を Kernel input に変換する |
| reference participant mapping | reference 実行 profile の participant identifier を Kernel input に変換する |
| reference signaling command wrapper | join / leave / offer / answer / candidate 等の command surface |
| reference signaling observation mapper | command result / reason / audit pointer を evidence shape へ投影する |

### 2.2 所有しないもの（MUST NOT own）

- Kernel Signaling semantics
- Kernel reason catalog
- authentication issuance
- product tenant policy
- production deployment
- live endpoint operation
- media routing
- TURN relay allocation

### 2.3 依存規則

```text
reference-implementation/signaling -> Kernel public contract
reference-implementation/signaling -> Kernel SDK projection
reference-implementation/signaling -> implementations evidence shape

reference-implementation/signaling -X-> Kernel core source modification
reference-implementation/signaling -X-> product policy
```

### 2.4 input / output 境界（実行順）

Signaling reference implementation は外部入力を次の順で扱うこと（MUST）。

1. reference command input を受ける。
2. input を reference-local validation に通す。
3. Kernel contract input へ mapping する。
4. Kernel contract を呼び出す。
5. Kernel output / reason を reference evidence shape へ mapping する。

reference-local validation は command shape と required field の確認に限定します。accept / reject semantics は Kernel 側の判断として扱います。

### 2.5 evidence 境界

Signaling reference implementation test は次を示し得ます。

- reference Signaling command が Kernel contract に接続されていること
- mapping が決定的であること
- reason / output が evidence shape に投影されること

次を示しません（MUST NOT）: production readiness、live readiness、product authentication correctness、SFU routing correctness、TURN relay correctness。

### 2.6 崩壊条件

- Signaling reference implementation が Kernel Signaling semantics を再実装する。
- reference-local validation が Kernel accept / reject decision を置き換える。
- Signaling reference result を production readiness として扱う。
- Signaling reference implementation が product policy または live endpoint operation を所有する。

---

## 3. TURN reference implementation design

TURN reference implementation は allocation / permission / relay の reference runtime surface を構築します。network relay の実行面を扱いますが、Kernel TURN semantics を所有しません。所有面は `reference-implementation/turn/` です。

### 3.1 所有するもの

| Owned item | 意味 |
|---|---|
| reference allocation runtime wrapper | allocation command / lifecycle を reference runtime へ接続する |
| reference permission runtime wrapper | permission command / expiry を reference runtime へ接続する |
| reference relay runtime wrapper | relay packet flow を Kernel contract input/output に接続する |
| reference relay address mapper | reference profile の address / candidate を Kernel input に変換する |
| reference TURN observation mapper | allocation / permission / relay result を evidence shape へ投影する |

### 3.2 所有しないもの（MUST NOT own）

- Kernel TURN allocation semantics
- Kernel permission semantics
- Kernel relay semantics
- product network policy
- public deployment topology
- production relay capacity claim
- live endpoint operation

### 3.3 依存規則

```text
reference-implementation/turn -> Kernel public contract
reference-implementation/turn -> Kernel core-owned port definition
reference-implementation/turn -> implementations evidence shape

reference-implementation/turn -X-> Kernel core source modification
reference-implementation/turn -X-> product network policy
reference-implementation/turn -X-> production readiness claim
```

### 3.4 packet / relay 境界（実行順）

TURN reference implementation は packet / relay data を次の順で扱うこと（MUST）。

1. reference runtime が relay input を受ける。
2. runtime-specific input を Kernel-owned contract input へ mapping する。
3. Kernel contract が allocation / permission / relay decision を返す。
4. reference runtime が decision に従って relay action を実行する。
5. result / reason / metric を evidence shape に投影する。

reference runtime は relay execution を担当できます（MAY）。allocation / permission / relay acceptability の意味論は Kernel 側の decision として扱います。

### 3.5 evidence 境界

TURN reference implementation test は次を示し得ます。

- allocation / permission / relay runtime wrapper が Kernel contract に接続されていること
- relay input / output mapping が決定的であること
- Kernel reason が evidence shape に投影されること

次を示しません（MUST NOT）: production relay capacity、public NAT traversal success、live endpoint readiness、product network policy correctness、SFU media routing correctness。

### 3.6 崩壊条件

- TURN reference implementation が Kernel allocation / permission / relay semantics を再実装する。
- reference relay success を production relay capacity として扱う。
- product network policy を reference TURN layer に混入する。
- live endpoint operation を reference TURN test だけで主張する。

---

## 4. SFU reference implementation design

SFU reference implementation は media routing / forwarding / quality observation の reference runtime surface を構築します。packet forwarding の実行面を扱いますが、Kernel SFU routing semantics を所有しません。所有面は `reference-implementation/sfu/` です。

### 4.1 所有するもの

| Owned item | 意味 |
|---|---|
| reference media runtime wrapper | media packet / stream input を reference runtime に接続する |
| reference subscriber mapper | subscriber / publisher relation を Kernel input に変換する |
| reference routing action executor | Kernel routing decision に従い reference forwarding action を実行する |
| reference quality observation mapper | loss / jitter / bitrate / backpressure observation を evidence shape へ投影する |
| reference packet observation wrapper | packet metadata を Kernel contract input へ mapping する |

### 4.2 所有しないもの（MUST NOT own）

- Kernel SFU routing semantics
- Kernel quality decision semantics
- Kernel congestion / pacing semantics
- codec negotiation semantics
- product media policy
- production media capacity claim
- live media readiness

### 4.3 依存規則

```text
reference-implementation/sfu -> Kernel public contract
reference-implementation/sfu -> Kernel core-owned port definition
reference-implementation/sfu -> implementations evidence shape

reference-implementation/sfu -X-> Kernel core source modification
reference-implementation/sfu -X-> product media policy
reference-implementation/sfu -X-> live media readiness claim
```

### 4.4 packet 境界（実行順）

SFU reference implementation は packet / media input を次の順で扱うこと（MUST）。

1. reference media runtime が packet / stream observation を受ける。
2. runtime-specific metadata を Kernel-owned packet / routing input に mapping する。
3. Kernel contract が routing / quality / backpressure decision を返す。
4. reference runtime が decision に従って forwarding action を実行する。
5. result / reason / metric を evidence shape に投影する。

packet payload の所有権、buffer lifetime、zero-copy / copy policy は Kernel contract と driver/runtime boundary に従います。SFU reference implementation は packet semantics を再定義しません（MUST NOT）。

### 4.5 evidence 境界

SFU reference implementation test は次を示し得ます。

- packet / stream input が Kernel routing contract に接続されていること
- routing action executor が Kernel decision に従うこと
- quality observation が evidence shape に投影されること

次を示しません（MUST NOT）: production media capacity、live media readiness、codec compatibility completeness、product media policy correctness、public endpoint availability。

### 4.6 崩壊条件

- SFU reference implementation が Kernel routing / quality semantics を再実装する。
- reference forwarding success を live media readiness として扱う。
- product media policy を reference SFU layer に混入する。
- packet ownership / buffer lifetime を Kernel contract と無関係に定義する。

---

## 5. reference composition design

reference composition の価値は、3 plane（Signaling / TURN / SFU）を Kernel frozen contract に従って組み合わせ、benchmark / real-device / product 実装の基礎になる composition surface を作ることです。composition は product implementation ではなく、production readiness / live readiness を主張しません。所有面は `reference-implementation/composition/` です。

### 5.1 所有するもの

| Owned item | 意味 |
|---|---|
| plane wiring | Signaling / TURN / SFU reference implementations の接続 |
| identity/session bridge | reference profile の identity/session を plane 間で結合する |
| command flow orchestration | reference execution flow の command 順序を定義する |
| observation correlation | plane 間 evidence を correlation id で束ねる |
| benchmark profile binding | benchmark profile から reference composition を起動する |

### 5.2 所有しないもの（MUST NOT own）

- Kernel cross-plane semantics
- product policy
- production deployment
- live endpoint operation
- public distribution
- readiness claim

### 5.3 composition flow

reference composition は次の flow を所有します。

1. reference profile を読み込む。
2. Signaling reference implementation を起動する。
3. TURN reference implementation を起動する。
4. SFU reference implementation を起動する。
5. identity/session/correlation id を plane 間に渡す。
6. reference command flow を実行する。
7. plane 別 result を evidence shape へ集約する。

flow orchestration は plane 呼び出し順と data handoff を所有します。plane semantics と cross-plane validity は Kernel contract の判断として扱います。

### 5.4 evidence 境界

reference composition test は次を示し得ます。

- Signaling / TURN / SFU reference implementations が接続されること
- identity/session/correlation id が決定的に渡されること
- plane 別 result が evidence shape に集約されること

次を示しません（MUST NOT）: product completion、production readiness、live readiness、public endpoint availability、benchmark threshold satisfaction。

### 5.5 崩壊条件

- composition が Kernel cross-plane semantics を再実装する。
- composition success を product completion として扱う。
- composition success を production readiness / live readiness として扱う。
- product deployment / monitoring / rollback を reference composition に混入する。

---

## 6. composition runtime bridge

composition runtime bridge は reference Signaling / TURN / SFU を順序付きに呼び出す local orchestration です。Kernel semantics、product policy、readiness claim を所有しません。本節は入力型、実行順、出力型、fail-closed branch を固定します。

### 6.1 必須型

| Type | File | 必須フィールド |
|---|---|---|
| `ReferenceCompositionStepInput` | `reference-implementation/composition/src/step_input.rs` | `correlation_id: CorrelationId`, `step: ReferenceCompositionStep` |
| `ReferenceCompositionStep` | `reference-implementation/composition/src/step_input.rs` | 下記 enum variants |
| `ReferenceCompositionPlaneOutcome` | `reference-implementation/composition/src/runtime_bridge.rs` | enum variants `Signaling(ReferenceSignalingOutcome)`, `Turn(ReferenceTurnOutcome)`, `Sfu(ReferenceSfuOutcome)`, `Composition` |
| `ReferenceCompositionStepOutcome` | `reference-implementation/composition/src/runtime_bridge.rs` | `correlation_id: CorrelationId`, `implementation_reason: ImplementationEvidenceReason`, `applied_plane: ImplementationPlane`, `plane_outcome: ReferenceCompositionPlaneOutcome`, `non_claim_scope: Vec<ImplementationNonClaimScope>` |

`ReferenceCompositionStep` の variants は次に固定します。

- `ApplySignaling(ReferenceSignalingCommandInput)`
- `ApplyTurn(ReferenceTurnCommandInput)`
- `ApplySfu(ReferenceSfuAction)`
- `BindSignalingToTurn { room_id: RoomId, allocation_id: AllocationId }`
- `BindSignalingToSfu { room_id: RoomId, session_id: SessionId, route_id: RouteId }`
- `Validate`

`ApplySfu` は `SfuReferenceSet` または `SfuDecisionKind` を input として受け取ってはなりません（MUST NOT）。SFU decision kind は `apply_reference_sfu` が返す `ReferenceSfuOutcome` からのみ観測します。

### 6.2 必須関数

| Function | 必須シグネチャ形状 | 規則 |
|---|---|---|
| `run_reference_composition_step` | `(&mut ReferenceCompositionState, ReferenceCompositionStepInput) -> Result<ReferenceCompositionStepOutcome, ReferenceCompositionError>` | 1 つの local reference composition step を実行する |

### 6.3 実行規則

| Step | 必須操作 |
|---|---|
| `ApplySignaling` | `state.signaling` に対し `apply_reference_signaling` を呼ぶ |
| `ApplyTurn` | `state.turn` に対し `apply_reference_turn` を呼ぶ |
| `ApplySfu` | `apply_reference_sfu(&mut state.sfu, &action)` を呼び、結果を `ReferenceCompositionPlaneOutcome::Sfu` として格納する |
| `BindSignalingToTurn` | `bind_signaling_to_turn` を呼ぶ |
| `BindSignalingToSfu` | `bind_signaling_to_sfu` を呼ぶ |
| `Validate` | `validate_reference_composition_state` を呼ぶ |

各 step は最大 1 つの plane mutation または 1 つの binding validation のみを実行します。いかなる step も public endpoint を開かず、provider runtime を spawn せず、database state を書かず、readiness を主張しません（MUST NOT）。

### 6.4 outcome 規則

すべての success outcome は次を設定すること（MUST）。

- `implementation_reason = ImplementationOk`
- `non_claim_scope` に `ProductionReadinessNotClaimed` と `LiveReadinessNotClaimed` を含む
- `applied_plane` は実行した plane に一致、binding / validation の場合は `Composition`

すべての failure は `ReferenceCompositionError::implementation_reason()` を通して mapping します。

### 6.5 fail-closed 規則

| Failure | 必須 error |
|---|---|
| missing correlation id equivalent | `ReferenceCompositionError::EvidenceFieldsIncomplete` |
| orphan room / allocation / route | `ReferenceCompositionError::StateBoundaryViolation` |
| unsupported Kernel contract shape | `ReferenceCompositionError::KernelContractMismatch` |
| readiness claim attempted through bridge | `ReferenceCompositionError::EvidenceFieldsIncomplete` |

### 6.6 崩壊条件

- bridge が multiple plane mutation を single step に混在させる。
- bridge が product policy、provider runtime、database state を所有する。
- bridge が reference success を product completion / readiness として扱う。
- bridge outcome が non-claim scope を欠く。
- bridge が Kernel private field を読む。

---

## 7. runtime topology

reference implementation の初期 runtime topology は single host / controlled process topology に固定します。

| Topology | 採否 | claim boundary |
|---|---|---|
| in-process composition | reference unit / composition test で採用可能 | live readiness を主張しない |
| single host controlled process | reference benchmark / controlled integration で採用 | public live readiness を主張しない |
| multi host deployment | product implementation readiness 以降で扱う | reference scope では不採用 |
| public live endpoint | live readiness scope で扱う | reference scope では不採用 |

### 7.1 runtime ownership

reference runtime topology が所有するものは次です。

- process start / stop
- local port allocation
- controlled clock / timeout profile
- local correlation id propagation
- local metrics / logs capture
- benchmark profile binding

所有しないものは次です（MUST NOT own）: public endpoint operation、production deployment topology、production SLO、live monitoring / alerting、rollback / drain operation。

### 7.2 plane topology

reference runtime は次の plane を同一 host 上で接続します。

```text
reference signaling process/surface
reference turn process/surface
reference sfu process/surface
reference composition controller
```

初期 topology は public internet traversal を主張しません。NAT traversal / public relay / public media readiness は live readiness scope で扱います。

### 7.3 non-goals

- production process supervisor を固定すること。
- container / orchestrator / cloud deployment を固定すること。
- live endpoint availability を主張すること。

### 7.4 崩壊条件

- reference runtime success を live readiness として扱う。
- single host controlled process を production deployment として扱う。
- multi host / public endpoint を reference scope に混入する。
- topology が Kernel semantics を所有する。

---

## 8. transport 境界（staged transport）

reference implementation の transport は staged transport として扱います。

| Stage | Transport | Owner | claim boundary |
|---|---|---|---|
| RT0 | in-memory / direct command transport | reference tests | protocol wiring only |
| RT1 | loopback process transport | reference controlled integration | local runtime behavior |
| RT2 | loopback UDP/TCP/WebSocket（該当時） | reference benchmark / integration | controlled network behavior |
| RT3 | public network / browser / native device transport | real-device / live readiness scope | reference scope では不採用 |

### 8.1 plane transport 規則

| Plane | 初期 reference transport |
|---|---|
| Signaling | RT0 then RT1; WebSocket-like behavior は RT2 で扱う |
| TURN | RT0 then RT2; UDP/TCP relay behavior は controlled loopback に限定する |
| SFU | RT0 then RT2; packet forwarding は controlled loopback に限定する |
| composition | RT0/RT1/RT2 を profile で切り替える |

### 8.2 transport ownership

所有できるもの: input/output framing、local socket / loopback binding、command dispatch、packet receive/send wrapper、transport-level timeout / retry、transport observation mapping。

所有しないもの（MUST NOT own）: Kernel accept / reject semantics、Kernel routing / allocation / signaling decision、production network policy、public traversal success、live endpoint readiness。

### 8.3 upgrade 規則

transport stage を上げる場合、次を明示すること（MUST）。

1. stage id
2. target plane
3. command surface
4. environment class
5. expected outcome
6. non-claim scope

### 8.4 崩壊条件

- RT0 / RT1 success を public network success として扱う。
- RT2 loopback success を live readiness として扱う。
- transport wrapper が Kernel protocol semantics を所有する。
- browser / native transport を reference scope に無条件で混入する。

---

## 9. configuration profile 境界

reference implementation profile を次に固定します。

| Profile | Purpose | May claim | MUST NOT claim |
|---|---|---|---|
| `local-reference` | local controlled execution | reference wiring behavior | benchmark / readiness |
| `benchmark-reference` | controlled benchmark execution | benchmark observation | benchmark threshold satisfaction |
| `real-device-reference` | bounded device command input | command result | live readiness |
| `product-preflight` | product implementation preflight | product behavior input | production readiness |

### 9.1 configuration ownership

所有できるもの: local port assignment、local address / bind configuration、controlled timeout、deterministic clock / randomness setting、fixture / workload identifier、log / metric output path、correlation id prefix。

所有しないもの（MUST NOT own）: Kernel semantics、product tenant policy、production secret value、public endpoint ownership、readiness conclusion。

### 9.2 secret 規則

reference profile では production secret を扱いません（MUST NOT）。secret-like input が必要な場合は次に限定します: deterministic local test token、local fixture credential、non-production key material。production secret handling は product implementation / production readiness scope で扱います。

### 9.3 clock / randomness 規則

reference profile は deterministic clock / randomness を使用できます（MAY）。deterministic execution success は production runtime readiness を示しません。

### 9.4 profile file 仕様（local.toml / benchmark.toml）

両 profile file は同一 top-level schema を使用します。

| Section | Keys |
|---|---|
| `[profile]` | `name`, `environment_class`, `opens_public_endpoint`, `non_claim_scope` |
| `[runtime]` | `planes`, `shutdown_mode`, `timeout_ms` |
| `[fixtures]` | `rooms`, `participants_per_room`, `turn_allocations`, `sfu_routes`, `packet_payload_bytes` |
| `[evidence]` | `emit_json`, `output_dir`, `correlation_prefix` |

`opens_public_endpoint` はすべての reference profile で `false` であること（MUST）。profile files は Rust workspace member ではありません。

`local.toml` は次に固定します。

```toml
[profile]
name = "reference-local"
environment_class = "local_single_host"
opens_public_endpoint = false
non_claim_scope = ["production_readiness_not_claimed", "live_readiness_not_claimed", "public_distribution_readiness_not_claimed"]

[runtime]
planes = ["signaling", "turn", "sfu", "composition"]
shutdown_mode = "GracefulLocal"
timeout_ms = 5000

[fixtures]
rooms = 2
participants_per_room = 3
turn_allocations = 2
sfu_routes = 4
packet_payload_bytes = 1200

[evidence]
emit_json = true
output_dir = "target/reference-evidence/local"
correlation_prefix = "impl-reference-local"
```

`benchmark.toml` は次に固定します。

```toml
[profile]
name = "reference-benchmark"
environment_class = "benchmark_host"
opens_public_endpoint = false
non_claim_scope = ["benchmark_threshold_not_claimed", "production_readiness_not_claimed", "live_readiness_not_claimed"]

[runtime]
planes = ["signaling", "turn", "sfu", "composition"]
shutdown_mode = "GracefulLocal"
timeout_ms = 30000

[fixtures]
rooms = 8
participants_per_room = 6
turn_allocations = 8
sfu_routes = 24
packet_payload_bytes = 1200

[evidence]
emit_json = true
output_dir = "target/reference-evidence/benchmark"
correlation_prefix = "impl-reference-benchmark"
```

### 9.5 profile validation 規則

profile validation は次を拒否します（fail-closed）。

- `opens_public_endpoint = true`
- 空の `planes`
- `timeout_ms = 0`
- `packet_payload_bytes = 0`
- `non_claim_scope` の欠落
- 許容された non-claim scope wire 集合外の `non_claim_scope` item
- `target/reference-evidence/` 外の output path

### 9.6 崩壊条件

- reference profile が public endpoint を開く。
- profile file が real credential、real endpoint、secret、token、private key を含む。
- benchmark profile success を benchmark threshold satisfaction として扱う。
- local profile success を production readiness または live readiness として扱う。
- reference profile に production secret を置く。
- deterministic clock / randomness success を production readiness として扱う。
- profile が readiness conclusion を所有する。
- profile が Kernel behavior を変更する。

---

## 10. 本章の不変条件（要約）

1. 依存方向は `implementations -> Kernel`（contract のみ）。reference implementation は Kernel contract を変更しません（MUST NOT）。
2. reference success は production readiness / live readiness / benchmark threshold satisfaction を意味しません（fail-closed）。
3. product implementation が消費できる reference 型は 3-type allow-list（`ReferenceSignalingOutcome` / `ReferenceTurnOutcome` / `ReferenceSfuOutcome`）に限定します。詳細は第08章を参照。
4. すべての reference error は `ImplementationEvidenceReason` に mapping し、`Unknown` を持ちません。
5. すべての outcome は correlation id と non-claim scope を伴います。これらを欠く場合 fail-closed で拒否します。
