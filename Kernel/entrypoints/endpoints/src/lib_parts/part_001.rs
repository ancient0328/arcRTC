// entrypoints/endpoints は public endpoint lifecycle と edge/proxy trust wiring の surface です。
//
// endpoint class の選択や edge metadata の扱いを entrypoint wiring として分離し、
// core の Signaling/SFU/TURN semantics をここで定義しません。

use arcrtc_core_reason::CatalogedReasonRef;

/// entrypoint endpoints package の所有境界を示す marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntrypointEndpointsSurface;

/// v0.2 initial architecture の endpoint class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PublicEndpointClass {
    /// WebSocket / HTTP signaling ingress.
    SignalingPublic,
    /// STUN/TURN UDP/TCP/TLS listener.
    TurnPublicRelay,
    /// WebRTC media transport ingress/egress.
    SfuMediaPublic,
    /// explicitly admitted readonly health/readiness observation.
    HealthPublicReadonly,
    /// private operator/admin route.
    AdminPrivate,
    /// private service-to-service control route.
    InternalControlPrivate,
    /// local test/fake surface only.
    TestOnlyEndpoint,
}

/// endpoint が露出する scope の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EndpointExposureScope {
    /// local test/fake scope only.
    LocalTestOnly,
    /// private operator or service network scope.
    PrivateNetwork,
    /// public edge/proxy terminated scope.
    PublicEdgeTerminated,
    /// direct public listener scope.
    DirectPublic,
}

impl EndpointExposureScope {
    /// local test scope だけに閉じているかを返します。
    pub const fn is_local_test_only(self) -> bool {
        matches!(self, Self::LocalTestOnly)
    }
}

impl PublicEndpointClass {
    /// public listener として扱う endpoint class です。
    pub const fn is_public_surface(self) -> bool {
        matches!(
            self,
            Self::SignalingPublic
                | Self::TurnPublicRelay
                | Self::SfuMediaPublic
                | Self::HealthPublicReadonly
        )
    }

    /// local test scope にだけ閉じる endpoint class です。
    pub const fn is_test_only(self) -> bool {
        matches!(self, Self::TestOnlyEndpoint)
    }

    /// private class が public listener に既定 bind されてはいけないかを返します。
    pub const fn is_private_control_or_admin(self) -> bool {
        matches!(self, Self::AdminPrivate | Self::InternalControlPrivate)
    }

    /// endpoint class が許容する target contract です。
    pub const fn admits_target_contract(self, target_contract: EndpointTargetContract) -> bool {
        matches!(
            (self, target_contract),
            (Self::SignalingPublic, EndpointTargetContract::Signaling)
                | (Self::TurnPublicRelay, EndpointTargetContract::Turn)
                | (Self::SfuMediaPublic, EndpointTargetContract::Sfu)
                | (
                    Self::HealthPublicReadonly,
                    EndpointTargetContract::HealthReadonly
                )
                | (Self::AdminPrivate, EndpointTargetContract::OperatorAdmin)
                | (
                    Self::InternalControlPrivate,
                    EndpointTargetContract::InternalControl
                )
                | (Self::TestOnlyEndpoint, EndpointTargetContract::TestDouble)
        )
    }

