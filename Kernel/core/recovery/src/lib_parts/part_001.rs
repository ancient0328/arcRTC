// core/recovery は durable recovery / restore / replay の core-owned surface です。
//
// DB 行、object key、file layout、driver migration detail は持ち込まず、
// state family ごとの recovery class と restore/replay eligibility だけを扱います。

use arcrtc_core_identity::{CorrelationId, OpaqueReference, StartupRunId};
use arcrtc_core_state::{StateClass, StateFamily};

/// core recovery package の所有境界を示す marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreRecoverySurface;

/// v0.2 initial architecture が認める recovery class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecoveryClass {
    /// restart 後に state を復元しません。
    NoRestore,
    /// core-owned checkpoint intent からのみ復元候補にできます。
    CheckpointRestoreCandidate,
    /// replay は integrity verification に限定され、domain mutation を行いません。
    AuditReplayVerificationOnly,
    /// driver-owned retry queue を bounded に再開します。
    DriverRetryRecovery,
}

impl RecoveryClass {
    /// core state mutation の復元候補にできる recovery class かどうかです。
    pub const fn permits_core_state_restore_candidate(self) -> bool {
        matches!(self, Self::CheckpointRestoreCandidate)
    }
}

/// state family ごとの recovery policy を固定する入力です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateFamilyRecoveryPolicy {
    state_family: StateFamily,
    recovery_class: RecoveryClass,
    explicit_restore_policy_configured: bool,
}

impl StateFamilyRecoveryPolicy {
    /// state family policy を作ります。
    const fn new(
        state_family: StateFamily,
        recovery_class: RecoveryClass,
        explicit_restore_policy_configured: bool,
    ) -> Self {
        Self {
            state_family,
            recovery_class,
            explicit_restore_policy_configured,
        }
    }

    /// state family の default recovery policy を返します。
    pub const fn default_for(state_family: StateFamily) -> Self {
        let recovery_class = match state_family {
            StateFamily::SignalingIdempotency => RecoveryClass::NoRestore,
            StateFamily::AuditEvent | StateFamily::AuditHashChainRecord => {
                RecoveryClass::AuditReplayVerificationOnly
            }
            StateFamily::DriverRetryStore => RecoveryClass::DriverRetryRecovery,
            StateFamily::MetricsBacklog => RecoveryClass::NoRestore,
            StateFamily::SignalingRoom
            | StateFamily::SignalingParticipant
            | StateFamily::SfuForwardingState
            | StateFamily::TurnRelayAuthorizationState
            | StateFamily::AtomicityCompensationEvidence
            | StateFamily::ResourceBoundCounters
            | StateFamily::ConfigurationDecision
            | StateFamily::SdkConnectionState => RecoveryClass::NoRestore,
        };

        Self::new(state_family, recovery_class, false)
    }

    /// checkpoint restore policy が明示された idempotency state の policy です。
    pub const fn signaling_idempotency_checkpoint_restore_configured() -> Self {
        Self::new(
            StateFamily::SignalingIdempotency,
            RecoveryClass::CheckpointRestoreCandidate,
            true,
        )
    }

    /// metrics retry policy が明示された metrics backlog の policy です。
    pub const fn metrics_backlog_retry_configured() -> Self {
        Self::new(
            StateFamily::MetricsBacklog,
            RecoveryClass::DriverRetryRecovery,
            true,
        )
    }

    /// core state restore candidate として許可された policy かどうかです。
    pub const fn permits_core_restore_candidate(self) -> bool {
        matches!(self.state_family, StateFamily::SignalingIdempotency)
            && matches!(
                self.recovery_class,
                RecoveryClass::CheckpointRestoreCandidate
            )
            && self.explicit_restore_policy_configured
    }

    /// replay による domain mutation candidate を許可できる policy かどうかです。
    pub const fn permits_domain_mutation_replay_candidate(self) -> bool {
        self.permits_core_restore_candidate()
    }
}

/// restore attempt が満たすべき precondition set です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RestorePreconditionSet {
    lifecycle_observation_classified_when_required: bool,
    restore_policy_allows_state_family: bool,
    accepted_checkpoint_version: bool,
    correlation_owner_aggregate_verified: bool,
    audit_hash_chain_gap_absent_when_required: bool,
    checkpoint_replay_conflict_absent: bool,
    resource_lifetime_bounds_valid: bool,
    failure_mapping_cataloged: bool,
    distributed_policy_recorded_when_required: bool,
}

