// entrypoints/admin は health/readiness/liveness/admin/operator wiring の surface です。
//
// health や admin の応答を domain success と混同せず、後続 task で
// entrypoint-owned operational entrypoint を配置する場所です。

use arcrtc_core_reason::CatalogedReasonRef;

/// entrypoint admin package の所有境界を示す marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntrypointAdminSurface;

/// v0.2 initial architecture の probe class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdminProbeClass {
    /// process/runtime loop is observable.
    ProcessLiveness,
    /// selected driver dependency can initialize/respond.
    DriverDependencyReadiness,
    /// required core policy bundle accepted.
    CorePolicyReadiness,
    /// selected entrypoint wiring completed.
    CompositionReadiness,
    /// entrypoint is in normal/draining/maintenance mode.
    MaintenanceStatus,
    /// admin/CLI action result.
    OperatorActionResult,
}

/// health/readiness/liveness/admin の outcome 閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HealthAdminOutcome {
    /// requested probe/action was satisfied.
    Satisfied,
    /// readiness or action condition was not satisfied.
    NotSatisfied,
    /// requested probe/action failed.
    Failed,
    /// action was blocked by maintenance/drain state.
    Blocked,
}

impl HealthAdminOutcome {
    /// 非成功 outcome では cataloged reason を必須にします。
    pub const fn requires_reason(self) -> bool {
        matches!(self, Self::NotSatisfied | Self::Failed | Self::Blocked)
    }
}

/// maintenance status の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MaintenanceStatusClass {
    /// entrypoint accepts normal operational actions.
    Normal,
    /// entrypoint is draining.
    Draining,
    /// entrypoint is in maintenance mode.
    Maintenance,
}

/// admin / maintenance command の許可 class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdminMaintenanceActionClass {
    /// request drain/shutdown through cross-plane shutdown/drain.
    RequestDrainShutdown,
    /// inspect redacted state references.
    InspectRedactedStateReference,
    /// trigger bounded verification commands.
    TriggerBoundedVerificationCommand,
    /// request driver dependency probes.
    RequestDriverDependencyProbe,
    /// request audit/hash-chain verification.
    RequestAuditHashChainVerification,
}

/// readiness composition guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReadinessCompositionGuard {
    probe_class: AdminProbeClass,
    outcome: HealthAdminOutcome,
    startup_run_id_declared: bool,
    entrypoint_name_declared: bool,
    profile_class_declared_when_applicable: bool,
    selected_drivers_declared: bool,
    deployment_topology_and_node_scope_declared_when_relevant: bool,
    service_discovery_resolution_declared_when_affects_readiness: bool,
    distributed_state_failover_declared_when_affects_readiness: bool,
    runtime_task_supervision_declared_when_affects_readiness: bool,
    internal_service_trust_declared_when_affects_readiness: bool,
    public_endpoint_lifecycle_declared_when_endpoint_behavior_target_claim: bool,
    probe_class_declared: bool,
    included_dependency_checks_declared: bool,
    excluded_checks_declared: bool,
    outcome_declared: bool,
    cataloged_reason_declared_for_non_success: bool,
    evidence_class_declared: bool,
    close_not_claimed_scope_declared: bool,
    readiness_response_not_single_unqualified_boolean: bool,
    unevaluated_required_component_prevents_satisfied_claim: bool,
}

/// readiness composition の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReadinessCompositionError {
    /// StartupRunId がありません。
    StartupRunIdMissing,
    /// entrypoint name がありません。
    EntrypointNameMissing,
    /// applicable profile class がありません。
    ProfileClassMissing,
    /// selected drivers がありません。
    SelectedDriversMissing,
    /// topology class / node scope がありません。
    TopologyNodeScopeMissing,
    /// service discovery source / resolution state がありません。
    ServiceDiscoveryResolutionMissing,
    /// distributed state / failover status がありません。
    DistributedStateFailoverMissing,
    /// runtime task class / supervision state がありません。
    RuntimeTaskSupervisionMissing,
    /// internal service trust class がありません。
    InternalServiceTrustMissing,
    /// public endpoint lifecycle state がありません。
    PublicEndpointLifecycleMissing,
    /// probe class がありません。
    ProbeClassMissing,
    /// included dependency checks がありません。
    IncludedDependencyChecksMissing,
    /// excluded checks がありません。
    ExcludedChecksMissing,
    /// outcome がありません。
    OutcomeMissing,
    /// non-success の cataloged reason がありません。
    CatalogedReasonMissing,
    /// evidence class がありません。
    EvidenceClassMissing,
    /// close-not-claimed scope がありません。
    CloseNotClaimedScopeMissing,
    /// readiness が単一 boolean になっています。
    SingleUnqualifiedBooleanReadiness,
    /// 未評価 component を含む claim を satisfied としています。
    UnevaluatedComponentClaimedSatisfied,
}

