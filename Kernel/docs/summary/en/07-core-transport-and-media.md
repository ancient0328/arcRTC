# core-transport-and-media

Status: SSOT consolidated edition
Date: 2026-06-28 JST

## Purpose

This chapter internalizes the current complete specification of the pure transport contract owned by `core/transport` of arcRTC v0.2 Kernel, together with SDP/ICE negotiation, ICE candidate policy / connectivity lifecycle, media codec/track/layer negotiation, packet rewrite / media transform boundary, and secure media session lifecycle, at a granularity sufficient for reimplementation from this chapter alone. This chapter makes the Sans-IO semantics core-owned and separates I/O implementations such as socket / runtime / str0m concrete type / browser/native WebRTC API as driver-owned. This chapter does not claim concrete SDP grammar, ICE agent, DTLS/SRTP runtime, codec implementation, transcoding, or media engine runtime as implemented.

Dependency direction notation: `A <- B` means "B depends on A". The pure semantics of Signaling / SFU / TURN are owned by core and separated from I/O.

---

## 1. core transport contract (Sans-IO transport contract)

### 1.1 Boundary (ownership)

`core/transport` MUST own the following.

- core-owned type of transport command / event
- abstract representation of WebRTC transport capability
- abstract representation of SDP / ICE-adjacent semantic fragment
- connection point to RTP / RTCP packet semantic view
- transport-level version / capability negotiation semantics
- connection to transport failure reason

driver MUST own the following.

- str0m concrete event / state / error
- UDP / TCP / HTTP / WebSocket socket I/O
- browser / native WebRTC API
- DTLS / SRTP / ICE stack implementation detail
- concrete SDP string encoding / parsing backend
- runtime task / worker execution

The transport contract is Sans-IO semantics and MUST NOT own socket, runtime, str0m concrete type, or browser/native WebRTC API.

### 1.2 Core Transport Types (closed set)

| Type family | Owner | Rule |
|---|---|---|
| `TransportCommand` | core | abstract command toward driver implementation |
| `TransportEvent` | core | external transport event converted into core meaning |
| `TransportCapability` | core | semantics of capability negotiation |
| `SessionDescriptionRef` | core | opaque/validated semantic reference, not the SDP payload itself |
| `IceCandidateRef` | core | core-owned reference of candidate relay / negotiation intent |
| `PacketSemanticView` | core | borrowed packet view needed for routing |

`SessionDescriptionRef` and `IceCandidateRef` do not authorize core to parse or retain external concrete library types. They do not create cross-plane binding by themselves.

### 1.3 SDP / ICE Rule (transport level)

- Signaling MAY relay offer / answer / ICE candidate intent.
- core transport MAY validate version, correlation, and policy boundaries of that intent.
- Concrete SDP grammar parsing, ICE agent execution, connectivity checks, and candidate gathering are driver-owned implementation details (MUST).
- If driver cannot map external SDP / ICE material into the core-owned reference shape, it MUST fail before core entry (driver conversion boundary).

### 1.4 RTP / RTCP Rule (transport level)

- RTP / RTCP raw bytes are driver-owned (MUST).
- core transport and core SFU MAY observe only borrowed semantic views (per the packet buffer lifetime rules in §4/§7).
- core transport MUST NOT retain packet bytes, buffer lease, packet cache, transmit queue, parser crate object, or str0m packet object.

### 1.5 Version / Capability Rule

- transport-facing version / capability negotiation MUST connect to the protocol version system.
- Unsupported transport contract version MUST use `unsupported_media_contract_version` or the applicable protocol-specific reason.
- transport version mismatch MUST NOT be treated as a retryable free-text error.

### 1.6 Driver Mapping Rule

- `drivers/webrtc-str0m` implements the WebRTC transport port.
- `drivers/network` supplies concrete network I/O.
- Neither driver MAY define alternate transport semantics.
- When str0m emits an event, the driver MUST map it to a core-owned `TransportEvent` or fail conversion with a cataloged reason.
- When core emits a `TransportCommand`, the driver MAY translate it to str0m / browser / native / network actions, but MUST NOT change the domain decision.

### 1.7 Prohibitions (core transport contract)

- core imports `str0m::*`.
- core imports socket / runtime / browser / native WebRTC concrete type.
- driver exposes concrete SDP / ICE parser objects as core API.
- core stores raw RTP / RTCP bytes.
- driver decides Signaling / SFU / TURN accept or reject based on transport implementation convenience.
- transport reference or connectivity observation creates cross-plane binding by itself.
- transport version mismatch is treated as a retryable free-text error.

### 1.8 fail-closed / collapse conditions (core transport contract)

The judgment of this contract collapses when any of the following holds.

- core transport becomes a str0m wrapper.
- SDP / ICE concrete type crosses into core.
- ICE connectivity or secure media readiness is inferred from Signaling relay or listener startup.
- packet bytes or buffer leases become core-owned.
- driver transport event changes core domain decision semantics.
- transport reference or event bypasses cross-plane binding rule.
- unsupported transport version lacks cataloged reason.

---

## 2. SDP / ICE Negotiation Boundary

This section subdivides the owner and fail-closed conditions of SDP offer/answer, ICE candidate, and capability/version negotiation. core does not own SDP string, ICE candidate string, str0m event, or browser/native WebRTC concrete type; core owns only the semantics of `SessionDescriptionRef`, `IceCandidateRef`, and capability/version/correlation.

### 2.1 Boundary (owner)

