//! shared implementation reason のre-export helperです。

use arcrtc_implementation_evidence::ImplementationEvidenceReason;

/// implementations evidence reason の閉集合を返します。
pub const fn implementation_reason_closed_set() -> &'static [ImplementationEvidenceReason] {
    &[
        ImplementationEvidenceReason::ImplementationOk,
        ImplementationEvidenceReason::KernelContractUnavailable,
        ImplementationEvidenceReason::KernelContractMismatch,
        ImplementationEvidenceReason::DependencyNotAdmitted,
        ImplementationEvidenceReason::RuntimeExecutorError,
        ImplementationEvidenceReason::StateBoundaryViolation,
        ImplementationEvidenceReason::FixtureIdentityInvalid,
        ImplementationEvidenceReason::EvidenceFieldsIncomplete,
        ImplementationEvidenceReason::CommandScopeMismatch,
        ImplementationEvidenceReason::BenchmarkScopeMismatch,
        ImplementationEvidenceReason::RealDeviceScopeMismatch,
        ImplementationEvidenceReason::ReadinessNotAdmitted,
    ]
}