impl ReadinessCompositionGuard {
    /// readiness response が claims に採用可能な shape かを検査します。
    pub const fn try_new(
        probe_class: AdminProbeClass,
        outcome: HealthAdminOutcome,
        startup_run_id_declared: bool,
        entrypoint_name_declared: bool,
        profile_class_declared_when_applicable: bool,
        selected_drivers_declared: bool,
        deployment_topology_and_node_scope_declared_when_relevant: bool,
        service_discovery_resolution_declared_when_affects_readiness: bool,
        distributed_state_failover_declared_when_affects_readiness: bool,
        runtime_task_supervision_declared_when_affects_readiness: bool,
        internal_service_trust_declared_when_affects_readiness: bool,
        public_endpoint_lifecycle_declared_when_endpoint_behavior_target_claim: bool,
        probe_class_declared: bool,
        included_dependency_checks_declared: bool,
        excluded_checks_declared: bool,
        outcome_declared: bool,
        cataloged_reason_declared_for_non_success: bool,
        evidence_class_declared: bool,
        close_not_claimed_scope_declared: bool,
        readiness_response_not_single_unqualified_boolean: bool,
        unevaluated_required_component_prevents_satisfied_claim: bool,
    ) -> Result<Self, ReadinessCompositionError> {
        if !startup_run_id_declared {
            return Err(ReadinessCompositionError::StartupRunIdMissing);
        }
        if !entrypoint_name_declared {
            return Err(ReadinessCompositionError::EntrypointNameMissing);
        }
        if !profile_class_declared_when_applicable {
            return Err(ReadinessCompositionError::ProfileClassMissing);
        }
        if !selected_drivers_declared {
            return Err(ReadinessCompositionError::SelectedDriversMissing);
        }
        if !deployment_topology_and_node_scope_declared_when_relevant {
            return Err(ReadinessCompositionError::TopologyNodeScopeMissing);
        }
        if !service_discovery_resolution_declared_when_affects_readiness {
            return Err(ReadinessCompositionError::ServiceDiscoveryResolutionMissing);
        }
        if !distributed_state_failover_declared_when_affects_readiness {
            return Err(ReadinessCompositionError::DistributedStateFailoverMissing);
        }
        if !runtime_task_supervision_declared_when_affects_readiness {
            return Err(ReadinessCompositionError::RuntimeTaskSupervisionMissing);
        }
        if !internal_service_trust_declared_when_affects_readiness {
            return Err(ReadinessCompositionError::InternalServiceTrustMissing);
        }
        if !public_endpoint_lifecycle_declared_when_endpoint_behavior_target_claim {
            return Err(ReadinessCompositionError::PublicEndpointLifecycleMissing);
        }
        if !probe_class_declared {
            return Err(ReadinessCompositionError::ProbeClassMissing);
        }
        if !included_dependency_checks_declared {
            return Err(ReadinessCompositionError::IncludedDependencyChecksMissing);
        }
        if !excluded_checks_declared {
            return Err(ReadinessCompositionError::ExcludedChecksMissing);
        }
        if !outcome_declared {
            return Err(ReadinessCompositionError::OutcomeMissing);
        }
        if outcome.requires_reason() && !cataloged_reason_declared_for_non_success {
            return Err(ReadinessCompositionError::CatalogedReasonMissing);
        }
        if !evidence_class_declared {
            return Err(ReadinessCompositionError::EvidenceClassMissing);
        }
        if !close_not_claimed_scope_declared {
            return Err(ReadinessCompositionError::CloseNotClaimedScopeMissing);
        }
        if !readiness_response_not_single_unqualified_boolean {
            return Err(ReadinessCompositionError::SingleUnqualifiedBooleanReadiness);
        }
        if !unevaluated_required_component_prevents_satisfied_claim {
            return Err(ReadinessCompositionError::UnevaluatedComponentClaimedSatisfied);
        }

        Ok(Self {
            probe_class,
            outcome,
            startup_run_id_declared,
            entrypoint_name_declared,
            profile_class_declared_when_applicable,
            selected_drivers_declared,
            deployment_topology_and_node_scope_declared_when_relevant,
            service_discovery_resolution_declared_when_affects_readiness,
            distributed_state_failover_declared_when_affects_readiness,
            runtime_task_supervision_declared_when_affects_readiness,
            internal_service_trust_declared_when_affects_readiness,
            public_endpoint_lifecycle_declared_when_endpoint_behavior_target_claim,
            probe_class_declared,
            included_dependency_checks_declared,
            excluded_checks_declared,
            outcome_declared,
            cataloged_reason_declared_for_non_success,
            evidence_class_declared,
            close_not_claimed_scope_declared,
            readiness_response_not_single_unqualified_boolean,
            unevaluated_required_component_prevents_satisfied_claim,
        })
    }
}

