# core-protocol-and-serialization

Status: SSOT consolidated edition
Date: 2026-06-28 JST

## Purpose

This chapter internalizes, at a re-implementable level of detail, the arcRTC v0.2 Kernel deterministic / canonical serialization rules, the protocol / contract versioning system, the compatibility / deprecation rules (including the point that v0.1 is not made automatically compatible), the structure and fields of the external wire envelope, and the feature flag / capability / experimental surface lifecycle. This chapter does not claim a concrete serializer implementation, wire API implementation, release plan, or production enablement.

Dependency notation `A <- B` reads as "B depends on A". The layering is `core <- drivers <- entrypoints`. Canonical serialization (evidence/hash input) and external wire encoding are kept separate.

## 1. Canonical Serialization / Deterministic Encoding

So that audit hash-chain, evidence digest, compatibility test, and golden fixture do not acquire a different meaning due to driver/platform/JSON formatting differences, canonical input is separated from external encoding. An external wire format does not automatically become canonical encoding.

### 1.1 Ownership Boundary

| Concern | Owner | Rule |
|---|---|---|
| canonical semantic field set | core | hash/evidence/compatibility target |
| canonical encoding rule | core | deterministic ordering and normalization |
| external wire encoding | driver/sdk | HTTP/JSON/binary/STUN/TURN representation |
| storage row/object format | driver | persistence detail, not canonical semantic input |
| golden fixture serialization | testing docs/support | must declare canonical/external class |
| hash-chain input | core/audit | uses canonical encoding only |

### 1.2 Canonical Field Rule

Canonical encoding MUST define:

- field set
- field order
- absent/null handling
- enum representation
- integer representation
- decimal/ratio precision
- string normalization
- binary digest representation
- timestamp representation
- unknown field handling
- version field inclusion
- redaction before digest

If any item is undefined, that encoding cannot be used for hash-chain or golden compatibility evidence.

### 1.3 Initial Encoding Policy

v0.2 initial architecture fixes only the policy, not a concrete library. A concrete encoding format MUST be selected by a specification update before implementation.

| Data class | Canonical rule |
|---|---|
| audit event hash input | deterministic field order and normalized primitive values required |
| core reason | category/code textual value, no localized text |
| core reference | opaque reference canonical string or bytes with declared representation |
| timestamp for evidence | UTC epoch milliseconds if included |
| duration | normalized milliseconds from the unit/time normalization authority |
| binary payload | raw sensitive payload excluded; digest/reference only if admitted |
| unknown field | ignored only if the compatibility rule says so; otherwise reject |

### 1.4 Canonical Serialization Failure

| Failure | Required reason |
|---|---|
| canonical encoding cannot be produced | `canonical_serialization_failed` |
| canonical verification mismatch | `canonical_serialization_mismatch` |
| external payload cannot decode to core type | `external_decode_failed` |
| external response encoding failed | `external_encode_failed` |
| required canonical field missing | `missing_required_wire_field` |
| unsupported canonical version | applicable unsupported version reason |

### 1.5 Canonical Evidence Rule

Evidence using digest, hash-chain, golden fixture, or compatibility snapshot MUST record: canonical format/version; field set; redaction statement; raw external source class if any; digest/hash value; verification command/procedure; close-not-claimed scope. External JSON pretty-print, DB row order, or log text is not canonical evidence.

## 2. Protocol / Contract Versioning

### 2.1 Versioned Surfaces

| Surface | Owner | Examples |
|---|---|---|
| Signaling contract | core | command / event schema |
| TURN contract | core | request / response semantic model |
| SFU contract | core | endpoint / stream / route / decision model |
| SDK public contract | sdk | platform client API mapped to Signaling contract |
| Audit schema | core | audit event schema |
| Driver wire encoding | drivers | JSON, binary frame, WebSocket, HTTP |

### 2.2 Version Ownership