| Concern | Owner | Rule |
|---|---|---|
| offer / answer relay intent | core/signaling | evaluates room/participant/correlation/version boundary |
| ICE candidate relay intent | core/signaling | evaluates accepted participant and ordering boundary |
| transport capability semantics | core/transport | evaluates accepted version and capability relation |
| media codec/track/layer capability semantics | core/transport + core/sfu | accepted media contract and SFU intent |
| SDP grammar parse / serialize | driver | concrete parser/backend detail |
| ICE gathering / connectivity check | driver | concrete stack detail |
| ICE candidate exposure / consent observation | core policy + driver observation | detailed lifecycle per §3 |
| DTLS / SRTP setup | driver | crypto/session implementation detail |
| SDK public API mapping | sdk | maps Signaling-only contract to platform API |
| executable wiring | entrypoints | assembly of selected driver and typed configuration |

### 2.2 Negotiation Flow (inbound order, MUST)

Inbound negotiation material MUST follow this order.

1. driver or sdk boundary receives external SDP/ICE material.
2. driver enforces wire shape, size, required field, driver wire version, and concrete decode.
3. driver maps external material to core-owned reference shape.
4. core/signaling evaluates room/participant/order/version/correlation.
5. core returns accepted relay, rejected relay, or protocol violation decision.
6. driver/sdk maps the core decision to external response/event without changing reason.

- Concrete ICE connectivity success or media transport readiness is NOT proven by Signaling relay acceptance (MUST).
- Offer/answer/ICE relay acceptance does not bind Signaling participant, SFU endpoint, TURN allocation, or secure media session without an admitted cross-plane binding (MUST).

### 2.3 Version / Capability Rule

The separation between protocol contract version and driver wire encoding version MUST be preserved.

| Version surface | Owner | Failure reason |
|---|---|---|
| Signaling command/event version | core | `unsupported_command_version` |
| media-facing transport contract version | core/transport | `unsupported_media_contract_version` |
| driver wire encoding version | driver | `unsupported_driver_wire_version` |
| SDK public API version | sdk | SDK-local mapping, preserving server reason |

- Capability does not replace version.
- Capability MAY select optional behavior inside an accepted version, but MUST NOT change required Signaling state transitions or SFU routing semantics.
- Codec/track/layer capability MUST NOT be inferred from SDP string alone; it MUST pass media negotiation mapping.

### 2.4 Ordering Rule

- offer/answer/ICE relay ordering is core Signaling semantics.
- driver MAY preserve receive order, but MUST NOT decide command ordering validity.
- Rejected ordering uses `command_order_violation`.
- Duplicate command uses `duplicate_command` when idempotency semantics rejects a duplicate.

### 2.5 Failure Mapping (SDP/ICE negotiation)

| Failure | Required reason |
|---|---|
| external SDP/ICE material cannot decode to reference shape | `external_decode_failed` |
| candidate class or exposure policy rejected | `ice_candidate_policy_violation` |
| candidate connectivity or consent observation failed | `ice_connectivity_check_failed` or `ice_consent_expired` |
| required wire field absent | `missing_required_wire_field` |
| external enum or attribute class cannot map to core reference | `external_enum_unmapped` |
| Signaling command version unsupported | `unsupported_command_version` |
| media-facing contract version unsupported | `unsupported_media_contract_version` |
| media codec/profile unsupported | `media_codec_not_supported` |
| media payload/track/layer mapping invalid | `media_payload_mapping_invalid` |
| participant not joined | `participant_not_joined` |
| command arrives in invalid order | `command_order_violation` |
| room is draining | `room_draining` |
| room is closed | `room_closed` |
| network send failed | `network_send_failed` |
| network receive failed | `network_receive_failed` |
| driver shutdown | `driver_shutdown` |
| required cross-plane binding absent or invalid | cross-plane binding reason |

Driver-local parser/backend errors MUST NOT become free-text core reasons.

### 2.6 SDK Parity Rule

TypeScript / Android / iOS SDKs MAY expose different platform idioms, but they MUST preserve:

- the same semantic command/event set for offer / answer / ICE candidate relay;
- the same correlation propagation;
- the same server reason category/code for rejection;
- the same version negotiation behavior;
- the same Signaling-only boundary.

SDK MUST NOT expose PeerConnection / media readiness as server-side Signaling success.

### 2.7 Prohibitions (SDP/ICE negotiation)

- core parses concrete SDP string.
- core stores ICE candidate concrete parser object.
- driver accepts offer/answer/ICE ordering by local convenience after core rejection.
- Signaling relay acceptance is reported as DTLS/SRTP/media readiness.
- SDP codec/payload field is treated as accepted SFU media negotiation by itself.
- SDP/ICE relay success is treated as cross-plane binding by itself.
- SDK platform difference changes offer/answer/ICE command semantics.
- unsupported version falls back silently.

### 2.8 collapse conditions (SDP/ICE negotiation)

- SDP/ICE concrete type crosses into core.
- driver wire version is treated as core protocol version.
- ICE connectivity result is inferred from Signaling relay event.
- ICE candidate exposure/restart/connectivity/consent lifecycle bypasses §3.
- malformed negotiation material reaches core as concrete external type.
- server rejection reason is erased by SDK/platform mapping.
- media negotiation support is implied without media negotiation rule.
- cross-plane binding is inferred from offer/answer/ICE correlation alone.

---

## 3. ICE Candidate Policy / Connectivity Lifecycle

This section fixes SDP/ICE relay intent, candidate gathering, candidate exposure policy, relay-only policy, ICE restart, connectivity check, and consent freshness so they are not conflated with Signaling relay acceptance or driver implementation detail. Signaling acceptance of an ICE candidate relay does not prove connectivity (MUST).

