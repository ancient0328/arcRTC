//! reference runtime / ops のtyped errorです。

use arcrtc_implementation_evidence::ImplementationEvidenceReason;

/// reference runtime error の閉集合です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceRuntimeError {
    /// runtime executor errorです。
    RuntimeExecutorError,
    /// command scope mismatchです。
    CommandScopeMismatch,
    /// evidence fieldが不足しています。
    EvidenceFieldsIncomplete,
    /// evidence write errorです。
    EvidenceWriteError,
}

impl ReferenceRuntimeError {
    /// evidence reasonへ変換します。
    pub const fn implementation_reason(&self) -> ImplementationEvidenceReason {
        match self {
            Self::RuntimeExecutorError => ImplementationEvidenceReason::RuntimeExecutorError,
            Self::CommandScopeMismatch => ImplementationEvidenceReason::CommandScopeMismatch,
            Self::EvidenceFieldsIncomplete | Self::EvidenceWriteError => {
                ImplementationEvidenceReason::EvidenceFieldsIncomplete
            }
        }
    }
}
