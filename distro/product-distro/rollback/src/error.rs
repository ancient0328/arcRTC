//! product rollback のtyped errorです。

use arcrtc_distro_evidence::DistroEvidenceReason;

/// product rollback error の閉集合です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProductRollbackError {
    /// runtime executor errorです。
    RuntimeExecutorError,
    /// readiness が採用されていません。
    ReadinessNotAdmitted,
}

impl ProductRollbackError {
    /// distro evidence reasonへ変換します。
    pub const fn distro_reason(&self) -> DistroEvidenceReason {
        match self {
            Self::RuntimeExecutorError => DistroEvidenceReason::RuntimeExecutorError,
            Self::ReadinessNotAdmitted => DistroEvidenceReason::ReadinessNotAdmitted,
        }
    }
}
