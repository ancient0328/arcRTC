use arcrtc_core_operation::{
    AtomicCommitBoundary, AtomicLedgerAppendIntent, AtomicLedgerEffectSlot,
    CompensationRequiredReason,
};

/// Ledger append driver へ渡す core-owned input です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerAppendPortInput<Details> {
    append_intent: AtomicLedgerAppendIntent<Details>,
}

impl<Details> LedgerAppendPortInput<Details> {
    /// atomic append intent を port input として包みます。
    pub const fn new(append_intent: AtomicLedgerAppendIntent<Details>) -> Self {
        Self { append_intent }
    }

    /// core/operation が所有する atomic append intent です。
    pub const fn append_intent(&self) -> &AtomicLedgerAppendIntent<Details> {
        &self.append_intent
    }
}

/// Ledger append driver から返す core-owned output です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LedgerAppendPortOutput {
    effect_slot: AtomicLedgerEffectSlot,
    commit_boundary: AtomicCommitBoundary,
    durable_observed: bool,
}

impl LedgerAppendPortOutput {
    /// append / sync が durable observation まで到達したことを port output として表します。
    pub const fn new(
        effect_slot: AtomicLedgerEffectSlot,
        commit_boundary: AtomicCommitBoundary,
        durable_observed: bool,
    ) -> Self {
        Self {
            effect_slot,
            commit_boundary,
            durable_observed,
        }
    }

    /// effect slot です。
    pub const fn effect_slot(&self) -> AtomicLedgerEffectSlot {
        self.effect_slot
    }

    /// commit boundary です。
    pub const fn commit_boundary(&self) -> AtomicCommitBoundary {
        self.commit_boundary
    }

    /// driver が durable observation を返したかです。domain commit proof ではありません。
    pub const fn durable_observed(&self) -> bool {
        self.durable_observed
    }
}

/// Ledger append port failure の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LedgerAppendPortFailureKind {
    /// audit event append に失敗しました。
    AppendFailed,
    /// hash-chain record append と audit event append の関係が一致しません。
    HashChainMismatch,
    /// filesystem path / store path が driver boundary で拒否されました。
    PathRejected,
    /// driver-local serialization に失敗しました。
    SerializationFailed,
    /// durable sync に失敗しました。
    SyncFailed,
    /// driver shutdown により実行できません。
    DriverShutdown,
}

impl LedgerAppendPortFailureKind {
    /// compensation が必要な failure だけを core/operation の閉集合に接続します。
    pub const fn compensation_required_reason(self) -> Option<CompensationRequiredReason> {
        match self {
            Self::AppendFailed => Some(CompensationRequiredReason::AppendFailed),
            Self::HashChainMismatch => Some(CompensationRequiredReason::HashChainMismatch),
            Self::SyncFailed => Some(CompensationRequiredReason::SyncFailed),
            Self::PathRejected | Self::SerializationFailed | Self::DriverShutdown => None,
        }
    }
}
