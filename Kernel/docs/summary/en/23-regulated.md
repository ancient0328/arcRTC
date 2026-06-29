# Regulated: independent boundary, enrichment lifecycle, out-of-scope feature admission

Status: SSOT consolidated edition
Date: 2026-06-28 JST

## Purpose

This chapter specifies, in a fully self-contained form (understandable without opening any other file, source dev-doc, or implementation code), the regulated boundary of the arcRTC v0.2 Kernel. It covers three areas: the regulated independent boundary (optional domain support); the full states of the enrichment lifecycle when referencing core event / audit pointer / non-sensitive tag; and the out-of-scope feature admission / exclusion mechanism (admission decision rules, the handling of every non-goal, and future adoption conditions). The granularity is sufficient for reimplementation from this chapter alone.

arcRTC v0.2 is a generic WebRTC communication foundation. Regulated is optional domain support and is not part of the generic communication foundation. Regulated support (audit enrichment, HIPAA/GDPR helpers, etc.) MUST NOT be mixed into the generic communication core. This chapter fixes the exclusion conditions and future adoption conditions so that chat, recording, screen share, DataChannel application semantics, UI/end-user workflow, media capture workflow, and regulated workflow do not implicitly leak into Signaling / SFU / SDK / entrypoints.

This chapter writes the dependency direction as `A <- B` ("B depends on A"). The only direction that MAY be permitted is `regulated -> core`, and even this is limited to references to an opaque communication event, an audit pointer, or a non-sensitive tag. `core -> regulated`, `drivers -> regulated`, `entrypoints -> regulated`, `sdk -> regulated`, `regulated -> sdk`, `regulated -> drivers`, and `regulated -> entrypoints` are prohibited.

---

## Part A. Regulated Independent Boundary

### A-1 Allowed Dependency (the only permitted direction)

The only dependency direction that MAY be conditionally permitted is:

```text
regulated -> core
```

This dependency is limited to references to an opaque communication event, an audit pointer, or a non-sensitive tag. Regulated does not own the authority of the generic audit event base schema or the quality metric model (MUST NOT).

### A-2 Prohibited Dependencies

The following are prohibited.

```text
core -> regulated
drivers -> regulated
entrypoints -> regulated
sdk -> regulated
regulated -> sdk
regulated -> drivers
regulated -> entrypoints
```

### A-3 Regulated May Own (owned concerns)

Regulated MAY own the following.

- domain-specific enrichment
- compliance mapping helper
- audit pointer mapping
- non-sensitive tag classification
- external regulatory integration helper
- domain-specific tests
- regulatory pointer mapping
- regulated-domain test fixtures

### A-4 Regulated Must Not Own (concerns not owned)

Regulated MUST NOT own the following.

- Signaling protocol semantics
- TURN protocol semantics
- SFU routing semantics
- SDK Signaling contract
- generic audit event base schema
- transport identity model
- token issuance

### A-5 Generic Core Responsibilities (concerns owned by generic core)

The generic core owns only the following.

- communication event model
- opaque reference
- audit event base schema
- token verification boundary
- quality metrics model

### A-6 Data Boundary

The generic core does not hold a sensitive domain payload. Regulated references a core event as an opaque reference and performs any required domain mapping on the regulated side. Core does not import a regulated type (MUST NOT).

### A-7 Part A Collapse Conditions

The judgments of Part A break when:

- core imports a regulated type (`core -> regulated` occurs).
- the SDK depends on regulated (`sdk -> regulated` occurs).
- regulated depends on the SDK (`regulated -> sdk` occurs).
- regulated depends on drivers / entrypoints (`regulated -> drivers` / `regulated -> entrypoints` occurs).
- a domain payload becomes required in a generic audit event.
- a regulated helper holds authority over a communication protocol decision.
- core owns a HIPAA/GDPR rule.
- medical workflow semantics enter the Signaling / TURN / SFU protocol.

---

## Part B. Full States of the Enrichment Lifecycle

### B-1 Boundary

