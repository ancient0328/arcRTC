//! reference SFU のtyped errorです。

use arcrtc_implementation_evidence::ImplementationEvidenceReason;

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
    pub const fn implementation_reason(&self) -> ImplementationEvidenceReason {
        match self {
            Self::KernelContractUnavailable => {
                ImplementationEvidenceReason::KernelContractUnavailable
            }
            Self::KernelContractMismatch => ImplementationEvidenceReason::KernelContractMismatch,
            Self::StateBoundaryViolation => ImplementationEvidenceReason::StateBoundaryViolation,
            Self::InvalidFixtureRouteAdmission => {
                ImplementationEvidenceReason::FixtureIdentityInvalid
            }
            Self::EvidenceFieldsIncomplete => {
                ImplementationEvidenceReason::EvidenceFieldsIncomplete
            }
        }
    }
}
