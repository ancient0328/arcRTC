//! product monitoring のtyped errorです。

use arcrtc_implementation_evidence::ImplementationEvidenceReason;

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
    /// implementation evidence reasonへ変換します。
    pub const fn implementation_reason(&self) -> ImplementationEvidenceReason {
        match self {
            Self::EvidenceFieldsIncomplete => {
                ImplementationEvidenceReason::EvidenceFieldsIncomplete
            }
            Self::CommandScopeMismatch => ImplementationEvidenceReason::CommandScopeMismatch,
            Self::ReadinessNotAdmitted => ImplementationEvidenceReason::ReadinessNotAdmitted,
        }
    }
}
