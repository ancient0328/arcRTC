/// state owner の node/process scope です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OwnerNodeScope {
    /// single node/process.
    SingleNode,
    /// process instance.
    ProcessInstance,
    /// service instance.
    ServiceInstance,
    /// cluster/global scope request.
    ClusterScopeRequested,
}

/// command routing rule です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandRoutingRule {
    /// owning node へ affinity routing が必要です。
    OwnerAffinityRequired,
    /// cross-node command access is rejected.
    CrossNodeRejected,
    /// command is not routed across node boundary.
    NotCrossNodeRouted,
}

/// packet routing rule です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacketRoutingRule {
    /// packet must reach owning node.
    OwnerAffinityRequired,
    /// cross-node packet route/relay is rejected.
    CrossNodeRejected,
    /// not packet scoped.
    NotPacketScoped,
}

/// recovery/restore relation for distributed state evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecoveryRestoreRelation {
    /// no restore relation is claimed.
    NoRestoreRelation,
    /// checkpoint candidate relation under durable recovery Canonical.
    CheckpointCandidate,
    /// replay verification only.
    ReplayVerificationOnly,
    /// explicit restore evidence is required before failover claim.
    ExplicitRestoreEvidenceRequired,
}

/// owner-scoped conflict rule です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DistributedConflictRule {
    /// conflicting owner is rejected.
    RejectConflictingOwner,
    /// drain and mark close-not-claimed.
    DrainAndCloseNotClaimed,
    /// admitted consensus/conflict rule is required; initial v0.2 では成立しません。
    RequiresFutureConsensusAdmission,
}

/// audit event relation です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DistributedAuditRelation {
    /// distributed_state_failover_decision event required.
    DistributedStateFailoverDecision,
    /// audit/hash-chain verification relation only.
    AuditVerificationOnly,
}

/// distributed state policy の core-owned shape です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DistributedStatePolicy {
    state_family: StateFamily,
    distributed_state_class: DistributedStateClass,
    owner_scope: OwnerNodeScope,
    owner_ref: OpaqueReference,
    affinity_key: Option<OpaqueReference>,
    command_routing_rule: CommandRoutingRule,
    packet_routing_rule: PacketRoutingRule,
    recovery_restore_relation: RecoveryRestoreRelation,
    conflict_rule: DistributedConflictRule,
    failure_reason_mapping_cataloged: bool,
    audit_event_relation: DistributedAuditRelation,
}

/// distributed state policy の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DistributedStatePolicyError {
    /// owner node/process reference がありません。
    OwnerReferenceMissing,
    /// cluster/global scope request は initial v0.2 では admitted できません。
    ClusterScopeNotAdmitted,
    /// affinity-required state に affinity key がありません。
    AffinityKeyMissing,
    /// future consensus 前提の conflict rule は initial v0.2 では admitted できません。
    FutureConsensusConflictRuleNotAdmitted,
    /// failure reason mapping が cataloged reason へ接続されていません。
    FailureReasonMappingMissing,
}

impl DistributedStatePolicy {
    /// distributed state policy を作ります。
    pub fn try_new(
        state_family: StateFamily,
        distributed_state_class: DistributedStateClass,
        owner_scope: OwnerNodeScope,
        owner_ref: Option<OpaqueReference>,
        affinity_key: Option<OpaqueReference>,
        command_routing_rule: CommandRoutingRule,
        packet_routing_rule: PacketRoutingRule,
        recovery_restore_relation: RecoveryRestoreRelation,
        conflict_rule: DistributedConflictRule,
        failure_reason_mapping_cataloged: bool,
        audit_event_relation: DistributedAuditRelation,
    ) -> Result<Self, DistributedStatePolicyError> {
        let owner_ref = owner_ref.ok_or(DistributedStatePolicyError::OwnerReferenceMissing)?;
        if matches!(owner_scope, OwnerNodeScope::ClusterScopeRequested) {
            return Err(DistributedStatePolicyError::ClusterScopeNotAdmitted);
        }
        if matches!(
            conflict_rule,
            DistributedConflictRule::RequiresFutureConsensusAdmission
        ) {
            return Err(DistributedStatePolicyError::FutureConsensusConflictRuleNotAdmitted);
        }

        let affinity_is_required = matches!(
            distributed_state_class,
            DistributedStateClass::AffinityRequiredState
        ) || matches!(
            command_routing_rule,
            CommandRoutingRule::OwnerAffinityRequired
        ) || matches!(
            packet_routing_rule,
            PacketRoutingRule::OwnerAffinityRequired
        );

        if affinity_is_required && affinity_key.is_none() {
            return Err(DistributedStatePolicyError::AffinityKeyMissing);
        }
        if !failure_reason_mapping_cataloged {
            return Err(DistributedStatePolicyError::FailureReasonMappingMissing);
        }

        Ok(Self {
            state_family,
            distributed_state_class,
            owner_scope,
            owner_ref,
            affinity_key,
            command_routing_rule,
            packet_routing_rule,
            recovery_restore_relation,
            conflict_rule,
            failure_reason_mapping_cataloged,
            audit_event_relation,
        })
    }
}

