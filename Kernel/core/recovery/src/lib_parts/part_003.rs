/// prior drain status です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PriorDrainStatus {
    /// graceful drain evidence recorded.
    GracefulDrainEvidenceRecorded,
    /// drain did not complete or evidence is absent.
    DrainIncompleteOrAbsent,
}

/// audit persistence status です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuditPersistenceStatus {
    /// audit persistence completed.
    AuditPersistenceCompleted,
    /// audit persistence absent or incomplete.
    AuditPersistenceIncompleteOrAbsent,
}

/// restart 後 readiness の evidence class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RestartReadinessClass {
    /// readiness not claimed.
    NotClaimed,
    /// process uptime/readiness observation only.
    ProcessReadinessObservationOnly,
    /// readiness is separately evidenced and not used as restore proof.
    SeparateReadinessEvidence,
}

/// restore/replay policy application status です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RestoreReplayPolicyApplication {
    /// restore/replay policy not applied and no domain state claim may be made.
    NotApplied,
    /// durable recovery restore eligibility was applied.
    RestoreEligibilityApplied,
    /// audit replay verification only.
    ReplayVerificationOnly,
}

/// close-not-claimed scope です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CloseNotClaimedScope {
    /// no closeout claim is made for affected domain state.
    AffectedDomainState,
    /// only process lifecycle observation is claimed.
    ProcessLifecycleOnly,
}

/// process lifecycle observation の分類結果です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProcessFailureClassification {
    startup_run_before_restart: Option<StartupRunId>,
    startup_run_after_restart: Option<StartupRunId>,
    process_identity: OpaqueReference,
    failure_class: ProcessFailureClass,
    prior_drain_status: PriorDrainStatus,
    audit_persistence_status: AuditPersistenceStatus,
    restore_replay_policy_application: RestoreReplayPolicyApplication,
    readiness_class_after_restart: RestartReadinessClass,
    close_not_claimed_scope: CloseNotClaimedScope,
}

/// process failure classification の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProcessFailureClassificationError {
    /// process identity がありません。
    ProcessIdentityMissing,
    /// prior drain status がありません。
    PriorDrainStatusMissing,
    /// audit persistence status がありません。
    AuditPersistenceStatusMissing,
}

impl ProcessFailureClassification {
    /// process failure observation を分類します。
    pub fn try_new(
        startup_run_before_restart: Option<StartupRunId>,
        startup_run_after_restart: Option<StartupRunId>,
        process_identity: Option<OpaqueReference>,
        failure_class: ProcessFailureClass,
        prior_drain_status: Option<PriorDrainStatus>,
        audit_persistence_status: Option<AuditPersistenceStatus>,
        restore_replay_policy_application: RestoreReplayPolicyApplication,
        readiness_class_after_restart: RestartReadinessClass,
        close_not_claimed_scope: CloseNotClaimedScope,
    ) -> Result<Self, ProcessFailureClassificationError> {
        let process_identity =
            process_identity.ok_or(ProcessFailureClassificationError::ProcessIdentityMissing)?;
        let prior_drain_status =
            prior_drain_status.ok_or(ProcessFailureClassificationError::PriorDrainStatusMissing)?;
        let audit_persistence_status = audit_persistence_status
            .ok_or(ProcessFailureClassificationError::AuditPersistenceStatusMissing)?;

        Ok(Self {
            startup_run_before_restart,
            startup_run_after_restart,
            process_identity,
            failure_class,
            prior_drain_status,
            audit_persistence_status,
            restore_replay_policy_application,
            readiness_class_after_restart,
            close_not_claimed_scope,
        })
    }

    /// closeout claim 上は unclean と扱うべき classification かどうかです。
    pub const fn treated_as_unclean_for_closeout(&self) -> bool {
        matches!(
            self.prior_drain_status,
            PriorDrainStatus::DrainIncompleteOrAbsent
        ) || matches!(
            self.audit_persistence_status,
            AuditPersistenceStatus::AuditPersistenceIncompleteOrAbsent
        ) || matches!(
            self.failure_class,
            ProcessFailureClass::PanicObserved
                | ProcessFailureClass::ProcessCrashObserved
                | ProcessFailureClass::UncleanShutdownDetected
                | ProcessFailureClass::SupervisorRestartObserved
                | ProcessFailureClass::StartupAfterUncleanExit
        )
    }
}

/// restart 後の domain state claim です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RestartDomainStateClaim {
    classification: ProcessFailureClassification,
    restore_eligibility: RestoreEligibility,
    crash_restart_evidence: CrashRestartEvidenceShape,
}

/// restart domain state claim の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RestartDomainStateClaimError {
    /// restart/failure observation alone cannot prove domain recovery.
    RestoreEligibilityRequired,
    /// crash/restart evidence shape がありません。
    CrashRestartEvidenceRequired,
}

impl RestartDomainStateClaim {
    /// restart 後の domain state claim は restore eligibility を必須にします。
    pub fn try_new(
        classification: ProcessFailureClassification,
        restore_eligibility: Option<RestoreEligibility>,
        crash_restart_evidence: Option<CrashRestartEvidenceShape>,
    ) -> Result<Self, RestartDomainStateClaimError> {
        let crash_restart_evidence = crash_restart_evidence
            .ok_or(RestartDomainStateClaimError::CrashRestartEvidenceRequired)?;
        if classification
            .failure_class
            .requires_restore_policy_before_state_claim()
        {
            let restore_eligibility = restore_eligibility
                .ok_or(RestartDomainStateClaimError::RestoreEligibilityRequired)?;
            return Ok(Self {
                classification,
                restore_eligibility,
                crash_restart_evidence,
            });
        }

        let restore_eligibility =
            restore_eligibility.ok_or(RestartDomainStateClaimError::RestoreEligibilityRequired)?;
        Ok(Self {
            classification,
            restore_eligibility,
            crash_restart_evidence,
        })
    }
}

