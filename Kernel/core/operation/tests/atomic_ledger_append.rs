#![allow(non_snake_case)]

use arcrtc_core_audit::{
    AuditLedgerAppendInput, AuditLedgerFailureKind, AuditLedgerRecord, AuditLedgerWriterAuthority,
    AuditReason, CanonicalEventPayloadDigest, HashAlgorithm, PreviousRecordHash,
};
use arcrtc_core_command::UseCaseOutcome;
use arcrtc_core_operation::{
    decide_atomic_write, prepare_atomic_ledger_append, AtomicCommitBoundary,
    AtomicLedgerEffectSlot, AtomicWriteBoundary, AtomicWriteBoundaryObservation,
    AtomicWriteDecision, AtomicWriteEffectSlot, AtomicWriteInput, CorruptionRejectionReason,
    PartialFailureCompensation,
};

// Test Roadmap の named assertion rule は task id と assertion target を
// `assert_<task>__<target>` で結合するため、この test source だけ lint を限定します。

fn audit_event_type() -> arcrtc_core_audit::AuditEventType {
    arcrtc_core_audit::find_audit_event_definition("atomicity_compensation_decision")
        .expect("test fixture must use a cataloged audit event type")
        .event_type()
}

fn payload_digest() -> CanonicalEventPayloadDigest {
    CanonicalEventPayloadDigest::new(HashAlgorithm::Sha256, vec![8, 9, 10, 11])
        .expect("test fixture digest is non-empty")
}

fn accepted_record() -> AuditLedgerRecord<()> {
    let input = AuditLedgerAppendInput::new(
        audit_event_type(),
        UseCaseOutcome::Accepted,
        Some(AuditReason::None),
        Some(PreviousRecordHash::Genesis),
        Some(payload_digest()),
        Some(AuditLedgerWriterAuthority::KernelRuntime),
    );
    AuditLedgerRecord::try_from_append_input(input).expect("accepted fixture must build record")
}

#[test]
fn t_audit_02_atomic_append_and_corruption_rejection() {
    assert_t_audit_02__accepted_append();
    assert_t_audit_02__rejected_append();
    assert_t_audit_02__compensation_required();
    assert_t_audit_02__corruption_rejection();
}

fn assert_t_audit_02__accepted_append() {
    let intent = prepare_atomic_ledger_append(accepted_record());

    assert_eq!(
        intent.effect_slot(),
        AtomicLedgerEffectSlot::AuditLedgerAppend
    );
    assert_eq!(intent.commit_boundary(), AtomicCommitBoundary::Prepare);
    assert_eq!(intent.record().outcome(), UseCaseOutcome::Accepted);
}

fn assert_t_audit_02__rejected_append() {
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

fn assert_t_audit_02__compensation_required() {
    let input = AtomicWriteInput::new(
        AtomicWriteEffectSlot::AuditMetadataWrite,
        AtomicWriteBoundary::Commit,
        AtomicWriteBoundaryObservation::new(true, true, true, true, false),
        PartialFailureCompensation::Required(CorruptionRejectionReason::PartialWriteObserved),
    );

    assert_eq!(
        decide_atomic_write(input),
        AtomicWriteDecision::Rejected(CorruptionRejectionReason::PartialWriteObserved)
    );
}

fn assert_t_audit_02__corruption_rejection() {
    let input = AtomicWriteInput::new(
        AtomicWriteEffectSlot::AuditMetadataWrite,
        AtomicWriteBoundary::Commit,
        AtomicWriteBoundaryObservation::new(true, true, true, true, true),
        PartialFailureCompensation::NotRequired,
    );

    assert_eq!(
        decide_atomic_write(input),
        AtomicWriteDecision::Rejected(CorruptionRejectionReason::CorruptionObserved)
    );
}
