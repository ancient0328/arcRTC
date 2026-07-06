# Toolchain / runtime / dependency（実装 toolchain・runtime・依存許可）

状態: SSOT 統合版
日付: 2026-06-28 JST

## 目的

本章は、arcRTC v0.2 の distro 領域（Kernel 外実装領域）における実装言語 / toolchain 境界、runtime（executor）境界、workspace 構成、および dependency admission（依存許可）の閉集合を完全に固定します。本章は、SFU / TURN / Signaling の primary distro 言語、Rust workspace root / member、runtime executor の所有・非所有、Kernel local path dependency matrix、distro-local dependency matrix、external dependency の許可リスト、admission record の必須フィールド、禁止 pattern、および fail-closed 条件を内在化します。本章は自己完結であり、他文書を開かずに再現実装できる粒度で記述します。

## 依存方向の不変条件

distro 側 dependency は Kernel contract を消費するための technical dependency であり、Kernel semantics を所有しません（`distro -> Kernel`、`Kernel -X-> distro`）。runtime executor も external dependency も、Kernel semantic authority / Kernel reason catalog / Kernel port definition を所有してはなりません（MUST NOT）。dependency API の都合で Kernel semantics を変えてはなりません（MUST NOT）。

## 1. 実装言語 / toolchain 境界

distro の SFU / TURN / Signaling primary distro language は Rust に固定します（MUST）。Kernel は Rust workspace として成立しており、distro 側は Kernel を変更せず、Kernel contract を利用する側の source package として設計します。

| Surface | 決定 | Authority boundary |
|---|---|---|
| reference distro | Rust | SFU / TURN / Signaling の reference runtime behavior を所有する |
| product distro | Rust を primary とする | product policy / persistence / deployment は product distro 側で所有する |
| benchmark harness | Rust primary | Kernel / distro の comparable workload を実行する |
| real-device harness | 必要時に browser / native / shell harness を追加可能 | real-device command evidence に限定し、distro authority を所有しない |
| TypeScript / Swift / Kotlin | harness / projection に限定 | SFU / TURN / Signaling distro owner にしない |

Rust package は `distro/` 配下に配置します（MUST）。Kernel source を distro 側へコピーしてはなりません（MUST NOT）。Kernel crate は version / path pin boundary に従って利用します。

### Toolchain 規則（閉集合）

| Toolchain | 規則 |
|---|---|
| Rust / Cargo | distro primary toolchain |
| pnpm | TypeScript / browser harness を導入する場合のみ使用する |
| npm / yarn | 使用しない |
| Swift / Gradle | real-device / native harness が必要な場合のみ採用する |

toolchain version / command / working directory / package name の authority boundary は本章で固定します。具体的な command surface は第14章で規定します。

## 2. Workspace root

distro の Rust workspace root を次に固定します。

| Item | Fixed value |
|---|---|
| working directory | `distro/` |
| workspace manifest | `distro/Cargo.toml` |
| edition | `2021` |
| rust-version | `1.96` |
| resolver | `2` |
| license | `Apache-2.0` |
| publish | `true` |
| unsafe code | `forbid` |

Kernel workspace へ distro package を member として追加してはなりません（MUST NOT）。distro は別 workspace として Kernel local path dependency を利用します。

## 3. Workspace members（閉集合）

初期 workspace members を次に固定します。