| Surface | Owner | Rule |
|---|---|---|
| communication event semantics | core | Signaling / SFU / TURN / audit semantics |
| opaque communication event reference | core | regulated may reference only |
| audit pointer mapping | regulated | pointer only, no core schema rewrite |
| non-sensitive tag classification | regulated | must not become a core required field |
| domain-specific payload | regulated | never required by the generic core |
| compliance/export helper | regulated | optional support only |

### B-2 Enrichment Lifecycle (all steps)

The v0.2 initial regulated enrichment lifecycle is:

1. core emits a communication event / audit event with opaque references.
2. regulated receives or reads an allowed opaque pointer.
3. regulated maps the pointer to a domain-specific context outside the generic core.
4. regulated MAY emit a regulated-local enrichment record.
5. a regulated-local record MAY reference a core audit pointer but MUST NOT mutate a core audit event.

Regulated enrichment is post-core and optional. It MUST NOT participate in a core accept/reject decision.

### B-3 Allowed Reference Shape

Regulated MAY reference the following.

- `CorrelationId`
- `RoomId` (only as an opaque communication room reference)
- `ParticipantId` (only as an opaque participant reference)
- `AuditEventId` / audit pointer
- a non-sensitive communication tag admitted by the core contract
- a hash-chain record pointer

Regulated MUST NOT require the following.

- raw media payload
- raw token / credential
- SDK platform object
- driver buffer/packet lease
- entrypoint-specific user account object
- a medical or regulated domain payload inside a core event

### B-4 Tag Rule (rule for the non-sensitive tag)

A non-sensitive tag MUST be:

- closed in the regulated-local vocabulary
- not required by the generic core
- not used by core to decide Signaling / SFU / TURN behavior
- safe to expose only in the regulated-local context after regulated validation

If a tag becomes required for generic communication semantics, it is no longer regulated enrichment and requires a core contract update.

### B-5 Audit Relation

Regulated MAY create regulated-local audit/enrichment records. Those records do not replace generic audit events and do not alter audit hash chain semantics. The core audit pointer remains immutable from the regulated perspective.

### B-6 Prohibitions (enrichment lifecycle)

- core imports a regulated type.
- regulated owns a Signaling / SFU / TURN accept/reject decision.
- regulated enrichment mutates a core audit event.
- regulated requires a domain payload in a generic communication event.
- the SDK depends on regulated enrichment.
- drivers/entrypoints call a regulated helper in the generic communication path.
- a regulated tag becomes a hidden authorization policy for core.

### B-7 Part B Collapse Conditions

- regulated enrichment becomes required for generic core operation.
- a domain payload is added to the core audit event schema.
- a regulated pointer changes a core event meaning.
- a regulated helper is wired into the entrypoints/drivers generic path.
- the SDK platform API depends on the regulated enrichment model.

---

## Part C. Out-of-Scope Feature Admission / Exclusion

### C-1 Boundary

| Concern | Owner | Rule |
|---|---|---|
| v0.2 initial feature scope | core contract | generic communication infrastructure only |
| out-of-scope feature request | entrypoints/sdk/drivers at boundary | reject or close-not-claimed with cataloged reason |
| future feature admission | core contract update | owner, boundary, reason, evidence, migration required |
| SDK public surface | sdk | must not expose an out-of-scope feature as supported server behavior |
| demo/local helper | entrypoints/demo | must not become production policy or scope expansion |
| regulated workflow | regulated | optional support only, not generic core |

Absence from v0.2 initial scope is not, by itself, an implementation gap. It becomes a boundary violation only if an out-of-scope feature is exposed, tested, or claimed as a v0.2 capability without core contract admission.

### C-2 Excluded Feature Classes (closed set, every non-goal)

The excluded feature classes of v0.2 initial architecture are limited to the following.

| Class | Meaning | Rule |
|---|---|---|
| `chat_application_semantics` | user/application chat content or workflow | excluded from generic communication core |
| `recording_workflow` | media recording, storage, retrieval, retention workflow | excluded unless a separate scope expansion |
| `screen_share_workflow` | capture/share workflow beyond media routing | excluded unless admitted |
| `datachannel_application_semantics` | SCTP/DataChannel entrypoint protocol semantics | excluded from the Signaling-only SDK and core |
| `ui_end_user_workflow` | user-facing UI workflow | excluded from v0.2 infrastructure scope |
| `media_capture_workflow` | camera/microphone/screen capture permission UX | browser/native driver may observe only the allowed platform boundary |
| `regulated_domain_workflow` | medical/regulated domain workflow | regulated optional support only |

