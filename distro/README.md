# arcRTC v0.2 distro

arcRTC v0.2 distro is the out-of-Kernel area that builds the SFU / TURN / Signaling reference distro and product distro on top of the frozen arcRTC v0.2 Kernel contract.

It consumes the Kernel public contract, SDK projection, and documented command surface, and does not modify or own the Kernel's semantics. It is not a bundled WebRTC service or a finished production deployment.

日本語版は [README_ja.md](README_ja.md) を参照してください。

Start with [Specification](#specification-ssot) if you want the full description, or [Quick Check](#quick-check) if you want to run the current checks first.

## Purpose of distro

The distro track turns the frozen Kernel contract into runnable reference communication paths. It exists to show that the Kernel can be implemented, exercised, and measured — through reference Signaling / TURN / SFU paths, a composition example, and a benchmark harness. In short, it is constructive evidence of Kernel implementability.

- **reference distro** — shows the Kernel contract runs as working Signaling / TURN / SFU paths.
- **benchmark** — makes that behavior reproducible and measurable.
- **product distro** — a future productization track; its presence is not a readiness claim.

Reference distro are not product distro, and benchmark results are not production-readiness claims. The current scope is experimental and reference-oriented; it does not claim production readiness, clinical / regulated deployment, or a managed-RTC-SDK replacement.

## Overview

distro is organized as four layers that sit outside the Kernel and depend on it.

- `distro-support/` — shared distro-local evidence / reason types.
- `reference-distro/` — the minimal SFU / TURN / Signaling distro built on the frozen Kernel contract (signaling / turn / sfu / output / composition / deployment-profiles / ops).
- `product-distro/` — the product runtime that consumes only the reference output allow-list, and owns product policy / persistence topology / deployment / monitoring / rollback.
- `tests/` — reference / product / benchmark / real-device / production-readiness / live evidence.

The reference layer makes no production-readiness claim; the product layer establishes production / live readiness only through its own dedicated evidence.

## Design Rationale

Keeping the Kernel frozen lets the distro evolve without eroding the communication semantics. distro exists to build a working SFU / TURN / Signaling stack on top of that frozen contract, pin to a specific Kernel source snapshot, and keep production concerns (policy, deployment, monitoring, rollback) clearly separated from the reference behavior.

## Intended Audience

- Engineers building a reference or product SFU / TURN / Signaling stack on the Kernel contract.
- Engineers integrating product policy, persistence, deployment, monitoring, and rollback.
- Maintainers checking that the Kernel contract is consumed without modification.
- Reviewers who need a self-contained description of the distro area.

## Quick Check

From this directory (`distro/`), run the Rust workspace checks:

```sh
cargo build --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
```

These commands verify the distro scope only.

## Architecture Model

| Layer | Responsibility | Boundary |
|---|---|---|
| distro-support | Shared evidence / reason types for reference, product, and tests | Does not own Kernel semantics, product policy, or readiness claims |
| reference-distro | Minimal SFU / TURN / Signaling built on the frozen Kernel contract | Makes no production-readiness claim |
| product-distro | Product runtime, policy, persistence, deployment, monitoring, rollback | Consumes only the reference output allow-list; does not fork the reference into product policy |
| tests | Reference / product / benchmark / real-device / readiness evidence | Evidence is not a substitute authority for the Kernel contract |

The dependency direction is:

```text
distro -> Kernel public contract
distro -> Kernel SDK projection
distro -> documented Kernel command surface

Kernel  -X-> distro
distro -X-> Kernel semantic authority overwrite
```

## Specification (SSOT)

The complete, self-contained specification (SSOT) lives under `docs/summary/`, with an English edition in `en/` and a Japanese edition in `ja/` (identical chapter structure). It is the current, complete specification — not a development history — and is written so an engineer can re-implement an equivalent from it without reading the source tree.

| Area | What to check | Specification (SSOT) |
|---|---|---|
| Index and scope | Chapter map, SSOT principles, claim / non-claim | [Summary index (EN)](docs/summary/en/00-INDEX.md) · [日本語](docs/summary/ja/00-INDEX.md) |
| Overview and architecture | Mission, System Boundary, layers, dependency rule | [01 Overview](docs/summary/en/01-overview-and-scope.md) · [02 Architecture](docs/summary/en/02-architecture-and-boundaries.md) |
| Kernel consumption / toolchain | Contract import, version pin, toolchain, dependency admission | [03–04](docs/summary/en/00-INDEX.md) |
| Evidence / state / reference | Evidence records, state & security, reference design and runtime | [05–08](docs/summary/en/00-INDEX.md) |
| Product / benchmark / real-device | Product distro, benchmark, real-device | [09–11](docs/summary/en/00-INDEX.md) |
| Readiness / testing / CI | Production-live readiness requirements, testing, CI gates | [12–14](docs/summary/en/00-INDEX.md) |
| Troubleshooting | Cross-cutting failure modes and remediation | [15 Troubleshooting](docs/summary/en/15-troubleshooting.md) |

## Scope Limits

This repository establishes the following only through dedicated evidence; none is implied by the existence of the source or by Kernel evidence:

- reference / product distro completion,
- production readiness,
- live readiness,
- native application readiness,
- benchmark threshold satisfaction,
- real-device success,
- re-adjudication of Kernel completion / freeze,
- automatic inheritance of v0.1 behavior.

## Change Guidance

When proposing changes, preserve the boundaries described above. In particular, do not modify the Kernel contract from here (a versioned contract revision on the Kernel side is required first), do not let the product layer consume anything beyond the reference output allow-list, and do not treat reference behavior or Kernel evidence as a production / live readiness claim.

## License and Publication

This README does not grant license rights, package publication guarantees, production support, or service availability. Check repository metadata and explicit license files before redistribution or production use.