core owns contract version semantics. drivers own external encoding version. sdk owns public API version but does not change Signaling semantics. SDK public API projection and contract generation follow the SDK contract generation authority.

### 2.3 Compatibility Rule

| Change | Compatibility | Rule |
|---|---|---|
| add optional field with default behavior | compatible | accepted if default is closed and explicit |
| add required field | breaking | new major contract version |
| remove field | breaking | new major contract version |
| change reason code semantics | breaking | prohibited without new version |
| add reason code | conditional | allowed only through catalog update |
| change state transition | breaking | new contract version |
| add driver encoding | compatible | if core contract unchanged |

### 2.4 Negotiation Rule

Version negotiation is core semantics for the protocol contract and driver semantics for the wire format. The negotiation result MUST be one of:

- accepted exact version
- accepted compatible version
- rejected unsupported version

Implicit fallback to an older version is prohibited.

### 2.5 Capability Rule

Capability is not a substitute for version. Capability describes optional behavior within an accepted version. Capability MUST NOT alter required state transition semantics. Capability lifecycle and disabled-capability failure handling follow section 4 of this chapter.

### 2.6 v0.1 Relation

v0.1 protocol shape is not v0.2 version 0. v0.1 shape MAY be referenced only when the v0.2 specification explicitly adopts it.

## 3. Protocol Compatibility / Deprecation

This prevents fail-open of compatibility window, deprecation notice, removal, SDK parity, and driver wire compatibility. Compatibility is not implicit. Any accepted older or compatible version MUST be listed by surface and version range.

### 3.1 Compatibility Boundary

| Concern | Owner | Rule |
|---|---|---|
| core contract compatibility | core | Signaling / SFU / TURN / audit semantic version |
| driver wire compatibility | driver | external encoding version and parser mapping |
| SDK public compatibility | sdk | platform public API while preserving Signaling semantics |
| deprecation decision | specification | removal window and affected surface must be recorded |
| release communication | entrypoints/project docs | does not change domain semantics |
| evidence report | reports | compatibility test result and unsupported version rejection |

### 3.2 Compatibility Surface Matrix

| Surface | Compatibility owner | Required evidence class |
|---|---|---|
| Signaling command/event contract | core | command/event compatibility test |
| SFU media-facing contract | core | route/packet decision compatibility test when implemented |
| TURN contract | core | allocation/permission/relay compatibility test when implemented |
| Audit event schema | core | event encode/decode and hash-chain compatibility test |
| Driver wire encoding | driver | external decode/encode compatibility test |
| SDK public API | sdk | platform parity and server reason preservation test |

The presence of an evidence class in this table does not mean the evidence has been executed.

### 3.3 Deprecation Lifecycle

The deprecation lifecycle is limited to the following order. Skipping any step prohibits claiming deprecation/removal readiness.

1. identify the affected surface and version.
2. record a specification update with reason, owner, and compatibility window.
3. define unsupported-version behavior and the cataloged reason.
4. update SDK parity and driver mapping rules where affected.
5. add a compatibility/negative test plan.
6. record execution evidence in reports when tests are run.
7. remove support only after the documented removal condition is satisfied.

### 3.4 Fail-Closed Compatibility Rule

An unsupported version MUST be rejected with a cataloged reason. Silent fallback to an older version is prohibited. Best-effort decode of removed fields is prohibited unless a compatibility rule explicitly maps the removed/optional field.

| Failure | Required reason |
|---|---|
| Signaling command/event version unsupported | `unsupported_command_version` |
| media-facing SFU/transport contract unsupported | `unsupported_media_contract_version` |
| TURN contract version unsupported | `unsupported_turn_contract_version` |
| driver wire encoding unsupported | `unsupported_driver_wire_version` |
| required wire field missing after compatibility mapping | `missing_required_wire_field` |
| external enum cannot map after compatibility mapping | `external_enum_unmapped` |

### 3.5 SDK Compatibility Rule

