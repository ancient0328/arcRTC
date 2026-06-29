# Overview and Scope
Status: SSOT consolidated edition
Date: 2026-06-28 JST

## Purpose

This chapter fixes what the arcRTC v0.2 Kernel is and is not. It presents, self-contained, the Mission, the value v0.2 sets out to demonstrate, the definition of the Kernel, the System Boundary (Kernel root and implementations root), the responsibility summary of core/drivers/entrypoints/sdk/regulated, the full enumeration of Non-goals, and a glossary of key terms. From this chapter alone, the full picture of the problem v0.2 solves and its boundaries can be understood.

## Mission

arcRTC v0.2 redesigns the Kernel of a WebRTC communication platform using DDD (Domain-Driven Design) / hexagonal architecture, with the goal of cleanly separating the responsibility boundaries of Signaling / SFU / TURN / SDK / regulated support.

v0.2 is not an extension or patch of v0.1. It is treated as a system that re-fixes the architecture on a DDD / hexagonal basis. As a Kernel, it fixes the `core / drivers / entrypoints + sdk + regulated` boundary first.

v0.2 is not the SFU / TURN / Signaling implementation system itself. It is the object to be completed and frozen as a Kernel. The SFU / TURN / Signaling implementation systems are built as reference implementations / product implementations in the out-of-Kernel implementations area.

## Value of v0.2

The value of v0.2 is not feature expansion. It lies in demonstrating the following four points within a structure whose boundaries do not collapse.

| Value axis | Meaning |
|---|---|
| Kernel viability | The Kernel closes as the `core / drivers / entrypoints / sdk / regulated` boundary, and each layer's responsibility is fixed without confusion. |
| Observability | Communication events, decisions, and reasons are observable and can be evidenced with correlation IDs, closed-set reasons, and reproducible procedures. |
| Stability | Boundaries are not eroded; what each layer owns and does not own is fixed in authority. |
| Verifiability | build / test / runtime verification is backed by evidence, and close / complete / ready is backed by a Closed Gate Report before being claimed. |

## What the Kernel Is / Is Not

The Kernel is the completion-and-freeze object that owns the semantic authority of arcRTC v0.2. The Kernel root is `Kernel/`. This root is the object to be completed and frozen as arcRTC v0.2 Kernel.

The Kernel is a structure that places the pure semantics of Signaling / SFU / TURN in core and separates them from binary / I/O. The in-Kernel `entrypoints/*-server` are not product systems; they are limited to executable contract / composition evidence surfaces.

The Kernel is NOT the following.

- The Kernel is not an alias for `core/`. `core/` is the semantic nucleus of the Kernel, not the whole Kernel. The Kernel includes drivers, entrypoints, sdk, and regulated in addition to core.
- The Kernel is not the SFU / TURN / Signaling implementation system itself. The implementation systems are owned by the out-of-Kernel implementations.
- The Kernel does not own product deployment, live endpoint operation, monitoring, rollback, production SLO, or product-specific topology. These are owned by implementations.

implementations MUST NOT override the Kernel semantic authority.

## System Boundary

The v0.2 boundary consists of two: the Kernel root and the out-of-Kernel implementations root.

| Boundary | Path | Position |
|---|---|---|
| Kernel root | `Kernel/` | The object to be completed and frozen as arcRTC v0.2 Kernel. Owns Kernel authority. |
| implementations root | `implementations/` | The place to build out-of-Kernel reference / product implementations. Not included in the Kernel completion conditions. |

Out-of-Kernel implementations are not included in the completion conditions of `Kernel/`. implementations build SFU / TURN / Signaling reference or product implementations using the Kernel contract / port / SDK projection. implementations MAY own product deployment, live endpoint operation, monitoring, rollback, production SLO, and product-specific topology, but MUST NOT override the Kernel semantic authority.

The implementations root has the following owned surfaces.

| Surface | Position |
|---|---|
| `implementations/reference-implementation/` | reference implementation construction area |
| `implementations/product-implementation/` | product implementation construction area |
| `implementations/tests/` | area for implementation readiness / production / live evidence |

On a responsibility conflict between the Kernel and implementations, the Kernel authority takes precedence. If implementations require a change to a Kernel contract, a Kernel-side versioned specification MUST be established first; implementations MUST NOT directly alter the Kernel contract.

The principal in-Kernel boundary (the formal structural axis) is as follows.

```text
core <- drivers
core <- entrypoints
drivers <- entrypoints
```

The dependency-direction notation `A <- B` means "B depends on A (B may reference A)." That is, drivers and entrypoints depend on core, and entrypoints depend on drivers. `sdk` is an independent Signaling-only boundary. `regulated` is optional domain support. The full set of permitted/forbidden boundary directions is internalized in Chapter 02.

## Responsibility Summary of Each Layer

### core

`core/` holds the central semantics as the semantic nucleus, center, and highest authority of the arcRTC Kernel. `core/` is not an alias for the whole Kernel.

What `core/` owns:

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

`core/` MUST NOT depend on drivers, entrypoints, framework, runtime, OS, browser, DB, or cloud SDK.

### drivers

`drivers/` implement the ports owned by core.

