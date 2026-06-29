# arcRTC v0.2 Kernel — System & Development Summary (SSOT, English)

Status: SSOT consolidated edition
Date: 2026-06-28 JST
Scope: `Kernel/`

## Purpose of this summary

This summary (all chapters under `docs/summary/en/`) is the **Single Source of Truth (SSOT)** for the arcRTC v0.2 Kernel: a self-contained specification and reference. Reading this summary alone — without consulting any other file or the source code — is intended to convey everything about the v0.2 Kernel: its intent, purpose, philosophy, structure, paths, properties, operations, verification, and troubleshooting.

This is not a "history of how it was built"; it is the "complete present-day specification". It does not record dead ends, course corrections, or retired documents. Every statement describes the current authoritative state.

The Kernel's implementability — that these contracts can be realized into executable communication paths — is validated separately by the out-of-Kernel implementations track, which consumes the frozen contract without owning Kernel authority and is not itself a product.

### SSOT principles (apply to all chapters)

- This summary does not depend on redirection to, or citation of, external documents (the source tree, source code, or v0.1 material). All required normative content is internalized in the chapter bodies.
- Cross-references between chapters are navigation within the same SSOT (by chapter number) only. Each chapter stays self-contained so the reader never needs to jump elsewhere to understand it.
- Normative keywords: "MUST", "MUST NOT", "MAY", and "fail-closed" (anything ambiguous or outside a closed set is not admitted — it fails toward rejection) are used consistently.
- Terminology: technical proper nouns (core / drivers / entrypoints / sdk / regulated / port / Sans-IO / SFU / TURN / Signaling, etc.) are kept in their original form; explanation is in plain English.

### Dependency-direction notation

`A <- B` means "B depends on A (B may reference A)". The formal structural axes of this system are:

```text
core <- drivers
core <- entrypoints
drivers <- entrypoints

sdk:       independent Signaling-only boundary
regulated: optional domain support (independent)
```

Forbidden dependency directions (any occurrence is a boundary violation):

```text
core -> drivers / core -> entrypoints / core -> regulated
drivers -> entrypoints / drivers -> regulated
entrypoints -> regulated / sdk -> regulated
```

Only `regulated -> core` MAY be allowed, and only for references to opaque communication events, audit pointers, and non-sensitive tags explicitly authorized by an ADR.

## Chapter structure

| Chapter | Content |
|---|---|
| 00-INDEX | This index, SSOT principles, terminology, notation |
| 01-overview-and-scope | Mission, value, definition of the Kernel, System Boundary, Non-goals, glossary |
| 02-architecture-and-boundaries | Layer model, dependency direction, crate/package boundary, semantic modular monolith, source shard |
| 03-core-domain-and-identity | DDD domain model, aggregate, use case, identity/reference, cross-plane identity/session binding |
| 04-core-command-and-reason | command/decision/event/result shape, idempotency/replay/correlation, reason catalog, external error mapping |
| 05-core-ports | core ports, port contract shape |
| 06-core-protocol-and-serialization | deterministic encoding, protocol versioning, compatibility/deprecation, wire envelope, feature flag/capability lifecycle |
| 07-core-transport-and-media | core transport contract (Sans-IO), SDP/ICE negotiation, ICE candidate lifecycle, media codec/track/layer, packet rewrite/transform, secure media session lifecycle |
| 08-core-signaling-plane | Signaling contract, Signaling state machine |
| 09-core-sfu-plane | SFU contract, SFU state machine, packet semantic view, packet buffer lifecycle, congestion/pacing/retransmission |
| 10-core-turn-plane | TURN contract, TURN lifecycle |
| 11-core-runtime-time-concurrency | runtime/clock/randomness, task/worker lifecycle, retry/timeout/cancellation, concurrency/ordering/lock, atomicity/transaction/compensation, unit/measurement, time sync/clock skew, resource bounds/backpressure |
| 12-core-security-and-audit | token verification, authorization context/policy, audit event, audit hash-chain |
| 13-core-quality-and-admission | quality metrics, rate limit/quota/admission |
| 14-drivers-transport-network | driver conversion, network I/O, transport driver, browser/native driver, TURN wire driver |
| 15-drivers-persistence-state | persistence, state persistence policy, durable recovery/restore/replay, schema/migration, export/backup, distributed state/replication/failover |
| 16-drivers-observability-privacy | observability boundary, signal taxonomy, privacy/redaction/retention |
| 17-drivers-security-secrets | security key source/verifier driver, transport security configuration, secret rotation lifecycle |
| 18-entrypoints-composition-config | composition root, configuration boundary, profile/policy bundle, runtime reconfiguration/hot-swap |
| 19-entrypoints-endpoints-edge | public endpoint/connection lifecycle, edge/proxy trust boundary |
| 20-entrypoints-operations-lifecycle | health/readiness/liveness/admin/maintenance, cross-plane shutdown/drain, crash/panic/supervisor restart, operator/admin authorization |
| 21-entrypoints-topology-control-plane | deployment topology/service boundary, internal control-plane contract, internal service identity/trust, service discovery/endpoint resolution |
| 22-sdk | SDK Signaling-only, platform parity, reconnect/session resumption, public API contract generation |
| 23-regulated | regulated boundary, enrichment lifecycle, out-of-scope feature admission |
| 24-packaging-supplychain-release | crate/package boundary (addendum), supply chain/dependency/license, release artifact/distribution/provenance |
| 25-ci-quality-gates | CI quality gate, CI/testing command matrix |
| 26-testing-and-evidence | testing evidence, code coverage thresholds and denominator, test double/fixture boundary, evidence record fields |
| 27-benchmark | benchmark scope, benchmark scenario matrix |
| 28-troubleshooting | Cross-cutting failure modes, fail-closed operations, detection and remediation of boundary violations |

## Current acceptance scope (claim / non-claim)

The acceptance scope this summary fixes is below.

- claim (adopted): fixed boundaries of core / drivers / entrypoints / sdk / regulated; placement of Signaling/SFU/TURN semantics in core; establishment of the Kernel completion / freeze scope.
- non-claim (not asserted): production readiness, live readiness, native application readiness, completion of (out-of-Kernel) implementations reference/product systems, satisfaction of benchmark acceptance thresholds, inheritance of v0.1 behavior.

## Japanese edition

A Japanese edition with identical structure is under `docs/summary/ja/` (chapter numbers and names correspond).