SDK platform compatibility MAY preserve API shape through wrappers, but cannot: alter Signaling command/event semantics; hide server reason category/code; create platform-only server state; expose SFU/TURN/media semantics as the SDK Signaling contract; claim compatibility without contract tests for all supported platforms.

### 3.6 v0.1 Relation

v0.1 protocol shape is historical input only. v0.1 compatibility requires explicit v0.2 adoption by surface and version. Absent explicit adoption, v0.1 shape is not an accepted v0.2 compatible version.

## 4. Feature Flag / Capability / Experimental Lifecycle

So that configuration flag, protocol capability, driver selection, and experimental feature are not confused and core semantics do not implicitly branch, owner and lifecycle are fixed. A feature flag is not a substitute for a protocol version. Capability is not permission to change required state transition semantics.

### 4.1 Ownership Boundary

| Surface | Owner | Rule |
|---|---|---|
| core protocol capability | core | optional behavior within accepted version only |
| feature flag value | entrypoints/config | selected driver/exporter/profile gating |
| driver implementation selection | entrypoints | does not change core semantics |
| SDK capability exposure | sdk | preserves server capability semantics |
| experimental lifecycle decision | specification | scope, evidence, removal/adoption condition |
| out-of-scope feature admission | specification | excluded feature cannot be enabled by flag alone |
| runtime enablement evidence | reports | profile and correlation required |
| runtime flag/profile change | runtime reconfiguration authority | generation and apply scope required |

### 4.2 Flag Classes (closed set)

| Flag class | Allowed use | Prohibited use |
|---|---|---|
| `driver_selection` | choose concrete implementation | change core decision |
| `exporter_selection` | enable metrics/log/audit sink implementation | change audit meaning |
| `profile_selection` | choose config profile/bundle | bypass required validation |
| `experimental_surface_gate` | gate an explicitly documented experimental path | silently alter stable contract |
| `test_only_gate` | enable fake/deterministic support | runtime/prod evidence |

A new flag class requires a specification update.

### 4.3 Capability Rule

Capability MUST declare: surface; owning layer; accepted contract version; optional behavior; required fallback when capability absent; reason when capability is required but absent; SDK parity requirement when client-visible; evidence class required before adoption. When a required capability is absent, use `capability_not_enabled` unless a more specific unsupported-version reason applies.

### 4.4 Experimental Lifecycle Stages

A stage name is documentation classification, not a runtime success claim.

| Stage | Meaning | Promotion condition |
|---|---|---|
| `draft_documented` | a draft specification exists | scope and owner fixed |
| `gated_scaffold` | scaffold exists behind explicit gate | dependency direction evidence |
| `gated_implemented` | implementation exists behind explicit gate | unit/contract evidence |
| `controlled_integration` | selected integration evidence exists | integration report |
| `adopted_contract` | promoted into normal contract | specification update and compatibility rule |
| `removed` | support removed | compatibility/deprecation lifecycle satisfied |

### 4.5 Feature / Capability Failure Mapping

| Failure | Required reason |
|---|---|
| required capability absent | `capability_not_enabled` |
| protocol version unsupported | surface-specific unsupported version reason |
| feature/profile configuration invalid | `runtime_config_invalid` or `core_policy_config_invalid` |
| experimental surface used without gate | `capability_not_enabled` |
| out-of-scope feature used without admission specification | `feature_admission_not_documented` |
| test-only gate used outside test evidence | evidence rejection, not runtime success |
| runtime flag change not admitted | `runtime_reconfiguration_not_allowed` |

## 5. External Wire Protocol Envelope

Wire encoding is driver-owned, but the semantic envelope that reaches core MUST have been converted to core-owned command / event / reason / reference. Canonical serialization (section 1) is separate from external wire encoding; an external wire representation does not become canonical serialization unless the specification explicitly admits the exact field set/order/normalization/version.

### 5.1 Wire Envelope Boundary

