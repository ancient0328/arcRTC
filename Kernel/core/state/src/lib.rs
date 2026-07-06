//! core/state は state persistence policy の core-owned surface です。
//!
//! driver schema、DB transaction、file/object layout はここに置かず、
//! state class、checkpoint intent、audit-only relation、source-of-truth 境界だけを定義します。
include!("lib_parts/part_002.rs");

/// core state package の所有境界を示す marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreStateSurface;

/// v0.2 initial architecture が認める state class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StateClass {
    /// runtime memory 上の domain state.
    EphemeralCoreState,
    /// restart/recovery のため保存可能な core-owned snapshot intent.
    CheckpointEligibleState,
    /// decision/evidence として audit/hash-chain に残す state.
    AuditOnlyState,
    /// socket, buffer, retry queue, external client/session detail.
    DriverLocalState,
    /// startup/wiring/config validation scope.
    ConfigurationScopeState,
    /// SDK connection/client state.
    SdkLocalState,
}

/// state persistence rule です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PersistenceRule {
    /// persistence required ではありません。
    PersistenceNotRequired,
    /// PersistencePort 経由でのみ保存できます。
    PersistOnlyViaPersistencePort,
    /// AuditSinkPort / hash-chain record 経由です。
    AuditSinkOrHashChainOnly,
    /// driver-owned and not core source-of-truth.
    DriverOwnedNotSourceOfTruth,
    /// configuration decision and startup references.
    ConfigurationDecisionStartupReferences,
    /// SDK-owned and not server/core source-of-truth.
    SdkOwnedNotServerState,
}

impl StateClass {
    /// state class ごとの persistence rule です。
    pub const fn persistence_rule(self) -> PersistenceRule {
        match self {
            Self::EphemeralCoreState => PersistenceRule::PersistenceNotRequired,
            Self::CheckpointEligibleState => PersistenceRule::PersistOnlyViaPersistencePort,
            Self::AuditOnlyState => PersistenceRule::AuditSinkOrHashChainOnly,
            Self::DriverLocalState => PersistenceRule::DriverOwnedNotSourceOfTruth,
            Self::ConfigurationScopeState => {
                PersistenceRule::ConfigurationDecisionStartupReferences
            }
            Self::SdkLocalState => PersistenceRule::SdkOwnedNotServerState,
        }
    }

    /// core source-of-truth として扱える class かどうかです。
    pub const fn can_be_core_source_of_truth(self) -> bool {
        match self {
            Self::EphemeralCoreState
            | Self::CheckpointEligibleState
            | Self::AuditOnlyState
            | Self::ConfigurationScopeState => true,
            Self::DriverLocalState | Self::SdkLocalState => false,
        }
    }
}

/// aggregate/state family の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StateFamily {
    /// Signaling room state.
    SignalingRoom,
    /// Signaling participant state.
    SignalingParticipant,
    /// Signaling idempotency state.
    SignalingIdempotency,
    /// SFU session / endpoint / publication / subscription / route state.
    SfuForwardingState,
    /// TURN allocation / permission / channel bind state.
    TurnRelayAuthorizationState,
    /// Audit event.
    AuditEvent,
    /// Audit hash-chain record.
    AuditHashChainRecord,
    /// Atomicity/compensation evidence.
    AtomicityCompensationEvidence,
    /// Resource bound counters.
    ResourceBoundCounters,
    /// Driver retry store.
    DriverRetryStore,
    /// Metrics export backlog.
    MetricsBacklog,
    /// Configuration decision.
    ConfigurationDecision,
    /// SDK connection state.
    SdkConnectionState,
}

impl StateFamily {
    /// initial v0.2 の default state class です。
    pub const fn default_class(self) -> StateClass {
        match self {
            Self::SignalingRoom | Self::SignalingParticipant | Self::SfuForwardingState => {
                StateClass::EphemeralCoreState
            }
            Self::SignalingIdempotency => StateClass::CheckpointEligibleState,
            Self::TurnRelayAuthorizationState => StateClass::EphemeralCoreState,
            Self::AuditEvent | Self::AuditHashChainRecord | Self::AtomicityCompensationEvidence => {
                StateClass::AuditOnlyState
            }
            Self::ResourceBoundCounters => StateClass::EphemeralCoreState,
            Self::DriverRetryStore | Self::MetricsBacklog => StateClass::DriverLocalState,
            Self::ConfigurationDecision => StateClass::ConfigurationScopeState,
            Self::SdkConnectionState => StateClass::SdkLocalState,
        }
    }

