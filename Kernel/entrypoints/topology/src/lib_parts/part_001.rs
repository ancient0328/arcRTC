// entrypoints/topology は deployment topology と service discovery wiring の surface です。
//
// topology によって core semantics を変えず、service discovery や endpoint
// resolution の接続面だけを後続 task で配置します。

use arcrtc_core_reason::CatalogedReasonRef;

/// entrypoint topology package の所有境界を示す marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntrypointTopologySurface;

/// v0.2 initial architecture の topology class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeploymentTopologyClass {
    /// one executable composes selected planes.
    SingleProcessLocal,
    /// separate Signaling/SFU/TURN processes on same host.
    SplitPlaneSameHost,
    /// planes communicate over network.
    SplitPlaneNetworked,
    /// multiple nodes for one plane.
    MultiNodeExperimental,
    /// external service supplies dependency.
    ExternalManagedDependency,
}

impl DeploymentTopologyClass {
    /// service endpoint wiring が必須になる topology です。
    pub const fn requires_explicit_service_endpoint_wiring(self) -> bool {
        matches!(self, Self::SplitPlaneSameHost | Self::SplitPlaneNetworked)
    }

    /// networked internal service relation が必須になる topology です。
    pub const fn requires_networked_internal_service_relation(self) -> bool {
        matches!(self, Self::SplitPlaneNetworked)
    }

    /// production claim に ADR/evidence が必須になる experimental topology です。
    pub const fn requires_experimental_admission_for_production_claim(self) -> bool {
        matches!(self, Self::MultiNodeExperimental)
    }

    /// driver contract と health/readiness evidence が必須になる dependency です。
    pub const fn requires_external_dependency_contract(self) -> bool {
        matches!(self, Self::ExternalManagedDependency)
    }
}

/// node-local state class の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeLocalStateClass {
    /// Signaling room/session state.
    RoomState,
    /// SFU endpoint/route state.
    SfuEndpointRouteState,
    /// TURN allocation/permission state.
    TurnAllocationPermissionState,
    /// packet cache.
    PacketCache,
    /// runtime queue/mailbox state.
    RuntimeQueue,
    /// SDK client-local state.
    SdkLocalState,
}

/// topology audit event type の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeploymentTopologyAuditEventType {
    /// deployment topology decision.
    DeploymentTopologyDecision,
}

impl DeploymentTopologyAuditEventType {
    /// audit event catalog に接続する event type です。
    pub const fn event_type(self) -> &'static str {
        match self {
            Self::DeploymentTopologyDecision => "deployment_topology_decision",
        }
    }
}

/// topology class admission guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeploymentTopologyAdmissionGuard {
    topology_class: DeploymentTopologyClass,
    topology_class_declared: bool,
    audit_shape_declared: bool,
    service_composition_declared: bool,
    topology_does_not_change_core_semantics: bool,
    selected_driver_wiring_declared: bool,
    explicit_service_endpoint_wiring_declared_when_required: bool,
    service_endpoint_failure_mapping_declared_when_networked: bool,
    internal_service_trust_relation_declared_when_networked: bool,
    edge_proxy_policy_declared_when_ingress_metadata_affects_path: bool,
    distributed_state_policy_declared_when_node_local_state_can_move: bool,
    external_dependency_contract_and_health_declared_when_required: bool,
    experimental_adr_and_evidence_present_before_production_claim: bool,
    topology_policy_input_is_typed_configuration: bool,
    local_dev_topology_not_used_as_production_topology: bool,
}

