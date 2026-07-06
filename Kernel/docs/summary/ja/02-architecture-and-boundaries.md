# アーキテクチャと境界
状態: public summary projection
日付: 2026-07-06 JST

## 目的

本章は arcRTC v0.2 Kernel の層モデル、依存方向（許可/禁止の全方向）、各層が所有/非所有するもの、crate / package 境界と package 役割、semantic modular monolith の原則、source shard modularity、no-copy / selective extraction 方針を要約します。core 内サブモジュールの一覧と責務、境界判断順序、崩壊条件を提示します。

## 層モデル

arcRTC v0.2 は DDD / ヘキサゴナルアーキテクチャを採用します。設計の中心は、Kernel の semantic authority、external driver、Kernel 内 entrypoint、SDK、regulated support、Kernel 外 distro を混同しないことです。

```text
core <- drivers
core <- entrypoints
drivers <- entrypoints

sdk: independent Signaling-only boundary
regulated: optional domain support
```

依存方向記法 `A <- B` は「B が A に依存する（B から A を参照可）」を意味します。`Kernel/` は Kernel root です。Kernel 内の `core` は semantic nucleus、中枢、最高権威であり、Kernel 全体の別名ではありません。Kernel 外 distro は Kernel contract を利用する側であり、SFU / TURN / Signaling reference distro または product distro を所有します。Kernel 内の `entrypoints/*-server` は product system ではなく、executable contract / composition evidence surface に限定されます。

## Contract Change Scope（契約変更保護対象）

Kernel contract、semantic vocabulary、port ownership、dependency direction は protected change surface です。これらの変更には、versioned contract と compatibility / deprecation rule（第06章の protocol versioning / deprecation 規範）が必要です。これらの surface の保護は production / live / native application readiness を意味せず、それらは distro-owned claim であり、Kernel completion evidence ではありません。

## 依存方向（許可/禁止の全方向）

許可する依存方向は次に限定されます。

```text
core <- drivers
core <- entrypoints
drivers <- entrypoints
```

条件付きで許可し得る方向は次のみです。

```text
core <- regulated
```

`core <- regulated` は、通信イベント型や opaque ID 参照のために必要な場合のみ、仕様で明示したうえで許可し得ます。具体的には仕様で明示された opaque communication event、audit pointer、non-sensitive tag の参照に限定します。`sdk` は Signaling-only 独立境界であり、regulated を直接所有しません。

禁止する依存方向（Hard Prohibitions）は次のとおりです。1つも緩和されません。

```text
core -> drivers
core -> entrypoints
core -> regulated
drivers -> entrypoints
drivers -> regulated
entrypoints -> regulated
sdk -> regulated
regulated -> sdk
regulated -> drivers
regulated -> entrypoints
```

加えて、次の具体的混入を禁止します。

- core に WebSocket / HTTP / UDP / TCP socket concrete type を置く。
- core に str0m concrete type を置く。
- core に PostgreSQL / Redis / S3 / filesystem sink concrete implementation を置く。
- core に environment variable parsing を置く。
- entrypoints に protocol decision を置く。
- entrypoints を SFU / TURN / Signaling product distro として扱う。
- distro product policy を Kernel contract として採用する。

## 各層が所有 / 非所有するもの

| Layer | 所有するもの | 所有しないもの |
|---|---|---|
| core | Kernel semantic nucleus、domain semantics、use case、port、contract、state、decision | I/O、runtime、framework、DB、cloud SDK、browser/native 型、distro product policy |
| drivers | port implementation、external type conversion、I/O | domain rule、accept/reject rule |
| entrypoints | executable contract evidence、CLI、demo、composition root | domain rule、protocol semantics、SFU / TURN / Signaling product distro |
| sdk | Signaling-only public client contract | media、auth issuance、regulated workflow |
| regulated | optional domain support、enrichment、pointer mapping | generic communication protocol の正 |

drivers では外部型を drivers 境界で core-owned type へ変換します。entrypoints は Kernel 内の executable contract / composition evidence surface と dependency wiring に限定され、domain rule を所有しません。

