//! Test Roadmap が所有する explicit test package と target file shape を検査します。

use std::{fs, path::PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("distro root must exist")
}

fn read(path: &str) -> String {
    fs::read_to_string(root().join(path)).expect("test manifest must be readable")
}

#[test]
fn explicit_test_package_manifests_match_canonical_package_names() {
    for (path, name) in [
        (
            "tests/boundary/Cargo.toml",
            "arcrtc-distro-boundary-tests",
        ),
        (
            "tests/reference/Cargo.toml",
            "arcrtc-distro-reference-tests",
        ),
        (
            "tests/product/Cargo.toml",
            "arcrtc-distro-product-tests",
        ),
        (
            "tests/benchmark/Cargo.toml",
            "arcrtc-distro-benchmark-tests",
        ),
        (
            "tests/real-device/Cargo.toml",
            "arcrtc-distro-real-device-tests",
        ),
        (
            "tests/production-readiness/Cargo.toml",
            "arcrtc-distro-production-readiness-tests",
        ),
        ("tests/live/Cargo.toml", "arcrtc-distro-live-tests"),
    ] {
        let body = read(path);
        assert!(body.contains(&format!("name = \"{name}\"")), "{path}");
        assert!(body.contains("edition = \"2021\""), "{path}");
        assert!(body.contains("rust-version = \"1.96\""), "{path}");
        assert!(body.contains("[workspace]"), "{path}");
        assert!(body.contains("unsafe_code = \"forbid\""), "{path}");
    }
}

#[test]
fn target_bound_test_files_exist() {
    for path in [
        "tests/boundary/tests/old_vocabulary.rs",
        "tests/boundary/tests/command_evidence.rs",
        "tests/boundary/tests/evidence_schema.rs",
        "tests/boundary/tests/dependency_admission.rs",
        "tests/boundary/tests/secret_literal.rs",
        "tests/boundary/tests/module_export.rs",
        "tests/boundary/tests/test_package_shape.rs",
        "tests/boundary/tests/evidence_owner.rs",
        "tests/boundary/tests/product_reference_output_boundary.rs",
        "tests/reference/tests/signaling_contract.rs",
        "tests/reference/tests/signaling_state.rs",
        "tests/reference/tests/signaling_fixture.rs",
        "tests/reference/tests/turn_contract.rs",
        "tests/reference/tests/turn_state.rs",
        "tests/reference/tests/turn_fixture.rs",
        "tests/reference/tests/sfu_contract.rs",
        "tests/reference/tests/sfu_state.rs",
        "tests/reference/tests/sfu_fixture.rs",
        "tests/reference/tests/composition.rs",
        "tests/reference/tests/runtime.rs",
        "tests/reference/tests/error_reason.rs",
        "tests/reference/tests/reference_api_signature.rs",
        "tests/reference/tests/borrowed_packet_view.rs",
        "tests/reference/tests/profile.rs",
        "tests/product/tests/product_api.rs",
        "tests/product/tests/product_policy.rs",
        "tests/product/tests/persistence_topology.rs",
        "tests/product/tests/deployment.rs",
        "tests/product/tests/monitoring.rs",
        "tests/product/tests/rollback.rs",
        "tests/product/tests/error_reason.rs",
        "tests/benchmark/tests/scenario_set.rs",
        "tests/benchmark/tests/measurement_schema.rs",
        "tests/benchmark/tests/harness_layout.rs",
        "tests/benchmark/tests/comparison_row.rs",
        "tests/real-device/tests/command_matrix.rs",
        "tests/real-device/tests/non_claim_scope.rs",
        "tests/real-device/tests/wrapper_command.rs",
        "tests/production-readiness/tests/production_matrix.rs",
        "tests/production-readiness/tests/fail_closed.rs",
        "tests/live/tests/live_matrix.rs",
        "tests/live/tests/fail_closed.rs",
    ] {
        assert!(root().join(path).is_file(), "missing {path}");
    }
}
