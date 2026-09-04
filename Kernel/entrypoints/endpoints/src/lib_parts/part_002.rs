/// public endpoint failure mapping の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PublicEndpointFailureKind {
    /// endpoint class is not admitted.
    PublicEndpointNotAllowed,
    /// public endpoint version is unsupported.
    PublicEndpointVersionUnsupported,
    /// required public endpoint authentication is absent.
    PublicEndpointAuthRequired,
    /// protocol upgrade or handshake fails before core entry.
    PublicEndpointUpgradeFailed,
    /// connection lifecycle state transition is invalid.
    ConnectionLifecycleViolation,
    /// connection idle, consent, or lifetime expires.
    ConnectionIdleTimeout,
    /// connection close policy cannot produce required audit/reference material.
    ConnectionClosePolicyViolation,
}

impl PublicEndpointFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::PublicEndpointNotAllowed => "public_endpoint_not_allowed",
            Self::PublicEndpointVersionUnsupported => "public_endpoint_version_unsupported",
            Self::PublicEndpointAuthRequired => "public_endpoint_auth_required",
            Self::PublicEndpointUpgradeFailed => "public_endpoint_upgrade_failed",
            Self::ConnectionLifecycleViolation => "connection_lifecycle_violation",
            Self::ConnectionIdleTimeout => "connection_idle_timeout",
            Self::ConnectionClosePolicyViolation => "connection_close_policy_violation",
        }
    }
}

/// public endpoint failure です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PublicEndpointFailure {
    kind: PublicEndpointFailureKind,
    reason: CatalogedReasonRef,
}

impl PublicEndpointFailure {
    /// public endpoint failure を cataloged reason に接続します。
    pub fn from_kind(kind: PublicEndpointFailureKind) -> Self {
        let reason = CatalogedReasonRef::from_code(kind.reason_code())
            .expect("public endpoint reason code must be registered");
        Self { kind, reason }
    }
}

/// public endpoint lifecycle 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedPublicEndpointBehavior {
    /// public listener existence is treated as readiness.
    ListenerExistenceTreatedAsReadiness,
    /// driver socket state is treated as domain state.
    DriverSocketStateTreatedAsDomainState,
    /// internal control/admin route is exposed as public by default.
    InternalOrAdminRouteExposedPublicByDefault,
    /// WebSocket upgrade success is treated as participant admission.
    WebSocketUpgradeTreatedAsParticipantAdmission,
    /// UDP/TCP bind success is treated as TURN allocation or SFU route success.
    ListenerBindTreatedAsDomainSuccess,
    /// public endpoint error hides cataloged core reason behind generic success.
    PublicErrorHidesCatalogedCoreReason,
    /// endpoint class is added by naming convention without a source contract.
    EndpointClassAddedWithoutSourceContract,
    /// proxy metadata changes separation without edge trust admission.
    ProxyMetadataChangesSeparationWithoutTrustAdmission,
    /// service discovery changes separation without endpoint admission.
    DiscoveryChangesSeparationWithoutEndpointAdmission,
}

/// v0.2 initial architecture の edge/proxy class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeProxyClass {
    /// entrypoint/driver listener is directly exposed.
    DirectPublicListener,
    /// HTTP/WebSocket reverse proxy in front of entrypoint.
    ReverseProxyHttpWs,
    /// L4 load balancer before TURN/SFU/listener.
    TcpUdpLoadBalancer,
    /// TLS terminates before entrypoint listener.
    TlsTerminatingEdge,
    /// mesh sidecar/gateway supplies ingress metadata.
    ServiceMeshIngress,
    /// local test/fake edge.
    TestEdgeSimulator,
}

impl EdgeProxyClass {
    /// edge class が許容する metadata class です。
    pub const fn admits_metadata_class(self, metadata_class: TrustedMetadataClass) -> bool {
        matches!(
            (self, metadata_class),
            (
                Self::DirectPublicListener,
                TrustedMetadataClass::ClientAddressObservation
            ) | (Self::ReverseProxyHttpWs, TrustedMetadataClass::ForwardedFor)
                | (
                    Self::ReverseProxyHttpWs,
                    TrustedMetadataClass::ForwardedProto
                )
                | (
                    Self::ReverseProxyHttpWs,
                    TrustedMetadataClass::ForwardedHost
                )
                | (Self::ReverseProxyHttpWs, TrustedMetadataClass::OriginHeader)
                | (Self::ReverseProxyHttpWs, TrustedMetadataClass::HostHeader)
                | (
                    Self::ReverseProxyHttpWs,
                    TrustedMetadataClass::EdgeRequestId
                )
                | (
                    Self::TcpUdpLoadBalancer,
                    TrustedMetadataClass::ClientAddressObservation
                )
                | (
                    Self::TlsTerminatingEdge,
                    TrustedMetadataClass::ForwardedProto
                )
                | (
                    Self::TlsTerminatingEdge,
                    TrustedMetadataClass::ForwardedHost
                )
                | (Self::TlsTerminatingEdge, TrustedMetadataClass::HostHeader)
                | (Self::TlsTerminatingEdge, TrustedMetadataClass::SniHost)
                | (
                    Self::TlsTerminatingEdge,
                    TrustedMetadataClass::ClientAddressObservation
                )
                | (
                    Self::TlsTerminatingEdge,
                    TrustedMetadataClass::EdgeRequestId
                )
                | (
                    Self::ServiceMeshIngress,
                    TrustedMetadataClass::ClientAddressObservation
                )
                | (
                    Self::ServiceMeshIngress,
                    TrustedMetadataClass::EdgeRequestId
                )
                | (Self::ServiceMeshIngress, TrustedMetadataClass::HostHeader)
                | (Self::TestEdgeSimulator, TrustedMetadataClass::ForwardedFor)
                | (
                    Self::TestEdgeSimulator,
                    TrustedMetadataClass::ForwardedProto
                )
                | (Self::TestEdgeSimulator, TrustedMetadataClass::ForwardedHost)
                | (Self::TestEdgeSimulator, TrustedMetadataClass::OriginHeader)
                | (Self::TestEdgeSimulator, TrustedMetadataClass::HostHeader)
                | (Self::TestEdgeSimulator, TrustedMetadataClass::SniHost)
                | (
                    Self::TestEdgeSimulator,
                    TrustedMetadataClass::ClientAddressObservation
                )
                | (Self::TestEdgeSimulator, TrustedMetadataClass::EdgeRequestId)
        )
    }

