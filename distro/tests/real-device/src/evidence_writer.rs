//! 検証済み real-device evidence record の永続化境界です。

use std::{
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use crate::evidence::{validate_real_device_evidence_record, RealDeviceEvidenceRecord};
use arcrtc_distro_evidence::DISTRO_EVIDENCE_ROOT;

/// real-device evidence の書き込みerrorです。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RealDeviceEvidenceWriteError {
    /// record validation が失敗しました。
    ValidationFailed,
    /// record の保存先が固定root外です。
    PathOutsideEvidenceRoot,
    /// JSON serialization が失敗しました。
    SerializationFailed,
    /// evidence directory を作成できませんでした。
    DirectoryCreationFailed,
    /// temporary evidence file を書き込めませんでした。
    WriteFailed,
    /// temporary evidence file を確定先へ移動できませんでした。
    CommitFailed,
}

/// 検証済みrecordをdistro root配下の固定evidence pathへatomicに保存します。
pub fn write_real_device_evidence_record(
    distro_root: &Path,
    record: &RealDeviceEvidenceRecord,
) -> Result<PathBuf, RealDeviceEvidenceWriteError> {
    validate_real_device_evidence_record(record)
        .map_err(|_| RealDeviceEvidenceWriteError::ValidationFailed)?;

    let repository_relative_root = format!("{DISTRO_EVIDENCE_ROOT}/real-device/");
    let relative_path = record
        .logs_metrics_location
        .strip_prefix(&repository_relative_root)
        .ok_or(RealDeviceEvidenceWriteError::PathOutsideEvidenceRoot)?;
    let file_name = Path::new(relative_path);
    if file_name.components().count() != 1 || file_name.extension().is_none_or(|ext| ext != "json")
    {
        return Err(RealDeviceEvidenceWriteError::PathOutsideEvidenceRoot);
    }

    let output_dir = distro_root.join("target/distro-evidence/real-device");
    std::fs::create_dir_all(&output_dir)
        .map_err(|_| RealDeviceEvidenceWriteError::DirectoryCreationFailed)?;
    let output_path = output_dir.join(file_name);
    let json = serde_json::to_vec_pretty(record)
        .map_err(|_| RealDeviceEvidenceWriteError::SerializationFailed)?;
    let temporary_path = unique_temporary_path(&output_path);
    let mut temporary_file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary_path)
        .map_err(|_| RealDeviceEvidenceWriteError::WriteFailed)?;
    if temporary_file
        .write_all(&json)
        .and_then(|_| temporary_file.sync_all())
        .is_err()
    {
        let _ = std::fs::remove_file(&temporary_path);
        return Err(RealDeviceEvidenceWriteError::WriteFailed);
    }
    if std::fs::rename(&temporary_path, &output_path).is_err() {
        let _ = std::fs::remove_file(&temporary_path);
        return Err(RealDeviceEvidenceWriteError::CommitFailed);
    }
    std::fs::File::open(&output_dir)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| RealDeviceEvidenceWriteError::CommitFailed)?;
    Ok(output_path)
}

fn unique_temporary_path(output_path: &Path) -> PathBuf {
    static NEXT_TEMPORARY_ID: AtomicU64 = AtomicU64::new(0);
    let id = NEXT_TEMPORARY_ID.fetch_add(1, Ordering::Relaxed);
    output_path.with_extension(format!("json.{}.{}.tmp", std::process::id(), id))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{write_real_device_evidence_record, RealDeviceEvidenceWriteError};
    use crate::{
        command_output::RealDeviceCommandOutput, device_observation::parse_real_device_observation,
        dispatch::dispatch_kpi_real_device_success_command, evidence::RealDeviceClass,
        wrapper::build_kpi_real_device_success_record,
    };

    fn temporary_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "arcrtc-real-device-evidence-writer-{label}-{}",
            std::process::id()
        ))
    }

    fn android_physical_record() -> crate::evidence::RealDeviceEvidenceRecord {
        let dispatch = dispatch_kpi_real_device_success_command(
            crate::cli::CliPlatform::Android,
            crate::cli::CliDeviceClass::AndroidPhysical,
        )
        .expect("android physical dispatch");
        let output = RealDeviceCommandOutput::success(
            "List of devices attached\nABC123 device product:pixel",
            "",
            "adb devices -l",
        );
        let observation = parse_real_device_observation(&dispatch, &output).expect("observation");
        build_kpi_real_device_success_record(
            dispatch,
            "reference-local".to_owned(),
            output,
            Some(observation),
        )
    }

    #[test]
    fn writer_persists_validated_record_at_fixed_path() {
        let root = temporary_root("success");
        let _ = std::fs::remove_dir_all(&root);
        let record = android_physical_record();
        let output_path =
            write_real_device_evidence_record(&root, &record).expect("write evidence");
        let persisted: crate::evidence::RealDeviceEvidenceRecord =
            serde_json::from_slice(&std::fs::read(&output_path).expect("read persisted evidence"))
                .expect("parse persisted evidence");
        assert_eq!(persisted.base.correlation_id, "KPI-011-ANDROID-PHYSICAL");
        assert_eq!(persisted.device_class, RealDeviceClass::AndroidPhysical);
        assert_eq!(persisted.base.exit_status, Some(0));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn writer_rejects_path_outside_fixed_root() {
        let root = temporary_root("outside");
        let _ = std::fs::remove_dir_all(&root);
        let mut record = android_physical_record();
        record.logs_metrics_location =
            "distro/target/distro-evidence/real-device/../android-physical.json".to_owned();
        assert_eq!(
            write_real_device_evidence_record(&root, &record),
            Err(RealDeviceEvidenceWriteError::PathOutsideEvidenceRoot)
        );
        assert!(!root.exists());
    }

    #[test]
    fn concurrent_writers_leave_one_valid_record_and_no_temporary_files() {
        let root = temporary_root("concurrent");
        let _ = std::fs::remove_dir_all(&root);
        let first_root = root.clone();
        let second_root = root.clone();
        let first_record = android_physical_record();
        let second_record = first_record.clone();
        let first = std::thread::spawn(move || {
            write_real_device_evidence_record(&first_root, &first_record)
        });
        let second = std::thread::spawn(move || {
            write_real_device_evidence_record(&second_root, &second_record)
        });
        let output_path = first.join().expect("first writer").expect("first write");
        second.join().expect("second writer").expect("second write");
        let persisted: crate::evidence::RealDeviceEvidenceRecord =
            serde_json::from_slice(&std::fs::read(&output_path).expect("read persisted evidence"))
                .expect("parse persisted evidence");
        assert_eq!(persisted.base.exit_status, Some(0));
        let output_dir = output_path.parent().expect("output directory");
        assert!(std::fs::read_dir(output_dir)
            .expect("read output directory")
            .all(|entry| !entry
                .expect("entry")
                .path()
                .to_string_lossy()
                .ends_with(".tmp")));
        let _ = std::fs::remove_dir_all(root);
    }
}
