//! reference SFU のtyped errorです。

use arcrtc_distro_evidence::DistroEvidenceReason;

/// reference SFU error の閉集合です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceSfuError {
    /// Kernel contract が利用できません。
    KernelContractUnavailable,
    /// Kernel contract shape が一致しません。
    KernelContractMismatch,
    /// state boundary violationです。
    StateBoundaryViolation,
    /// fixture route admissionが不正です。
    InvalidFixtureRouteAdmission,
    /// evidence field が不足しています。
    EvidenceFieldsIncomplete,
}

impl ReferenceSfuError {
    /// evidence reasonへ変換します。
    pub const fn distro_reason(&self) -> DistroEvidenceReason {
        match self {
            Self::KernelContractUnavailable => {
                DistroEvidenceReason::KernelContractUnavailable
            }
            Self::KernelContractMismatch => DistroEvidenceReason::KernelContractMismatch,
            Self::StateBoundaryViolation => DistroEvidenceReason::StateBoundaryViolation,
            Self::InvalidFixtureRouteAdmission => {
                DistroEvidenceReason::FixtureIdentityInvalid
            }
            Self::EvidenceFieldsIncomplete => {
                DistroEvidenceReason::EvidenceFieldsIncomplete
            }
        }
    }
}
