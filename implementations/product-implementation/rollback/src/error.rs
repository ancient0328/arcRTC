//! product rollback のtyped errorです。

use arcrtc_implementation_evidence::ImplementationEvidenceReason;

/// product rollback error の閉集合です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProductRollbackError {
    /// runtime executor errorです。
    RuntimeExecutorError,
    /// readiness が採用されていません。
    ReadinessNotAdmitted,
}

impl ProductRollbackError {
    /// implementation evidence reasonへ変換します。
    pub const fn implementation_reason(&self) -> ImplementationEvidenceReason {
        match self {
            Self::RuntimeExecutorError => ImplementationEvidenceReason::RuntimeExecutorError,
            Self::ReadinessNotAdmitted => ImplementationEvidenceReason::ReadinessNotAdmitted,
        }
    }
}