/// admin / maintenance command guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AdminMaintenanceCommandGuard {
    action_class: AdminMaintenanceActionClass,
    action_class_declared: bool,
    core_use_case_or_driver_operation_boundary_declared: bool,
    privileged_action_requires_operator_admin_authorization: bool,
    operator_admin_authorization_satisfied_before_execution: bool,
    raw_secret_token_packet_regulated_payload_not_exposed: bool,
    state_mutation_only_through_core_use_case: bool,
    probe_success_not_used_as_domain_acceptance: bool,
    closeout_completion_requires_evidence_report: bool,
    maintenance_status_not_driver_local_flag_only: bool,
}

/// admin / maintenance command の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdminMaintenanceCommandError {
    /// action class がありません。
    ActionClassMissing,
    /// core use case / driver operation boundary がありません。
    ActionBoundaryMissing,
    /// privileged action の operator/admin authorization rule がありません。
    AuthorizationRuleMissing,
    /// operator/admin authorization が target action より前に成立していません。
    AuthorizationMissing,
    /// raw secret/token/packet/regulated payload が露出しています。
    SensitiveRawPayloadExposed,
    /// core use case 外で core aggregate state を mutate しています。
    StateMutationBypassesCoreUseCase,
    /// probe success を domain acceptance として扱っています。
    ProbeSuccessUsedAsDomainAcceptance,
    /// evidence report なしに closeout complete を扱っています。
    CloseoutWithoutEvidenceReport,
    /// maintenance status が driver-local flag だけで表現されています。
    MaintenanceStatusDriverLocalOnly,
}

impl AdminMaintenanceCommandGuard {
    /// admin / maintenance command の実行境界を検査します。
    pub const fn try_new(
        action_class: AdminMaintenanceActionClass,
        action_class_declared: bool,
        core_use_case_or_driver_operation_boundary_declared: bool,
        privileged_action_requires_operator_admin_authorization: bool,
        operator_admin_authorization_satisfied_before_execution: bool,
        raw_secret_token_packet_regulated_payload_not_exposed: bool,
        state_mutation_only_through_core_use_case: bool,
        probe_success_not_used_as_domain_acceptance: bool,
        closeout_completion_requires_evidence_report: bool,
        maintenance_status_not_driver_local_flag_only: bool,
    ) -> Result<Self, AdminMaintenanceCommandError> {
        if !action_class_declared {
            return Err(AdminMaintenanceCommandError::ActionClassMissing);
        }
        if !core_use_case_or_driver_operation_boundary_declared {
            return Err(AdminMaintenanceCommandError::ActionBoundaryMissing);
        }
        if !privileged_action_requires_operator_admin_authorization {
            return Err(AdminMaintenanceCommandError::AuthorizationRuleMissing);
        }
        if !operator_admin_authorization_satisfied_before_execution {
            return Err(AdminMaintenanceCommandError::AuthorizationMissing);
        }
        if !raw_secret_token_packet_regulated_payload_not_exposed {
            return Err(AdminMaintenanceCommandError::SensitiveRawPayloadExposed);
        }
        if !state_mutation_only_through_core_use_case {
            return Err(AdminMaintenanceCommandError::StateMutationBypassesCoreUseCase);
        }
        if !probe_success_not_used_as_domain_acceptance {
            return Err(AdminMaintenanceCommandError::ProbeSuccessUsedAsDomainAcceptance);
        }
        if !closeout_completion_requires_evidence_report {
            return Err(AdminMaintenanceCommandError::CloseoutWithoutEvidenceReport);
        }
        if !maintenance_status_not_driver_local_flag_only {
            return Err(AdminMaintenanceCommandError::MaintenanceStatusDriverLocalOnly);
        }

        Ok(Self {
            action_class,
            action_class_declared,
            core_use_case_or_driver_operation_boundary_declared,
            privileged_action_requires_operator_admin_authorization,
            operator_admin_authorization_satisfied_before_execution,
            raw_secret_token_packet_regulated_payload_not_exposed,
            state_mutation_only_through_core_use_case,
            probe_success_not_used_as_domain_acceptance,
            closeout_completion_requires_evidence_report,
            maintenance_status_not_driver_local_flag_only,
        })
    }
}

/// health/readiness/liveness/admin の audit event type です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HealthAdminAuditEventType {
    /// health/readiness/liveness/admin probe observed.
    OperationalProbeObservation,
    /// admin or maintenance action decision.
    AdminMaintenanceDecision,
}