A new excluded/admitted feature class is out of the v0.2 initial scope.

### C-3 Future Admission Rule (future adoption conditions)

Future admission of an excluded feature MUST declare:

- feature class;
- owner package/layer;
- relation to the generic communication core;
- public SDK/API surface, if any;
- driver/runtime dependency boundary;
- security/privacy/redaction boundary;
- reason catalog additions;
- audit event relation;
- evidence class;
- migration/deprecation relation.

Until this exists, the feature MUST fail closed or remain close-not-claimed (fail-closed).

### C-4 Failure Mapping (closed-set reasons)

| Failure | Required reason |
|---|---|
| requested feature is outside v0.2 scope | `feature_out_of_scope` |
| feature admission is not documented in the core contract | `feature_admission_not_documented` |
| chat semantics requested | `chat_not_supported` |
| recording workflow requested | `recording_not_supported` |
| screen share workflow requested | `screen_share_not_supported` |
| DataChannel application semantics requested | `datachannel_not_supported` |
| UI/end-user workflow requested | `ui_workflow_not_supported` |
| media capture workflow requested as core/server feature | `media_capture_not_supported` |
| regulated workflow requested as generic core feature | `regulated_workflow_not_supported` |

### C-5 Audit Rule / Evidence Rule

Out-of-scope feature decisions use audit event type `out_of_scope_feature_decision`. The event MUST carry `CorrelationId` when command-scoped, the feature class, the requested surface, and the rejection reason.

Evidence involving an excluded feature MUST record:

- feature class;
- requested surface;
- whether the feature is rejected, ignored as close-not-claimed, or covered by a separate admitted contract;
- correlation ID when command-scoped;
- expected outcome;
- actual outcome;
- cataloged reason for rejection;
- close-not-claimed scope.

Demo, mock, or test helper behavior does not admit a feature into v0.2 scope (MUST NOT).

### C-6 Prohibitions (out-of-scope feature)

- the SDK exposes an out-of-scope feature as a supported server capability.
- a demo/local helper expands production scope.
- chat/recording/screen share/DataChannel/UI semantics enter the generic core.
- media capture permission state becomes core communication state.
- a regulated workflow becomes required generic communication behavior.
- an out-of-scope feature pass is used as v0.2 readiness evidence.

### C-7 Part C Collapse Conditions

- an excluded feature is implemented without core contract admission.
- the SDK public API claims excluded feature support.
- a demo or local helper becomes scope authority.
- an out-of-scope feature rejection lacks a cataloged reason.
- future admission omits the owner, security/privacy, audit, or evidence boundary.

---

## Part D. Invariants (summary)

- Regulated is optional domain support and is not part of the generic communication foundation. The only permitted dependency direction is `regulated -> core` (references only to an opaque communication event / audit pointer / non-sensitive tag).
- `core -> regulated`, `drivers/entrypoints/sdk -> regulated`, and `regulated -> sdk/drivers/entrypoints` are all prohibited.
- The generic core holds no sensitive domain payload, and core does not import a regulated type.
- Regulated enrichment is post-core and optional; it does not participate in a core accept/reject decision, does not mutate a core audit event, and does not become required for generic core operation.
- A non-sensitive tag is closed within the regulated-local vocabulary; the moment it becomes required by the generic core it is no longer regulated enrichment and requires a core contract update.
- Absence from v0.2 scope is not an implementation gap. The excluded feature classes (chat/recording/screen share/DataChannel/UI/media capture/regulated workflow) become a boundary violation only upon exposure/test/claim without core contract admission, and fail closed with a cataloged reason when requested.
- Future admission fails closed until it declares the owner / core relation / public surface / dependency boundary / security-privacy-redaction / reason catalog / audit / evidence / migration in full.
