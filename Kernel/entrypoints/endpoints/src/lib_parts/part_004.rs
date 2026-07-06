/// public network traversal の観測分類です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PublicNetworkTraversalClass {
    /// direct public listener observation.
    Direct,
    /// TURN relay traversal observation.
    TurnRelay,
    /// edge proxy traversal observation.
    EdgeProxy,
}

/// edge proxy trust observation の opaque reference です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeProxyTrustObservationRef {
    /// edge proxy trust observation の opaque value です。
    pub value: &'static str,
}

impl EdgeProxyTrustObservationRef {
    /// edge proxy trust observation ref を作ります。
    pub const fn new(value: &'static str) -> Self {
        Self { value }
    }
}

/// public endpoint lifecycle observation です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PublicEndpointLifecycleObservation {
    /// endpoint reference です。
    pub endpoint_ref: &'static str,
    /// lifecycle state reference です。
    pub lifecycle_state_ref: &'static str,
}

impl PublicEndpointLifecycleObservation {
    /// endpoint lifecycle observation を束ねます。
    pub const fn new(endpoint_ref: &'static str, lifecycle_state_ref: &'static str) -> Self {
        Self {
            endpoint_ref,
            lifecycle_state_ref,
        }
    }
}

/// endpoint connection observation です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EndpointConnectionObservation {
    /// endpoint reference です。
    pub endpoint_ref: &'static str,
    /// connection state reference です。
    pub connection_state_ref: &'static str,
}

impl EndpointConnectionObservation {
    /// endpoint connection observation を束ねます。
    pub const fn new(endpoint_ref: &'static str, connection_state_ref: &'static str) -> Self {
        Self {
            endpoint_ref,
            connection_state_ref,
        }
    }
}

/// public endpoint observation input です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PublicEndpointObservationInput {
    /// lifecycle observation です。
    pub lifecycle_observation: PublicEndpointLifecycleObservation,
    /// network traversal class です。
    pub traversal_class: PublicNetworkTraversalClass,
    /// connection observation です。
    pub connection_observation: EndpointConnectionObservation,
    /// edge proxy trust observation ref です。
    pub edge_proxy_trust_observation_ref: EdgeProxyTrustObservationRef,
}

impl PublicEndpointObservationInput {
    /// endpoint observation input を束ねます。
    pub const fn new(
        lifecycle_observation: PublicEndpointLifecycleObservation,
        traversal_class: PublicNetworkTraversalClass,
        connection_observation: EndpointConnectionObservation,
        edge_proxy_trust_observation_ref: EdgeProxyTrustObservationRef,
    ) -> Self {
        Self {
            lifecycle_observation,
            traversal_class,
            connection_observation,
            edge_proxy_trust_observation_ref,
        }
    }
}

/// public endpoint observation surface です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PublicEndpointObservation {
    /// lifecycle observation です。
    pub lifecycle_observation: PublicEndpointLifecycleObservation,
    /// network traversal class です。
    pub traversal_class: PublicNetworkTraversalClass,
    /// connection observation です。
    pub connection_observation: EndpointConnectionObservation,
    /// edge proxy trust observation ref です。
    pub edge_proxy_trust_observation_ref: EdgeProxyTrustObservationRef,
}

/// public endpoint を observation として構成します。
///
/// trust decision、domain admission、live readiness proof は生成しません。
pub const fn observe_public_endpoint(
    input: PublicEndpointObservationInput,
) -> PublicEndpointObservation {
    PublicEndpointObservation {
        lifecycle_observation: input.lifecycle_observation,
        traversal_class: input.traversal_class,
        connection_observation: input.connection_observation,
        edge_proxy_trust_observation_ref: input.edge_proxy_trust_observation_ref,
    }
}