| Path | Package name | Role |
|---|---|---|
| `distro-support/evidence` | `arcrtc-distro-evidence` | shared distro evidence / reason types |
| `reference-distro/output` | `arcrtc-reference-output` | reference output outcome types; product input subset is the 3-type allow-list |
| `reference-distro/signaling` | `arcrtc-reference-signaling` | Signaling reference distro |
| `reference-distro/turn` | `arcrtc-reference-turn` | TURN reference distro |
| `reference-distro/sfu` | `arcrtc-reference-sfu` | SFU reference distro |
| `reference-distro/composition` | `arcrtc-reference-composition` | reference composition |
| `reference-distro/ops` | `arcrtc-reference-ops` | reference execution helper |
| `product-distro/signaling` | `arcrtc-product-signaling` | product Signaling distro |
| `product-distro/turn` | `arcrtc-product-turn` | product TURN distro |
| `product-distro/sfu` | `arcrtc-product-sfu` | product SFU distro |
| `product-distro/product-policy` | `arcrtc-product-policy` | product policy |
| `product-distro/persistence-topology` | `arcrtc-product-persistence-topology` | product persistence topology |
| `product-distro/deployment` | `arcrtc-product-deployment` | product deployment profile |
| `product-distro/monitoring` | `arcrtc-product-monitoring` | product monitoring / probes |
| `product-distro/rollback` | `arcrtc-product-rollback` | product rollback / drain |

`reference-distro/deployment-profiles/` は runtime profile data を置く領域であり、初期状態では Rust package にしません。`tests/` 配下の packages は `tests/` 配下で別途所有します。

## 4. Runtime executor

reference / product runtime executor は Tokio-compatible runtime を primary executor として採用します（MUST）。runtime executor は distro 側 dependency であり、Kernel dependency ではありません。

### runtime executor が所有するもの（閉集合）

- task scheduling
- socket polling
- local timer / timeout
- process start / stop orchestration
- graceful shutdown signal handling
- benchmark / integration command execution profile

### runtime executor が所有しないもの（閉集合）

- Kernel protocol semantics
- Kernel reason catalog
- Kernel port definition
- product readiness claim
- live readiness claim
- security policy authority

### runtime surface 所有 file

| Runtime surface | 作成する owner file | 所有内容 |
|---|---|---|
| reference executor | `reference-distro/ops/src/runtime.rs` | local process orchestration, timer, shutdown signal |
| reference composition runner | `reference-distro/composition/src/runtime_bridge.rs` | Signaling / TURN / SFU local composition execution |
| product executor | `product-distro/deployment/src/runtime.rs` | product process profile |
| product shutdown / drain | `product-distro/rollback/src/drain.rs` | product drain orchestration |

## 5. Initial source file contract

すべての workspace package に `Cargo.toml` と `src/lib.rs` を必須とします（MUST）。`Cargo.toml` の literal shape は第02章の workspace manifest / dependency path 規則が定め、`src/lib.rs` の public export list は第02章の module export 規則が定めます。

`src/kernel_contract.rs` は universal file ではありません。Kernel public contract を直接 adapt する package にだけ必須です。

| Package class | Packages | 必須追加 file |
|---|---|---|
| distro support | `arcrtc-distro-evidence` | `src/reason.rs`, `src/record.rs`, `src/readiness_extension.rs`, `src/validation.rs`, `src/error.rs` |
| reference output | `arcrtc-reference-output` | `src/signaling.rs`, `src/turn.rs`, `src/sfu.rs`, `src/composition.rs`, `src/error.rs` |
| reference plane | `arcrtc-reference-signaling`, `arcrtc-reference-turn`, `arcrtc-reference-sfu` | `src/kernel_contract.rs`, `src/state.rs`, reference distro で固定された fixture/auth files, `src/error.rs` |
| reference composition | `arcrtc-reference-composition` | `src/step_input.rs`, `src/composition_state.rs`, `src/runtime_bridge.rs`, `src/error.rs` |
| reference ops | `arcrtc-reference-ops` | `src/runtime.rs`, `src/evidence.rs`, `src/reason.rs`, `src/error.rs` |
| product plane | `arcrtc-product-signaling`, `arcrtc-product-turn`, `arcrtc-product-sfu` | `src/kernel_contract.rs`, `src/error.rs` |
| product policy | `arcrtc-product-policy` | `src/auth_policy.rs`, `src/security_reason.rs`, `src/error.rs` |
| product persistence | `arcrtc-product-persistence-topology` | `src/topology.rs`, `src/mapper.rs`, `src/error.rs` |
| product deployment | `arcrtc-product-deployment` | `src/runtime.rs`, `src/profile.rs`, `src/error.rs` |
| product monitoring | `arcrtc-product-monitoring` | `src/observability.rs`, `src/evidence.rs`, `src/error.rs` |
| product rollback | `arcrtc-product-rollback` | `src/drain.rs`, `src/restore.rs`, `src/error.rs` |

