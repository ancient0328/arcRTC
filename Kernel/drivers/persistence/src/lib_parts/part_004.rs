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
                let record_line = format!(
                    "intent_class={:?} operation={:?} state_family={:?} state_class={:?}\n",
                    intent.intent_class(),
                    intent.operation(),
                    intent.state_family(),
                    intent.state_class()
                );
                self.write_checkpoint_record(record_line.as_bytes())?;

                Ok(PersistencePortOutput::Acknowledgement(
                    arcrtc_core_ports::PersistenceAcknowledgement::new(
                        intent.intent_class(),
                        None,
                        true,
                    ),
                ))
            }
            arcrtc_core_ports::PersistenceOperationKind::LoadCheckpoint => {
                let bytes = std::fs::read(self.checkpoint_path())
                    .map_err(|_error| persistence_unavailable())?;
                if !checkpoint_record_is_valid(&bytes, intent) {
                    return Err(persistence_unavailable());
                }
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

    fn checkpoint_path(&self) -> std::path::PathBuf {
        self.root.join("checkpoint.record")
    }

    fn checkpoint_tmp_path(&self) -> std::path::PathBuf {
        self.root.join("checkpoint.record.tmp")
    }

    fn write_checkpoint_record(&self, bytes: &[u8]) -> Result<(), PersistencePortFailure> {
        let tmp_path = self.checkpoint_tmp_path();
        let final_path = self.checkpoint_path();
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp_path)
            .map_err(|_error| persistence_unavailable())?;

        // checkpoint は temp file を sync してから rename し、partial record を final として採用しません。
        std::io::Write::write_all(&mut file, bytes).map_err(|_error| persistence_unavailable())?;
        file.sync_all().map_err(|_error| persistence_unavailable())?;
        drop(file);
        std::fs::rename(tmp_path, final_path).map_err(|_error| persistence_unavailable())?;

        Ok(())
    }
}

fn persistence_unavailable() -> PersistencePortFailure {
    PersistencePortFailure::from_kind(
        arcrtc_core_ports::PersistencePortFailureKind::PersistenceUnavailable,
    )
}

fn checkpoint_record_is_valid(
    bytes: &[u8],
    intent: &arcrtc_core_ports::PersistencePortIntent,
) -> bool {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return false;
    };
    let mut lines = text.lines();
    let Some(line) = lines.next() else {
        return false;
    };
    if lines.next().is_some() {
        return false;
    }

    line.contains("intent_class=StateCheckpoint")
        && line.contains("operation=PersistCheckpoint")
        && line.contains(&format!("state_family={:?}", intent.state_family()))
        && line.contains(&format!("state_class={:?}", intent.state_class()))
}
