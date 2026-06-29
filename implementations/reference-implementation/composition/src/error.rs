//! reference composition のtyped errorです。

use arcrtc_implementation_evidence::ImplementationEvidenceReason;

/// reference composition error の閉集合です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceCompositionError {
    /// Kernel contract shape が一致しません。
    KernelContractMismatch,
    /// state boundary violationです。
    StateBoundaryViolation,
    /// evidence field が不足しています。
    EvidenceFieldsIncomplete,
    /// plane実行に失敗しました。
    PlaneExecutionFailed,
}

impl ReferenceCompositionError {
    /// evidence reasonへ変換します。
    pub const fn implementation_reason(&self) -> ImplementationEvidenceReason {
        match self {
            Self::KernelContractMismatch => ImplementationEvidenceReason::KernelContractMismatch,
            Self::StateBoundaryViolation => ImplementationEvidenceReason::StateBoundaryViolation,
            Self::EvidenceFieldsIncomplete => {
                ImplementationEvidenceReason::EvidenceFieldsIncomplete
            }
            Self::PlaneExecutionFailed => ImplementationEvidenceReason::RuntimeExecutorError,
        }
    }
}