## 境界判断順序

ある対象がどの層に属するかは、次の順序で判断します。

1. その対象は protocol / domain semantics か。→ 該当すれば core 候補。
2. その対象は external technology implementation か。→ 該当すれば drivers 候補。
3. その対象は executable composition root か。→ 該当すれば entrypoints 候補。
4. その対象は public SDK client boundary か。→ 該当すれば sdk 候補。
5. その対象は optional regulated support か。→ 該当すれば regulated 候補。

複数に該当する場合は、同じ file / module に混在させず分離します。

## core 内サブモジュールの一覧と責務

`core/` は外部 I/O に依存しない Kernel の semantic nucleus、中枢、最高権威です。core 全体は単一の semantic authority であり、内部の module/package は deployment boundary ではなく semantic boundary です。core 内サブモジュールの一覧と責務は次のとおりです。

| Path | 責務 |
|---|---|
| `core/transport/` | Sans-IO transport contract、RTP / RTCP / ICE-adjacent semantics |
| `core/signaling/` | Signaling contract、room state、command semantics、accept / reject boundary |
| `core/sfu/` | routing、quality decision、backpressure semantics |
| `core/turn/` | allocation、permission、relay semantics |
| `core/security/` | token verification boundary、identity-neutral primitives |
| `core/audit/` | audit event model、hash-chain contract、audit ports |
| `core/quality/` | metrics model、quality decisions |
| `core/ports/` | network、persistence、clock、metrics、audit sink 等の port |
| `core/domain/` | DDD domain model、aggregate、domain service、application use case |
| `core/identity/` | opaque references、correlation IDs、external identity separation |
| `core/command/` | command、decision、event、response、idempotency、replay、correlation |
| `core/reason/` | closed reason categories and codes |
| `core/protocol/` | deterministic encoding、protocol versioning、compatibility/deprecation |
| `core/configuration/` | core-owned configuration boundary types |
| `core/features/` | out-of-scope feature admission and rejection |
| `core/cross-plane/` | cross-plane identity / session binding semantics |
| `core/operation/` | operation semantics、operational boundary model |
| `core/runtime/` | runtime abstraction、task / worker lifecycle contract |
| `core/time/` | time、clock、timestamp trust semantics |
| `core/state/` | state transition、state ownership semantics |
| `core/recovery/` | recovery、retry、compensation semantics |

これらは同じ workspace / release unit に属してよいものです。ただし semantic owner と dependency direction は module/package 境界で固定します。

## drivers / entrypoints のサブモジュール

`drivers/` は core port の実装です。

| Path | 責務 |
|---|---|
| `drivers/webrtc-str0m/` | str0m を用いた WebRTC transport port implementation |
| `drivers/network/` | tokio UDP / TCP、HTTP、WebSocket、socket I/O |
| `drivers/persistence/` | PostgreSQL、S3、filesystem、in-memory |
| `drivers/observability/` | tracing、metrics exporter、log sinks |
| `drivers/security/` | key source、verifier driver、secret rotation |
| `drivers/browser/` | browser-facing helper boundary |
| `drivers/native/` | native platform binding boundary |

`entrypoints/` は Kernel 内の起動単位、contract probe、dependency wiring、composition evidence surface です。SFU / TURN / Signaling product system は所有しません。

| Path | 責務 |
|---|---|
| `entrypoints/signaling-server/` | Signaling executable contract / composition evidence surface |
| `entrypoints/sfu-server/` | SFU executable contract / composition evidence surface |
| `entrypoints/turn-server/` | TURN executable contract / composition evidence surface |
| `entrypoints/cli/` | operator / developer CLI |
| `entrypoints/demo/` | demo composition |
| `entrypoints/configuration/` | configuration profile、policy bundle、feature/capability wiring |
| `entrypoints/endpoints/` | public endpoint lifecycle、edge/proxy trust wiring |
| `entrypoints/topology/` | deployment topology、service discovery、endpoint resolution wiring |
| `entrypoints/internal-control/` | internal service identity and control-plane wiring |
| `entrypoints/admin/` | health/readiness/liveness/admin/operator wiring |