| Surface | Owner | Rule |
|---|---|---|
| JSON / binary / HTTP / WebSocket / UDP / TCP frame | driver | external wire encoding |
| STUN / TURN wire message bytes | driver | decode/encode implementation |
| SDK platform message codec | sdk | Signaling public contract wrapper |
| semantic command/event | core | core-owned type only |
| reason category/code | core | section 4 of this chapter and the chapter 04 reason catalog |
| protocol version and capability semantics | core | sections 2 and 4 of this chapter |

The external envelope shape does not become the core API. The core semantic envelope does not include driver concrete frame/request/response objects.

### 5.2 Semantic Envelope Fields

Any command/event that crosses the driver/core boundary MUST be represented by a core-owned semantic envelope with the following field classes.

| Field class | Required | Owner | Rule |
|---|---|---|---|
| `surface` | yes | core | closed set: signaling, sfu, turn, transport, driver |
| `contract_version` | yes | core | version semantics follow section 2 |
| `correlation_id` | yes after boundary | core | missing/invalid pre-core follows driver conversion rule |
| `command_or_event_type` | yes | core | closed set per contract specification |
| `subject_reference` | conditional | core | RoomId, ParticipantId, AllocationId, etc. when naturally materialized |
| `payload_model` | conditional | core | core-owned payload type, not wire object |
| `reason` | conditional | core | required for non-success outcome |

A driver MAY carry external metadata outside this envelope, but that metadata is non-authoritative unless admitted by the specification.

### 5.3 External Envelope Rule

The external wire envelope MAY include transport-specific fields (HTTP method/path/status, WebSocket message type/close code, JSON object field names, binary frame tag, UDP peer address, STUN/TURN method and attribute encoding). These are driver-owned and MUST be mapped into the semantic envelope or rejected before core entry.

### 5.4 Error Response Rule

A core rejection / failure MUST be exposed externally without changing the semantic reason.

| Core outcome | External mapping rule |
|---|---|
| accepted / allowed / forwarded / selected | driver may map to success response |
| rejected / denied / suppressed / dropped / expired / failed | driver must preserve cataloged reason category/code in a traceable external response when a response is emitted |
| protocol violation before core entry | driver emits conversion/resource failure audit and may map to external error response |

HTTP status, WebSocket close code, SDK error wrapper, or STUN/TURN error code is not the authoritative reason. The authoritative reason is the core catalog category/code or the driver conversion reason recorded before core entry. Detailed external error projection follows section 11 of chapter 04.

### 5.5 Correlation Rule

A driver validates or creates only the allowed correlation boundary.

- If an external command carries a valid client correlation ID, the driver maps it to core `CorrelationId`.
- If an external command lacks the required correlation ID, the driver fails before core entry with `missing_correlation_id`.
- A driver MUST NOT synthesize a fake client correlation ID for a failed client command.
- For startup / configuration paths, `StartupRunId` and `ConfigurationScopeRef` are used (the identity rules in chapter 03).

After a command crosses the driver/core boundary, `CorrelationId` is mandatory.

### 5.6 Version Rule

External wire version and core contract version MUST be mapped explicitly. When the version field was successfully decoded, a version mismatch MUST NOT be treated as a generic decode failure.

| Failure | Required reason |
|---|---|
| unsupported driver wire version | `unsupported_driver_wire_version` |
| unsupported Signaling command version | `unsupported_command_version` |
| unsupported TURN contract version | `unsupported_turn_contract_version` |
| unsupported media-facing contract version | `unsupported_media_contract_version` |

### 5.7 Wire Envelope Failure Mapping

| Failure | Required reason |
|---|---|
| external payload cannot decode to core type | `external_decode_failed` |
| missing required wire field | `missing_required_wire_field` |
| external enum has no core mapping | `external_enum_unmapped` |
| external type leakage detected | `external_type_leak_blocked` |
| external response encoding failed | `external_encode_failed` |
| frame size bound exceeded | `frame_size_bound_exceeded` |

