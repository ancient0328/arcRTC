//! reference composition state境界です。

use std::collections::BTreeMap;

use arcrtc_core_identity::{AllocationId, CorrelationId, RoomId, RouteId, SessionId};
use arcrtc_distro_evidence::DistroEvidenceReason;
use arcrtc_reference_output::ReferenceCompositionOutcome;
use arcrtc_reference_sfu::{state::SfuRouteState, ReferenceSfuState};
use arcrtc_reference_signaling::ReferenceSignalingState;
use arcrtc_reference_turn::{state::AllocationState, ReferenceTurnState};

use crate::error::ReferenceCompositionError;

/// reference composition stateです。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReferenceCompositionState {
    /// Signaling reference stateです。
    pub signaling: ReferenceSignalingState,
    /// TURN reference stateです。
    pub turn: ReferenceTurnState,
    /// SFU reference stateです。
    pub sfu: ReferenceSfuState,
    /// 複合key文字列をkeyにしたcross-plane bindingです。
    pub bindings: BTreeMap<String, ReferenceCrossPlaneBinding>,
}

/// reference cross-plane bindingです。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceCrossPlaneBinding {
    /// binding correlation idです。
    pub correlation_id: CorrelationId,
    /// room referenceです。
    pub room_id: Option<RoomId>,
    /// session referenceです。
    pub session_id: Option<SessionId>,
    /// allocation referenceです。
    pub allocation_id: Option<AllocationId>,
    /// route referenceです。
    pub route_id: Option<RouteId>,
}

/// Signaling state と TURN state のcross-plane referenceを記録します。
pub fn bind_signaling_to_turn(
    state: &mut ReferenceCompositionState,
    correlation_id: CorrelationId,
    room_id: RoomId,
    allocation_id: AllocationId,
) -> Result<ReferenceCompositionOutcome, ReferenceCompositionError> {
    ensure_room_has_participant(&state.signaling, &room_id)?;
    let allocation = state
        .turn
        .allocations
        .get(allocation_id.as_str())
        .ok_or(ReferenceCompositionError::StateBoundaryViolation)?;
    if allocation.phase != AllocationState::Active {
        return Err(ReferenceCompositionError::StateBoundaryViolation);
    }
    let binding_key = format!(
        "{}:{}:{}",
        correlation_id.as_str(),
        room_id.as_str(),
        allocation_id.as_str()
    );
    insert_binding(
        state,
        binding_key,
        ReferenceCrossPlaneBinding {
            correlation_id: correlation_id.clone(),
            room_id: Some(room_id),
            session_id: None,
            allocation_id: Some(allocation_id),
            route_id: None,
        },
    );
    Ok(ReferenceCompositionOutcome::new(
        correlation_id,
        DistroEvidenceReason::DistroOk,
    ))
}

/// Signaling state と SFU state のcross-plane referenceを記録します。
pub fn bind_signaling_to_sfu(
    state: &mut ReferenceCompositionState,
    correlation_id: CorrelationId,
    room_id: RoomId,
    session_id: SessionId,
    route_id: RouteId,
) -> Result<ReferenceCompositionOutcome, ReferenceCompositionError> {
    ensure_room_has_participant(&state.signaling, &room_id)?;
    let route = state
        .sfu
        .routes
        .get(route_id.as_str())
        .ok_or(ReferenceCompositionError::StateBoundaryViolation)?;
    if route.session_id.as_str() != session_id.as_str()
        || matches!(route.phase, SfuRouteState::Closed | SfuRouteState::Dropped)
    {
        return Err(ReferenceCompositionError::StateBoundaryViolation);
    }
    let binding_key = format!(
        "{}:{}:{}:{}",
        correlation_id.as_str(),
        room_id.as_str(),
        session_id.as_str(),
        route_id.as_str()
    );
    insert_binding(
        state,
        binding_key,
        ReferenceCrossPlaneBinding {
            correlation_id: correlation_id.clone(),
            room_id: Some(room_id),
            session_id: Some(session_id),
            allocation_id: None,
            route_id: Some(route_id),
        },
    );
    Ok(ReferenceCompositionOutcome::new(
        correlation_id,
        DistroEvidenceReason::DistroOk,
    ))
}

/// reference composition stateを検証します。
pub fn validate_reference_composition_state(
    state: &ReferenceCompositionState,
) -> Result<(), ReferenceCompositionError> {
    for binding in state.bindings.values() {
        if let Some(room_id) = &binding.room_id {
            ensure_room_has_participant(&state.signaling, room_id)?;
        }
        if let Some(allocation_id) = &binding.allocation_id {
            if !state.turn.allocations.contains_key(allocation_id.as_str()) {
                return Err(ReferenceCompositionError::StateBoundaryViolation);
            }
        }
        if let Some(session_id) = &binding.session_id {
            if !state.sfu.sessions.contains_key(session_id.as_str()) {
                return Err(ReferenceCompositionError::StateBoundaryViolation);
            }
        }
        if let Some(route_id) = &binding.route_id {
            if !state.sfu.routes.contains_key(route_id.as_str()) {
                return Err(ReferenceCompositionError::StateBoundaryViolation);
            }
        }
    }
    Ok(())
}

fn ensure_room_has_participant(
    signaling: &ReferenceSignalingState,
    room_id: &RoomId,
) -> Result<(), ReferenceCompositionError> {
    let room = signaling
        .rooms
        .get(room_id.as_str())
        .ok_or(ReferenceCompositionError::StateBoundaryViolation)?;
    if room.participant_ids.is_empty() {
        return Err(ReferenceCompositionError::StateBoundaryViolation);
    }
    Ok(())
}

fn insert_binding(
    state: &mut ReferenceCompositionState,
    binding_key: String,
    binding: ReferenceCrossPlaneBinding,
) {
    state.bindings.insert(binding_key, binding);
}
