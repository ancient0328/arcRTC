impl InternalServiceTrustAuditGuard {
    /// internal_service_trust_decision audit event の必須 field を検査します。
    pub const fn try_new(
        event_type: InternalServiceTrustAuditEventType,
        outcome: InternalServiceTrustDecisionOutcome,
        startup_run_id_declared: bool,
        outcome_declared: bool,
        trust_class_declared: bool,
        source_service_declared: bool,
        target_service_declared: bool,
        topology_class_declared: bool,
        credential_peer_proof_reference_class_declared: bool,
        trust_policy_reference_declared: bool,
        endpoint_scope_declared_when_resolution_involved: bool,
        correlation_rule_declared_when_command_scoped: bool,
        cataloged_reason_declared_for_rejected_expired_failed_outcome: bool,
    ) -> Result<Self, InternalServiceTrustAuditError> {
        if !startup_run_id_declared {
            return Err(InternalServiceTrustAuditError::StartupRunIdMissing);
        }
        if !outcome_declared {
            return Err(InternalServiceTrustAuditError::OutcomeMissing);
        }
        if !trust_class_declared {
            return Err(InternalServiceTrustAuditError::TrustClassMissing);
        }
        if !source_service_declared {
            return Err(InternalServiceTrustAuditError::SourceServiceMissing);
        }
        if !target_service_declared {
            return Err(InternalServiceTrustAuditError::TargetServiceMissing);
        }
        if !topology_class_declared {
            return Err(InternalServiceTrustAuditError::TopologyClassMissing);
        }
        if !credential_peer_proof_reference_class_declared {
            return Err(InternalServiceTrustAuditError::ProofReferenceClassMissing);
        }
        if !trust_policy_reference_declared {
            return Err(InternalServiceTrustAuditError::TrustPolicyReferenceMissing);
        }
        if !endpoint_scope_declared_when_resolution_involved {
            return Err(InternalServiceTrustAuditError::EndpointScopeMissing);
        }
        if !correlation_rule_declared_when_command_scoped {
            return Err(InternalServiceTrustAuditError::CorrelationRuleMissing);
        }
        if outcome.requires_reason()
            && !cataloged_reason_declared_for_rejected_expired_failed_outcome
        {
            return Err(InternalServiceTrustAuditError::CatalogedReasonMissing);
        }

        Ok(Self {
            event_type,
            outcome,
            startup_run_id_declared,
            outcome_declared,
            trust_class_declared,
            source_service_declared,
            target_service_declared,
            topology_class_declared,
            credential_peer_proof_reference_class_declared,
            trust_policy_reference_declared,
            endpoint_scope_declared_when_resolution_involved,
            correlation_rule_declared_when_command_scoped,
            cataloged_reason_declared_for_rejected_expired_failed_outcome,
        })
    }
}

/// internal service trust failure mapping の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalServiceTrustFailureKind {
    /// trust class is not admitted.
    InternalServiceIdentitySourceNotAdmitted,
    /// required service identity is absent.
    InternalServiceIdentityMissing,
    /// service identity material cannot be mapped.
    InternalServiceIdentityInvalid,
    /// service identity cannot be trusted for target path.
    InternalServiceIdentityUntrusted,
    /// identity scope does not match target service/topology/contract.
    InternalServiceIdentityScopeConflict,
    /// peer verification failed for internal service trust.
    InternalServicePeerVerificationFailed,
    /// service credential or peer proof expired.
    InternalServiceCredentialExpired,
    /// required service trust policy is absent.
    InternalServiceTrustPolicyMissing,
    /// endpoint resolution scope conflicts before trust mapping.
    ServiceEndpointScopeConflict,
    /// internal control authorization context is absent after trust mapping.
    InternalControlAuthorizationMissing,
    /// internal control authorization denies the call.
    InternalControlAuthorizationDenied,
}

impl InternalServiceTrustFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
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
            Self::InternalServiceCredentialExpired => "internal_service_credential_expired",
            Self::InternalServiceTrustPolicyMissing => "internal_service_trust_policy_missing",
            Self::ServiceEndpointScopeConflict => "service_endpoint_scope_conflict",
            Self::InternalControlAuthorizationMissing => "internal_control_authorization_missing",
            Self::InternalControlAuthorizationDenied => "internal_control_authorization_denied",
        }
    }
}

/// internal service trust failure です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InternalServiceTrustFailure {
    kind: InternalServiceTrustFailureKind,
    reason: CatalogedReasonRef,
}

impl InternalServiceTrustFailure {
    /// internal service trust failure を cataloged reason に接続します。
    pub fn from_kind(kind: InternalServiceTrustFailureKind) -> Self {
        let reason = CatalogedReasonRef::from_code(kind.reason_code())
            .expect("internal service trust reason code must be registered");
        Self { kind, reason }
    }
}

/// internal service trust 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedInternalServiceTrustBehavior {
    /// service discovery endpoint success is treated as trusted service identity.
    EndpointResolutionSuccessTreatedAsTrustedServiceIdentity,
    /// TLS/mTLS listener startup is treated as peer trust success.
    TlsListenerStartupTreatedAsPeerTrustSuccess,
    /// peer verification success is treated as internal control authorization success.
    PeerVerificationTreatedAsAuthorizationSuccess,
    /// mesh policy name or route name becomes core service identity.
    MeshPolicyNameBecomesCoreServiceIdentity,
    /// public endpoint credential is reused as internal service identity.
    PublicEndpointCredentialReusedAsInternalServiceIdentity,
    /// raw service token/certificate/private key/mesh assertion/secret appears in core state/audit body/log/SDK surface/output.
    RawServiceCredentialMaterialEscapes,
    /// in-process identity is reused as networked service trust.
    InProcessIdentityReusedAsNetworkedTrust,
    /// internal service trust failure is recorded as free-text only.
    TrustFailureRecordedAsFreeTextOnly,
}

