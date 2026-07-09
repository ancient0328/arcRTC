//! reference composition のtyped errorです。

use arcrtc_distro_evidence::DistroEvidenceReason;

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
    pub const fn distro_reason(&self) -> DistroEvidenceReason {
        match self {
            Self::KernelContractMismatch => DistroEvidenceReason::KernelContractMismatch,
            Self::StateBoundaryViolation => DistroEvidenceReason::StateBoundaryViolation,
            Self::EvidenceFieldsIncomplete => DistroEvidenceReason::EvidenceFieldsIncomplete,
            Self::PlaneExecutionFailed => DistroEvidenceReason::RuntimeExecutorError,
        }
    }
}