/// initial v0.2 での distributed state admission です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DistributedStateAdmission {
    policy: DistributedStatePolicy,
}

/// distributed state admission の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DistributedStateAdmissionError {
    /// distributed state class is not admitted.
    DistributedStateNotAdmitted,
    /// state replication requested without admitted policy.
    StateReplicationNotAdmitted,
    /// consensus or leader election requested without admitted policy.
    ConsensusNotAdmitted,
    /// automatic failover requested without admitted policy/evidence.
    FailoverNotProven,
}

impl DistributedStateAdmission {
    /// initial v0.2 で admitted な distributed state policy だけを受理します。
    pub fn admit_initial_v0_2(
        policy: DistributedStatePolicy,
    ) -> Result<Self, DistributedStateAdmissionError> {
        match policy.distributed_state_class {
            DistributedStateClass::ReplicatedStateRequested => {
                Err(DistributedStateAdmissionError::StateReplicationNotAdmitted)
            }
            DistributedStateClass::ConsensusStateRequested => {
                Err(DistributedStateAdmissionError::ConsensusNotAdmitted)
            }
            DistributedStateClass::AutomaticFailoverRequested => {
                Err(DistributedStateAdmissionError::FailoverNotProven)
            }
            admitted if admitted.admitted_in_initial_v0_2() => Ok(Self { policy }),
            _ => Err(DistributedStateAdmissionError::DistributedStateNotAdmitted),
        }
    }
}

/// failover claim の class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FailoverClaimClass {
    /// process restart observation only.
    ProcessRestartObservation,
    /// endpoint resolution or service discovery fallback only.
    ServiceDiscoveryFallback,
    /// health/readiness probe success only.
    HealthProbeSuccess,
    /// explicit evidence-backed failover candidate.
    EvidenceBackedFailoverCandidate,
}

/// failover evidence に必要な field shape です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FailoverEvidenceShape {
    failed_owner_observed: bool,
    replacement_owner: OpaqueReference,
    affected_state_family: StateFamily,
    affinity_sticky_routing_updated: bool,
    restore_replay_relation: RecoveryRestoreRelation,
    conflict_and_duplicate_handling_defined: bool,
    resource_lifetime_revalidated: bool,
    audit_continuity_or_close_not_claimed_scope_recorded: bool,
}

/// failover evidence shape 生成時の未検査入力です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FailoverEvidenceShapeInput {
    pub failed_owner_observed: bool,
    pub replacement_owner: Option<OpaqueReference>,
    pub affected_state_family: StateFamily,
    pub affinity_sticky_routing_updated: bool,
    pub restore_replay_relation: RecoveryRestoreRelation,
    pub conflict_and_duplicate_handling_defined: bool,
    pub resource_lifetime_revalidated: bool,
    pub audit_continuity_or_close_not_claimed_scope_recorded: bool,
}

/// failover evidence shape の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FailoverEvidenceShapeError {
    /// failed owner/node observation がありません。
    FailedOwnerObservationMissing,
    /// replacement owner/node がありません。
    ReplacementOwnerMissing,
    /// affinity/sticky routing update が記録されていません。
    AffinityRoutingUpdateMissing,
    /// conflict and duplicate handling が定義されていません。
    ConflictDuplicateHandlingMissing,
    /// resource/lifetime revalidation が記録されていません。
    ResourceLifetimeRevalidationMissing,
    /// audit continuity または close-not-claimed scope が記録されていません。
    AuditContinuityOrCloseNotClaimedScopeMissing,
}

