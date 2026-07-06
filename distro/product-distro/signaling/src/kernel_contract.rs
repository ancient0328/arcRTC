//! product Signaling と Kernel / reference outcome の接続境界です。

use arcrtc_core_identity::CorrelationId;
use arcrtc_distro_evidence::{
    DistroEvidenceReason, DistroNonClaimScope, DistroPlane,
};
use arcrtc_product_policy::ProductPolicyDecision;
use arcrtc_reference_output::ReferenceSignalingOutcome;

use crate::error::ProductSignalingError;

/// product Signaling policy inputです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductSignalingPolicyInput {
    /// correlation idです。
    pub correlation_id: CorrelationId,
    /// reference Signaling outcomeです。
    pub reference_outcome: ReferenceSignalingOutcome,
    /// product policy decisionです。
    pub policy_decision: ProductPolicyDecision,
}

/// product Signaling outcomeです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductSignalingOutcome {
    /// correlation idです。
    pub correlation_id: CorrelationId,
    /// product local allowed flagです。
    pub allowed: bool,
    /// distro reasonです。
    pub distro_reason: DistroEvidenceReason,
    /// non-claim scopeです。
    pub non_claim_scope: Vec<DistroNonClaimScope>,
}

/// product Signaling runtime descriptorです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductSignalingRuntime {
    /// target planeです。
    pub target_plane: DistroPlane,
    /// public endpoint claimです。
    pub public_endpoint_claimed: bool,
    /// admitted live endpoint evidence refです。
    pub live_endpoint_evidence_ref: Option<String>,
    /// distro reasonです。
    pub distro_reason: DistroEvidenceReason,
}

/// product Signaling runtime descriptorを構築します。
pub const fn build_product_signaling_runtime() -> ProductSignalingRuntime {
    ProductSignalingRuntime {
        target_plane: DistroPlane::Signaling,
        public_endpoint_claimed: false,
        live_endpoint_evidence_ref: None,
        distro_reason: DistroEvidenceReason::DistroOk,
    }
}

/// admitted live endpoint evidence ref から product Signaling runtime descriptorを構築します。
pub fn build_live_product_signaling_runtime(
    live_endpoint_evidence_ref: &str,
) -> Result<ProductSignalingRuntime, ProductSignalingError> {
    if live_endpoint_evidence_ref.trim().is_empty() {
        return Err(ProductSignalingError::ReadinessNotAdmitted);
    }
    Ok(ProductSignalingRuntime {
        target_plane: DistroPlane::Signaling,
        public_endpoint_claimed: true,
        live_endpoint_evidence_ref: Some(live_endpoint_evidence_ref.to_owned()),
        distro_reason: DistroEvidenceReason::DistroOk,
    })
}

/// product Signaling policyを適用します。
pub fn apply_product_signaling_policy(
    input: &ProductSignalingPolicyInput,
) -> Result<ProductSignalingOutcome, ProductSignalingError> {
    if input.reference_outcome.correlation_id.as_str() != input.correlation_id.as_str()
        || input.policy_decision.non_claim_scope.is_empty()
    {
        return Err(ProductSignalingError::StateBoundaryViolation);
    }
    if input.policy_decision.distro_reason
        == DistroEvidenceReason::ReadinessNotAdmitted
    {
        return Err(ProductSignalingError::ReadinessNotAdmitted);
    }
    Ok(ProductSignalingOutcome {
        correlation_id: input.correlation_id.clone(),
        allowed: input.policy_decision.allowed,
        distro_reason: input.policy_decision.distro_reason,
        non_claim_scope: input.policy_decision.non_claim_scope.clone(),
    })
}