/// topology admission の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeploymentTopologyAdmissionError {
    /// topology class がありません。
    TopologyClassMissing,
    /// topology audit shape がありません。
    AuditShapeMissing,
    /// service composition がありません。
    ServiceCompositionMissing,
    /// topology により core semantics を変更しています。
    TopologyChangesCoreSemantics,
    /// selected driver wiring がありません。
    SelectedDriverWiringMissing,
    /// 必要な service endpoint wiring がありません。
    ExplicitServiceEndpointWiringMissing,
    /// networked topology の service endpoint failure mapping がありません。
    ServiceEndpointFailureMappingMissing,
    /// networked topology の internal service trust relation がありません。
    InternalServiceTrustRelationMissing,
    /// ingress metadata 影響時の edge/proxy policy がありません。
    EdgeProxyPolicyMissing,
    /// node-local state 移動時の distributed state policy がありません。
    DistributedStatePolicyMissing,
    /// external dependency の driver contract / health evidence がありません。
    ExternalDependencyContractHealthMissing,
    /// multi-node experimental の production claim 前 ADR/evidence がありません。
    ExperimentalAdmissionMissing,
    /// topology policy input が typed configuration ではありません。
    TopologyPolicyInputNotTyped,
    /// local dev topology を production topology として扱っています。
    LocalDevTopologyUsedAsProduction,
}

impl DeploymentTopologyAdmissionGuard {
    /// topology selection が core semantics を暗黙変更しないことを検査します。
    pub const fn try_new(
        topology_class: DeploymentTopologyClass,
        topology_class_declared: bool,
        audit_shape_declared: bool,
        service_composition_declared: bool,
        topology_does_not_change_core_semantics: bool,
        selected_driver_wiring_declared: bool,
        explicit_service_endpoint_wiring_declared_when_required: bool,
        service_endpoint_failure_mapping_declared_when_networked: bool,
        internal_service_trust_relation_declared_when_networked: bool,
        edge_proxy_policy_declared_when_ingress_metadata_affects_path: bool,
        distributed_state_policy_declared_when_node_local_state_can_move: bool,
        external_dependency_contract_and_health_declared_when_required: bool,
        experimental_adr_and_evidence_present_before_production_claim: bool,
        topology_policy_input_is_typed_configuration: bool,
        local_dev_topology_not_used_as_production_topology: bool,
    ) -> Result<Self, DeploymentTopologyAdmissionError> {
        if !topology_class_declared {
            return Err(DeploymentTopologyAdmissionError::TopologyClassMissing);
        }
        if !audit_shape_declared {
            return Err(DeploymentTopologyAdmissionError::AuditShapeMissing);
        }
        if !service_composition_declared {
            return Err(DeploymentTopologyAdmissionError::ServiceCompositionMissing);
        }
        if !topology_does_not_change_core_semantics {
            return Err(DeploymentTopologyAdmissionError::TopologyChangesCoreSemantics);
        }
        if !selected_driver_wiring_declared {
            return Err(DeploymentTopologyAdmissionError::SelectedDriverWiringMissing);
        }
        if topology_class.requires_explicit_service_endpoint_wiring()
            && !explicit_service_endpoint_wiring_declared_when_required
        {
            return Err(DeploymentTopologyAdmissionError::ExplicitServiceEndpointWiringMissing);
        }
        if topology_class.requires_networked_internal_service_relation()
            && !service_endpoint_failure_mapping_declared_when_networked
        {
            return Err(DeploymentTopologyAdmissionError::ServiceEndpointFailureMappingMissing);
        }
        if topology_class.requires_networked_internal_service_relation()
            && !internal_service_trust_relation_declared_when_networked
        {
            return Err(DeploymentTopologyAdmissionError::InternalServiceTrustRelationMissing);
        }
        if !edge_proxy_policy_declared_when_ingress_metadata_affects_path {
            return Err(DeploymentTopologyAdmissionError::EdgeProxyPolicyMissing);
        }
        if !distributed_state_policy_declared_when_node_local_state_can_move {
            return Err(DeploymentTopologyAdmissionError::DistributedStatePolicyMissing);
        }
        if topology_class.requires_external_dependency_contract()
            && !external_dependency_contract_and_health_declared_when_required
        {
            return Err(DeploymentTopologyAdmissionError::ExternalDependencyContractHealthMissing);
        }
        if topology_class.requires_experimental_admission_for_production_claim()
            && !experimental_adr_and_evidence_present_before_production_claim
        {
            return Err(DeploymentTopologyAdmissionError::ExperimentalAdmissionMissing);
        }
        if !topology_policy_input_is_typed_configuration {
            return Err(DeploymentTopologyAdmissionError::TopologyPolicyInputNotTyped);
        }
        if !local_dev_topology_not_used_as_production_topology {
            return Err(DeploymentTopologyAdmissionError::LocalDevTopologyUsedAsProduction);
        }

        Ok(Self {
            topology_class,
            topology_class_declared,
            audit_shape_declared,
            service_composition_declared,
            topology_does_not_change_core_semantics,
            selected_driver_wiring_declared,
            explicit_service_endpoint_wiring_declared_when_required,
            service_endpoint_failure_mapping_declared_when_networked,
            internal_service_trust_relation_declared_when_networked,
            edge_proxy_policy_declared_when_ingress_metadata_affects_path,
            distributed_state_policy_declared_when_node_local_state_can_move,
            external_dependency_contract_and_health_declared_when_required,
            experimental_adr_and_evidence_present_before_production_claim,
            topology_policy_input_is_typed_configuration,
            local_dev_topology_not_used_as_production_topology,
        })
    }
}

