//! reference Signaling のin-memory state境界です。

use std::collections::{BTreeMap, BTreeSet};

use arcrtc_core_identity::{ParticipantId, RoomId};
use arcrtc_core_signaling::{
    SignalingCommandKind, SignalingEvent, SignalingEventKind, SignalingSubject,
};
use arcrtc_distro_evidence::DistroEvidenceReason;
use arcrtc_reference_output::ReferenceSignalingOutcome;

use crate::{
    error::ReferenceSignalingError,
    fixture_identity::FixtureSessionDescriptionDirection,
    kernel_contract::{ReferenceSignalingCommandInput, ReferenceSignalingPayload},
};

/// reference room phaseです。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferenceRoomPhase {
    /// open roomです。
    Open,
    /// closed roomです。
    Closed,
}

/// reference participant phaseです。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferenceParticipantPhase {
    /// joined participantです。
    Joined,
    /// left participantです。
    Left,
}

/// reference room stateです。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceRoomState {
    /// room referenceです。
    pub room_id: RoomId,
    /// room phaseです。
    pub phase: ReferenceRoomPhase,
    /// participant id setです。
    pub participant_ids: BTreeSet<String>,
}

/// reference participant stateです。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceParticipantState {
    /// participant referenceです。
    pub participant_id: ParticipantId,
    /// room referenceです。
    pub room_id: RoomId,
    /// participant phaseです。
    pub phase: ReferenceParticipantPhase,
}

/// reference Signaling stateです。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReferenceSignalingState {
    /// room_id文字列をkeyにしたroom stateです。
    pub rooms: BTreeMap<String, ReferenceRoomState>,
    /// participant_id文字列をkeyにしたparticipant stateです。
    pub participants: BTreeMap<String, ReferenceParticipantState>,
}

/// reference Signaling stateへcommandを適用します。
pub fn apply_reference_signaling(
    state: &mut ReferenceSignalingState,
    input: &ReferenceSignalingCommandInput,
) -> Result<ReferenceSignalingOutcome, ReferenceSignalingError> {
    match input.kind {
        SignalingCommandKind::JoinRoom => apply_join_room(state, input),
        SignalingCommandKind::LeaveRoom => apply_leave_room(state, input),
        SignalingCommandKind::SendOffer => {
            match &input.payload {
                ReferenceSignalingPayload::SendOffer { session }
                    if session.direction == FixtureSessionDescriptionDirection::Offer => {}
                _ => return Err(ReferenceSignalingError::InvalidFixtureIdentity),
            }
            ensure_participant_joined(state, input)?;
            Ok(outcome(input, SignalingEventKind::OfferReceived))
        }
        SignalingCommandKind::SendAnswer => {
            match &input.payload {
                ReferenceSignalingPayload::SendAnswer { session }
                    if session.direction == FixtureSessionDescriptionDirection::Answer => {}
                _ => return Err(ReferenceSignalingError::InvalidFixtureIdentity),
            }
            ensure_participant_joined(state, input)?;
            Ok(outcome(input, SignalingEventKind::AnswerReceived))
        }
        SignalingCommandKind::SendIceCandidate => {
            if !matches!(
                &input.payload,
                ReferenceSignalingPayload::SendIceCandidate { .. }
            ) {
                return Err(ReferenceSignalingError::InvalidFixtureIdentity);
            }
            ensure_participant_joined(state, input)?;
            Ok(outcome(input, SignalingEventKind::IceCandidateReceived))
        }
        SignalingCommandKind::RequestTurnCredential => {
            ensure_participant_joined(state, input)?;
            Ok(outcome(input, SignalingEventKind::TurnCredentialAvailable))
        }
        SignalingCommandKind::AcknowledgeForward => {
            ensure_participant_joined(state, input)?;
            Ok(outcome(input, SignalingEventKind::Joined))
        }
    }
}

/// reference Signaling outcomeをKernel eventへprojectします。
pub fn project_reference_signaling_event(
    outcome: &ReferenceSignalingOutcome,
    payload: ReferenceSignalingPayload,
) -> SignalingEvent<ReferenceSignalingPayload> {
    let subject = SignalingSubject::new(outcome.room_id.clone(), outcome.participant_id.clone());
    SignalingEvent::new(
        outcome.correlation_id.clone(),
        outcome.kind,
        subject,
        payload,
    )
}