### 3.1 Boundary (owner)

| Concern | Owner | Rule |
|---|---|---|
| candidate relay intent | core/signaling | evaluates room, participant, ordering, correlation |
| candidate exposure policy | core/transport policy + entrypoints configuration | fixes host/srflx/relay/mDNS/redaction rule |
| candidate gathering/execution | driver/WebRTC backend | concrete ICE agent detail |
| connectivity check observation | driver | runtime observation, not Signaling success |
| consent freshness observation | driver | observation converted to core-owned event/reason |
| ICE restart authorization | core/signaling + core/transport | restart intent and allowed state |
| evidence | reports | connectivity class and scope required |

### 3.2 Candidate Classes (closed set)

The candidate classes of v0.2 initial architecture are limited to the following. A new class requires a specification update.

| Class | Meaning | Rule |
|---|---|---|
| `host_candidate_ref` | host candidate reference after policy | raw address exposure requires explicit policy |
| `srflx_candidate_ref` | server-reflexive candidate reference | not identity proof |
| `relay_candidate_ref` | TURN relay candidate reference | TURN allocation/permission relation required when claimed |
| `mdns_candidate_ref` | mDNS-obfuscated candidate reference | raw local address must remain hidden |
| `trickle_candidate_ref` | incremental candidate relay | ordering/correlation required |
| `ice_restart_intent` | restart negotiation intent | prior state and restart policy required |
| `connectivity_observation` | driver observed connectivity result | diagnostic/evidence class, not relay acceptance |
| `consent_freshness_observation` | driver observed consent state | closed reason when expired/failed |

### 3.3 Policy Rule

ICE policy MUST declare:

- accepted candidate classes;
- relay-only or host/srflx allowance;
- mDNS / raw address exposure rule;
- candidate size and field validation;
- trickle ordering rule;
- ICE restart allowance;
- connectivity/consent observation mapping;
- audit event relation.

If candidate policy is absent where required, candidate handling MUST fail closed.

### 3.4 Failure Mapping (ICE candidate/connectivity)

| Failure | Required reason |
|---|---|
| candidate class not allowed by policy | `ice_candidate_policy_violation` |
| candidate cannot map to core reference | `ice_candidate_mapping_invalid` |
| candidate exposes address/material requiring redaction | `ice_candidate_redaction_required` |
| candidate gathering failed before relay evidence | `ice_gathering_failed` |
| connectivity check failed | `ice_connectivity_check_failed` |
| consent freshness expired or failed | `ice_consent_expired` |
| ICE restart not allowed in current state | `ice_restart_not_allowed` |
| external ICE material cannot decode | `external_decode_failed` |
| command ordering invalid | `command_order_violation` |
| network receive failed | `network_receive_failed` |
| network send failed | `network_send_failed` |

### 3.5 Audit / Evidence Rule

- ICE candidate / connectivity decisions use audit event type `ice_candidate_connectivity_decision`. The event MUST carry `CorrelationId`, candidate/connectivity class, policy reference, participant/room reference when materialized, and observation class when driver connectivity evidence is used.
- ICE evidence MUST record: candidate/connectivity class; accepted policy class; correlation ID; participant/room reference when materialized; relay-only or exposure policy result; trickle/restart relation when relevant; connectivity/consent observation class when claimed; expected outcome; actual outcome; cataloged reason for non-success; close-not-claimed scope.
- Candidate relay evidence does not prove connectivity or consent freshness. Relay candidate evidence does not prove TURN allocation/permission or SFU endpoint binding without admitted cross-plane binding and target-plane evidence.

### 3.6 Prohibitions (ICE candidate/connectivity)

- raw host address exposure is accepted without policy.
- ICE connectivity success is inferred from Signaling relay acceptance.
- driver ICE backend decides room/participant ordering.
- ICE restart silently recreates room, participant, SFU, or TURN state.
- ICE candidate or connectivity observation is treated as cross-plane binding without binding class.
- mDNS candidate is expanded into raw local address in audit/log/report.
- connectivity observation is used as production readiness without evidence scope.

### 3.7 collapse conditions (ICE candidate/connectivity)

- candidate exposure policy is implicit.
- ICE restart has no authorization/state rule.
- consent freshness failure is recorded as generic network failure only.
- candidate relay and connectivity evidence are conflated.
- driver ICE object enters core state.
- ICE evidence is used as TURN/SFU/secure media binding evidence without cross-plane binding rule.

---

## 4. Media Codec / Track / Layer Negotiation

This section fixes the owner and fail-closed conditions so that SDP, RTP payload type, SSRC, RID/MID, simulcast/SVC, RTX/FEC, and RTCP feedback are not conflated with SFU routing semantics or driver parser detail. This section does not claim codec implementation, transcoding, media engine runtime, or browser/native media readiness.

### 4.1 Boundary (owner)

| Concern | Owner | Rule |
|---|---|---|
| media-facing contract semantics | core/transport + core/sfu | accepted capability and routing intent |
| codec implementation | driver/browser/native/webrtc backend | concrete encoder/decoder detail |
| SDP/RTP payload mapping | driver conversion + core accepted reference | parser detail cannot become semantic authority alone |
| publication/subscription track policy | core/sfu | accepted track/layer decision |
| simulcast/SVC layer selection | core intent, driver execution | no silent driver-only subscription semantics |
| RTX/FEC/RTCP feedback parsing | driver conversion | core sees admitted feedback/reference only |
| transcode/media transform execution | driver | prohibited unless §5 admits |

