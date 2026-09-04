use arcrtc_core_identity::{OpaqueReference, ReferenceAuthority};
use arcrtc_core_security::{
    select_key_for_verification, CredentialVerifierKeyRef, KeyRevocationListRef,
    KeyRotationOverlapPolicy, KeyRotationWindow, KeySelectionDecision, KeySelectionEvaluation,
    KeySelectionVerificationContext, RotationInstantRef, SecretRotationDecisionRef,
};

fn reference(value: &str) -> OpaqueReference {
    OpaqueReference::accept(value, ReferenceAuthority::CorePolicy)
        .expect("test fixture uses accepted opaque references")
}

fn key_ref() -> CredentialVerifierKeyRef {
    CredentialVerifierKeyRef::new(reference("key:active"))
}

fn rotation_window(policy: KeyRotationOverlapPolicy) -> KeyRotationWindow {
    KeyRotationWindow::new(
        RotationInstantRef::new(reference("rotation:from")),
        RotationInstantRef::new(reference("rotation:until")),
        policy,
    )
}

fn selection_context(
    key: Option<CredentialVerifierKeyRef>,
    policy: KeyRotationOverlapPolicy,
    evaluation: KeySelectionEvaluation,
) -> KeySelectionVerificationContext {
    KeySelectionVerificationContext::new(
        key,
        rotation_window(policy),
        KeyRevocationListRef::new(reference("revocation-list:active")),
        SecretRotationDecisionRef::new(reference("rotation-decision:active")),
        evaluation,
    )
}

#[test]
fn rotation_revocation_key_selection_is_deterministic() {
    assert_active_key();
    assert_overlap_window();
    assert_revoked_key();
    assert_stale_key();
    assert_rotation_failure();
}

fn assert_active_key() {
    let decision = select_key_for_verification(selection_context(
        Some(key_ref()),
        KeyRotationOverlapPolicy::NoOverlap,
        KeySelectionEvaluation::new(false, false, false, false),
    ));

    assert_eq!(decision, KeySelectionDecision::Selected);
}

fn assert_overlap_window() {
    let context = selection_context(
        Some(key_ref()),
        KeyRotationOverlapPolicy::BoundedOverlap,
        KeySelectionEvaluation::new(false, false, false, false),
    );

    assert_eq!(
        context.rotation_window().overlap_policy(),
        KeyRotationOverlapPolicy::BoundedOverlap
    );
    assert_eq!(
        select_key_for_verification(context),
        KeySelectionDecision::Selected
    );
}

fn assert_revoked_key() {
    let decision = select_key_for_verification(selection_context(
        Some(key_ref()),
        KeyRotationOverlapPolicy::NoOverlap,
        KeySelectionEvaluation::new(false, false, true, false),
    ));

    assert_eq!(decision, KeySelectionDecision::Revoked);
}

fn assert_stale_key() {
    let decision = select_key_for_verification(selection_context(
        Some(key_ref()),
        KeyRotationOverlapPolicy::NoOverlap,
        KeySelectionEvaluation::new(false, true, false, false),
    ));

    assert_eq!(decision, KeySelectionDecision::Expired);
}

fn assert_rotation_failure() {
    let decision = select_key_for_verification(selection_context(
        Some(key_ref()),
        KeyRotationOverlapPolicy::NoOverlap,
        KeySelectionEvaluation::new(false, false, false, true),
    ));

    assert_eq!(decision, KeySelectionDecision::Failed);
}
