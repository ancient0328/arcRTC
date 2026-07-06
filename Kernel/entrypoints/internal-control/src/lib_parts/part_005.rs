/// internal service identity observation の opaque reference です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InternalServiceIdentityObservationRef {
    /// identity observation の opaque value です。
    pub value: &'static str,
}

impl InternalServiceIdentityObservationRef {
    /// internal service identity observation ref を作ります。
    pub const fn new(value: &'static str) -> Self {
        Self { value }
    }
}

/// internal control wiring の opaque reference です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InternalControlWiringRef {
    /// wiring observation の opaque value です。
    pub value: &'static str,
}

impl InternalControlWiringRef {
    /// internal control wiring ref を作ります。
    pub const fn new(value: &'static str) -> Self {
        Self { value }
    }
}

/// mutual trust proof observation の opaque reference です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MutualTrustProofObservationRef {
    /// mutual trust proof observation の opaque value です。
    pub value: &'static str,
}

impl MutualTrustProofObservationRef {
    /// mutual trust proof observation ref を作ります。
    pub const fn new(value: &'static str) -> Self {
        Self { value }
    }
}

/// internal control wiring observation input です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InternalControlWiringObservationInput {
    /// identity observation ref です。
    pub identity_observation_ref: InternalServiceIdentityObservationRef,
    /// internal control wiring ref です。
    pub wiring_ref: InternalControlWiringRef,
    /// mutual trust proof observation ref です。
    pub mutual_trust_proof_observation_ref: MutualTrustProofObservationRef,
}

impl InternalControlWiringObservationInput {
    /// internal control wiring observation input を束ねます。
    pub const fn new(
        identity_observation_ref: InternalServiceIdentityObservationRef,
        wiring_ref: InternalControlWiringRef,
        mutual_trust_proof_observation_ref: MutualTrustProofObservationRef,
    ) -> Self {
        Self {
            identity_observation_ref,
            wiring_ref,
            mutual_trust_proof_observation_ref,
        }
    }
}

/// internal control wiring observation です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InternalControlWiringObservation {
    /// identity observation ref です。
    pub identity_observation_ref: InternalServiceIdentityObservationRef,
    /// internal control wiring ref です。
    pub wiring_ref: InternalControlWiringRef,
    /// mutual trust proof observation ref です。
    pub mutual_trust_proof_observation_ref: MutualTrustProofObservationRef,
}

/// internal control wiring を observation として返します。
///
/// identity decision、trust decision、target domain authorization は生成しません。
pub const fn observe_internal_control_wiring(
    input: InternalControlWiringObservationInput,
) -> InternalControlWiringObservation {
    InternalControlWiringObservation {
        identity_observation_ref: input.identity_observation_ref,
        wiring_ref: input.wiring_ref,
        mutual_trust_proof_observation_ref: input.mutual_trust_proof_observation_ref,
    }
}
