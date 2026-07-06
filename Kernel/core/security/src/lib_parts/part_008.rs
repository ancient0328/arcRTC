/// trust policy が扱う boundary class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrustBoundaryClass {
    /// process boundary.
    Process,
    /// network boundary.
    Network,
    /// storage boundary.
    Storage,
    /// release artifact boundary.
    ReleaseArtifact,
}

/// release/supply-chain policy の opaque reference です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SupplyChainPolicyRef {
    /// supply-chain policy source を指す opaque value です。
    pub value: &'static str,
}

impl SupplyChainPolicyRef {
    /// supply-chain policy ref を作ります。
    pub const fn new(value: &'static str) -> Self {
        Self { value }
    }
}

/// crypto key policy の source model です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CryptoKeyPolicy {
    /// algorithm reference です。
    pub algorithm_ref: &'static str,
    /// rotation window reference です。
    pub rotation_window_ref: &'static str,
    /// revocation reference です。
    pub revocation_ref: &'static str,
}

impl CryptoKeyPolicy {
    /// key policy reference 群を束ねます。
    pub const fn new(
        algorithm_ref: &'static str,
        rotation_window_ref: &'static str,
        revocation_ref: &'static str,
    ) -> Self {
        Self {
            algorithm_ref,
            rotation_window_ref,
            revocation_ref,
        }
    }
}

/// trust policy input です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TrustPolicyInput {
    /// trust boundary class です。
    pub boundary_class: TrustBoundaryClass,
    /// crypto key policy reference 群です。
    pub crypto_key_policy: CryptoKeyPolicy,
    /// supply-chain policy ref です。
    pub supply_chain_policy_ref: SupplyChainPolicyRef,
}

impl TrustPolicyInput {
    /// trust policy input を束ねます。
    pub const fn new(
        boundary_class: TrustBoundaryClass,
        crypto_key_policy: CryptoKeyPolicy,
        supply_chain_policy_ref: SupplyChainPolicyRef,
    ) -> Self {
        Self {
            boundary_class,
            crypto_key_policy,
            supply_chain_policy_ref,
        }
    }
}

/// trust policy decision の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrustPolicyDecision {
    /// trust policy input を受理します。
    Accepted,
    /// trust policy input を拒否します。
    Rejected,
}

/// trust policy input を closed decision へ写像します。
///
/// dependency/license/vulnerability/SBOM/signing/provenance policy semantics はここでは評価しません。
pub fn decide_trust_policy(input: TrustPolicyInput) -> TrustPolicyDecision {
    let key_refs_present = !input.crypto_key_policy.algorithm_ref.is_empty()
        && !input.crypto_key_policy.rotation_window_ref.is_empty()
        && !input.crypto_key_policy.revocation_ref.is_empty();
    let supply_chain_ref_present = !input.supply_chain_policy_ref.value.is_empty();

    if key_refs_present && supply_chain_ref_present {
        TrustPolicyDecision::Accepted
    } else {
        TrustPolicyDecision::Rejected
    }
}
