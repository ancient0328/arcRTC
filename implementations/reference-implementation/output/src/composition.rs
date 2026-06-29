//! reference composition outcome 型です。

use arcrtc_core_identity::CorrelationId;
use arcrtc_implementation_evidence::ImplementationEvidenceReason;

/// reference composition が返すoutcomeです。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceCompositionOutcome {
    /// composition step のcorrelation idです。
    pub correlation_id: CorrelationId,
    /// implementations-local evidence reasonです。
    pub implementation_reason: ImplementationEvidenceReason,
}

impl ReferenceCompositionOutcome {
    /// composition outcomeを作ります。
    pub const fn new(
        correlation_id: CorrelationId,
        implementation_reason: ImplementationEvidenceReason,
    ) -> Self {
        Self {
            correlation_id,
            implementation_reason,
        }
    }
}
