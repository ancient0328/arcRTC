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
    /// distro evidence reasonへ変換します。
    pub const fn distro_reason(
        &self,
    ) -> arcrtc_distro_evidence::DistroEvidenceReason {
        match self {
            Self::ProviderNotAdmitted => {
                arcrtc_distro_evidence::DistroEvidenceReason::ReadinessNotAdmitted
            }
            Self::ProjectionMappingViolation => {
                arcrtc_distro_evidence::DistroEvidenceReason::StateBoundaryViolation
            }
        }
    }
}