Payload type, SSRC, RID, MID, or codec string is not authoritative core identity until mapped into accepted core-owned references (MUST). Track publication/subscription mapping does not bind a Signaling participant to an SFU endpoint unless the cross-plane binding rule admits that relation (MUST).

### 4.2 Negotiation Classes (closed set)

The media negotiation classes of v0.2 initial architecture are limited to the following. A new class requires a specification update.

| Class | Meaning | Rule |
|---|---|---|
| `codec_capability_offer` | offered codec/profile capability | core may accept/reject semantic capability |
| `track_publication_offer` | endpoint wants to publish a media track | maps to SFU publication decision |
| `track_subscription_request` | endpoint wants to subscribe to media track/layer | maps to SFU subscription decision |
| `payload_type_mapping` | external RTP payload type to accepted media reference | driver maps, core validates |
| `simulcast_layer_selection` | RID/layer selection intent | core selection, driver execution |
| `svc_layer_selection` | SVC dependency/layer selection intent | explicit support required |
| `rtx_fec_feedback_support` | retransmission/recovery support | driver execution with core policy relation |

### 4.3 Mapping Rule

Codec/track/layer mapping MUST define:

- media contract version;
- accepted codec/profile set;
- payload type mapping rule;
- SSRC/RID/MID mapping rule;
- track and stream reference relation;
- cross-plane binding relation when track/stream claim crosses Signaling/SFU/TURN/secure media planes;
- layer selection rule;
- unsupported feature reason;
- audit/event relation.

External SDP/RTP fields remain untrusted until mapped and accepted (MUST).

### 4.4 Transcode Rule

- Initial v0.2 does not define transcoding as core behavior.
- If transcoding / payload transform / codec rewrite / media normalization is required, §5 defines the admission boundary, and a separate future specification MUST define exact owner, resource bounds, quality relation, privacy boundary, and evidence rule.
- Header rewrite intent MAY be returned by core only as routing/forwarding intent. Actual RTP/RTCP byte rewrite remains driver-owned (MUST).

### 4.5 Failure Mapping (media negotiation)

| Failure | Required reason |
|---|---|
| codec/profile not supported | `media_codec_not_supported` |
| track publication/subscription not allowed | `media_track_not_allowed` |
| requested media layer unavailable | `media_layer_not_available` |
| payload type / SSRC / RID / MID mapping invalid | `media_payload_mapping_invalid` |
| RTCP feedback / RTX / FEC support not available | `media_feedback_not_supported` |
| transcoding or media transform requested but not supported | `media_transcode_not_supported` |
| media-facing contract version unsupported | `unsupported_media_contract_version` |
| external negotiation material cannot decode | `external_decode_failed` |

### 4.6 Audit / Evidence Rule

- Media negotiation decisions use the target event types already defined for SFU publication/subscription/forwarding/protocol violation. When the decision is specifically about media negotiation mapping before a target SFU decision, audit event type `media_negotiation_decision` is used.
- Evidence for media negotiation claims MUST include: media contract version; negotiation class; accepted codec/profile or rejected capability; payload type / SSRC / RID / MID mapping result when relevant; track/layer reference when materialized; SFU target decision event or `media_negotiation_decision`; closed reason for unsupported codec, disallowed track, unavailable layer, invalid mapping, unsupported feedback, or unsupported transcode request.
- Browser/native media readiness evidence does not prove SFU routing semantics unless connected to the core media negotiation decision. Media negotiation evidence does not prove Signaling participant, TURN permission, ICE connectivity, or secure media binding without cross-plane binding evidence (MUST).

### 4.7 Prohibitions (media negotiation)

- payload type alone becomes codec authority.
- driver parser decides publication/subscription authorization.
- SDK exposes codec/media readiness as Signaling success.
- unsupported simulcast/SVC layer is silently downgraded without reason.
- transcoding is implied by routing decision.
- RTP/RTCP parser object enters core domain state.
- media negotiation success is treated as cross-plane participant/endpoint/session binding.

### 4.8 collapse conditions (media negotiation)

- SDP/RTP field value becomes core identity without mapping.
- codec support differs by driver without a specification update.
- media layer selection is hidden inside driver execution.
- transcode/media transform appears without owner/resource/evidence rule.
- SFU route decision depends on concrete codec implementation object.
- packet rewrite/media transform executes without §5.
- cross-plane binding is inferred from codec/track/layer negotiation alone.

---

## 5. Packet Rewrite / Media Transform Boundary

This section fixes the boundary of packet rewrite, header rewrite, SSRC/sequence mapping, payload transform, and media transform. It defines the boundary between the rewrite/transform intent that core may return and the byte operation that driver executes, and does not claim transcoding implementation, media engine runtime, payload transform backend, or production media transform readiness.

### 5.1 Boundary (owner)

Packet rewrite / media transform is the process where the driver converts packet bytes into a transmittable form after or during SFU routing/forwarding intent. core MAY return the semantic intent of rewrite/transform, but does not own packet bytes, payload bytes, codec engine, encryption/framing backend, or buffer ownership (MUST).

| Concern | Owner | Rule |
|---|---|---|
| route/target selection | core/sfu | forwarding target and suppression decision |
| rewrite intent | core/sfu (when semantic) | closed intent class only |
| header byte rewrite | driver | concrete RTP/RTCP byte mutation |
| payload transform execution | driver | prohibited unless class is admitted |
| codec transcode execution | driver/media backend | out of initial v0.2 unless the specification admits |
| copy/allocation strategy | driver | must not leak copied buffer to core |
| evidence/reporting | reports | intent/execution/copy class and failure reason required |

### 5.2 Rewrite / Transform Classes (closed set)

