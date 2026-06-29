//! product security reason と evidence reason の対応境界です。

use arcrtc_implementation_evidence::ImplementationEvidenceReason;

use crate::error::ProductPolicyError;

/// product policy error を implementations evidence reason へ写像します。
pub const fn map_product_security_reason(
    error: ProductPolicyError,
) -> ImplementationEvidenceReason {
    match error {
        ProductPolicyError::InvalidFixtureIdentity => {
            ImplementationEvidenceReason::FixtureIdentityInvalid
        }
        ProductPolicyError::SecurityReasonMappingFailed => {
            ImplementationEvidenceReason::StateBoundaryViolation
        }
        ProductPolicyError::ReadinessNotAdmitted => {
            ImplementationEvidenceReason::ReadinessNotAdmitted
        }
    }
}