`distro-support/evidence` は `src/kernel_contract.rs` を持ちません。product support packages も上表に列挙されない限り `src/kernel_contract.rs` を作りません。binary entrypoint は reference / product completion 前には作りません。CLI / command binary が必要な場合は第14章で固定された command surface に従って追加します。

## 6. Dependency admission record（必須フィールド）

dependency を source に追加する場合、該当 `Cargo.toml` と report に次を記録します（MUST）。record を持たない dependency は未採用として扱います。

| Field | 必須値 |
|---|---|
| dependency name | crate name or local package name |
| dependency source | `local-kernel-path` / `crates-io` / `workspace` |
| class | `kernel-contract` / `reference-output` / `runtime` / `serialization` / `observability` / `command` / `benchmark` / `test-only` |
| owner package | distro package name |
| allowed use | one sentence |
| forbidden use | one sentence |
| version rule | exact lockfile version or local path |
| feature rule | explicit feature list |
| non-claim scope | readiness / completion claim not implied |

### Dependency admission class（閉集合）

| Class | Examples | Admission owner | Claim boundary |
|---|---|---|---|
| runtime | async executor / timer | runtime surface | executor behavior に限定 |
| transport | UDP / TCP / WebSocket / TLS crate | transport surface | I/O distro に限定 |
| serialization | JSON / binary codec | mapping surface | wire mapping に限定 |
| observability | logging / metrics / tracing | observability surface | evidence emission に限定 |
| command | CLI / config loader | command surface | operator command surface に限定 |
| benchmark / test | criterion / harness helper | benchmark surface | benchmark / test execution に限定 |

## 7. Allowed Kernel dependency matrix（閉集合）

Kernel local path dependency は次の範囲に限定します。表に列挙されない Kernel package の import は禁止です（MUST NOT）。

| distro package | Allowed Kernel packages |
|---|---|
| `arcrtc-distro-evidence` | none |
| `arcrtc-reference-output` | `arcrtc-core-signaling`, `arcrtc-core-turn`, `arcrtc-core-sfu`, `arcrtc-core-identity` |
| `arcrtc-reference-signaling` | `arcrtc-core-signaling`, `arcrtc-core-command`, `arcrtc-core-identity`, `arcrtc-core-reason`, `arcrtc-core-ports`, `arcrtc-core-protocol`, `arcrtc-core-configuration` |
| `arcrtc-reference-turn` | `arcrtc-core-turn`, `arcrtc-core-identity`, `arcrtc-core-transport`, `arcrtc-core-security`, `arcrtc-core-time`, `arcrtc-core-reason`, `arcrtc-core-ports`, `arcrtc-core-configuration` |
| `arcrtc-reference-sfu` | `arcrtc-core-sfu`, `arcrtc-core-identity`, `arcrtc-core-transport`, `arcrtc-core-quality`, `arcrtc-core-state`, `arcrtc-core-reason`, `arcrtc-core-ports`, `arcrtc-core-configuration` |
| `arcrtc-reference-composition` | `arcrtc-core-cross-plane`, `arcrtc-core-operation`, `arcrtc-core-runtime`, `arcrtc-core-reason`, `arcrtc-core-identity` |
| `arcrtc-reference-ops` | `arcrtc-core-command`, `arcrtc-core-operation`, `arcrtc-core-runtime`, `arcrtc-core-identity` |
| `arcrtc-product-signaling` | `arcrtc-core-signaling`, `arcrtc-core-command`, `arcrtc-core-identity`, `arcrtc-core-reason` |
| `arcrtc-product-turn` | `arcrtc-core-turn`, `arcrtc-core-transport`, `arcrtc-core-security`, `arcrtc-core-identity`, `arcrtc-core-reason` |
| `arcrtc-product-sfu` | `arcrtc-core-sfu`, `arcrtc-core-transport`, `arcrtc-core-quality`, `arcrtc-core-state`, `arcrtc-core-identity`, `arcrtc-core-reason` |
| `arcrtc-product-policy` | `arcrtc-core-command`, `arcrtc-core-identity`, `arcrtc-core-reason` |
| `arcrtc-product-persistence-topology` | `arcrtc-core-state`, `arcrtc-core-reason` |
| `arcrtc-product-deployment` | `arcrtc-core-configuration`, `arcrtc-core-operation`, `arcrtc-core-runtime`, `arcrtc-core-identity` |
| `arcrtc-product-monitoring` | `arcrtc-core-audit`, `arcrtc-core-quality`, `arcrtc-core-identity`, `arcrtc-core-reason` |
| `arcrtc-product-rollback` | `arcrtc-core-recovery`, `arcrtc-core-operation`, `arcrtc-core-identity`, `arcrtc-core-reason` |

