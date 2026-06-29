# Kernel contract consumption

Status: SSOT consolidated edition
Date: 2026-06-28 JST

## Purpose

This chapter fixes completely how the implementations area of arcRTC v0.2 (the non-Kernel implementation area) imports the frozen Kernel public contract, pins its version, maps it into reference / product builders, and pins the consumed Kernel source as a frozen read-only input. With the invariant that the implementations side MUST NOT own, overwrite, or duplicate Kernel semantic authority, this chapter internalizes the importable surface set, the import path rule, the full builder constructor-chain mappings, and the required inputs and fail-closed conditions of the Kernel source pin. This chapter is self-contained and is written at a granularity sufficient for re-implementation without opening any other document.

## Dependency-direction invariant

implementations depends on the Kernel public contract / SDK projection / documented command surface (`implementations -> Kernel`). The reverse dependency (`Kernel -> implementations`) is forbidden. implementations MUST NOT own or overwrite Kernel semantic authority. When a change to the Kernel contract is required, the implementations side MUST fail closed, and a formal versioned Kernel-side contract revision MUST come first. implementations MUST NOT bypass, duplicate, or extend the Kernel contract on its own side.

## 1. Importable surface

implementations MAY use the following Kernel surfaces. This is a closed set; using any surface not listed is forbidden.

| Surface | Allowed use |
|---|---|
| Kernel public contract | input contract for SFU / TURN / Signaling implementation |
| Kernel SDK projection | Signaling-side public API projection |
| core-owned port definition | contract for the driver / implementation boundary |
| documented command surface | origin of benchmark / test / evidence commands |
| Kernel report references | basis for the non-claim boundary |

The consumption boundary for each Kernel surface is fixed by the following table.

| Kernel surface | implementations-side use |
|---|---|
| public contract | input contract for Signaling / TURN / SFU implementation |
| SDK projection | Signaling public projection |
| core-owned port definition | connection contract for implementation mapper / runtime wrapper |
| documented command surface | origin of benchmark / test / evidence commands |
| Kernel report reference | basis for non-claim / boundary |

## 2. Non-importable surface

implementations MUST NOT import, overwrite, duplicate, or own the following. This is a closed set.

- Kernel core semantics
- Kernel reason catalog ownership
- Kernel port definition ownership
- Kernel dependency direction
- Kernel final Closed Gate authority
- Kernel completion / freeze claim authority
- Kernel evidence authority

## 3. Import path rule

A Kernel crate MAY import only the packages enumerated in the Kernel local path dependency matrix of dependency admission (the Kernel local path dependency matrix in Chapter 04, "Toolchain / runtime / dependency", of this specification is the authoritative matrix). When an additional import becomes necessary, the dependency matrix MUST be updated before any source change, and the additional import MUST NOT encroach on Kernel semantic authority.

The Kernel reference method is fixed to local path dependency / local source dependency. The following rules are fixed.

| Option | Rule | Reason |
|---|---|---|
| local path dependency | adopted | directly references the frozen Kernel contract within the same repository |
| published crate dependency | not adopted | publication / version distribution readiness is not yet claimed |
| binary / artifact dependency | not adopted | weakens source-level contract consumption and evidence connection |
| source copy | forbidden | breaks the Kernel freeze boundary |

## 4. Version pin rule

implementations references the Kernel under the following conditions (MUST).

- The reference target MUST be limited to `Kernel/`.
- The reference MUST be a dependency reference, not a source copy.
- The dependency reference MUST record a commit / path / version-equivalent identifier in the evidence report.
- When a Kernel-side change is required, implementations MUST fail closed.

### Upgrade rule

When updating a Kernel dependency, the implementations side MUST do the following.

1. Confirm whether a formal versioned Kernel-side contract revision exists.
2. Enumerate the affected implementations surfaces.
3. Separate the impact on reference / product / tests / benchmark / readiness.
4. Connect dedicated evidence and a verdict for the change.

## 5. Consumption design procedure

An implementation that uses the Kernel contract MUST be designed in the following order.