/// v0.2 initial architecture の internal control-plane class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalControlPlaneClass {
    /// single process composition calls core use case directly.
    InProcessPlaneCall,
    /// split process same-host internal control.
    SameHostPlaneCall,
    /// split process networked internal control.
    NetworkedPlaneCall,
    /// command must reach node owning state.
    NodeAffinityPlaneCall,
    /// operator/admin invokes control path.
    AdminPlaneCall,
}

impl InternalControlPlaneClass {
    /// endpoint と authorization context が contract 上必要になる class です。
    pub const fn requires_endpoint_and_auth_context(self) -> bool {
        matches!(
            self,
            Self::SameHostPlaneCall | Self::NetworkedPlaneCall | Self::NodeAffinityPlaneCall
        )
    }

    /// service discovery / endpoint resolution relation が必要になる class です。
    pub const fn requires_service_discovery_relation(self) -> bool {
        matches!(self, Self::NetworkedPlaneCall | Self::NodeAffinityPlaneCall)
    }

    /// internal service trust class が必要になる class です。
    pub const fn requires_internal_service_trust_context(self) -> bool {
        matches!(
            self,
            Self::SameHostPlaneCall | Self::NetworkedPlaneCall | Self::NodeAffinityPlaneCall
        )
    }

    /// node affinity key と unavailable behavior が必要になる class です。
    pub const fn requires_node_affinity_rule(self) -> bool {
        matches!(self, Self::NodeAffinityPlaneCall)
    }

    /// operator/admin authorization policy への relation が必要になる class です。
    pub const fn requires_admin_authorization_relation(self) -> bool {
        matches!(self, Self::AdminPlaneCall)
    }
}

/// internal control contract の command/event class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalControlMessageClass {
    /// internal command.
    Command,
    /// internal event.
    Event,
}

/// core-owned/admitted な internal control command/event type reference です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalControlCommandEventType {
    /// Signaling core command contract reference.
    SignalingCommandContractReference,
    /// SFU core command contract reference.
    SfuCommandContractReference,
    /// TURN core command contract reference.
    TurnCommandContractReference,
    /// cross-plane shutdown/drain command contract reference.
    CrossPlaneShutdownDrainCommandReference,
    /// topology or node-affinity command contract reference.
    TopologyNodeAffinityCommandReference,
    /// admin or maintenance command contract reference.
    AdminMaintenanceCommandReference,
    /// internal control-plane decision event reference.
    InternalControlPlaneDecisionEventReference,
    /// service discovery / endpoint resolution event reference.
    ServiceDiscoveryResolutionEventReference,
    /// distributed state ownership/failover event reference.
    DistributedStateOwnershipEventReference,
}

impl InternalControlCommandEventType {
    /// type reference が属する command/event class です。
    pub const fn message_class(self) -> InternalControlMessageClass {
        match self {
            Self::SignalingCommandContractReference
            | Self::SfuCommandContractReference
            | Self::TurnCommandContractReference
            | Self::CrossPlaneShutdownDrainCommandReference
            | Self::TopologyNodeAffinityCommandReference
            | Self::AdminMaintenanceCommandReference => InternalControlMessageClass::Command,
            Self::InternalControlPlaneDecisionEventReference
            | Self::ServiceDiscoveryResolutionEventReference
            | Self::DistributedStateOwnershipEventReference => InternalControlMessageClass::Event,
        }
    }

    /// message class と admitted type reference の対応を固定します。
    pub const fn admits_message_class(self, message_class: InternalControlMessageClass) -> bool {
        matches!(
            (self.message_class(), message_class),
            (
                InternalControlMessageClass::Command,
                InternalControlMessageClass::Command
            ) | (
                InternalControlMessageClass::Event,
                InternalControlMessageClass::Event
            )
        )
    }
}

/// internal control contract version の admission state です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalControlContractVersionState {
    /// declared and admitted contract version.
    SupportedDeclared,
    /// declared but unsupported contract version.
    UnsupportedRequested,
    /// contract version is absent.
    Missing,
}

impl InternalControlContractVersionState {
    /// contract version が宣言されている状態です。
    pub const fn is_declared(self) -> bool {
        !matches!(self, Self::Missing)
    }

    /// contract version が採用可能な状態です。
    pub const fn is_supported(self) -> bool {
        matches!(self, Self::SupportedDeclared)
    }
}

/// internal control authorization context の contract class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalControlAuthorizationContextClass {
    /// service credential was mapped to an opaque authorization context.
    ServiceAuthorizationContext,
    /// operator/admin credential was mapped to an opaque authorization context.
    OperatorAuthorizationContext,
    /// entrypoint-owned credential was mapped to an opaque authorization context.
    ApplicationAuthorizationContext,
    /// authorization context is absent.
    MissingAuthorizationContextRequested,
}

impl InternalControlAuthorizationContextClass {
    /// authorization context 欠落要求は contract admission で拒否します。
    pub const fn is_missing_requested(self) -> bool {
        matches!(self, Self::MissingAuthorizationContextRequested)
    }
}

/// internal control-plane contract shape guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InternalControlPlaneContractGuard {
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
}
