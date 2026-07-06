//! reference composition outcome 型です。

use arcrtc_core_identity::CorrelationId;
use arcrtc_distro_evidence::DistroEvidenceReason;

/// reference composition が返すoutcomeです。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceCompositionOutcome {
    /// composition step のcorrelation idです。
    pub correlation_id: CorrelationId,
    /// distro-local evidence reasonです。
    pub distro_reason: DistroEvidenceReason,
}

impl ReferenceCompositionOutcome {
    /// composition outcomeを作ります。
    pub const fn new(
        correlation_id: CorrelationId,
        distro_reason: DistroEvidenceReason,
    ) -> Self {
        Self {
            correlation_id,
            distro_reason,
        }
    }
}