    /// endpoint class が許容する concrete protocol class です。
    pub const fn admits_protocol_class(self, protocol_class: PublicEndpointProtocolClass) -> bool {
        matches!(
            (self, protocol_class),
            (Self::SignalingPublic, PublicEndpointProtocolClass::Http)
                | (
                    Self::SignalingPublic,
                    PublicEndpointProtocolClass::WebSocket
                )
                | (Self::TurnPublicRelay, PublicEndpointProtocolClass::Udp)
                | (Self::TurnPublicRelay, PublicEndpointProtocolClass::Tcp)
                | (Self::TurnPublicRelay, PublicEndpointProtocolClass::TlsTcp)
                | (
                    Self::SfuMediaPublic,
                    PublicEndpointProtocolClass::SecureMedia
                )
                | (
                    Self::HealthPublicReadonly,
                    PublicEndpointProtocolClass::Http
                )
                | (Self::AdminPrivate, PublicEndpointProtocolClass::Http)
                | (Self::AdminPrivate, PublicEndpointProtocolClass::WebSocket)
                | (
                    Self::InternalControlPrivate,
                    PublicEndpointProtocolClass::Http
                )
                | (
                    Self::InternalControlPrivate,
                    PublicEndpointProtocolClass::WebSocket
                )
                | (
                    Self::InternalControlPrivate,
                    PublicEndpointProtocolClass::TlsTcp
                )
                | (Self::TestOnlyEndpoint, PublicEndpointProtocolClass::Http)
                | (
                    Self::TestOnlyEndpoint,
                    PublicEndpointProtocolClass::WebSocket
                )
                | (Self::TestOnlyEndpoint, PublicEndpointProtocolClass::Udp)
                | (Self::TestOnlyEndpoint, PublicEndpointProtocolClass::Tcp)
        )
    }
}

/// public endpoint の concrete protocol class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PublicEndpointProtocolClass {
    /// HTTP request/response route.
    Http,
    /// WebSocket upgrade and frames.
    WebSocket,
    /// UDP datagram listener.
    Udp,
    /// TCP stream listener.
    Tcp,
    /// TLS-protected TCP listener.
    TlsTcp,
    /// DTLS/SRTP media path.
    SecureMedia,
}

/// endpoint target contract の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EndpointTargetContract {
    /// Signaling contract.
    Signaling,
    /// TURN contract.
    Turn,
    /// SFU contract.
    Sfu,
    /// health/readiness operational contract.
    HealthReadonly,
    /// operator/admin contract.
    OperatorAdmin,
    /// internal control-plane contract.
    InternalControl,
    /// test double contract.
    TestDouble,
}

/// public endpoint audit event type の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PublicEndpointAuditEventType {
    /// public endpoint and connection lifecycle decision.
    PublicEndpointConnectionDecision,
}

impl PublicEndpointAuditEventType {
    /// audit event catalog に接続する event type です。
    pub const fn event_type(self) -> &'static str {
        match self {
            Self::PublicEndpointConnectionDecision => "public_endpoint_connection_decision",
        }
    }
}

/// endpoint declaration admission guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PublicEndpointDeclarationGuard {
    endpoint_class: PublicEndpointClass,
    exposure_scope: EndpointExposureScope,
    protocol_class: PublicEndpointProtocolClass,
    target_contract: EndpointTargetContract,
    endpoint_class_declared: bool,
    service_discovery_scope_declared_when_resolved: bool,
    concrete_protocol_and_listener_owner_declared: bool,
    edge_proxy_trust_policy_declared_when_metadata_affects_endpoint: bool,
    target_contract_declared: bool,
    authentication_authorization_requirement_declared: bool,
    transport_security_profile_declared_when_exposed: bool,
    rate_quota_resource_connection_bound_policy_declared: bool,
    correlation_propagation_rule_declared: bool,
    audit_event_type: PublicEndpointAuditEventType,
    public_error_mapping_declared: bool,
    endpoint_metadata_redaction_rule_declared: bool,
}

/// endpoint declaration の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PublicEndpointDeclarationError {
    /// endpoint class がありません。
    EndpointClassMissing,
    /// endpoint class と protocol class が一致していません。
    EndpointProtocolClassMismatch,
    /// endpoint class と target contract が一致していません。
    EndpointTargetContractMismatch,
    /// test-only endpoint が local test scope 外に出ています。
    TestOnlyEndpointExposedOutsideLocalTest,
    /// resolved endpoint の service discovery source/scope がありません。
    ServiceDiscoveryScopeMissing,
    /// concrete protocol/listener owner がありません。
    ProtocolOrListenerOwnerMissing,
    /// edge/proxy metadata 影響時の trust policy がありません。
    EdgeProxyTrustPolicyMissing,
    /// target contract がありません。
    TargetContractMissing,
    /// authentication/authorization requirement がありません。
    AuthenticationAuthorizationMissing,
    /// local test scope 外 endpoint の transport security profile がありません。
    TransportSecurityProfileMissing,
    /// rate/quota/resource/connection bound policy がありません。
    BoundPolicyMissing,
    /// correlation propagation rule がありません。
    CorrelationPropagationMissing,
    /// public error mapping がありません。
    PublicErrorMappingMissing,
    /// endpoint metadata redaction rule がありません。
    EndpointMetadataRedactionMissing,
}