impl FailoverEvidenceShape {
    /// failover evidence の最小 field set を作ります。
    pub fn try_new(input: FailoverEvidenceShapeInput) -> Result<Self, FailoverEvidenceShapeError> {
        let FailoverEvidenceShapeInput {
            failed_owner_observed,
            replacement_owner,
            affected_state_family,
            affinity_sticky_routing_updated,
            restore_replay_relation,
            conflict_and_duplicate_handling_defined,
            resource_lifetime_revalidated,
            audit_continuity_or_close_not_claimed_scope_recorded,
        } = input;

        if !failed_owner_observed {
            return Err(FailoverEvidenceShapeError::FailedOwnerObservationMissing);
        }
        let replacement_owner =
            replacement_owner.ok_or(FailoverEvidenceShapeError::ReplacementOwnerMissing)?;
        if !affinity_sticky_routing_updated {
            return Err(FailoverEvidenceShapeError::AffinityRoutingUpdateMissing);
        }
        if !conflict_and_duplicate_handling_defined {
            return Err(FailoverEvidenceShapeError::ConflictDuplicateHandlingMissing);
        }
        if !resource_lifetime_revalidated {
            return Err(FailoverEvidenceShapeError::ResourceLifetimeRevalidationMissing);
        }
        if !audit_continuity_or_close_not_claimed_scope_recorded {
            return Err(FailoverEvidenceShapeError::AuditContinuityOrCloseNotClaimedScopeMissing);
        }

        Ok(Self {
            failed_owner_observed,
            replacement_owner,
            affected_state_family,
            affinity_sticky_routing_updated,
            restore_replay_relation,
            conflict_and_duplicate_handling_defined,
            resource_lifetime_revalidated,
            audit_continuity_or_close_not_claimed_scope_recorded,
        })
    }
}

/// failover admission です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FailoverAdmission {
    claim_class: FailoverClaimClass,
    evidence: FailoverEvidenceShape,
}

/// failover admission の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FailoverAdmissionError {
    /// process restart observation is not failover success.
    ProcessRestartIsNotFailoverSuccess,
    /// service discovery fallback is not failover success.
    ServiceDiscoveryFallbackIsNotFailoverSuccess,
    /// health/readiness success is not failover success.
    HealthProbeIsNotFailoverSuccess,
}

impl FailoverAdmission {
    /// evidence-backed failover candidate だけを admission します。
    pub fn admit(
        claim_class: FailoverClaimClass,
        evidence: FailoverEvidenceShape,
    ) -> Result<Self, FailoverAdmissionError> {
        match claim_class {
            FailoverClaimClass::EvidenceBackedFailoverCandidate => Ok(Self {
                claim_class,
                evidence,
            }),
            FailoverClaimClass::ProcessRestartObservation => {
                Err(FailoverAdmissionError::ProcessRestartIsNotFailoverSuccess)
            }
            FailoverClaimClass::ServiceDiscoveryFallback => {
                Err(FailoverAdmissionError::ServiceDiscoveryFallbackIsNotFailoverSuccess)
            }
            FailoverClaimClass::HealthProbeSuccess => {
                Err(FailoverAdmissionError::HealthProbeIsNotFailoverSuccess)
            }
        }
    }
}

/// split-brain guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SplitBrainGuard {
    two_nodes_can_accept_same_owner_scope: bool,
    admitted_conflict_or_consensus_rule_present: bool,
}

/// split-brain guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SplitBrainGuardError {
    /// two owners can accept same owner-scoped state without rule.
    SplitBrainRiskDetected,
}

impl SplitBrainGuard {
    /// split-brain risk を fail-closed に評価します。
    pub fn try_new(
        two_nodes_can_accept_same_owner_scope: bool,
        admitted_conflict_or_consensus_rule_present: bool,
    ) -> Result<Self, SplitBrainGuardError> {
        if two_nodes_can_accept_same_owner_scope && !admitted_conflict_or_consensus_rule_present {
            return Err(SplitBrainGuardError::SplitBrainRiskDetected);
        }

        Ok(Self {
            two_nodes_can_accept_same_owner_scope,
            admitted_conflict_or_consensus_rule_present,
        })
    }
}

/// distributed_state_failover_decision audit event の core-owned shape です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DistributedStateFailoverAuditShape {
    startup_run_id: StartupRunId,
    correlation_id: Option<CorrelationId>,
    distributed_state_class: DistributedStateClass,
    state_family: StateFamily,
    owner_scope: OwnerNodeScope,
    owner_ref: OpaqueReference,
    affinity_key: Option<OpaqueReference>,
    failover_class: FailoverClaimClass,
    replacement_owner: Option<OpaqueReference>,
    cataloged_reason: &'static str,
}