The rewrite/transform classes of v0.2 initial architecture are limited to the following. A new class requires a specification update.

| Class | Meaning | Rule |
|---|---|---|
| `no_rewrite_forward` | original packet lease can be forwarded unchanged | copy prohibited by default |
| `header_rewrite_only` | driver rewrites RTP/RTCP header fields without payload transform | header-only allocation or scatter/gather preferred |
| `sequence_number_mapping` | driver applies core-owned sequence mapping intent | mapping intent must be closed and target-scoped |
| `ssrc_rewrite` | driver rewrites SSRC based on accepted route/stream reference | must not change stream identity semantics |
| `rtcp_feedback_rewrite` | driver maps admitted RTCP feedback reference | parser detail remains driver-owned |
| `framing_security_backend_copy` | encryption/framing backend requires driver-local copy | diagnostic unless evidence records backend class |
| `payload_transform_requested` | payload bytes must be modified | rejected unless a future specification admits exact class |
| `codec_transcode_requested` | codec decode/encode/transcode is required | rejected in initial v0.2 |

### 5.3 Intent Rule

Core rewrite intent MUST record:

- route or forwarding decision reference;
- target endpoint/route reference when materialized;
- rewrite/transform class;
- source stream/packet semantic reference;
- target stream/packet semantic reference;
- mapping table reference when sequence/SSRC mapping is used;
- copy allowance class;
- failure reason mapping;
- audit event relation.

Core intent MUST NOT include raw bytes, mutable packet slice, codec engine object, driver buffer lease, OS buffer type, or backend-specific frame object.

### 5.4 Driver Execution Rule

Driver MAY execute only the admitted class associated with the current forwarding decision (MUST). Driver execution MUST preserve:

- original buffer lease ownership;
- target-specific rewrite isolation;
- payload non-copy default;
- bounded allocation/copy budget when copy is necessary;
- release reason mapping;
- privacy/redaction rule for diagnostics.

If driver cannot execute the admitted class exactly, it MUST fail closed and MUST NOT silently downgrade to another transform path.

### 5.5 Transcode / Payload Transform Rule

Initial v0.2 does not admit codec transcoding or payload transform as generic SFU behavior. If a future feature requires payload transform, codec rewrite, transcoding, media normalization, insertable-stream style transform, or E2EE media transform, a future specification MUST define: transform class; owner and dependency boundary; resource and copy bounds; quality/latency relation; privacy/redaction rule; secure media relation; failure reason mapping; evidence and audit rule. Until then, such requests are rejected as not admitted.

### 5.6 Failure Mapping (packet rewrite / media transform)

| Failure | Required reason |
|---|---|
| rewrite/transform class is not admitted | `packet_rewrite_class_not_admitted` |
| core rewrite intent is missing required field or conflicts with route | `packet_rewrite_intent_invalid` |
| driver or backend tries to own routing/quality semantics through rewrite | `packet_rewrite_owner_violation` |
| payload transform requested without admitted specification | `payload_transform_not_admitted` |
| codec transcode requested in initial v0.2 | `media_transcode_not_supported` |
| driver transform execution failed | `payload_transform_failed` |
| copy/allocation bound for rewrite path is exceeded | `rewrite_copy_bound_exceeded` |
| concrete packet bytes are requested by core | `packet_rewrite_owner_violation` |

Packet buffer release is governed by the SFU packet buffer lifecycle of §6/§7.

### 5.7 Audit / Evidence Rule

- Packet rewrite / media transform decisions use audit event type `packet_rewrite_transform_decision`. The event MUST carry `CorrelationId`, `PacketId` when materialized, route/target reference when materialized, rewrite/transform class, copy allowance class, execution owner, and cataloged reason for rejected/failed outcomes.
- Evidence MUST record rewrite/transform class, source forwarding decision, target route/endpoint, copy allowance class, driver execution class, resource/copy bound, payload transform admission status, command/procedure, working directory, and rerun condition. Packet forwarding evidence does not prove transform readiness unless this class evidence is recorded. Backend diagnostic logs without class and closed reason are not adopted evidence.

### 5.8 Prohibitions (packet rewrite / media transform)

- core mutates packet bytes or payload bytes.
- core owns driver buffer lease, codec engine, OS buffer, or backend frame object.
- driver rewrite path changes routing or quality semantics.
- payload transform is silently enabled by driver capability.
- codec transcode is implied by route or media negotiation success.
- packet copy is hidden from evidence when the claim depends on zero-copy or bounded-copy behavior.
- transform failure is recorded only as generic network send failure.

### 5.9 collapse conditions (packet rewrite / media transform)

- rewrite/transform class is open-ended.
- core intent includes concrete packet bytes or driver buffer handle.
- payload transform can execute without specification admission.
- copy/allocation bound is absent for a rewrite path that can copy.
- driver transform backend becomes SFU routing authority.

---

## 6. Packet Semantic View (field boundary, core-readable scope)

This section fixes the field boundary of the packet semantic view that SFU passes to core. §7 (packet buffer lifecycle) defines packet bytes ownership / lifetime; this section fixes the semantic fields core may read and the forbidden fields.

### 6.1 Boundary (owner)

| Surface | Owner | Rule |
|---|---|---|
| raw RTP / RTCP bytes | driver | never core-owned |
| parser concrete object | driver | no core exposure |
| semantic packet view type | core | borrowed, minimal field set |
| routing decision | core | uses semantic fields only |
| header rewrite execution | driver | per §5 |
| packet release | driver | closed release reason |

### 6.2 Required Semantic Fields (closed set)

Core packet semantic view may include only the following initial field classes. A new field requires a specification update.

