//! product security reason と evidence reason の対応境界です。

use arcrtc_distro_evidence::DistroEvidenceReason;

use crate::error::ProductPolicyError;

/// product policy error を distro evidence reason へ写像します。
pub const fn map_product_security_reason(
    error: ProductPolicyError,
) -> DistroEvidenceReason {
    match error {
        ProductPolicyError::InvalidFixtureIdentity => {
            DistroEvidenceReason::FixtureIdentityInvalid
        }
        ProductPolicyError::SecurityReasonMappingFailed => {
            DistroEvidenceReason::StateBoundaryViolation
        }
        ProductPolicyError::ReadinessNotAdmitted => {
            DistroEvidenceReason::ReadinessNotAdmitted
        }
    }
}