impl DistributedStateFailoverAuditShape {
    /// distributed_state_failover_decision audit shape を作ります。
    pub const fn new(
        startup_run_id: StartupRunId,
        correlation_id: Option<CorrelationId>,
        distributed_state_class: DistributedStateClass,
        state_family: StateFamily,
        owner_scope: OwnerNodeScope,
        owner_ref: OpaqueReference,
        affinity_key: Option<OpaqueReference>,
        failover_class: FailoverClaimClass,
        replacement_owner: Option<OpaqueReference>,
        cataloged_reason: &'static str,
    ) -> Self {
        Self {
            startup_run_id,
            correlation_id,
            distributed_state_class,
            state_family,
            owner_scope,
            owner_ref,
            affinity_key,
            failover_class,
            replacement_owner,
            cataloged_reason,
        }
    }
}

/// distributed state / failover failure mapping key です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DistributedStateFailureKind {
    /// distributed state class is not admitted.
    DistributedStateNotAdmitted,
    /// state replication requested without admitted policy.
    StateReplicationNotAdmitted,
    /// consensus or leader election requested without admitted policy.
    ConsensusNotAdmitted,
    /// automatic failover requested without admitted policy/evidence.
    FailoverNotProven,
    /// node affinity is required but absent.
    NodeAffinityRequired,
    /// owner node/state is unavailable.
    NodeStateUnavailable,
    /// cross-node route or relay is not allowed.
    CrossNodeRouteNotAllowed,
    /// two owners conflict for the same state scope.
    StateOwnerConflict,
    /// split-brain risk is detected.
    SplitBrainRiskDetected,
    /// replication lag or handoff window exceeds bound.
    ReplicationLagBoundExceeded,
}

impl DistributedStateFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::DistributedStateNotAdmitted => "distributed_state_not_admitted",
            Self::StateReplicationNotAdmitted => "state_replication_not_admitted",
            Self::ConsensusNotAdmitted => "consensus_not_admitted",
            Self::FailoverNotProven => "failover_not_proven",
            Self::NodeAffinityRequired => "node_affinity_required",
            Self::NodeStateUnavailable => "node_state_unavailable",
            Self::CrossNodeRouteNotAllowed => "cross_node_route_not_allowed",
            Self::StateOwnerConflict => "state_owner_conflict",
            Self::SplitBrainRiskDetected => "split_brain_risk_detected",
            Self::ReplicationLagBoundExceeded => "replication_lag_bound_exceeded",
        }
    }
}

/// distributed state 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedDistributedStateBehavior {
    /// node-local state is treated as cluster-global by default.
    NodeLocalStateAsClusterGlobalDefault,
    /// service discovery fallback is treated as failover success.
    ServiceDiscoveryFallbackAsFailoverSuccess,
    /// health/readiness success is treated as state handoff success.
    HealthSuccessAsStateHandoffSuccess,
    /// audit replay mutates state without restore policy.
    AuditReplayMutatesWithoutRestorePolicy,
    /// two nodes accept owner-scoped commands without conflict/consensus rule.
    TwoOwnersAcceptedWithoutConflictRule,
    /// replication lag or handoff window is unbounded.
    UnboundedReplicationLagOrHandoffWindow,
    /// test fake multi-node behavior is used as production distributed state evidence.
    TestFakeAsProductionDistributedStateEvidence,
}

/// v0.2 initial architecture が認める process failure class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProcessFailureClass {
    /// runtime/application panic observed.
    PanicObserved,
    /// runtime task/worker panic observed.
    TaskPanicObserved,
    /// process exits unexpectedly.
    ProcessCrashObserved,
    /// shutdown lacks drain/audit completion evidence.
    UncleanShutdownDetected,
    /// external supervisor restarted process.
    SupervisorRestartObserved,
    /// entrypoint starts after prior unclean exit.
    StartupAfterUncleanExit,
    /// controlled crash/restart test report.
    CrashRecoveryEvidence,
}

impl ProcessFailureClass {
    /// domain state claim の前に restore policy/evidence が必要な failure class です。
    pub const fn requires_restore_policy_before_state_claim(self) -> bool {
        match self {
            Self::PanicObserved
            | Self::ProcessCrashObserved
            | Self::UncleanShutdownDetected
            | Self::SupervisorRestartObserved
            | Self::StartupAfterUncleanExit
            | Self::CrashRecoveryEvidence => true,
            Self::TaskPanicObserved => false,
        }
    }

    /// task panic を process crash と同一視できるかどうかです。
    pub const fn is_process_crash(self) -> bool {
        matches!(self, Self::ProcessCrashObserved)
    }
}

