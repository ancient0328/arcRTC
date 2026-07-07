use arcrtc_core_security::{
    decide_internal_service_identity, decide_secure_media_peer_verification, decide_trust_policy,
    CryptoKeyPolicy, InternalServiceIdentityDecision, InternalServiceIdentityInput,
    MutualTrustProofClass, SecureMediaPeerVerificationDecision, SecureMediaPeerVerificationInput,
    SupplyChainPolicyRef, TrustBoundaryClass, TrustPolicyDecision, TrustPolicyInput,
};

const SUPPLY_CHAIN_POLICY: &str = include_str!("../../scripts/release/supply-chain-policy.toml");

fn accepted_key_policy() -> CryptoKeyPolicy {
    CryptoKeyPolicy::new("algorithm-ref", "rotation-window-ref", "revocation-ref")
}

fn supply_chain_policy_ref() -> SupplyChainPolicyRef {
    SupplyChainPolicyRef::new("scripts/release/supply-chain-policy.toml")
}

#[test]
fn trust_policy_accepts_all_declared_trust_boundaries_with_key_and_supply_chain_refs() {
    for boundary_class in [
        TrustBoundaryClass::Process,
        TrustBoundaryClass::Network,
        TrustBoundaryClass::Storage,
        TrustBoundaryClass::ReleaseArtifact,
    ] {
        // core/security は policy ref を判定対象にし、release policy semantics は scripts/release に残します。
        assert_eq!(
            decide_trust_policy(TrustPolicyInput::new(
                boundary_class,
                accepted_key_policy(),
                supply_chain_policy_ref(),
            )),
            TrustPolicyDecision::Accepted
        );
    }
}

#[test]
fn trust_policy_rejects_missing_crypto_key_or_supply_chain_policy_refs() {
    let missing_algorithm = TrustPolicyInput::new(
        TrustBoundaryClass::Network,
        CryptoKeyPolicy::new("", "rotation-window-ref", "revocation-ref"),
        supply_chain_policy_ref(),
    );
    let missing_rotation = TrustPolicyInput::new(
        TrustBoundaryClass::Storage,
        CryptoKeyPolicy::new("algorithm-ref", "", "revocation-ref"),
        supply_chain_policy_ref(),
    );
    let missing_supply_chain_ref = TrustPolicyInput::new(
        TrustBoundaryClass::ReleaseArtifact,
        accepted_key_policy(),
        SupplyChainPolicyRef::new(""),
    );

    for input in [
        missing_algorithm,
        missing_rotation,
        missing_supply_chain_ref,
    ] {
        assert_eq!(decide_trust_policy(input), TrustPolicyDecision::Rejected);
    }
}

#[test]
fn mutual_trust_proofs_gate_internal_identity_and_secure_media_peer_verification() {
    for proof_class in [
        MutualTrustProofClass::Mtls,
        MutualTrustProofClass::SignedAssertion,
        MutualTrustProofClass::PinnedKey,
    ] {
        assert!(!proof_class.as_str().is_empty());
        assert_eq!(
            decide_internal_service_identity(InternalServiceIdentityInput::new(
                "internal-service",
                proof_class,
                "trust-policy-ref",
            )),
            InternalServiceIdentityDecision::Accepted
        );
        assert_eq!(
            decide_secure_media_peer_verification(SecureMediaPeerVerificationInput::new(
                "secure-media-peer",
                proof_class,
                "trust-policy-ref",
            )),
            SecureMediaPeerVerificationDecision::Accepted
        );
    }

    assert_eq!(
        decide_internal_service_identity(InternalServiceIdentityInput::new(
            "",
            MutualTrustProofClass::Mtls,
            "trust-policy-ref",
        )),
        InternalServiceIdentityDecision::Rejected
    );
    assert_eq!(
        decide_secure_media_peer_verification(SecureMediaPeerVerificationInput::new(
            "secure-media-peer",
            MutualTrustProofClass::PinnedKey,
            "",
        )),
        SecureMediaPeerVerificationDecision::Rejected
    );
}

#[test]
fn supply_chain_hardening_policy_source_remains_present_but_not_readiness_evidence() {
    for section in [
        "[dependency_admission]",
        "[license_admission]",
        "[vulnerability_gate]",
        "[sbom]",
        "[artifact_signature]",
        "[provenance]",
    ] {
        assert!(SUPPLY_CHAIN_POLICY.contains(section), "{section}");
    }
    assert!(SUPPLY_CHAIN_POLICY.contains("release_blocking_severities = [\"critical\", \"high\"]"));
    assert!(SUPPLY_CHAIN_POLICY.contains("signature_ref_required = true"));
    assert!(SUPPLY_CHAIN_POLICY
        .contains("non_adoption = \"vulnerability gate policy is not scanner output\""));
}