/// crash/restart evidence に必要な field shape です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CrashRestartEvidenceShape {
    exact_command_or_procedure: &'static str,
    supervisor_or_process_runner: &'static str,
    expected_failure_class: ProcessFailureClass,
    observed_failure_class: ProcessFailureClass,
    startup_run_before_restart: StartupRunId,
    startup_run_after_restart: StartupRunId,
    logs_are_diagnostic_support_only: bool,
    audit_status_is_separate_evidence_class: bool,
    restore_status_is_separate_evidence_class: bool,
    readiness_status_is_separate_evidence_class: bool,
}

/// crash/restart evidence shape の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CrashRestartEvidenceShapeError {
    /// exact command/procedure がありません。
    CommandOrProcedureMissing,
    /// supervisor or process runner がありません。
    SupervisorOrRunnerMissing,
    /// startup run ID before restart がありません。
    StartupRunBeforeMissing,
    /// startup run ID after restart がありません。
    StartupRunAfterMissing,
    /// log text が authoritative reason として扱われています。
    LogsUsedAsAuthoritativeReason,
    /// audit status が separate evidence class になっていません。
    AuditStatusNotSeparated,
    /// restore status が separate evidence class になっていません。
    RestoreStatusNotSeparated,
    /// readiness status が separate evidence class になっていません。
    ReadinessStatusNotSeparated,
}

impl CrashRestartEvidenceShape {
    /// crash/restart evidence に必要な field を検査して作ります。
    pub fn try_new(
        exact_command_or_procedure: &'static str,
        supervisor_or_process_runner: &'static str,
        expected_failure_class: ProcessFailureClass,
        observed_failure_class: ProcessFailureClass,
        startup_run_before_restart: Option<StartupRunId>,
        startup_run_after_restart: Option<StartupRunId>,
        logs_are_diagnostic_support_only: bool,
        audit_status_is_separate_evidence_class: bool,
        restore_status_is_separate_evidence_class: bool,
        readiness_status_is_separate_evidence_class: bool,
    ) -> Result<Self, CrashRestartEvidenceShapeError> {
        if exact_command_or_procedure.is_empty() {
            return Err(CrashRestartEvidenceShapeError::CommandOrProcedureMissing);
        }
        if supervisor_or_process_runner.is_empty() {
            return Err(CrashRestartEvidenceShapeError::SupervisorOrRunnerMissing);
        }
        let startup_run_before_restart = startup_run_before_restart
            .ok_or(CrashRestartEvidenceShapeError::StartupRunBeforeMissing)?;
        let startup_run_after_restart = startup_run_after_restart
            .ok_or(CrashRestartEvidenceShapeError::StartupRunAfterMissing)?;
        if !logs_are_diagnostic_support_only {
            return Err(CrashRestartEvidenceShapeError::LogsUsedAsAuthoritativeReason);
        }
        if !audit_status_is_separate_evidence_class {
            return Err(CrashRestartEvidenceShapeError::AuditStatusNotSeparated);
        }
        if !restore_status_is_separate_evidence_class {
            return Err(CrashRestartEvidenceShapeError::RestoreStatusNotSeparated);
        }
        if !readiness_status_is_separate_evidence_class {
            return Err(CrashRestartEvidenceShapeError::ReadinessStatusNotSeparated);
        }

        Ok(Self {
            exact_command_or_procedure,
            supervisor_or_process_runner,
            expected_failure_class,
            observed_failure_class,
            startup_run_before_restart,
            startup_run_after_restart,
            logs_are_diagnostic_support_only,
            audit_status_is_separate_evidence_class,
            restore_status_is_separate_evidence_class,
            readiness_status_is_separate_evidence_class,
        })
    }
}

/// crash/panic/supervisor restart failure mapping key です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProcessFailureMappingKind {
    /// process panic observed.
    ProcessPanicDetected,
    /// task panic observed without process crash.
    RuntimeTaskPanicDetected,
    /// process crash observed.
    ProcessCrashDetected,
    /// unclean shutdown detected.
    UncleanShutdownDetected,
    /// supervisor restart observed.
    SupervisorRestartObserved,
    /// driver shutdown during crash handling.
    DriverShutdown,
}

impl ProcessFailureMappingKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::ProcessPanicDetected => "process_panic_detected",
            Self::RuntimeTaskPanicDetected => "runtime_task_panic_detected",
            Self::ProcessCrashDetected => "process_crash_detected",
            Self::UncleanShutdownDetected => "unclean_shutdown_detected",
            Self::SupervisorRestartObserved => "supervisor_restart_observed",
            Self::DriverShutdown => "driver_shutdown",
        }
    }
}

/// panic/crash/restart 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedProcessFailureBehavior {
    /// unclean crash is reported as graceful shutdown.
    UncleanCrashAsGracefulShutdown,
    /// supervisor restart is reported as readiness.
    SupervisorRestartAsReadiness,
    /// crash recovery success is inferred without restore/replay evidence.
    CrashRecoveryInferredWithoutRestoreEvidence,
    /// panic log text becomes authoritative reason.
    PanicLogTextAsAuthoritativeReason,
    /// restarted process reuses previous domain state without restore policy.
    RestartReusesDomainStateWithoutRestorePolicy,
    /// crash evidence omits prior audit/drain status.
    CrashEvidenceOmitsAuditOrDrainStatus,
    /// task panic is reported only as process readiness or generic driver failure.
    TaskPanicCollapsedIntoReadinessOrDriverFailure,
}
