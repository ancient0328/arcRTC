# SDK: Signaling-only boundary, platform parity, reconnect/resumption, public API contract generation, native command evidence

Status: public summary projection
Date: 2026-07-06 JST

## Purpose

This chapter specifies, in a fully self-contained form (understandable without opening any other file, source dev-doc, or the actual code), the SDK boundary of the arcRTC v0.2 Kernel. It covers five areas: the SDK Signaling-only boundary; TypeScript / Android / iOS platform parity; the full states and procedures for reconnect / session resumption; public API contract generation / projection; and the native SDK command evidence boundary. The granularity is sufficient for re-implementation from this chapter alone.

The SDK is a client boundary by which a consumer connects to the Signaling contract. It is neither a media framework nor a regulated workflow SDK. This chapter fixes that the SDK does not own media, auth issuance, or regulated domain support. It internalizes all owners and failure mappings so that SDK-local behavior (reconnect, threading, lifecycle, error wrapper, public API shape) is not implicitly promoted to server-side semantics.

This chapter writes the dependency direction as `A <- B` ("B depends on A"). The SDK is an independent Signaling-only boundary and does not directly own regulated. Both `sdk -> regulated` and `regulated -> sdk` are prohibited.

---

## Part A. SDK Signaling-only Boundary

### A-1 SDK Scope (owned concerns)

The SDK owns the following (MAY).

- signaling endpoint connection
- signaling command send
- signaling event receive
- correlation ID propagation
- message codec
- typed error mapping
- reconnect / close behavior (as a client transport concern)
- local SDK state required for the signaling session
- send/receive of version / capability values for the Signaling contract and handling of the negotiation result

### A-2 SDK Out of Scope (concerns not owned)

The SDK does not own the following (MUST NOT).

- PeerConnection abstraction / PeerConnection lifecycle
- camera / microphone capture
- media track rendering / capture / render
- screen sharing
- recording
- chat
- DataChannel application semantics
- UI / end-user workflow
- auth issuance / token issuance
- user account management / role authorization
- regulated workflow
- medical data model

Absence from v0.2 initial scope is not, by itself, an implementation gap. It becomes a boundary violation only if an out-of-scope feature is exposed, tested, or claimed as a v0.2 capability without core contract admission.

### A-3 Version Negotiation Boundary

Signaling contract version negotiation semantics are core-owned. The SDK MAY send supported version / capability values and handle the accepted or rejected result. The SDK MUST NOT decide server-side accept / reject semantics for version or capability.

### A-4 Reconnect / Resumption Boundary (summary; details in Part C)

SDK reconnect is client-local transport behavior. Session resumption MAY preserve client-side correlation and pending command state only when the reconnect contract (Part C) allows it. It MUST NOT recreate server-side room membership, participant lifecycle, SFU route, TURN allocation, or acceptance result without a new or accepted Signaling contract event.

### A-5 Regulated Rule

The SDK does not directly own regulated support (MUST NOT). Even when regulated integration is required, regulated references a core-owned opaque communication event, an audit pointer, or a non-sensitive tag rather than an SDK event. `regulated -> sdk` and `sdk -> regulated` are prohibited.

### A-6 SDK Error Boundary (closed-set SDK-local error codes)

SDK errors are classified into the following closed SDK-local codes.

| SDK error code | Owner | Meaning | Server reason required |
|---|---|---|---|
| `connection_failed` | sdk | client transport connection failed | no |
| `server_protocol_violation` | sdk wrapper, core semantics | received server `ProtocolViolation` event | yes |
| `local_protocol_violation` | sdk | SDK detected local protocol misuse before send | no |
| `rejected_by_server` | sdk | server rejected command or negotiation | yes |
| `timeout` | sdk | client-side wait deadline exceeded | no |
| `local_unsupported_version` | sdk | SDK cannot send the configured Signaling version | no |
| `server_unsupported_version` | sdk wrapper, core semantics | server rejected Signaling version | yes |
| `malformed_event` | sdk | event cannot be decoded to SDK Signaling model | no |
| `local_serialization_error` | sdk | SDK cannot encode local command | no |
| `closed_by_remote` | sdk | remote closed Signaling connection | required for server-origin close with catalog reason; absent for local transport close without catalog reason |

Invariants (error boundary):

