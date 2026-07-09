//! product固有auth policyの境界です。

use arcrtc_core_identity::CorrelationId;
use arcrtc_distro_evidence::{DistroEvidenceReason, DistroNonClaimScope, DistroPlane};

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

/// product tenant admission の閉集合です。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProductTenantClass {
    /// local fixture tenant が採用済みです。
    LocalFixtureTenant,
    /// tenant admission が採用されていません。
    TenantNotAdmitted,
}

/// product quota policy の閉集合です。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProductQuotaClass {
    /// local controlled quota が利用可能です。
    LocalQuotaAvailable,
    /// quota policy が対象 action を許可しません。
    QuotaExceeded,
}

/// product operational policy の閉集合です。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProductOperationalPolicyClass {
    /// local / controlled operation として実行可能です。
    ControlledLocal,
    /// maintenance 中で drain / restore / observe だけを許可します。
    MaintenanceDrainOnly,
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
    /// product-owned tenant admissionです。
    pub tenant_class: ProductTenantClass,
    /// product-owned quota policyです。
    pub quota_class: ProductQuotaClass,
    /// product-owned operational policyです。
    pub operational_policy_class: ProductOperationalPolicyClass,
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
    let distro_reason = product_policy_reason(input);
    Ok(ProductPolicyDecision {
        allowed: distro_reason == DistroEvidenceReason::DistroOk,
        distro_reason,
        non_claim_scope,
    })
}

fn product_policy_reason(input: &ProductPolicyInput) -> DistroEvidenceReason {
    if input.fixture_identity.is_none() {
        return DistroEvidenceReason::FixtureIdentityInvalid;
    }
    if input.tenant_class != ProductTenantClass::LocalFixtureTenant {
        return DistroEvidenceReason::DependencyNotAdmitted;
    }
    if input.quota_class != ProductQuotaClass::LocalQuotaAvailable {
        return DistroEvidenceReason::StateBoundaryViolation;
    }
    // operational policy は product 側の実行許可であり、Kernel / reference semantic へ委譲しません。
    if !is_action_allowed_by_operational_policy(
        input.operational_policy_class,
        input.requested_action,
    ) {
        return DistroEvidenceReason::StateBoundaryViolation;
    }
    DistroEvidenceReason::DistroOk
}

const fn is_action_allowed_by_operational_policy(
    policy_class: ProductOperationalPolicyClass,
    action: ProductAction,
) -> bool {
    match policy_class {
        ProductOperationalPolicyClass::ControlledLocal => true,
        ProductOperationalPolicyClass::MaintenanceDrainOnly => matches!(
            action,
            ProductAction::Observe | ProductAction::Drain | ProductAction::Restore
        ),
    }
}
