//! production monitoring probe の境界です。

use arcrtc_core_identity::CorrelationId;
use arcrtc_implementation_evidence::{ImplementationEvidenceReason, ImplementationPlane};

use crate::error::ProductMonitoringError;

/// production monitoring probe record です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductProductionMonitoringProbe {
    /// correlation id です。
    pub correlation_id: CorrelationId,
    /// target plane です。
    pub target_plane: ImplementationPlane,
    /// metric name です。
    pub metric_name: &'static str,
    /// probe boundary です。
    pub probe_boundary: &'static str,
    /// implementation reason です。
    pub implementation_reason: ImplementationEvidenceReason,
}

/// production monitoring probe を構築します。
pub fn build_production_monitoring_probe(
    correlation_id: CorrelationId,
    target_plane: ImplementationPlane,
    metric_name: &'static str,
) -> Result<ProductProductionMonitoringProbe, ProductMonitoringError> {
    if metric_name.trim().is_empty() {
        return Err(ProductMonitoringError::EvidenceFieldsIncomplete);
    }
    Ok(ProductProductionMonitoringProbe {
        correlation_id,
        target_plane,
        metric_name,
        probe_boundary: "production-readiness-probe-evidence-only",
        implementation_reason: ImplementationEvidenceReason::ImplementationOk,
    })
}
