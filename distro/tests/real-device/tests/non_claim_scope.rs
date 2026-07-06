//! real-device evidence non-claim scope 境界を検査します。

use std::{fs, path::PathBuf, process::Command};

use arcrtc_distro_evidence::DistroNonClaimScope;
use serde_json::Value;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("distro root must exist")
}

#[test]
fn kpi_real_device_result_does_not_admit_live_readiness() {
    let output = Command::new(env!(
        "CARGO_BIN_EXE_arcrtc-distro-real-device-tests"
    ))
    .current_dir(root())
    .args([
        "browser",
        "--profile",
        "reference-local",
        "--device-class",
        "desktop-browser",
    ])
    .output()
    .expect("wrapper binary must execute");
    assert_eq!(output.status.code(), Some(0));

    let record: Value =
        serde_json::from_slice(&output.stdout).expect("wrapper stdout must be evidence json");
    let scopes = record["non_claim_scope"]
        .as_array()
        .expect("non_claim_scope must be array");
    for expected in [
        DistroNonClaimScope::NativeApplicationReadinessNotClaimed,
        DistroNonClaimScope::PublicDistributionReadinessNotClaimed,
        DistroNonClaimScope::ProductionReadinessNotClaimed,
        DistroNonClaimScope::LiveReadinessNotClaimed,
        DistroNonClaimScope::KernelCompletionNotClaimed,
    ] {
        let expected = serde_json::to_value(expected).expect("scope must serialize");
        assert!(scopes.contains(&expected), "missing {expected}");
    }
    assert_eq!(
        record["actual_outcome"],
        "real-device success evidence row observed with raw identifier absent"
    );
}

#[test]
fn real_device_evidence_includes_all_required_non_claim_scopes() {
    let source = fs::read_to_string(root().join("tests/real-device/src/wrapper.rs"))
        .expect("wrapper source must be readable");
    for expected in [
        "NativeApplicationReadinessNotClaimed",
        "PublicDistributionReadinessNotClaimed",
        "ProductionReadinessNotClaimed",
        "LiveReadinessNotClaimed",
        "KernelCompletionNotClaimed",
    ] {
        assert!(source.contains(expected), "missing {expected}");
    }
}
