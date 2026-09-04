impl ServiceDiscoveryResolutionAdmissionGuard {
    /// endpoint resolution policy の必須 field と責務分離を検査します。
    pub const fn try_new(
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
        test_resolver_is_test_only: bool,
    ) -> Result<Self, ServiceDiscoveryResolutionAdmissionError> {
        if !discovery_source_class_declared {
            return Err(ServiceDiscoveryResolutionAdmissionError::DiscoverySourceClassMissing);
        }
        if !discovery_source_class.admits_topology(topology_class) {
            return Err(ServiceDiscoveryResolutionAdmissionError::DiscoverySourceTopologyMismatch);
        }
        if !topology_class_declared {
            return Err(ServiceDiscoveryResolutionAdmissionError::TopologyClassMissing);
        }
        if !target_service_declared {
            return Err(ServiceDiscoveryResolutionAdmissionError::TargetServiceMissing);
        }
        if !expected_endpoint_scope_declared {
            return Err(ServiceDiscoveryResolutionAdmissionError::EndpointScopeMissing);
        }
        if !public_internal_endpoint_relation_declared {
            return Err(
                ServiceDiscoveryResolutionAdmissionError::PublicInternalEndpointRelationMissing,
            );
        }
        if !endpoint_contract_reference_declared_when_applicable {
            return Err(ServiceDiscoveryResolutionAdmissionError::EndpointContractReferenceMissing);
        }
        if !ttl_cache_staleness_rule_declared {
            return Err(ServiceDiscoveryResolutionAdmissionError::TtlCacheStalenessRuleMissing);
        }
        if !fallback_behavior_declared {
            return Err(ServiceDiscoveryResolutionAdmissionError::FallbackBehaviorMissing);
        }
        if !authorization_context_relation_declared {
            return Err(
                ServiceDiscoveryResolutionAdmissionError::AuthorizationContextRelationMissing,
            );
        }
        if (endpoint_scope.requires_internal_service_trust_relation()
            || target_service.requires_internal_service_trust_relation())
            && !service_identity_trust_relation_declared_when_internal_target
        {
            return Err(
                ServiceDiscoveryResolutionAdmissionError::ServiceIdentityTrustRelationMissing,
            );
        }
        if !readiness_relation_declared {
            return Err(ServiceDiscoveryResolutionAdmissionError::ReadinessRelationMissing);
        }
        if !audit_shape_declared {
            return Err(ServiceDiscoveryResolutionAdmissionError::AuditShapeMissing);
        }
        if discovery_source_class.requires_registry_contract()
            && !registry_contract_declared_when_required
        {
            return Err(ServiceDiscoveryResolutionAdmissionError::RegistryContractMissing);
        }
        if discovery_source_class.is_mesh_resolution() && !mesh_policy_not_used_as_authorization {
            return Err(ServiceDiscoveryResolutionAdmissionError::MeshPolicyUsedAsAuthorization);
        }
        if discovery_source_class.is_test_only() && !test_resolver_is_test_only {
            return Err(
                ServiceDiscoveryResolutionAdmissionError::TestResolverUsedOutsideTestScope,
            );
        }

        Ok(Self {
            discovery_source_class,
            topology_class,
            target_service,
            endpoint_scope,
            discovery_source_class_declared,
            topology_class_declared,
            target_service_declared,
            expected_endpoint_scope_declared,
            public_internal_endpoint_relation_declared,
            endpoint_contract_reference_declared_when_applicable,
            ttl_cache_staleness_rule_declared,
            fallback_behavior_declared,
            authorization_context_relation_declared,
            service_identity_trust_relation_declared_when_internal_target,
            readiness_relation_declared,
            audit_shape_declared,
            registry_contract_declared_when_required,
            mesh_policy_not_used_as_authorization,
            test_resolver_is_test_only,
        })
    }
}

/// fallback endpoint policy guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EndpointFallbackPolicyGuard {
    fallback_source_class_declared: bool,
    accepted_target_scope_declared: bool,
    stale_endpoint_rejection_rule_declared: bool,
    public_internal_separation_rule_declared: bool,
    audit_reason_for_primary_failure_declared: bool,
    fallback_scope_limitation_declared: bool,
    unverified_endpoint_fails_closed: bool,
}

/// fallback endpoint policy の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EndpointFallbackPolicyError {
    /// fallback source class がありません。
    FallbackSourceClassMissing,
    /// accepted target scope がありません。
    AcceptedTargetScopeMissing,
    /// stale endpoint rejection rule がありません。
    StaleEndpointRejectionRuleMissing,
    /// public/internal separation rule がありません。
    PublicInternalSeparationRuleMissing,
    /// primary failure の audit reason がありません。
    PrimaryFailureAuditReasonMissing,
    /// fallback scope limitation がありません。
    FallbackScopeLimitationMissing,
    /// unverified endpoint が fail-closed ではありません。
    UnverifiedEndpointNotFailClosed,
}