/// deployment topology audit shape guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeploymentTopologyAuditGuard {
    event_type: DeploymentTopologyAuditEventType,
    topology_class_declared: bool,
    entrypoint_service_reference_declared: bool,
    startup_run_id_declared: bool,
    correlation_rule_declared_when_command_scoped: bool,
}

/// deployment topology audit shape の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeploymentTopologyAuditError {
    /// topology class がありません。
    TopologyClassMissing,
    /// entrypoint/service reference がありません。
    EntrypointServiceReferenceMissing,
    /// StartupRunId がありません。
    StartupRunIdMissing,
    /// command-scoped decision の CorrelationId rule がありません。
    CorrelationRuleMissing,
}

impl DeploymentTopologyAuditGuard {
    /// deployment_topology_decision audit event の必須 field を検査します。
    pub const fn try_new(
        event_type: DeploymentTopologyAuditEventType,
        topology_class_declared: bool,
        entrypoint_service_reference_declared: bool,
        startup_run_id_declared: bool,
        correlation_rule_declared_when_command_scoped: bool,
    ) -> Result<Self, DeploymentTopologyAuditError> {
        if !topology_class_declared {
            return Err(DeploymentTopologyAuditError::TopologyClassMissing);
        }
        if !entrypoint_service_reference_declared {
            return Err(DeploymentTopologyAuditError::EntrypointServiceReferenceMissing);
        }
        if !startup_run_id_declared {
            return Err(DeploymentTopologyAuditError::StartupRunIdMissing);
        }
        if !correlation_rule_declared_when_command_scoped {
            return Err(DeploymentTopologyAuditError::CorrelationRuleMissing);
        }

        Ok(Self {
            event_type,
            topology_class_declared,
            entrypoint_service_reference_declared,
            startup_run_id_declared,
            correlation_rule_declared_when_command_scoped,
        })
    }
}

/// node affinity policy guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeAffinityPolicyGuard {
    state_class: NodeLocalStateClass,
    affinity_key_declared: bool,
    owning_node_scope_declared: bool,
    failover_behavior_declared: bool,
    unavailable_node_reason_declared: bool,
    recovery_replay_relation_declared: bool,
    evidence_class_declared: bool,
    wrong_node_access_fails_closed_without_distributed_state_policy: bool,
}

/// node affinity policy guard 生成時の未検査入力です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeAffinityPolicyGuardInput {
    pub state_class: NodeLocalStateClass,
    pub affinity_key_declared: bool,
    pub owning_node_scope_declared: bool,
    pub failover_behavior_declared: bool,
    pub unavailable_node_reason_declared: bool,
    pub recovery_replay_relation_declared: bool,
    pub evidence_class_declared: bool,
    pub wrong_node_access_fails_closed_without_distributed_state_policy: bool,
}

/// node affinity policy の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeAffinityPolicyError {
    /// affinity key がありません。
    AffinityKeyMissing,
    /// owning node scope がありません。
    OwningNodeScopeMissing,
    /// failover behavior がありません。
    FailoverBehaviorMissing,
    /// unavailable-node reason がありません。
    UnavailableNodeReasonMissing,
    /// recovery/replay relation がありません。
    RecoveryReplayRelationMissing,
    /// evidence class がありません。
    EvidenceClassMissing,
    /// distributed policy なしの wrong-node access が fail-closed ではありません。
    WrongNodeAccessNotFailClosed,
}

