//! reference TURN state mutation 境界を直接実行して検査します。

use arcrtc_core_identity::{
    AllocationId, ChannelBindId, CredentialRef, OpaqueReference, PacketId, PermissionId,
    ReferenceAuthority,
};
use arcrtc_core_turn::{
    CorePeerAddress, TurnCommandKind, TurnDecisionKind, TurnReferenceSet,
    TurnRequestedLifetimeSeconds, TurnTransactionId,
};
use arcrtc_reference_turn::{
    apply_reference_turn, build_kernel_turn_command, validate_reference_turn_state,
    ReferenceTurnCommandInput, ReferenceTurnError, ReferenceTurnState,
};

fn accepted(value: &str) -> OpaqueReference {
    OpaqueReference::accept(value, ReferenceAuthority::CoreValidatedUntrustedInput)
        .expect("test reference must be accepted")
}

fn allocation() -> AllocationId {
    AllocationId::new(accepted("turn-allocation"))
}

fn permission() -> PermissionId {
    PermissionId::new(accepted("turn-permission"))
}

fn channel_bind() -> ChannelBindId {
    ChannelBindId::new(accepted("turn-channel"))
}

fn credential() -> CredentialRef {
    CredentialRef::new(accepted("turn-credential"))
}

fn packet() -> PacketId {
    PacketId::new(accepted("turn-packet"))
}

fn transaction(value: &str) -> TurnTransactionId {
    TurnTransactionId::new(accepted(value))
}

fn references() -> TurnReferenceSet {
    TurnReferenceSet::new(
        Some(allocation()),
        Some(permission()),
        Some(channel_bind()),
        Some(credential()),
    )
}

fn input(kind: TurnCommandKind) -> ReferenceTurnCommandInput {
    ReferenceTurnCommandInput {
        kind,
        transaction_id: transaction("turn-transaction"),
        references: references(),
        allocation_id: Some(allocation()),
        permission_id: Some(permission()),
        channel_bind_id: Some(channel_bind()),
        credential_ref: Some(credential()),
        peer_address: Some(CorePeerAddress::new("192.0.2.1:3478").expect("peer address")),
        requested_lifetime: Some(TurnRequestedLifetimeSeconds::try_new(600).expect("lifetime")),
        relay_packet_id: Some(packet()),
    }
}

fn kernel_driven_input(kind: TurnCommandKind) -> ReferenceTurnCommandInput {
    let base = input(kind);
    let kernel_command =
        build_kernel_turn_command(&base).expect("Kernel TURN command builder must succeed");

    assert_eq!(kernel_command.kind(), kind);
    assert_eq!(kernel_command.transaction_id(), &base.transaction_id);

    ReferenceTurnCommandInput {
        kind: kernel_command.kind(),
        transaction_id: kernel_command.transaction_id().clone(),
        references: kernel_command.references().clone(),
        allocation_id: base.allocation_id,
        permission_id: base.permission_id,
        channel_bind_id: base.channel_bind_id,
        credential_ref: base.credential_ref,
        peer_address: base.peer_address,
        requested_lifetime: base.requested_lifetime,
        relay_packet_id: base.relay_packet_id,
    }
}

#[test]
fn kpi_reference_turn_executes_kernel_contract_driven_state_transition() {
    let mut state = ReferenceTurnState::default();

    // KPI-T3-2 は allocation / permission / channel bind を Kernel command output 起点で固定する。
    let allocated =
        apply_reference_turn(&mut state, &kernel_driven_input(TurnCommandKind::Allocate))
            .expect("Kernel-derived allocation must succeed");
    assert_eq!(allocated.kind, TurnDecisionKind::Allocation);
    assert!(state.allocations.contains_key("turn-allocation"));

    let permitted = apply_reference_turn(
        &mut state,
        &kernel_driven_input(TurnCommandKind::CreatePermission),
    )
    .expect("Kernel-derived permission must succeed");
    assert_eq!(permitted.kind, TurnDecisionKind::Permission);
    assert!(state.permissions.contains_key("turn-permission"));

    let bound = apply_reference_turn(
        &mut state,
        &kernel_driven_input(TurnCommandKind::ChannelBind),
    )
    .expect("Kernel-derived channel bind must succeed");
    assert_eq!(bound.kind, TurnDecisionKind::ChannelBind);
    assert!(state.channel_binds.contains_key("turn-channel"));

    assert_eq!(validate_reference_turn_state(&state), Ok(()));
}

#[test]
fn turn_state_applies_all_reference_mutations_in_order() {
    let mut state = ReferenceTurnState::default();

    let allocated =
        apply_reference_turn(&mut state, &input(TurnCommandKind::Allocate)).expect("allocate");
    assert_eq!(allocated.kind, TurnDecisionKind::Allocation);

    let refreshed =
        apply_reference_turn(&mut state, &input(TurnCommandKind::Refresh)).expect("refresh");
    assert_eq!(refreshed.kind, TurnDecisionKind::Refresh);

    let permitted = apply_reference_turn(&mut state, &input(TurnCommandKind::CreatePermission))
        .expect("permission");
    assert_eq!(permitted.kind, TurnDecisionKind::Permission);

    let bound =
        apply_reference_turn(&mut state, &input(TurnCommandKind::ChannelBind)).expect("bind");
    assert_eq!(bound.kind, TurnDecisionKind::ChannelBind);

    let relayed =
        apply_reference_turn(&mut state, &input(TurnCommandKind::RelayData)).expect("relay");
    assert_eq!(relayed.kind, TurnDecisionKind::Relay);
    assert_eq!(validate_reference_turn_state(&state), Ok(()));
}

#[test]
fn turn_state_rejects_relay_without_prior_permission() {
    let mut state = ReferenceTurnState::default();
    apply_reference_turn(&mut state, &input(TurnCommandKind::Allocate)).expect("allocate");

    assert_eq!(
        apply_reference_turn(&mut state, &input(TurnCommandKind::RelayData)),
        Err(ReferenceTurnError::StateBoundaryViolation)
    );
}
