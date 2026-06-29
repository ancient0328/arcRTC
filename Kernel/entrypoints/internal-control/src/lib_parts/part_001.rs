// entrypoints/internal-control は internal service identity と control-plane wiring の surface です。
//
// service-to-service の契約や認可の意味論をここで再定義せず、許可された
// core/drivers 境界を接続する entrypoint-owned surface とします。

use arcrtc_core_reason::CatalogedReasonRef;

/// entrypoint internal-control package の所有境界を示す marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntrypointInternalControlSurface;

/// v0.2 initial architecture の internal service trust class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalServiceTrustClass {
    /// in-process call where no network peer exists.
    ServiceIdentityNotRequired,
    /// startup config binds source/target service identity.
    StaticConfiguredServiceIdentity,
    /// mTLS peer certificate or equivalent peer proof is observed.
    MtlsPeerIdentity,
    /// signed service credential is verified.
    SignedServiceTokenIdentity,
    /// service mesh or sidecar asserts service identity.
    MeshAssertedServiceIdentity,
    /// deterministic fake identity for tests.
    TestServiceIdentity,
    /// internal service call has no admitted identity proof.
    UnauthenticatedInternalServiceRequested,
}

impl InternalServiceTrustClass {
    /// credential/freshness rule が必要な trust class です。
    pub const fn is_credential_bearing(self) -> bool {
        matches!(
            self,
            Self::MtlsPeerIdentity
                | Self::SignedServiceTokenIdentity
                | Self::MeshAssertedServiceIdentity
        )
    }

    /// test evidence にだけ閉じる trust class です。
    pub const fn is_test_only(self) -> bool {
        matches!(self, Self::TestServiceIdentity)
    }

    /// v0.2 では常に rejected になる request class です。
    pub const fn is_rejected_request_class(self) -> bool {
        matches!(self, Self::UnauthenticatedInternalServiceRequested)
    }

    /// trust class と proof/reference class の対応を固定します。
    pub const fn admits_proof_reference(self, proof: ServiceIdentityProofReferenceClass) -> bool {
        matches!(
            (self, proof),
            (
                Self::ServiceIdentityNotRequired,
                ServiceIdentityProofReferenceClass::NoCredentialInProcess
            ) | (
                Self::StaticConfiguredServiceIdentity,
                ServiceIdentityProofReferenceClass::StaticConfigIdentityReference
            ) | (
                Self::MtlsPeerIdentity,
                ServiceIdentityProofReferenceClass::PeerCertificateReference
            ) | (
                Self::SignedServiceTokenIdentity,
                ServiceIdentityProofReferenceClass::SignedServiceTokenReference
            ) | (
                Self::MeshAssertedServiceIdentity,
                ServiceIdentityProofReferenceClass::MeshAssertionReference
            ) | (
                Self::TestServiceIdentity,
                ServiceIdentityProofReferenceClass::TestIdentityReference
            ) | (
                Self::UnauthenticatedInternalServiceRequested,
                ServiceIdentityProofReferenceClass::NoCredentialInProcess
            )
        )
    }
}

/// service identity proof/reference class の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceIdentityProofReferenceClass {
    /// in-process path with no network credential.
    NoCredentialInProcess,
    /// startup configuration identity reference.
    StaticConfigIdentityReference,
    /// mTLS/peer certificate opaque reference.
    PeerCertificateReference,
    /// signed service token opaque reference.
    SignedServiceTokenReference,
    /// mesh assertion opaque reference.
    MeshAssertionReference,
    /// deterministic fake identity reference.
    TestIdentityReference,
}

/// internal service role の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalServiceRole {
    /// Signaling service.
    Signaling,
    /// SFU service.
    Sfu,
    /// TURN service.
    Turn,
    /// internal control service.
    InternalControl,
    /// admin service.
    Admin,
    /// topology/configuration service.
    TopologyConfiguration,
}

/// internal service trust audit event type の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalServiceTrustAuditEventType {
    /// internal service trust decision.
    InternalServiceTrustDecision,
}

impl InternalServiceTrustAuditEventType {
    /// audit event catalog に接続する event type です。
    pub const fn event_type(self) -> &'static str {
        match self {
            Self::InternalServiceTrustDecision => "internal_service_trust_decision",
        }
    }
}