impl EndpointFallbackPolicyGuard {
    /// fallback endpoint の採用条件を検査します。
    pub const fn try_new(
        fallback_source_class_declared: bool,
        accepted_target_scope_declared: bool,
        stale_endpoint_rejection_rule_declared: bool,
        public_internal_separation_rule_declared: bool,
        audit_reason_for_primary_failure_declared: bool,
        fallback_scope_limitation_declared: bool,
        unverified_endpoint_fails_closed: bool,
    ) -> Result<Self, EndpointFallbackPolicyError> {
        if !fallback_source_class_declared {
            return Err(EndpointFallbackPolicyError::FallbackSourceClassMissing);
        }
        if !accepted_target_scope_declared {
            return Err(EndpointFallbackPolicyError::AcceptedTargetScopeMissing);
        }
        if !stale_endpoint_rejection_rule_declared {
            return Err(EndpointFallbackPolicyError::StaleEndpointRejectionRuleMissing);
        }
        if !public_internal_separation_rule_declared {
            return Err(EndpointFallbackPolicyError::PublicInternalSeparationRuleMissing);
        }
        if !audit_reason_for_primary_failure_declared {
            return Err(EndpointFallbackPolicyError::PrimaryFailureAuditReasonMissing);
        }
        if !fallback_scope_limitation_declared {
            return Err(EndpointFallbackPolicyError::FallbackScopeLimitationMissing);
        }
        if !unverified_endpoint_fails_closed {
            return Err(EndpointFallbackPolicyError::UnverifiedEndpointNotFailClosed);
        }

        Ok(Self {
            fallback_source_class_declared,
            accepted_target_scope_declared,
            stale_endpoint_rejection_rule_declared,
            public_internal_separation_rule_declared,
            audit_reason_for_primary_failure_declared,
            fallback_scope_limitation_declared,
            unverified_endpoint_fails_closed,
        })
    }
}

/// service discovery audit shape guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ServiceDiscoveryAuditGuard {
    event_type: ServiceDiscoveryAuditEventType,
    startup_run_id_declared: bool,
    correlation_rule_declared_when_command_scoped: bool,
    discovery_source_class_declared: bool,
    topology_class_declared: bool,
    target_service_declared: bool,
    endpoint_scope_declared: bool,
    resolution_state_declared: bool,
    fallback_class_declared_when_applicable: bool,
    cataloged_reason_declared_for_rejected_or_failed_outcome: bool,
}

/// service discovery audit shape の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceDiscoveryAuditError {
    /// StartupRunId がありません。
    StartupRunIdMissing,
    /// command-scoped decision の CorrelationId rule がありません。
    CorrelationRuleMissing,
    /// discovery source class がありません。
    DiscoverySourceClassMissing,
    /// topology class がありません。
    TopologyClassMissing,
    /// target service がありません。
    TargetServiceMissing,
    /// endpoint scope がありません。
    EndpointScopeMissing,
    /// resolution state がありません。
    ResolutionStateMissing,
    /// fallback class がありません。
    FallbackClassMissing,
    /// rejected/failed outcome の cataloged reason がありません。
    CatalogedReasonMissing,
}

