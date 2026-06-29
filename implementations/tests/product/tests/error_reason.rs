//! product error reason mapping 境界を検査します。

use std::{fs, path::PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("implementations root must exist")
}

#[test]
fn product_errors_map_to_closed_implementation_reason_set() {
    for path in [
        "product-implementation/signaling/src/error.rs",
        "product-implementation/turn/src/error.rs",
        "product-implementation/sfu/src/error.rs",
        "product-implementation/product-policy/src/error.rs",
        "product-implementation/persistence-topology/src/error.rs",
        "product-implementation/deployment/src/error.rs",
        "product-implementation/monitoring/src/error.rs",
        "product-implementation/rollback/src/error.rs",
    ] {
        let source = fs::read_to_string(root().join(path)).expect("error source must be readable");
        assert!(source.contains("ImplementationEvidenceReason::"), "{path}");
        assert!(!source.contains("Unknown"), "{path}");
        assert!(!source.contains("Other"), "{path}");
    }
}