impl NodeAffinityPolicyGuard {
    /// node-local state が cluster-global として扱われないことを検査します。
    pub const fn try_new(
        input: NodeAffinityPolicyGuardInput,
    ) -> Result<Self, NodeAffinityPolicyError> {
        let NodeAffinityPolicyGuardInput {
            state_class,
            affinity_key_declared,
            owning_node_scope_declared,
            failover_behavior_declared,
            unavailable_node_reason_declared,
            recovery_replay_relation_declared,
            evidence_class_declared,
            wrong_node_access_fails_closed_without_distributed_state_policy,
        } = input;

        if !affinity_key_declared {
            return Err(NodeAffinityPolicyError::AffinityKeyMissing);
        }
        if !owning_node_scope_declared {
            return Err(NodeAffinityPolicyError::OwningNodeScopeMissing);
        }
        if !failover_behavior_declared {
            return Err(NodeAffinityPolicyError::FailoverBehaviorMissing);
        }
        if !unavailable_node_reason_declared {
            return Err(NodeAffinityPolicyError::UnavailableNodeReasonMissing);
        }
        if !recovery_replay_relation_declared {
            return Err(NodeAffinityPolicyError::RecoveryReplayRelationMissing);
        }
        if !evidence_class_declared {
            return Err(NodeAffinityPolicyError::EvidenceClassMissing);
        }
        if !wrong_node_access_fails_closed_without_distributed_state_policy {
            return Err(NodeAffinityPolicyError::WrongNodeAccessNotFailClosed);
        }

        Ok(Self {
            state_class,
            affinity_key_declared,
            owning_node_scope_declared,
            failover_behavior_declared,
            unavailable_node_reason_declared,
            recovery_replay_relation_declared,
            evidence_class_declared,
            wrong_node_access_fails_closed_without_distributed_state_policy,
        })
    }
}

/// service discovery relation guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TopologyServiceDiscoveryRelationGuard {
    endpoint_resolution_does_not_own_domain_decision: bool,
    endpoint_resolution_does_not_own_protocol_version_semantics: bool,
    endpoint_resolution_does_not_own_authorization_policy: bool,
    endpoint_resolution_does_not_prove_other_plane_readiness: bool,
    endpoint_resolution_does_not_prove_failover_recovery: bool,
    discovery_failure_not_hidden_by_unverified_fallback: bool,
    networked_resolved_endpoint_paired_with_trust_evidence_or_close_not_claimed: bool,
}

/// topology/service discovery relation の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TopologyServiceDiscoveryRelationError {
    /// service discovery が domain decision を所有しています。
    ServiceDiscoveryOwnsDomainDecision,
    /// service discovery が protocol version semantics を所有しています。
    ServiceDiscoveryOwnsProtocolVersionSemantics,
    /// service discovery が authorization policy を所有しています。
    ServiceDiscoveryOwnsAuthorizationPolicy,
    /// service discovery が他 plane readiness を証明しています。
    ServiceDiscoveryProvesOtherPlaneReadiness,
    /// service discovery が failover recovery を証明しています。
    ServiceDiscoveryProvesFailoverRecovery,
    /// discovery failure が unverified fallback で隠れています。
    DiscoveryFailureHiddenByFallback,
    /// networked endpoint evidence に trust evidence / close-not-claimed がありません。
    NetworkedEndpointTrustEvidenceMissing,
}

