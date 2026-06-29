/// health/readiness/liveness evidence の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HealthAdminEvidenceError {
    /// command/probe endpoint がありません。
    CommandProbeEndpointMissing,
    /// working directory / target entrypoint がありません。
    WorkingDirectoryTargetEntrypointMissing,
    /// StartupRunId がありません。
    StartupRunIdMissing,
    /// command-scoped evidence の CorrelationId がありません。
    CorrelationIdMissing,
    /// probe class がありません。
    ProbeClassMissing,
    /// included/excluded checks がありません。
    IncludedExcludedChecksMissing,
    /// topology class / node scope がありません。
    TopologyNodeScopeMissing,
    /// service discovery source / resolution state がありません。
    ServiceDiscoveryResolutionMissing,
    /// distributed state class / failover status がありません。
    DistributedStateFailoverMissing,
    /// runtime task class / supervision state がありません。
    RuntimeTaskSupervisionMissing,
    /// internal service trust class がありません。
    InternalServiceTrustMissing,
    /// expected outcome がありません。
    ExpectedOutcomeMissing,
    /// actual outcome がありません。
    ActualOutcomeMissing,
    /// non-success の cataloged reason がありません。
    CatalogedReasonMissing,
    /// close-not-claimed scope がありません。
    CloseNotClaimedScopeMissing,
    /// required fields のない probe output を evidence として採用しています。
    DiagnosticProbeOutputAdoptedAsEvidence,
    /// probe success を build/test/runtime/production proof として扱っています。
    ProbeSuccessUsedAsExternalProof,
}

impl HealthAdminEvidenceGuard {
    /// health/readiness/liveness/admin evidence の採用条件を検査します。
    pub const fn try_new(
        probe_class: AdminProbeClass,
        outcome: HealthAdminOutcome,
        command_or_probe_endpoint_declared: bool,
        working_directory_or_target_entrypoint_declared: bool,
        startup_run_id_declared: bool,
        correlation_id_declared_when_command_scoped: bool,
        probe_class_declared: bool,
        included_excluded_checks_declared: bool,
        topology_node_scope_declared_when_relevant: bool,
        service_discovery_resolution_declared_when_affects_probe: bool,
        distributed_state_failover_declared_when_affects_probe: bool,
        runtime_task_supervision_declared_when_affects_probe: bool,
        internal_service_trust_declared_when_affects_probe: bool,
        expected_outcome_declared: bool,
        actual_outcome_declared: bool,
        cataloged_reason_declared_for_non_success: bool,
        close_not_claimed_scope_declared: bool,
        probe_output_without_required_fields_not_adopted_as_evidence: bool,
        probe_success_not_used_as_build_test_runtime_or_production_proof: bool,
    ) -> Result<Self, HealthAdminEvidenceError> {
        if !command_or_probe_endpoint_declared {
            return Err(HealthAdminEvidenceError::CommandProbeEndpointMissing);
        }
        if !working_directory_or_target_entrypoint_declared {
            return Err(HealthAdminEvidenceError::WorkingDirectoryTargetEntrypointMissing);
        }
        if !startup_run_id_declared {
            return Err(HealthAdminEvidenceError::StartupRunIdMissing);
        }
        if !correlation_id_declared_when_command_scoped {
            return Err(HealthAdminEvidenceError::CorrelationIdMissing);
        }
        if !probe_class_declared {
            return Err(HealthAdminEvidenceError::ProbeClassMissing);
        }
        if !included_excluded_checks_declared {
            return Err(HealthAdminEvidenceError::IncludedExcludedChecksMissing);
        }
        if !topology_node_scope_declared_when_relevant {
            return Err(HealthAdminEvidenceError::TopologyNodeScopeMissing);
        }
        if !service_discovery_resolution_declared_when_affects_probe {
            return Err(HealthAdminEvidenceError::ServiceDiscoveryResolutionMissing);
        }
        if !distributed_state_failover_declared_when_affects_probe {
            return Err(HealthAdminEvidenceError::DistributedStateFailoverMissing);
        }
        if !runtime_task_supervision_declared_when_affects_probe {
            return Err(HealthAdminEvidenceError::RuntimeTaskSupervisionMissing);
        }
        if !internal_service_trust_declared_when_affects_probe {
            return Err(HealthAdminEvidenceError::InternalServiceTrustMissing);
        }
        if !expected_outcome_declared {
            return Err(HealthAdminEvidenceError::ExpectedOutcomeMissing);
        }
        if !actual_outcome_declared {
            return Err(HealthAdminEvidenceError::ActualOutcomeMissing);
        }
        if outcome.requires_reason() && !cataloged_reason_declared_for_non_success {
            return Err(HealthAdminEvidenceError::CatalogedReasonMissing);
        }
        if !close_not_claimed_scope_declared {
            return Err(HealthAdminEvidenceError::CloseNotClaimedScopeMissing);
        }
        if !probe_output_without_required_fields_not_adopted_as_evidence {
            return Err(HealthAdminEvidenceError::DiagnosticProbeOutputAdoptedAsEvidence);
        }
        if !probe_success_not_used_as_build_test_runtime_or_production_proof {
            return Err(HealthAdminEvidenceError::ProbeSuccessUsedAsExternalProof);
        }

        Ok(Self {
            probe_class,
            outcome,
            command_or_probe_endpoint_declared,
            working_directory_or_target_entrypoint_declared,
            startup_run_id_declared,
            correlation_id_declared_when_command_scoped,
            probe_class_declared,
            included_excluded_checks_declared,
            topology_node_scope_declared_when_relevant,
            service_discovery_resolution_declared_when_affects_probe,
            distributed_state_failover_declared_when_affects_probe,
            runtime_task_supervision_declared_when_affects_probe,
            internal_service_trust_declared_when_affects_probe,
            expected_outcome_declared,
            actual_outcome_declared,
            cataloged_reason_declared_for_non_success,
            close_not_claimed_scope_declared,
            probe_output_without_required_fields_not_adopted_as_evidence,
            probe_success_not_used_as_build_test_runtime_or_production_proof,
        })
    }
}

