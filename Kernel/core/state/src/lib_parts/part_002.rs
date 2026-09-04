/// restore 対象として core/state が認める durable state family です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DurableStateFamily {
    /// Signaling idempotency checkpoint.
    SignalingIdempotency,
    /// audit event restore pointer.
    AuditEvent,
    /// audit hash-chain restore pointer.
    AuditHashChainRecord,
    /// atomicity / compensation state restore pointer.
    AtomicityCompensationEvidence,
    /// bounded resource counter checkpoint.
    ResourceBoundCounters,
    /// configuration decision checkpoint.
    ConfigurationDecision,
}

impl DurableStateFamily {
    /// durable restore が domain source-of-truth 昇格ではない family だけを通します。
    pub const fn admits_restore(self) -> bool {
        match self {
            Self::SignalingIdempotency
            | Self::AuditEvent
            | Self::AuditHashChainRecord
            | Self::AtomicityCompensationEvidence
            | Self::ResourceBoundCounters
            | Self::ConfigurationDecision => true,
        }
    }
}

/// restore checkpoint の opaque reference です。
///
/// filesystem path や object key の形式は driver boundary に閉じ、core/state は値を opaque ref として扱います。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RestoreCheckpointRef {
    value: &'static str,
}

impl RestoreCheckpointRef {
    /// opaque checkpoint ref を作ります。
    pub const fn new(value: &'static str) -> Self {
        Self { value }
    }

    /// opaque checkpoint ref value です。filesystem format ではありません。
    pub const fn as_str(&self) -> &'static str {
        self.value
    }
}

/// state restore failure の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StateRestoreFailureKind {
    /// durable family が restore 対象ではありません。
    FamilyNotDurable,
    /// checkpoint ref がありません。
    CheckpointRefMissing,
    /// restore policy ref がありません。
    RestorePolicyMissing,
    /// checkpoint ref が拒否されました。
    CheckpointRefRejected,
    /// corruption decision が拒否を返しました。
    CorruptionRejected,
}

/// checkpoint payload / metadata の corruption 判定です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StateCorruptionDecision {
    /// corruption は観測されていません。
    Accepted,
    /// corruption により restore は拒否されます。
    Rejected,
}

/// state restore decision です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StateRestoreDecision {
    /// restore checkpoint ref が受理されました。
    Restored(RestoreCheckpointRef),
    /// restore は閉じた理由で拒否されました。
    Rejected(StateRestoreFailureKind),
}

/// state restore 判定入力です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateRestoreInput {
    family: DurableStateFamily,
    checkpoint_ref: Option<RestoreCheckpointRef>,
    restore_policy_ref: Option<&'static str>,
    corruption: StateCorruptionDecision,
}

impl StateRestoreInput {
    /// restore 判定に必要な core-owned refs を束ねます。
    pub const fn new(
        family: DurableStateFamily,
        checkpoint_ref: Option<RestoreCheckpointRef>,
        restore_policy_ref: Option<&'static str>,
        corruption: StateCorruptionDecision,
    ) -> Self {
        Self {
            family,
            checkpoint_ref,
            restore_policy_ref,
            corruption,
        }
    }
}

/// restore checkpoint を domain source-of-truth 昇格なしで判定します。
///
/// core/state は filesystem format、driver schema、object key、DB transaction を知りません。
pub const fn decide_state_restore(input: StateRestoreInput) -> StateRestoreDecision {
    if !input.family.admits_restore() {
        return StateRestoreDecision::Rejected(StateRestoreFailureKind::FamilyNotDurable);
    }

    let Some(checkpoint_ref) = input.checkpoint_ref else {
        return StateRestoreDecision::Rejected(StateRestoreFailureKind::CheckpointRefMissing);
    };

    if checkpoint_ref.as_str().is_empty() {
        return StateRestoreDecision::Rejected(StateRestoreFailureKind::CheckpointRefRejected);
    }

    let Some(restore_policy_ref) = input.restore_policy_ref else {
        return StateRestoreDecision::Rejected(StateRestoreFailureKind::RestorePolicyMissing);
    };

    if restore_policy_ref.is_empty() {
        return StateRestoreDecision::Rejected(StateRestoreFailureKind::RestorePolicyMissing);
    }

    match input.corruption {
        StateCorruptionDecision::Accepted => StateRestoreDecision::Restored(checkpoint_ref),
        StateCorruptionDecision::Rejected => {
            StateRestoreDecision::Rejected(StateRestoreFailureKind::CorruptionRejected)
        }
    }
}
