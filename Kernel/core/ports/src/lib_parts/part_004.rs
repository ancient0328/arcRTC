use arcrtc_core_state::{
    DurableStateFamily, RestoreCheckpointRef, StateCorruptionDecision, StateRestoreDecision,
};

/// State restore driver へ渡す core-owned input です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateRestorePortInput {
    family: DurableStateFamily,
    checkpoint_ref: RestoreCheckpointRef,
}

impl StateRestorePortInput {
    /// durable state family と opaque checkpoint ref を port input として包みます。
    pub const fn new(family: DurableStateFamily, checkpoint_ref: RestoreCheckpointRef) -> Self {
        Self {
            family,
            checkpoint_ref,
        }
    }

    /// restore target family です。
    pub const fn family(&self) -> DurableStateFamily {
        self.family
    }

    /// opaque checkpoint ref です。filesystem path ではありません。
    pub const fn checkpoint_ref(&self) -> RestoreCheckpointRef {
        self.checkpoint_ref
    }
}

/// State restore driver から返す core-owned output です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateRestorePortOutput {
    decision: StateRestoreDecision,
    corruption: StateCorruptionDecision,
}

impl StateRestorePortOutput {
    /// core/state が所有する restore decision と corruption decision を port output として束ねます。
    pub const fn new(
        decision: StateRestoreDecision,
        corruption: StateCorruptionDecision,
    ) -> Self {
        Self {
            decision,
            corruption,
        }
    }

    /// restore decision です。
    pub const fn decision(&self) -> StateRestoreDecision {
        self.decision
    }

    /// corruption decision です。
    pub const fn corruption(&self) -> StateCorruptionDecision {
        self.corruption
    }
}

/// State restore port failure の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StateRestorePortFailureKind {
    /// checkpoint が driver boundary で見つかりません。
    CheckpointUnavailable,
    /// checkpoint payload の decode が driver boundary で拒否されました。
    CheckpointDecodeRejected,
    /// restore payload digest がありません。
    PayloadDigestMissing,
    /// driver shutdown により restore を実行できません。
    DriverShutdown,
}
