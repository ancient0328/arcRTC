use arcrtc_core_audit::{
    AuditLedgerAppendInput, AuditLedgerFailureKind, AuditLedgerRecord, AuditLedgerWriterAuthority,
    AuditReason, CanonicalEventPayloadDigest, HashAlgorithm, HashChainRecordError,
    PreviousRecordHash,
};
use arcrtc_core_command::UseCaseOutcome;

fn audit_event_type() -> arcrtc_core_audit::AuditEventType {
    arcrtc_core_audit::find_audit_event_definition("atomicity_compensation_decision")
        .expect("test fixture must use a cataloged audit event type")
        .event_type()
}

fn payload_digest() -> CanonicalEventPayloadDigest {
    CanonicalEventPayloadDigest::new(HashAlgorithm::Sha256, vec![1, 2, 3, 4])
        .expect("test fixture digest is non-empty")
}

#[test]
fn ledger_constructor_is_fail_closed() {
    assert_missing_reason();
    assert_missing_previous_hash_when_hash_chain_required();
    assert_empty_digest();
    assert_wrong_writer_authority();
}

fn assert_missing_reason() {
    let input = AuditLedgerAppendInput::<()>::new(
        audit_event_type(),
        UseCaseOutcome::Rejected,
        None,
        Some(PreviousRecordHash::Genesis),
        Some(payload_digest()),
        Some(AuditLedgerWriterAuthority::KernelRuntime),
    );

    assert_eq!(
        AuditLedgerRecord::try_from_append_input(input),
        Err(AuditLedgerFailureKind::MissingReason)
    );
}

fn assert_missing_previous_hash_when_hash_chain_required() {
    let input = AuditLedgerAppendInput::<()>::new(
        audit_event_type(),
        UseCaseOutcome::Accepted,
        Some(AuditReason::None),
        None,
        Some(payload_digest()),
        Some(AuditLedgerWriterAuthority::KernelRuntime),
    );

    assert_eq!(
        AuditLedgerRecord::try_from_append_input(input),
        Err(AuditLedgerFailureKind::MissingPreviousHash)
    );
}

fn assert_empty_digest() {
    assert_eq!(
        CanonicalEventPayloadDigest::new(HashAlgorithm::Sha256, Vec::new()),
        Err(HashChainRecordError::EmptyDigest)
    );

    let input = AuditLedgerAppendInput::<()>::new(
        audit_event_type(),
        UseCaseOutcome::Accepted,
        Some(AuditReason::None),
        Some(PreviousRecordHash::Genesis),
        None,
        Some(AuditLedgerWriterAuthority::KernelRuntime),
    );

    assert_eq!(
        AuditLedgerRecord::try_from_append_input(input),
        Err(AuditLedgerFailureKind::MissingPayloadDigest)
    );
}

fn assert_wrong_writer_authority() {
    let input = AuditLedgerAppendInput::<()>::new(
        audit_event_type(),
        UseCaseOutcome::Accepted,
        Some(AuditReason::None),
        Some(PreviousRecordHash::Genesis),
        Some(payload_digest()),
        // writer authority は閉集合で KernelRuntime しか無いため、不在を invalid writer として閉じます。
        None,
    );

    assert_eq!(
        AuditLedgerRecord::try_from_append_input(input),
        Err(AuditLedgerFailureKind::InvalidWriterAuthority)
    );
}
