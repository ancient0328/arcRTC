//! product observability record の境界です。

use arcrtc_core_identity::CorrelationId;
use arcrtc_distro_evidence::{DistroEvidenceReason, DistroPlane};

/// product observability recordです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductObservabilityRecord {
    /// correlation idです。
    pub correlation_id: CorrelationId,
    /// target planeです。
    pub target_plane: DistroPlane,
    /// metric nameです。
    pub metric_name: &'static str,
    /// distro reasonです。
    pub distro_reason: DistroEvidenceReason,
}

/// product observability recordを構築します。
pub fn build_observability_record(
    correlation_id: CorrelationId,
    target_plane: DistroPlane,
    metric_name: &'static str,
) -> ProductObservabilityRecord {
    ProductObservabilityRecord {
        correlation_id,
        target_plane,
        metric_name,
        distro_reason: if metric_name.trim().is_empty() {
            DistroEvidenceReason::EvidenceFieldsIncomplete
        } else {
            DistroEvidenceReason::DistroOk
        },
    }
}
