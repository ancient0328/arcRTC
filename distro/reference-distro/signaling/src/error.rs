//! reference Signaling のtyped errorです。

use arcrtc_distro_evidence::DistroEvidenceReason;

/// reference Signaling error の閉集合です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceSignalingError {
    /// Kernel contract が利用できません。
    KernelContractUnavailable,
    /// Kernel contract shape が一致しません。
    KernelContractMismatch,
    /// state boundary violationです。
    StateBoundaryViolation,
    /// fixture identityまたはpayloadが不正です。
    InvalidFixtureIdentity,
    /// evidence field が不足しています。
    EvidenceFieldsIncomplete,
}

impl ReferenceSignalingError {
    /// evidence reasonへ変換します。
    pub const fn distro_reason(&self) -> DistroEvidenceReason {
        match self {
            Self::KernelContractUnavailable => DistroEvidenceReason::KernelContractUnavailable,
            Self::KernelContractMismatch => DistroEvidenceReason::KernelContractMismatch,
            Self::StateBoundaryViolation => DistroEvidenceReason::StateBoundaryViolation,
            Self::InvalidFixtureIdentity => DistroEvidenceReason::FixtureIdentityInvalid,
            Self::EvidenceFieldsIncomplete => DistroEvidenceReason::EvidenceFieldsIncomplete,
        }
    }
}
