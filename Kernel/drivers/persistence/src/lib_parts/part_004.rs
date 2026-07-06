use arcrtc_core_identity::{OpaqueReference, ReferenceAuthority, UntrustedReference};

/// std::fs を使う driver-owned persistence executor です。
pub struct FileSystemPersistenceExecutor {
    root: std::path::PathBuf,
}

impl FileSystemPersistenceExecutor {
    /// executor の root path を保持します。この時点では I/O を実行しません。
    pub fn new(root: std::path::PathBuf) -> Self {
        Self { root }
    }

    /// core-owned persistence intent を driver-owned filesystem record へ写像して実行します。
    pub fn execute(
        &self,
        input: &PersistencePortInput,
    ) -> Result<PersistencePortOutput, PersistencePortFailure> {
        std::fs::create_dir_all(&self.root).map_err(|_error| {
            PersistencePortFailure::from_kind(
                arcrtc_core_ports::PersistencePortFailureKind::PersistenceUnavailable,
            )
        })?;

        let PersistencePortInput::ExecuteIntent(intent) = input;

        match intent.operation() {
            arcrtc_core_ports::PersistenceOperationKind::PersistCheckpoint => {
                let path = self.root.join("checkpoint.record");
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(path)
                    .map_err(|_error| {
                        PersistencePortFailure::from_kind(
                            arcrtc_core_ports::PersistencePortFailureKind::PersistenceUnavailable,
                        )
                    })?;
                let record_line = format!(
                    "intent_class={:?} operation={:?} state_family={:?} state_class={:?}\n",
                    intent.intent_class(),
                    intent.operation(),
                    intent.state_family(),
                    intent.state_class()
                );
                std::io::Write::write_all(&mut file, record_line.as_bytes()).map_err(|_error| {
                    PersistencePortFailure::from_kind(
                        arcrtc_core_ports::PersistencePortFailureKind::PersistenceUnavailable,
                    )
                })?;
                file.sync_all().map_err(|_error| {
                    PersistencePortFailure::from_kind(
                        arcrtc_core_ports::PersistencePortFailureKind::PersistenceUnavailable,
                    )
                })?;

                Ok(PersistencePortOutput::Acknowledgement(
                    arcrtc_core_ports::PersistenceAcknowledgement::new(
                        intent.intent_class(),
                        None,
                        true,
                    ),
                ))
            }
            arcrtc_core_ports::PersistenceOperationKind::LoadCheckpoint => {
                let _bytes = std::fs::read(self.root.join("checkpoint.record")).map_err(|_error| {
                    PersistencePortFailure::from_kind(
                        arcrtc_core_ports::PersistencePortFailureKind::PersistenceUnavailable,
                    )
                })?;
                let record_ref = arcrtc_core_ports::PersistenceRecordRef::new(
                    OpaqueReference::accept_untrusted(
                        UntrustedReference::new("fs_checkpoint_record"),
                        ReferenceAuthority::CoreValidatedUntrustedInput,
                    )
                    .expect("fixed record reference token is non-empty and control-free"),
                );

                Ok(PersistencePortOutput::LoadedState(
                    arcrtc_core_ports::LoadedCoreStateRef::new(
                        intent.state_family(),
                        intent.state_class(),
                        record_ref,
                    ),
                ))
            }
            arcrtc_core_ports::PersistenceOperationKind::AppendAuditEvent => {
                let path = self.root.join("audit_events.log");
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .map_err(|_error| {
                        PersistencePortFailure::from_kind(
                            arcrtc_core_ports::PersistencePortFailureKind::PersistenceUnavailable,
                        )
                    })?;
                let record_line = format!(
                    "intent_class={:?} operation={:?} state_family={:?} state_class={:?}\n",
                    intent.intent_class(),
                    intent.operation(),
                    intent.state_family(),
                    intent.state_class()
                );
                std::io::Write::write_all(&mut file, record_line.as_bytes()).map_err(|_error| {
                    PersistencePortFailure::from_kind(
                        arcrtc_core_ports::PersistencePortFailureKind::PersistenceUnavailable,
                    )
                })?;
                file.sync_all().map_err(|_error| {
                    PersistencePortFailure::from_kind(
                        arcrtc_core_ports::PersistencePortFailureKind::PersistenceUnavailable,
                    )
                })?;

                Ok(PersistencePortOutput::Acknowledgement(
                    arcrtc_core_ports::PersistenceAcknowledgement::new(
                        intent.intent_class(),
                        None,
                        true,
                    ),
                ))
            }
            arcrtc_core_ports::PersistenceOperationKind::AppendHashChainRecord => {
                let path = self.root.join("hash_chain_records.log");
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .map_err(|_error| {
                        PersistencePortFailure::from_kind(
                            arcrtc_core_ports::PersistencePortFailureKind::PersistenceUnavailable,
                        )
                    })?;
                let record_line = format!(
                    "intent_class={:?} operation={:?} state_family={:?} state_class={:?}\n",
                    intent.intent_class(),
                    intent.operation(),
                    intent.state_family(),
                    intent.state_class()
                );
                std::io::Write::write_all(&mut file, record_line.as_bytes()).map_err(|_error| {
                    PersistencePortFailure::from_kind(
                        arcrtc_core_ports::PersistencePortFailureKind::PersistenceUnavailable,
                    )
                })?;
                file.sync_all().map_err(|_error| {
                    PersistencePortFailure::from_kind(
                        arcrtc_core_ports::PersistencePortFailureKind::PersistenceUnavailable,
                    )
                })?;

                Ok(PersistencePortOutput::Acknowledgement(
                    arcrtc_core_ports::PersistenceAcknowledgement::new(
                        intent.intent_class(),
                        None,
                        true,
                    ),
                ))
            }
            arcrtc_core_ports::PersistenceOperationKind::EnqueueRetry
            | arcrtc_core_ports::PersistenceOperationKind::DequeueRetry
            | arcrtc_core_ports::PersistenceOperationKind::AcknowledgeRetry => {
                Err(PersistencePortFailure::from_kind(
                    arcrtc_core_ports::PersistencePortFailureKind::PersistenceUnavailable,
                ))
            }
        }
    }
}
