//! product TURN と Kernel / reference outcome の接続境界です。

use arcrtc_core_identity::CorrelationId;
use arcrtc_implementation_evidence::{
    ImplementationEvidenceReason, ImplementationNonClaimScope, ImplementationPlane,
};
use arcrtc_product_policy::ProductPolicyDecision;
use arcrtc_reference_output::ReferenceTurnOutcome;

use crate::error::ProductTurnError;

/// product TURN policy inputです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductTurnPolicyInput {
    /// correlation idです。
    pub correlation_id: CorrelationId,
    /// reference TURN outcomeです。
    pub reference_outcome: ReferenceTurnOutcome,
    /// product policy decisionです。
    pub policy_decision: ProductPolicyDecision,
}

/// product TURN outcomeです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductTurnOutcome {
    /// correlation idです。
    pub correlation_id: CorrelationId,
    /// product local allowed flagです。
    pub allowed: bool,
    /// implementation reasonです。
    pub implementation_reason: ImplementationEvidenceReason,
    /// non-claim scopeです。
    pub non_claim_scope: Vec<ImplementationNonClaimScope>,
}

/// product TURN runtime descriptorです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductTurnRuntime {
    /// target planeです。
    pub target_plane: ImplementationPlane,
    /// relay public endpoint claimです。
    pub relay_public_endpoint_claimed: bool,
    /// admitted live endpoint evidence refです。
    pub live_endpoint_evidence_ref: Option<String>,
    /// implementation reasonです。
    pub implementation_reason: ImplementationEvidenceReason,
}

/// product TURN runtime descriptorを構築します。
pub const fn build_product_turn_runtime() -> ProductTurnRuntime {
    ProductTurnRuntime {
        target_plane: ImplementationPlane::Turn,
        relay_public_endpoint_claimed: false,
        live_endpoint_evidence_ref: None,
        implementation_reason: ImplementationEvidenceReason::ImplementationOk,
    }
}

/// admitted live endpoint evidence ref から product TURN runtime descriptorを構築します。
pub fn build_live_product_turn_runtime(
    live_endpoint_evidence_ref: &str,
) -> Result<ProductTurnRuntime, ProductTurnError> {
    if live_endpoint_evidence_ref.trim().is_empty() {
        return Err(ProductTurnError::ReadinessNotAdmitted);
    }
    Ok(ProductTurnRuntime {
        target_plane: ImplementationPlane::Turn,
        relay_public_endpoint_claimed: true,
        live_endpoint_evidence_ref: Some(live_endpoint_evidence_ref.to_owned()),
        implementation_reason: ImplementationEvidenceReason::ImplementationOk,
    })
}

/// product TURN policyを適用します。
pub fn apply_product_turn_policy(
    input: &ProductTurnPolicyInput,
) -> Result<ProductTurnOutcome, ProductTurnError> {
    if input.policy_decision.non_claim_scope.is_empty() {
        return Err(ProductTurnError::StateBoundaryViolation);
    }
    if input.policy_decision.implementation_reason
        == ImplementationEvidenceReason::ReadinessNotAdmitted
    {
        return Err(ProductTurnError::ReadinessNotAdmitted);
    }
    Ok(ProductTurnOutcome {
        correlation_id: input.correlation_id.clone(),
        allowed: input.policy_decision.allowed,
        implementation_reason: input.policy_decision.implementation_reason,
        non_claim_scope: input.policy_decision.non_claim_scope.clone(),
    })
}
