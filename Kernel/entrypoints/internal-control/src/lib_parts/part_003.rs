/// internal control-plane contract admission の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalControlPlaneContractError {
    /// control-plane class がありません。
    ControlPlaneClassMissing,
    /// source service がありません。
    SourceServiceMissing,
    /// target service がありません。
    TargetServiceMissing,
    /// command/event type がありません。
    CommandEventTypeMissing,
    /// command/event class と admitted type reference が一致していません。
    CommandEventClassMismatch,
    /// contract version がありません。
    ContractVersionMissing,
    /// contract version が未対応です。
    ContractVersionUnsupported,
    /// CorrelationId がありません。
    CorrelationIdMissing,
    /// replay 可能 command の idempotency class がありません。
    IdempotencyClassMissing,
    /// authorization context class がありません。
    AuthorizationContextClassMissing,
    /// authorization context 欠落要求です。
    AuthorizationContextMissing,
    /// internal service trust class が必要な control path で宣言されていません。
    InternalServiceTrustClassMissing,
    /// endpoint と authorization context の relation がありません。
    EndpointAuthorizationRelationMissing,
    /// service discovery / endpoint resolution relation がありません。
    ServiceDiscoveryRelationMissing,
    /// node affinity key と unavailable behavior がありません。
    NodeAffinityRuleMissing,
    /// admin authorization relation がありません。
    AdminAuthorizationRelationMissing,
    /// timeout/cancellation policy がありません。
    TimeoutCancellationPolicyMissing,
    /// failure reason mapping がありません。
    FailureReasonMappingMissing,
    /// audit event relation がありません。
    AuditEventRelationMissing,
    /// driver status / RPC exception が core reason を置換しています。
    DriverStatusBecameAuthoritativeReason,
    /// external client command が public Signaling contract を迂回しています。
    ExternalClientBypassesPublicSignalingContract,
    /// internal transport encoding が domain semantics を変更しています。
    InternalTransportEncodingChangesDomainSemantics,
    /// service discovery が domain decision authority になっています。
    ServiceDiscoveryOwnsDomainDecision,
}

