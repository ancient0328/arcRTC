//! TURN reference outcome 型です。

use arcrtc_core_turn::TurnDecisionKind;
use arcrtc_implementation_evidence::ImplementationEvidenceReason;

/// reference TURN が返すoutcomeです。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceTurnOutcome {
    /// Kernel TURN decision kindへのprojectionです。
    pub kind: TurnDecisionKind,
    /// implementations-local evidence reasonです。
    pub implementation_reason: ImplementationEvidenceReason,
}

impl ReferenceTurnOutcome {
    /// TURN outcomeを作ります。
    pub const fn new(
        kind: TurnDecisionKind,
        implementation_reason: ImplementationEvidenceReason,
    ) -> Self {
        Self {
            kind,
            implementation_reason,
        }
    }
}