impl PublicEndpointDeclarationGuard {
    /// endpoint admission rule の必須 field を検査します。
    pub const fn try_new(
        endpoint_class: PublicEndpointClass,
        exposure_scope: EndpointExposureScope,
        protocol_class: PublicEndpointProtocolClass,
        target_contract: EndpointTargetContract,
        endpoint_class_declared: bool,
        service_discovery_scope_declared_when_resolved: bool,
        concrete_protocol_and_listener_owner_declared: bool,
        edge_proxy_trust_policy_declared_when_metadata_affects_endpoint: bool,
        target_contract_declared: bool,
        authentication_authorization_requirement_declared: bool,
        transport_security_profile_declared_when_exposed: bool,
        rate_quota_resource_connection_bound_policy_declared: bool,
        correlation_propagation_rule_declared: bool,
        audit_event_type: PublicEndpointAuditEventType,
        public_error_mapping_declared: bool,
        endpoint_metadata_redaction_rule_declared: bool,
    ) -> Result<Self, PublicEndpointDeclarationError> {
        if !endpoint_class_declared {
            return Err(PublicEndpointDeclarationError::EndpointClassMissing);
        }
        if !endpoint_class.admits_protocol_class(protocol_class) {
            return Err(PublicEndpointDeclarationError::EndpointProtocolClassMismatch);
        }
        if !endpoint_class.admits_target_contract(target_contract) {
            return Err(PublicEndpointDeclarationError::EndpointTargetContractMismatch);
        }
        if endpoint_class.is_test_only() && !exposure_scope.is_local_test_only() {
            return Err(PublicEndpointDeclarationError::TestOnlyEndpointExposedOutsideLocalTest);
        }
        if !service_discovery_scope_declared_when_resolved {
            return Err(PublicEndpointDeclarationError::ServiceDiscoveryScopeMissing);
        }
        if !concrete_protocol_and_listener_owner_declared {
            return Err(PublicEndpointDeclarationError::ProtocolOrListenerOwnerMissing);
        }
        if !edge_proxy_trust_policy_declared_when_metadata_affects_endpoint {
            return Err(PublicEndpointDeclarationError::EdgeProxyTrustPolicyMissing);
        }
        if !target_contract_declared {
            return Err(PublicEndpointDeclarationError::TargetContractMissing);
        }
        if !authentication_authorization_requirement_declared {
            return Err(PublicEndpointDeclarationError::AuthenticationAuthorizationMissing);
        }
        if !exposure_scope.is_local_test_only() && !transport_security_profile_declared_when_exposed
        {
            return Err(PublicEndpointDeclarationError::TransportSecurityProfileMissing);
        }
        if !rate_quota_resource_connection_bound_policy_declared {
            return Err(PublicEndpointDeclarationError::BoundPolicyMissing);
        }
        if !correlation_propagation_rule_declared {
            return Err(PublicEndpointDeclarationError::CorrelationPropagationMissing);
        }
        if !public_error_mapping_declared {
            return Err(PublicEndpointDeclarationError::PublicErrorMappingMissing);
        }
        if !endpoint_metadata_redaction_rule_declared {
            return Err(PublicEndpointDeclarationError::EndpointMetadataRedactionMissing);
        }

        Ok(Self {
            endpoint_class,
            exposure_scope,
            protocol_class,
            target_contract,
            endpoint_class_declared,
            service_discovery_scope_declared_when_resolved,
            concrete_protocol_and_listener_owner_declared,
            edge_proxy_trust_policy_declared_when_metadata_affects_endpoint,
            target_contract_declared,
            authentication_authorization_requirement_declared,
            transport_security_profile_declared_when_exposed,
            rate_quota_resource_connection_bound_policy_declared,
            correlation_propagation_rule_declared,
            audit_event_type,
            public_error_mapping_declared,
            endpoint_metadata_redaction_rule_declared,
        })
    }
}