| Field | Required | Rule |
|---|---|---|
| `packet_id` | yes | core-owned opaque packet reference after materialization |
| `correlation_id` | conditional | required when packet is attached to a correlated flow |
| `source_endpoint_id` | yes | core-owned endpoint reference |
| `stream_id` | yes | core-owned stream reference |
| `media_kind` | conditional | closed value if needed for routing/quality |
| `packet_kind` | yes | RTP or RTCP semantic class |
| `sequence_number` | conditional | routing/retransmission semantic only, no raw parser type |
| `timestamp` | conditional | RTP timestamp semantic only |
| `ssrc_ref` | conditional | opaque/core-owned SSRC reference, not raw parser object |
| `payload_type_ref` | conditional | payload type reference, not codec negotiation authority |
| `marker` | conditional | routing/quality hint only |
| `packet_length` | yes | length for resource/quality decision |
| `arrival_time` | conditional | core time observation, not driver clock object |

### 6.3 Forbidden Fields

Core packet semantic view MUST NOT include: raw packet bytes as owned buffer; mutable payload slice; driver `BufferLease`; driver packet cache reference; transmit queue reference; parser crate object; socket address object; str0m packet/event object; browser/native platform buffer type; codec implementation object; encryption key or SRTP backend detail; regulated payload or application user identity.

Borrowed raw slice may exist only under the §7 lifetime rule and MUST NOT be stored by core.

### 6.4 Field Use Rule

Semantic fields may be used only for: route selection; target suppression/drop decision; quality/backpressure decision; packet cache intent; header rewrite intent; release/audit reference.

Semantic fields MUST NOT become: codec negotiation authority; regulated domain identity; SDK public API field; driver cache key authority outside driver; durable persistence schema.

### 6.5 RTCP Rule

RTCP semantic class may expose only fields required for routing/quality/feedback/retransmission intent. Concrete RTCP packet parser object, compound packet layout, and raw feedback payload are driver-owned unless the specification admits a specific semantic field (MUST).

### 6.6 Failure Mapping (packet semantic view)

| Failure | Required reason |
|---|---|
| parser cannot produce required semantic view | `external_decode_failed` |
| frame size bound exceeded | `frame_size_bound_exceeded` |
| unsupported media contract version | `unsupported_media_contract_version` |
| buffer release failed | `buffer_release_failed` |

Failure before packet identity materialization follows the audit absent reference rule.

### 6.7 Prohibitions / collapse conditions (packet semantic view)

Prohibitions: core stores borrowed packet view beyond routing call; core mutates payload bytes; packet semantic view carries raw regulated payload; payload type field becomes codec implementation authority; media layer/codec selection inferred from payload field without negotiation mapping; packet view field added without a specification update; driver parser object crosses core boundary.

Collapse: semantic view becomes a raw packet wrapper; field set expands by implementation convenience; core owns parser/cache/queue detail; packet view becomes SDK or regulated public data model; packet field persisted as domain source-of-truth without state persistence policy; packet semantic view becomes media negotiation source-of-truth; packet semantic view carries transform backend or rewrite execution detail.

---

## 7. SFU Packet Buffer Lifecycle (RTP/RTCP bytes ownership / lifetime / copy admission)

This section fixes ownership, borrowing, lifetime, and copy admission of RTP / RTCP packet bytes in the SFU. This section is not an implementation procedure and does not specify a particular crate, pool size, lock-free data structure, or SIMD backend.

### 7.1 Boundary summary

```text
driver owns raw bytes / buffer lease / queue / cache
core owns packet abstract view / routing semantics / forwarding decision
driver executes forwarding and releases buffers
```

### 7.2 Ownership Rule

| Target | Owner | Reason |
|---|---|---|
| raw RTP / RTCP bytes | driver | concrete data from socket / str0m / parser / buffer pool |
| receive buffer | driver | memory allocation and reuse strategy is infrastructure detail |
| buffer lease | driver | bytes lifetime, ref-count, loan-count is physical resource management |
| retransmission cache | driver | NACK / retransmission is packet retention strategy |
| transmit queue | driver | async scheduling and backpressure execution detail |
| packet abstract view type | core | core-owned input needed for routing semantics |
| RTP header semantic view | core | abstract representation needed for routing / quality / stream identity |
| routing decision | core | SFU domain semantics |
| forwarding execution | driver | concrete I/O operation |

The driver ownership in this table means ownership of physical bytes, lease, retention, and allocation/reuse strategy.

### 7.3 Core Packet View

The packet core receives is not owned bytes but a borrowed abstract view. The conceptual structure is as follows.

```text
SfuPacketView<'packet>
  packet_id
  stream identity
  source endpoint
  RTP / RTCP semantic header view
  borrowed raw packet slice
  borrowed payload slice
```

`SfuPacketView` is a core-owned type, but the raw packet slice and payload slice inside it borrow driver-owned buffer (MUST).

### 7.4 Lifetime Rule

core reads the borrowed packet view only during the routing call. core MUST NOT store packet bytes, payload slice, raw slice, or driver buffer handle.

Allowed: read header/payload metadata for routing decision; read packet length/timestamp for quality/backpressure decision; include packet identity and stream identity in the decision.

Prohibited: core owns `Vec<u8>`; core owns `Bytes` / `Arc<[u8]>` / driver buffer handle; core stores `&[u8]` in queue/cache/async task; core mutates raw bytes; core manipulates driver-local reference count.

### 7.5 Routing Decision Boundary

core does not return bytes. core returns only the semantic forwarding decision. Routing decision MUST express: packet identity; forwarding targets; suppress/drop decision; rejection/suppression/drop reason; quality/backpressure related action; header rewrite intent if required; payload transform intent if required.