/// health/readiness/liveness/admin failure mapping の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HealthAdminFailureKind {
    /// readiness component not satisfied.
    ReadinessNotSatisfied,
    /// health probe cannot execute due driver/entrypoint failure.
    HealthProbeUnavailable,
    /// action blocked by maintenance mode.
    MaintenanceModeActive,
    /// admin action not allowed by boundary/policy.
    AdminActionNotAllowed,
    /// operator/admin authorization denied.
    OperatorActionDenied,
    /// operator/admin authorization context missing.
    OperatorAuthorizationContextMissing,
    /// required runtime configuration missing.
    RuntimeConfigMissing,
    /// runtime configuration invalid.
    RuntimeConfigInvalid,
    /// driver shutdown during probe/action.
    DriverShutdown,
    /// service discovery unavailable.
    ServiceDiscoveryUnavailable,
    /// endpoint resolution stale.
    ServiceEndpointStale,
    /// fallback endpoint not admitted.
    ServiceEndpointFallbackNotAllowed,
    /// node-local state unavailable.
    NodeStateUnavailable,
    /// failover state not proven.
    FailoverNotProven,
    /// runtime task class is not admitted.
    RuntimeTaskClassNotAdmitted,
    /// runtime task owner/supervision scope is invalid.
    RuntimeTaskOwnerViolation,
    /// task has no admitted supervision scope.
    RuntimeTaskSupervisionMissing,
    /// detached task is requested.
    RuntimeTaskDetachedNotAllowed,
    /// runtime cannot spawn required task.
    RuntimeTaskSpawnFailed,
    /// task join/wait observation failed.
    RuntimeTaskJoinFailed,
    /// task cancellation failed or could not be observed.
    RuntimeTaskCancelFailed,
    /// task panic was observed.
    RuntimeTaskPanicDetected,
    /// task queue/mailbox/join bound exceeded.
    RuntimeTaskQueueBoundExceeded,
    /// internal service trust source not admitted.
    InternalServiceIdentitySourceNotAdmitted,
    /// required service identity is absent.
    InternalServiceIdentityMissing,
    /// service identity material cannot be mapped.
    InternalServiceIdentityInvalid,
    /// service identity cannot be trusted for target path.
    InternalServiceIdentityUntrusted,
    /// identity scope does not match target.
    InternalServiceIdentityScopeConflict,
    /// peer verification failed for internal service trust.
    InternalServicePeerVerificationFailed,
    /// service credential or peer proof expired.
    InternalServiceCredentialExpired,
    /// required service trust policy is absent.
    InternalServiceTrustPolicyMissing,
}

impl HealthAdminFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::ReadinessNotSatisfied => "readiness_not_satisfied",
            Self::HealthProbeUnavailable => "health_probe_unavailable",
            Self::MaintenanceModeActive => "maintenance_mode_active",
            Self::AdminActionNotAllowed => "admin_action_not_allowed",
            Self::OperatorActionDenied => "operator_action_denied",
            Self::OperatorAuthorizationContextMissing => "operator_authorization_context_missing",
            Self::RuntimeConfigMissing => "runtime_config_missing",
            Self::RuntimeConfigInvalid => "runtime_config_invalid",
            Self::DriverShutdown => "driver_shutdown",
            Self::ServiceDiscoveryUnavailable => "service_discovery_unavailable",
            Self::ServiceEndpointStale => "service_endpoint_stale",
            Self::ServiceEndpointFallbackNotAllowed => "service_endpoint_fallback_not_allowed",
            Self::NodeStateUnavailable => "node_state_unavailable",
            Self::FailoverNotProven => "failover_not_proven",
            Self::RuntimeTaskClassNotAdmitted => "runtime_task_class_not_admitted",
            Self::RuntimeTaskOwnerViolation => "runtime_task_owner_violation",
            Self::RuntimeTaskSupervisionMissing => "runtime_task_supervision_missing",
            Self::RuntimeTaskDetachedNotAllowed => "runtime_task_detached_not_allowed",
            Self::RuntimeTaskSpawnFailed => "runtime_task_spawn_failed",
            Self::RuntimeTaskJoinFailed => "runtime_task_join_failed",
            Self::RuntimeTaskCancelFailed => "runtime_task_cancel_failed",
            Self::RuntimeTaskPanicDetected => "runtime_task_panic_detected",
            Self::RuntimeTaskQueueBoundExceeded => "runtime_task_queue_bound_exceeded",
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
        }
    }
}

