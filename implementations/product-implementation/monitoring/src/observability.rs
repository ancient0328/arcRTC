//! product observability record の境界です。

use arcrtc_core_identity::CorrelationId;
use arcrtc_implementation_evidence::{ImplementationEvidenceReason, ImplementationPlane};

/// product observability recordです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductObservabilityRecord {
    /// correlation idです。
    pub correlation_id: CorrelationId,
    /// target planeです。
    pub target_plane: ImplementationPlane,
    /// metric nameです。
    pub metric_name: &'static str,
    /// implementation reasonです。
    pub implementation_reason: ImplementationEvidenceReason,
}

/// product observability recordを構築します。
pub fn build_observability_record(
    correlation_id: CorrelationId,
    target_plane: ImplementationPlane,
    metric_name: &'static str,
) -> ProductObservabilityRecord {
    ProductObservabilityRecord {
        correlation_id,
        target_plane,
        metric_name,
        implementation_reason: if metric_name.trim().is_empty() {
            ImplementationEvidenceReason::EvidenceFieldsIncomplete
        } else {
            ImplementationEvidenceReason::ImplementationOk
        },
    }
}
