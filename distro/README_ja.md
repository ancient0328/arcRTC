# arcRTC v0.2 distro

arcRTC v0.2 distro は、凍結済みの arcRTC v0.2 Kernel contract の上に、SFU / TURN / Signaling の reference distro と product distro を構築する Kernel 外の領域です。

Kernel の public contract・SDK projection・documented command surface を利用するだけで、Kernel の semantics を変更も所有もしません。完成済みの WebRTC サービスや本番デプロイ完成形ではありません。

English version: [README.md](README.md).

全体像は [仕様書（SSOT）](#仕様書ssot)、先に現在の確認コマンドを実行する場合は [クイックチェック](#クイックチェック) から始めてください。

## distro の目的

distro は、凍結済み Kernel contract を、実際に動く reference の通信経路へと具体化する層です。reference の Signaling / TURN / SFU 経路、composition example、benchmark harness を通じて、Kernel が実装・実行・測定できることを示します。要するに、Kernel の実装可能性に関する構成的な証跡です。

- **reference distro** — Kernel contract が動く Signaling / TURN / SFU 経路として成立することを示す。
- **benchmark** — その振る舞いを再現可能・測定可能にする。
- **product distro** — 将来の製品化トラックであり、その存在は readiness 主張ではない。

reference distro は product distro ではなく、benchmark 結果は production-readiness の主張ではありません。現在の公開範囲は実験的・reference 中心であり、production readiness・clinical / regulated deployment・managed RTC SDK の置換は主張しません。

## 概要

distro は、Kernel の外側にあって Kernel に依存する4層で構成されます。

- `distro-support/` — reference / product / tests が共有する distro-local な evidence / reason 型。
- `reference-distro/` — 凍結 Kernel contract 上に構築する最小の SFU / TURN / Signaling 実装（signaling / turn / sfu / output / composition / deployment-profiles / ops）。
- `product-distro/` — reference の output allow-list のみを入力に消費する product runtime。product policy / persistence topology / deployment / monitoring / rollback を所有。
- `tests/` — reference / product / benchmark / real-device / production-readiness / live の証跡。

reference 層は production readiness を主張しません。product 層の production / live readiness は、それ専用の証跡によってのみ成立します。

## 設計方針

Kernel を凍結したまま実装を進化させることで、通信意味論を侵食せずに実装を発展させられます。distro は、その凍結 contract 上で動く SFU / TURN / Signaling スタックを構築し、特定の Kernel source snapshot に pin し、本番関連の関心事（policy・deployment・monitoring・rollback）を reference 挙動から明確に分離するために存在します。

## 想定読者

- Kernel contract 上に reference / product の SFU / TURN / Signaling スタックを構築するエンジニア。
- product policy・persistence・deployment・monitoring・rollback を統合するエンジニア。
- Kernel contract を変更せずに利用しているか確認する maintainer。
- 実装領域の自己完結な説明を必要とする reviewer。

## クイックチェック

このディレクトリ（`distro/`）で Rust workspace の確認を実行します。

```sh
cargo build --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
```

これらのコマンドは distro scope だけを確認します。

## アーキテクチャモデル

| 層 | 責務 | 境界 |
|---|---|---|
| distro-support | reference / product / tests が共有する evidence / reason 型 | Kernel semantics・product policy・readiness claim は所有しない |
| reference-distro | 凍結 Kernel contract 上の最小 SFU / TURN / Signaling 実装 | production readiness を主張しない |
| product-distro | product runtime・policy・persistence・deployment・monitoring・rollback | reference の output allow-list のみ消費し、reference を product policy に fork しない |
| tests | reference / product / benchmark / real-device / readiness の証跡 | 証跡は Kernel contract に対する代替権威にならない |

依存方向:

```text
distro -> Kernel public contract
distro -> Kernel SDK projection
distro -> documented Kernel command surface

Kernel  -X-> distro
distro -X-> Kernel semantic authority overwrite
```

## 仕様書（SSOT）

完全かつ自己完結な仕様書（SSOT）は `docs/summary/` 以下にあり、英語版が `en/`、日本語版が `ja/` です（章構成は同一）。開発の歴史ではなく現在の完全な仕様書であり、ソースツリーを読まずとも同等物を再実装できる粒度で記述しています。

| 領域 | 確認する内容 | 仕様書（SSOT） |
|---|---|---|
| 索引と範囲 | 章構成、SSOT 原則、claim / non-claim | [サマリー索引（日本語）](docs/summary/ja/00-INDEX.md) · [English](docs/summary/en/00-INDEX.md) |
| 概観とアーキ | Mission、System Boundary、層、依存規則 | [01 概観](docs/summary/ja/01-overview-and-scope.md) · [02 アーキテクチャ](docs/summary/ja/02-architecture-and-boundaries.md) |
| Kernel 消費 / toolchain | contract import、version pin、toolchain、dependency admission | [03〜04](docs/summary/ja/00-INDEX.md) |
| evidence / state / reference | evidence record、state & security、reference design / runtime | [05〜08](docs/summary/ja/00-INDEX.md) |
| product / benchmark / real-device | product distro、benchmark、real-device | [09〜11](docs/summary/ja/00-INDEX.md) |
| readiness / testing / CI | production-live readiness 要件、testing、CI gate | [12〜14](docs/summary/ja/00-INDEX.md) |
| トラブルシューティング | 横断的な失敗モードと是正 | [15 トラブルシューティング](docs/summary/ja/15-troubleshooting.md) |

## 対象外の範囲

次は、それぞれ専用の証跡によってのみ成立します。ソースの存在や Kernel 証跡からは導出しません。

- reference / product distro の完成、
- production readiness、
- live readiness、
- native application readiness、
- benchmark threshold satisfaction、
- real-device success、
- Kernel completion / freeze の再判定、
- v0.1 behavior の自動継承。

## 変更提案時の注意

変更を提案する場合は、上記の境界を維持してください。特に、ここから Kernel contract を変更しないこと（先に Kernel 側の versioned contract revision が必要）、product 層が reference の output allow-list を超えて消費しないこと、reference 挙動や Kernel 証跡を production / live readiness の主張に転用しないこと。

## ライセンスと公開状態

この README は license rights、package publication guarantees、production support、service availability を付与しません。再配布または production use の前に、repository metadata と明示的な license file を確認してください。
