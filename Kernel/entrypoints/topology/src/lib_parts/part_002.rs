impl DeploymentTopologyEvidenceGuard {
    /// topology evidence の採用条件を検査します。
    pub const fn try_new(
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
    ) -> Result<Self, DeploymentTopologyEvidenceError> {
        if !topology_class_declared {
            return Err(DeploymentTopologyEvidenceError::TopologyClassMissing);
        }
        if !audit_shape_declared {
            return Err(DeploymentTopologyEvidenceError::AuditShapeMissing);
        }
        if !edge_proxy_class_declared_when_ingress_metadata_affects_path {
            return Err(DeploymentTopologyEvidenceError::EdgeProxyClassMissing);
        }
        if !process_entrypoint_set_declared {
            return Err(DeploymentTopologyEvidenceError::ProcessEntrypointSetMissing);
        }
        if !node_scope_declared {
            return Err(DeploymentTopologyEvidenceError::NodeScopeMissing);
        }
        if !selected_service_endpoints_or_redacted_references_declared {
            return Err(DeploymentTopologyEvidenceError::SelectedEndpointReferenceMissing);
        }
        if !node_affinity_rule_declared_when_relevant {
            return Err(DeploymentTopologyEvidenceError::NodeAffinityRuleMissing);
        }
        if !discovery_failure_behavior_declared {
            return Err(DeploymentTopologyEvidenceError::DiscoveryFailureBehaviorMissing);
        }
        if !discovery_source_cache_fallback_class_declared_when_resolution_affects_evidence {
            return Err(DeploymentTopologyEvidenceError::DiscoverySourceCacheFallbackMissing);
        }
        if !internal_service_trust_class_declared_when_network_identity_affects_evidence {
            return Err(DeploymentTopologyEvidenceError::InternalServiceTrustClassMissing);
        }
        if !distributed_state_class_and_failover_admission_declared_when_state_can_move {
            return Err(DeploymentTopologyEvidenceError::DistributedStateFailoverAdmissionMissing);
        }
        if !health_readiness_relation_declared {
            return Err(DeploymentTopologyEvidenceError::HealthReadinessRelationMissing);
        }
        if !close_not_claimed_scope_declared {
            return Err(DeploymentTopologyEvidenceError::CloseNotClaimedScopeMissing);
        }
        if !single_node_evidence_not_used_as_multi_node_proof {
            return Err(DeploymentTopologyEvidenceError::SingleNodeEvidenceUsedAsMultiNodeProof);
        }

        Ok(Self {
            topology_class_declared,
            audit_shape_declared,
            edge_proxy_class_declared_when_ingress_metadata_affects_path,
            process_entrypoint_set_declared,
            node_scope_declared,
            selected_service_endpoints_or_redacted_references_declared,
            node_affinity_rule_declared_when_relevant,
            discovery_failure_behavior_declared,
            discovery_source_cache_fallback_class_declared_when_resolution_affects_evidence,
            internal_service_trust_class_declared_when_network_identity_affects_evidence,
            distributed_state_class_and_failover_admission_declared_when_state_can_move,
            health_readiness_relation_declared,
            close_not_claimed_scope_declared,
            single_node_evidence_not_used_as_multi_node_proof,
        })
    }
}

/// topology failure mapping の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeploymentTopologyFailureKind {
    /// selected topology not supported by current architecture phase.
    DeploymentTopologyUnsupported,
    /// service discovery cannot resolve required endpoint.
    ServiceDiscoveryUnavailable,
    /// discovery source is invalid.
    ServiceDiscoverySourceNotAdmitted,
    /// endpoint scope is invalid.
    ServiceEndpointScopeConflict,
    /// internal service identity/trust source is not admitted.
    InternalServiceIdentitySourceNotAdmitted,
    /// required service identity is absent.
    InternalServiceIdentityMissing,
    /// service identity material cannot be mapped.
    InternalServiceIdentityInvalid,
    /// service identity cannot be trusted.
    InternalServiceIdentityUntrusted,
    /// service identity scope conflicts with target.
    InternalServiceIdentityScopeConflict,
    /// peer verification failed for internal service trust.
    InternalServicePeerVerificationFailed,
    /// internal service trust policy is absent.
    InternalServiceTrustPolicyMissing,
    /// node affinity is required but absent.
    NodeAffinityRequired,
    /// required node-local state unavailable.
    NodeStateUnavailable,
    /// cross-node route or relay not allowed by policy.
    CrossNodeRouteNotAllowed,
    /// topology configuration missing.
    RuntimeConfigMissing,
    /// topology configuration invalid.
    RuntimeConfigInvalid,
    /// network send path unavailable.
    NetworkSendFailed,
    /// network receive path unavailable.
    NetworkReceiveFailed,
    /// driver/runtime shutdown.
    DriverShutdown,
}

