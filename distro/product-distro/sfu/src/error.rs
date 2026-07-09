//! product SFU のtyped errorです。

use arcrtc_distro_evidence::DistroEvidenceReason;

/// product SFU error の閉集合です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProductSfuError {
    /// Kernel contract が利用不能です。
    KernelContractUnavailable,
    /// Kernel contract が一致しません。
    KernelContractMismatch,
    /// state boundary violationです。
    StateBoundaryViolation,
    /// readiness が採用されていません。
    ReadinessNotAdmitted,
}

impl ProductSfuError {
    /// distro evidence reasonへ変換します。
    pub const fn distro_reason(&self) -> DistroEvidenceReason {
        match self {
            Self::KernelContractUnavailable => DistroEvidenceReason::KernelContractUnavailable,
            Self::KernelContractMismatch => DistroEvidenceReason::KernelContractMismatch,
            Self::StateBoundaryViolation => DistroEvidenceReason::StateBoundaryViolation,
            Self::ReadinessNotAdmitted => DistroEvidenceReason::ReadinessNotAdmitted,
        }
    }
}
