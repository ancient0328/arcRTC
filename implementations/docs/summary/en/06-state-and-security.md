# State and Security

Status: SSOT consolidated edition
Date: 2026-06-28 JST

## Purpose

This chapter fixes, at a granularity sufficient for re-implementation, the state / persistence boundary and the identity / auth / security boundary of the arcRTC v0.2 implementations domain (the non-Kernel implementation domain). This chapter is fully self-contained; it permits only references to chapter numbers within this specification, and it can be understood and re-implemented without opening any external file, Kernel document, or source code.

The central propositions of this chapter are the following two points.

1. What implementations persists and what it does not persist, and what the source-of-truth rule of state is.
2. What implementations owns and does not own regarding identity / auth / security, and what the fail-closed boundary is.

The dependency rule is `implementations -> Kernel` (contract / SDK / command surface only). Kernel owns the communication semantics and contract, and implementations-side state is runtime state / product state / evidence state that uses the Kernel contract. implementations is not the Kernel semantic authority.

---

## 1. State / Persistence Boundary

This section fixes the boundary of reference state, product persistence topology, benchmark fixture state, and evidence artifact.

### 1.1 Context and Principle

SFU / TURN / Signaling implementations handle session, participant, allocation, route, relay, subscription, and command evidence. Leaving the state / persistence boundary unfixed causes reference state, product database, benchmark fixture, and evidence artifact to crosstalk, and reference success to be diverted into product persistence readiness.

Kernel owns the communication semantics and contract. implementations-side state is runtime state / product state / evidence state that uses the Kernel contract, and it is not the Kernel semantic authority.

### 1.2 State Class Ownership Matrix

| State class | Owner | Persistence rule | Claim boundary |
|---|---|---|---|
| reference runtime state | reference implementation | in-memory only | limited to reference behavior |
| benchmark fixture state | benchmark harness | fixture file / in-memory | limited to benchmark scenario execution |
| command evidence state | evidence report / command output | report artifact | limited to evidence scope |
| product runtime state | product implementation | requires a product-owned persistence admission record | limited to product scope |
| production database state | product implementation / operations | requires a production readiness admission record | not adopted in reference scope |

The reference implementation MUST NOT hold a production database. When a product implementation adopts a database / queue / external storage, it MUST first record a product persistence admission decision.

### 1.3 Reference State Rule

The state of the reference implementation is fixed to in-memory deterministic state only. The reference implementation MUST NOT hold a database, queue, external storage, or cloud persistence.

#### Required Reference State Files

| Package | File | Owns |
|---|---|---|
| `arcrtc-reference-signaling` | `src/state.rs` | room / session / participant projection |
| `arcrtc-reference-turn` | `src/state.rs` | allocation / permission / relay projection |
| `arcrtc-reference-sfu` | `src/state.rs` | route / subscription / forwarding projection |
| `arcrtc-reference-composition` | `src/composition_state.rs` | cross-plane correlation projection |

Each state file MUST satisfy the following.

- Has a deterministic constructor.
- Does not read the wall-clock directly.
- Does not generate a random value directly.
- Does not own a Kernel state type.
- Does not include a product persistence schema.

### 1.4 Product Persistence Rule

Product persistence is owned by `product-implementation/persistence-topology/`. Initial product persistence is limited to a topology scaffold, and a concrete database provider is not adopted.

| File | Role |
|---|---|
| `product-implementation/persistence-topology/src/lib.rs` | product persistence topology export |
| `product-implementation/persistence-topology/src/topology.rs` | persistence topology model |
| `product-implementation/persistence-topology/src/mapper.rs` | Kernel / product projection mapping |
| `product-implementation/persistence-topology/src/error.rs` | persistence evidence reason bridge |

A provider-specific database item shape MUST NOT be added without a recorded product persistence provider admission decision.

### 1.5 State Ownership Rule

What a state module MAY own:

- session lifecycle projection
- participant registry projection
- relay allocation projection
- local routing projection
- benchmark fixture projection
- evidence correlation projection

What a state module MUST NOT own:

- Kernel port definition
- Kernel reason catalog
- Kernel semantic decision
- product policy authority
- production readiness claim

### 1.6 Evidence State Rule (source-of-truth rule)

An evidence artifact is not the source of truth of runtime state. An evidence artifact MAY be adopted only as a report. An evidence artifact MUST NOT be loaded as runtime state.

### 1.7 Persistence Prohibition

The following are not permitted (MUST NOT).

- Treating reference in-memory state as production persistence.
- Treating a product persistence schema as a Kernel contract.
- Making a database item shape the Kernel semantic authority.
- Making a benchmark fixture a basis for production readiness.
- Making an evidence artifact a runtime source of truth.

### 1.8 State Persistence Non-claims

The state / persistence boundary does not claim the following.

- product persistence design completion
- production database readiness
- durability guarantee
- backup / restore readiness
- operational readiness