impl DeploymentTopologyFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::DeploymentTopologyUnsupported => "deployment_topology_unsupported",
            Self::ServiceDiscoveryUnavailable => "service_discovery_unavailable",
            Self::ServiceDiscoverySourceNotAdmitted => "service_discovery_source_not_admitted",
            Self::ServiceEndpointScopeConflict => "service_endpoint_scope_conflict",
            Self::InternalServiceIdentitySourceNotAdmitted => {
                "internal_service_identity_source_not_admitted"
            }
            Self::InternalServiceIdentityMissing => "internal_service_identity_missing",
            Self::InternalServiceIdentityInvalid => "internal_service_identity_invalid",
            Self::InternalServiceIdentityUntrusted => "internal_service_identity_untrusted",
            Self::InternalServiceIdentityScopeConflict => {
                "internal_service_identity_scope_conflict"
            }
            Self::InternalServicePeerVerificationFailed => {
                "internal_service_peer_verification_failed"
            }
            Self::InternalServiceTrustPolicyMissing => "internal_service_trust_policy_missing",
            Self::NodeAffinityRequired => "node_affinity_required",
            Self::NodeStateUnavailable => "node_state_unavailable",
            Self::CrossNodeRouteNotAllowed => "cross_node_route_not_allowed",
            Self::RuntimeConfigMissing => "runtime_config_missing",
            Self::RuntimeConfigInvalid => "runtime_config_invalid",
            Self::NetworkSendFailed => "network_send_failed",
            Self::NetworkReceiveFailed => "network_receive_failed",
            Self::DriverShutdown => "driver_shutdown",
        }
    }
}

/// topology failure です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeploymentTopologyFailure {
    kind: DeploymentTopologyFailureKind,
    reason: CatalogedReasonRef,
}

impl DeploymentTopologyFailure {
    /// topology failure を cataloged reason に接続します。
    pub fn from_kind(kind: DeploymentTopologyFailureKind) -> Self {
        let reason = CatalogedReasonRef::from_code(kind.reason_code())
            .expect("deployment topology reason code must be registered");
        Self { kind, reason }
    }
}

/// topology 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedDeploymentTopologyBehavior {
    /// topology selection changes core semantics silently.
    TopologySelectionChangesCoreSemantics,
    /// service discovery owns domain decision.
    ServiceDiscoveryOwnsDomainDecision,
    /// discovery or TLS listener startup is treated as internal service trust.
    DiscoveryOrTlsStartupTreatedAsInternalTrust,
    /// node-local SFU/TURN state is treated as cluster-global by default.
    NodeLocalStateTreatedAsClusterGlobal,
    /// failover success is claimed without recovery/replay evidence.
    FailoverClaimedWithoutRecoveryReplayEvidence,
    /// failover success is claimed from service discovery fallback alone.
    FailoverClaimedFromDiscoveryFallbackAlone,
    /// replication/consensus is implied by multi-node topology.
    ReplicationConsensusImpliedByMultiNodeTopology,
    /// sticky routing requirement is hidden.
    StickyRoutingRequirementHidden,
    /// local dev topology is treated as production topology.
    LocalDevTopologyTreatedAsProduction,
    /// proxy/LB metadata is treated as trusted topology evidence without policy.
    ProxyMetadataTrustedWithoutEdgePolicy,
}