1. Identify the Kernel surface to be used by its source path.
2. Define the required wrapper / mapper / runtime configuration on the implementations side.
3. Limit the wrapper / mapper to merely transforming or invoking the Kernel contract.
4. The wrapper / mapper MUST NOT newly define Kernel semantics.
5. When a Kernel contract gap is found, stop the implementations task.
6. A contract for which no formal versioned Kernel-side contract revision exists MUST NOT be alternatively defined on the implementations side.

### Allowed wrapper

The wrappers that MAY be placed on the implementations side are limited to the following closed set.

| Wrapper class | Allowed responsibility |
|---|---|
| runtime wrapper | connection of process / async runtime / socket / environment |
| configuration mapper | convert product / reference config into Kernel contract input |
| error mapper | project Kernel reason into an implementation-facing report reason |
| command wrapper | launch assistance for benchmark / test / execution commands |
| observation mapper | convert metrics / log / trace output into the implementation evidence shape |

### Prohibited wrapper

The wrappers that MUST NOT be placed on the implementations side are the following.

- a wrapper that newly defines a Kernel reason
- a wrapper that extends a Kernel port definition
- a wrapper that overwrites a Kernel state transition
- a wrapper that re-implements a Kernel routing / allocation / signaling decision
- a wrapper that converts Kernel completion evidence into implementations readiness proof

## 6. Builder mapping (Kernel public constructor invocation rule)

This fixes the order and arguments with which a reference builder invokes Kernel public constructors. This mapping does not redefine Kernel semantics. The Kernel source MUST NOT be changed, and implementations consumes the Kernel public API.

### Shared builder rule

A reference builder satisfies the following (MUST / MUST NOT).

- MUST NOT read Kernel private fields.
- MUST NOT leak a Kernel constructor error as a raw error.
- MUST convert a Kernel constructor error into a local error variant.
- MUST use only the command type literal tables in this chapter.
- MUST fix command version to `CommandVersion::new(1)` in the initial reference implementation.

### 6.1 Signaling builder mapping

`build_kernel_signaling_command` is fixed to the following constructor chain.

| Step | Constructor | Argument mapping |
|---|---|---|
| 1 | `SignalingSubject::new` | `input.room_id`, `input.participant_id` |
| 2 | `CommandType::new` | table `Signaling Command Type Literal` |
| 3 | `CommandVersion::new` | `1` |
| 4 | `CommandEnvelope::new` | `input.correlation_id`, command type, command version, `TargetSurface::Signaling`, signaling subject |
| 5 | `SignalingCommand::new` | envelope, `input.kind`, `input.payload` |

#### Signaling Command Type Literal (closed set)

| `SignalingCommandKind` | `CommandType` literal |
|---|---|
| `JoinRoom` | `reference.signaling.join_room` |
| `LeaveRoom` | `reference.signaling.leave_room` |
| `SendOffer` | `reference.signaling.send_offer` |
| `SendAnswer` | `reference.signaling.send_answer` |
| `SendIceCandidate` | `reference.signaling.send_ice_candidate` |
| `RequestTurnCredential` | `reference.signaling.request_turn_credential` |
| `AcknowledgeForward` | `reference.signaling.acknowledge_forward` |

If `CommandEnvelope::new` or `SignalingCommand::new` shape changes in the Kernel, the builder MUST fail closed with `ReferenceSignalingError::KernelContractMismatch`.

### 6.2 Signaling event projection mapping

`project_reference_signaling_event` is fixed to:

| Step | Constructor | Argument mapping |
|---|---|---|
| 1 | `SignalingSubject::new` | `outcome.room_id`, `outcome.participant_id` |
| 2 | `SignalingEvent::new` | `outcome.correlation_id`, `outcome.kind`, subject, payload |

`project_reference_signaling_event` MUST NOT derive the event payload from the Kernel private command payload.

### 6.3 TURN builder mapping

`build_kernel_turn_command` is fixed to:

| Step | Constructor | Argument mapping |
|---|---|---|
| 1 | `TurnCommand::try_new` | `input.kind`, `input.transaction_id`, `input.references`, `input.peer_address`, `input.requested_lifetime`, `input.relay_packet_id` |

Kernel `TurnContractError` maps to `ReferenceTurnError::KernelContractMismatch`.

#### TURN required field gate

