use std::fs;
use std::path::PathBuf;

use arcrtc_core_identity::{CredentialRef, OpaqueReference, ReferenceAuthority};
use arcrtc_core_security::{
    CredentialRevocationState, CredentialSubjectRef, CredentialVerificationDecision,
    CredentialVerifierFailureKind, CredentialVerifierKeyRef, CredentialVerifierPortInput,
    KeySelectionContext, VerifierProfileRef,
};
use arcrtc_driver_security::FileCredentialVerifierAdapter;

fn reference(value: &str, authority: ReferenceAuthority) -> OpaqueReference {
    OpaqueReference::accept(value, authority).expect("test fixture uses accepted opaque references")
}

fn verifier_input(
    subject: Option<CredentialSubjectRef>,
    key: Option<CredentialVerifierKeyRef>,
    selected: bool,
    revocation_state: CredentialRevocationState,
) -> CredentialVerifierPortInput {
    CredentialVerifierPortInput::new(
        CredentialRef::new(reference(
            "credential:test",
            ReferenceAuthority::DriverCredentialConversion,
        )),
        VerifierProfileRef::new(reference(
            "verifier-profile:test",
            ReferenceAuthority::CorePolicy,
        )),
        KeySelectionContext::new(subject, key, selected),
        revocation_state,
    )
}

fn accepted_subject() -> CredentialSubjectRef {
    CredentialSubjectRef::new(reference("subject:test", ReferenceAuthority::CorePolicy))
}

fn accepted_key() -> CredentialVerifierKeyRef {
    CredentialVerifierKeyRef::new(reference("key:test", ReferenceAuthority::CorePolicy))
}

fn fixture_key_material_path(name: &str, bytes: &[u8]) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "arcrtc-driver-security-{name}-{}",
        std::process::id()
    ));
    fs::write(&path, bytes).expect("test fixture key material must be writable");
    path
}

#[test]
fn credential_verifier_positive_and_negative_paths() {
    assert_accepted_credential();
    assert_missing_key();
    assert_revoked_credential();
    assert_invalid_signature_class();
    assert_expired_credential();
}

fn assert_accepted_credential() {
    let path = fixture_key_material_path("accepted", b"test-key-material");
    let adapter = FileCredentialVerifierAdapter::new(path.clone());
    let output = adapter
        .verify(verifier_input(
            Some(accepted_subject()),
            Some(accepted_key()),
            true,
            CredentialRevocationState::NotRevoked,
        ))
        .expect("selected, non-revoked credential must be accepted");

    assert_eq!(output.decision(), CredentialVerificationDecision::Accepted);
    fs::remove_file(path).ok();
}

fn assert_missing_key() {
    let path = fixture_key_material_path("missing-key", b"test-key-material");
    let adapter = FileCredentialVerifierAdapter::new(path.clone());
    let failure = adapter
        .verify(verifier_input(
            Some(accepted_subject()),
            None,
            true,
            CredentialRevocationState::NotRevoked,
        ))
        .expect_err("missing key reference must fail closed");

    assert_eq!(failure, CredentialVerifierFailureKind::KeyRefMissing);
    fs::remove_file(path).ok();
}

fn assert_revoked_credential() {
    let path = fixture_key_material_path("revoked", b"test-key-material");
    let adapter = FileCredentialVerifierAdapter::new(path.clone());
    let output = adapter
        .verify(verifier_input(
            Some(accepted_subject()),
            Some(accepted_key()),
            true,
            CredentialRevocationState::Revoked,
        ))
        .expect("revoked credential must produce closed rejected decision");

    assert_eq!(output.decision(), CredentialVerificationDecision::Rejected);
    fs::remove_file(path).ok();
}

fn assert_invalid_signature_class() {
    let path = fixture_key_material_path("invalid-signature", b"crypto_backend=failed");
    let adapter = FileCredentialVerifierAdapter::new(path.clone());
    let failure = adapter
        .verify(verifier_input(
            Some(accepted_subject()),
            Some(accepted_key()),
            true,
            CredentialRevocationState::NotRevoked,
        ))
        .expect_err("driver-local crypto backend rejection must fail closed");

    assert_eq!(
        failure,
        CredentialVerifierFailureKind::CryptoBackendRejected
    );
    fs::remove_file(path).ok();
}

fn assert_expired_credential() {
    let path = fixture_key_material_path("expired", b"test-key-material");
    let adapter = FileCredentialVerifierAdapter::new(path.clone());
    let failure = adapter
        .verify(verifier_input(
            Some(accepted_subject()),
            Some(accepted_key()),
            false,
            CredentialRevocationState::NotRevoked,
        ))
        .expect_err("expired/stale key selection is represented as selected=false");

    assert_eq!(failure, CredentialVerifierFailureKind::KeySelectionRejected);
    fs::remove_file(path).ok();
}
