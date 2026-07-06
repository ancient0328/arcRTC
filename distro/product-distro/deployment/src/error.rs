//! product deployment/runtime のtyped errorです。

use arcrtc_distro_evidence::DistroEvidenceReason;

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
    /// distro evidence reasonへ変換します。
    pub const fn distro_reason(&self) -> DistroEvidenceReason {
        match self {
            Self::RuntimeExecutorError => DistroEvidenceReason::RuntimeExecutorError,
            Self::CommandScopeMismatch => DistroEvidenceReason::CommandScopeMismatch,
            Self::ReadinessNotAdmitted => DistroEvidenceReason::ReadinessNotAdmitted,
        }
    }
}
