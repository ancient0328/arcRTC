use arcrtc_core_audit::{
    find_audit_event_definition, CanonicalEventPayloadDigest, CanonicalRecordFormat, HashAlgorithm,
    HashChainRecord, HashChainRecordInput, HashChainScope, HashChainSequence, PreviousRecordHash,
    RecordHash,
};
use arcrtc_core_command::UseCaseOutcome;
use arcrtc_core_ports::{
    PersistenceConsistencyRequirement, PersistenceIntentClass, PersistenceOperationKind,
    PersistencePortIntent,
};
use arcrtc_core_protocol::{
    CanonicalDataClass, CanonicalDigest, CanonicalEncodingFailureKind, CanonicalEncodingRuleSet,
    CanonicalFormatVersion, CanonicalRuleStatus, UnknownFieldHandling,
};
use arcrtc_core_signaling::{SignalingCommandKind, SignalingEventKind};
use arcrtc_core_state::{StateClass, StateFamily};

use super::support::{correlation_id, deterministic_digest_bytes, mix_bytes};

pub(super) fn audit_hash_chain_workload() -> u64 {
    let event_type = find_audit_event_definition("driver_error_converted")
        .expect("audit event type exists")
        .event_type();
    let format = CanonicalRecordFormat::new("arcrtc.audit.hash-chain", "v0.2");
    let mut previous = PreviousRecordHash::Genesis;
    let mut state = 0u64;
    for index in 0..256u64 {
        let payload_digest = CanonicalEventPayloadDigest::new(
            HashAlgorithm::Sha256,
            deterministic_digest_bytes(state, index),
        )
        .expect("payload digest is non-empty");
        let record_hash = RecordHash::new(
            HashAlgorithm::Sha256,
            deterministic_digest_bytes(index, state),
        )
        .expect("record hash is non-empty");
        let record = HashChainRecord::new(HashChainRecordInput {
            scope: HashChainScope::Driver,
            sequence: HashChainSequence::new(index),
            previous_hash: previous,
            event_type,
            outcome: UseCaseOutcome::ConvertedFailure,
            canonical_format: format,
            payload_digest,
            record_hash: record_hash.clone(),
        });
        state = mix_bytes(
            state,
            format!("{:?}:{}", record.scope(), record.sequence().value()).as_bytes(),
        );
        previous = PreviousRecordHash::Previous(record_hash);
    }
    state
}

pub(super) fn canonical_serialization_workload() -> u64 {
    let rules = CanonicalEncodingRuleSet::new(
        CanonicalRuleStatus::Defined,
        CanonicalRuleStatus::Defined,
        CanonicalRuleStatus::Defined,
        CanonicalRuleStatus::Defined,
        CanonicalRuleStatus::Defined,
        CanonicalRuleStatus::Defined,
        CanonicalRuleStatus::Defined,
        CanonicalRuleStatus::Defined,
        CanonicalRuleStatus::Defined,
        UnknownFieldHandling::Reject,
        CanonicalRuleStatus::Defined,
        CanonicalRuleStatus::Defined,
    );
    let mut state = 0u64;
    for index in 0..512u64 {
        let digest = CanonicalDigest::new(
            CanonicalFormatVersion::new("arcrtc.canonical", "v0.2"),
            "deterministic-test-digest",
            deterministic_digest_bytes(index, state),
        )
        .expect("canonical digest is non-empty");
        state = mix_bytes(
            state,
            format!(
                "{:?}:{:?}:{:?}:{}",
                CanonicalDataClass::AuditEventHashInput,
                CanonicalEncodingFailureKind::CanonicalSerializationMismatch,
                digest.format_version(),
                rules.usable_for_canonical_evidence()
            )
            .as_bytes(),
        );
    }
    state
}

pub(super) fn persistence_export_workload() -> u64 {
    let mut state = 0u64;
    for index in 0..512 {
        let intent = PersistencePortIntent::try_new(
            PersistenceIntentClass::HashChainRecordPersistence,
            PersistenceOperationKind::AppendHashChainRecord,
            StateFamily::AuditHashChainRecord,
            StateClass::AuditOnlyState,
            vec![
                PersistenceConsistencyRequirement::OrderedAppend,
                PersistenceConsistencyRequirement::RetentionPolicy,
            ],
            Some(correlation_id(format!("persist-corr-{index}"))),
        )
        .expect("valid persistence port intent");
        state = mix_bytes(
            state,
            format!("{:?}:{:?}", intent.intent_class(), intent.state_class()).as_bytes(),
        );
    }
    state
}

pub(super) fn sdk_signaling_roundtrip_workload() -> u64 {
    let commands = [
        SignalingCommandKind::JoinRoom,
        SignalingCommandKind::SendOffer,
        SignalingCommandKind::SendAnswer,
        SignalingCommandKind::SendIceCandidate,
        SignalingCommandKind::RequestTurnCredential,
        SignalingCommandKind::AcknowledgeForward,
    ];
    let events = [
        SignalingEventKind::Joined,
        SignalingEventKind::OfferReceived,
        SignalingEventKind::AnswerReceived,
        SignalingEventKind::IceCandidateReceived,
        SignalingEventKind::TurnCredentialAvailable,
    ];
    let mut state = 0u64;
    for index in 0..2_048 {
        let payload = format!(
            "sdk=v0.2;cmd={:?};event={:?};corr=sdk-corr-{};opaque=true",
            commands[index % commands.len()],
            events[index % events.len()],
            index
        );
        state = mix_bytes(state, payload.as_bytes());
    }
    state
}
