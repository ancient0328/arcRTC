# 概観と適用範囲

状態: SSOT 統合版
日付: 2026-06-28 JST

## 目的

本章は、arcRTC v0.2 の implementations 領域（Kernel 外実装領域）の全体像、すなわち Mission、所有する surface 境界、Kernel に対する境界、適用範囲（In Scope）、適用外範囲（Out of Scope）、固定ゴール（Fixed Goal）、受け入れ焦点（Acceptance Focus）、および用語を、他文書を参照せずに理解できる粒度で確定することを目的とします。本章の規則・語彙・境界はすべて本仕様書内で完結し、外部文書への誘導を含みません。

## Mission

`implementations/` は、凍結済み Kernel contract を利用して SFU / TURN / Signaling の reference implementation / product implementation を構築する Kernel 外実装領域です。

この領域の第一の目的は、凍結済み Kernel contract が、実際に動く SFU / TURN / Signaling の通信基盤を構築するのに十分であることを示すことです。reference implementation は、Kernel の contract が実通信経路へ構成可能であることを、構成的・反証可能・実測可能な形で示す証跡です（実装可能性の証跡であり、完成の主張ではありません）。Kernel を検証しますが、Kernel の権威は所有せず、それ自体は製品でも本番デプロイでもありません。

その上で、Kernel を変更せずに、SFU / TURN / Signaling を組み合わせた reference / product implementation、benchmark、実機 test、production / live readiness の evidence を implementations 側の責務として扱います。次の 5 項目は本領域の評価目標であり、それぞれ専用の evidence がある時にのみ成立します（このコードの存在からは導出されません）。

- production readiness
- live readiness
- benchmark threshold satisfaction
- real-device success
- Kernel completion / freeze

implementations は Kernel の semantic authority を所有しません。implementations は凍結済み Kernel contract の利用者であり、Kernel 側の意味論・port 定義・dependency 方向・freeze claim・final Closed Gate を変更しません。

## System Boundary（implementations が所有する surface 全表）

implementations が所有する surface は次の閉集合です。各 surface はそれぞれの責務に閉じ、責務外を所有しません。

| Surface | implementations が所有する責務 |
|---|---|
| `implementation-support/evidence/` | implementations-local evidence / reason shared type owner |
| `reference-implementation/signaling/` | Kernel contract を使った Signaling reference implementation |
| `reference-implementation/turn/` | Kernel contract を使った TURN reference implementation |
| `reference-implementation/sfu/` | Kernel contract を使った SFU reference implementation |
| `reference-implementation/output/` | reference output outcome types; product input subset は 3-type allow-list |
| `reference-implementation/composition/` | Signaling / TURN / SFU の reference composition |
| `reference-implementation/deployment-profiles/` | reference 実行 profile |
| `reference-implementation/ops/` | reference 運用補助 |
| `product-implementation/signaling/` | product Signaling implementation |
| `product-implementation/turn/` | product TURN implementation |
| `product-implementation/sfu/` | product SFU implementation |
| `product-implementation/product-policy/` | product policy |
| `product-implementation/persistence-topology/` | product persistence topology |
| `product-implementation/deployment/` | product deployment |
| `product-implementation/monitoring/` | product monitoring |
| `product-implementation/rollback/` | product rollback |
| `tests/boundary/` | docs / dependency / export boundary tests |
| `tests/reference/` | reference implementation tests |
| `tests/product/` | product implementation tests |
| `tests/benchmark/` | benchmark scenario / Criterion evidence tests |
| `tests/real-device/` | bounded real-device command evidence wrapper tests |
| `tests/production-readiness/` | production readiness evidence tests |
| `tests/live/` | live readiness evidence tests |

## Kernel Boundary（利用可能項目／非所有項目）

### implementations が利用できる Kernel 項目

implementations は Kernel の次を利用できます。

- public contract
- SDK projection
- core-owned port definition
- driver-facing contract
- entrypoint composition evidence surface の公開済み挙動
- benchmark / test 用の documented command surface

