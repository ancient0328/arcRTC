// Roadmap の assertion 名をそのまま残すため、このテストファイルだけ許可します。
#![allow(non_snake_case)]

use arcrtc_core_identity::{OpaqueReference, ReferenceAuthority, UntrustedReference};
use arcrtc_core_ports::{
    LoadedCoreStateRef, PersistenceAcknowledgement, PersistenceIntentClass,
    PersistenceOperationKind, PersistencePortFailure, PersistencePortFailureKind,
    PersistencePortInput, PersistencePortIntent, PersistencePortOutput, PersistenceRecordRef,
};
use arcrtc_core_state::{
    decide_state_restore, DurableStateFamily, RestoreCheckpointRef, StateClass,
    StateCorruptionDecision, StateFamily, StateRestoreDecision, StateRestoreFailureKind,
    StateRestoreInput,
};
use arcrtc_driver_persistence::FileSystemPersistenceExecutor;

struct DirGuard(std::path::PathBuf);

impl Drop for DirGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn temp_dir(label: &str) -> DirGuard {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time must be after epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "arcrtc_durable_state_restore_{}_{}_{}",
        std::process::id(),
        label,
        nonce
    ));
    std::fs::create_dir_all(&path).expect("temp dir must be created");
    DirGuard(path)
}

fn checkpoint_intent(operation: PersistenceOperationKind) -> PersistencePortIntent {
    PersistencePortIntent::try_new(
        PersistenceIntentClass::StateCheckpoint,
        operation,
        StateFamily::SignalingIdempotency,
        StateClass::CheckpointEligibleState,
        vec![],
        None,
    )
    .expect("checkpoint intent fixture must be valid")
}

fn loaded_state_ref() -> LoadedCoreStateRef {
    LoadedCoreStateRef::new(
        StateFamily::SignalingIdempotency,
        StateClass::CheckpointEligibleState,
        PersistenceRecordRef::new(
            OpaqueReference::accept_untrusted(
                UntrustedReference::new("fs_checkpoint_record"),
                ReferenceAuthority::CoreValidatedUntrustedInput,
            )
            .expect("fixed record ref must be accepted"),
        ),
    )
}

fn persist(
    executor: &FileSystemPersistenceExecutor,
) -> Result<PersistencePortOutput, PersistencePortFailure> {
    executor.execute(&PersistencePortInput::ExecuteIntent(checkpoint_intent(
        PersistenceOperationKind::PersistCheckpoint,
    )))
}

fn load(
    executor: &FileSystemPersistenceExecutor,
) -> Result<PersistencePortOutput, PersistencePortFailure> {
    executor.execute(&PersistencePortInput::ExecuteIntent(checkpoint_intent(
        PersistenceOperationKind::LoadCheckpoint,
    )))
}

fn checkpoint_path(root: &std::path::Path) -> std::path::PathBuf {
    root.join("checkpoint.record")
}

fn checkpoint_tmp_path(root: &std::path::Path) -> std::path::PathBuf {
    root.join("checkpoint.record.tmp")
}

#[test]
fn assert_t_persist_01__restore() {
    let dir = temp_dir("restore");
    let executor = FileSystemPersistenceExecutor::new(dir.0.clone());

    assert_eq!(
        persist(&executor),
        Ok(PersistencePortOutput::Acknowledgement(
            PersistenceAcknowledgement::new(PersistenceIntentClass::StateCheckpoint, None, true)
        ))
    );
    assert_eq!(
        load(&executor),
        Ok(PersistencePortOutput::LoadedState(loaded_state_ref()))
    );

    let restore_decision = decide_state_restore(StateRestoreInput::new(
        DurableStateFamily::SignalingIdempotency,
        Some(RestoreCheckpointRef::new("fs_checkpoint_record")),
        Some("restore-policy:signaling-idempotency"),
        StateCorruptionDecision::Accepted,
    ));
    assert_eq!(
        restore_decision,
        StateRestoreDecision::Restored(RestoreCheckpointRef::new("fs_checkpoint_record"))
    );
}

#[test]
fn assert_t_persist_01__atomic_write() {
    let dir = temp_dir("atomic");
    let executor = FileSystemPersistenceExecutor::new(dir.0.clone());

    std::fs::write(checkpoint_tmp_path(&dir.0), b"partial").expect("partial tmp must be writable");
    assert!(persist(&executor).is_ok());

    let checkpoint_text =
        std::fs::read_to_string(checkpoint_path(&dir.0)).expect("checkpoint must exist");
    assert!(!checkpoint_tmp_path(&dir.0).exists());
    assert_eq!(checkpoint_text.lines().count(), 1);
    assert!(checkpoint_text.contains("intent_class=StateCheckpoint"));
    assert!(checkpoint_text.contains("operation=PersistCheckpoint"));
}

#[test]
fn assert_t_persist_01__partial_failure() {
    let dir = temp_dir("partial");
    let executor = FileSystemPersistenceExecutor::new(dir.0.clone());

    assert!(persist(&executor).is_ok());
    let stable_checkpoint =
        std::fs::read(checkpoint_path(&dir.0)).expect("stable checkpoint must exist");
    // crash 中の temp record は final checkpoint として扱わず、load は stable checkpoint だけを見る。
    std::fs::write(checkpoint_tmp_path(&dir.0), b"partial").expect("partial tmp must be writable");

    assert_eq!(
        load(&executor),
        Ok(PersistencePortOutput::LoadedState(loaded_state_ref()))
    );
    assert_eq!(
        std::fs::read(checkpoint_path(&dir.0)).expect("stable checkpoint must remain"),
        stable_checkpoint
    );
}

#[test]
fn assert_t_persist_01__corruption_rejection() {
    let dir = temp_dir("corruption");
    let executor = FileSystemPersistenceExecutor::new(dir.0.clone());

    assert!(persist(&executor).is_ok());
    std::fs::write(checkpoint_path(&dir.0), b"corrupt")
        .expect("corruption fixture must be writable");
    assert_eq!(
        load(&executor),
        Err(PersistencePortFailure::from_kind(
            PersistencePortFailureKind::PersistenceUnavailable
        ))
    );

    let restore_decision = decide_state_restore(StateRestoreInput::new(
        DurableStateFamily::SignalingIdempotency,
        Some(RestoreCheckpointRef::new("fs_checkpoint_record")),
        Some("restore-policy:signaling-idempotency"),
        StateCorruptionDecision::Rejected,
    ));
    assert_eq!(
        restore_decision,
        StateRestoreDecision::Rejected(StateRestoreFailureKind::CorruptionRejected)
    );
}
