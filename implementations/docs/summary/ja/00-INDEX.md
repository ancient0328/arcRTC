# arcRTC v0.2 implementations システム／開発サマリー（SSOT・日本語版）

状態: SSOT 統合版
日付: 2026-06-28 JST
対象: `implementations/`

## このサマリーの位置づけ

本サマリー（`docs/summary/ja/` 配下の全章）は、arcRTC v0.2 implementations の **Single Source of Truth (SSOT)** として成立する、自己完結の仕様書・説明書です。
本サマリーだけを読めば、他のファイルや実コードを参照しなくても、implementations の意図・目的・理念・構造・経路・性質・運用・検証・トラブルシューティングのすべてが分かることを目的とします。

implementations は、**凍結済み Kernel contract を利用して** SFU / TURN / Signaling の reference implementation / product implementation を構築する Kernel 外実装領域です。第一の目的は、凍結 Kernel contract が実際に動く通信基盤を構築するのに十分であることを示すこと——reference implementation は、Kernel の contract が実通信経路へ構成可能であることを、構成的・反証可能・実測可能に示す証跡です（実装可能性の証跡であり、完成の主張ではありません）。Kernel を検証する層であって、それ自体は製品ではありません。production readiness・live readiness・benchmark threshold satisfaction・real-device success・Kernel completion / freeze の5項目は本領域の評価目標であり、それぞれ専用の evidence がある時にのみ成立します（コードの存在からは導出されません）。

本サマリーは「作成経過の歴史書」ではなく「現在の完全な仕様書」です。失敗・紆余曲折・退役文書の経緯は含めません。

### SSOT 原則（全章共通）

- 本サマリーは外部文書（ソースツリー、Kernel 文書、実コード）への誘導や引用に依存しません。必要な規範内容はすべて各章本文に内在化します。
- 章どうしの相互参照は、同一 SSOT 内のナビゲーション（章番号）に限ります。各章は自己完結を保ちます。
- 規範語: 「必須（MUST）」「禁止（MUST NOT）」「許可（MAY）」「fail-closed（曖昧・閉集合外は不採用＝拒否側に倒す）」を一貫して用います。
- 用語: 技術固有名詞（Kernel / reference implementation / product implementation / evidence / reason / readiness / benchmark / real-device 等）は英語表記のまま保持し、説明は日本語丁寧語で記します。

### Kernel との関係（依存規則）

```text
implementations -> Kernel public contract
implementations -> Kernel SDK projection
implementations -> documented Kernel command surface

Kernel  -X-> implementations
implementations -X-> Kernel semantic authority overwrite
```

implementations は Kernel の core semantics・port definition・reason catalog・dependency direction・freeze claim・final acceptance authority・evidence authority を**所有しません**。Kernel contract の変更が必要になった場合は fail-closed とし、先に Kernel 側の versioned contract revision を要します。

## 章構成

| 章 | 内容 |
|---|---|
| 00-INDEX | 本索引・SSOT 原則・用語・依存規則・claim/non-claim |
| 01-overview-and-scope | Mission、System Boundary、Kernel Boundary、In/Out scope、Fixed Goal、Acceptance Focus、用語集 |
| 02-architecture-and-boundaries | root model、層（implementation-support/reference/product/tests）、依存規則、workspace manifest/dependency path、module export、source/package 境界、branch identity |
| 03-kernel-contract-consumption | Kernel contract import、version pin、builder mapping、Kernel completion/freeze binding |
| 04-toolchain-runtime-dependency | 実装言語/toolchain、runtime、dependency admission、runtime/executor admission |
| 05-evidence-reason-and-observability | evidence reason ownership、observability evidence reason、error reason mapping、evidence schema、evidence wire format、evidence extension record |
| 06-state-and-security | state persistence、identity/auth/security |
| 07-reference-implementation-design | reference implementation 構造、signaling/turn/sfu/composition design、runtime topology、transport、configuration profile、composition runtime bridge |
| 08-reference-runtime-state-output | reference phase & fixture payload、state mutation、API & state、output boundary（3-type allow-list）、runtime lifecycle、command evidence runner |
| 09-product-implementation | product implementation、product API policy、plane API signature、product admission |
| 10-benchmark | benchmark scenario workload、harness layout、benchmark & real-device evidence、threshold satisfaction、comparability |
| 11-real-device | real-device command matrix、wrapper command、real-device success、scope |
| 12-readiness-production-live | readiness matrix、production provider admission、live endpoint traversal、Kernel production implementability（KPI-001〜KPI-015） |
| 13-testing-and-assertions | test package & assertion、test workspace manifest |
| 14-ci-quality-gates | command CI quality gate、quality gate state |
| 15-troubleshooting | 横断的な失敗モード、fail-closed 運用、境界違反の検知と是正 |

## 現在の受容範囲（claim / non-claim）

- claim（採用済み）: Kernel から分離された implementations authority、reference/product implementation の境界分離、shared evidence/reason owner の分離、Kernel 修正の原則禁止。
- non-claim（不主張・専用 evidence と Closed Gate でのみ成立）: reference/product implementation completion、production readiness、live readiness、native application readiness、benchmark threshold satisfaction、real-device success、Kernel completion/freeze の再判定、v0.1 behavior 継承。

## 英語版

同一構造の英語版は `docs/summary/en/` 配下にあります（章番号・章名は対応）。