### implementations が所有しない Kernel 項目

implementations は Kernel の次を所有しません。

- core semantics
- Kernel port definition
- Kernel reason catalog
- Kernel dependency direction
- Kernel freeze claim
- Kernel final Closed Gate
- Kernel evidence authority

### Kernel contract 変更が必要になった場合

Kernel contract 変更が必要になった場合、implementations 側では fail-closed とします。すなわち、implementations は Kernel contract を迂回・複製・拡張・上書きしてはなりません（禁止）。Kernel 側の正式な版管理された契約改定が先に必要です（必須）。

## In Scope（15 項目）

implementations の適用範囲は次の 15 項目です。

1. SFU / TURN / Signaling reference implementation
2. SFU / TURN / Signaling product implementation
3. reference composition
4. product composition
5. benchmark execution surface
6. real-device / native command evidence surface
7. production readiness evidence surface
8. live readiness evidence surface
9. implementations-specific CI / command matrix
10. benchmark threshold satisfaction evidence and verdict
11. real-device success evidence and verdict
12. production readiness success evidence and verdict
13. live readiness success evidence and verdict
14. Kernel completion / freeze evidence binding
15. final full fixed-goal evidence and verdict

## Out of Scope（適用外範囲）

次は implementations の適用外であり、禁止です。

- Kernel source modification
- Kernel semantic authority modification
- Kernel evidence を production readiness / live readiness proof として転用すること
- v0.1 behavior inheritance claim
- benchmark threshold を測定後に成功へ合わせて設定すること
- implementations の completion claim を専用 evidence と verdict なしに行うこと
- production readiness / live readiness / benchmark threshold satisfaction / real-device success / Kernel completion / freeze のいずれかを、専用 evidence と verdict なしに final fixed goal へ採用すること

## Fixed Goal（固定ゴール 5 項目）

implementations の fixed goal は、本領域が目指す次の 5 つの評価目標を示します。これらは達成済みではなく、目標です。

1. production readiness
2. live readiness
3. benchmark threshold satisfaction
4. real-device success
5. Kernel completion / freeze

この fixed goal は、凍結済み Kernel contract を侵食せず、SFU / TURN / Signaling の reference implementation / product implementation を Kernel 外で構築したうえで、5 項目すべてを専用 evidence と verdict に接続します。各項目は専用 evidence と verdict を欠いたまま final fixed goal へ採用してはなりません（禁止／fail-closed）。

## Acceptance Focus（受け入れ焦点）

初期 authority setup の acceptance focus は次です。

- implementations 領域が Kernel から分離されている。
- SFU / TURN / Signaling が implementations の対象として明示されている。
- reference implementation と product implementation が分離されている。
- shared evidence / reason owner が reference / product のどちらにも偏らず分離されている。
- product implementation が消費できる reference surface は `arcrtc-reference-output` の `ReferenceSignalingOutcome` / `ReferenceTurnOutcome` / `ReferenceSfuOutcome` に限定されている。
- implementation source と test source が分離されている。
- Kernel 修正を原則禁止し、必要時は Kernel 側の正式な版管理された契約改定を要求する。
- production readiness / live readiness を Kernel evidence から自動導出せず、専用 gate で成立させる。
- close-like claim は専用 evidence と verdict なしに行わない。
- full fixed goal の達成は、source closeout / test closeout / benchmark threshold satisfaction / real-device success / production readiness / live readiness / Kernel completion freeze evidence を final full fixed-goal verdict に接続する。

## Governance 最上位境界

本領域の運用上の最上位境界は次です。これらは implementations と Kernel の authority 分離を保証する不変条件です。

- implementations は Kernel frozen contract を利用する。
- implementations は Kernel semantic authority を変更しない（禁止）。
- implementations は SFU / TURN / Signaling reference implementation / product implementation を所有する。
- implementations の production readiness、live readiness、native application readiness、public distribution readiness は Kernel evidence から自動導出しない（禁止）。
- Kernel contract 変更が必要な場合、implementations 側では fail-closed とし、Kernel 側の正式な版管理された契約改定を先に必要とする。