/// health/readiness/liveness/admin failure です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HealthAdminFailure {
    kind: HealthAdminFailureKind,
    reason: CatalogedReasonRef,
}

impl HealthAdminFailure {
    /// health/admin failure を cataloged reason に接続します。
    pub fn from_kind(kind: HealthAdminFailureKind) -> Self {
        let reason = CatalogedReasonRef::from_code(kind.reason_code())
            .expect("health/admin reason code must be registered");
        Self { kind, reason }
    }
}

/// health/readiness/liveness/admin 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedHealthAdminBehavior {
    /// listener bind success is reported as full readiness.
    ListenerBindSuccessReportedAsFullReadiness,
    /// liveness success is reported as domain acceptance.
    LivenessSuccessReportedAsDomainAcceptance,
    /// admin command mutates state outside core use case.
    AdminCommandMutatesStateOutsideCoreUseCase,
    /// admin command executes without operator/admin authorization evidence.
    AdminCommandExecutesWithoutAuthorizationEvidence,
    /// maintenance mode is represented only by driver-local flag.
    MaintenanceModeRepresentedOnlyByDriverLocalFlag,
    /// probe response hides failed dependency.
    ProbeResponseHidesFailedDependency,
    /// readiness output is used as production evidence without an evidence record.
    ReadinessOutputUsedAsProductionEvidenceWithoutReport,
    /// single-node readiness is used as multi-node readiness.
    SingleNodeReadinessUsedAsMultiNodeReadiness,
    /// public endpoint connection success is treated as domain readiness.
    PublicEndpointConnectionSuccessTreatedAsDomainReadiness,
    /// service discovery success is treated as readiness without target dependency/probe evidence.
    ServiceDiscoverySuccessTreatedAsReadiness,
    /// failover is treated as healthy from replacement endpoint reachability alone.
    FailoverTreatedHealthyFromEndpointReachability,
    /// worker task is treated as healthy from spawn success alone.
    WorkerTaskHealthyFromSpawnSuccessAlone,
    /// internal service trust is treated as healthy from endpoint resolution or TLS listener startup alone.
    InternalServiceTrustHealthyFromEndpointOrTlsStartup,
    /// probe success is used as build/test/runtime proof outside its evidence class.
    ProbeSuccessUsedAsExternalProof,
    /// health probe exposes sensitive raw data.
    HealthProbeExposesSensitiveRawData,
}

/// v0.2 initial architecture の operator/admin class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperatorAdminClass {
    /// operator may request diagnostic probe.
    OperatorProbeContext,
    /// operator may request admin action.
    OperatorAdminActionContext,
    /// operator may request drain/maintenance mode.
    MaintenanceActionContext,
    /// operator may verify evidence/hash-chain/report.
    EvidenceVerificationContext,
    /// local development helper context.
    DeveloperLocalContext,
}

impl OperatorAdminClass {
    /// developer-local context は production/operator evidence ではありません。
    pub const fn is_developer_local(self) -> bool {
        matches!(self, Self::DeveloperLocalContext)
    }

    /// operator/admin class ごとの action class 互換性です。
    pub const fn admits_action_class(self, action_class: OperatorAdminAllowedActionClass) -> bool {
        matches!(
            (self, action_class),
            (
                Self::OperatorProbeContext,
                OperatorAdminAllowedActionClass::DiagnosticProbeAction
                    | OperatorAdminAllowedActionClass::DriverDependencyProbeAction
            ) | (
                Self::OperatorAdminActionContext,
                OperatorAdminAllowedActionClass::AdminAction
            ) | (
                Self::MaintenanceActionContext,
                OperatorAdminAllowedActionClass::MaintenanceAction
            ) | (
                Self::EvidenceVerificationContext,
                OperatorAdminAllowedActionClass::EvidenceVerificationAction
            ) | (
                Self::DeveloperLocalContext,
                OperatorAdminAllowedActionClass::DiagnosticProbeAction
                    | OperatorAdminAllowedActionClass::DriverDependencyProbeAction
            )
        )
    }