The builder MUST call `TurnCommand::try_new` directly and MUST NOT duplicate Kernel validation as authoritative semantics. The local pre-check MAY only classify fixture incompleteness before Kernel construction.

| Command kind | Required local fixture before Kernel call | Missing fixture reason |
|---|---|---|
| `Allocate` | transaction id and credential fixture reference | `InvalidFixtureCredential` |
| `Refresh` | active allocation id reference | `StateBoundaryViolation` |
| `CreatePermission` | active allocation id reference and peer address | `StateBoundaryViolation` |
| `ChannelBind` | permission id, channel bind id, peer address | `StateBoundaryViolation` |
| `RelayData` | allocation id, permission id, peer address, packet id | `StateBoundaryViolation` |

### 6.4 SFU builder mapping

`build_kernel_sfu_item` is fixed to:

| Step | Constructor | Argument mapping |
|---|---|---|
| 1 | `SfuContractItem::new` | `input.model_kind`, `input.references`, `input.payload` |

`build_borrowed_packet_view` is fixed to:

| Step | Constructor | Argument mapping |
|---|---|---|
| 1 | `SfuPacketView::new` | `packet_id`, `stream_id`, `source_endpoint_id`, `header`, `raw_packet`, `payload` |

The `raw_packet` and `payload` arguments MUST be borrowed slices received from caller scope. The builder MUST NOT allocate a new buffer, clone bytes, or store the byte slices in reference state.

### 6.5 Builder error mapping (closed set)

| Builder | Kernel / local failure | Local error |
|---|---|---|
| `build_kernel_signaling_command` | constructor shape mismatch | `ReferenceSignalingError::KernelContractMismatch` |
| `project_reference_signaling_event` | missing outcome field | `ReferenceSignalingError::EvidenceFieldsIncomplete` |
| `build_kernel_turn_command` | `TurnCommand::try_new` error | `ReferenceTurnError::KernelContractMismatch` |
| `build_kernel_sfu_item` | missing reference set member required by fixture | `ReferenceSfuError::KernelContractMismatch` |
| `build_borrowed_packet_view` | missing packet id / stream id / endpoint id fixture | `ReferenceSfuError::KernelContractMismatch` |

## 7. Kernel source pin (frozen contract)

implementations pins the consumed Kernel source snapshot as a read-only input. The implementations side MUST NOT change the Kernel source, and MUST NOT treat the Kernel source as a substitute evidence for production readiness / live readiness / benchmark threshold satisfaction / real-device success.

### 7.1 Pin fields (required inputs)

| Field | Required value |
|---|---|
| `kernel_root` | `Kernel/` |
| `kernel_source_file_count` | `216` |
| `kernel_source_snapshot_identifier` | `sha256:0f4963a6b3e57624dbfcca0bc4aa0372a72ccc2c7bd7e936d7ffb2d50097c71e` |
| `implementations_kernel_mutation_absence` | `Kernel source is read-only input for the binding; the binding writes only under implementations/` |

### 7.2 Snapshot command

The consumed Kernel source snapshot identifier is computed from the implementations root with:

```sh
find ../Kernel -path '*/target' -prune -o -type f \( -name '*.rs' -o -name 'Cargo.toml' -o -name 'Cargo.lock' \) -print | sort | while IFS= read -r f; do shasum -a 256 "$f"; done | shasum -a 256
```

The first field of the command output MUST equal:

```text
0f4963a6b3e57624dbfcca0bc4aa0372a72ccc2c7bd7e936d7ffb2d50097c71e
```

The consumed file count is computed from the implementations root with:

```sh
find ../Kernel -path '*/target' -prune -o -type f \( -name '*.rs' -o -name 'Cargo.toml' -o -name 'Cargo.lock' \) -print | sort | wc -l
```

The command output MUST be:

```text
216
```

### 7.3 Binding assertion

`tests/boundary/tests/kernel_completion_freeze_binding.rs` owns the following assertion.

```text
kpi_kernel_completion_freeze_binding_accepts_consumed_snapshot_without_kernel_mutation
```

The assertion MUST verify:

