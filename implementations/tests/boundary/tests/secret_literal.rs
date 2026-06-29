//! product/reference source に raw secret literal が混入していないことを検査します。

use std::{fs, path::PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("implementations root must exist")
}

#[test]
fn source_and_current_reports_do_not_contain_raw_secret_literals() {
    assert_no_raw_identifier_or_secret_literal();
}

#[test]
fn kernel_production_implementability_rejects_raw_identifier_and_secret_literals() {
    assert_no_raw_identifier_or_secret_literal();
}

fn assert_no_raw_identifier_or_secret_literal() {
    let markers = [
        "AKIA",
        "-----BEGIN PRIVATE KEY-----",
        "-----BEGIN",
        "xoxb-",
        "ghp_",
        "secret=",
        "token=",
        "password=",
        "client_secret=",
        "aws_secret_access_key",
        "private_key",
        "packet_payload=",
        "raw_packet=",
        "authorization:",
        "bearer ",
        "serial:",
        "serial=",
        "udid:",
        "udid=",
        "android_id:",
        "android_id=",
        "device_id:",
        "device_id=",
        "imei:",
        "imei=",
        "meid:",
        "meid=",
        "account:",
        "account=",
        "private_key:",
        "private_key=",
        "device_name:",
        "device_name=",
    ];
    let mut findings = Vec::new();
    for file in scan_files() {
        let relative_path = file
            .strip_prefix(root())
            .expect("scanned file must be under implementations root")
            .to_string_lossy()
            .into_owned();
        if raw_marker_policy_owner_files().contains(&relative_path.as_str()) {
            continue;
        }
        let body = fs::read_to_string(&file).expect("scanned file must be readable");
        let lower = body.to_ascii_lowercase();
        for marker in markers {
            if lower.contains(&marker.to_ascii_lowercase()) {
                findings.push(format!("{} contains {marker}", file.display()));
            }
        }
    }
    assert!(findings.is_empty(), "{}", findings.join("\n"));
}

fn raw_marker_policy_owner_files() -> &'static [&'static str] {
    &["implementation-support/evidence/src/validation.rs"]
}

fn scan_files() -> Vec<PathBuf> {
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
        if matches!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("rs" | "toml" | "md" | "json")
        ) {
            files.push(path);
        }
        return;
    }
    for entry in fs::read_dir(path).expect("scan directory must be readable") {
        collect(
            entry.expect("directory entry must be readable").path(),
            files,
        );
    }
}
