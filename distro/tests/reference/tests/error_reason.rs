//! reference error reason mapping 境界を検査します。

use std::{fs, path::PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("distro root must exist")
}

#[test]
fn reference_errors_map_to_closed_distro_reason_set() {
    for path in [
        "reference-distro/signaling/src/error.rs",
        "reference-distro/turn/src/error.rs",
        "reference-distro/sfu/src/error.rs",
        "reference-distro/composition/src/error.rs",
        "reference-distro/ops/src/error.rs",
    ] {
        let source = fs::read_to_string(root().join(path)).expect("error source must be readable");
        assert!(source.contains("DistroEvidenceReason::"));
        assert!(!source.contains("Unknown"));
        assert!(!source.contains("Other"));
    }
}
