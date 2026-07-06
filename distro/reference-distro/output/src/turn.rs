//! TURN reference outcome 型です。

use arcrtc_core_turn::TurnDecisionKind;
use arcrtc_distro_evidence::DistroEvidenceReason;

/// reference TURN が返すoutcomeです。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceTurnOutcome {
    /// Kernel TURN decision kindへのprojectionです。
    pub kind: TurnDecisionKind,
    /// distro-local evidence reasonです。
    pub distro_reason: DistroEvidenceReason,
}

impl ReferenceTurnOutcome {
    /// TURN outcomeを作ります。
    pub const fn new(
        kind: TurnDecisionKind,
        distro_reason: DistroEvidenceReason,
    ) -> Self {
        Self {
            kind,
            distro_reason,
        }
    }
}
