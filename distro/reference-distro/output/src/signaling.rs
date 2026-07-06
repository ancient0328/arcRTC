//! Signaling reference outcome 型です。

use arcrtc_core_identity::{CorrelationId, ParticipantId, RoomId};
use arcrtc_core_signaling::SignalingEventKind;
use arcrtc_distro_evidence::DistroEvidenceReason;

/// reference Signaling が返すoutcomeです。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceSignalingOutcome {
    /// command / event chain のcorrelation idです。
    pub correlation_id: CorrelationId,
    /// Kernel event kindへのprojectionです。
    pub kind: SignalingEventKind,
    /// 対象roomです。
    pub room_id: RoomId,
    /// 対象participantです。
    pub participant_id: Option<ParticipantId>,
    /// distro-local evidence reasonです。
    pub distro_reason: DistroEvidenceReason,
}

impl ReferenceSignalingOutcome {
    /// Signaling outcomeを作ります。
    pub const fn new(
        correlation_id: CorrelationId,
        kind: SignalingEventKind,
        room_id: RoomId,
        participant_id: Option<ParticipantId>,
        distro_reason: DistroEvidenceReason,
    ) -> Self {
        Self {
            correlation_id,
            kind,
            room_id,
            participant_id,
            distro_reason,
        }
    }
}