    /// implicit durable domain source-of-truth が許可されているかどうかです。
    pub const fn implicit_durable_source_of_truth_allowed(self) -> bool {
        match self {
            Self::SignalingRoom
            | Self::SignalingParticipant
            | Self::SfuForwardingState
            | Self::TurnRelayAuthorizationState
            | Self::DriverRetryStore
            | Self::MetricsBacklog
            | Self::SdkConnectionState => false,
            Self::SignalingIdempotency
            | Self::AuditEvent
            | Self::AuditHashChainRecord
            | Self::AtomicityCompensationEvidence
            | Self::ResourceBoundCounters
            | Self::ConfigurationDecision => false,
        }
    }
}

/// checkpoint intent の owner boundary です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CheckpointOwnerBoundary {
    /// core owns checkpoint intent and version.
    CoreIntentAndVersion,
    /// driver owns schema/table/key/object/file layout.
    DriverSchemaAndStorageLayout,
}

/// checkpoint intent です。checkpoint は source-of-truth 昇格ではありません。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CheckpointIntent {
    state_family: StateFamily,
    state_class: StateClass,
    owner_boundary: CheckpointOwnerBoundary,
    restore_policy_required: bool,
    distributed_policy_required_for_failover: bool,
}

/// checkpoint intent の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CheckpointIntentError {
    /// checkpoint requires checkpoint-eligible state.
    StateClassNotCheckpointEligible,
    /// checkpoint restore policy must be explicit.
    RestorePolicyRequired,
}

impl CheckpointIntent {
    /// checkpoint intent を作ります。
    pub fn try_new(
        state_family: StateFamily,
        state_class: StateClass,
        owner_boundary: CheckpointOwnerBoundary,
        restore_policy_required: bool,
        distributed_policy_required_for_failover: bool,
    ) -> Result<Self, CheckpointIntentError> {
        if !matches!(state_class, StateClass::CheckpointEligibleState) {
            return Err(CheckpointIntentError::StateClassNotCheckpointEligible);
        }
        if !restore_policy_required {
            return Err(CheckpointIntentError::RestorePolicyRequired);
        }

        Ok(Self {
            state_family,
            state_class,
            owner_boundary,
            restore_policy_required,
            distributed_policy_required_for_failover,
        })
    }
}

/// source-of-truth claim class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceOfTruthClaim {
    /// no implicit durable domain source-of-truth.
    NoImplicitDurableDomainSourceOfTruth,
    /// audit replay verification only.
    AuditReplayVerificationOnly,
    /// restore policy required before domain state creation.
    RestorePolicyRequired,
    /// distributed policy required before replication/failover claim.
    DistributedPolicyRequired,
}

/// state persistence failure mapping key です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatePersistenceFailureKind {
    /// persistence unavailable.
    PersistenceUnavailable,
    /// retry store bound exceeded.
    PersistenceRetryBoundExceeded,
    /// retry duration exceeded.
    PersistenceRetryDurationExceeded,
    /// audit backlog exceeded.
    AuditBacklogBoundExceeded,
    /// driver shutdown.
    DriverShutdown,
}

impl StatePersistenceFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::PersistenceUnavailable => "persistence_unavailable",
            Self::PersistenceRetryBoundExceeded => "persistence_retry_bound_exceeded",
            Self::PersistenceRetryDurationExceeded => "persistence_retry_duration_exceeded",
            Self::AuditBacklogBoundExceeded => "audit_backlog_bound_exceeded",
            Self::DriverShutdown => "driver_shutdown",
        }
    }
}

/// state persistence 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedStatePersistenceBehavior {
    /// driver persistence schema becomes domain source-of-truth.
    DriverSchemaAsDomainSourceOfTruth,
    /// checkpoint restore creates domain state without restore Canonical.
    CheckpointRestoreWithoutRestoreCanonical,
    /// SFU route state is durable by default.
    SfuRouteDurableByDefault,
    /// TURN allocation is silently restored from driver storage.
    TurnAllocationSilentlyRestored,
    /// audit log is treated as mutable state store.
    AuditLogAsMutableStateStore,
    /// driver DB transaction is aggregate commit authority.
    DriverDbTransactionAsAggregateCommitAuthority,
    /// SDK local connection state is server-side participant state.
    SdkLocalStateAsServerParticipantState,
    /// driver retry queue is unbounded or used as domain state.
    DriverRetryQueueAsDomainState,
    /// persisted state is replicated/failover-ready without distributed state policy.
    PersistedStateAsFailoverReadyWithoutPolicy,
}
