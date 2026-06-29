//! product deployment/runtime のtyped errorです。

use arcrtc_implementation_evidence::ImplementationEvidenceReason;

/// product runtime error の閉集合です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProductRuntimeError {
    /// runtime executor errorです。
    RuntimeExecutorError,
    /// command scope mismatchです。
    CommandScopeMismatch,
    /// readiness が採用されていません。
    ReadinessNotAdmitted,
}

impl ProductRuntimeError {
    /// implementation evidence reasonへ変換します。
    pub const fn implementation_reason(&self) -> ImplementationEvidenceReason {
        match self {
            Self::RuntimeExecutorError => ImplementationEvidenceReason::RuntimeExecutorError,
            Self::CommandScopeMismatch => ImplementationEvidenceReason::CommandScopeMismatch,
            Self::ReadinessNotAdmitted => ImplementationEvidenceReason::ReadinessNotAdmitted,
        }
    }
}
