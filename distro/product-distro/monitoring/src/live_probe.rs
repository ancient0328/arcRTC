//! live monitoring probe の境界です。

use arcrtc_core_identity::CorrelationId;
use arcrtc_distro_evidence::{DistroEvidenceReason, DistroPlane};

use crate::error::ProductMonitoringError;

/// live monitoring probe record です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductLiveMonitoringProbe {
    /// correlation id です。
    pub correlation_id: CorrelationId,
    /// target plane です。
    pub target_plane: DistroPlane,
    /// metric name です。
    pub metric_name: &'static str,
    /// probe boundary です。
    pub probe_boundary: &'static str,
    /// distro reason です。
    pub distro_reason: DistroEvidenceReason,
}

/// live monitoring probe を構築します。
pub fn build_live_monitoring_probe(
    correlation_id: CorrelationId,
    target_plane: DistroPlane,
    metric_name: &'static str,
) -> Result<ProductLiveMonitoringProbe, ProductMonitoringError> {
    if metric_name.trim().is_empty() {
        return Err(ProductMonitoringError::EvidenceFieldsIncomplete);
    }
    Ok(ProductLiveMonitoringProbe {
        correlation_id,
        target_plane,
        metric_name,
        probe_boundary: "live-readiness-probe-evidence-only",
        distro_reason: DistroEvidenceReason::DistroOk,
    })
}