---

## 2. Identity / Auth / Security Boundary

This section fixes the boundary of reference fixture identity, local auth, TURN credential, product auth policy, and secret handling.

### 2.1 Context and Principle

SFU / TURN / Signaling implementations handle identity, credential, authorization, transport security, TURN credential, and session admission. Leaving the identity / auth / security boundary unfixed causes reference fixture credential, product authentication, live endpoint security, and Kernel reason to crosstalk.

Kernel does not own an auth provider, a tenant identity provider, or a production credential issuer. The implementations side also does not issue production auth in the reference scope.

### 2.2 Surface Decision Matrix

| Surface | Decision | Claim boundary |
|---|---|---|
| reference identity | local fixture identity | limited to reference behavior |
| reference signaling auth | deterministic local token / command fixture | does not claim production auth |
| reference TURN credential | local fixture credential | does not claim production TURN security |
| product identity provider | requires a product implementation admission record | limited to product scope |
| production security | requires a production readiness admission record | not adopted in reference scope |

A secret, credential, token, or private key MUST NOT be stored as a real secret in reports or source fixtures. A fixture credential MUST be limited to a value that is explicitly recognizable as a fixture.

### 2.3 Reference Fixture Identity

The reference implementation MUST use deterministic fixture identity only.

| Fixture field | Format | Owner |
|---|---|---|
| `identity_id` | `fixture-identity-{n}` | reference signaling / composition |
| `session_id` | `fixture-session-{n}` | reference signaling |
| `room_id` | `fixture-room-{n}` | reference signaling |
| `turn_credential_id` | `fixture-turn-credential-{n}` | reference TURN |
| `sfu_route_id` | `fixture-sfu-route-{n}` | reference SFU |

A fixture value MUST NOT be treated as a production credential. A secret, private key, or real token MUST NOT be stored in the repository / report.

### 2.4 Required Security Files

| Package | File | Role |
|---|---|---|
| `arcrtc-reference-signaling` | `src/fixture_identity.rs` | local identity / session fixture |
| `arcrtc-reference-signaling` | `src/local_auth.rs` | deterministic local auth check |
| `arcrtc-reference-turn` | `src/fixture_credential.rs` | local TURN credential fixture |
| `arcrtc-reference-sfu` | `src/fixture_route_auth.rs` | local route admission fixture |
| `arcrtc-product-policy` | `src/auth_policy.rs` | product auth policy boundary |
| `arcrtc-product-policy` | `src/security_reason.rs` | product security evidence reason bridge |

### 2.5 Security Boundary Rule

What a security module MAY own:

- fixture identity parsing
- fixture credential verification
- local command admission check
- product auth policy interface
- implementation evidence reason mapping

What a security module MUST NOT own:

- Kernel reason catalog
- Kernel port definition
- production identity provider authority
- production secret lifecycle
- live readiness claim

### 2.6 Product Auth Rule / Admission

The product auth provider is not adopted in the initial state. When a product implementation adopts an auth provider, it MUST first fix the following.

- provider owner
- credential source
- token validation rule
- secret storage boundary
- failure reason closed set
- audit evidence boundary
- non-claim scope

### 2.7 Forbidden Pattern

The following are not permitted (MUST NOT).

- Treating a fixture token as production auth.
- Adding a product auth failure reason to the Kernel reason catalog.
- Recording a secret in a report.
- Treating auth command success as live readiness.

### 2.8 Identity / Auth / Security Non-claims

The identity / auth / security boundary does not claim the following.

- production auth readiness
- security readiness
- TURN production credential readiness
- public endpoint security
- identity provider integration completion

---

## 3. Invariants and Fail-Closed Conditions (Collapse Conditions)

The authority of this chapter collapses if any of the following occurs. These are fail-closed boundaries, and all are forbidden (MUST NOT).

### 3.1 State / Persistence Invariants

- The reference implementation directly holding a database / queue / external storage.
- The reference implementation directly adopting a production database.
- Treating a product persistence schema as a Kernel contract.
- A product persistence schema overriding Kernel semantics.
- Making a database item shape the Kernel semantic authority.
- Loading an evidence artifact as runtime state.
- Making an evidence artifact a runtime source of truth.
- Inheriting benchmark fixture state as product state.
- Treating state persistence success as live readiness.
- A persistence owner being added to source without a recorded admission decision.

### 3.2 Identity / Auth / Security Invariants

- The reference implementation holding a real credential issuer.
- The reference implementation holding a production credential issuer.
- A fixture identity becoming the product identity source of truth.
- Treating a fixture token / fixture credential as production auth.
- Storing a secret in reports / source fixtures.
- Introducing a product auth provider without a recorded admission decision.
- Mixing a product auth failure reason into the Kernel reason catalog.
- Treating auth command success as live readiness.
- Security evidence automatically claiming production security readiness.
- Introducing an identity provider choice into source without a recorded admission decision.
