//! Signaling reference input と Kernel command builder の境界です。

use arcrtc_core_command::{CommandEnvelope, CommandType, CommandVersion, TargetSurface};
use arcrtc_core_identity::{CorrelationId, ParticipantId, RoomId};
use arcrtc_core_signaling::{SignalingCommand, SignalingCommandKind, SignalingSubject};

use crate::{
    error::ReferenceSignalingError,
    fixture_identity::{FixtureIceCandidate, FixtureSessionDescription},
};

/// reference Signaling command inputです。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceSignalingCommandInput {
    /// command chain correlation idです。
    pub correlation_id: CorrelationId,
    /// room referenceです。
    pub room_id: RoomId,
    /// participant referenceです。
    pub participant_id: Option<ParticipantId>,
    /// Signaling command kindです。
    pub kind: SignalingCommandKind,
    /// reference local payloadです。
    pub payload: ReferenceSignalingPayload,
}

impl ReferenceSignalingCommandInput {
    /// reference Signaling command inputを作ります。
    pub const fn new(
        correlation_id: CorrelationId,
        room_id: RoomId,
        participant_id: Option<ParticipantId>,
        kind: SignalingCommandKind,
        payload: ReferenceSignalingPayload,
    ) -> Self {
        Self {
            correlation_id,
            room_id,
            participant_id,
            kind,
            payload,
        }
    }
}

/// reference Signaling payload の閉集合です。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReferenceSignalingPayload {
    /// room join requestです。
    JoinRoom,
    /// room leave requestです。
    LeaveRoom,
    /// offer fixture送信です。
    SendOffer {
        /// offer session fixtureです。
        session: FixtureSessionDescription,
    },
    /// answer fixture送信です。
    SendAnswer {
        /// answer session fixtureです。
        session: FixtureSessionDescription,
    },
    /// ICE candidate fixture送信です。
    SendIceCandidate {
        /// ICE candidate fixtureです。
        candidate: FixtureIceCandidate,
    },
    /// TURN credential requestです。
    RequestTurnCredential,
    /// forwarding acknowledgementです。
    AcknowledgeForward,
}

/// Kernel Signaling commandを構築します。
pub fn build_kernel_signaling_command(
    input: &ReferenceSignalingCommandInput,
) -> Result<SignalingCommand<ReferenceSignalingPayload>, ReferenceSignalingError> {
    let subject = SignalingSubject::new(input.room_id.clone(), input.participant_id.clone());
    let envelope = CommandEnvelope::new(
        input.correlation_id.clone(),
        signaling_command_type(input.kind),
        CommandVersion::new(1),
        TargetSurface::Signaling,
        subject,
    );
    Ok(SignalingCommand::new(
        envelope,
        input.kind,
        input.payload.clone(),
    ))
}

const fn signaling_command_type(kind: SignalingCommandKind) -> CommandType {
    match kind {
        SignalingCommandKind::JoinRoom => CommandType::new("reference.signaling.join_room"),
        SignalingCommandKind::LeaveRoom => CommandType::new("reference.signaling.leave_room"),
        SignalingCommandKind::SendOffer => CommandType::new("reference.signaling.send_offer"),
        SignalingCommandKind::SendAnswer => CommandType::new("reference.signaling.send_answer"),
        SignalingCommandKind::SendIceCandidate => {
            CommandType::new("reference.signaling.send_ice_candidate")
        }
        SignalingCommandKind::RequestTurnCredential => {
            CommandType::new("reference.signaling.request_turn_credential")
        }
        SignalingCommandKind::AcknowledgeForward => {
            CommandType::new("reference.signaling.acknowledge_forward")
        }
    }
}