- str0m driver
- network I/O
- persistence
- observability
- security verifier / key source / secret rotation
- browser boundary
- native boundary
- clock / randomness / runtime gateway

External types MUST be converted to core-owned types at the drivers boundary.

### entrypoints

`entrypoints/` hold the in-Kernel startup units, contract probes, and composition evidence surfaces.

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

entrypoints MUST NOT own domain rules. entrypoints MUST NOT own SFU / TURN / Signaling product implementations.

### sdk

`sdk/` is an independent Signaling-only boundary. The SDK exposes the Signaling public client contract to consumers but MUST NOT directly own regulated support.

### regulated

`regulated/` is optional domain support. It MUST NOT be included in the generic communication core. `regulated -> core` is limited to references of specification-declared opaque communication events, audit pointers, and non-sensitive tags. `regulated -> drivers`, `regulated -> entrypoints`, `regulated -> sdk`, `core -> regulated`, `drivers -> regulated`, `entrypoints -> regulated`, and `sdk -> regulated` are forbidden.

## Non-goals

The non-goals of the v0.2 initial design are as follows. These are what v0.2 does not set out to achieve, enumerated without omitting a single item.

- mechanical port of the v0.1 implementation
- automatic inheritance of v0.1 authority
- medical domain features themselves
- medical data transfer
- chat, recording, screen sharing, DataChannel
- authentication infrastructure or token issuance
- UI / end-user workflow
- finished deployment form
- SFU / TURN / Signaling product implementation
- implementations operation of SFU / TURN / Signaling reference implementations
- production readiness
- live readiness

These non-goals are explicit exclusions at the initial design stage. Whether they are admitted in the future is decided per the out-of-scope feature admission / exclusion authority (the mechanism of that admission is owned by another chapter).

## Kernel Completion / Freeze Acceptance Focus

The current acceptance focus after Kernel Completion / Freeze (KCF) completion / freeze is as follows. These are the acceptance viewpoints that MUST be satisfied to claim the Kernel is complete and frozen.

- The core / drivers / entrypoints / sdk / regulated boundaries are fixed in authority.
- The Signaling / SFU / TURN semantics are placed in core and separated from binary / I/O.
- The policy that v0.2 closes as a Kernel and places the SFU / TURN / Signaling implementation systems in out-of-Kernel implementations is fixed in authority.
- str0m is treated as a drivers-side port implementation.
- regulated has not leaked into the generic communication core.

## Glossary of Key Terms

| Term | Meaning |
|---|---|
| Kernel | The completion-and-freeze object owning the semantic authority of arcRTC v0.2. Root is `Kernel/`. |
| implementations | The area outside the Kernel where SFU / TURN / Signaling reference / product implementations are built. Root is `implementations/`. Not included in the Kernel completion conditions. |
| core | The semantic nucleus, center, and highest authority of the Kernel. Owns domain / use case / port / pure protocol / transport contract and does not depend on external I/O. Not an alias for the whole Kernel. |
| drivers | Implementations of the ports owned by core. Own external I/O, runtime, network, persistence, observability, security verifier / key source / secret rotation, and str0m connection. |
| entrypoints | In-Kernel startup units, executable contracts, composition evidence surfaces, and dependency wiring. Do not own domain rules or product implementations. |
| sdk | Independent Signaling-only boundary. Exposes the Signaling public client contract. Does not directly own regulated support. |
| regulated | Optional domain support. Not included in the generic communication core. Only `regulated -> core` is conditionally allowed under explicit specification admission. |
| semantic nucleus | The central semantics held by core. The center of the Kernel's semantic authority. |
| semantic authority | The authority over semantics. Owned by the Kernel; implementations cannot override it. |
| Signaling | The plane handling room state, command semantics, and accept / reject boundary. Pure semantics are placed in core. |
| SFU | Selective Forwarding Unit. The plane handling routing, quality decision, and backpressure semantics. Pure semantics are placed in core. |
| TURN | Traversal Using Relays around NAT. The plane handling allocation, permission, and relay semantics. Pure semantics are placed in core. |
| port | The abstract boundary owned by core: network / persistence / clock / metrics / audit sink, etc. Implemented by drivers. |
| Sans-IO | The design style of a pure transport contract without I/O. Adopted by core/transport. |
| executable contract | The in-Kernel executable contract surface held by entrypoints. Not a product system. |
| composition evidence surface | The composition evidence surface held by entrypoints. Demonstrates dependency-wiring evidence. |
| reference implementation | The reference implementation owned by implementations. Uses the Kernel contract / port / SDK projection. |
| product implementation | The product implementation owned by implementations. May own deployment / SLO / topology but cannot override the Kernel semantic authority. |
| KCF | Kernel Completion / Freeze. The scope and acceptance conditions for completing and freezing the Kernel. |
| Closed Gate Report | The verification-scope evidence report produced as a precondition for any close / complete / resolved / ready claim. |
| opaque communication event | An opaque-content communication event type that regulated may reference from core. Requires explicit specification admission. |
| audit pointer | A reference into audit that regulated may reference from core. Requires explicit specification admission. |
| non-sensitive tag | A non-sensitive tag that regulated may reference from core. Requires explicit specification admission. |
