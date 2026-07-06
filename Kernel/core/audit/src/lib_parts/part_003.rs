/// audit ledger append を許可する唯一の writer authority です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuditLedgerWriterAuthority {
    /// Kernel resident runtime だけが ledger append record を生成できます。
    KernelRuntime,
}

/// audit ledger record construction の fail-closed failure です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuditLedgerFailureKind {
    /// outcome に対する reason slot が明示されていません。
    MissingReason,
    /// hash-chain の previous hash slot が明示されていません。
    MissingPreviousHash,
    /// payload digest が明示されていません。
    MissingPayloadDigest,
    /// Kernel runtime 以外の writer、または writer 不在です。
    InvalidWriterAuthority,
}

/// audit ledger append record へ変換する前の core-owned input です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditLedgerAppendInput<Details> {
    /// ledger に記録する audit event type です。
    pub event_type: AuditEventType,
    /// ledger に記録する use-case outcome です。
    pub outcome: UseCaseOutcome,
    /// outcome に対する reason slot です。success の場合も `AuditReason::None` を明示します。
    pub reason: Option<AuditReason<Details>>,
    /// hash-chain の previous hash slot です。
    pub previous_hash: Option<PreviousRecordHash>,
    /// canonical payload digest です。
    pub payload_digest: Option<CanonicalEventPayloadDigest>,
    /// append record を生成する writer authority です。
    pub writer_authority: Option<AuditLedgerWriterAuthority>,
}

impl<Details> AuditLedgerAppendInput<Details> {
    /// 未検査の append 材料を named input として束ねます。
    pub fn new(
        event_type: AuditEventType,
        outcome: UseCaseOutcome,
        reason: Option<AuditReason<Details>>,
        previous_hash: Option<PreviousRecordHash>,
        payload_digest: Option<CanonicalEventPayloadDigest>,
        writer_authority: Option<AuditLedgerWriterAuthority>,
    ) -> Self {
        Self {
            event_type,
            outcome,
            reason,
            previous_hash,
            payload_digest,
            writer_authority,
        }
    }
}

/// audit ledger に append する正規化済み record です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditLedgerRecord<Details> {
    event_hash: RecordHash,
    previous_hash: PreviousRecordHash,
    event_type: AuditEventType,
    outcome: UseCaseOutcome,
    reason: AuditReason<Details>,
    payload_digest: CanonicalEventPayloadDigest,
}

impl<Details> AuditLedgerRecord<Details> {
    /// append input の欠落を closed failure へ写像し、ledger record を生成します。
    pub fn try_from_append_input(
        input: AuditLedgerAppendInput<Details>,
    ) -> Result<Self, AuditLedgerFailureKind> {
        let AuditLedgerAppendInput {
            event_type,
            outcome,
            reason,
            previous_hash,
            payload_digest,
            writer_authority,
        } = input;

        match writer_authority {
            Some(AuditLedgerWriterAuthority::KernelRuntime) => {}
            None => return Err(AuditLedgerFailureKind::InvalidWriterAuthority),
        }

        let reason = reason.ok_or(AuditLedgerFailureKind::MissingReason)?;
        if outcome.requires_reason()
            && matches!(reason.presence(), AuditReasonPresence::None)
        {
            return Err(AuditLedgerFailureKind::MissingReason);
        }
        let previous_hash = previous_hash.ok_or(AuditLedgerFailureKind::MissingPreviousHash)?;
        let payload_digest = payload_digest.ok_or(AuditLedgerFailureKind::MissingPayloadDigest)?;

        // ledger schema は hash algorithm の選択を持たず、payload digest と同じ
        // canonical algorithm 上の record hash 材料として固定します。
        let event_hash =
            RecordHash::new(payload_digest.algorithm(), payload_digest.digest().to_vec())
                .map_err(|_error| AuditLedgerFailureKind::MissingPayloadDigest)?;

        Ok(Self {
            event_hash,
            previous_hash,
            event_type,
            outcome,
            reason,
            payload_digest,
        })
    }

    /// record hash です。
    pub const fn event_hash(&self) -> &RecordHash {
        &self.event_hash
    }

    /// previous hash slot です。
    pub const fn previous_hash(&self) -> &PreviousRecordHash {
        &self.previous_hash
    }

    /// audit event type です。
    pub const fn event_type(&self) -> AuditEventType {
        self.event_type
    }

    /// use-case outcome です。
    pub const fn outcome(&self) -> UseCaseOutcome {
        self.outcome
    }

    /// outcome に対する reason slot です。
    pub const fn reason(&self) -> &AuditReason<Details> {
        &self.reason
    }

    /// canonical payload digest です。
    pub const fn payload_digest(&self) -> &CanonicalEventPayloadDigest {
        &self.payload_digest
    }
}
