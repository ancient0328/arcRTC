# arcRTC v0.2

arcRTC v0.2 is a WebRTC communication-semantics project in two parts: a **Kernel**
that defines and freezes the communication contract, and **distro** that
build a reference SFU / TURN / Signaling stack on top of that frozen contract.

This repository is organized as a semantic boundary and its constructive evidence —
not as a bundled WebRTC service, a production Signaling / SFU / TURN server, or a
native SDK release.

日本語版は [README_ja.md](README_ja.md) を参照してください。

## Layout

- [`Kernel/`](Kernel/README.md) — the WebRTC communication kernel: protocol and
  command / event / decision / result shape, closed reason and failure vocabulary,
  state and lifecycle boundaries, port ownership and driver conversion, executable
  composition boundaries, Signaling-only SDK projection, and evidence-scoped
  verification. Organized around DDD and hexagonal architecture.
- [`distro/`](distro/README.md) — the out-of-Kernel track that
  consumes the frozen Kernel contract and turns it into runnable reference
  Signaling / TURN / SFU paths, a composition example, a benchmark harness, and a
  product-distro track. It does not modify or own Kernel semantics.

`distro` depends on `Kernel`; `Kernel` never depends on `distro`.

## Scope

The Kernel presents a stable semantic boundary; the distro track is
experimental and reference-oriented. Reference-distro output is not
product-distro output, and benchmark results are not production-readiness claims. This
project does not claim production readiness, clinical / regulated deployment, or a
managed-RTC-SDK replacement.

## Toolchain

Rust 1.96.0, pinned via [`rust-toolchain.toml`](rust-toolchain.toml).

## Quick check

```sh
# Kernel
cd Kernel
cargo build --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets

# distro (reference / product source)
cd ../distro
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings

# distro behavior tests (each area is its own workspace)
for area in reference product benchmark real-device live production-readiness boundary; do
  ( cd tests/"$area" && cargo test )
done
```

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE).

## Security

To report a vulnerability, see [SECURITY.md](SECURITY.md).
