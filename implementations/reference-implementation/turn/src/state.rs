//! reference TURN のin-memory state境界です。

use std::collections::BTreeMap;

use arcrtc_core_identity::{AllocationId, ChannelBindId, CredentialRef, PermissionId};
use arcrtc_core_turn::{CorePeerAddress, TurnDecisionKind};
use arcrtc_implementation_evidence::ImplementationEvidenceReason;
use arcrtc_reference_output::ReferenceTurnOutcome;

use crate::{error::ReferenceTurnError, kernel_contract::ReferenceTurnCommandInput};

/// reference allocation phaseです。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AllocationState {
    /// active allocationです。
    Active,
    /// released allocationです。
    Released,
}

/// reference permission phaseです。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PermissionState {
    /// active permissionです。
    Active,
    /// revoked permissionです。
    Revoked,
}

/// reference channel bind phaseです。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChannelBindState {
    /// active channel bindです。
    Active,
    /// expired channel bindです。
    Expired,
}

/// reference allocation stateです。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceAllocationState {
    /// allocation referenceです。
    pub allocation_id: AllocationId,
    /// allocation phaseです。
    pub phase: AllocationState,
    /// credential referenceです。
    pub credential_ref: Option<CredentialRef>,
}

/// reference permission stateです。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferencePermissionState {
    /// permission referenceです。
    pub permission_id: PermissionId,
    /// allocation referenceです。
    pub allocation_id: AllocationId,
    /// peer addressです。
    pub peer_address: CorePeerAddress,
    /// permission phaseです。
    pub phase: PermissionState,
}

/// reference channel bind stateです。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceChannelBindState {
    /// channel bind referenceです。
    pub channel_bind_id: ChannelBindId,
    /// permission referenceです。
    pub permission_id: PermissionId,
    /// channel bind phaseです。
    pub phase: ChannelBindState,
}

/// reference TURN stateです。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReferenceTurnState {
    /// allocation_id文字列をkeyにしたallocation stateです。
    pub allocations: BTreeMap<String, ReferenceAllocationState>,
    /// permission_id文字列をkeyにしたpermission stateです。
    pub permissions: BTreeMap<String, ReferencePermissionState>,
    /// channel_bind_id文字列をkeyにしたchannel bind stateです。
    pub channel_binds: BTreeMap<String, ReferenceChannelBindState>,
}

/// reference TURN stateへcommandを適用します。
pub fn apply_reference_turn(
    state: &mut ReferenceTurnState,
    input: &ReferenceTurnCommandInput,
) -> Result<ReferenceTurnOutcome, ReferenceTurnError> {
    match input.kind {
        arcrtc_core_turn::TurnCommandKind::Allocate => apply_allocate(state, input),
        arcrtc_core_turn::TurnCommandKind::Refresh => apply_refresh(state, input),
        arcrtc_core_turn::TurnCommandKind::CreatePermission => {
            apply_create_permission(state, input)
        }
        arcrtc_core_turn::TurnCommandKind::ChannelBind => apply_channel_bind(state, input),
        arcrtc_core_turn::TurnCommandKind::RelayData => apply_relay_data(state, input),
    }
}

/// reference TURN stateを検証します。
pub fn validate_reference_turn_state(state: &ReferenceTurnState) -> Result<(), ReferenceTurnError> {
    for permission in state.permissions.values() {
        if !state
            .allocations
            .contains_key(permission.allocation_id.as_str())
        {
            return Err(ReferenceTurnError::StateBoundaryViolation);
        }
    }
    for channel_bind in state.channel_binds.values() {
        if !state
            .permissions
            .contains_key(channel_bind.permission_id.as_str())
        {
            return Err(ReferenceTurnError::StateBoundaryViolation);
        }
    }
    Ok(())
}

/// TURN decision kind helperです。
pub const fn turn_decision_kind(kind: TurnDecisionKind) -> TurnDecisionKind {
    kind
}