/// v0.2 initial architecture の discovery source class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiscoverySourceClass {
    /// endpoint is supplied by typed startup configuration.
    StaticConfigEndpoint,
    /// entrypoints composition resolves in-process or same-host service.
    LocalProcessRegistry,
    /// DNS name resolves service endpoint.
    DnsResolution,
    /// registry/control-plane store resolves endpoint.
    ServiceRegistryLookup,
    /// mesh/sidecar resolves route.
    ServiceMeshResolution,
    /// deterministic fake resolver for tests.
    TestResolver,
}

impl DiscoverySourceClass {
    /// topology class と discovery source class の整合です。
    pub const fn admits_topology(self, topology_class: DeploymentTopologyClass) -> bool {
        matches!(
            (self, topology_class),
            (
                Self::StaticConfigEndpoint,
                DeploymentTopologyClass::SingleProcessLocal
            ) | (
                Self::StaticConfigEndpoint,
                DeploymentTopologyClass::SplitPlaneSameHost
            ) | (
                Self::StaticConfigEndpoint,
                DeploymentTopologyClass::SplitPlaneNetworked
            ) | (
                Self::StaticConfigEndpoint,
                DeploymentTopologyClass::MultiNodeExperimental
            ) | (
                Self::StaticConfigEndpoint,
                DeploymentTopologyClass::ExternalManagedDependency
            ) | (
                Self::LocalProcessRegistry,
                DeploymentTopologyClass::SingleProcessLocal
            ) | (
                Self::LocalProcessRegistry,
                DeploymentTopologyClass::SplitPlaneSameHost
            ) | (
                Self::DnsResolution,
                DeploymentTopologyClass::SplitPlaneNetworked
            ) | (
                Self::DnsResolution,
                DeploymentTopologyClass::MultiNodeExperimental
            ) | (
                Self::DnsResolution,
                DeploymentTopologyClass::ExternalManagedDependency
            ) | (
                Self::ServiceRegistryLookup,
                DeploymentTopologyClass::SplitPlaneNetworked
            ) | (
                Self::ServiceRegistryLookup,
                DeploymentTopologyClass::MultiNodeExperimental
            ) | (
                Self::ServiceRegistryLookup,
                DeploymentTopologyClass::ExternalManagedDependency
            ) | (
                Self::ServiceMeshResolution,
                DeploymentTopologyClass::SplitPlaneNetworked
            ) | (
                Self::ServiceMeshResolution,
                DeploymentTopologyClass::MultiNodeExperimental
            ) | (
                Self::TestResolver,
                DeploymentTopologyClass::SingleProcessLocal
            )
        )
    }

    /// TTL/cache/staleness rule が必須になる source です。
    pub const fn requires_ttl_cache_rule(self) -> bool {
        matches!(
            self,
            Self::DnsResolution | Self::ServiceRegistryLookup | Self::ServiceMeshResolution
        )
    }

    /// registry contract が必須になる source です。
    pub const fn requires_registry_contract(self) -> bool {
        matches!(self, Self::ServiceRegistryLookup)
    }

    /// mesh policy を authorization として扱ってはいけない source です。
    pub const fn is_mesh_resolution(self) -> bool {
        matches!(self, Self::ServiceMeshResolution)
    }

    /// test evidence にだけ閉じる source です。
    pub const fn is_test_only(self) -> bool {
        matches!(self, Self::TestResolver)
    }
}

/// endpoint resolution state の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EndpointResolutionState {
    /// selected topology does not require endpoint lookup.
    ResolutionNotRequired,
    /// lookup has not produced accepted endpoint.
    ResolutionPending,
    /// endpoint reference accepted for declared scope.
    ResolutionAccepted,
    /// lookup result rejected by policy.
    ResolutionRejected,
    /// cached endpoint exceeded TTL/generation/window.
    ResolutionStale,
    /// lookup failed or driver unavailable.
    ResolutionFailed,
}

impl EndpointResolutionState {
    /// cataloged reason が必要な non-success state です。
    pub const fn requires_reason(self) -> bool {
        matches!(
            self,
            Self::ResolutionRejected | Self::ResolutionStale | Self::ResolutionFailed
        )
    }
}