A pre-core failure does not enter the domain state machine.

## 6. Prohibitions

- driver wire encoding is treated as canonical encoding by default.
- free-text/log formatting affects hash input.
- unknown fields are accepted without a compatibility rule.
- raw sensitive payload is included in the canonical digest.
- floating point values are encoded without a precision rule.
- a hash-chain record uses non-deterministic field order.
- driver wire version is treated as the core protocol version.
- SDK platform version changes Signaling semantics.
- an unsupported version falls back silently.
- a breaking change is introduced without a new contract version.
- v0.1 message shape is accepted as a v0.2 contract without specification adoption.
- a deprecation notice lacks affected surface/version.
- removal occurs without a specification update.
- the SDK keeps old behavior by changing server semantics.
- a driver accepts removed fields as authoritative core semantics without a compatibility rule.
- a compatibility claim relies only on release notes without tests/evidence.
- a feature flag changes the core state machine without a specification update.
- a capability alters required behavior inside an accepted version.
- an experimental surface is enabled by default.
- an out-of-scope feature is enabled by flag without specification admission.
- the SDK exposes a capability that the server contract does not define.
- a test-only gate is used as runtime/prod evidence.
- removal bypasses the protocol compatibility/deprecation lifecycle.
- a JSON / HTTP / WebSocket / STUN / TURN wire object is used as the core command type.
- an external status code replaces the core reason.
- a driver maps a rejected core decision to a successful external semantic response.
- a version mismatch is hidden as a generic driver failure.
- a missing correlation ID is repaired with a fake client ID.
- an SDK platform envelope defines server-side semantics.
- external error mapping exposes unsafe reason detail or loses cataloged reason traceability.
- external wire encoding is treated as canonical evidence encoding without a deterministic serialization rule.

## 7. Fail-Closed / Invariants

- An unsupported version is rejected with a cataloged reason (no silent fallback).
- If any canonical field set item is undefined, it cannot be used for hash-chain/golden evidence.
- When canonical encoding cannot be produced, fail closed with `canonical_serialization_failed`.
- `CorrelationId` is mandatory after the core boundary; a pre-core absence fails with `missing_correlation_id`.
- A pre-core conversion failure does not enter the domain state machine.
- An absent required capability does not fall back silently; use `capability_not_enabled`.
- An experimental surface is not enabled by default.

## 8. Collapse Conditions

- the same semantic event can produce a different canonical hash on different drivers.
- the canonical field set is implicit.
- an external storage/wire shape becomes canonical semantic input without a specification update.
- digest evidence lacks format/version.
- a canonical mismatch is ignored while claiming evidence validity.
- driver wire version is treated as the core protocol version.
- SDK platform version changes Signaling semantics.
- an unsupported version falls back silently.
- a breaking change is introduced without a new contract version.
- v0.1 message shape is accepted as a v0.2 contract without explicit adoption.
- accepted compatible versions are not enumerated.
- an unsupported version is not rejected with a cataloged reason.
- SDK/platform compatibility changes core semantics.
- removal occurs without a documented compatibility window and evidence plan.
- SDK public API drift is accepted without a compatibility/deprecation rule.
- a flag value becomes hidden domain policy.
- an absent capability falls back silently.
- an experimental stage is treated as runtime readiness.
- SDK/platform capability diverges from server semantics.
- feature removal occurs without compatibility/deprecation evidence.
- a feature flag is used as the admission authority for an excluded feature.
- a runtime feature/capability switch lacks reconfiguration generation and apply scope.
- a wire frame shape becomes the core API.
- the core reason category/code is lost in external error mapping.
- a pre-core conversion failure enters the domain state machine.
- the correlation rule differs per driver without a specification update.
- protocol version semantics are owned by the driver.
- a canonical evidence hash depends on driver wire formatting.
