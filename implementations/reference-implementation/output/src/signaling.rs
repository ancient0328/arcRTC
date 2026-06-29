//! Signaling reference outcome 型です。

use arcrtc_core_identity::{CorrelationId, ParticipantId, RoomId};
use arcrtc_core_signaling::SignalingEventKind;
use arcrtc_implementation_evidence::ImplementationEvidenceReason;

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
    /// implementations-local evidence reasonです。
    pub implementation_reason: ImplementationEvidenceReason,
}

impl ReferenceSignalingOutcome {
    /// Signaling outcomeを作ります。
    pub const fn new(
        correlation_id: CorrelationId,
        kind: SignalingEventKind,
        room_id: RoomId,
        participant_id: Option<ParticipantId>,
        implementation_reason: ImplementationEvidenceReason,
    ) -> Self {
        Self {
            correlation_id,
            kind,
            room_id,
            participant_id,
            implementation_reason,
        }
    }
}
