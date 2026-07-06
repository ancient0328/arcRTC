//! reference TURN のtyped errorです。

use arcrtc_distro_evidence::DistroEvidenceReason;

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
    pub const fn distro_reason(&self) -> DistroEvidenceReason {
        match self {
            Self::KernelContractUnavailable => {
                DistroEvidenceReason::KernelContractUnavailable
            }
            Self::KernelContractMismatch => DistroEvidenceReason::KernelContractMismatch,
            Self::StateBoundaryViolation => DistroEvidenceReason::StateBoundaryViolation,
            Self::InvalidFixtureCredential => DistroEvidenceReason::FixtureIdentityInvalid,
            Self::EvidenceFieldsIncomplete => {
                DistroEvidenceReason::EvidenceFieldsIncomplete
            }
        }
    }
}