## 8. Allowed distro-local dependency matrix（閉集合）

distro-local path dependency は次の範囲に限定します。表に列挙されない distro-local package への依存は禁止です（MUST NOT）。

| distro package | Allowed distro-local packages |
|---|---|
| `arcrtc-distro-evidence` | none |
| `arcrtc-reference-output` | `arcrtc-distro-evidence` |
| `arcrtc-reference-signaling` | `arcrtc-distro-evidence`, `arcrtc-reference-output` |
| `arcrtc-reference-turn` | `arcrtc-distro-evidence`, `arcrtc-reference-output` |
| `arcrtc-reference-sfu` | `arcrtc-distro-evidence`, `arcrtc-reference-output` |
| `arcrtc-reference-composition` | `arcrtc-distro-evidence`, `arcrtc-reference-output`, `arcrtc-reference-signaling`, `arcrtc-reference-turn`, `arcrtc-reference-sfu` |
| `arcrtc-reference-ops` | `arcrtc-distro-evidence`, `arcrtc-reference-composition` |
| `arcrtc-product-signaling` | `arcrtc-distro-evidence`, `arcrtc-reference-output`, `arcrtc-product-policy` |
| `arcrtc-product-turn` | `arcrtc-distro-evidence`, `arcrtc-reference-output`, `arcrtc-product-policy` |
| `arcrtc-product-sfu` | `arcrtc-distro-evidence`, `arcrtc-reference-output`, `arcrtc-product-policy` |
| `arcrtc-product-policy` | `arcrtc-distro-evidence` |
| `arcrtc-product-persistence-topology` | `arcrtc-distro-evidence` |
| `arcrtc-product-deployment` | `arcrtc-distro-evidence`, `arcrtc-product-rollback` |
| `arcrtc-product-monitoring` | `arcrtc-distro-evidence` |
| `arcrtc-product-rollback` | `arcrtc-distro-evidence` |

Kernel package を追加で必要とする場合は、source 変更前にこの matrix を更新します（MUST）。literal local path と package manifest shape は第02章の workspace manifest / dependency path 規則に従います。Test package dependency admission は test workspace manifest が所有し、distro workspace package の dependency admission と混在させません。product plane package から参照できる reference-facing dependency は `arcrtc-reference-output` のみに限定します。product plane package は `arcrtc-reference-signaling`、`arcrtc-reference-turn`、`arcrtc-reference-sfu`、`arcrtc-reference-composition`、`arcrtc-reference-ops` に依存してはなりません（MUST NOT）。