    /// test scope にだけ閉じる edge class です。
    pub const fn is_test_only(self) -> bool {
        matches!(self, Self::TestEdgeSimulator)
    }
}

/// proxy-derived metadata class の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrustedMetadataClass {
    /// Forwarded / X-Forwarded-For.
    ForwardedFor,
    /// Forwarded / X-Forwarded-Proto.
    ForwardedProto,
    /// Forwarded / X-Forwarded-Host.
    ForwardedHost,
    /// Origin header.
    OriginHeader,
    /// Host / :authority header.
    HostHeader,
    /// TLS SNI host.
    SniHost,
    /// socket peer or proxy protocol address observation.
    ClientAddressObservation,
    /// proxy-generated request ID.
    EdgeRequestId,
}

impl TrustedMetadataClass {
    /// core identity として直接扱ってはいけない metadata class です。
    pub const fn raw_value_must_not_be_core_identity(self) -> bool {
        match self {
            Self::ForwardedFor
            | Self::ForwardedProto
            | Self::ForwardedHost
            | Self::OriginHeader
            | Self::HostHeader
            | Self::SniHost
            | Self::ClientAddressObservation
            | Self::EdgeRequestId => true,
        }
    }
}

/// edge/proxy audit event type の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeProxyAuditEventType {
    /// edge/proxy trust decision.
    EdgeProxyTrustDecision,
}

impl EdgeProxyAuditEventType {
    /// audit event catalog に接続する event type です。
    pub const fn event_type(self) -> &'static str {
        match self {
            Self::EdgeProxyTrustDecision => "edge_proxy_trust_decision",
        }
    }
}

/// edge/proxy trust admission guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeProxyTrustAdmissionGuard {
    edge_class: EdgeProxyClass,
    metadata_class: TrustedMetadataClass,
    edge_class_declared: bool,
    deployment_topology_class_declared: bool,
    trusted_upstream_identity_or_network_scope_declared: bool,
    accepted_header_metadata_classes_declared: bool,
    header_precedence_and_conflict_rule_declared: bool,
    maximum_hop_count_declared_when_forwarded_chain_accepted: bool,
    tls_termination_and_downstream_security_relation_declared: bool,
    origin_host_admission_rule_declared: bool,
    client_address_use_limit_declared: bool,
    rate_quota_admission_relation_declared: bool,
    audit_reference_rule_declared: bool,
    redaction_rule_declared: bool,
    test_edge_is_test_only: bool,
}

/// edge/proxy trust admission の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeProxyTrustAdmissionError {
    /// edge class がありません。
    EdgeClassMissing,
    /// edge class と metadata class が一致していません。
    MetadataClassNotAdmittedForEdgeClass,
    /// deployment topology class がありません。
    DeploymentTopologyClassMissing,
    /// trusted upstream identity/network scope がありません。
    TrustedUpstreamScopeMissing,
    /// accepted metadata class declaration がありません。
    AcceptedMetadataClassMissing,
    /// header precedence/conflict rule がありません。
    HeaderPrecedenceConflictRuleMissing,
    /// forwarded chain の maximum hop count がありません。
    MaximumHopCountMissing,
    /// TLS termination/downstream security relation がありません。
    TlsTerminationDownstreamSecurityRelationMissing,
    /// origin/host admission rule がありません。
    OriginHostAdmissionRuleMissing,
    /// client address use limit がありません。
    ClientAddressUseLimitMissing,
    /// rate/quota/admission relation がありません。
    RateQuotaAdmissionRelationMissing,
    /// audit reference rule がありません。
    AuditReferenceRuleMissing,
    /// redaction rule がありません。
    RedactionRuleMissing,
    /// test edge simulator が test scope の外で使われています。
    TestEdgeUsedOutsideTestScope,
}