- The SDK MUST NOT convert a server-side rejection reason into a different semantic.
- The SDK-local error category is a wrapper only.
- Only `rejected_by_server`, `server_protocol_violation`, `server_unsupported_version`, and `closed_by_remote` MAY carry a server-side catalog reason.
- When one of these SDK errors carries a server-side catalog reason, the SDK MUST preserve the cataloged `category` and `code`.
- The SDK MUST NOT collapse a server-side catalog reason into SDK-local wrapper text only.

### A-7 Part A Collapse Conditions

The judgments of Part A break when:

- the SDK exposes PeerConnection / media as a core contract.
- the SDK directly owns a regulated API.
- regulated depends on an SDK event / SDK API.
- Signaling command / event semantics differ per platform.
- the SDK reimplements server-side accept / reject rules.
- the SDK owns version / capability accept-reject semantics.
- SDK reconnect / session resumption implicitly restores server-side state.
- a generated / public SDK API artifact becomes server semantic authority.
- the SDK exposes an out-of-scope feature as a supported server capability.

---

## Part B. Platform Parity (TypeScript / Android / iOS)

### B-1 Boundary

| Surface | Owner | Rule |
|---|---|---|
| Signaling command / event semantics | core | no platform difference |
| SDK public API shape | sdk | platform idiom allowed |
| platform transport implementation | sdk or driver implementation layer | does not change core semantics |
| error mapping | sdk wrapper (with core reason preservation) | preserves server reason category/code |
| threading / lifecycle integration | sdk | client-local concern only |
| regulated workflow | regulated | SDK does not directly own |

### B-2 Parity Dimensions (closed set)

v0.2 initial SDK parity is limited to the following dimensions.

| Dimension | Required parity |
|---|---|
| command set | connect, close, join, leave, offer, answer, ice candidate, capability/version negotiation |
| event set | connected/closed, join result, participant lifecycle, offer/answer/ice relay, rejection/protocol violation |
| correlation | generated/supplied correlation ID must be traceable per command |
| version negotiation | same semantic accept/reject behavior |
| compatibility / deprecation | same accepted/rejected version semantics per supported platform |
| capability exposure | same server-defined capability semantics per supported platform |
| server reason preservation | same catalog category/code when server reason exists |
| local error classes | platform-specific wrapper allowed, but closed local category required |
| lifecycle | local cancel/close must not become server-side state transition without command |
| reconnect/session resumption | same client-local retry/resumption class and same server reason preservation |
| public API projection | same semantic command/event/reason/correlation matrix from source contract |

A new parity dimension is out of the v0.2 initial scope.

### B-3 Platform API Rule

Platform idiom is allowed. Examples:

- TypeScript may expose Promise / callback / event emitter style.
- Android may expose suspend / callback / Flow style.
- iOS may expose async / delegate / Combine-like wrapper style.

These API style differences MUST NOT change:

- command semantics
- event semantics
- server reason category/code
- correlation rule
- version negotiation semantics
- the SDK out-of-scope media/regulated boundaries
- reconnect/session resumption semantics

### B-4 Error Mapping Rule

SDK-local errors are wrapper errors. Server-origin rejection MUST preserve the server catalog category/code.

| Error origin | SDK behavior |
|---|---|
| local serialization failure | SDK-local closed error |
| local transport connection failure | SDK-local closed error |
| local timeout | SDK-local closed error |
| server rejection | SDK wrapper plus preserved server reason |
| server protocol violation | SDK wrapper plus preserved server reason |
| malformed server event | SDK-local closed error and no invented server reason |

The SDK MUST NOT convert a server `token_verification_failed`, `room_closed`, `unsupported_command_version`, or other cataloged reason into platform-only free text.

### B-5 Threading / Lifecycle Rule

Threading, coroutine, event loop, lifecycle callback, and cancellation primitive are platform-local. They do not define server room state, participant state, SFU route state, or TURN allocation state.

A local SDK close MAY close the client transport. It MUST NOT be reported as a server-side leave unless a Signaling leave command is sent and accepted/rejected through the contract. SDK reconnect/resumption MAY retry the client transport or replay allowed client-local pending command state, only as defined by the reconnect contract (Part C). It MUST NOT silently recreate server-side participant, room, SFU, or TURN state.

