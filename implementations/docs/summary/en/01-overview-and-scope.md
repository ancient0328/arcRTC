# Overview and Scope

Status: SSOT consolidated edition
Date: 2026-06-28 JST

## Purpose

This chapter fixes the whole picture of the arcRTC v0.2 implementations domain (the Kernel-external implementation domain): its Mission, the surface boundary it owns, its boundary against the Kernel, what is In Scope, what is Out of Scope, the Fixed Goal, the Acceptance Focus, and the terminology, at a granularity that is understandable without referring to any other document. All rules, vocabulary, and boundaries in this chapter are self-contained within this specification and contain no external redirection.

## Mission

`implementations/` is the Kernel-external implementation domain that builds the SFU / TURN / Signaling reference implementation / product implementation by consuming the frozen Kernel contract.

The primary purpose of this domain is to demonstrate that the frozen Kernel contract is sufficient to build a working SFU / TURN / Signaling communication base: the reference implementation provides constructive, falsifiable, and measurable evidence that the Kernel's contracts can be realized into executable WebRTC communication paths — evidence of Kernel implementability, not a claim of completeness. It validates the Kernel; it does not own Kernel authority, and it is not itself a product or a production deployment.

On top of that, and without modifying the Kernel, this domain treats as its own responsibilities the reference / product implementation combining SFU / TURN / Signaling, the benchmark, the real-device test, and the production / live readiness evidence. The following five items are the domain's evaluation goals; each is established only when its own dedicated evidence exists, and none is implied by the existence of this code.

- production readiness
- live readiness
- benchmark threshold satisfaction
- real-device success
- Kernel completion / freeze

implementations MUST NOT own the Kernel semantic authority. implementations is a consumer of the frozen Kernel contract, and MUST NOT modify the Kernel-side semantics, port definition, dependency direction, freeze claim, or final Closed Gate.

## System Boundary (full table of surfaces owned by implementations)

The surfaces owned by implementations are the following closed set. Each surface is confined to its own responsibility and MUST NOT own anything outside that responsibility.

| Surface | Responsibility owned by implementations |
|---|---|
| `implementation-support/evidence/` | implementations-local evidence / reason shared type owner |
| `reference-implementation/signaling/` | Signaling reference implementation using the Kernel contract |
| `reference-implementation/turn/` | TURN reference implementation using the Kernel contract |
| `reference-implementation/sfu/` | SFU reference implementation using the Kernel contract |
| `reference-implementation/output/` | reference output outcome types; product input subset is the 3-type allow-list |
| `reference-implementation/composition/` | reference composition of Signaling / TURN / SFU |
| `reference-implementation/deployment-profiles/` | reference execution profile |
| `reference-implementation/ops/` | reference operation helper |
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

## Kernel Boundary (available items / non-owned items)

### Kernel items that implementations MAY use

implementations MAY use the following Kernel items.

- public contract
- SDK projection
- core-owned port definition
- driver-facing contract
- the published behavior of the entrypoint composition evidence surface
- the documented command surface for benchmark / test

### Kernel items that implementations MUST NOT own

implementations MUST NOT own the following Kernel items.

- core semantics
- Kernel port definition
- Kernel reason catalog
- Kernel dependency direction
- Kernel freeze claim
- Kernel final Closed Gate
- Kernel evidence authority

### When a Kernel contract change becomes necessary

When a Kernel contract change becomes necessary, the implementations side is fail-closed. That is, implementations MUST NOT bypass, duplicate, extend, or overwrite the Kernel contract. A formal versioned Kernel-side contract revision is required first (MUST).

## In Scope (15 items)

The scope of implementations is the following 15 items.

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

## Out of Scope

The following are out of scope of implementations and MUST NOT be done.

- Kernel source modification
- Kernel semantic authority modification
- repurposing Kernel evidence as production readiness / live readiness proof
- v0.1 behavior inheritance claim
- setting a benchmark threshold to fit success after measurement
- making an implementations completion claim without dedicated evidence and a verdict
- adopting any of production readiness / live readiness / benchmark threshold satisfaction / real-device success / Kernel completion / freeze into the final fixed goal without dedicated evidence and a verdict

## Fixed Goal (5 items)

The fixed goal of implementations names the five evaluation targets the domain is designed to drive toward. They are goals, not current achievements.

1. production readiness
2. live readiness
3. benchmark threshold satisfaction
4. real-device success
5. Kernel completion / freeze

This fixed goal, without eroding the frozen Kernel contract, builds the SFU / TURN / Signaling reference implementation / product implementation outside the Kernel, and then connects all five items to dedicated evidence and a verdict. Each item MUST NOT be adopted into the final fixed goal while lacking its dedicated evidence and verdict (fail-closed).

## Acceptance Focus

The acceptance focus of the initial authority setup is the following.

