impl PersistencePortIntent {
    /// persistence intent を作ります。
    pub fn try_new(
        intent_class: PersistenceIntentClass,
        operation: PersistenceOperationKind,
        state_family: StateFamily,
        state_class: StateClass,
        consistency_requirements: Vec<PersistenceConsistencyRequirement>,
        correlation_id: Option<CorrelationId>,
    ) -> Result<Self, PersistencePortIntentError> {
        if state_class != state_family.default_class() {
            return Err(PersistencePortIntentError::StateFamilyClassPolicyMismatch);
        }

        match intent_class {
            PersistenceIntentClass::StateCheckpoint => {
                if !matches!(
                    operation,
                    PersistenceOperationKind::PersistCheckpoint
                        | PersistenceOperationKind::LoadCheckpoint
                ) {
                    return Err(PersistencePortIntentError::OperationClassMismatch);
                }
                if !matches!(state_class, StateClass::CheckpointEligibleState) {
                    return Err(
                        PersistencePortIntentError::CheckpointRequiresCheckpointEligibleState,
                    );
                }
            }
            PersistenceIntentClass::AuditPersistence => {
                if !matches!(operation, PersistenceOperationKind::AppendAuditEvent) {
                    return Err(PersistencePortIntentError::OperationClassMismatch);
                }
                if !matches!(state_class, StateClass::AuditOnlyState) {
                    return Err(PersistencePortIntentError::AuditIntentRequiresAuditOnlyState);
                }
                if !matches!(
                    state_family,
                    StateFamily::AuditEvent | StateFamily::AtomicityCompensationEvidence
                ) {
                    return Err(
                        PersistencePortIntentError::AuditPersistenceRequiresAuditEventState,
                    );
                }
            }
            PersistenceIntentClass::HashChainRecordPersistence => {
                if !matches!(operation, PersistenceOperationKind::AppendHashChainRecord) {
                    return Err(PersistencePortIntentError::OperationClassMismatch);
                }
                if !matches!(state_class, StateClass::AuditOnlyState) {
                    return Err(PersistencePortIntentError::AuditIntentRequiresAuditOnlyState);
                }
                if !matches!(state_family, StateFamily::AuditHashChainRecord) {
                    return Err(
                        PersistencePortIntentError::HashChainPersistenceRequiresHashChainState,
                    );
                }
            }
            PersistenceIntentClass::RetryStore => {
                if !matches!(
                    operation,
                    PersistenceOperationKind::EnqueueRetry
                        | PersistenceOperationKind::DequeueRetry
                        | PersistenceOperationKind::AcknowledgeRetry
                ) {
                    return Err(PersistencePortIntentError::OperationClassMismatch);
                }
                if !matches!(state_class, StateClass::DriverLocalState) {
                    return Err(PersistencePortIntentError::RetryIntentRequiresDriverLocalState);
                }
                if !matches!(state_family, StateFamily::DriverRetryStore) {
                    return Err(PersistencePortIntentError::RetryIntentRequiresDriverRetryStore);
                }
                if !consistency_requirements.iter().any(|requirement| {
                    matches!(
                        requirement,
                        PersistenceConsistencyRequirement::BoundedRetryStore
                    )
                }) {
                    return Err(PersistencePortIntentError::RetryIntentRequiresBoundedRetry);
                }
            }
        }

        Ok(Self {
            intent_class,
            operation,
            state_family,
            state_class,
            consistency_requirements,
            correlation_id,
        })
    }

    /// intent class です。
    pub const fn intent_class(&self) -> PersistenceIntentClass {
        self.intent_class
    }

    /// state class です。
    pub const fn state_class(&self) -> StateClass {
        self.state_class
    }
}

/// persistence driver が返す core-owned record reference です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PersistenceRecordRef(OpaqueReference);

impl PersistenceRecordRef {
    /// accepted opaque reference から persistence record reference を作ります。
    pub const fn new(reference: OpaqueReference) -> Self {
        Self(reference)
    }

    /// opaque value です。schema/table/key として解釈してはいけません。
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// persistence write / append / retry operation の acknowledgement です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistenceAcknowledgement {
    intent_class: PersistenceIntentClass,
    record_ref: Option<PersistenceRecordRef>,
    durable_observed: bool,
}

impl PersistenceAcknowledgement {
    /// acknowledgement を作ります。domain commit success ではありません。
    pub const fn new(
        intent_class: PersistenceIntentClass,
        record_ref: Option<PersistenceRecordRef>,
        durable_observed: bool,
    ) -> Self {
        Self {
            intent_class,
            record_ref,
            durable_observed,
        }
    }
}

/// loaded state は raw row/object ではなく core-owned reference として返します。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedCoreStateRef {
    state_family: StateFamily,
    state_class: StateClass,
    state_ref: PersistenceRecordRef,
}