### B-6 Test Parity Rule

Each platform SDK MUST have contract tests that cover:

- command encoding for the same semantic command set
- event decoding for the same semantic event set
- server reason preservation
- local error wrapper classification
- version negotiation handling
- compatibility/deprecation behavior when a surface is listed as supported or removed
- capability disabled behavior and server reason preservation
- correlation propagation
- reconnect/session resumption class, retry bound, and server reason preservation

Test results are evidence only when adopted (adoption of test evidence is owned by the testing authority). SDK public API projection evidence follows Part D.

### B-7 Prohibitions (platform parity)

- one platform exposes additional server-side semantics not present in the Signaling contract.
- a platform-specific lifecycle event implicitly becomes a server-side command.
- SDK error mapping erases the server reason category/code.
- the SDK imports a regulated package or exposes a regulated workflow.
- the SDK exposes PeerConnection/media as a v0.2 Signaling contract.
- a platform SDK relies on core internals or driver concrete types as public API.
- SDK reconnect/resumption invents server-side state without an accepted Signaling command.
- a generated SDK artifact becomes source-of-truth for server semantics.

### B-8 Part B Collapse Conditions

- TypeScript / Android / iOS command or event semantics diverge.
- a platform wrapper hides a server-side reason.
- lifecycle/threading behavior changes core protocol state.
- the SDK public API depends on regulated or media responsibilities.
- contract tests cannot identify a common semantic command/event matrix.
- reconnect/session resumption behavior diverges by platform or hides a server reason.
- the SDK public API projection lacks a source Signaling contract reference.

---

## Part C. Full States and Procedures for Reconnect / Session Resumption

### C-1 Boundary

| Concern | Owner | Rule |
|---|---|---|
| SDK local reconnect loop | sdk | client-local behavior |
| reconnect backoff timer | sdk | does not define server state |
| Signaling reconnect command/event | core if contract admits it | explicit command semantics required |
| server participant state | core/signaling | SDK local connection does not mutate it by itself |
| session resumption policy | core only if the contract defines it | no implicit restore |
| transport reconnection I/O | sdk/driver | external implementation detail |
| durable restore | server core/drivers/entrypoints | separate from SDK reconnect |

SDK local reconnect is not server-side resumption. Command idempotency, replay window, response replay, and correlation rules are owned by the command idempotency authority.

### C-2 Reconnect Classes (closed set)

The reconnect classes of v0.2 initial architecture are limited to the following.

| Class | Meaning | Claim limit |
|---|---|---|
| `local_transport_reconnect` | SDK reconnects underlying transport | not server participant restoration |
| `signaling_rejoin_command` | SDK sends explicit join/rejoin command | server decision required |
| `event_stream_resubscribe` | SDK resubscribes to client event stream | not missed-event proof |
| `session_resumption_requested` | explicit resumption request if contract exists | denied unless core contract admits |
| `reconnect_exhausted` | SDK local retry/backoff limit reached | local SDK failure, not server state |

A new reconnect class is out of the v0.2 initial scope.

### C-3 Resumption Rule (admission preconditions)

Session resumption is prohibited (fail-closed) unless a Signaling contract defines:

- resumption command;
- identity/correlation requirements;
- accepted prior state references;
- replay/missed-event behavior;
- command idempotency and response replay behavior;
- expiration window;
- failure reasons;
- SDK parity behavior;
- evidence requirements.

Absent that contract, the SDK MUST use normal join/leave/close semantics and preserve the server rejection reason.

### C-4 Failure Mapping (closed-set reasons)

| Failure | Required reason |
|---|---|
| session resumption not allowed | `session_resumption_not_allowed` |
| SDK reconnect attempts exhausted | `sdk_reconnect_exhausted` |
| SDK replay conflicts with server idempotency policy | `idempotency_payload_mismatch` or `replay_not_allowed` |
| server rejects explicit join/rejoin | server cataloged reason |
| local transport send/receive failed | SDK-local closed error or mapped network failure |
| operation cancelled | `operation_cancelled` or SDK-local cancellation class |
| operation deadline exceeded | `operation_deadline_exceeded` |

SDK-local errors MUST NOT invent a server-side reason.

### C-5 Evidence Rule (reconnect)

SDK reconnect evidence MUST record:

- platform;
- reconnect class;
- local retry/backoff policy;
- explicit server command if sent;
- server reason preservation;
- whether resumption was in scope;
- close-not-claimed scope.

Reconnect tests using a fake server or fake driver follow the test double authority.

### C-6 Prohibitions (reconnect)

- SDK local reconnect marks a participant joined without server acceptance.
- reconnect success is reported as media readiness.
- the SDK invents a server resumption contract.
- missed events are assumed replayed without a replay policy.
- platform-specific reconnect behavior changes Signaling semantics.
- reconnect exhaustion is hidden as a normal close.

### C-7 Part C Collapse Conditions

- SDK reconnect mutates server-side state without a command.
- resumption is accepted without a Signaling contract.
- SDK platform wrappers expose different server semantics.
- reconnect evidence omits the server command/reason relation.
- local reconnect is used as durable recovery proof.

---

## Part D. Public API Contract Generation / Projection

### D-1 Boundary

| Concern | Owner | Rule |
|---|---|---|
| Signaling command/event semantics | core/signaling contract | source-of-truth for SDK projection |
| SDK public API shape | sdk | platform idiom allowed, semantics unchanged |
| contract projection rule | sdk + docs governance | mapping table and golden evidence required |
| generated code/artifact | build tooling/sdk | not semantic authority |
| SDK compatibility snapshot | testing/docs support | evidence only when adopted by report |
| migration guide | docs/sdk | not compatibility proof by itself |

The SDK public API MAY differ by platform idiom, but semantic command/event/reason/correlation behavior MUST NOT diverge.

### D-2 Contract Artifact Classes (closed set)

| Class | Meaning | Rule |
|---|---|---|
| `semantic_contract_model` | core-owned Signaling command/event/reason matrix | source-of-truth |
| `platform_api_projection` | SDK API mapping for a platform | must cite the semantic contract model |
| `generated_sdk_type` | generated or mechanical type artifact | not source-of-truth |
| `sdk_golden_snapshot` | deterministic SDK contract fixture | canonical serialization / fixture rule applies |
| `sdk_compatibility_matrix` | supported platform/version behavior | test evidence required |
| `sdk_migration_guide` | user-facing migration document | not proof without tests |

A new artifact class is out of the v0.2 initial scope.

### D-3 Projection Rule (mandatory declaration per projection)

Each SDK public API projection MUST declare:

- source Signaling command/event;
- platform API symbol;
- request/response/callback/event shape;
- correlation propagation rule;
- server reason preservation rule;
- local SDK error wrapper;
- version/capability behavior;
- reconnect/resumption relation;
- unsupported surface behavior;
- out-of-scope feature rejection/projection rule.

Generated code MUST NOT add server-side semantics not present in the source contract.

### D-4 Drift Rule

SDK drift exists when any platform:

- lacks a supported semantic command/event;
- changes server reason preservation;
- changes the correlation rule;
- treats local lifecycle as server state;
- exposes media/regulated/auth issuance as a Signaling SDK responsibility;
- exposes an out-of-scope feature as a supported server capability;
- accepts version/capability differently from the source contract.

Drift blocks parity/readiness claims until resolved or explicitly removed through compatibility/deprecation rules (fail-closed).

### D-5 Failure Mapping (closed-set reasons)

| Failure | Required reason |
|---|---|
| SDK contract generation/projection failed | `sdk_contract_generation_failed` |
| SDK platform contract drift detected | `sdk_contract_drift_detected` |
| Signaling surface has no SDK public mapping | `sdk_public_api_unmapped` |
| SDK golden snapshot mismatch | `sdk_golden_mismatch` |
| platform projection violates Signaling semantics | `sdk_platform_projection_invalid` |
| SDK projection exposes out-of-scope feature without admission | `feature_admission_not_documented` |
| canonical golden encoding cannot be produced | `canonical_serialization_failed` |
| fixture/golden data invalid | `fixture_invalid` |

### D-6 Evidence Rule / Audit Rule (projection)

SDK contract evidence MUST record: platform, source contract version, projection artifact class, command/event matrix, reason preservation test, correlation test, golden snapshot class when used, and close-not-claimed scope. SDK build success alone does not prove semantic parity.