/// connection lifecycle state の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectionLifecycleState {
    /// listener accepted or received transport material before protocol admission.
    PreOpen,
    /// external protocol/version/frame shape was checked by driver.
    ProtocolChecked,
    /// required transport/security/auth material was checked or rejected.
    SecurityChecked,
    /// core accepted the target semantic action.
    CoreAdmitted,
    /// connection is allowed to exchange admitted traffic.
    Active,
    /// endpoint/entrypoint is rejecting new work and finishing allowed in-flight work.
    Draining,
    /// idle/consent/lifetime policy ended the connection.
    IdleExpired,
    /// normal close completed with audit/reference material.
    ClosedSuccess,
    /// policy closed the connection with cataloged reason.
    ClosedByPolicy,
    /// driver/runtime failure ended the connection with cataloged reason.
    Failed,
}

impl ConnectionLifecycleState {
    /// active traffic に入れる state です。
    pub const fn is_active_traffic_state(self) -> bool {
        matches!(self, Self::Active)
    }

    /// close/failure reason が必要な state です。
    pub const fn requires_close_or_failure_reason(self) -> bool {
        matches!(
            self,
            Self::IdleExpired | Self::ClosedByPolicy | Self::Failed
        )
    }
}

/// connection lifecycle transition guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionLifecycleTransitionGuard {
    state: ConnectionLifecycleState,
    protocol_checked_before_core_entry: bool,
    security_checked_before_active_traffic: bool,
    core_admission_required_before_active_traffic: bool,
    physical_open_state_not_used_as_domain_admission: bool,
    core_rejection_allowed_after_physical_open: bool,
    close_or_failure_reason_declared_when_required: bool,
    close_success_has_audit_or_reference_material: bool,
}

/// connection lifecycle transition の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectionLifecycleTransitionError {
    /// protocol check 前に core entry へ進んでいます。
    ProtocolCheckMissing,
    /// security/auth check 前に active traffic へ進んでいます。
    SecurityCheckMissing,
    /// core admission 前に active traffic へ進んでいます。
    CoreAdmissionMissing,
    /// physical open state を domain admission として扱っています。
    PhysicalOpenUsedAsDomainAdmission,
    /// physical open 後の core rejection を許容していません。
    CoreRejectionAfterPhysicalOpenNotRepresented,
    /// close/failure reason が必要 state でありません。
    CloseOrFailureReasonMissing,
    /// closed_success に audit/reference material がありません。
    CloseSuccessAuditReferenceMissing,
}

