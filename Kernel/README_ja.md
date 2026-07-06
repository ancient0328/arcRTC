# arcRTC v0.2 Kernel

arcRTC v0.2 Kernel は、WebRTC 通信意味論を確認するための Kernel リポジトリです。

Signaling、SFU、TURN、SDK public API projection、regulated support の Kernel-owned な責務境界を定義し、実装し、検証します。

このリポジトリは、本番デプロイ対象ではなく、Kernel-owned な通信意味論と runtime evidence の surface として評価します。

完成済みの WebRTC サービス、production Signaling server、SFU、TURN server、native SDK release として提示するものではありません。

English version: [README.md](README.md).

リポジトリを確認する場合は [評価範囲](#評価範囲) から読み、先に現在の確認コマンドを実行する場合は [クイックチェック](#クイックチェック) から始めてください。

## 概要

arcRTC v0.2 Kernel は、DDD とヘキサゴナルアーキテクチャに基づく WebRTC 通信 Kernel です。

主に扱うもの:

- プロトコルと通信意味論、
- command、event、decision、result の形、
- 閉じた reason と failure vocabulary、
- state と lifecycle の境界、
- port ownership と driver conversion、
- executable composition boundary、
- Signaling-only SDK 投影、
- evidence-scoped verification。

このリポジトリは、Kernel の意味論上の境界と verification scope を提示します。product-distro の出力は Kernel authority の外側にあり、Kernel completion evidence としては採用しません。Kernel の実装可能性は、この tree 内の Kernel-owned drivers、entrypoints、実 socket/datagram 交換、`dev-docs/90-reports/` 配下の report によって評価します。

## 設計方針

リアルタイム通信システムでは、製品挙動、ネットワーク I/O、実行時ライブラリ、プラットフォーム SDK、ベンチマーク、デプロイ前提が同時に意味を定義し始めると、設計の確認が難しくなります。

arcRTC v0.2 Kernel は、本番実装に入る前に通信モデルを確認できるようにするための設計基盤です。プロトコル上の意味をどの層が所有するか、外部観測をどこで変換するか、executable contract をどこで組み立てるか、現在の evidence でどの範囲まで主張できるかを確認する入口になります。

## 想定読者

- WebRTC Signaling、SFU、TURN、SDK の境界設計を確認するエンジニア。
- Kernel authority を所有せずに reference distro または product distro から Kernel contract を利用するエンジニア。
- 意味論の所有関係が保たれているか確認する maintainer。
- 現在の Kernel evidence と scope への公開入口が必要な reviewer。

## クイックチェック

リポジトリを clone し、Kernel の Rust workspace checks を実行します。

```sh
git clone https://github.com/ancient0328/arcRTC.git
cd arcRTC/Kernel
cargo build --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
```

これらのコマンドは Kernel scope だけを確認します。

## アーキテクチャモデル

| 概念 | 公開上の意味 | 境界 |
|---|---|---|
| 中核意味論 | プロトコル上の意味、domain rule、command、result、reason、state、port を所有する | concrete I/O、runtime library、platform API、database、deployment policy は所有しない |
| ドライバ境界 | 外部観測を Kernel-owned command、observation、intent、closed failure へ変換する | domain meaning や product policy は定義しない |
| エントリポイント構成 | dependency を組み立て、executable Kernel contract surface を公開する | product deployment proof にはならない |
| SDK 投影 | Signaling-only public API shape を公開する | media、auth issuance、regulated workflow、driver internals は所有しない |
| Regulated support | optional support projection と non-sensitive enrichment を提供する | generic communication core にはならない |
| External distro | Kernel authority の外側で参照挙動または製品挙動を構築する | Kernel completion evidence を供給しない |

概念上の依存方向:

```text
Core <- Drivers
Core <- Entrypoints
Drivers <- Entrypoints
```

## 評価範囲

公開 summary projection は `docs/summary/` 以下にあり、英語版が `en/`、日本語版が `ja/` です。索引とそこに挙がる各章を読むことで、Kernel の意図・構造・契約・運用・検証・トラブルシューティングの概要を把握できます。completion と evidence の現在の authority は `dev-docs/` にあります。

| 評価対象 | 確認する内容 | 公開 summary |
|---|---|---|
| summary 索引 | 章構成、summary 原則、用語、claim / non-claim 範囲 | [サマリー索引（日本語）](docs/summary/ja/00-INDEX.md) · [English](docs/summary/en/00-INDEX.md) |
| 範囲と境界 | Mission、System Boundary、Non-goals、層モデル、依存方向 | [01 概観](docs/summary/ja/01-overview-and-scope.md) · [02 アーキテクチャ](docs/summary/ja/02-architecture-and-boundaries.md) |
| Core 意味論 | domain、command/reason、ports、protocol、transport/media、Signaling/SFU/TURN | [03〜13](docs/summary/ja/00-INDEX.md) |
| Drivers / entrypoints | port 実装、変換、composition、運用、topology | [14〜21](docs/summary/ja/00-INDEX.md) |
| SDK / regulated / packaging | Signaling-only projection、optional support、supply chain、release | [22〜24](docs/summary/ja/00-INDEX.md) |
| 検証 | CI quality gate、testing evidence、benchmark scope | [25〜27](docs/summary/ja/00-INDEX.md) |
| トラブルシューティング | 横断的な失敗モード、fail-closed 運用、境界違反の是正 | [28](docs/summary/ja/28-troubleshooting.md) |

## 対象外の範囲

現在の claim は、対応する report を持つ Kernel-owned semantics と Kernel-owned evidence に限定されます。

このリポジトリは次を主張しません。

- production readiness、
- live readiness、
- native application readiness、
- 完成済みの SFU、TURN、Signaling product-distro build、
- benchmark acceptance threshold satisfaction、
- v0.1 behavior の自動継承。

benchmark scenario は measurement と reportability の surface です。別途 threshold rule が明示されていない限り、benchmark output は scoped measurement として扱います。

## Summary Projection

公開 summary projection は `docs/summary/` 以下にあり、英語版を `en/`、日本語版を `ja/` に置いています（章構成は同一）。これは現在の Kernel 構造と evidence scope への orientation surface であり、`dev-docs/` authority や source-level verification の代替ではありません。

推奨読順（日本語版。英語版も同一構成です）:

1. [00 索引](docs/summary/ja/00-INDEX.md)
2. [01 概観と範囲](docs/summary/ja/01-overview-and-scope.md)
3. [02 アーキテクチャと境界](docs/summary/ja/02-architecture-and-boundaries.md)
4. Core 各章 [03〜13]、続いて drivers/entrypoints [14〜21]
5. SDK / regulated / packaging [22〜24] と検証 [25〜27]
6. [28 トラブルシューティング](docs/summary/ja/28-troubleshooting.md)

## 変更提案時の注意

この README は public support guarantee を定義しません。

変更を提案する場合は、上記の ownership boundary を維持してください。特に、protocol meaning を driver code や entrypoint code へ移動してはいけません。また、別途 evidence と project approval がない状態で、executable contract surface を product-readiness claim に変えてはいけません。

## ライセンスと公開状態

この README は license rights、package publication guarantees、production support、service availability を付与しません。再配布または production use の前に、repository metadata と明示的な license file を確認してください。