impl InternalControlPlaneContractGuard {
    /// internal control-plane contract の必須 shape を検査します。
    pub const fn try_new(
        control_plane_class: InternalControlPlaneClass,
        source_service: InternalServiceRole,
        target_service: InternalServiceRole,
        message_class: InternalControlMessageClass,
        command_event_type: InternalControlCommandEventType,
        contract_version_state: InternalControlContractVersionState,
        authorization_context_class: InternalControlAuthorizationContextClass,
        control_plane_class_declared: bool,
        source_service_declared: bool,
        target_service_declared: bool,
        command_event_type_declared: bool,
        correlation_id_declared: bool,
        command_replay_possible: bool,
        idempotency_class_declared_when_command_replay_possible: bool,
        authorization_context_class_declared: bool,
        internal_service_trust_class_declared_when_required: bool,
        endpoint_and_auth_context_declared_when_required: bool,
        service_discovery_relation_declared_when_required: bool,
        node_affinity_key_and_unavailable_behavior_declared_when_required: bool,
        admin_authorization_relation_declared_when_required: bool,
        timeout_cancellation_policy_declared: bool,
        failure_reason_mapping_declared: bool,
        audit_event_relation_declared: bool,
        driver_status_or_rpc_exception_not_authoritative_reason: bool,
        external_client_bypass_to_internal_surface_prevented: bool,
        internal_transport_encoding_not_domain_semantics: bool,
        service_discovery_not_domain_authority: bool,
    ) -> Result<Self, InternalControlPlaneContractError> {
        if !control_plane_class_declared {
            return Err(InternalControlPlaneContractError::ControlPlaneClassMissing);
        }
        if !source_service_declared {
            return Err(InternalControlPlaneContractError::SourceServiceMissing);
        }
        if !target_service_declared {
            return Err(InternalControlPlaneContractError::TargetServiceMissing);
        }
        if !command_event_type_declared {
            return Err(InternalControlPlaneContractError::CommandEventTypeMissing);
        }
        if !command_event_type.admits_message_class(message_class) {
            return Err(InternalControlPlaneContractError::CommandEventClassMismatch);
        }
        if !contract_version_state.is_declared() {
            return Err(InternalControlPlaneContractError::ContractVersionMissing);
        }
        if !contract_version_state.is_supported() {
            return Err(InternalControlPlaneContractError::ContractVersionUnsupported);
        }
        if !correlation_id_declared {
            return Err(InternalControlPlaneContractError::CorrelationIdMissing);
        }
        if command_replay_possible && !idempotency_class_declared_when_command_replay_possible {
            return Err(InternalControlPlaneContractError::IdempotencyClassMissing);
        }
        if !authorization_context_class_declared {
            return Err(InternalControlPlaneContractError::AuthorizationContextClassMissing);
        }
        if authorization_context_class.is_missing_requested() {
            return Err(InternalControlPlaneContractError::AuthorizationContextMissing);
        }
        if control_plane_class.requires_internal_service_trust_context()
            && !internal_service_trust_class_declared_when_required
        {
            return Err(InternalControlPlaneContractError::InternalServiceTrustClassMissing);
        }
        if control_plane_class.requires_endpoint_and_auth_context()
            && !endpoint_and_auth_context_declared_when_required
        {
            return Err(InternalControlPlaneContractError::EndpointAuthorizationRelationMissing);
        }
        if control_plane_class.requires_service_discovery_relation()
            && !service_discovery_relation_declared_when_required
        {
            return Err(InternalControlPlaneContractError::ServiceDiscoveryRelationMissing);
        }
        if control_plane_class.requires_node_affinity_rule()
            && !node_affinity_key_and_unavailable_behavior_declared_when_required
        {
            return Err(InternalControlPlaneContractError::NodeAffinityRuleMissing);
        }
        if control_plane_class.requires_admin_authorization_relation()
            && !admin_authorization_relation_declared_when_required
        {
            return Err(InternalControlPlaneContractError::AdminAuthorizationRelationMissing);
        }
        if !timeout_cancellation_policy_declared {
            return Err(InternalControlPlaneContractError::TimeoutCancellationPolicyMissing);
        }
        if !failure_reason_mapping_declared {
            return Err(InternalControlPlaneContractError::FailureReasonMappingMissing);
        }
        if !audit_event_relation_declared {
            return Err(InternalControlPlaneContractError::AuditEventRelationMissing);
        }
        if !driver_status_or_rpc_exception_not_authoritative_reason {
            return Err(InternalControlPlaneContractError::DriverStatusBecameAuthoritativeReason);
        }
        if !external_client_bypass_to_internal_surface_prevented {
            return Err(
                InternalControlPlaneContractError::ExternalClientBypassesPublicSignalingContract,
            );
        }
        if !internal_transport_encoding_not_domain_semantics {
            return Err(
                InternalControlPlaneContractError::InternalTransportEncodingChangesDomainSemantics,
            );
        }
        if !service_discovery_not_domain_authority {
            return Err(InternalControlPlaneContractError::ServiceDiscoveryOwnsDomainDecision);
        }

        Ok(Self {
            control_plane_class,
            source_service,
            target_service,
            message_class,
            command_event_type,
            contract_version_state,
            authorization_context_class,
            control_plane_class_declared,
            source_service_declared,
            target_service_declared,
            command_event_type_declared,
            correlation_id_declared,
            command_replay_possible,
            idempotency_class_declared_when_command_replay_possible,
            authorization_context_class_declared,
            internal_service_trust_class_declared_when_required,
            endpoint_and_auth_context_declared_when_required,
            service_discovery_relation_declared_when_required,
            node_affinity_key_and_unavailable_behavior_declared_when_required,
            admin_authorization_relation_declared_when_required,
            timeout_cancellation_policy_declared,
            failure_reason_mapping_declared,
            audit_event_relation_declared,
            driver_status_or_rpc_exception_not_authoritative_reason,
            external_client_bypass_to_internal_surface_prevented,
            internal_transport_encoding_not_domain_semantics,
            service_discovery_not_domain_authority,
        })
    }
}

