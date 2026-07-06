# 概観とスコープ
状態: public summary projection
日付: 2026-07-06 JST

## 目的

本章は arcRTC v0.2 Kernel が「何であり、何でないか」を要約します。Mission、v0.2 が示そうとする価値、Kernel の定義、System Boundary（Kernel root と distro root）、core/drivers/entrypoints/sdk/regulated の責務概要、Non-goals（非目標）の全列挙、主要用語集を提示します。

## Mission

arcRTC v0.2 は、WebRTC 通信基盤の Kernel を DDD（Domain-Driven Design）/ ヘキサゴナルアーキテクチャで再設計し、Signaling / SFU / TURN / SDK / regulated support の責務境界を明確に分離することを目的とします。

v0.2 は v0.1 の延長補修ではありません。DDD / ヘキサゴナルアーキテクチャを前提に、architecture を再固定する系として扱います。Kernel として、`core / drivers / entrypoints + sdk + regulated` の境界を先に固定します。

v0.2 は SFU / TURN / Signaling product system 本体ではありません。Kernel として、意味論上の境界に加え、実通信経路を Kernel-owned evidence で証明するために必要な Kernel-owned drivers と entrypoints を所有します。product deployment と live operation は Kernel authority 外の distro claim です。

## v0.2 の価値

v0.2 の価値は、多機能化ではありません。次の4点を、境界が崩れない構造で示すことに置きます。

| 価値軸 | 意味 |
|---|---|
| Kernel としての成立性 | Kernel が `core / drivers / entrypoints / sdk / regulated` の境界として閉じ、各層の責務が混同されずに固定され、report 済み scope について executable Kernel evidence が存在すること。 |
| 可観測性 | 通信イベント、決定、reason が観測可能であり、相関ID・閉集合 reason・再現可能な手順とともに証跡化できること。 |
| 安定性 | 境界が侵食されず、各層が所有するもの・所有しないものが規範で固定されていること。 |
| 検証可能性 | build / test / runtime verification が証跡として裏付けられ、close / complete / ready を主張する前に Closed Gate Report で裏付けられること。 |

## Kernel とは何か / 何でないか

Kernel とは、arcRTC v0.2 の semantic authority（意味論の権威）を所有する意味論上・機能上の中枢です。Kernel root は `Kernel/` です。この root は、Kernel-owned source、drivers、entrypoints、tests、reports によって評価されます。

Kernel は、Signaling / SFU / TURN の pure semantics（純粋な意味論）を core に置き、binary / I/O から分離した構造体です。Kernel 内の `entrypoints/*-server` は product system ではなく、executable contract（実行可能契約）/ composition evidence surface（合成証跡面）に限定されます。

Kernel は次のものではありません。

- Kernel は `core/` の別名ではありません。`core/` は Kernel の semantic nucleus（意味論の中核）であり、Kernel 全体ではありません。Kernel は core に加えて drivers、entrypoints、sdk、regulated を含みます。
- Kernel は SFU / TURN / Signaling product system 本体ではありません。product deployment と live operation は Kernel 外 distro が所有します。
- Kernel は product deployment、live endpoint operation、monitoring、rollback、production SLO、product-specific topology を所有しません。これらは distro が所有します。

distro は Kernel semantic authority を上書きできません。

## System Boundary

v0.2 の境界は、Kernel root と Kernel 外 distro root の2つで構成されます。

| Boundary | Path | 位置づけ |
|---|---|---|
| Kernel root | `Kernel/` | Kernel authority、Kernel-owned drivers、Kernel-owned entrypoints、report 済み scope の Kernel-owned evidence を所有する。 |
| distro root | `distro/` | Kernel 外 distro の reference distro / product distro を構築する場所。Kernel completion evidence に含めない。 |

Kernel 外 distro は Kernel completion evidence に含めません。distro は Kernel contract / port / SDK projection を利用して SFU / TURN / Signaling reference distro または product distro を構築します。distro は product deployment、live endpoint operation、monitoring、rollback、production SLO、product-specific topology を所有できますが、Kernel semantic authority を上書きできません。

distro root が所有する surface は次のとおりです。

| Surface | 位置づけ |
|---|---|
| `distro/reference-distro/` | reference distro 構築領域 |
| `distro/product-distro/` | product distro 構築領域 |
| `distro/tests/` | distro readiness / production / live の証跡領域 |

Kernel と distro の責務が衝突した場合、Kernel authority を優先します。distro が Kernel contract の変更を要する場合、Kernel 側で versioned 仕様を先に確立しなければならず、distro が Kernel contract を直接改変してはなりません。

Kernel 内の主要境界（正式な構造軸）は次のとおりです。

```text
core <- drivers
core <- entrypoints
drivers <- entrypoints
```

依存方向記法 `A <- B` は「B が A に依存する（B から A を参照可）」を意味します。すなわち drivers と entrypoints が core に依存し、entrypoints が drivers に依存します。`sdk` は独立した Signaling-only 境界です。`regulated` は optional domain support です。境界の許可/禁止の全方向は第02章で内在化します。

## 各層の責務概要

### core

`core/` は arcRTC Kernel の semantic nucleus、中枢、最高権威として中核意味論を保持します。`core/` は Kernel 全体の別名ではありません。

`core/` が所有するもの:

- DDD domain
- application use case
- port
- pure protocol / transport contract
- Signaling contract and state semantics
- SFU routing / quality / backpressure semantics
- TURN allocation / permission / relay semantics
- security primitives
- audit event model / audit port
- quality model / decision logic

`core/` は drivers、entrypoints、framework、runtime、OS、browser、DB、cloud SDK に依存しません。

### drivers

`drivers/` は core が所有する port を実装します。

