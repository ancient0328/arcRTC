use arcrtc_core_identity::{OpaqueReference, ReferenceAuthority, UntrustedReference};
use arcrtc_core_ports::{
    LoadedCoreStateRef, PersistenceAcknowledgement, PersistenceConsistencyRequirement,
    PersistenceIntentClass, PersistenceOperationKind, PersistencePortFailure,
    PersistencePortFailureKind, PersistencePortInput, PersistencePortIntent, PersistencePortOutput,
    PersistenceRecordRef,
};
use arcrtc_core_state::{StateClass, StateFamily};
use arcrtc_driver_persistence::FileSystemPersistenceExecutor;

struct DirGuard(std::path::PathBuf);

impl Drop for DirGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn temp_dir() -> DirGuard {
    let path = std::env::temp_dir().join(format!("arcrtc_fs_persistence_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
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

#[test]
fn filesystem_persistence_roundtrip_and_fail_closed_retry_store() {
    let dir = temp_dir();
    let executor = FileSystemPersistenceExecutor::new(dir.0.clone());

    let checkpoint = checkpoint_intent(PersistenceOperationKind::PersistCheckpoint);
    let checkpoint_output = executor.execute(&PersistencePortInput::ExecuteIntent(checkpoint));
    println!("checkpoint_output={checkpoint_output:?}");
    assert_eq!(
        checkpoint_output,
        Ok(PersistencePortOutput::Acknowledgement(
            PersistenceAcknowledgement::new(PersistenceIntentClass::StateCheckpoint, None, true)
        ))
    );

    let checkpoint_path = dir.0.join("checkpoint.record");
    let checkpoint_bytes = std::fs::read(&checkpoint_path).expect("checkpoint file must exist");
    println!("checkpoint_path={checkpoint_path:?} checkpoint_bytes={checkpoint_bytes:?}");
    assert!(!checkpoint_bytes.is_empty());

    let loaded_state_ref = LoadedCoreStateRef::new(
        StateFamily::SignalingIdempotency,
        StateClass::CheckpointEligibleState,
        PersistenceRecordRef::new(
            OpaqueReference::accept_untrusted(
                UntrustedReference::new("fs_checkpoint_record"),
                ReferenceAuthority::CoreValidatedUntrustedInput,
            )
            .expect("fixed record ref must be accepted"),
        ),
    );
    let load = checkpoint_intent(PersistenceOperationKind::LoadCheckpoint);
    let load_output = executor.execute(&PersistencePortInput::ExecuteIntent(load));
    println!("load_output={load_output:?}");
    assert_eq!(
        load_output,
        Ok(PersistencePortOutput::LoadedState(loaded_state_ref))
    );

    let audit_intent = PersistencePortIntent::try_new(
        PersistenceIntentClass::AuditPersistence,
        PersistenceOperationKind::AppendAuditEvent,
        StateFamily::AuditEvent,
        StateClass::AuditOnlyState,
        vec![],
        None,
    )
    .expect("audit intent fixture must be valid");
    for attempt in 1..=2 {
        let audit_output =
            executor.execute(&PersistencePortInput::ExecuteIntent(audit_intent.clone()));
        println!("audit_attempt={attempt} output={audit_output:?}");
        assert_eq!(
            audit_output,
            Ok(PersistencePortOutput::Acknowledgement(
                PersistenceAcknowledgement::new(
                    PersistenceIntentClass::AuditPersistence,
                    None,
                    true
                )
            ))
        );
    }
    let audit_path = dir.0.join("audit_events.log");
    let audit_text = std::fs::read_to_string(&audit_path).expect("audit log must exist");
    println!("audit_path={audit_path:?} audit_text={audit_text:?}");
    assert_eq!(audit_text.lines().count(), 2);

    let retry_intent = PersistencePortIntent::try_new(
        PersistenceIntentClass::RetryStore,
        PersistenceOperationKind::EnqueueRetry,
        StateFamily::DriverRetryStore,
        StateClass::DriverLocalState,
        vec![PersistenceConsistencyRequirement::BoundedRetryStore],
        None,
    )
    .expect("retry intent fixture must be valid");
    let retry_output = executor.execute(&PersistencePortInput::ExecuteIntent(retry_intent));
    println!("retry_output={retry_output:?}");
    assert_eq!(
        retry_output,
        Err(PersistencePortFailure::from_kind(
            PersistencePortFailureKind::PersistenceUnavailable
        ))
    );
}
