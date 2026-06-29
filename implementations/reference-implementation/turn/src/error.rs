//! reference TURN のtyped errorです。

use arcrtc_implementation_evidence::ImplementationEvidenceReason;

/// reference TURN error の閉集合です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceTurnError {
    /// Kernel contract が利用できません。
    KernelContractUnavailable,
    /// Kernel contract shape が一致しません。
    KernelContractMismatch,
    /// state boundary violationです。
    StateBoundaryViolation,
    /// fixture credentialが不正です。
    InvalidFixtureCredential,
    /// evidence field が不足しています。
    EvidenceFieldsIncomplete,
}

impl ReferenceTurnError {
    /// evidence reasonへ変換します。
    pub const fn implementation_reason(&self) -> ImplementationEvidenceReason {
        match self {
            Self::KernelContractUnavailable => {
                ImplementationEvidenceReason::KernelContractUnavailable
            }
            Self::KernelContractMismatch => ImplementationEvidenceReason::KernelContractMismatch,
            Self::StateBoundaryViolation => ImplementationEvidenceReason::StateBoundaryViolation,
            Self::InvalidFixtureCredential => ImplementationEvidenceReason::FixtureIdentityInvalid,
            Self::EvidenceFieldsIncomplete => {
                ImplementationEvidenceReason::EvidenceFieldsIncomplete
            }
        }
    }
}