    /// operator/admin class ごとの target scope 互換性です。
    pub const fn admits_target_scope(self, target_scope: OperatorAdminTargetScopeClass) -> bool {
        matches!(
            (self, target_scope),
            (
                Self::OperatorProbeContext,
                OperatorAdminTargetScopeClass::EntrypointScope
                    | OperatorAdminTargetScopeClass::ServiceScope
                    | OperatorAdminTargetScopeClass::NodeScope
                    | OperatorAdminTargetScopeClass::DeploymentScope
            ) | (
                Self::OperatorAdminActionContext,
                OperatorAdminTargetScopeClass::EntrypointScope
                    | OperatorAdminTargetScopeClass::ServiceScope
                    | OperatorAdminTargetScopeClass::NodeScope
                    | OperatorAdminTargetScopeClass::DeploymentScope
                    | OperatorAdminTargetScopeClass::ExplicitDomainScope
            ) | (
                Self::MaintenanceActionContext,
                OperatorAdminTargetScopeClass::EntrypointScope
                    | OperatorAdminTargetScopeClass::ServiceScope
                    | OperatorAdminTargetScopeClass::NodeScope
                    | OperatorAdminTargetScopeClass::DeploymentScope
            ) | (
                Self::EvidenceVerificationContext,
                OperatorAdminTargetScopeClass::EvidenceReportScope
            ) | (
                Self::DeveloperLocalContext,
                OperatorAdminTargetScopeClass::EntrypointScope
            )
        )
    }

    /// operator/admin class、action、target scope の組み合わせ互換性です。
    pub const fn admits_action_scope(
        self,
        action_class: OperatorAdminAllowedActionClass,
        target_scope: OperatorAdminTargetScopeClass,
    ) -> bool {
        self.admits_action_class(action_class) && self.admits_target_scope(target_scope)
    }
}

/// operator credential / context source の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperatorCredentialContextSourceClass {
    /// external identity provider credential/reference.
    ExternalIdentityProviderCredential,
    /// entrypoints/drivers supplied opaque credential reference.
    EntrypointDriverCredentialReference,
    /// preconfigured operator context reference.
    ConfiguredOperatorContextReference,
    /// local development helper credential reference.
    LocalDeveloperCredentialReference,
}

/// operator/admin authorization が許可する action class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperatorAdminAllowedActionClass {
    /// diagnostic probe action.
    DiagnosticProbeAction,
    /// admin action.
    AdminAction,
    /// maintenance/drain action.
    MaintenanceAction,
    /// evidence/hash-chain/report verification action.
    EvidenceVerificationAction,
    /// driver dependency probe action.
    DriverDependencyProbeAction,
}

/// operator/admin target scope の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperatorAdminTargetScopeClass {
    /// entrypoint-scoped action.
    EntrypointScope,
    /// service-scoped action.
    ServiceScope,
    /// node-scoped action.
    NodeScope,
    /// deployment-scoped action.
    DeploymentScope,
    /// evidence/report-scoped action.
    EvidenceReportScope,
    /// explicitly domain-scoped action.
    ExplicitDomainScope,
}

/// operator/admin authorization outcome の audit/evidence 閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperatorAdminAuthorizationOutcome {
    /// authorization accepted.
    Accepted,
    /// authorization rejected.
    Rejected,
    /// authorization context expired.
    Expired,
    /// authorization processing failed.
    Failed,
}

impl OperatorAdminAuthorizationOutcome {
    /// rejected/expired/failed outcome では cataloged reason を必須にします。
    pub const fn requires_reason(self) -> bool {
        matches!(self, Self::Rejected | Self::Expired | Self::Failed)
    }
}

/// operator/admin authorization context mapping guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OperatorAdminAuthorizationGuard {
    operator_admin_class: OperatorAdminClass,
    credential_source_class: OperatorCredentialContextSourceClass,
    allowed_action_class: OperatorAdminAllowedActionClass,
    target_scope_class: OperatorAdminTargetScopeClass,
    operator_admin_class_declared: bool,
    credential_context_source_declared: bool,
    allowed_action_class_declared: bool,
    allowed_target_scope_declared: bool,
    lifetime_expiry_declared: bool,
    redaction_rule_declared: bool,
    target_command_boundary_declared: bool,
    audit_event_relation_declared: bool,
    failure_reason_mapping_declared: bool,
    raw_credential_not_core_identity: bool,
    credential_verification_result_typed_only: bool,
    communication_participant_authorization_not_reused: bool,
    operator_admin_authorization_not_used_for_participant_domain_action: bool,
    target_action_keeps_core_or_driver_boundary: bool,
    target_action_event_required_when_execution_claimed: bool,
    developer_local_context_not_used_as_production_operator_evidence: bool,
}