/// internal service trust decision の audit outcome 閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalServiceTrustDecisionOutcome {
    /// trust decision が受理されました。
    Accepted,
    /// trust decision が拒否されました。
    Rejected,
    /// credential / proof freshness が失効しました。
    Expired,
    /// trust decision の評価に失敗しました。
    Failed,
}

impl InternalServiceTrustDecisionOutcome {
    /// 非成功 outcome では cataloged reason を必須にします。
    pub const fn requires_reason(self) -> bool {
        matches!(self, Self::Rejected | Self::Expired | Self::Failed)
    }
}

/// internal service identity mapping guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InternalServiceIdentityMappingGuard {
    trust_class: InternalServiceTrustClass,
    proof_reference_class: ServiceIdentityProofReferenceClass,
    source_service: InternalServiceRole,
    target_service: InternalServiceRole,
    trust_class_declared: bool,
    source_service_declared: bool,
    target_service_declared: bool,
    topology_class_declared: bool,
    credential_or_peer_proof_reference_class_declared: bool,
    trust_anchor_or_verifier_reference_declared: bool,
    accepted_scope_declared: bool,
    contract_version_relation_declared: bool,
    lifetime_expiry_rule_declared: bool,
    replay_or_freshness_rule_declared_when_credential_bearing: bool,
    service_discovery_endpoint_scope_relation_declared: bool,
    internal_control_authorization_context_relation_declared: bool,
    audit_shape_declared: bool,
    raw_credential_or_peer_material_absent_from_core_state_audit_log_report: bool,
    sdk_public_surface_not_present_or_raw_material_absent: bool,
    opaque_reference_or_redacted_summary_only: bool,
    test_identity_evidence_is_test_only: bool,
    in_process_identity_evidence_only_when_identity_not_required: bool,
}

/// internal service identity mapping の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalServiceIdentityMappingError {
    /// trust class がありません。
    TrustClassMissing,
    /// trust class と proof/reference class が一致していません。
    ProofReferenceClassMismatch,
    /// unauthenticated internal service request は許可されません。
    UnauthenticatedInternalServiceNotAdmitted,
    /// source service がありません。
    SourceServiceMissing,
    /// target service がありません。
    TargetServiceMissing,
    /// topology class がありません。
    TopologyClassMissing,
    /// credential/peer proof reference class がありません。
    ProofReferenceClassMissing,
    /// trust anchor/verifier reference がありません。
    TrustAnchorVerifierReferenceMissing,
    /// accepted scope がありません。
    AcceptedScopeMissing,
    /// contract version relation がありません。
    ContractVersionRelationMissing,
    /// lifetime/expiry rule がありません。
    LifetimeExpiryRuleMissing,
    /// credential-bearing trust class の replay/freshness rule がありません。
    ReplayFreshnessRuleMissing,
    /// service discovery endpoint scope relation がありません。
    ServiceDiscoveryEndpointScopeRelationMissing,
    /// internal control authorization context relation がありません。
    InternalControlAuthorizationContextRelationMissing,
    /// audit shape がありません。
    AuditShapeMissing,
    /// raw certificate/private key/service token/mesh assertion/secret が core state/audit body/log/SDK surface/report に入っています。
    RawCredentialMaterialEscaped,
    /// opaque reference/redacted summary 以外が evidence に入っています。
    NonOpaqueOrUnredactedEvidenceMaterial,
    /// test identity が test evidence の外で使われています。
    TestIdentityUsedOutsideTestEvidence,
    /// service_identity_not_required が in-process evidence の外で使われています。
    IdentityNotRequiredUsedOutsideInProcessEvidence,
}