/// internal control-plane audit event type の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalControlPlaneAuditEventType {
    /// internal control-plane decision.
    InternalControlPlaneDecision,
}

impl InternalControlPlaneAuditEventType {
    /// audit event catalog に接続する event type です。
    pub const fn event_type(self) -> &'static str {
        match self {
            Self::InternalControlPlaneDecision => "internal_control_plane_decision",
        }
    }
}

/// internal control-plane audit shape guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InternalControlPlaneAuditGuard {
    event_type: InternalControlPlaneAuditEventType,
    outcome: InternalControlPlaneOutcome,
    correlation_id_declared: bool,
    outcome_declared: bool,
    source_service_declared: bool,
    target_service_declared: bool,
    control_plane_class_declared: bool,
    contract_version_declared: bool,
    topology_class_declared_when_available: bool,
    cataloged_reason_declared_for_rejected_expired_failed_outcome: bool,
}

/// internal control-plane audit shape の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalControlPlaneAuditError {
    /// CorrelationId がありません。
    CorrelationIdMissing,
    /// decision outcome がありません。
    OutcomeMissing,
    /// source service がありません。
    SourceServiceMissing,
    /// target service がありません。
    TargetServiceMissing,
    /// control-plane class がありません。
    ControlPlaneClassMissing,
    /// contract version がありません。
    ContractVersionMissing,
    /// available topology class がありません。
    TopologyClassMissing,
    /// non-success outcome の cataloged reason がありません。
    CatalogedReasonMissing,
}

impl InternalControlPlaneAuditGuard {
    /// internal_control_plane_decision audit event の必須 field を検査します。
    pub const fn try_new(
        event_type: InternalControlPlaneAuditEventType,
        outcome: InternalControlPlaneOutcome,
        correlation_id_declared: bool,
        outcome_declared: bool,
        source_service_declared: bool,
        target_service_declared: bool,
        control_plane_class_declared: bool,
        contract_version_declared: bool,
        topology_class_declared_when_available: bool,
        cataloged_reason_declared_for_rejected_expired_failed_outcome: bool,
    ) -> Result<Self, InternalControlPlaneAuditError> {
        if !correlation_id_declared {
            return Err(InternalControlPlaneAuditError::CorrelationIdMissing);
        }
        if !outcome_declared {
            return Err(InternalControlPlaneAuditError::OutcomeMissing);
        }
        if !source_service_declared {
            return Err(InternalControlPlaneAuditError::SourceServiceMissing);
        }
        if !target_service_declared {
            return Err(InternalControlPlaneAuditError::TargetServiceMissing);
        }
        if !control_plane_class_declared {
            return Err(InternalControlPlaneAuditError::ControlPlaneClassMissing);
        }
        if !contract_version_declared {
            return Err(InternalControlPlaneAuditError::ContractVersionMissing);
        }
        if !topology_class_declared_when_available {
            return Err(InternalControlPlaneAuditError::TopologyClassMissing);
        }
        if outcome.requires_reason()
            && !cataloged_reason_declared_for_rejected_expired_failed_outcome
        {
            return Err(InternalControlPlaneAuditError::CatalogedReasonMissing);
        }

        Ok(Self {
            event_type,
            outcome,
            correlation_id_declared,
            outcome_declared,
            source_service_declared,
            target_service_declared,
            control_plane_class_declared,
            contract_version_declared,
            topology_class_declared_when_available,
            cataloged_reason_declared_for_rejected_expired_failed_outcome,
        })
    }
}

/// internal control-plane runtime/audit outcome の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalControlPlaneOutcome {
    /// expected decision succeeded.
    Success,
    /// internal control-plane decision rejected the call.
    Rejected,
    /// deadline, credential, endpoint, or freshness condition expired.
    Expired,
    /// internal control-plane processing failed.
    Failed,
}

impl InternalControlPlaneOutcome {
    /// 非成功 outcome では cataloged reason を必須にします。
    pub const fn requires_reason(self) -> bool {
        matches!(self, Self::Rejected | Self::Expired | Self::Failed)
    }
}
