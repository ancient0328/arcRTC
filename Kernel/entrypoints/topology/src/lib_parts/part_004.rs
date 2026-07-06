/// topology wiring の node scope class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeScopeClass {
    /// single node observation.
    SingleNode,
    /// cluster node observation.
    ClusterNode,
    /// edge node observation.
    EdgeNode,
}

/// service discovery resolution observation です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ServiceDiscoveryResolutionObservation {
    /// service reference です。
    pub service_ref: &'static str,
    /// resolved endpoint reference です。
    pub resolved_endpoint_ref: &'static str,
}

impl ServiceDiscoveryResolutionObservation {
    /// service discovery resolution observation を束ねます。
    pub const fn new(service_ref: &'static str, resolved_endpoint_ref: &'static str) -> Self {
        Self {
            service_ref,
            resolved_endpoint_ref,
        }
    }
}

/// topology rollback command surface です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TopologyRollbackCommand {
    /// topology reference です。
    pub topology_ref: &'static str,
    /// rollback reference です。
    pub rollback_ref: &'static str,
}

impl TopologyRollbackCommand {
    /// topology rollback command を束ねます。
    pub const fn new(topology_ref: &'static str, rollback_ref: &'static str) -> Self {
        Self {
            topology_ref,
            rollback_ref,
        }
    }
}

/// distributed state failover observation の opaque reference です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DistributedStateFailoverObservationRef {
    /// failover observation の opaque value です。
    pub value: &'static str,
}

impl DistributedStateFailoverObservationRef {
    /// distributed state failover observation ref を作ります。
    pub const fn new(value: &'static str) -> Self {
        Self { value }
    }
}

/// topology wiring input です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TopologyWiringInput {
    /// service discovery resolution observation です。
    pub service_discovery_resolution: ServiceDiscoveryResolutionObservation,
    /// topology rollback command surface です。
    pub rollback_command: TopologyRollbackCommand,
    /// node scope class です。
    pub node_scope_class: NodeScopeClass,
    /// distributed state failover observation ref です。
    pub distributed_state_failover_observation_ref: DistributedStateFailoverObservationRef,
}

impl TopologyWiringInput {
    /// topology wiring input を束ねます。
    pub const fn new(
        service_discovery_resolution: ServiceDiscoveryResolutionObservation,
        rollback_command: TopologyRollbackCommand,
        node_scope_class: NodeScopeClass,
        distributed_state_failover_observation_ref: DistributedStateFailoverObservationRef,
    ) -> Self {
        Self {
            service_discovery_resolution,
            rollback_command,
            node_scope_class,
            distributed_state_failover_observation_ref,
        }
    }
}

/// topology wiring observation です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TopologyWiringObservation {
    /// service discovery resolution observation です。
    pub service_discovery_resolution: ServiceDiscoveryResolutionObservation,
    /// topology rollback command surface です。
    pub rollback_command: TopologyRollbackCommand,
    /// node scope class です。
    pub node_scope_class: NodeScopeClass,
    /// distributed state failover observation ref です。
    pub distributed_state_failover_observation_ref: DistributedStateFailoverObservationRef,
}

/// topology wiring を observation として構成します。
///
/// service discovery authority、domain authority、distributed failover decision は生成しません。
pub const fn build_topology_wiring(input: TopologyWiringInput) -> TopologyWiringObservation {
    TopologyWiringObservation {
        service_discovery_resolution: input.service_discovery_resolution,
        rollback_command: input.rollback_command,
        node_scope_class: input.node_scope_class,
        distributed_state_failover_observation_ref: input.distributed_state_failover_observation_ref,
    }
}