impl InternalServiceIdentityMappingGuard {
    /// raw proof material を opaque identity context に写す前提条件を検査します。
    pub const fn try_new(
        trust_class: InternalServiceTrustClass,
        proof_reference_class: ServiceIdentityProofReferenceClass,
        source_service: InternalServiceRole,
        target_service: InternalServiceRole,
        trust_class_declared: bool,
        source_service_declared: bool,
        target_service_declared: bool,
        topology_class_declared: bool,
        credential_or_peer_proof_reference_class_declared: bool,
        trust_anchor_or_verifier_reference_declared: bool,
        accepted_scope_declared: bool,
        contract_version_relation_declared: bool,
        lifetime_expiry_rule_declared: bool,
        replay_or_freshness_rule_declared_when_credential_bearing: bool,
        service_discovery_endpoint_scope_relation_declared: bool,
        internal_control_authorization_context_relation_declared: bool,
        audit_shape_declared: bool,
        raw_credential_or_peer_material_absent_from_core_state_audit_log_report: bool,
        sdk_public_surface_not_present_or_raw_material_absent: bool,
        opaque_reference_or_redacted_summary_only: bool,
        test_identity_evidence_is_test_only: bool,
        in_process_identity_evidence_only_when_identity_not_required: bool,
    ) -> Result<Self, InternalServiceIdentityMappingError> {
        if !trust_class_declared {
            return Err(InternalServiceIdentityMappingError::TrustClassMissing);
        }
        if !trust_class.admits_proof_reference(proof_reference_class) {
            return Err(InternalServiceIdentityMappingError::ProofReferenceClassMismatch);
        }
        if trust_class.is_rejected_request_class() {
            return Err(
                InternalServiceIdentityMappingError::UnauthenticatedInternalServiceNotAdmitted,
            );
        }
        if !source_service_declared {
            return Err(InternalServiceIdentityMappingError::SourceServiceMissing);
        }
        if !target_service_declared {
            return Err(InternalServiceIdentityMappingError::TargetServiceMissing);
        }
        if !topology_class_declared {
            return Err(InternalServiceIdentityMappingError::TopologyClassMissing);
        }
        if !credential_or_peer_proof_reference_class_declared {
            return Err(InternalServiceIdentityMappingError::ProofReferenceClassMissing);
        }
        if !trust_anchor_or_verifier_reference_declared {
            return Err(InternalServiceIdentityMappingError::TrustAnchorVerifierReferenceMissing);
        }
        if !accepted_scope_declared {
            return Err(InternalServiceIdentityMappingError::AcceptedScopeMissing);
        }
        if !contract_version_relation_declared {
            return Err(InternalServiceIdentityMappingError::ContractVersionRelationMissing);
        }
        if !lifetime_expiry_rule_declared {
            return Err(InternalServiceIdentityMappingError::LifetimeExpiryRuleMissing);
        }
        if trust_class.is_credential_bearing()
            && !replay_or_freshness_rule_declared_when_credential_bearing
        {
            return Err(InternalServiceIdentityMappingError::ReplayFreshnessRuleMissing);
        }
        if !service_discovery_endpoint_scope_relation_declared {
            return Err(
                InternalServiceIdentityMappingError::ServiceDiscoveryEndpointScopeRelationMissing,
            );
        }
        if !internal_control_authorization_context_relation_declared {
            return Err(
                InternalServiceIdentityMappingError::InternalControlAuthorizationContextRelationMissing,
            );
        }
        if !audit_shape_declared {
            return Err(InternalServiceIdentityMappingError::AuditShapeMissing);
        }
        if !raw_credential_or_peer_material_absent_from_core_state_audit_log_report
            || !sdk_public_surface_not_present_or_raw_material_absent
        {
            return Err(InternalServiceIdentityMappingError::RawCredentialMaterialEscaped);
        }
        if !opaque_reference_or_redacted_summary_only {
            return Err(InternalServiceIdentityMappingError::NonOpaqueOrUnredactedEvidenceMaterial);
        }
        if trust_class.is_test_only() && !test_identity_evidence_is_test_only {
            return Err(InternalServiceIdentityMappingError::TestIdentityUsedOutsideTestEvidence);
        }
        if matches!(
            trust_class,
            InternalServiceTrustClass::ServiceIdentityNotRequired
        ) && !in_process_identity_evidence_only_when_identity_not_required
        {
            return Err(
                InternalServiceIdentityMappingError::IdentityNotRequiredUsedOutsideInProcessEvidence,
            );
        }

        Ok(Self {
            trust_class,
            proof_reference_class,
            source_service,
            target_service,
            trust_class_declared,
            source_service_declared,
            target_service_declared,
            topology_class_declared,
            credential_or_peer_proof_reference_class_declared,
            trust_anchor_or_verifier_reference_declared,
            accepted_scope_declared,
            contract_version_relation_declared,
            lifetime_expiry_rule_declared,
            replay_or_freshness_rule_declared_when_credential_bearing,
            service_discovery_endpoint_scope_relation_declared,
            internal_control_authorization_context_relation_declared,
            audit_shape_declared,
            raw_credential_or_peer_material_absent_from_core_state_audit_log_report,
            sdk_public_surface_not_present_or_raw_material_absent,
            opaque_reference_or_redacted_summary_only,
            test_identity_evidence_is_test_only,
            in_process_identity_evidence_only_when_identity_not_required,
        })
    }
}

