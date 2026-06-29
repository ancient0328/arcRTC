//! reference composition runtime step inputです。

use arcrtc_core_identity::{AllocationId, CorrelationId, RoomId, RouteId, SessionId};
use arcrtc_reference_sfu::ReferenceSfuAction;
use arcrtc_reference_signaling::ReferenceSignalingCommandInput;
use arcrtc_reference_turn::ReferenceTurnCommandInput;

/// reference composition step classです。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReferenceCompositionStep {
    /// Signaling plane commandを適用します。
    ApplySignaling(ReferenceSignalingCommandInput),
    /// TURN plane commandを適用します。
    ApplyTurn(ReferenceTurnCommandInput),
    /// SFU plane actionを適用します。
    ApplySfu(ReferenceSfuAction),
    /// Signaling to TURN binding stepです。
    BindSignalingToTurn {
        /// room referenceです。
        room_id: RoomId,
        /// allocation referenceです。
        allocation_id: AllocationId,
    },
    /// Signaling to SFU binding stepです。
    BindSignalingToSfu {
        /// room referenceです。
        room_id: RoomId,
        /// session referenceです。
        session_id: SessionId,
        /// route referenceです。
        route_id: RouteId,
    },
    /// composition stateを検証します。
    Validate,
}

/// reference composition step inputです。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceCompositionStepInput {
    /// step correlation idです。
    pub correlation_id: CorrelationId,
    /// 実行するstepです。
    pub step: ReferenceCompositionStep,
}