impl EdgeProxyTrustAdmissionGuard {
    /// proxy-derived metadata を信用できる条件を検査します。
    pub const fn try_new(
        edge_class: EdgeProxyClass,
        metadata_class: TrustedMetadataClass,
        edge_class_declared: bool,
        deployment_topology_class_declared: bool,
        trusted_upstream_identity_or_network_scope_declared: bool,
        accepted_header_metadata_classes_declared: bool,
        header_precedence_and_conflict_rule_declared: bool,
        maximum_hop_count_declared_when_forwarded_chain_accepted: bool,
        tls_termination_and_downstream_security_relation_declared: bool,
        origin_host_admission_rule_declared: bool,
        client_address_use_limit_declared: bool,
        rate_quota_admission_relation_declared: bool,
        audit_reference_rule_declared: bool,
        redaction_rule_declared: bool,
        test_edge_is_test_only: bool,
    ) -> Result<Self, EdgeProxyTrustAdmissionError> {
        if !edge_class_declared {
            return Err(EdgeProxyTrustAdmissionError::EdgeClassMissing);
        }
        if !edge_class.admits_metadata_class(metadata_class) {
            return Err(EdgeProxyTrustAdmissionError::MetadataClassNotAdmittedForEdgeClass);
        }
        if !deployment_topology_class_declared {
            return Err(EdgeProxyTrustAdmissionError::DeploymentTopologyClassMissing);
        }
        if !trusted_upstream_identity_or_network_scope_declared {
            return Err(EdgeProxyTrustAdmissionError::TrustedUpstreamScopeMissing);
        }
        if !accepted_header_metadata_classes_declared {
            return Err(EdgeProxyTrustAdmissionError::AcceptedMetadataClassMissing);
        }
        if !header_precedence_and_conflict_rule_declared {
            return Err(EdgeProxyTrustAdmissionError::HeaderPrecedenceConflictRuleMissing);
        }
        if !maximum_hop_count_declared_when_forwarded_chain_accepted {
            return Err(EdgeProxyTrustAdmissionError::MaximumHopCountMissing);
        }
        if !tls_termination_and_downstream_security_relation_declared {
            return Err(
                EdgeProxyTrustAdmissionError::TlsTerminationDownstreamSecurityRelationMissing,
            );
        }
        if !origin_host_admission_rule_declared {
            return Err(EdgeProxyTrustAdmissionError::OriginHostAdmissionRuleMissing);
        }
        if !client_address_use_limit_declared {
            return Err(EdgeProxyTrustAdmissionError::ClientAddressUseLimitMissing);
        }
        if !rate_quota_admission_relation_declared {
            return Err(EdgeProxyTrustAdmissionError::RateQuotaAdmissionRelationMissing);
        }
        if !audit_reference_rule_declared {
            return Err(EdgeProxyTrustAdmissionError::AuditReferenceRuleMissing);
        }
        if !redaction_rule_declared {
            return Err(EdgeProxyTrustAdmissionError::RedactionRuleMissing);
        }
        if edge_class.is_test_only() && !test_edge_is_test_only {
            return Err(EdgeProxyTrustAdmissionError::TestEdgeUsedOutsideTestScope);
        }

        Ok(Self {
            edge_class,
            metadata_class,
            edge_class_declared,
            deployment_topology_class_declared,
            trusted_upstream_identity_or_network_scope_declared,
            accepted_header_metadata_classes_declared,
            header_precedence_and_conflict_rule_declared,
            maximum_hop_count_declared_when_forwarded_chain_accepted,
            tls_termination_and_downstream_security_relation_declared,
            origin_host_admission_rule_declared,
            client_address_use_limit_declared,
            rate_quota_admission_relation_declared,
            audit_reference_rule_declared,
            redaction_rule_declared,
            test_edge_is_test_only,
        })
    }
}

/// forwarded header / source address trust guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeProxyHeaderSourceGuard {
    edge_class: EdgeProxyClass,
    metadata_class: TrustedMetadataClass,
    immediate_upstream_trusted_by_policy: bool,
    deterministic_precedence_declared_when_sources_conflict: bool,
    raw_metadata_not_used_as_core_identity: bool,
    mapped_only_to_typed_policy_input: bool,
    proxy_request_id_not_used_as_correlation_id: bool,
}

/// header/source address trust の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeProxyHeaderSourceError {
    /// edge class と metadata class が一致していません。
    MetadataClassNotAdmittedForEdgeClass,
    /// immediate upstream が trust policy により信頼されていません。
    ImmediateUpstreamUntrusted,
    /// source conflict 時の deterministic precedence がありません。
    DeterministicPrecedenceMissing,
    /// raw metadata を core identity として使っています。
    RawMetadataUsedAsCoreIdentity,
    /// metadata が typed policy input を経由していません。
    TypedPolicyInputMissing,
    /// proxy request ID が CorrelationId を置換しています。
    ProxyRequestIdReplacesCorrelationId,
}