SDK public API contract decisions use audit event type `sdk_public_api_contract_decision`. The event MUST carry the SDK platform, projection class, source contract version, and the `CorrelationId` from the generation or evidence run.

### D-7 Prohibitions / Collapse (projection)

Prohibitions:

- a generated SDK type becomes server semantic authority.
- platform idiom changes command/event meaning.
- the SDK public API owns media, regulated workflow, auth issuance, or server authorization.
- an SDK local error erases the server catalog reason.
- a migration guide is treated as compatibility proof.
- an SDK golden snapshot changes without semantic diff classification.

Collapse Conditions:

- the SDK projection lacks a source Signaling contract reference.
- platform API drift is accepted without a compatibility/deprecation path.
- a generated artifact changes semantics silently.
- SDK contract evidence lacks the command/event/reason/correlation matrix.
- the SDK public API exposes out-of-scope media/regulated/auth responsibilities.

---

## Part E. Native SDK Command Evidence Boundary

### E-1 Context

arcRTC v0.2 handles the SDK public contract across TypeScript / Android / iOS. A source marker test MAY be used for static confirmation of the SDK boundary, but MUST NOT substitute for native command success.

- On Android, the Gradle wrapper, Android SDK, Kotlin/Android Gradle Plugin, compileSdk, unit test, build, and lint are part of the same command evidence surface.
- On iOS, the SwiftPM test command is the native command evidence surface.

### E-2 Android Native Command Evidence Rule

Android native SDK command evidence uses `sdk/android` as the working directory and runs the following simultaneously via the Gradle wrapper command (MUST).

- `:sdk:testDebugUnitTest`
- `:sdk:assembleDebug`
- `:sdk:lintDebug`
- `--warning-mode all`

The Android report MUST carry: exact `Command:`, exact `Working directory:`, `BUILD SUCCESSFUL`, and `Warning/deprecation output: none observed`. The Android lint report MUST NOT contain warnings or errors. The Android command uses the project Gradle wrapper, not the system Gradle.

### E-3 iOS Native Command Evidence Rule

iOS native SDK command evidence is `swift test` with `sdk/ios` as the working directory, and MUST NOT be substituted by a source marker test.

### E-4 TypeScript / Cross-platform Governance Rule

The TypeScript SDK test is closed within the TypeScript package. Cross-platform governance of Android / iOS source projection is handled by native tests or cross-platform governance tests, and MUST NOT be duplicated in the TypeScript package test.

### E-5 Consequences / Claim Scope

- The ambiguous gate label `SDK Android tests` is not used; use `SDK Android unit/build/lint command`.
- Android build success, unit test success, and lint no-issue are native command evidence, not proof of production readiness / live readiness / public distribution / server semantic correctness.
- The native command report parser MUST NOT adopt based on a substring marker alone.

### E-6 Part E Collapse Conditions

- a source marker test substitutes for native command success.
- any of `:sdk:testDebugUnitTest`, `:sdk:assembleDebug`, `:sdk:lintDebug`, `--warning-mode all` is removed from the Android command.
- adopted evidence retains Android lint warnings / errors.
- the command surface mixes system Gradle execution and wrapper execution.
- the TypeScript package test owns the duplicate check of Android / iOS source projection.

---

## Part F. Invariants (summary)

- The SDK is a Signaling-only boundary and does not own media / auth issuance / regulated workflow. These are out-of-scope features, and exposing them without core contract admission is a boundary violation.
- Signaling semantics (command / event / reason / correlation / version negotiation) are invariant under platform; platform changes only the API idiom.
- A server-side rejection reason (catalog category/code) is always preserved through the wrapper, and only `rejected_by_server` / `server_protocol_violation` / `server_unsupported_version` / `closed_by_remote` may carry it.
- SDK-local reconnect / threading / lifecycle / close are not promoted to server-side state (room / participant / SFU / TURN / acceptance) without an accepted Signaling command.
- Session resumption fails closed unless a dedicated Signaling contract defines every item.
- A generated / public SDK API artifact is not source-of-truth; the `semantic_contract_model` is source-of-truth. Drift blocks parity/readiness claims.
- Native command evidence (Android unit/build/lint, iOS swift test) is proof of build/test, not proof of production / live / distribution / server semantic correctness.
- Both `sdk -> regulated` and `regulated -> sdk` are prohibited.