## crate / package 境界と package 役割

v0.2 package layer は architecture layer に従います。package boundary は Cargo.toml 実体の作成許可ではなく、実装 scaffold 時に破ってはならない package dependency rule を固定するものです。

| Layer | Package role | Dependency rule |
|---|---|---|
| core | domain, use case, port, contract | drivers/entrypoints に依存しない |
| drivers | port implementation | core に依存できる |
| entrypoints | Kernel executable contract, CLI, demo, wiring | core と selected drivers に依存できる |
| sdk | Signaling-only public client package | core internals / drivers / regulated に依存しない |
| regulated | optional domain support | 仕様で許可された opaque core references のみ |

### Naming Rule

Rust package naming は layer と role を反映しなければなりません。technology-specific name は driver package name にのみ現れてよいものです。

許可する命名パターン例:

| Layer | Pattern |
|---|---|
| core | `arcrtc-core` |
| driver | `arcrtc-driver-network`, `arcrtc-driver-webrtc-str0m`, `arcrtc-driver-persistence-*` |
| entrypoint | `arcrtc-signaling-server`, `arcrtc-sfu-server`, `arcrtc-turn-server`, `arcrtc-cli` |
| sdk | `arcrtc-sdk-*` |
| regulated | `arcrtc-regulated-*` |

禁止する命名パターン例:

- `arcrtc-core-str0m`
- `arcrtc-core-postgres`
- `arcrtc-entrypoint-domain`
- `arcrtc-sdk-regulated`

### Feature Flag Rule

feature flag は architecture dependency を反転させてはなりません。

禁止:

- core feature が driver dependency を core に引き込む。
- core default feature が DB / S3 / HTTP / str0m / tokio concrete implementation を有効化する。
- entrypoint feature が domain semantics を変更する。
- driver feature が alternate reason catalog を定義する。
- sdk feature が regulated support を import する。

許可:

- driver package feature が external implementation detail を選択する。
- entrypoint package feature が composition profile を選択する。
- core test feature が external implementation dependency なしに test helper を公開する。

### Workspace Rule

workspace membership は architecture permission を意味しません。dependency admission、license、vulnerability、lockfile、toolchain gate は supply-chain policy を満たさなければなりません。単一 workspace 内であっても、dependency direction は次を維持します。

```text
core <- drivers
core <- entrypoints
drivers <- entrypoints
```

`sdk` は independent Signaling-only boundary を維持します。`regulated -> core` は明示的な仕様制約のもとでのみ許可されます。

### Test Utility Rule

test utility package は hidden production dependency になってはなりません。test helper は core test API と fake driver に依存してよいものですが、core production package は fake driver package に依存してはなりません。v0.1 の tests と benches は、v0.2 testing / benchmark 規範で再認定されるまで evidence-only です。

### Package 境界の禁止事項

- core package が driver package に依存する。
- driver package が entrypoint package に依存する。
- entrypoint package が port trait を定義する。
- SDK package が regulated package に依存する。
- regulated package が drivers/entrypoints/sdk に依存する。
- feature flag が仕様境界を迂回する。
- workspace root が concrete driver type を core API として re-export する。
- source shard を semantic owner の移動や dependency direction 隠蔽に使う。
- source-shape test が root file のみを検査し、source shard が実装本体を運ぶ。
- package manager の install 成功を dependency policy 受諾として扱う。
- dependency の license/vulnerability/toolchain status を release/readiness 主張のために隠す。
- package build output を provenance なしに distributable release artifact として扱う。
- source file 行数を、semantic owner / dependency direction / forbidden import 不在より強い close 条件として扱う。
- core package が undeclared core peer dependency を導入する。
- production source が workspace policy と semantic record boundary の代わりに local `#[allow(clippy::too_many_arguments)]` を使う。

## semantic modular monolith の原則