impl TopologyServiceDiscoveryRelationGuard {
    /// service discovery を topology の補助関係に限定します。
    pub const fn try_new(
        endpoint_resolution_does_not_own_domain_decision: bool,
        endpoint_resolution_does_not_own_protocol_version_semantics: bool,
        endpoint_resolution_does_not_own_authorization_policy: bool,
        endpoint_resolution_does_not_prove_other_plane_readiness: bool,
        endpoint_resolution_does_not_prove_failover_recovery: bool,
        discovery_failure_not_hidden_by_unverified_fallback: bool,
        networked_resolved_endpoint_paired_with_trust_evidence_or_close_not_claimed: bool,
    ) -> Result<Self, TopologyServiceDiscoveryRelationError> {
        if !endpoint_resolution_does_not_own_domain_decision {
            return Err(TopologyServiceDiscoveryRelationError::ServiceDiscoveryOwnsDomainDecision);
        }
        if !endpoint_resolution_does_not_own_protocol_version_semantics {
            return Err(
                TopologyServiceDiscoveryRelationError::ServiceDiscoveryOwnsProtocolVersionSemantics,
            );
        }
        if !endpoint_resolution_does_not_own_authorization_policy {
            return Err(
                TopologyServiceDiscoveryRelationError::ServiceDiscoveryOwnsAuthorizationPolicy,
            );
        }
        if !endpoint_resolution_does_not_prove_other_plane_readiness {
            return Err(
                TopologyServiceDiscoveryRelationError::ServiceDiscoveryProvesOtherPlaneReadiness,
            );
        }
        if !endpoint_resolution_does_not_prove_failover_recovery {
            return Err(
                TopologyServiceDiscoveryRelationError::ServiceDiscoveryProvesFailoverRecovery,
            );
        }
        if !discovery_failure_not_hidden_by_unverified_fallback {
            return Err(TopologyServiceDiscoveryRelationError::DiscoveryFailureHiddenByFallback);
        }
        if !networked_resolved_endpoint_paired_with_trust_evidence_or_close_not_claimed {
            return Err(
                TopologyServiceDiscoveryRelationError::NetworkedEndpointTrustEvidenceMissing,
            );
        }

        Ok(Self {
            endpoint_resolution_does_not_own_domain_decision,
            endpoint_resolution_does_not_own_protocol_version_semantics,
            endpoint_resolution_does_not_own_authorization_policy,
            endpoint_resolution_does_not_prove_other_plane_readiness,
            endpoint_resolution_does_not_prove_failover_recovery,
            discovery_failure_not_hidden_by_unverified_fallback,
            networked_resolved_endpoint_paired_with_trust_evidence_or_close_not_claimed,
        })
    }
}

/// topology evidence guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeploymentTopologyEvidenceGuard {
    topology_class_declared: bool,
    audit_shape_declared: bool,
    edge_proxy_class_declared_when_ingress_metadata_affects_path: bool,
    process_entrypoint_set_declared: bool,
    node_scope_declared: bool,
    selected_service_endpoints_or_redacted_references_declared: bool,
    node_affinity_rule_declared_when_relevant: bool,
    discovery_failure_behavior_declared: bool,
    discovery_source_cache_fallback_class_declared_when_resolution_affects_evidence: bool,
    internal_service_trust_class_declared_when_network_identity_affects_evidence: bool,
    distributed_state_class_and_failover_admission_declared_when_state_can_move: bool,
    health_readiness_relation_declared: bool,
    close_not_claimed_scope_declared: bool,
    single_node_evidence_not_used_as_multi_node_proof: bool,
}

/// topology evidence の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeploymentTopologyEvidenceError {
    /// topology class がありません。
    TopologyClassMissing,
    /// topology audit shape がありません。
    AuditShapeMissing,
    /// edge/proxy class がありません。
    EdgeProxyClassMissing,
    /// process/entrypoint set がありません。
    ProcessEntrypointSetMissing,
    /// node scope がありません。
    NodeScopeMissing,
    /// selected endpoint/reference がありません。
    SelectedEndpointReferenceMissing,
    /// node affinity rule がありません。
    NodeAffinityRuleMissing,
    /// discovery failure behavior がありません。
    DiscoveryFailureBehaviorMissing,
    /// discovery source/cache/fallback class がありません。
    DiscoverySourceCacheFallbackMissing,
    /// internal service trust class がありません。
    InternalServiceTrustClassMissing,
    /// distributed state/failover admission がありません。
    DistributedStateFailoverAdmissionMissing,
    /// health/readiness relation がありません。
    HealthReadinessRelationMissing,
    /// close-not-claimed scope がありません。
    CloseNotClaimedScopeMissing,
    /// single-node evidence を multi-node proof として扱っています。
    SingleNodeEvidenceUsedAsMultiNodeProof,
}