- The current Kernel source snapshot digest equals `0f4963a6b3e57624dbfcca0bc4aa0372a72ccc2c7bd7e936d7ffb2d50097c71e`.
- This pin contains the same snapshot digest and file count.
- This pin treats the Kernel source as a read-only input and does not admit Kernel source mutation.

### 7.4 Binding report required fields

The binding report MUST contain:

| Required field | Value source |
|---|---|
| correlation id | report header |
| command | `cargo test --manifest-path tests/boundary/Cargo.toml kpi_kernel_completion_freeze_binding_accepts_consumed_snapshot_without_kernel_mutation` |
| working directory | `implementations` |
| target package | `arcrtc-implementation-boundary-tests` |
| target scope | Kernel source pin (frozen contract) |
| expected outcome | consumed Kernel snapshot is connected to Kernel final report and Kernel mutation is not used |
| actual outcome | command stdout / stderr summary and exit status |
| environment / toolchain | `rustc -Vv`, `cargo -V`, `shasum -a 256` |
| reason classification | `ImplementationOk` |
| non-claim scope | `none` |
| rerun condition | any Kernel source file, Kernel final report, this binding, or the binding report/index rule changes |

### 7.5 Rerun condition

The pin validation MUST be rerun if any of the following changes:

- `Kernel/**/*.rs`
- `Kernel/**/Cargo.toml`
- `Kernel/**/Cargo.lock`
- `implementations/tests/boundary/tests/kernel_completion_freeze_binding.rs`

### 7.6 Basis of the pin

The implementations side consumes the Kernel source as a read-only, version-pinned input identified by its snapshot digest and file count. The implementations side MUST NOT repurpose the consumed Kernel source into implementations product completion or readiness evidence; production readiness / live readiness / benchmark threshold satisfaction / real-device success are established by implementations-side dedicated evidence, not derived from the Kernel source pin. The correct responsibility of the implementations side is limited to consuming the Kernel source without modification and fixing the rerun conditions.

## 8. Fail-closed rule

When a Kernel contract gap is found during implementation, it MUST be handled in the following order.

1. Stop the implementations-side task.
2. Record the gap in the implementations report.
3. State explicitly that a formal versioned Kernel-side contract revision is required.
4. MUST NOT bypass, duplicate, or extend the Kernel contract on the implementations side.

### Fail-closed conditions of the Kernel source pin

If any of the following occurs, the Kernel source pin MUST NOT be accepted as valid (fail closed).

- The consumed Kernel source snapshot identifier does not match the recomputed value.
- The consumed Kernel source file count does not match the recomputed value.
- The Kernel source is changed during implementations-side fixed-goal work.
- The Kernel source pin is used as substitute evidence for production readiness / live readiness / benchmark threshold satisfaction / real-device success / implementations product completion.

## 9. Invariants (collapse conditions)

The boundary defined by this chapter collapses the moment any of the following occurs. These are forbidden, and on occurrence the system MUST fail closed.

- implementations changes Kernel semantic authority.
- A Kernel contract gap is filled with an alternative type on the implementations side.
- A Kernel crate not in the dependency matrix is imported.
- Kernel evidence is adopted as implementations completion / readiness proof.
- Kernel source modification is treated as an ordinary implementations task.
- implementations duplicates, extends, or bypasses the Kernel contract.
- A wrapper / mapper owns Kernel semantics.
- An implementations task continues when a Kernel contract gap exists.
- implementations copies the Kernel source.
- A published crate / artifact dependency is adopted without readiness proof.
- A Kernel dependency update is performed without evidence.
- Kernel source modification is treated as an ordinary task on the grounds of path dependency.
- A builder reads a Kernel private field.
- A builder uses a command type literal not in this chapter.
- A builder exposes a Kernel constructor error as a raw dependency error.
- `build_borrowed_packet_view` copies / allocates / persists packet bytes.
- Builder success is treated as reference behavior completion / benchmark success / readiness success.

## 10. Non-claim

This chapter does not by itself claim re-judgement of the Kernel final report, re-freeze of the Kernel source, production readiness, live readiness, benchmark threshold satisfaction, real-device success, or full fixed-goal completion. This chapter fixes only the boundary of implementations-side Kernel contract consumption / version pin / builder mapping / Kernel source pin.
