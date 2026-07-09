//! product monitoring のtyped errorです。

use arcrtc_distro_evidence::DistroEvidenceReason;

/// product monitoring error の閉集合です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProductMonitoringError {
    /// evidence field が不足しています。
    EvidenceFieldsIncomplete,
    /// command scope が一致しません。
    CommandScopeMismatch,
    /// readiness が採用されていません。
    ReadinessNotAdmitted,
}

impl ProductMonitoringError {
    /// distro evidence reasonへ変換します。
    pub const fn distro_reason(&self) -> DistroEvidenceReason {
        match self {
            Self::EvidenceFieldsIncomplete => DistroEvidenceReason::EvidenceFieldsIncomplete,
            Self::CommandScopeMismatch => DistroEvidenceReason::CommandScopeMismatch,
            Self::ReadinessNotAdmitted => DistroEvidenceReason::ReadinessNotAdmitted,
        }
    }
}
