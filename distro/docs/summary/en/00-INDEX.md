# arcRTC v0.2 distro — System & Development Summary (SSOT, English)

Status: SSOT consolidated edition
Date: 2026-06-28 JST
Scope: `distro/`

## Purpose of this summary

This summary (all chapters under `docs/summary/en/`) is the **Single Source of Truth (SSOT)** for arcRTC v0.2 distro: a self-contained specification and reference. Reading this summary alone — without consulting any other file or the source code — is intended to convey everything about distro: its intent, purpose, philosophy, structure, paths, properties, operations, verification, and troubleshooting.

distro is the out-of-Kernel area that builds the SFU / TURN / Signaling reference distro / product distro **on top of the frozen Kernel contract**. Its primary purpose is to demonstrate that the frozen Kernel contract is sufficient to build a working communication base: the reference distro provides constructive, falsifiable, and measurable evidence that the Kernel's contracts can be realized into executable communication paths — evidence of Kernel implementability, not a claim of completeness. It validates the Kernel and is not itself a product. Five further items — production readiness, live readiness, benchmark threshold satisfaction, real-device success, and Kernel completion / freeze — are the domain's evaluation goals, each established only when its own dedicated evidence exists (not implied by the existence of the code).

This is not a "history of how it was built"; it is the "complete present-day specification". It does not record dead ends, course corrections, or retired documents.

### SSOT principles (apply to all chapters)

- This summary does not depend on redirection to, or citation of, external documents (the source tree, the Kernel documents, or source code). All required normative content is internalized in the chapter bodies.
- Cross-references between chapters are navigation within the same SSOT (by chapter number) only. Each chapter stays self-contained.
- Normative keywords: "MUST", "MUST NOT", "MAY", and "fail-closed" (anything ambiguous or outside a closed set is not admitted — it fails toward rejection) are used consistently.
- Terminology: technical proper nouns (Kernel / reference distro / product distro / evidence / reason / readiness / benchmark / real-device, etc.) are kept in their original form; explanation is in plain English.

### Relationship to the Kernel (dependency rule)

```text
distro -> Kernel public contract
distro -> Kernel SDK projection
distro -> documented Kernel command surface

Kernel  -X-> distro
distro -X-> Kernel semantic authority overwrite
```

distro does **not** own the Kernel's core semantics, port definitions, reason catalog, dependency direction, freeze claim, final acceptance authority, or evidence authority. If a Kernel contract change becomes necessary, distro fails closed and a versioned contract revision on the Kernel side is required first.

## Chapter structure

| Chapter | Content |
|---|---|
| 00-INDEX | This index, SSOT principles, terminology, dependency rule, claim/non-claim |
| 01-overview-and-scope | Mission, System Boundary, Kernel Boundary, In/Out scope, Fixed Goal, Acceptance Focus, glossary |
| 02-architecture-and-boundaries | root model, layers (distro-support/reference/product/tests), dependency rule, workspace manifest/dependency path, module export, source/package boundary, branch identity |
| 03-kernel-contract-consumption | Kernel contract import, version pin, builder mapping, Kernel completion/freeze binding |
| 04-toolchain-runtime-dependency | distro language/toolchain, runtime, dependency admission, runtime/executor admission |
| 05-evidence-reason-and-observability | evidence reason ownership, observability evidence reason, error reason mapping, evidence schema, evidence wire format, evidence extension record |
| 06-state-and-security | state persistence, identity/auth/security |
| 07-reference-distro-design | reference distro structure, signaling/turn/sfu/composition design, runtime topology, transport, configuration profile, composition runtime bridge |
| 08-reference-runtime-state-output | reference phase & fixture payload, state mutation, API & state, output boundary (3-type allow-list), runtime lifecycle, command evidence runner |
| 09-product-distro | product distro, product API policy, plane API signature, product admission |
| 10-benchmark | benchmark scenario workload, harness layout, benchmark & real-device evidence, threshold satisfaction, comparability |
| 11-real-device | real-device command matrix, wrapper command, real-device success, scope |
| 12-readiness-production-live | readiness matrix, production provider admission, live endpoint traversal, Kernel production implementability (KPI-001 through KPI-015) |
| 13-testing-and-assertions | test package & assertion, test workspace manifest |
| 14-ci-quality-gates | command CI quality gate, quality gate state |
| 15-troubleshooting | Cross-cutting failure modes, fail-closed operations, detection and remediation of boundary violations |

## Current acceptance scope (claim / non-claim)

- claim (adopted): distro authority separated from the Kernel; boundary separation of reference/product distro; separation of the shared evidence/reason owner; prohibition (in principle) of Kernel modification.
- non-claim (established only via dedicated evidence and a Closed Gate): reference/product distro completion, production readiness, live readiness, native application readiness, benchmark threshold satisfaction, real-device success, re-adjudication of Kernel completion/freeze, inheritance of v0.1 behavior.

## Japanese edition

A Japanese edition with identical structure is under `docs/summary/ja/` (chapter numbers and names correspond).
