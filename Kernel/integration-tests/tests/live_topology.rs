use arcrtc_entrypoint_endpoints::{
    observe_public_endpoint, EdgeProxyTrustObservationRef, EndpointConnectionObservation,
    PublicEndpointLifecycleObservation, PublicEndpointObservationInput,
    PublicNetworkTraversalClass,
};
use arcrtc_entrypoint_topology::{
    build_topology_wiring, DistributedStateFailoverObservationRef, NodeScopeClass,
    ServiceDiscoveryResolutionObservation, TopologyRollbackCommand, TopologyWiringInput,
};

#[test]
fn public_endpoint_lifecycle_traversal_and_monitoring_are_observed_without_trust_decision() {
    let traversal_cases = [
        PublicNetworkTraversalClass::Direct,
        PublicNetworkTraversalClass::TurnRelay,
        PublicNetworkTraversalClass::EdgeProxy,
    ];

    for traversal_class in traversal_cases {
        let observation = observe_public_endpoint(PublicEndpointObservationInput::new(
            PublicEndpointLifecycleObservation::new("endpoint-ref", "lifecycle-active"),
            traversal_class,
            EndpointConnectionObservation::new("endpoint-ref", "connection-monitored"),
            EdgeProxyTrustObservationRef::new("edge-proxy-trust-observation"),
        ));

        // endpoint entrypoint は観測 surface のみを構成し、trust/domain admission は所有しません。
        assert_eq!(observation.traversal_class, traversal_class);
        assert_eq!(
            observation.lifecycle_observation.lifecycle_state_ref,
            "lifecycle-active"
        );
        assert_eq!(
            observation.connection_observation.connection_state_ref,
            "connection-monitored"
        );
        assert_eq!(
            observation.edge_proxy_trust_observation_ref.value,
            "edge-proxy-trust-observation"
        );
    }
}

#[test]
fn topology_discovery_rollback_and_failover_are_observed_without_failover_decision() {
    let node_scopes = [
        NodeScopeClass::SingleNode,
        NodeScopeClass::ClusterNode,
        NodeScopeClass::EdgeNode,
    ];

    for node_scope_class in node_scopes {
        let observation = build_topology_wiring(TopologyWiringInput::new(
            ServiceDiscoveryResolutionObservation::new("signaling-service", "endpoint-ref"),
            TopologyRollbackCommand::new("topology-ref", "rollback-ref"),
            node_scope_class,
            DistributedStateFailoverObservationRef::new("failover-observation"),
        ));

        assert_eq!(observation.node_scope_class, node_scope_class);
        assert_eq!(
            observation
                .service_discovery_resolution
                .resolved_endpoint_ref,
            "endpoint-ref"
        );
        assert_eq!(observation.rollback_command.rollback_ref, "rollback-ref");
        assert_eq!(
            observation.distributed_state_failover_observation_ref.value,
            "failover-observation"
        );
    }
}
