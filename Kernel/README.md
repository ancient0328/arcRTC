# arcRTC v0.2 Kernel

arcRTC v0.2 Kernel is a WebRTC communication-semantics repository.

It defines, implements, and verifies the Kernel-owned responsibility boundaries for Signaling, SFU, TURN, SDK public API projection, and regulated support.

This repository is evaluated as a Kernel-owned communication semantics and runtime evidence surface, not as a production deployment target.

It is not a bundled WebRTC service, production Signaling server, SFU, TURN server, or native SDK release.

日本語版は [README_ja.md](README_ja.md) を参照してください。

Start with [Evaluation Scope](#evaluation-scope) if you are reviewing the repository, or [Quick Check](#quick-check) if you want to run the current checks first.

## Overview

arcRTC v0.2 Kernel is a WebRTC communication kernel organized around DDD and hexagonal architecture.

It focuses on:

- protocol and communication semantics,
- command, event, decision, and result shape,
- closed reason and failure vocabulary,
- state and lifecycle boundaries,
- port ownership and driver conversion,
- executable composition boundaries,
- Signaling-only SDK projection,
- evidence-scoped verification.

The repository presents the Kernel's semantic boundary and verification scope. Product-distro output is outside Kernel authority and is not used as Kernel completion evidence. Kernel implementability is evaluated inside this tree through Kernel-owned drivers, entrypoints, real socket or datagram exchanges, and reports under `dev-docs/90-reports/`.

## Design Rationale

Realtime communication systems become hard to review when product behavior, network I/O, runtime libraries, platform SDKs, benchmarks, and deployment assumptions all define meaning at the same time.

arcRTC v0.2 Kernel exists to make the communication model reviewable before production implementation work begins. It gives engineers a place to inspect who owns protocol meaning, where external observations are converted, which layer wires executable contracts, and which claims are supported by current evidence.

## Intended Audience

- Engineers reviewing WebRTC Signaling, SFU, TURN, and SDK boundary design.
- Engineers consuming Kernel contracts from reference or product distro without owning Kernel authority.
- Maintainers checking whether a change preserves semantic ownership.
- Reviewers who need a public entrypoint into the current Kernel evidence and scope.

## Quick Check

Clone the repository and run the Kernel Rust workspace checks:

```sh
git clone https://github.com/ancient0328/arcRTC.git
cd arcRTC/Kernel
cargo build --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
```

These commands verify the Kernel scope only.

## Architecture Model

| Concept | Public meaning | Boundary |
|---|---|---|
| Core semantics | Owns protocol meaning, domain rules, commands, results, reasons, states, and ports | Does not own concrete I/O, runtime libraries, platform APIs, databases, or deployment policy |
| Driver boundary | Converts external observations into Kernel-owned commands, observations, intents, or closed failures | Does not define domain meaning or product policy |
| Entrypoint composition | Wires dependencies and exposes executable Kernel contract surfaces | Does not become product deployment proof |
| SDK projection | Exposes the Signaling-only public API shape | Does not own media, auth issuance, regulated workflow, or driver internals |
| Regulated support | Provides optional support projection and non-sensitive enrichment | Does not become the generic communication core |
| External distro | Builds reference or product behavior outside Kernel authority | Does not supply Kernel completion evidence |

The conceptual dependency direction is:

```text
Core <- Drivers
Core <- Entrypoints
Drivers <- Entrypoints
```

## Evaluation Scope

The public summary projection lives under `docs/summary/`, with an English edition in `en/` and a Japanese edition in `ja/`. Reading the index alone, then the chapters it lists, gives an overview of the Kernel intent, structure, contracts, operations, verification, and troubleshooting. Current authority for completion and evidence remains in `dev-docs/`.

| Evaluation target | What to check | Public summary |
|---|---|---|
| Full summary index | Chapter map, summary principles, terminology, claim / non-claim scope | [Summary index (EN)](docs/summary/en/00-INDEX.md) · [日本語](docs/summary/ja/00-INDEX.md) |
| Scope and boundaries | Mission, System Boundary, Non-goals, layer model, dependency direction | [01 Overview](docs/summary/en/01-overview-and-scope.md) · [02 Architecture](docs/summary/en/02-architecture-and-boundaries.md) |
| Core semantics | Domain, command/reason, ports, protocol, transport/media, Signaling/SFU/TURN | [03–13](docs/summary/en/00-INDEX.md) |
| Drivers / entrypoints | Port implementations, conversion, composition, operations, topology | [14–21](docs/summary/en/00-INDEX.md) |
| SDK / regulated / packaging | Signaling-only projection, optional support, supply chain, release | [22–24](docs/summary/en/00-INDEX.md) |
| Verification | CI quality gates, testing evidence, benchmark scope | [25–27](docs/summary/en/00-INDEX.md) |
| Troubleshooting | Cross-cutting failure modes, fail-closed operations, boundary-violation remediation | [28](docs/summary/en/28-troubleshooting.md) |

## Scope Limits

The current claim is limited to Kernel-owned semantics and Kernel-owned evidence that has a matching report.

This repository does not claim:

- production readiness,
- live readiness,
- native application readiness,
- completed SFU, TURN, or Signaling product-distro build,
- benchmark acceptance threshold satisfaction,
- automatic inheritance of v0.1 behavior.

Benchmark scenarios are measurement and reportability surfaces. Treat benchmark output as scoped measurement unless a separate threshold rule is explicitly provided.

## Summary Projection

The public summary projection is under `docs/summary/`: English in `en/`, Japanese in `ja/` (identical chapter structure). It is an orientation surface for the current Kernel structure and evidence scope, not a substitute for `dev-docs/` authority or source-level verification.

Recommended reading order (English; the Japanese edition mirrors it):

1. [00 Index](docs/summary/en/00-INDEX.md)
2. [01 Overview and scope](docs/summary/en/01-overview-and-scope.md)
3. [02 Architecture and boundaries](docs/summary/en/02-architecture-and-boundaries.md)
4. Core chapters [03–13], then drivers/entrypoints [14–21]
5. SDK / regulated / packaging [22–24] and verification [25–27]
6. [28 Troubleshooting](docs/summary/en/28-troubleshooting.md)

## Change Guidance

No public support guarantee is defined by this README.

When proposing changes, preserve the ownership boundaries described above. In particular, do not move protocol meaning into driver or entrypoint code, and do not turn executable contract surfaces into product-readiness claims without separate evidence and project approval.

## License and Publication

This README does not grant license rights, package publication guarantees, production support, or service availability. Check repository metadata and explicit license files before redistribution or production use.