impl LoadedCoreStateRef {
    /// loaded state reference を作ります。
    pub const fn new(
        state_family: StateFamily,
        state_class: StateClass,
        state_ref: PersistenceRecordRef,
    ) -> Self {
        Self {
            state_family,
            state_class,
            state_ref,
        }
    }
}

/// core-owned PersistencePort input です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistencePortInput {
    /// checkpoint / audit / hash-chain / retry intent を実行します。
    ExecuteIntent(PersistencePortIntent),
}

/// core-owned PersistencePort output です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistencePortOutput {
    /// persistence acknowledgement です。domain commit proof ではありません。
    Acknowledgement(PersistenceAcknowledgement),
    /// loaded checkpoint/state reference です。
    LoadedState(LoadedCoreStateRef),
}

/// PersistencePort failure の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PersistencePortFailureKind {
    /// concrete persistence unavailable.
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

impl PersistencePortFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::PersistenceUnavailable => {
                StatePersistenceFailureKind::PersistenceUnavailable.reason_code()
            }
            Self::PersistenceRetryBoundExceeded => {
                StatePersistenceFailureKind::PersistenceRetryBoundExceeded.reason_code()
            }
            Self::PersistenceRetryDurationExceeded => {
                StatePersistenceFailureKind::PersistenceRetryDurationExceeded.reason_code()
            }
            Self::AuditBacklogBoundExceeded => {
                StatePersistenceFailureKind::AuditBacklogBoundExceeded.reason_code()
            }
            Self::DriverShutdown => StatePersistenceFailureKind::DriverShutdown.reason_code(),
        }
    }
}

/// closed reason と resource-bound mapping に接続した PersistencePort failure です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PersistencePortFailure {
    kind: PersistencePortFailureKind,
    reason: CatalogedReasonRef,
    resource_bound_decision: Option<ResourceBoundDecision>,
}

impl PersistencePortFailure {
    /// persistence port failure を作ります。
    pub fn from_kind(kind: PersistencePortFailureKind) -> Self {
        let reason = CatalogedReasonRef::from_code(kind.reason_code())
            .expect("persistence port failure reason code must be registered");
        let resource_bound_decision = match kind {
            PersistencePortFailureKind::PersistenceRetryBoundExceeded => {
                Some(persistence_resource_bound_decision(
                    ResourceBoundKind::PersistenceRetryStore,
                    kind.reason_code(),
                ))
            }
            PersistencePortFailureKind::PersistenceRetryDurationExceeded => {
                Some(persistence_resource_bound_decision(
                    ResourceBoundKind::PersistenceRetryStore,
                    kind.reason_code(),
                ))
            }
            PersistencePortFailureKind::AuditBacklogBoundExceeded => {
                Some(persistence_resource_bound_decision(
                    ResourceBoundKind::AuditSinkBacklog,
                    kind.reason_code(),
                ))
            }
            PersistencePortFailureKind::PersistenceUnavailable
            | PersistencePortFailureKind::DriverShutdown => None,
        };

        Self {
            kind,
            reason,
            resource_bound_decision,
        }
    }

    /// failure kind です。
    pub const fn kind(&self) -> PersistencePortFailureKind {
        self.kind
    }

    /// cataloged reason reference です。
    pub const fn reason(&self) -> CatalogedReasonRef {
        self.reason
    }

    /// resource-bound failure の required audit mapping です。
    pub const fn resource_bound_decision(&self) -> Option<&ResourceBoundDecision> {
        self.resource_bound_decision.as_ref()
    }
}

fn persistence_resource_bound_decision(
    resource: ResourceBoundKind,
    reason_code: &'static str,
) -> ResourceBoundDecision {
    let closed_action = REQUIRED_RESOURCE_BOUND_CLOSED_ACTIONS
        .iter()
        .find(|action| action.resource() == resource && action.reason_code() == reason_code)
        .copied()
        .expect("persistence resource bound must be present in core quality catalog");

    ResourceBoundDecision::try_new(closed_action, ResourceBoundReferenceSet::none())
        .expect("persistence resource bound references must satisfy canonical shape")
}

/// persistence intent が扱う state class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PersistenceStateClass {
    /// source-of-truth data です。
    SourceOfTruth,
    /// checkpoint data です。
    Checkpoint,
    /// audit-only data です。
    AuditOnly,
    /// driver retry data です。
    DriverRetryData,
}

/// PacketViewPort が返せる view class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacketViewClass {
    /// driver lease より長く生存しない borrowed semantic header view です。
    BorrowedSemanticHeaderView,
}

/// RuntimePort が返せる opaque reference / observation class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimePortOutputClass {
    /// concrete runtime handle ではない schedule reference です。
    OpaqueScheduleReference,
    /// cancellation observation です。
    CancellationObservation,
    /// execution observation です。
    ExecutionObservation,
}
