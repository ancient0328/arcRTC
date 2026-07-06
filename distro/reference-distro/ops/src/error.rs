//! reference runtime / ops のtyped errorです。

use arcrtc_distro_evidence::DistroEvidenceReason;

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
    pub const fn distro_reason(&self) -> DistroEvidenceReason {
        match self {
            Self::RuntimeExecutorError => DistroEvidenceReason::RuntimeExecutorError,
            Self::CommandScopeMismatch => DistroEvidenceReason::CommandScopeMismatch,
            Self::EvidenceFieldsIncomplete | Self::EvidenceWriteError => {
                DistroEvidenceReason::EvidenceFieldsIncomplete
            }
        }
    }
}