/// reference Signaling stateを検証します。
pub fn validate_reference_signaling_state(
    state: &ReferenceSignalingState,
) -> Result<(), ReferenceSignalingError> {
    for participant in state.participants.values() {
        let room = state
            .rooms
            .get(participant.room_id.as_str())
            .ok_or(ReferenceSignalingError::StateBoundaryViolation)?;
        if participant.phase == ReferenceParticipantPhase::Joined
            && !room
                .participant_ids
                .contains(participant.participant_id.as_str())
        {
            return Err(ReferenceSignalingError::StateBoundaryViolation);
        }
    }
    for room in state.rooms.values() {
        for participant_id in &room.participant_ids {
            let participant = state
                .participants
                .get(participant_id)
                .ok_or(ReferenceSignalingError::StateBoundaryViolation)?;
            if participant.room_id.as_str() != room.room_id.as_str() {
                return Err(ReferenceSignalingError::StateBoundaryViolation);
            }
        }
    }
    Ok(())
}

/// Signaling commandから成功時event kindを投影するためのhelperです。
pub const fn signaling_success_event_kind(kind: SignalingEventKind) -> SignalingEventKind {
    kind
}

fn apply_join_room(
    state: &mut ReferenceSignalingState,
    input: &ReferenceSignalingCommandInput,
) -> Result<ReferenceSignalingOutcome, ReferenceSignalingError> {
    let participant_id = input
        .participant_id
        .clone()
        .ok_or(ReferenceSignalingError::StateBoundaryViolation)?;
    let room_key = input.room_id.as_str().to_owned();
    let participant_key = participant_id.as_str().to_owned();
    let room = state
        .rooms
        .entry(room_key)
        .or_insert_with(|| ReferenceRoomState {
            room_id: input.room_id.clone(),
            phase: ReferenceRoomPhase::Open,
            participant_ids: BTreeSet::new(),
        });
    if room.phase == ReferenceRoomPhase::Closed {
        return Err(ReferenceSignalingError::StateBoundaryViolation);
    }
    if let Some(existing) = state.participants.get(&participant_key) {
        if existing.room_id.as_str() != input.room_id.as_str()
            || existing.phase == ReferenceParticipantPhase::Joined
                && !room.participant_ids.contains(&participant_key)
        {
            return Err(ReferenceSignalingError::StateBoundaryViolation);
        }
    }
    room.participant_ids.insert(participant_key.clone());
    state.participants.insert(
        participant_key,
        ReferenceParticipantState {
            participant_id,
            room_id: input.room_id.clone(),
            phase: ReferenceParticipantPhase::Joined,
        },
    );
    Ok(outcome(input, SignalingEventKind::Joined))
}

fn apply_leave_room(
    state: &mut ReferenceSignalingState,
    input: &ReferenceSignalingCommandInput,
) -> Result<ReferenceSignalingOutcome, ReferenceSignalingError> {
    let participant_id = input
        .participant_id
        .clone()
        .ok_or(ReferenceSignalingError::StateBoundaryViolation)?;
    let participant_key = participant_id.as_str().to_owned();
    let participant = state
        .participants
        .get_mut(&participant_key)
        .ok_or(ReferenceSignalingError::StateBoundaryViolation)?;
    if participant.room_id.as_str() != input.room_id.as_str()
        || participant.phase != ReferenceParticipantPhase::Joined
    {
        return Err(ReferenceSignalingError::StateBoundaryViolation);
    }
    participant.phase = ReferenceParticipantPhase::Left;
    Ok(outcome(input, SignalingEventKind::ParticipantLeft))
}

fn ensure_participant_joined(
    state: &ReferenceSignalingState,
    input: &ReferenceSignalingCommandInput,
) -> Result<(), ReferenceSignalingError> {
    let participant_id = input
        .participant_id
        .as_ref()
        .ok_or(ReferenceSignalingError::StateBoundaryViolation)?;
    let participant = state
        .participants
        .get(participant_id.as_str())
        .ok_or(ReferenceSignalingError::StateBoundaryViolation)?;
    if participant.room_id.as_str() != input.room_id.as_str()
        || participant.phase != ReferenceParticipantPhase::Joined
    {
        return Err(ReferenceSignalingError::StateBoundaryViolation);
    }
    Ok(())
}

fn outcome(
    input: &ReferenceSignalingCommandInput,
    kind: SignalingEventKind,
) -> ReferenceSignalingOutcome {
    ReferenceSignalingOutcome::new(
        input.correlation_id.clone(),
        kind,
        input.room_id.clone(),
        input.participant_id.clone(),
        DistroEvidenceReason::DistroOk,
    )
}
