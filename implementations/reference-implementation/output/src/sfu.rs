//! SFU reference outcome 型です。

use arcrtc_core_sfu::SfuDecisionKind;
use arcrtc_implementation_evidence::ImplementationEvidenceReason;

/// reference SFU が返すoutcomeです。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceSfuOutcome {
    /// Kernel SFU decision kindへのprojectionです。
    pub kind: SfuDecisionKind,
    /// implementations-local evidence reasonです。
    pub implementation_reason: ImplementationEvidenceReason,
}

impl ReferenceSfuOutcome {
    /// SFU outcomeを作ります。
    pub const fn new(
        kind: SfuDecisionKind,
        implementation_reason: ImplementationEvidenceReason,
    ) -> Self {
        Self {
            kind,
            implementation_reason,
        }
    }
}