## 完了主張の前提（fail-closed）

complete、done、fixed、resolved、closed、ready、production-ready、live-ready、executable-ready、no findings を、対象 scope の専用 evidence と verdict を接続せずに主張してはなりません。対象 scope の必須条件のうち Yes と明確に言えないものが 1 つでもある場合、対象 claim を close / complete / ready と扱いません（fail-closed）。

## evidence record の必須 field

implementations 側の evidence は、次を欠く場合に evidence として受理しません（fail-closed）。

- 相関ID
- command
- working directory
- target package（evidence schema が要求する場合）
- target scope
- expected outcome
- actual outcome
- environment / toolchain
- rerun condition
- reason の閉集合
- UNKNOWN 不使用
- non-claim scope

correlation id と index 接続だけを示す evidence は、command result、test pass、readiness、completion、no findings の full evidence として受理しません（禁止）。実測 evidence は本仕様書が固定する契約・規則の代替 authority ではありません。

## authority surface の存在自体の non-claim scope

implementations の authority surface の存在それ自体は、次を一切主張しません。

- reference implementation completion
- product implementation completion
- production readiness
- live readiness
- native application readiness
- public distribution readiness
- benchmark threshold satisfaction
- Kernel completion / freeze の再判定

## 用語集

| 用語 | 意味 |
|---|---|
| Kernel | `Kernel/` に閉じる凍結対象領域。semantic authority、contract、port ownership、freeze claim、最終 close-out gate を所有する。implementations はこれを変更しない。 |
| implementations | `implementations/`。Kernel 外で SFU / TURN / Signaling の reference / product implementation を構築する実装領域。 |
| Kernel contract | implementations が利用できる Kernel の public contract / SDK projection / core-owned port definition / driver-facing contract / documented command surface の総称。 |
| reference implementation | Kernel frozen contract を使って SFU / TURN / Signaling を最小構成で組み合わせる実装層。production readiness を主張しない。 |
| product implementation | `arcrtc-reference-output` の 3 outcome 型だけを入力として消費し、product policy / deployment / monitoring / persistence topology / rollback を別 surface で所有する実装層。 |
| `arcrtc-reference-output` | reference output outcome types を所有する package。product input subset は `ReferenceSignalingOutcome` / `ReferenceTurnOutcome` / `ReferenceSfuOutcome` の 3-type allow-list。 |
| `ReferenceSignalingOutcome` / `ReferenceTurnOutcome` / `ReferenceSfuOutcome` | product implementation が入力として消費できる唯一の reference outcome 型（3-type allow-list）。 |
| `ReferenceCompositionOutcome` | reference composition の出力型。product implementation は入力として消費しない。 |
| benchmark threshold satisfaction | 測定後に threshold を成功へ合わせず、専用 evidence と verdict で benchmark threshold を満たすこと。 |
| real-device success | bounded な Android / iOS / browser 実機 command の成功 evidence と verdict。general live readiness を意味しない。 |
| production readiness | implementations 側の専用 evidence と verdict でのみ成立する production 受け入れ状態。Kernel evidence から自動導出しない。 |
| live readiness | implementations 側の専用 evidence と verdict でのみ成立する live 受け入れ状態。real-device command evidence とは別。 |
| Kernel completion / freeze | Kernel 側で確定する完了・凍結状態。implementations は再判定せず、evidence を必須入力として binding する。 |
| fail-closed | 必要条件を満たさない限り成功・完了・close を主張せず停止する原則。 |
| evidence | 相関ID・command・working directory 等の必須項目を満たし、UNKNOWN を含まない採用可能な実測証跡。 |
| fixed goal | production readiness / live readiness / benchmark threshold satisfaction / real-device success / Kernel completion freeze の 5 項目すべての達成。 |
