//! product persistence topology のtyped errorです。

/// product persistence topology error の閉集合です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProductPersistenceTopologyError {
    /// provider が採用されていません。
    ProviderNotAdmitted,
    /// projection mapping が違反しています。
    ProjectionMappingViolation,
}

impl ProductPersistenceTopologyError {
    /// implementation evidence reasonへ変換します。
    pub const fn implementation_reason(
        &self,
    ) -> arcrtc_implementation_evidence::ImplementationEvidenceReason {
        match self {
            Self::ProviderNotAdmitted => {
                arcrtc_implementation_evidence::ImplementationEvidenceReason::ReadinessNotAdmitted
            }
            Self::ProjectionMappingViolation => {
                arcrtc_implementation_evidence::ImplementationEvidenceReason::StateBoundaryViolation
            }
        }
    }
}