driver uses the routing decision and its own `packet_id -> BufferLease` correspondence to send the original bytes.

### 7.6 Async / Queue Rule

When crossing an async boundary, queue, worker handoff, pacing, or retransmission, responsibility for retaining bytes always remains with the driver (MUST).

Allowed: driver pushes `BufferLease` into a queue; driver retains in `PacketCache` as bounded retention; core returns `packet_id` and target set.

Prohibited: core carries borrowed slice past an async boundary; core owns packet cache; core owns transmit queue; core owns pacing queue.

### 7.7 Fan-Out Rule

In SFU fan-out, packet copy MUST NOT be done per target by default. driver treats the raw bytes of the same packet as a shared lease and decrements loan-count or reference count on each target send completion.

```text
one received packet
  -> one driver-owned BufferLease
  -> many target send attempts
  -> release after all loans are resolved
```

core decides the fan-out target set; driver manages fan-out memory retention and send execution.

### 7.8 Packet Cache Rule

Caches for NACK, retransmission, pacing, and short-term loss recovery are placed inside the driver. Packet cache MUST be bounded.

Required bounds: maximum packets; maximum bytes; maximum retention duration; eviction reason.

Prohibited: unbounded packet cache; core-owned packet cache; eviction reason being an open-ended string only; retransmission cache owning routing semantics.

### 7.9 Header Rewrite / Transform copy admission (summary)

This subsection is a summary of ownership/copy; rewrite/transform class, admission, and audit/evidence follow §5. When header rewrite, sequence number mapping, SSRC rewrite, or payload transform is required, core returns intent and the actual bytes operation is done by the driver.

| Condition | Copy policy |
|---|---|
| no rewrite | copy prohibited; use shared lease |
| header small rewrite | prefer header-only allocation or scatter/gather |
| target-specific header rewrite | target-specific header buffer allowed; avoid payload copy |
| payload transform required | copy-on-write or new buffer allocation allowed |
| encryption / framing backend requires copy | driver-local copy allowed; do not expose copy to core |

Even when a copy occurs, responsibility and ownership of the copy remain with the driver (MUST).

### 7.10 Backpressure Rule

backpressure decision is core semantics. Raw measurements of queue length, buffer pressure, and cache pressure are collected by driver and converted into core-owned metric type before reaching core. driver performs backpressure execution but does not own the authority of backpressure policy (prohibited).

### 7.11 Failure / Release Rule (release reason closed set)

driver MUST classify the termination reason of packet lifecycle with a closed set of release reason codes. unknown reason is not used. Release reason code MUST NOT be free text.

| Release reason code | Meaning | Catalog reason for non-success |
|---|---|---|
| `forwarded` | packet was forwarded to selected target | not required for success |
| `suppressed_by_routing_decision` | core route decision suppressed forwarding | `route_conflict` |
| `suppressed_by_quality_decision` | core quality decision suppressed packet forwarding | `packet_suppressed_by_quality` |
| `suppressed_by_backpressure` | core backpressure decision suppressed packet forwarding | `packet_suppressed_by_backpressure` |
| `dropped_by_backpressure` | core backpressure decision dropped packet or stopped retaining | `packet_dropped_by_backpressure` |
| `dropped_by_transmit_queue_bound` | SFU transmit queue bound prevented enqueue/send scheduling | `sfu_transmit_queue_bound_exceeded` |
| `target_unavailable` | target endpoint cannot receive packet | `target_unavailable` |
| `send_failed` | driver send operation failed | `network_send_failed` |
| `expired_from_packet_cache` | packet retention duration ended | `retention_duration_exceeded` |
| `evicted_by_cache_bound` | packet cache bound forced eviction | `packet_cache_bound_exceeded` |
| `transform_failed` | driver transform/encode step failed | `payload_transform_failed` |
| `release_failed` | driver failed to release buffer lease | `buffer_release_failed` |
| `driver_shutdown` | driver shutdown ended packet lifecycle | `driver_shutdown` |

Release reason audit mapping:

| Release reason code | Audit event type | Decision outcome | Catalog reason |
|---|---|---|---|
| `forwarded` | `sfu_forwarding_decision` | `forwarded` | not required for success |
| `suppressed_by_routing_decision` | `sfu_forwarding_decision` | `suppressed` | `route_conflict` |
| `suppressed_by_quality_decision` | `quality_violation_decision` | `suppressed` | `packet_suppressed_by_quality` |
| `suppressed_by_backpressure` | `backpressure_decision` | `suppressed` | `packet_suppressed_by_backpressure` |
| `dropped_by_backpressure` | `backpressure_decision` | `dropped` | `packet_dropped_by_backpressure` |
| `dropped_by_transmit_queue_bound` | `resource_bound_decision` | `dropped` | `sfu_transmit_queue_bound_exceeded` |
| `target_unavailable` | `sfu_forwarding_decision` | `failed` | `target_unavailable` |
| `send_failed` | `driver_error_converted` | `converted_failure` | `network_send_failed` |
| `expired_from_packet_cache` | `resource_bound_decision` | `expired` | `retention_duration_exceeded` |
| `evicted_by_cache_bound` | `resource_bound_decision` | `dropped` | `packet_cache_bound_exceeded` |
| `transform_failed` | `packet_rewrite_transform_decision` | `failed` | `payload_transform_failed` |
| `release_failed` | `driver_error_converted` | `converted_failure` | `buffer_release_failed` |
| `driver_shutdown` | `driver_error_converted` | `converted_failure` | `driver_shutdown` |