impl ServiceDiscoveryAuditGuard {
    /// service_discovery_resolution_decision audit event の必須 field を検査します。
    pub const fn try_new(
        event_type: ServiceDiscoveryAuditEventType,
        resolution_state: EndpointResolutionState,
        startup_run_id_declared: bool,
        correlation_rule_declared_when_command_scoped: bool,
        discovery_source_class_declared: bool,
        topology_class_declared: bool,
        target_service_declared: bool,
        endpoint_scope_declared: bool,
        resolution_state_declared: bool,
        fallback_class_declared_when_applicable: bool,
        cataloged_reason_declared_for_rejected_or_failed_outcome: bool,
    ) -> Result<Self, ServiceDiscoveryAuditError> {
        if !startup_run_id_declared {
            return Err(ServiceDiscoveryAuditError::StartupRunIdMissing);
        }
        if !correlation_rule_declared_when_command_scoped {
            return Err(ServiceDiscoveryAuditError::CorrelationRuleMissing);
        }
        if !discovery_source_class_declared {
            return Err(ServiceDiscoveryAuditError::DiscoverySourceClassMissing);
        }
        if !topology_class_declared {
            return Err(ServiceDiscoveryAuditError::TopologyClassMissing);
        }
        if !target_service_declared {
            return Err(ServiceDiscoveryAuditError::TargetServiceMissing);
        }
        if !endpoint_scope_declared {
            return Err(ServiceDiscoveryAuditError::EndpointScopeMissing);
        }
        if !resolution_state_declared {
            return Err(ServiceDiscoveryAuditError::ResolutionStateMissing);
        }
        if !fallback_class_declared_when_applicable {
            return Err(ServiceDiscoveryAuditError::FallbackClassMissing);
        }
        if resolution_state.requires_reason()
            && !cataloged_reason_declared_for_rejected_or_failed_outcome
        {
            return Err(ServiceDiscoveryAuditError::CatalogedReasonMissing);
        }

        Ok(Self {
            event_type,
            startup_run_id_declared,
            correlation_rule_declared_when_command_scoped,
            discovery_source_class_declared,
            topology_class_declared,
            target_service_declared,
            endpoint_scope_declared,
            resolution_state_declared,
            fallback_class_declared_when_applicable,
            cataloged_reason_declared_for_rejected_or_failed_outcome,
        })
    }
}

/// service discovery / endpoint resolution failure mapping の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceDiscoveryResolutionFailureKind {
    /// discovery source class is not admitted.
    ServiceDiscoverySourceNotAdmitted,
    /// selected discovery source is unavailable.
    ServiceDiscoveryUnavailable,
    /// endpoint cannot be resolved for target service.
    ServiceEndpointResolutionFailed,
    /// cached endpoint is stale or generation-invalid.
    ServiceEndpointStale,
    /// fallback endpoint is not admitted.
    ServiceEndpointFallbackNotAllowed,
    /// resolved endpoint does not match expected scope.
    ServiceEndpointScopeConflict,
    /// resolved endpoint contract/version is not accepted.
    ServiceEndpointContractMismatch,
    /// resolver maps public/internal endpoint incorrectly.
    PublicInternalRouteConfusion,
    /// resolver output is used as service identity without trust policy.
    InternalServiceIdentityUntrusted,
}

impl ServiceDiscoveryResolutionFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::ServiceDiscoverySourceNotAdmitted => "service_discovery_source_not_admitted",
            Self::ServiceDiscoveryUnavailable => "service_discovery_unavailable",
            Self::ServiceEndpointResolutionFailed => "service_endpoint_resolution_failed",
            Self::ServiceEndpointStale => "service_endpoint_stale",
            Self::ServiceEndpointFallbackNotAllowed => "service_endpoint_fallback_not_allowed",
            Self::ServiceEndpointScopeConflict => "service_endpoint_scope_conflict",
            Self::ServiceEndpointContractMismatch => "service_endpoint_contract_mismatch",
            Self::PublicInternalRouteConfusion => "public_internal_route_confusion",
            Self::InternalServiceIdentityUntrusted => "internal_service_identity_untrusted",
        }
    }
}

/// service discovery / endpoint resolution failure です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ServiceDiscoveryResolutionFailure {
    kind: ServiceDiscoveryResolutionFailureKind,
    reason: CatalogedReasonRef,
}

impl ServiceDiscoveryResolutionFailure {
    /// service discovery failure を cataloged reason に接続します。
    pub fn from_kind(kind: ServiceDiscoveryResolutionFailureKind) -> Self {
        let reason = CatalogedReasonRef::from_code(kind.reason_code())
            .expect("service discovery reason code must be registered");
        Self { kind, reason }
    }
}

/// service discovery / endpoint resolution 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedServiceDiscoveryResolutionBehavior {
    /// DNS/registry/mesh resolution success is treated as readiness.
    ResolutionSuccessTreatedAsReadiness,
    /// service discovery owns domain accept/reject decision.
    ServiceDiscoveryOwnsDomainDecision,
    /// fallback endpoint is used without policy and audit reason.
    FallbackEndpointUsedWithoutPolicyAndAuditReason,
    /// stale cached endpoint is used as a fresh resolution.
    StaleCachedEndpointUsedAsFreshResolution,
    /// resolved internal endpoint is exposed as public endpoint by naming.
    ResolvedInternalEndpointExposedPublicByNaming,
    /// mesh policy or resolver status becomes application authorization by default.
    MeshPolicyBecomesApplicationAuthorization,
    /// resolved endpoint is treated as trusted service identity.
    ResolvedEndpointTreatedAsTrustedServiceIdentity,
    /// in-process resolution is reused as networked discovery.
    InProcessResolutionReusedAsNetworkedDiscovery,
}
