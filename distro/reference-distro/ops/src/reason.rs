//! shared distro reason のre-export helperです。

use arcrtc_distro_evidence::DistroEvidenceReason;

/// distro evidence reason の閉集合を返します。
pub const fn distro_reason_closed_set() -> &'static [DistroEvidenceReason] {
    &[
        DistroEvidenceReason::DistroOk,
        DistroEvidenceReason::KernelContractUnavailable,
        DistroEvidenceReason::KernelContractMismatch,
        DistroEvidenceReason::DependencyNotAdmitted,
        DistroEvidenceReason::RuntimeExecutorError,
        DistroEvidenceReason::StateBoundaryViolation,
        DistroEvidenceReason::FixtureIdentityInvalid,
        DistroEvidenceReason::EvidenceFieldsIncomplete,
        DistroEvidenceReason::CommandScopeMismatch,
        DistroEvidenceReason::BenchmarkScopeMismatch,
        DistroEvidenceReason::RealDeviceScopeMismatch,
        DistroEvidenceReason::ReadinessNotAdmitted,
    ]
}
