//! shared evidence / reason owner の単一性を検査します。

use std::{fs, path::PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("implementations root must exist")
}

#[test]
fn shared_evidence_reason_and_record_are_owned_only_by_support_package() {
    let mut duplicate_owners = Vec::new();
    for file in rust_files() {
        let relative = file.strip_prefix(root()).expect("file must be under root");
        let relative_string = relative.to_string_lossy();
        if relative_string.starts_with("implementation-support/evidence/src/") {
            continue;
        }
        let body = fs::read_to_string(&file).expect("source file must be readable");
        for marker in [
            "pub enum ImplementationEvidenceReason",
            "pub struct ImplementationEvidenceRecord",
            "pub enum ImplementationNonClaimScope",
        ] {
            if body.contains(marker) {
                duplicate_owners.push(format!("{} contains {marker}", relative_string));
            }
        }
    }
    assert!(
        duplicate_owners.is_empty(),
        "{}",
        duplicate_owners.join("\n")
    );
}

#[test]
fn evidence_closed_set_contains_no_unknown_or_raw_string_reason() {
    let reason = fs::read_to_string(root().join("implementation-support/evidence/src/reason.rs"))
        .expect("reason owner must be readable");
    assert!(!reason.contains("Unknown"));
    assert!(!reason.contains("Other"));
    assert!(!reason.contains("String"));
}

fn rust_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    for path in [
        "implementation-support",
        "reference-implementation",
        "product-implementation",
    ] {
        collect(root().join(path), &mut files);
    }
    files
}

fn collect(path: PathBuf, files: &mut Vec<PathBuf>) {
    if path.is_file() {
        if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
        return;
    }
    for entry in fs::read_dir(path).expect("source directory must be readable") {
        collect(
            entry.expect("directory entry must be readable").path(),
            files,
        );
    }
}