/// restore precondition の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RestorePreconditionError {
    /// crash/panic/restart 起点の restore に必要な lifecycle observation が分類されていません。
    LifecycleObservationNotClassified,
    /// state family を許可する restore policy がありません。
    RestorePolicyDoesNotAllowStateFamily,
    /// checkpoint / snapshot version が accepted version として評価されていません。
    CheckpointVersionNotAccepted,
    /// correlation chain、owner、aggregate reference が検証されていません。
    CorrelationOwnerAggregateNotVerified,
    /// 必要な audit hash-chain relation に gap があります。
    AuditHashChainGapDetected,
    /// checkpoint と replay result が conflict しています。
    CheckpointReplayConflictDetected,
    /// resource / lifetime bound が restore 時点で再検証されていません。
    ResourceLifetimeBoundInvalid,
    /// driver/schema/migration failure が cataloged reason へ接続されていません。
    FailureMappingNotCataloged,
    /// distributed state/failover policy が必要な state family で owner/conflict rule が未記録です。
    DistributedPolicyNotRecorded,
}

impl RestorePreconditionSet {
    /// すべての restore precondition を満たす set だけを作ります。
    pub fn try_new(
        lifecycle_observation_classified_when_required: bool,
        restore_policy_allows_state_family: bool,
        accepted_checkpoint_version: bool,
        correlation_owner_aggregate_verified: bool,
        audit_hash_chain_gap_absent_when_required: bool,
        checkpoint_replay_conflict_absent: bool,
        resource_lifetime_bounds_valid: bool,
        failure_mapping_cataloged: bool,
        distributed_policy_recorded_when_required: bool,
    ) -> Result<Self, RestorePreconditionError> {
        if !lifecycle_observation_classified_when_required {
            return Err(RestorePreconditionError::LifecycleObservationNotClassified);
        }
        if !restore_policy_allows_state_family {
            return Err(RestorePreconditionError::RestorePolicyDoesNotAllowStateFamily);
        }
        if !accepted_checkpoint_version {
            return Err(RestorePreconditionError::CheckpointVersionNotAccepted);
        }
        if !correlation_owner_aggregate_verified {
            return Err(RestorePreconditionError::CorrelationOwnerAggregateNotVerified);
        }
        if !audit_hash_chain_gap_absent_when_required {
            return Err(RestorePreconditionError::AuditHashChainGapDetected);
        }
        if !checkpoint_replay_conflict_absent {
            return Err(RestorePreconditionError::CheckpointReplayConflictDetected);
        }
        if !resource_lifetime_bounds_valid {
            return Err(RestorePreconditionError::ResourceLifetimeBoundInvalid);
        }
        if !failure_mapping_cataloged {
            return Err(RestorePreconditionError::FailureMappingNotCataloged);
        }
        if !distributed_policy_recorded_when_required {
            return Err(RestorePreconditionError::DistributedPolicyNotRecorded);
        }

        Ok(Self {
            lifecycle_observation_classified_when_required,
            restore_policy_allows_state_family,
            accepted_checkpoint_version,
            correlation_owner_aggregate_verified,
            audit_hash_chain_gap_absent_when_required,
            checkpoint_replay_conflict_absent,
            resource_lifetime_bounds_valid,
            failure_mapping_cataloged,
            distributed_policy_recorded_when_required,
        })
    }
}

/// core state restore eligibility です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RestoreEligibility {
    state_family: StateFamily,
    state_class: StateClass,
    recovery_class: RecoveryClass,
    preconditions: RestorePreconditionSet,
}

/// restore eligibility の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RestoreEligibilityError {
    /// state family が restore policy の許可対象ではありません。
    StateFamilyNotRestoreEligible,
    /// recovery class が restore candidate ではありません。
    RecoveryClassDoesNotPermitRestore,
    /// state class が checkpoint-eligible ではありません。
    StateClassNotCheckpointEligible,
    /// restore policy が明示されていません。
    ExplicitRestorePolicyMissing,
}

impl RestoreEligibility {
    /// core state mutation の restore 候補を作ります。
    pub fn try_new(
        policy: StateFamilyRecoveryPolicy,
        state_class: StateClass,
        preconditions: RestorePreconditionSet,
    ) -> Result<Self, RestoreEligibilityError> {
        if !matches!(policy.state_family, StateFamily::SignalingIdempotency) {
            return Err(RestoreEligibilityError::StateFamilyNotRestoreEligible);
        }
        if !policy.recovery_class.permits_core_state_restore_candidate() {
            return Err(RestoreEligibilityError::RecoveryClassDoesNotPermitRestore);
        }
        if !matches!(state_class, StateClass::CheckpointEligibleState) {
            return Err(RestoreEligibilityError::StateClassNotCheckpointEligible);
        }
        if !policy.explicit_restore_policy_configured {
            return Err(RestoreEligibilityError::ExplicitRestorePolicyMissing);
        }
        if !policy.permits_core_restore_candidate() {
            return Err(RestoreEligibilityError::RecoveryClassDoesNotPermitRestore);
        }

        Ok(Self {
            state_family: policy.state_family,
            state_class,
            recovery_class: policy.recovery_class,
            preconditions,
        })
    }
}

