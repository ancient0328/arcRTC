use arcrtc_core_audit::AuditLedgerRecord;

/// ledger append が占有する atomic effect slot です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AtomicLedgerEffectSlot {
    /// audit ledger append の effect slot です。
    AuditLedgerAppend,
}

/// audit ledger append の atomic commit boundary です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AtomicCommitBoundary {
    /// append intent を生成し、driver execution 前の準備状態に置きます。
    Prepare,
    /// driver execution の durable observation を commit boundary として扱います。
    Commit,
}

/// append failure 後に compensation が必要な理由の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompensationRequiredReason {
    /// append 実行が失敗しました。
    AppendFailed,
    /// sync 実行が失敗しました。
    SyncFailed,
    /// hash-chain relation が一致しません。
    HashChainMismatch,
}

/// audit ledger append を atomic effect slot 付きで driver boundary へ渡す intent です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomicLedgerAppendIntent<Details> {
    record: AuditLedgerRecord<Details>,
    effect_slot: AtomicLedgerEffectSlot,
    commit_boundary: AtomicCommitBoundary,
}

impl<Details> AtomicLedgerAppendIntent<Details> {
    /// append record と effect slot / boundary を結びます。
    pub const fn new(
        record: AuditLedgerRecord<Details>,
        effect_slot: AtomicLedgerEffectSlot,
        commit_boundary: AtomicCommitBoundary,
    ) -> Self {
        Self {
            record,
            effect_slot,
            commit_boundary,
        }
    }

    /// append 対象 record です。
    pub const fn record(&self) -> &AuditLedgerRecord<Details> {
        &self.record
    }

    /// atomic effect slot です。
    pub const fn effect_slot(&self) -> AtomicLedgerEffectSlot {
        self.effect_slot
    }

    /// commit boundary です。
    pub const fn commit_boundary(&self) -> AtomicCommitBoundary {
        self.commit_boundary
    }
}

/// audit ledger record から append intent と effect slot を生成します。
pub fn prepare_atomic_ledger_append<Details>(
    record: AuditLedgerRecord<Details>,
) -> AtomicLedgerAppendIntent<Details> {
    AtomicLedgerAppendIntent::new(
        record,
        AtomicLedgerEffectSlot::AuditLedgerAppend,
        AtomicCommitBoundary::Prepare,
    )
}