core は semantic modular monolith です。v0.2 の主問題は network hop の不足ではなく、plane 間の意味論混線です。Signaling accepted、SFU admitted、TURN allocated、ICE connectivity、secure media protected、audit recorded、reason classified の関係が曖昧になると、core を小さく分けても system boundary は成立しません。したがって core を小さくすること、または source file を一定行数以下にすることは上位目的ではありません。上位目的は、単一の semantic authority を持つ core の内部を、意味論単位の module/package として厳格に分けることです。

原則は次のとおりです。

1. core 全体は単一の semantic authority である。
2. core 内 module/package は deployment boundary ではなく semantic boundary である。
3. module/package の分割軸は WebSocket、UDP、str0m、DB、metrics、HTTP などの技術ではなく、Signaling state、SFU routing、TURN lifecycle、identity、reason、audit、quality、ports、protocol、configuration、features、cross-plane binding である。
4. core module/package は、foundation core module と明示された peer core contract にのみ依存できる。
5. core module/package は driver、entrypoint、framework、async runtime、socket、DB、browser/native SDK、cloud SDK、concrete transport type を import してはならない。
6. cross-plane access は core-owned identity、command、reason、event、binding type を通す。
7. source file line count は auditability guideline であり、semantic boundary ではない。
8. 600 行は source comprehension の review signal として扱うが、core semantic modular monolith boundary より上位の hard gate ではない。

core package peer dependency は閉集合です。各 `core/*/Cargo.toml` は semantic modular monolith boundary tests が宣言する foundation/peer core package にのみ依存してよいものです。新しい `arcrtc-core-*` dependency の追加は、peer dependency matrix と関連する仕様 relation の更新を要します。

### core サブモジュールごとの semantic boundary

| Surface | 許可 | 禁止 |
|---|---|---|
| core.signaling | core identity / command / reason / protocol / ports への依存 | WebSocket / HTTP / tokio runtime / entrypoint server ownership |
| core.sfu | core identity / reason / quality / ports / cross-plane への依存 | str0m concrete event / driver packet buffer ownership |
| core.turn | core identity / reason / security / ports / cross-plane への依存 | UDP/TCP socket behavior / entrypoint server lifecycle |
| core.audit | core identity / reason / protocol / audit model への依存 | DB row shape / concrete storage retry policy |
| core.reason | closed reason vocabulary の所有 | drivers/entrypoints による reason extension |
| core.cross-plane | admitted binding class と target-plane relation の所有 | CorrelationId、network address、external user ID だけからの implicit binding |

## source shard modularity

source shard は、semantic owner を移動しない範囲で、監査容易性のために許可される物理分割です。これは semantic modular monolith boundary の下位規則です。実装が進むと、単一 `lib.rs` や単一 SDK surface file に複数の型、guard、failure、projection が集まり、監査時に責務境界が読みにくくなります。600 行を超える source file は境界違反を直接意味しませんが、監査・レビュー・後続作業の source comprehension を弱め、dead code、legacy path、重複実装、fail-open search の見落としを誘発しうるため、review signal として扱います。

規則は次のとおりです。

1. 1 source file は原則として読み切れる粒度に保ち、600 行は review signal として扱う。
2. Rust crate の public API を crate root に維持する必要がある場合、`include!` による source shard を許可する。
3. Rust source shard は同一 module scope に include し、owner、public API、semantic authority を移動しない。これは physical source split であり、semantic owner transfer ではない。
4. Swift SDK は Swift Package target の通常 source set 分割で責務別 file に分ける。分割は Signaling-only SDK surface を保ち、media、auth issuance、regulated workflow、driver internals を SDK に移してはならない。
5. source-shape tests は単一 `lib.rs` の文字列ではなく、対象 crate/package の source set 全体を検査する。
6. source shard は test や docs の evidence substitution ではない。build/test/lint の代替として採用しない。
7. tests は semantic owner / dependency direction / forbidden import / source-set visibility の崩壊を fail させる。source file が 600 行を超えたことだけでは fail させない。