/// audit replay の利用 mode です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReplayMode {
    /// audit integrity verification only.
    AuditIntegrityVerificationOnly,
    /// explicit restore policy がある場合だけ domain mutation candidate.
    DomainMutationCandidate,
}

/// replay で domain mutation を許可する場合に必要な policy coverage です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReplayPolicyCoverage {
    event_type_defined: bool,
    ordering_defined: bool,
    gap_handling_defined: bool,
    duplicate_handling_defined: bool,
    conflict_resolution_defined: bool,
    lifetime_resource_bound_revalidation_defined: bool,
    failure_reason_mapping_defined: bool,
}

/// replay policy coverage の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReplayPolicyCoverageError {
    /// replay 対象 event type が未定義です。
    EventTypeMissing,
    /// replay ordering が未定義です。
    OrderingMissing,
    /// gap handling が未定義です。
    GapHandlingMissing,
    /// duplicate handling が未定義です。
    DuplicateHandlingMissing,
    /// conflict resolution が未定義です。
    ConflictResolutionMissing,
    /// lifetime / resource bound revalidation が未定義です。
    BoundRevalidationMissing,
    /// replay failure reason mapping が未定義です。
    FailureReasonMappingMissing,
}

impl ReplayPolicyCoverage {
    /// domain mutation に使える replay policy coverage だけを作ります。
    pub fn try_new(
        event_type_defined: bool,
        ordering_defined: bool,
        gap_handling_defined: bool,
        duplicate_handling_defined: bool,
        conflict_resolution_defined: bool,
        lifetime_resource_bound_revalidation_defined: bool,
        failure_reason_mapping_defined: bool,
    ) -> Result<Self, ReplayPolicyCoverageError> {
        if !event_type_defined {
            return Err(ReplayPolicyCoverageError::EventTypeMissing);
        }
        if !ordering_defined {
            return Err(ReplayPolicyCoverageError::OrderingMissing);
        }
        if !gap_handling_defined {
            return Err(ReplayPolicyCoverageError::GapHandlingMissing);
        }
        if !duplicate_handling_defined {
            return Err(ReplayPolicyCoverageError::DuplicateHandlingMissing);
        }
        if !conflict_resolution_defined {
            return Err(ReplayPolicyCoverageError::ConflictResolutionMissing);
        }
        if !lifetime_resource_bound_revalidation_defined {
            return Err(ReplayPolicyCoverageError::BoundRevalidationMissing);
        }
        if !failure_reason_mapping_defined {
            return Err(ReplayPolicyCoverageError::FailureReasonMappingMissing);
        }
        Ok(Self {
            event_type_defined,
            ordering_defined,
            gap_handling_defined,
            duplicate_handling_defined,
            conflict_resolution_defined,
            lifetime_resource_bound_revalidation_defined,
            failure_reason_mapping_defined,
        })
    }
}

/// replay eligibility です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReplayEligibility {
    state_family: StateFamily,
    replay_mode: ReplayMode,
    policy_coverage: Option<ReplayPolicyCoverage>,
}

/// replay eligibility の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReplayEligibilityError {
    /// state family が domain mutation replay の許可対象ではありません。
    StateFamilyNotReplayMutationEligible,
    /// recovery class が domain mutation replay を許可しません。
    RecoveryClassDoesNotPermitDomainMutationReplay,
    /// audit replay は domain mutation の既定経路ではありません。
    DomainMutationRequiresExplicitRestorePolicy,
    /// domain mutation replay には full policy coverage が必要です。
    DomainMutationRequiresPolicyCoverage,
}

impl ReplayEligibility {
    /// audit integrity verification only の replay eligibility を作ります。
    pub const fn audit_integrity_verification_only(state_family: StateFamily) -> Self {
        Self {
            state_family,
            replay_mode: ReplayMode::AuditIntegrityVerificationOnly,
            policy_coverage: None,
        }
    }

    /// explicit restore policy がある場合だけ domain mutation candidate を作ります。
    pub fn domain_mutation_candidate(
        policy: StateFamilyRecoveryPolicy,
        policy_coverage: Option<ReplayPolicyCoverage>,
    ) -> Result<Self, ReplayEligibilityError> {
        if !matches!(policy.state_family, StateFamily::SignalingIdempotency) {
            return Err(ReplayEligibilityError::StateFamilyNotReplayMutationEligible);
        }
        if !policy.recovery_class.permits_core_state_restore_candidate() {
            return Err(ReplayEligibilityError::RecoveryClassDoesNotPermitDomainMutationReplay);
        }
        if !policy.explicit_restore_policy_configured {
            return Err(ReplayEligibilityError::DomainMutationRequiresExplicitRestorePolicy);
        }
        if !policy.permits_domain_mutation_replay_candidate() {
            return Err(ReplayEligibilityError::RecoveryClassDoesNotPermitDomainMutationReplay);
        }
        let policy_coverage =
            policy_coverage.ok_or(ReplayEligibilityError::DomainMutationRequiresPolicyCoverage)?;

        Ok(Self {
            state_family: policy.state_family,
            replay_mode: ReplayMode::DomainMutationCandidate,
            policy_coverage: Some(policy_coverage),
        })
    }
}

