/// prior drain status です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PriorDrainStatus {
    /// graceful drain was observed.
    GracefulDrainObserved,
    /// drain did not complete or observation is absent.
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

/// restart 後 readiness の observation class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RestartReadinessClass {
    /// readiness not claimed.
    NotClaimed,
    /// process uptime/readiness observation only.
    ProcessReadinessObservationOnly,
    /// readiness is observed separately and not used as restore proof.
    ReadinessObservedSeparately,
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
        })
    }

    /// unclean shutdown として扱うべき classification かどうかです。
    pub const fn is_unclean_shutdown(&self) -> bool {
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
    crash_restart_verification: CrashRestartVerification,
}

/// restart domain state claim の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RestartDomainStateClaimError {
    /// restart/failure observation alone cannot prove domain recovery.
    RestoreEligibilityRequired,
    /// crash/restart verification がありません。
    CrashRestartVerificationRequired,
}

impl RestartDomainStateClaim {
    /// restart 後の domain state claim は restore eligibility を必須にします。
    pub fn try_new(
        classification: ProcessFailureClassification,
        restore_eligibility: Option<RestoreEligibility>,
        crash_restart_verification: Option<CrashRestartVerification>,
    ) -> Result<Self, RestartDomainStateClaimError> {
        let crash_restart_verification = crash_restart_verification
            .ok_or(RestartDomainStateClaimError::CrashRestartVerificationRequired)?;
        if classification
            .failure_class
            .requires_restore_policy_before_state_claim()
        {
            let restore_eligibility = restore_eligibility
                .ok_or(RestartDomainStateClaimError::RestoreEligibilityRequired)?;
            return Ok(Self {
                classification,
                restore_eligibility,
                crash_restart_verification,
            });
        }

        let restore_eligibility =
            restore_eligibility.ok_or(RestartDomainStateClaimError::RestoreEligibilityRequired)?;
        Ok(Self {
            classification,
            restore_eligibility,
            crash_restart_verification,
        })
    }
}

/// crash/restart runtime verification です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CrashRestartVerification {
    supervisor_or_process_runner: &'static str,
    expected_failure_class: ProcessFailureClass,
    observed_failure_class: ProcessFailureClass,
    startup_run_before_restart: StartupRunId,
    startup_run_after_restart: StartupRunId,
    logs_are_diagnostic_support_only: bool,
    audit_status_is_distinct: bool,
    restore_status_is_distinct: bool,
    readiness_status_is_distinct: bool,
}

/// crash/restart verification の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CrashRestartVerificationError {
    /// supervisor or process runner がありません。
    SupervisorOrRunnerMissing,
    /// startup run ID before restart がありません。
    StartupRunBeforeMissing,
    /// startup run ID after restart がありません。
    StartupRunAfterMissing,
    /// log text が authoritative reason として扱われています。
    LogsUsedAsAuthoritativeReason,
    /// audit status が distinct になっていません。
    AuditStatusNotDistinct,
    /// restore status が distinct になっていません。
    RestoreStatusNotDistinct,
    /// readiness status が distinct になっていません。
    ReadinessStatusNotDistinct,
}

impl CrashRestartVerification {
    /// crash/restart の runtime invariants を検査して作ります。
    pub fn try_new(
        supervisor_or_process_runner: &'static str,
        expected_failure_class: ProcessFailureClass,
        observed_failure_class: ProcessFailureClass,
        startup_run_before_restart: Option<StartupRunId>,
        startup_run_after_restart: Option<StartupRunId>,
        logs_are_diagnostic_support_only: bool,
        audit_status_is_distinct: bool,
        restore_status_is_distinct: bool,
        readiness_status_is_distinct: bool,
    ) -> Result<Self, CrashRestartVerificationError> {
        if supervisor_or_process_runner.is_empty() {
            return Err(CrashRestartVerificationError::SupervisorOrRunnerMissing);
        }
        let startup_run_before_restart = startup_run_before_restart
            .ok_or(CrashRestartVerificationError::StartupRunBeforeMissing)?;
        let startup_run_after_restart = startup_run_after_restart
            .ok_or(CrashRestartVerificationError::StartupRunAfterMissing)?;
        if !logs_are_diagnostic_support_only {
            return Err(CrashRestartVerificationError::LogsUsedAsAuthoritativeReason);
        }
        if !audit_status_is_distinct {
            return Err(CrashRestartVerificationError::AuditStatusNotDistinct);
        }
        if !restore_status_is_distinct {
            return Err(CrashRestartVerificationError::RestoreStatusNotDistinct);
        }
        if !readiness_status_is_distinct {
            return Err(CrashRestartVerificationError::ReadinessStatusNotDistinct);
        }

        Ok(Self {
            supervisor_or_process_runner,
            expected_failure_class,
            observed_failure_class,
            startup_run_before_restart,
            startup_run_after_restart,
            logs_are_diagnostic_support_only,
            audit_status_is_distinct,
            restore_status_is_distinct,
            readiness_status_is_distinct,
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
    /// crash recovery success is inferred without restore/replay verification.
    CrashRecoveryInferredWithoutRestoreVerification,
    /// panic log text becomes authoritative reason.
    PanicLogTextAsAuthoritativeReason,
    /// restarted process reuses previous domain state without restore policy.
    RestartReusesDomainStateWithoutRestorePolicy,
    /// crash verification omits prior audit/drain status.
    CrashVerificationOmitsAuditOrDrainStatus,
    /// task panic is reported only as process readiness or generic driver failure.
    TaskPanicCollapsedIntoReadinessOrDriverFailure,
}
