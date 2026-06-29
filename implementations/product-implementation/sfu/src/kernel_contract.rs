//! product SFU と Kernel / reference outcome の接続境界です。

use arcrtc_core_identity::CorrelationId;
use arcrtc_implementation_evidence::{
    ImplementationEvidenceReason, ImplementationNonClaimScope, ImplementationPlane,
};
use arcrtc_product_policy::ProductPolicyDecision;
use arcrtc_reference_output::ReferenceSfuOutcome;

use crate::error::ProductSfuError;

/// product SFU policy inputです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductSfuPolicyInput {
    /// correlation idです。
    pub correlation_id: CorrelationId,
    /// reference SFU outcomeです。
    pub reference_outcome: ReferenceSfuOutcome,
    /// product policy decisionです。
    pub policy_decision: ProductPolicyDecision,
}

/// product SFU outcomeです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductSfuOutcome {
    /// correlation idです。
    pub correlation_id: CorrelationId,
    /// product local allowed flagです。
    pub allowed: bool,
    /// implementation reasonです。
    pub implementation_reason: ImplementationEvidenceReason,
    /// non-claim scopeです。
    pub non_claim_scope: Vec<ImplementationNonClaimScope>,
}

/// product SFU runtime descriptorです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductSfuRuntime {
    /// target planeです。
    pub target_plane: ImplementationPlane,
    /// media public endpoint claimです。
    pub media_public_endpoint_claimed: bool,
    /// admitted live endpoint evidence refです。
    pub live_endpoint_evidence_ref: Option<String>,
    /// implementation reasonです。
    pub implementation_reason: ImplementationEvidenceReason,
}

/// product SFU runtime descriptorを構築します。
pub const fn build_product_sfu_runtime() -> ProductSfuRuntime {
    ProductSfuRuntime {
        target_plane: ImplementationPlane::Sfu,
        media_public_endpoint_claimed: false,
        live_endpoint_evidence_ref: None,
        implementation_reason: ImplementationEvidenceReason::ImplementationOk,
    }
}

/// admitted live endpoint evidence ref から product SFU runtime descriptorを構築します。
pub fn build_live_product_sfu_runtime(
    live_endpoint_evidence_ref: &str,
) -> Result<ProductSfuRuntime, ProductSfuError> {
    if live_endpoint_evidence_ref.trim().is_empty() {
        return Err(ProductSfuError::ReadinessNotAdmitted);
    }
    Ok(ProductSfuRuntime {
        target_plane: ImplementationPlane::Sfu,
        media_public_endpoint_claimed: true,
        live_endpoint_evidence_ref: Some(live_endpoint_evidence_ref.to_owned()),
        implementation_reason: ImplementationEvidenceReason::ImplementationOk,
    })
}

/// product SFU policyを適用します。
pub fn apply_product_sfu_policy(
    input: &ProductSfuPolicyInput,
) -> Result<ProductSfuOutcome, ProductSfuError> {
    if input.policy_decision.non_claim_scope.is_empty() {
        return Err(ProductSfuError::StateBoundaryViolation);
    }
    if input.policy_decision.implementation_reason
        == ImplementationEvidenceReason::ReadinessNotAdmitted
    {
        return Err(ProductSfuError::ReadinessNotAdmitted);
    }
    Ok(ProductSfuOutcome {
        correlation_id: input.correlation_id.clone(),
        allowed: input.policy_decision.allowed,
        implementation_reason: input.policy_decision.implementation_reason,
        non_claim_scope: input.policy_decision.non_claim_scope.clone(),
    })
}
