//! product Signaling と Kernel / reference outcome の接続境界です。

use arcrtc_core_identity::CorrelationId;
use arcrtc_implementation_evidence::{
    ImplementationEvidenceReason, ImplementationNonClaimScope, ImplementationPlane,
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
    /// implementation reasonです。
    pub implementation_reason: ImplementationEvidenceReason,
    /// non-claim scopeです。
    pub non_claim_scope: Vec<ImplementationNonClaimScope>,
}

/// product Signaling runtime descriptorです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductSignalingRuntime {
    /// target planeです。
    pub target_plane: ImplementationPlane,
    /// public endpoint claimです。
    pub public_endpoint_claimed: bool,
    /// admitted live endpoint evidence refです。
    pub live_endpoint_evidence_ref: Option<String>,
    /// implementation reasonです。
    pub implementation_reason: ImplementationEvidenceReason,
}

/// product Signaling runtime descriptorを構築します。
pub const fn build_product_signaling_runtime() -> ProductSignalingRuntime {
    ProductSignalingRuntime {
        target_plane: ImplementationPlane::Signaling,
        public_endpoint_claimed: false,
        live_endpoint_evidence_ref: None,
        implementation_reason: ImplementationEvidenceReason::ImplementationOk,
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
        target_plane: ImplementationPlane::Signaling,
        public_endpoint_claimed: true,
        live_endpoint_evidence_ref: Some(live_endpoint_evidence_ref.to_owned()),
        implementation_reason: ImplementationEvidenceReason::ImplementationOk,
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
    if input.policy_decision.implementation_reason
        == ImplementationEvidenceReason::ReadinessNotAdmitted
    {
        return Err(ProductSignalingError::ReadinessNotAdmitted);
    }
    Ok(ProductSignalingOutcome {
        correlation_id: input.correlation_id.clone(),
        allowed: input.policy_decision.allowed,
        implementation_reason: input.policy_decision.implementation_reason,
        non_claim_scope: input.policy_decision.non_claim_scope.clone(),
    })
}
