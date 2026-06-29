//! product restore plan の境界です。

use arcrtc_core_identity::CorrelationId;
use arcrtc_implementation_evidence::ImplementationEvidenceReason;

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
    /// implementation reasonです。
    pub implementation_reason: ImplementationEvidenceReason,
}

/// product restore planを作成します。
pub fn plan_restore(
    correlation_id: CorrelationId,
    source: ProductRestoreSource,
) -> ProductRestorePlan {
    let implementation_reason = if source == ProductRestoreSource::ProviderDeferred {
        ImplementationEvidenceReason::ReadinessNotAdmitted
    } else {
        ImplementationEvidenceReason::ImplementationOk
    };
    ProductRestorePlan {
        correlation_id,
        source,
        implementation_reason,
    }
}
