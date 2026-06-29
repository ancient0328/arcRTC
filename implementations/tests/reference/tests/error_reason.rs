//! reference error reason mapping 境界を検査します。

use std::{fs, path::PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("implementations root must exist")
}

#[test]
fn reference_errors_map_to_closed_implementation_reason_set() {
    for path in [
        "reference-implementation/signaling/src/error.rs",
        "reference-implementation/turn/src/error.rs",
        "reference-implementation/sfu/src/error.rs",
        "reference-implementation/composition/src/error.rs",
        "reference-implementation/ops/src/error.rs",
    ] {
        let source = fs::read_to_string(root().join(path)).expect("error source must be readable");
        assert!(source.contains("ImplementationEvidenceReason::"));
        assert!(!source.contains("Unknown"));
        assert!(!source.contains("Other"));
    }
}
