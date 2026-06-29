//! product policy と readiness non-claim 境界を検査します。

use std::{fs, path::PathBuf};

use arcrtc_core_identity::{CorrelationId, OpaqueReference, ReferenceAuthority};
use arcrtc_implementation_evidence::{
    ImplementationEvidenceReason, ImplementationNonClaimScope, ImplementationPlane,
};
use arcrtc_product_policy::{
    evaluate_product_auth_policy, ProductAction, ProductPolicyError, ProductPolicyInput,
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
        .expect("implementations root must exist")
}

fn input(identity: Option<&str>) -> ProductPolicyInput {
    ProductPolicyInput {
        correlation_id: cid("product-policy"),
        target_plane: ImplementationPlane::Signaling,
        fixture_identity: identity.map(str::to_owned),
        requested_action: ProductAction::Join,
    }
}

#[test]
fn product_policy_has_explicit_non_claim_scope_and_identity_fail_closed_branch() {
    let allowed = evaluate_product_auth_policy(&input(Some("fixture-user")))
        .expect("fixture identity must be accepted");
    assert!(allowed.allowed);
    assert_eq!(
        allowed.implementation_reason,
        ImplementationEvidenceReason::ImplementationOk
    );
    assert!(allowed
        .non_claim_scope
        .contains(&ImplementationNonClaimScope::ProductionReadinessNotClaimed));
    assert!(allowed
        .non_claim_scope
        .contains(&ImplementationNonClaimScope::LiveReadinessNotClaimed));

    let denied = evaluate_product_auth_policy(&input(None)).expect("missing identity is denied");
    assert!(!denied.allowed);
    assert_eq!(
        denied.implementation_reason,
        ImplementationEvidenceReason::FixtureIdentityInvalid
    );

    assert_eq!(
        evaluate_product_auth_policy(&input(Some("   "))),
        Err(ProductPolicyError::InvalidFixtureIdentity)
    );
}

#[test]
fn kpi_product_policy_executes_without_kernel_semantic_ownership() {
    let source =
        fs::read_to_string(root().join("product-implementation/product-policy/src/auth_policy.rs"))
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
        target_plane: ImplementationPlane::Sfu,
        fixture_identity: Some("fixture-user".to_owned()),
        requested_action: ProductAction::Subscribe,
    })
    .expect("product authorization decision must be returned");

    assert!(decision.allowed);
    assert_eq!(
        decision.implementation_reason,
        ImplementationEvidenceReason::ImplementationOk
    );
    assert_eq!(
        decision.non_claim_scope,
        vec![
            ImplementationNonClaimScope::ProductionReadinessNotClaimed,
            ImplementationNonClaimScope::LiveReadinessNotClaimed,
        ]
    );
}