/// resolved endpoint scope の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResolvedEndpointScope {
    /// local in-process endpoint reference.
    LocalProcess,
    /// same-host service endpoint.
    SameHostService,
    /// internal service endpoint.
    InternalService,
    /// public endpoint reference.
    PublicEndpoint,
    /// external managed dependency endpoint.
    ExternalDependency,
    /// test-only endpoint.
    TestOnly,
}

impl ResolvedEndpointScope {
    /// internal service trust relation が必要な scope です。
    pub const fn requires_internal_service_trust_relation(self) -> bool {
        matches!(self, Self::InternalService)
    }

    /// public endpoint admission が必要な scope です。
    pub const fn requires_public_endpoint_admission(self) -> bool {
        matches!(self, Self::PublicEndpoint)
    }
}

/// endpoint resolution target service/plane です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResolutionTargetService {
    /// Signaling service/plane.
    Signaling,
    /// SFU service/plane.
    Sfu,
    /// TURN service/plane.
    Turn,
    /// health/readiness surface.
    HealthReadonly,
    /// internal control-plane.
    InternalControl,
    /// operator/admin surface.
    Admin,
    /// external dependency.
    ExternalDependency,
    /// deterministic test service.
    TestOnly,
}

impl ResolutionTargetService {
    /// target service 側から internal service trust relation が必要かを返します。
    pub const fn requires_internal_service_trust_relation(self) -> bool {
        matches!(self, Self::InternalControl)
    }
}

/// service discovery audit event type の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceDiscoveryAuditEventType {
    /// service discovery / endpoint resolution decision.
    ServiceDiscoveryResolutionDecision,
}

impl ServiceDiscoveryAuditEventType {
    /// audit event catalog に接続する event type です。
    pub const fn event_type(self) -> &'static str {
        match self {
            Self::ServiceDiscoveryResolutionDecision => "service_discovery_resolution_decision",
        }
    }
}

/// service discovery / endpoint resolution admission guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ServiceDiscoveryResolutionAdmissionGuard {
    discovery_source_class: DiscoverySourceClass,
    topology_class: DeploymentTopologyClass,
    target_service: ResolutionTargetService,
    endpoint_scope: ResolvedEndpointScope,
    discovery_source_class_declared: bool,
    topology_class_declared: bool,
    target_service_declared: bool,
    expected_endpoint_scope_declared: bool,
    public_internal_endpoint_relation_declared: bool,
    endpoint_contract_reference_declared_when_applicable: bool,
    ttl_cache_staleness_rule_declared: bool,
    fallback_behavior_declared: bool,
    authorization_context_relation_declared: bool,
    service_identity_trust_relation_declared_when_internal_target: bool,
    readiness_relation_declared: bool,
    audit_shape_declared: bool,
    registry_contract_declared_when_required: bool,
    mesh_policy_not_used_as_authorization: bool,
    test_resolver_evidence_is_test_only: bool,
}

/// service discovery admission の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceDiscoveryResolutionAdmissionError {
    /// discovery source class がありません。
    DiscoverySourceClassMissing,
    /// discovery source と topology class が一致していません。
    DiscoverySourceTopologyMismatch,
    /// topology class がありません。
    TopologyClassMissing,
    /// target service がありません。
    TargetServiceMissing,
    /// endpoint scope がありません。
    EndpointScopeMissing,
    /// public/internal endpoint relation がありません。
    PublicInternalEndpointRelationMissing,
    /// endpoint contract reference がありません。
    EndpointContractReferenceMissing,
    /// TTL/cache/staleness rule がありません。
    TtlCacheStalenessRuleMissing,
    /// fallback behavior がありません。
    FallbackBehaviorMissing,
    /// authorization/context relation がありません。
    AuthorizationContextRelationMissing,
    /// internal service identity/trust relation がありません。
    ServiceIdentityTrustRelationMissing,
    /// readiness relation がありません。
    ReadinessRelationMissing,
    /// audit shape がありません。
    AuditShapeMissing,
    /// registry contract がありません。
    RegistryContractMissing,
    /// mesh policy を authorization として扱っています。
    MeshPolicyUsedAsAuthorization,
    /// test resolver が test evidence の外で使われています。
    TestResolverUsedOutsideTestEvidence,
}