## 9. External dependency admission（閉集合）

初期 external dependency class を次に限定します。表に列挙されない external dependency の導入は禁止です（MUST NOT）。

| Dependency | Version requirement | Features | Class | Allowed use | Forbidden use |
|---|---|---|---|---|---|
| `tokio` | `1` | `rt-multi-thread`, `macros`, `net`, `time`, `signal`, `sync` | runtime | async executor, socket polling, timer, shutdown signal | Kernel semantics / Kernel reason ownership |
| `serde` | `1` | `derive` | serialization | config / evidence struct serialization | protocol authority ownership |
| `serde_json` | `1` | none | serialization | JSON command / evidence artifact | readiness proof by itself |
| `tracing` | `0.1` | none | observability | structured distro log / span | adoption as evidence by itself |
| `tracing-subscriber` | `0.3` | `fmt`, `env-filter` | observability | local subscriber for command execution | production monitoring readiness |
| `clap` | `4` | `derive` | command | bounded CLI argument parsing | product policy authority |
| `criterion` | `0.5` | none | benchmark / test-only | benchmark harness | benchmark threshold satisfaction |

exact crate version は source scaffold task で `Cargo.lock` に固定します（MUST）。version が `Cargo.lock` に存在しない dependency は正式 evidence に採用しません（MUST NOT）。

## 10. Prohibited dependency pattern（閉集合）

次は許可しません（MUST NOT）。

- dependency の raw error を Kernel reason として採用する。
- dependency default behavior を readiness evidence として扱う。
- product persistence provider を reference distro に追加する。
- cloud provider SDK を reference distro に追加する。
- dependency version を report 後に成功へ合わせて変更する。
- `DistroEvidenceReason` を使う package から `arcrtc-distro-evidence` dependency を省略する。
- product plane package から `arcrtc-reference-output` 以外の reference package dependency を追加する。
- dependency API に合わせて Kernel semantics を変える。
- dependency admission を product admission または production readiness とみなす。

## 11. 不変条件（collapse conditions）

本章が定義する境界は、次のいずれかが発生した時点で崩壊します。これらは禁止であり、発生時は fail-closed とします（MUST NOT）。

- distro package を Kernel workspace member に追加する。
- TypeScript / Swift / Kotlin / browser harness を SFU / TURN / Signaling primary distro owner にする。
- Kernel source を distro 側へコピーする。
- npm / yarn を distro toolchain として採用する。
- toolchain choice / dependency choice を production readiness または live readiness の根拠にする。
- 本章が固定する toolchain / language を未定義のままにし、後から別所で決定する。
- runtime executor が Kernel reason catalog または Kernel port definition を所有する。
- `distro-support/evidence` が Kernel crate dependency を持つ。
- product support package が不要な `src/kernel_contract.rs` を作り、Kernel contract owner と誤認される。
- product plane package が `arcrtc-reference-output` 以外の reference package dependency を持つ。
- `tests/` package を distro workspace の完了条件に混入する。
- binary entrypoint success / runtime executor success を reference / product completion として扱う。
- admission record なしに dependency を追加する。
- Kernel local path dependency matrix を超える import を行う。
- distro-local dependency matrix を超える import を行う。
- product plane package が reference state / fixture / local auth / runtime / composition package へ直接依存する。
- external dependency が Kernel semantic authority を所有する。
- `Cargo.lock` に固定されていない dependency result を正式 evidence として採用する。
- dependency admission class を持たない external dependency を source に導入する。
- 複数 runtime を正式な版管理された決定なしに混在させる。

## 12. Non-claim

本章は、dependency list finalization、supply-chain readiness、reference distro completion、product distro completion、benchmark execution、benchmark threshold satisfaction、real-device command success、production readiness、live readiness、security readiness、または Kernel toolchain の変更を主張しません。本章は distro 側の language / toolchain / runtime executor / dependency admission の境界のみを固定します。
