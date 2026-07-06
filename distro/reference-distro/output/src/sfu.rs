//! SFU reference outcome 型です。

use arcrtc_core_sfu::SfuDecisionKind;
use arcrtc_distro_evidence::DistroEvidenceReason;

/// reference SFU が返すoutcomeです。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceSfuOutcome {
    /// Kernel SFU decision kindへのprojectionです。
    pub kind: SfuDecisionKind,
    /// distro-local evidence reasonです。
    pub distro_reason: DistroEvidenceReason,
}

impl ReferenceSfuOutcome {
    /// SFU outcomeを作ります。
    pub const fn new(
        kind: SfuDecisionKind,
        distro_reason: DistroEvidenceReason,
    ) -> Self {
        Self {
            kind,
            distro_reason,
        }
    }
}