impl HealthAdminAuditEventType {
    /// audit event catalog に接続する event type です。
    pub const fn event_type(self) -> &'static str {
        match self {
            Self::OperationalProbeObservation => "operational_probe_observation",
            Self::AdminMaintenanceDecision => "admin_maintenance_decision",
        }
    }

    /// audit event type ごとの outcome 閉集合を検査します。
    pub const fn admits_outcome(self, outcome: HealthAdminAuditOutcome) -> bool {
        matches!(
            (self, outcome),
            (
                Self::OperationalProbeObservation,
                HealthAdminAuditOutcome::WithinBoundObserved
                    | HealthAdminAuditOutcome::Rejected
                    | HealthAdminAuditOutcome::Failed
            ) | (
                Self::AdminMaintenanceDecision,
                HealthAdminAuditOutcome::Accepted
                    | HealthAdminAuditOutcome::Rejected
                    | HealthAdminAuditOutcome::Failed
            )
        )
    }
}

/// health/admin audit outcome の event catalog 閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HealthAdminAuditOutcome {
    /// operational probe observed within required bound.
    WithinBoundObserved,
    /// admin/maintenance action accepted.
    Accepted,
    /// probe/action rejected.
    Rejected,
    /// probe/action failed.
    Failed,
}

impl HealthAdminAuditOutcome {
    /// rejected/failed outcome では cataloged reason を必須にします。
    pub const fn requires_reason(self) -> bool {
        matches!(self, Self::Rejected | Self::Failed)
    }
}

/// health/admin audit shape guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HealthAdminAuditGuard {
    event_type: HealthAdminAuditEventType,
    outcome: HealthAdminAuditOutcome,
    startup_run_id_declared: bool,
    probe_or_action_class_declared: bool,
    entrypoint_reference_declared_when_operational_probe: bool,
    admin_action_reference_declared_when_admin_maintenance: bool,
    correlation_id_declared_when_command_scoped: bool,
    cataloged_reason_declared_for_non_success: bool,
}

/// health/admin audit shape の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HealthAdminAuditError {
    /// StartupRunId がありません。
    StartupRunIdMissing,
    /// probe/action class がありません。
    ProbeActionClassMissing,
    /// audit event type と outcome が一致していません。
    OutcomeMismatch,
    /// operational probe の entrypoint reference がありません。
    EntrypointReferenceMissing,
    /// admin maintenance の admin action reference がありません。
    AdminActionReferenceMissing,
    /// command-scoped event の CorrelationId がありません。
    CorrelationIdMissing,
    /// non-success outcome の cataloged reason がありません。
    CatalogedReasonMissing,
}

impl HealthAdminAuditGuard {
    /// operational_probe_observation / admin_maintenance_decision の audit shape を検査します。
    pub const fn try_new(
        event_type: HealthAdminAuditEventType,
        outcome: HealthAdminAuditOutcome,
        startup_run_id_declared: bool,
        probe_or_action_class_declared: bool,
        entrypoint_reference_declared_when_operational_probe: bool,
        admin_action_reference_declared_when_admin_maintenance: bool,
        correlation_id_declared_when_command_scoped: bool,
        cataloged_reason_declared_for_non_success: bool,
    ) -> Result<Self, HealthAdminAuditError> {
        if !startup_run_id_declared {
            return Err(HealthAdminAuditError::StartupRunIdMissing);
        }
        if !probe_or_action_class_declared {
            return Err(HealthAdminAuditError::ProbeActionClassMissing);
        }
        if !event_type.admits_outcome(outcome) {
            return Err(HealthAdminAuditError::OutcomeMismatch);
        }
        if matches!(
            event_type,
            HealthAdminAuditEventType::OperationalProbeObservation
        ) && !entrypoint_reference_declared_when_operational_probe
        {
            return Err(HealthAdminAuditError::EntrypointReferenceMissing);
        }
        if matches!(
            event_type,
            HealthAdminAuditEventType::AdminMaintenanceDecision
        ) && !admin_action_reference_declared_when_admin_maintenance
        {
            return Err(HealthAdminAuditError::AdminActionReferenceMissing);
        }
        if !correlation_id_declared_when_command_scoped {
            return Err(HealthAdminAuditError::CorrelationIdMissing);
        }
        if outcome.requires_reason() && !cataloged_reason_declared_for_non_success {
            return Err(HealthAdminAuditError::CatalogedReasonMissing);
        }

        Ok(Self {
            event_type,
            outcome,
            startup_run_id_declared,
            probe_or_action_class_declared,
            entrypoint_reference_declared_when_operational_probe,
            admin_action_reference_declared_when_admin_maintenance,
            correlation_id_declared_when_command_scoped,
            cataloged_reason_declared_for_non_success,
        })
    }
}

/// health/readiness/liveness evidence guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HealthAdminEvidenceGuard {
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
}

