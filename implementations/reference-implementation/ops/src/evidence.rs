//! reference command evidence writer境界です。

use std::path::PathBuf;

use arcrtc_implementation_evidence::{
    validate_evidence_record, ImplementationCommandClass, ImplementationEvidenceRecord,
    ImplementationLayer, ImplementationPlane, IMPLEMENTATIONS_EVIDENCE_ROOT,
};

use crate::error::ReferenceRuntimeError;

/// evidence write outcomeです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImplementationEvidenceWriteOutcome {
    /// 書き込み先pathです。
    pub output_path: PathBuf,
}

/// implementation evidence recordを書き込みます。
pub fn write_implementation_evidence_record(
    record: &ImplementationEvidenceRecord,
) -> Result<ImplementationEvidenceWriteOutcome, ReferenceRuntimeError> {
    validate_evidence_record(record)
        .map_err(|_| ReferenceRuntimeError::EvidenceFieldsIncomplete)?;
    validate_reference_writer_ownership(record)?;
    validate_safe_file_component(&record.correlation_id)?;
    let output_dir = reference_output_directory(record.command_class)?;
    std::fs::create_dir_all(&output_dir).map_err(|_| ReferenceRuntimeError::EvidenceWriteError)?;
    let output_path = output_dir.join(format!("{}.json", record.correlation_id));
    let json =
        serde_json::to_vec_pretty(record).map_err(|_| ReferenceRuntimeError::EvidenceWriteError)?;
    std::fs::write(&output_path, json).map_err(|_| ReferenceRuntimeError::EvidenceWriteError)?;
    Ok(ImplementationEvidenceWriteOutcome { output_path })
}

fn reference_output_directory(
    command_class: ImplementationCommandClass,
) -> Result<PathBuf, ReferenceRuntimeError> {
    let class_directory = match command_class {
        ImplementationCommandClass::Build => "build",
        ImplementationCommandClass::Format
        | ImplementationCommandClass::Test
        | ImplementationCommandClass::Benchmark
        | ImplementationCommandClass::RealDevice
        | ImplementationCommandClass::ProductionReadiness
        | ImplementationCommandClass::LiveReadiness => {
            return Err(ReferenceRuntimeError::CommandScopeMismatch);
        }
    };
    Ok(PathBuf::from(IMPLEMENTATIONS_EVIDENCE_ROOT).join(class_directory))
}

fn validate_reference_writer_ownership(
    record: &ImplementationEvidenceRecord,
) -> Result<(), ReferenceRuntimeError> {
    if record.implementation_layer != ImplementationLayer::Reference
        || record.target_plane != ImplementationPlane::Ops
    {
        return Err(ReferenceRuntimeError::CommandScopeMismatch);
    }

    match record.command_class {
        ImplementationCommandClass::Build => {
            if record.target_scope != "reference-implementation/ops"
                || record.target_package.as_deref() != Some("arcrtc-reference-ops")
                || record.command.trim() != "cargo build --workspace --all-targets"
            {
                return Err(ReferenceRuntimeError::CommandScopeMismatch);
            }
        }
        ImplementationCommandClass::Format
        | ImplementationCommandClass::Test
        | ImplementationCommandClass::Benchmark
        | ImplementationCommandClass::RealDevice
        | ImplementationCommandClass::ProductionReadiness
        | ImplementationCommandClass::LiveReadiness => {
            return Err(ReferenceRuntimeError::CommandScopeMismatch);
        }
    }

    Ok(())
}

fn validate_safe_file_component(value: &str) -> Result<(), ReferenceRuntimeError> {
    if value == "." || value == ".." {
        return Err(ReferenceRuntimeError::CommandScopeMismatch);
    }
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
    {
        return Ok(());
    }
    Err(ReferenceRuntimeError::CommandScopeMismatch)
}
