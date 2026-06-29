# arcRTC v0.2

arcRTC v0.2 は、WebRTC の通信セマンティクスを扱うプロジェクトであり、2 つの部分から成ります。
通信契約を定義し凍結する **Kernel** と、その凍結契約の上に SFU / TURN / Signaling の
リファレンス実装を構築する **implementations** です。

本リポジトリは「セマンティック境界」と「その構成的証跡」として編成されています。
バンドル済み WebRTC サービス、本番 Signaling / SFU / TURN サーバー、ネイティブ SDK
リリースではありません。

English version: [README.md](README.md).

## 構成

- [`Kernel/`](Kernel/README_ja.md) — WebRTC 通信カーネル。プロトコルと
  command / event / decision / result の形、閉じた reason・失敗語彙、状態とライフサイクル
  境界、port 所有と driver 変換、実行可能な composition 境界、Signaling 限定の SDK 投影、
  証跡スコープの検証を定義します。DDD とヘキサゴナルアーキテクチャで編成されています。
- [`implementations/`](implementations/README_ja.md) — Kernel の外側のトラック。凍結された
  Kernel 契約を消費し、実行可能な Signaling / TURN / SFU のリファレンス経路、composition
  例、ベンチマークハーネス、product 実装トラックへと具体化します。Kernel のセマンティクスを
  改変・所有しません。

`implementations` は `Kernel` に依存します。`Kernel` は `implementations` に依存しません。

## スコープ

Kernel は安定したセマンティック境界を提示します。implementations トラックは実験的・
リファレンス志向です。リファレンス実装は product 実装ではなく、ベンチマーク結果は
production-readiness の主張ではありません。本プロジェクトは production readiness、
臨床・規制下での deployment、managed-RTC-SDK の置き換えを主張しません。

## ツールチェーン

Rust 1.96.0（[`rust-toolchain.toml`](rust-toolchain.toml) で固定）。

## クイックチェック

```sh
# Kernel
cd Kernel
cargo build --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets

# implementations（reference / product のソース）
cd ../implementations
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings

# implementations の挙動テスト（各 area が独立 workspace）
for area in reference product benchmark real-device live production-readiness boundary; do
  ( cd tests/"$area" && cargo test )
done
```

## ライセンス

Apache License, Version 2.0 の下で公開されています。[LICENSE](LICENSE) を参照してください。

## セキュリティ

脆弱性の報告は [SECURITY.md](SECURITY.md) を参照してください。
