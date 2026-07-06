//! product restore plan の境界です。

use arcrtc_core_identity::CorrelationId;
use arcrtc_distro_evidence::DistroEvidenceReason;

/// product restore sourceです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProductRestoreSource {
    /// in-memory projectionからrestoreします。
    InMemoryProjection,
    /// evidence reportからrestoreします。
    EvidenceReport,
    /// provider restore は未採用です。
    ProviderDeferred,
    /// live readiness admitted restoreです。
    LiveEvidenceReport,
}

/// product restore planです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductRestorePlan {
    /// correlation idです。
    pub correlation_id: CorrelationId,
    /// restore sourceです。
    pub source: ProductRestoreSource,
    /// distro reasonです。
    pub distro_reason: DistroEvidenceReason,
}

/// product restore planを作成します。
pub fn plan_restore(
    correlation_id: CorrelationId,
    source: ProductRestoreSource,
) -> ProductRestorePlan {
    let distro_reason = if source == ProductRestoreSource::ProviderDeferred {
        DistroEvidenceReason::ReadinessNotAdmitted
    } else {
        DistroEvidenceReason::DistroOk
    };
    ProductRestorePlan {
        correlation_id,
        source,
        distro_reason,
    }
}
