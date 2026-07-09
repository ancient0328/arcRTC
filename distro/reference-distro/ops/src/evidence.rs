//! reference command evidence writer境界です。

use std::path::PathBuf;

use arcrtc_distro_evidence::{
    validate_evidence_record, DistroCommandClass, DistroEvidenceRecord, DistroLayer, DistroPlane,
    DISTRO_EVIDENCE_ROOT,
};

use crate::error::ReferenceRuntimeError;

/// evidence write outcomeです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DistroEvidenceWriteOutcome {
    /// 書き込み先pathです。
    pub output_path: PathBuf,
}

/// distro evidence recordを書き込みます。
pub fn write_distro_evidence_record(
    record: &DistroEvidenceRecord,
) -> Result<DistroEvidenceWriteOutcome, ReferenceRuntimeError> {
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
    Ok(DistroEvidenceWriteOutcome { output_path })
}

fn reference_output_directory(
    command_class: DistroCommandClass,
) -> Result<PathBuf, ReferenceRuntimeError> {
    let class_directory = match command_class {
        DistroCommandClass::Build => "build",
        DistroCommandClass::Format
        | DistroCommandClass::Test
        | DistroCommandClass::Benchmark
        | DistroCommandClass::RealDevice
        | DistroCommandClass::ProductionReadiness
        | DistroCommandClass::LiveReadiness => {
            return Err(ReferenceRuntimeError::CommandScopeMismatch);
        }
    };
    Ok(PathBuf::from(DISTRO_EVIDENCE_ROOT).join(class_directory))
}

fn validate_reference_writer_ownership(
    record: &DistroEvidenceRecord,
) -> Result<(), ReferenceRuntimeError> {
    if record.distro_layer != DistroLayer::Reference || record.target_plane != DistroPlane::Ops {
        return Err(ReferenceRuntimeError::CommandScopeMismatch);
    }

    match record.command_class {
        DistroCommandClass::Build => {
            if record.target_scope != "reference-distro/ops"
                || record.target_package.as_deref() != Some("arcrtc-reference-ops")
                || record.command.trim() != "cargo build --workspace --all-targets"
            {
                return Err(ReferenceRuntimeError::CommandScopeMismatch);
            }
        }
        DistroCommandClass::Format
        | DistroCommandClass::Test
        | DistroCommandClass::Benchmark
        | DistroCommandClass::RealDevice
        | DistroCommandClass::ProductionReadiness
        | DistroCommandClass::LiveReadiness => {
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