/// recovery/restore/replay failure mapping key です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecoveryFailureKind {
    /// persistence read unavailable.
    PersistenceUnavailable,
    /// persisted representation cannot map to core type.
    ExternalDecodeFailed,
    /// persisted schema or version unsupported by driver encoding.
    UnsupportedDriverWireVersion,
    /// required persisted field absent.
    MissingRequiredWireField,
    /// persisted enum has no core mapping.
    ExternalEnumUnmapped,
    /// restore target state conflicts with current state.
    CommandOrderViolation,
    /// retry store bound exceeded during recovery.
    PersistenceRetryBoundExceeded,
    /// retry duration exceeded during recovery.
    PersistenceRetryDurationExceeded,
    /// driver shutdown during restore/replay.
    DriverShutdown,
    /// unclean source state detected.
    UncleanShutdownDetected,
    /// failover claimed without distributed verification.
    FailoverNotProven,
    /// state owner conflict detected during recovery.
    StateOwnerConflict,
}

impl RecoveryFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::PersistenceUnavailable => "persistence_unavailable",
            Self::ExternalDecodeFailed => "external_decode_failed",
            Self::UnsupportedDriverWireVersion => "unsupported_driver_wire_version",
            Self::MissingRequiredWireField => "missing_required_wire_field",
            Self::ExternalEnumUnmapped => "external_enum_unmapped",
            Self::CommandOrderViolation => "command_order_violation",
            Self::PersistenceRetryBoundExceeded => "persistence_retry_bound_exceeded",
            Self::PersistenceRetryDurationExceeded => "persistence_retry_duration_exceeded",
            Self::DriverShutdown => "driver_shutdown",
            Self::UncleanShutdownDetected => "unclean_shutdown_detected",
            Self::FailoverNotProven => "failover_not_proven",
            Self::StateOwnerConflict => "state_owner_conflict",
        }
    }
}

/// recovery 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedRecoveryBehavior {
    /// driver persistence schema becomes domain source-of-truth.
    DriverSchemaAsDomainSourceOfTruth,
    /// audit replay mutates domain state without explicit restore policy.
    AuditReplayMutatesWithoutExplicitRestorePolicy,
    /// SFU route state is silently restored.
    SfuRouteStateSilentlyRestored,
    /// TURN allocation, permission, or channel bind is silently restored.
    TurnRelayStateSilentlyRestored,
    /// SDK local reconnect state is server participant state.
    SdkReconnectStateAsServerParticipantState,
    /// restore conflict is resolved by driver-local preference.
    DriverLocalConflictResolution,
    /// restore success is claimed without checkpoint/replay verification and correlation ID.
    RestoreSuccessWithoutVerification,
    /// crash/restart observation is treated as restore success.
    CrashObservationAsRestoreSuccess,
    /// service discovery fallback or replacement process start is treated as failover success.
    ReplacementProcessAsFailoverSuccess,
    /// replicated/consensus state is inferred from restore/checkpoint behavior.
    ReplicationInferredFromRestore,
}

/// v0.2 initial architecture が認める distributed state class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DistributedStateClass {
    /// state exists only on owning node/process.
    NodeLocalState,
    /// command/packet must reach owning node.
    AffinityRequiredState,
    /// restart recovery candidate under explicit restore policy.
    CheckpointCandidateState,
    /// audit/hash-chain can verify ordering/integrity.
    AuditVerificationState,
    /// state replication is requested.
    ReplicatedStateRequested,
    /// consensus/leader/quorum behavior is requested.
    ConsensusStateRequested,
    /// live failover without explicit restore proof is requested.
    AutomaticFailoverRequested,
}

impl DistributedStateClass {
    /// initial v0.2 で admitted として扱える class かどうかです。
    pub const fn admitted_in_initial_v0_2(self) -> bool {
        match self {
            Self::NodeLocalState
            | Self::AffinityRequiredState
            | Self::CheckpointCandidateState
            | Self::AuditVerificationState => true,
            Self::ReplicatedStateRequested
            | Self::ConsensusStateRequested
            | Self::AutomaticFailoverRequested => false,
        }
    }
}