/// internal control authorization sequence guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InternalServiceAuthorizationSequenceGuard {
    service_discovery_resolved_endpoint_within_scope: bool,
    transport_security_observed_peer_proof_when_required: bool,
    identity_mapping_produced_admitted_opaque_context: bool,
    internal_control_contract_validates_required_fields: bool,
    target_core_use_case_keeps_domain_decision_ownership: bool,
    earlier_step_failure_not_reclassified_as_later_success: bool,
}

/// authorization sequence の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalServiceAuthorizationSequenceError {
    /// endpoint resolution scope が成立していません。
    EndpointScopeNotResolved,
    /// required peer proof が観測されていません。
    PeerProofMissing,
    /// admitted opaque identity context がありません。
    OpaqueIdentityContextMissing,
    /// internal control contract required fields が検査されていません。
    InternalControlContractValidationMissing,
    /// target core use case の domain decision 所有が崩れています。
    DomainDecisionOwnershipLost,
    /// earlier-step failure を later-step success に再分類しています。
    EarlierFailureReclassifiedAsSuccess,
}

impl InternalServiceAuthorizationSequenceGuard {
    /// service identity が internal control authorization を置換しないことを検査します。
    pub const fn try_new(
        service_discovery_resolved_endpoint_within_scope: bool,
        transport_security_observed_peer_proof_when_required: bool,
        identity_mapping_produced_admitted_opaque_context: bool,
        internal_control_contract_validates_required_fields: bool,
        target_core_use_case_keeps_domain_decision_ownership: bool,
        earlier_step_failure_not_reclassified_as_later_success: bool,
    ) -> Result<Self, InternalServiceAuthorizationSequenceError> {
        if !service_discovery_resolved_endpoint_within_scope {
            return Err(InternalServiceAuthorizationSequenceError::EndpointScopeNotResolved);
        }
        if !transport_security_observed_peer_proof_when_required {
            return Err(InternalServiceAuthorizationSequenceError::PeerProofMissing);
        }
        if !identity_mapping_produced_admitted_opaque_context {
            return Err(InternalServiceAuthorizationSequenceError::OpaqueIdentityContextMissing);
        }
        if !internal_control_contract_validates_required_fields {
            return Err(
                InternalServiceAuthorizationSequenceError::InternalControlContractValidationMissing,
            );
        }
        if !target_core_use_case_keeps_domain_decision_ownership {
            return Err(InternalServiceAuthorizationSequenceError::DomainDecisionOwnershipLost);
        }
        if !earlier_step_failure_not_reclassified_as_later_success {
            return Err(
                InternalServiceAuthorizationSequenceError::EarlierFailureReclassifiedAsSuccess,
            );
        }

        Ok(Self {
            service_discovery_resolved_endpoint_within_scope,
            transport_security_observed_peer_proof_when_required,
            identity_mapping_produced_admitted_opaque_context,
            internal_control_contract_validates_required_fields,
            target_core_use_case_keeps_domain_decision_ownership,
            earlier_step_failure_not_reclassified_as_later_success,
        })
    }
}

/// internal service trust audit shape guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InternalServiceTrustAuditGuard {
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
}

/// internal service trust audit shape の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalServiceTrustAuditError {
    /// StartupRunId がありません。
    StartupRunIdMissing,
    /// decision outcome がありません。
    OutcomeMissing,
    /// trust class がありません。
    TrustClassMissing,
    /// source service がありません。
    SourceServiceMissing,
    /// target service がありません。
    TargetServiceMissing,
    /// topology class がありません。
    TopologyClassMissing,
    /// credential/peer proof reference class がありません。
    ProofReferenceClassMissing,
    /// trust policy reference がありません。
    TrustPolicyReferenceMissing,
    /// endpoint scope がありません。
    EndpointScopeMissing,
    /// command-scoped decision の CorrelationId rule がありません。
    CorrelationRuleMissing,
    /// non-success outcome の cataloged reason がありません。
    CatalogedReasonMissing,
}

