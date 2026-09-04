/// internal control-plane failure mapping の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalControlPlaneFailureKind {
    /// internal control message cannot map to contract.
    InternalControlMessageInvalid,
    /// internal control contract version unsupported.
    InternalControlVersionUnsupported,
    /// internal control authorization context missing.
    InternalControlAuthorizationMissing,
    /// internal control authorization denied.
    InternalControlAuthorizationDenied,
    /// internal service identity / trust mapping failed.
    InternalServiceTrustFailure(InternalServiceTrustFailureKind),
    /// required service endpoint cannot be resolved.
    ServiceDiscoveryUnavailable,
    /// command requires node affinity but affinity is absent.
    NodeAffinityRequired,
    /// target node-local state unavailable.
    NodeStateUnavailable,
    /// cross-node route not allowed.
    CrossNodeRouteNotAllowed,
    /// internal control response timeout.
    OperationDeadlineExceeded,
    /// resolved endpoint is stale.
    ServiceEndpointStale,
    /// endpoint fallback is not admitted.
    ServiceEndpointFallbackNotAllowed,
    /// distributed owner/state conflict is detected.
    StateOwnerConflict,
    /// split-brain risk is detected.
    SplitBrainRiskDetected,
    /// network send failed.
    NetworkSendFailed,
    /// network receive failed.
    NetworkReceiveFailed,
    /// driver shutdown.
    DriverShutdown,
}

impl InternalControlPlaneFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::InternalControlMessageInvalid => "internal_control_message_invalid",
            Self::InternalControlVersionUnsupported => "internal_control_version_unsupported",
            Self::InternalControlAuthorizationMissing => "internal_control_authorization_missing",
            Self::InternalControlAuthorizationDenied => "internal_control_authorization_denied",
            Self::InternalServiceTrustFailure(kind) => kind.reason_code(),
            Self::ServiceDiscoveryUnavailable => "service_discovery_unavailable",
            Self::NodeAffinityRequired => "node_affinity_required",
            Self::NodeStateUnavailable => "node_state_unavailable",
            Self::CrossNodeRouteNotAllowed => "cross_node_route_not_allowed",
            Self::OperationDeadlineExceeded => "operation_deadline_exceeded",
            Self::ServiceEndpointStale => "service_endpoint_stale",
            Self::ServiceEndpointFallbackNotAllowed => "service_endpoint_fallback_not_allowed",
            Self::StateOwnerConflict => "state_owner_conflict",
            Self::SplitBrainRiskDetected => "split_brain_risk_detected",
            Self::NetworkSendFailed => "network_send_failed",
            Self::NetworkReceiveFailed => "network_receive_failed",
            Self::DriverShutdown => "driver_shutdown",
        }
    }
}

/// internal control-plane failure です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InternalControlPlaneFailure {
    kind: InternalControlPlaneFailureKind,
    reason: CatalogedReasonRef,
}

impl InternalControlPlaneFailure {
    /// internal control-plane failure を cataloged reason に接続します。
    pub fn from_kind(kind: InternalControlPlaneFailureKind) -> Self {
        let reason = CatalogedReasonRef::from_code(kind.reason_code())
            .expect("internal control-plane reason code must be registered");
        Self { kind, reason }
    }
}

/// internal control-plane 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedInternalControlPlaneBehavior {
    /// internal RPC status becomes core reason.
    InternalRpcStatusBecomesCoreReason,
    /// service discovery owns domain decision.
    ServiceDiscoveryOwnsDomainDecision,
    /// endpoint resolution success is treated as internal control success.
    EndpointResolutionSuccessTreatedAsInternalControlSuccess,
    /// external client command bypasses public Signaling contract.
    ExternalClientCommandBypassesPublicSignalingContract,
    /// internal control authorization is inferred from network reachability.
    AuthorizationInferredFromNetworkReachability,
    /// internal service identity is inferred from endpoint resolution, TLS listener startup, or mesh route name.
    ServiceIdentityInferredFromEndpointTlsListenerOrMeshRoute,
    /// topology change alters Signaling/SFU/TURN state semantics.
    TopologyChangeAltersDomainStateSemantics,
    /// driver-to-driver internal call becomes domain authority.
    DriverToDriverInternalCallBecomesDomainAuthority,
    /// internal transport encoding changes domain decision semantics.
    InternalTransportEncodingChangesDomainDecisionSemantics,
    /// service-to-service failure is recorded only as free-text or external status.
    ServiceToServiceFailureRecordedAsFreeTextOnly,
    /// in-process control relation is used as same-host, networked, or multi-node control.
    InProcessRelationUsedAsRemoteControl,
}
