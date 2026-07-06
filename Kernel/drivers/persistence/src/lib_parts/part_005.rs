use arcrtc_core_ports::{
    LedgerAppendPortFailureKind, LedgerAppendPortInput, LedgerAppendPortOutput,
};

/// audit ledger append 専用の filesystem executor です。
pub struct FileAuditLedgerAppendExecutor {
    root: std::path::PathBuf,
}

impl FileAuditLedgerAppendExecutor {
    /// executor の root path を保持します。この時点では I/O を実行しません。
    pub fn new(root: std::path::PathBuf) -> Self {
        Self { root }
    }

    /// core-owned ledger append input を concrete file append + sync に写像します。
    pub fn append<Details>(
        &self,
        input: &LedgerAppendPortInput<Details>,
    ) -> Result<LedgerAppendPortOutput, LedgerAppendPortFailureKind> {
        std::fs::create_dir_all(&self.root)
            .map_err(|_error| LedgerAppendPortFailureKind::PathRejected)?;

        let audit_line = serialize_audit_ledger_event_line(input)?;
        let hash_chain_line = serialize_audit_ledger_hash_chain_line(input)?;

        self.append_and_sync("audit_events.log", audit_line.as_bytes())?;
        self.append_and_sync("hash_chain_records.log", hash_chain_line.as_bytes())?;

        Ok(LedgerAppendPortOutput::new(
            input.append_intent().effect_slot(),
            input.append_intent().commit_boundary(),
            true,
        ))
    }

    fn append_and_sync(
        &self,
        file_name: &str,
        bytes: &[u8],
    ) -> Result<(), LedgerAppendPortFailureKind> {
        let path = self.root.join(file_name);
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|_error| LedgerAppendPortFailureKind::AppendFailed)?;

        std::io::Write::write_all(&mut file, bytes)
            .map_err(|_error| LedgerAppendPortFailureKind::AppendFailed)?;
        file.sync_all()
            .map_err(|_error| LedgerAppendPortFailureKind::SyncFailed)?;

        Ok(())
    }
}

fn serialize_audit_ledger_event_line<Details>(
    input: &LedgerAppendPortInput<Details>,
) -> Result<String, LedgerAppendPortFailureKind> {
    let record = input.append_intent().record();
    let mut line = String::new();
    line.try_reserve(256)
        .map_err(|_error| LedgerAppendPortFailureKind::SerializationFailed)?;

    line.push_str("event_type=");
    line.push_str(record.event_type().as_str());
    line.push_str(" outcome=");
    line.push_str(&format!("{:?}", record.outcome()));
    line.push_str(" reason_presence=");
    line.push_str(&format!("{:?}", record.reason().presence()));
    line.push_str(" payload_digest_algorithm=");
    line.push_str(record.payload_digest().algorithm().as_str());
    line.push_str(" payload_digest_len=");
    line.push_str(&record.payload_digest().digest().len().to_string());
    line.push('\n');

    Ok(line)
}

fn serialize_audit_ledger_hash_chain_line<Details>(
    input: &LedgerAppendPortInput<Details>,
) -> Result<String, LedgerAppendPortFailureKind> {
    let record = input.append_intent().record();
    let mut line = String::new();
    line.try_reserve(256)
        .map_err(|_error| LedgerAppendPortFailureKind::SerializationFailed)?;

    // driver は hash-chain の意味を決めず、core/audit record の field を保存形式へ写像します。
    line.push_str("previous_hash=");
    line.push_str(&format!("{:?}", record.previous_hash()));
    line.push_str(" event_hash_algorithm=");
    line.push_str(record.event_hash().algorithm().as_str());
    line.push_str(" event_hash_len=");
    line.push_str(&record.event_hash().digest().len().to_string());
    line.push('\n');

    Ok(line)
}