- str0m driver
- network I/O
- persistence
- observability
- security verifier / key source / secret rotation
- browser boundary
- native boundary
- clock / randomness / runtime gateway

外部型は drivers 境界で core-owned type へ変換します。

### entrypoints

`entrypoints/` は Kernel 内の起動単位、contract probe、composition evidence surface を保持します。

- Signaling executable contract / composition evidence surface
- SFU executable contract / composition evidence surface
- TURN executable contract / composition evidence surface
- CLI
- demo
- configuration / policy bundle wiring
- endpoint / edge trust wiring
- topology / service discovery wiring
- internal control-plane wiring
- health / admin / operator wiring
- dependency injection / wiring

entrypoints は domain rule を所有しません。entrypoints は SFU / TURN / Signaling product distro を所有しません。

### sdk

`sdk/` は Signaling-only 境界として独立します。SDK は Signaling public client contract を利用者向けに公開しますが、regulated support を直接所有しません。

### regulated

`regulated/` は optional domain support です。generic communication core には含めません。`regulated -> core` は仕様で明示された opaque communication event、audit pointer、non-sensitive tag の参照に限定します。`regulated -> drivers`、`regulated -> entrypoints`、`regulated -> sdk`、`core -> regulated`、`drivers -> regulated`、`entrypoints -> regulated`、`sdk -> regulated` は禁止します。

## Non-goals（非目標）

v0.2 初期設計の非目標は次のとおりです。これらは v0.2 が達成しようとしないものであり、1つも落とさず列挙します。

- v0.1 実装の機械的移植
- v0.1 規範の自動継承
- 医療ドメイン機能そのもの
- 医療データ転送
- チャット、録画、画面共有、DataChannel
- 認証基盤または token 発行
- UI / end-user workflow
- deployment 完成形
- SFU / TURN / Signaling product distro
- SFU / TURN / Signaling reference distro の運用
- production readiness
- live readiness

これらの非目標は、初期設計時点での明確な除外対象です。将来採用の可否は、out-of-scope feature の admission / exclusion 規範に従って判断します（その admission の機構詳細は別章が所有します）。

## Current Kernel Acceptance Focus

現在の acceptance focus は次のとおりです。close / complete / ready claim を行う前に、これらの観点は Kernel-owned evidence と Closed Gate Report で裏付けられていなければなりません。

- core / drivers / entrypoints / sdk / regulated の境界が規範で固定されている。
- Signaling / SFU / TURN の semantics が core に置かれ、binary / I/O と分離されている。
- Kernel-owned drivers と entrypoints が、明示的に report 済みの scope について executable evidence を提供する。
- product deployment と live operation は Kernel authority 外であり、Kernel completion evidence として採用しない。
- str0m は drivers 側の port implementation として扱われている。
- regulated が generic communication core へ混入していない。

## 主要用語集

| 用語 | 意味 |
|---|---|
| Kernel | arcRTC v0.2 の semantic authority を所有する意味論上・機能上の中枢。root は `Kernel/`。 |
| distro | Kernel 外で SFU / TURN / Signaling の reference / product distro を構築する領域。root は `distro/`。Kernel completion evidence に含めない。 |
| core | Kernel の semantic nucleus、中枢、最高権威。domain / use case / port / pure protocol / transport contract を所有し、外部 I/O に依存しない。Kernel 全体の別名ではない。 |
| drivers | core が所有する port の実装。external I/O、runtime、network、persistence、observability、security verifier / key source / secret rotation、str0m 接続を所有する。 |
| entrypoints | Kernel 内の起動単位、executable contract、composition evidence surface、dependency wiring。domain rule や product distro を所有しない。 |
| sdk | Signaling-only 独立境界。Signaling public client contract を公開する。regulated support を直接所有しない。 |
| regulated | optional domain support。generic communication core に含めない。`regulated -> core` のみ仕様明示で条件付き許可。 |
| semantic nucleus | core が保持する中核意味論。Kernel 全体の意味論的権威の中心。 |
| semantic authority | 意味論の権威。Kernel が所有し、distro はこれを上書きできない。 |
| Signaling | room state、command semantics、accept / reject boundary を扱う plane。pure semantics は core に置く。 |
| SFU | Selective Forwarding Unit。routing、quality decision、backpressure semantics を扱う plane。pure semantics は core に置く。 |
| TURN | Traversal Using Relays around NAT。allocation、permission、relay semantics を扱う plane。pure semantics は core に置く。 |
| port | core が所有する抽象境界。network / persistence / clock / metrics / audit sink 等。drivers が実装する。 |
| Sans-IO | I/O を含まない純粋な transport contract の設計様式。core/transport が採用する。 |
| executable contract | entrypoints が保持する Kernel 内の実行可能契約面。product system ではない。 |
| composition evidence surface | entrypoints が保持する合成証跡面。dependency wiring の証跡を示す。 |
| reference distro | distro が所有する参照実装。Kernel contract / port / SDK projection を利用する。 |
| product distro | distro が所有する製品実装。deployment / SLO / topology を所有できるが Kernel semantic authority を上書きできない。 |
| Kernel-owned evidence | Kernel source、Kernel drivers、Kernel entrypoints、tests、Kernel `dev-docs/90-reports/` 配下の reports から生成された evidence。 |
| Closed Gate Report | close / complete / resolved / ready 主張の前提として作成する、検証実施範囲の証跡報告。 |
| opaque communication event | regulated が core から参照し得る、内容不透明な通信イベント型。仕様での明示が必要。 |
| audit pointer | regulated が core から参照し得る audit への参照子。仕様での明示が必要。 |
| non-sensitive tag | regulated が core から参照し得る非機微タグ。仕様での明示が必要。 |
