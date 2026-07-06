//! reference API signature と ownership 境界を検査します。

use std::{fs, path::PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("distro root must exist")
}

#[test]
fn reference_public_api_exports_match_ownership_boundaries() {
    for (path, expected_symbols) in [
        (
            "reference-distro/signaling/src/lib.rs",
            &[
                "pub use state::{",
                "apply_reference_signaling",
                "project_reference_signaling_event",
                "validate_reference_signaling_state",
            ][..],
        ),
        (
            "reference-distro/turn/src/lib.rs",
            &[
                "pub use state::{",
                "apply_reference_turn",
                "validate_reference_turn_state",
            ][..],
        ),
        (
            "reference-distro/sfu/src/lib.rs",
            &[
                "pub use state::{",
                "apply_reference_sfu",
                "validate_reference_sfu_state",
            ][..],
        ),
        (
            "reference-distro/composition/src/lib.rs",
            &[
                "pub use runtime_bridge::{",
                "run_reference_composition_step",
                "ReferenceCompositionStepOutcome",
            ][..],
        ),
    ] {
        let source = fs::read_to_string(root().join(path)).expect("lib source must be readable");
        for expected in expected_symbols {
            assert!(source.contains(expected), "{path} missing {expected}");
        }
    }
}
