//! product TURN のtyped errorです。

use arcrtc_implementation_evidence::ImplementationEvidenceReason;

/// product TURN error の閉集合です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProductTurnError {
    /// Kernel contract が利用不能です。
    KernelContractUnavailable,
    /// Kernel contract が一致しません。
    KernelContractMismatch,
    /// state boundary violationです。
    StateBoundaryViolation,
    /// readiness が採用されていません。
    ReadinessNotAdmitted,
}

impl ProductTurnError {
    /// implementation evidence reasonへ変換します。
    pub const fn implementation_reason(&self) -> ImplementationEvidenceReason {
        match self {
            Self::KernelContractUnavailable => {
                ImplementationEvidenceReason::KernelContractUnavailable
            }
            Self::KernelContractMismatch => ImplementationEvidenceReason::KernelContractMismatch,
            Self::StateBoundaryViolation => ImplementationEvidenceReason::StateBoundaryViolation,
            Self::ReadinessNotAdmitted => ImplementationEvidenceReason::ReadinessNotAdmitted,
        }
    }
}