The packet release audit event MUST carry `CorrelationId` and `PacketId` when packet identity has been materialized. If not materialized, `PacketId` MUST be `absent_not_applicable`.

### 7.12 Prohibited Core Dependencies

The core packet view MUST NOT depend on: `tokio::net::*`; `str0m::*`; socket concrete type; OS buffer type; browser/native platform buffer type; concrete RTP parser crate type; driver `BufferLease`; driver `PacketCache`; driver `TxQueue`.

### 7.13 collapse conditions (packet buffer lifecycle)

- core owns packet bytes.
- core stores borrowed packet slice.
- core owns packet cache / transmit queue / pacing queue.
- driver owns routing / quality / backpressure semantics.
- per-target constant copy becomes the standard path.
- unbounded cache / queue / buffer retention is allowed.
- concrete bytes processing of header rewrite becomes core responsibility.
- rewrite/transform class or copy allowance bypasses §5.
- release reason becomes an open-ended string only.

---

## 8. Secure Media Session Lifecycle (DTLS/SRTP)

This section fixes the boundary of the DTLS/SRTP secure media session lifecycle. It fixes owner, failure mapping, and evidence relation so that transport security configuration, DTLS handshake, peer verification, SRTP protection state, media packet forwarding, and secret rotation/rekey are not conflated. This section does not claim DTLS/SRTP runtime implementation, cryptographic certification, or secure media readiness.

### 8.1 Boundary (owner)

| Concern | Owner | Rule |
|---|---|---|
| abstract secure media requirement | core/transport policy | required protection state and profile |
| media routing decision | core/sfu | secure state reference may be policy input only |
| DTLS/SRTP implementation | driver/WebRTC backend | concrete handshake, keying, packet protection |
| certificate/key source | entrypoints/drivers | typed reference and secret handling |
| rekey/rotation state | driver observation + core policy | follows secret rotation rule |
| secure media evidence | reports | handshake, peer verification, protection state are separate classes |

Listener startup, SDP/ICE relay acceptance, and DTLS handshake attempt do not prove SRTP-protected forwarding (MUST).

### 8.2 Secure Media Session Classes (closed set)

The secure media session classes of v0.2 initial architecture are limited to the following. A new class requires a specification update.

| Class | Meaning | Rule |
|---|---|---|
| `secure_media_required` | policy requires protected media path | absence fails closed |
| `dtls_handshake_observed` | driver observed handshake attempt/result | not SRTP forwarding proof |
| `peer_verification_observed` | driver observed peer verification result | not domain authorization |
| `srtp_protection_active` | driver observed packet protection active | evidence class limited to observed path |
| `secure_media_rekey_required` | key generation/rotation requires rekey | follows secret rotation lifecycle |
| `secure_media_session_closed` | secure media session ended | does not imply graceful domain drain |

### 8.3 Lifecycle Rule

Secure media session lifecycle MUST declare: required media/security contract version; accepted DTLS/SRTP profile or abstract profile class; peer verification requirement; relation to certificate/key source; relation to secret rotation/rekey; required protection state before packet forwarding claim; failure reason mapping; audit event relation.

core MUST NOT retain DTLS/SRTP session objects, raw keying material, cipher implementation objects, or protected packet bytes.

### 8.4 Failure Mapping (secure media)

| Failure | Required reason |
|---|---|
| secure media profile/version unsupported | `secure_media_profile_not_supported` |
| DTLS handshake failed | `secure_media_handshake_failed` |
| peer verification failed | `secure_media_peer_verification_failed` |
| SRTP protection not active where required | `secure_media_protection_not_active` |
| secure media key state invalid | `secure_media_key_state_invalid` |
| secure media session expired | `secure_media_session_expired` |
| rekey required before continuing | `secure_media_rekey_required` |
| required secret source unavailable | `secret_unavailable` |
| rotation state unavailable | `secret_rotation_state_unavailable` |
| revoked secret/key generation | `secret_key_revoked` |

### 8.5 Audit / Evidence Rule

- Secure media session decisions use audit event type `secure_media_session_decision`. The event MUST carry `CorrelationId`, secure media session class, media/security contract version, endpoint/session reference when materialized, and redacted key/certificate reference when applicable.
- Secure media evidence MUST record: secure media session class; media/security contract version; peer verification class; protection state class; redacted certificate/key reference when allowed; rotation/rekey relation when relevant; packet forwarding claim scope when claimed; expected outcome; actual outcome; cataloged reason for non-success; close-not-claimed scope.
- Handshake evidence does not prove SRTP-protected packet forwarding unless protection state evidence is present for the same scope. Secure media protection evidence does not prove Signaling participant authorization, SFU endpoint admission, or TURN permission without admitted cross-plane binding and target-plane decision.

### 8.6 Prohibitions (secure media)

- listener startup is treated as secure media readiness.
- DTLS handshake attempt is treated as peer verification success.
- peer verification success is treated as communication authorization success.
- secure media observation is treated as Signaling/SFU/TURN binding without cross-plane binding.
- SRTP backend object crosses into core.
- raw keying material appears in audit/log/report.
- stale or revoked media key is accepted due to driver cache.
- unprotected forwarding is allowed after secure media is required.

### 8.7 collapse conditions (secure media)

- secure media requirement is implicit.
- DTLS/SRTP concrete type becomes core-owned.
- handshake, peer verification, and SRTP protection evidence are conflated.
- secret rotation state is omitted while claiming secure media readiness.
- secure media failure is recorded only as generic network failure.
- secure media evidence is used as cross-plane binding evidence without binding class and target decision.
