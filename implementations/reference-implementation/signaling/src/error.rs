//! reference Signaling のtyped errorです。

use arcrtc_implementation_evidence::ImplementationEvidenceReason;

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
    pub const fn implementation_reason(&self) -> ImplementationEvidenceReason {
        match self {
            Self::KernelContractUnavailable => {
                ImplementationEvidenceReason::KernelContractUnavailable
            }
            Self::KernelContractMismatch => ImplementationEvidenceReason::KernelContractMismatch,
            Self::StateBoundaryViolation => ImplementationEvidenceReason::StateBoundaryViolation,
            Self::InvalidFixtureIdentity => ImplementationEvidenceReason::FixtureIdentityInvalid,
            Self::EvidenceFieldsIncomplete => {
                ImplementationEvidenceReason::EvidenceFieldsIncomplete
            }
        }
    }
}
