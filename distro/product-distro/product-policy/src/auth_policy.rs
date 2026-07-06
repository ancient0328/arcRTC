//! product固有auth policyの境界です。

use arcrtc_core_identity::CorrelationId;
use arcrtc_distro_evidence::{
    DistroEvidenceReason, DistroNonClaimScope, DistroPlane,
};

use crate::error::ProductPolicyError;

/// product policy が扱うaction閉集合です。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProductAction {
    /// Signaling join actionです。
    Join,
    /// TURN relay actionです。
    Relay,
    /// SFU publish actionです。
    Publish,
    /// SFU subscribe actionです。
    Subscribe,
    /// observability actionです。
    Observe,
    /// drain actionです。
    Drain,
    /// restore actionです。
    Restore,
}

/// product policy inputです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductPolicyInput {
    /// correlation idです。
    pub correlation_id: CorrelationId,
    /// target planeです。
    pub target_plane: DistroPlane,
    /// fixture identityです。
    pub fixture_identity: Option<String>,
    /// requested actionです。
    pub requested_action: ProductAction,
}

/// product policy decisionです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductPolicyDecision {
    /// policy decisionです。
    pub allowed: bool,
    /// distro reasonです。
    pub distro_reason: DistroEvidenceReason,
    /// non-claim scopeです。
    pub non_claim_scope: Vec<DistroNonClaimScope>,
}

/// product auth policyを評価します。
pub fn evaluate_product_auth_policy(
    input: &ProductPolicyInput,
) -> Result<ProductPolicyDecision, ProductPolicyError> {
    let non_claim_scope = vec![
        DistroNonClaimScope::ProductionReadinessNotClaimed,
        DistroNonClaimScope::LiveReadinessNotClaimed,
    ];
    if input
        .fixture_identity
        .as_deref()
        .map(str::trim)
        .is_some_and(str::is_empty)
    {
        return Err(ProductPolicyError::InvalidFixtureIdentity);
    }
    let distro_reason = if input.fixture_identity.is_some() {
        DistroEvidenceReason::DistroOk
    } else {
        DistroEvidenceReason::FixtureIdentityInvalid
    };
    Ok(ProductPolicyDecision {
        allowed: input.fixture_identity.is_some(),
        distro_reason,
        non_claim_scope,
    })
}