fn apply_allocate(
    state: &mut ReferenceTurnState,
    input: &ReferenceTurnCommandInput,
) -> Result<ReferenceTurnOutcome, ReferenceTurnError> {
    let allocation_id = input
        .allocation_id
        .clone()
        .ok_or(ReferenceTurnError::InvalidFixtureCredential)?;
    let credential_ref = input
        .credential_ref
        .clone()
        .ok_or(ReferenceTurnError::InvalidFixtureCredential)?;
    state.allocations.insert(
        allocation_id.as_str().to_owned(),
        ReferenceAllocationState {
            allocation_id,
            phase: AllocationState::Active,
            credential_ref: Some(credential_ref),
        },
    );
    Ok(outcome(TurnDecisionKind::Allocation))
}

fn apply_refresh(
    state: &mut ReferenceTurnState,
    input: &ReferenceTurnCommandInput,
) -> Result<ReferenceTurnOutcome, ReferenceTurnError> {
    let allocation_id = input
        .allocation_id
        .as_ref()
        .ok_or(ReferenceTurnError::StateBoundaryViolation)?;
    let allocation = state
        .allocations
        .get(allocation_id.as_str())
        .ok_or(ReferenceTurnError::StateBoundaryViolation)?;
    if allocation.phase != AllocationState::Active || input.requested_lifetime.is_none() {
        return Err(ReferenceTurnError::StateBoundaryViolation);
    }
    Ok(outcome(TurnDecisionKind::Refresh))
}

fn apply_create_permission(
    state: &mut ReferenceTurnState,
    input: &ReferenceTurnCommandInput,
) -> Result<ReferenceTurnOutcome, ReferenceTurnError> {
    let allocation_id = input
        .allocation_id
        .clone()
        .ok_or(ReferenceTurnError::StateBoundaryViolation)?;
    let permission_id = input
        .permission_id
        .clone()
        .ok_or(ReferenceTurnError::StateBoundaryViolation)?;
    let peer_address = input
        .peer_address
        .clone()
        .ok_or(ReferenceTurnError::StateBoundaryViolation)?;
    let allocation = state
        .allocations
        .get(allocation_id.as_str())
        .ok_or(ReferenceTurnError::StateBoundaryViolation)?;
    if allocation.phase != AllocationState::Active {
        return Err(ReferenceTurnError::StateBoundaryViolation);
    }
    state.permissions.insert(
        permission_id.as_str().to_owned(),
        ReferencePermissionState {
            permission_id,
            allocation_id,
            peer_address,
            phase: PermissionState::Active,
        },
    );
    Ok(outcome(TurnDecisionKind::Permission))
}

fn apply_channel_bind(
    state: &mut ReferenceTurnState,
    input: &ReferenceTurnCommandInput,
) -> Result<ReferenceTurnOutcome, ReferenceTurnError> {
    let permission_id = input
        .permission_id
        .clone()
        .ok_or(ReferenceTurnError::StateBoundaryViolation)?;
    let channel_bind_id = input
        .channel_bind_id
        .clone()
        .ok_or(ReferenceTurnError::StateBoundaryViolation)?;
    let permission = state
        .permissions
        .get(permission_id.as_str())
        .ok_or(ReferenceTurnError::StateBoundaryViolation)?;
    if permission.phase != PermissionState::Active {
        return Err(ReferenceTurnError::StateBoundaryViolation);
    }
    state.channel_binds.insert(
        channel_bind_id.as_str().to_owned(),
        ReferenceChannelBindState {
            channel_bind_id,
            permission_id,
            phase: ChannelBindState::Active,
        },
    );
    Ok(outcome(TurnDecisionKind::ChannelBind))
}

fn apply_relay_data(
    state: &mut ReferenceTurnState,
    input: &ReferenceTurnCommandInput,
) -> Result<ReferenceTurnOutcome, ReferenceTurnError> {
    let permission_id = input
        .permission_id
        .as_ref()
        .ok_or(ReferenceTurnError::StateBoundaryViolation)?;
    let permission = state
        .permissions
        .get(permission_id.as_str())
        .ok_or(ReferenceTurnError::StateBoundaryViolation)?;
    if permission.phase != PermissionState::Active || input.relay_packet_id.is_none() {
        return Err(ReferenceTurnError::StateBoundaryViolation);
    }
    Ok(outcome(TurnDecisionKind::Relay))
}

fn outcome(kind: TurnDecisionKind) -> ReferenceTurnOutcome {
    ReferenceTurnOutcome::new(kind, ImplementationEvidenceReason::ImplementationOk)
}
