/// atomic write が対象にする effect slot です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AtomicWriteEffectSlot {
    /// durable state restore checkpoint write.
    StateCheckpointWrite,
    /// audit-adjacent metadata write.
    AuditMetadataWrite,
}

/// atomic write の境界です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AtomicWriteBoundary {
    /// write intent の準備境界です。
    Prepare,
    /// durable observation 後の commit 境界です。
    Commit,
}

/// corruption rejection の閉じた理由です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CorruptionRejectionReason {
    /// prepare boundary がありません。
    MissingPrepareBoundary,
    /// commit boundary がありません。
    MissingCommitBoundary,
    /// durable observation がありません。
    MissingDurableObservation,
    /// partial write が観測されました。
    PartialWriteObserved,
    /// payload digest が一致しません。
    PayloadDigestMismatch,
    /// corruption marker が観測されました。
    CorruptionObserved,
}

/// partial failure compensation の状態です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PartialFailureCompensation {
    /// compensation は不要です。
    NotRequired,
    /// compensation が必要で、閉じた理由を保持します。
    Required(CorruptionRejectionReason),
    /// compensation は実行済みですが、元の write decision は拒否扱いにします。
    Applied(CorruptionRejectionReason),
}

/// atomic write decision です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AtomicWriteDecision {
    /// prepare boundary まで進みました。
    Prepared(AtomicWriteEffectSlot),
    /// commit boundary まで進みました。
    Committed(AtomicWriteEffectSlot),
    /// atomic write は閉じた理由で拒否されました。
    Rejected(CorruptionRejectionReason),
}

/// atomic write boundary の観測値です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AtomicWriteBoundaryObservation {
    prepare_boundary_declared: bool,
    commit_boundary_declared: bool,
    durable_observed: bool,
    payload_digest_matches: bool,
    corruption_observed: bool,
}

impl AtomicWriteBoundaryObservation {
    /// write boundary と integrity の観測値を束ねます。
    pub const fn new(
        prepare_boundary_declared: bool,
        commit_boundary_declared: bool,
        durable_observed: bool,
        payload_digest_matches: bool,
        corruption_observed: bool,
    ) -> Self {
        Self {
            prepare_boundary_declared,
            commit_boundary_declared,
            durable_observed,
            payload_digest_matches,
            corruption_observed,
        }
    }
}

/// atomic write 判定入力です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AtomicWriteInput {
    effect_slot: AtomicWriteEffectSlot,
    requested_boundary: AtomicWriteBoundary,
    observation: AtomicWriteBoundaryObservation,
    partial_failure: PartialFailureCompensation,
}

impl AtomicWriteInput {
    /// atomic write の判定材料を束ねます。
    pub const fn new(
        effect_slot: AtomicWriteEffectSlot,
        requested_boundary: AtomicWriteBoundary,
        observation: AtomicWriteBoundaryObservation,
        partial_failure: PartialFailureCompensation,
    ) -> Self {
        Self {
            effect_slot,
            requested_boundary,
            observation,
            partial_failure,
        }
    }
}

/// partial failure と corruption を閉じた rejection reason へ写像します。
///
/// concrete storage adapter は write 実行のみを担当し、compensation semantics は core/operation に閉じます。
pub const fn decide_atomic_write(input: AtomicWriteInput) -> AtomicWriteDecision {
    if !input.observation.prepare_boundary_declared {
        return AtomicWriteDecision::Rejected(CorruptionRejectionReason::MissingPrepareBoundary);
    }

    if !input.observation.payload_digest_matches {
        return AtomicWriteDecision::Rejected(CorruptionRejectionReason::PayloadDigestMismatch);
    }

    if input.observation.corruption_observed {
        return AtomicWriteDecision::Rejected(CorruptionRejectionReason::CorruptionObserved);
    }

    match input.partial_failure {
        PartialFailureCompensation::NotRequired => {}
        PartialFailureCompensation::Required(reason)
        | PartialFailureCompensation::Applied(reason) => {
            return AtomicWriteDecision::Rejected(reason);
        }
    }

    match input.requested_boundary {
        AtomicWriteBoundary::Prepare => AtomicWriteDecision::Prepared(input.effect_slot),
        AtomicWriteBoundary::Commit => {
            if !input.observation.commit_boundary_declared {
                return AtomicWriteDecision::Rejected(
                    CorruptionRejectionReason::MissingCommitBoundary,
                );
            }
            if !input.observation.durable_observed {
                return AtomicWriteDecision::Rejected(
                    CorruptionRejectionReason::MissingDurableObservation,
                );
            }
            AtomicWriteDecision::Committed(input.effect_slot)
        }
    }
}
