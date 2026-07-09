//! product policy と readiness non-claim 境界を検査します。

use std::{fs, path::PathBuf};

use arcrtc_core_identity::{CorrelationId, OpaqueReference, ReferenceAuthority};
use arcrtc_distro_evidence::{DistroEvidenceReason, DistroNonClaimScope, DistroPlane};
use arcrtc_product_policy::{
    evaluate_product_auth_policy, ProductAction, ProductOperationalPolicyClass, ProductPolicyError,
    ProductPolicyInput, ProductQuotaClass, ProductTenantClass,
};

fn accepted(value: &str) -> OpaqueReference {
    OpaqueReference::accept(value, ReferenceAuthority::CoreValidatedUntrustedInput)
        .expect("test reference must be accepted")
}

fn cid(value: &str) -> CorrelationId {
    CorrelationId::new(accepted(value))
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("distro root must exist")
}

fn input(identity: Option<&str>) -> ProductPolicyInput {
    ProductPolicyInput {
        correlation_id: cid("product-policy"),
        target_plane: DistroPlane::Signaling,
        fixture_identity: identity.map(str::to_owned),
        tenant_class: ProductTenantClass::LocalFixtureTenant,
        quota_class: ProductQuotaClass::LocalQuotaAvailable,
        operational_policy_class: ProductOperationalPolicyClass::ControlledLocal,
        requested_action: ProductAction::Join,
    }
}

#[test]
fn product_policy_has_explicit_non_claim_scope_and_identity_fail_closed_branch() {
    let allowed = evaluate_product_auth_policy(&input(Some("fixture-user")))
        .expect("fixture identity must be accepted");
    assert!(allowed.allowed);
    assert_eq!(allowed.distro_reason, DistroEvidenceReason::DistroOk);
    assert!(allowed
        .non_claim_scope
        .contains(&DistroNonClaimScope::ProductionReadinessNotClaimed));
    assert!(allowed
        .non_claim_scope
        .contains(&DistroNonClaimScope::LiveReadinessNotClaimed));

    let denied = evaluate_product_auth_policy(&input(None)).expect("missing identity is denied");
    assert!(!denied.allowed);
    assert_eq!(
        denied.distro_reason,
        DistroEvidenceReason::FixtureIdentityInvalid
    );

    assert_eq!(
        evaluate_product_auth_policy(&input(Some("   "))),
        Err(ProductPolicyError::InvalidFixtureIdentity)
    );
}

#[test]
fn kpi_product_policy_executes_without_kernel_semantic_ownership() {
    let source =
        fs::read_to_string(root().join("product-distro/product-policy/src/auth_policy.rs"))
            .expect("product policy source must be readable");
    for forbidden in [
        "arcrtc_core_signaling",
        "arcrtc_core_turn",
        "arcrtc_core_sfu",
        "SignalingEventKind",
        "TurnDecisionKind",
        "SfuDecisionKind",
        "ReferenceSignalingOutcome",
        "ReferenceTurnOutcome",
        "ReferenceSfuOutcome",
        "ReferenceCompositionOutcome",
        "CommandEnvelope",
    ] {
        assert!(
            !source.contains(forbidden),
            "product policy must not own Kernel communication semantic: {forbidden}"
        );
    }

    let decision = evaluate_product_auth_policy(&ProductPolicyInput {
        correlation_id: cid("product-policy-kpi"),
        target_plane: DistroPlane::Sfu,
        fixture_identity: Some("fixture-user".to_owned()),
        tenant_class: ProductTenantClass::LocalFixtureTenant,
        quota_class: ProductQuotaClass::LocalQuotaAvailable,
        operational_policy_class: ProductOperationalPolicyClass::ControlledLocal,
        requested_action: ProductAction::Subscribe,
    })
    .expect("product authorization decision must be returned");

    assert!(decision.allowed);
    assert_eq!(decision.distro_reason, DistroEvidenceReason::DistroOk);
    assert_eq!(
        decision.non_claim_scope,
        vec![
            DistroNonClaimScope::ProductionReadinessNotClaimed,
            DistroNonClaimScope::LiveReadinessNotClaimed,
        ]
    );
}

#[test]
fn product_policy_owns_tenant_quota_and_operational_decisions() {
    let tenant_denied = evaluate_product_auth_policy(&ProductPolicyInput {
        tenant_class: ProductTenantClass::TenantNotAdmitted,
        ..input(Some("fixture-user"))
    })
    .expect("tenant decision must return product policy decision");
    assert!(!tenant_denied.allowed);
    assert_eq!(
        tenant_denied.distro_reason,
        DistroEvidenceReason::DependencyNotAdmitted
    );

    let quota_denied = evaluate_product_auth_policy(&ProductPolicyInput {
        quota_class: ProductQuotaClass::QuotaExceeded,
        ..input(Some("fixture-user"))
    })
    .expect("quota decision must return product policy decision");
    assert!(!quota_denied.allowed);
    assert_eq!(
        quota_denied.distro_reason,
        DistroEvidenceReason::StateBoundaryViolation
    );

    let operation_denied = evaluate_product_auth_policy(&ProductPolicyInput {
        operational_policy_class: ProductOperationalPolicyClass::MaintenanceDrainOnly,
        requested_action: ProductAction::Join,
        ..input(Some("fixture-user"))
    })
    .expect("operational decision must return product policy decision");
    assert!(!operation_denied.allowed);
    assert_eq!(
        operation_denied.distro_reason,
        DistroEvidenceReason::StateBoundaryViolation
    );

    let drain_allowed = evaluate_product_auth_policy(&ProductPolicyInput {
        operational_policy_class: ProductOperationalPolicyClass::MaintenanceDrainOnly,
        requested_action: ProductAction::Drain,
        ..input(Some("fixture-user"))
    })
    .expect("maintenance drain action must be accepted");
    assert!(drain_allowed.allowed);
    assert_eq!(drain_allowed.distro_reason, DistroEvidenceReason::DistroOk);
}