impl ConnectionLifecycleTransitionGuard {
    /// physical connection state と semantic admission を分離して検査します。
    pub const fn try_new(
        state: ConnectionLifecycleState,
        protocol_checked_before_core_entry: bool,
        security_checked_before_active_traffic: bool,
        core_admission_required_before_active_traffic: bool,
        physical_open_state_not_used_as_domain_admission: bool,
        core_rejection_allowed_after_physical_open: bool,
        close_or_failure_reason_declared_when_required: bool,
        close_success_has_audit_or_reference_material: bool,
    ) -> Result<Self, ConnectionLifecycleTransitionError> {
        if !protocol_checked_before_core_entry {
            return Err(ConnectionLifecycleTransitionError::ProtocolCheckMissing);
        }
        if state.is_active_traffic_state() && !security_checked_before_active_traffic {
            return Err(ConnectionLifecycleTransitionError::SecurityCheckMissing);
        }
        if state.is_active_traffic_state() && !core_admission_required_before_active_traffic {
            return Err(ConnectionLifecycleTransitionError::CoreAdmissionMissing);
        }
        if !physical_open_state_not_used_as_domain_admission {
            return Err(ConnectionLifecycleTransitionError::PhysicalOpenUsedAsDomainAdmission);
        }
        if !core_rejection_allowed_after_physical_open {
            return Err(
                ConnectionLifecycleTransitionError::CoreRejectionAfterPhysicalOpenNotRepresented,
            );
        }
        if state.requires_close_or_failure_reason()
            && !close_or_failure_reason_declared_when_required
        {
            return Err(ConnectionLifecycleTransitionError::CloseOrFailureReasonMissing);
        }
        if matches!(state, ConnectionLifecycleState::ClosedSuccess)
            && !close_success_has_audit_or_reference_material
        {
            return Err(ConnectionLifecycleTransitionError::CloseSuccessAuditReferenceMissing);
        }

        Ok(Self {
            state,
            protocol_checked_before_core_entry,
            security_checked_before_active_traffic,
            core_admission_required_before_active_traffic,
            physical_open_state_not_used_as_domain_admission,
            core_rejection_allowed_after_physical_open,
            close_or_failure_reason_declared_when_required,
            close_success_has_audit_or_reference_material,
        })
    }
}

/// public/internal separation guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PublicInternalEndpointSeparationGuard {
    endpoint_class: PublicEndpointClass,
    private_endpoint_not_bound_to_public_listener_by_default: bool,
    shared_process_or_address_keeps_route_level_private_class: bool,
    own_authorization_canonical_required_for_private_route: bool,
    public_endpoint_does_not_inherit_internal_service_trust: bool,
}

/// public/internal separation の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PublicInternalEndpointSeparationError {
    /// private endpoint が public listener に既定公開されています。
    PrivateEndpointPublicByDefault,
    /// shared process/address で route-level private class が維持されていません。
    RouteLevelPrivateClassMissing,
    /// private route 固有の authorization Canonical がありません。
    PrivateRouteAuthorizationCanonicalMissing,
    /// public endpoint が internal service trust を継承しています。
    PublicEndpointInheritsInternalServiceTrust,
}

impl PublicInternalEndpointSeparationGuard {
    /// public endpoint admission と internal/admin route を分離します。
    pub const fn try_new(
        endpoint_class: PublicEndpointClass,
        private_endpoint_not_bound_to_public_listener_by_default: bool,
        shared_process_or_address_keeps_route_level_private_class: bool,
        own_authorization_canonical_required_for_private_route: bool,
        public_endpoint_does_not_inherit_internal_service_trust: bool,
    ) -> Result<Self, PublicInternalEndpointSeparationError> {
        if endpoint_class.is_private_control_or_admin()
            && !private_endpoint_not_bound_to_public_listener_by_default
        {
            return Err(PublicInternalEndpointSeparationError::PrivateEndpointPublicByDefault);
        }
        if endpoint_class.is_private_control_or_admin()
            && !shared_process_or_address_keeps_route_level_private_class
        {
            return Err(PublicInternalEndpointSeparationError::RouteLevelPrivateClassMissing);
        }
        if endpoint_class.is_private_control_or_admin()
            && !own_authorization_canonical_required_for_private_route
        {
            return Err(
                PublicInternalEndpointSeparationError::PrivateRouteAuthorizationCanonicalMissing,
            );
        }
        if !public_endpoint_does_not_inherit_internal_service_trust {
            return Err(
                PublicInternalEndpointSeparationError::PublicEndpointInheritsInternalServiceTrust,
            );
        }

        Ok(Self {
            endpoint_class,
            private_endpoint_not_bound_to_public_listener_by_default,
            shared_process_or_address_keeps_route_level_private_class,
            own_authorization_canonical_required_for_private_route,
            public_endpoint_does_not_inherit_internal_service_trust,
        })
    }
}