shard file は新たな core/drivers/entrypoints/sdk/regulated boundary、dependency permission、evidence class を作ってはなりません。高 arity の semantic evidence / guard record constructor は、local `#[allow(clippy::too_many_arguments)]` で隠してはなりません。workspace は中央 Clippy threshold を定義してよいものですが、production source は local suppression を導入してはなりません。

### source shard の境界

| Surface | Allowed | Forbidden |
|---|---|---|
| Rust crate root | public API-preserving `include!` source shard | semantic owner の移動 |
| Rust source-shape test | crate/package source set scan | `lib.rs` だけを authority とする検査 |
| SDK iOS | target source set split | Signaling-only surface の変更 |
| source-shape test scope | semantic modular monolith collapse detection and source-size review signal | line count を semantic boundary より上位の hard gate にすること |

## no-copy / selective extraction 方針

v0.2 では、v0.1.2 の機械的コピーを禁止し、selective extraction を採用します。selective extraction とは、v0.1.2 の source / docs / evidence から、v0.2 の仕様で明示採用された意味論、契約、型、state、port だけを取り出すことです。

以下は v0.2 に直接コピーしてはなりません。

- `server/signaling`
- `server/sfu`
- `server/turn`
- `core` crate 全体
- `sdk` 実装全体
- `regulated` 実装全体
- v0.1 開発記録 の完了語、close 語、証跡主張
- ignored / generated / local artifact

v0.1.2 由来要素を v0.2 に採用する条件は次のとおりです。

1. 対象要素が棚卸し report または manifest に存在する。
2. 採用対象の責務が v0.2 の `core / drivers / entrypoints / sdk / regulated` のどこに属するか明示される。
3. v0.2 仕様に採用理由、禁止境界、崩れる条件が記載される。
4. 外部依存、runtime、I/O、DB、cloud SDK、browser/native 型が core に残らない。
5. 過去の実測や close-like words を v0.2 の成立根拠に流用しない。

v0.1.2 は reference inventory であり、v0.2 current canonical ではありません。v0.1.2 の実装が機能していたことは、v0.2 の動作成立を証明しません。v0.2 は v0.1 の file layout をコピーせず、v0.2 の仕様で再採用された内容のみを採用します。protocol compatibility / deprecation の規範に従い、v0.1 shape を自動互換として扱いません。

## 崩壊条件（Collapse Conditions）

次のいずれかが成立すると、本章の architecture 境界判断は崩れます。

- core が external concrete implementation に依存する。
- drivers が domain rule を所有する。
- entrypoints が composition root を超えて protocol semantics を所有する。
- entrypoints が product-distro として扱われる。
- SDK が Signaling-only 境界を超える。
- regulated が generic communication core の必須依存になる。
- regulated が sdk / drivers / entrypoints に依存する。
- Cargo package dependency graph が architecture dependency direction に違反する。
- feature flag が concrete I/O を core に引き込む。
- SDK が Signaling public contract ではなく internal core/drivers API を import する。
- core module/package 境界が semantic authority ではなく concrete technology で切られる。
- source file size が semantic owner / dependency collapse なしに hard gate として使われる。
- source shard が module scope、public API、owner boundary を変える。
- `core.signaling` が concrete WebSocket / HTTP / entrypoint server type を import する。
- `core.sfu` が str0m packet/event object または driver-owned buffer/cache type を import する。
- `core.turn` が UDP/TCP socket behavior を所有する。
- `core.audit` が concrete DB row shape または persistence driver retry state に依存する。
- `core.reason` が drivers または entrypoints によって拡張される。
- core modules が宣言された aggregate/state owner の外で mutable global state を共有する。
- cross-plane binding が CorrelationId、token subject、network address、ICE username fragment、endpoint name、external user ID だけから推論される。
- line count が semantic owner と dependency boundary より強い証跡として扱われる。
- v0.1.2 の file layout を v0.2 の current canonical として扱う。
- v0.1 開発記録 の close-like words を v0.2 の成立根拠にする。
