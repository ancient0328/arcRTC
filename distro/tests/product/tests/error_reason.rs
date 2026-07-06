//! product error reason mapping 境界を検査します。

use std::{fs, path::PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("distro root must exist")
}

#[test]
fn product_errors_map_to_closed_distro_reason_set() {
    for path in [
        "product-distro/signaling/src/error.rs",
        "product-distro/turn/src/error.rs",
        "product-distro/sfu/src/error.rs",
        "product-distro/product-policy/src/error.rs",
        "product-distro/persistence-topology/src/error.rs",
        "product-distro/deployment/src/error.rs",
        "product-distro/monitoring/src/error.rs",
        "product-distro/rollback/src/error.rs",
    ] {
        let source = fs::read_to_string(root().join(path)).expect("error source must be readable");
        assert!(source.contains("DistroEvidenceReason::"), "{path}");
        assert!(!source.contains("Unknown"), "{path}");
        assert!(!source.contains("Other"), "{path}");
    }
}