- the implementations area is separated from the Kernel.
- SFU / TURN / Signaling are explicitly stated as targets of implementations.
- reference implementation and product implementation are separated.
- the shared evidence / reason owner is separated without bias toward either reference or product.
- the reference surface that product implementation MAY consume is limited to `ReferenceSignalingOutcome` / `ReferenceTurnOutcome` / `ReferenceSfuOutcome` of `arcrtc-reference-output`.
- the implementation source and the test source are separated.
- Kernel modification is forbidden in principle, and when necessary a formal versioned Kernel-side contract revision is required.
- production readiness / live readiness are not auto-derived from Kernel evidence, but established by a dedicated gate.
- a close-like claim is not made without dedicated evidence and a verdict.
- achievement of the full fixed goal connects source closeout / test closeout / benchmark threshold satisfaction / real-device success / production readiness / live readiness / Kernel completion freeze evidence to a final full fixed-goal verdict.

## Governance Top-Level Boundary

The top-level operational boundary of this domain is the following. These are invariants that guarantee the authority separation between implementations and the Kernel.

- implementations consumes the Kernel frozen contract.
- implementations MUST NOT modify the Kernel semantic authority.
- implementations owns the SFU / TURN / Signaling reference implementation / product implementation.
- implementations production readiness, live readiness, native application readiness, and public distribution readiness MUST NOT be auto-derived from Kernel evidence.
- when a Kernel contract change is necessary, the implementations side is fail-closed and requires a formal versioned Kernel-side contract revision first.

## Completion-claim precondition (fail-closed)

A claim such as complete, done, fixed, resolved, closed, ready, production-ready, live-ready, executable-ready, or no findings MUST NOT be made for the target scope unless its dedicated evidence and verdict are connected. If any required condition for the target scope cannot be clearly answered Yes, the target claim MUST NOT be treated as close / complete / ready (fail-closed).

## Required fields of an evidence record

implementations-side evidence MUST NOT be accepted as evidence if it lacks any of the following (fail-closed).

- correlation ID
- command
- working directory
- target package (when required by the evidence schema)
- target scope
- expected outcome
- actual outcome
- environment / toolchain
- rerun condition
- closed set of reasons
- non-use of UNKNOWN
- non-claim scope

Evidence that shows only the correlation id and index connection MUST NOT be accepted as full evidence for command result, test pass, readiness, completion, or no findings. Measured evidence is not a substitute authority for the contract and rules fixed in this specification.

## Non-claim scope of the existence of the authority surface itself

The existence of the implementations authority surface itself claims none of the following.

- reference implementation completion
- product implementation completion
- production readiness
- live readiness
- native application readiness
- public distribution readiness
- benchmark threshold satisfaction
- re-judgment of Kernel completion / freeze

## Glossary

| Term | Meaning |
|---|---|
| Kernel | The freeze-target domain confined to `Kernel/`. It owns the semantic authority, contract, port ownership, freeze claim, and the final close-out gate. implementations does not modify it. |
| implementations | `implementations/`. The implementation domain that builds the SFU / TURN / Signaling reference / product implementation outside the Kernel. |
| Kernel contract | The collective term for the Kernel public contract / SDK projection / core-owned port definition / driver-facing contract / documented command surface that implementations MAY use. |
| reference implementation | The implementation layer that combines SFU / TURN / Signaling in a minimal configuration using the Kernel frozen contract. It does not claim production readiness. |
| product implementation | The implementation layer that consumes only the 3 outcome types of `arcrtc-reference-output` as input and owns product policy / deployment / monitoring / persistence topology / rollback on separate surfaces. |
| `arcrtc-reference-output` | The package owning the reference output outcome types. The product input subset is the 3-type allow-list of `ReferenceSignalingOutcome` / `ReferenceTurnOutcome` / `ReferenceSfuOutcome`. |
| `ReferenceSignalingOutcome` / `ReferenceTurnOutcome` / `ReferenceSfuOutcome` | The only reference outcome types that product implementation MAY consume as input (3-type allow-list). |
| `ReferenceCompositionOutcome` | The output type of reference composition. product implementation MUST NOT consume it as input. |
| benchmark threshold satisfaction | Satisfying the benchmark threshold with dedicated evidence and a verdict, without fitting the threshold to success after measurement. |
| real-device success | The success evidence and verdict of bounded Android / iOS / browser real-device commands. It does not mean general live readiness. |
| production readiness | The production acceptance state established only by implementations-side dedicated evidence and a verdict. It is not auto-derived from Kernel evidence. |
| live readiness | The live acceptance state established only by implementations-side dedicated evidence and a verdict. It is separate from real-device command evidence. |
| Kernel completion / freeze | The completion / freeze state determined on the Kernel side. implementations does not re-judge it and binds the evidence as a required input. |
| fail-closed | The principle of not claiming success / completion / close and stopping unless the required conditions are met. |
| evidence | Adoptable measured evidence that satisfies the required fields such as correlation ID / command / working directory and contains no UNKNOWN. |
| fixed goal | The achievement of all five items: production readiness / live readiness / benchmark threshold satisfaction / real-device success / Kernel completion freeze. |
