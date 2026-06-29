//! public module export surface が Canonical 境界から逸脱していないことを検査します。

use std::{fs, path::PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("implementations root must exist")
}

fn read(path: &str) -> String {
    fs::read_to_string(root().join(path)).expect("source file must be readable")
}

fn assert_ordered(source: &str, expected: &[&str], path: &str) {
    let mut previous = 0;
    for item in expected {
        let index = source[previous..]
            .find(item)
            .map(|offset| previous + offset)
            .unwrap_or_else(|| panic!("{path} missing ordered export {item}"));
        previous = index + item.len();
    }
}

#[test]
fn support_evidence_exports_match_canonical_surface() {
    let lib = read("implementation-support/evidence/src/lib.rs");
    assert_ordered(
        &lib,
        &[
            "pub mod error;",
            "pub mod reason;",
            "pub mod record;",
            "pub mod readiness_extension;",
            "pub mod validation;",
            "pub use error::ImplementationEvidenceError;",
            "pub use readiness_extension::",
            "pub use reason::ImplementationEvidenceReason;",
            "pub use record::",
            "pub use validation::",
        ],
        "implementation-support/evidence/src/lib.rs",
    );
    for expected in [
        "pub mod error;",
        "pub mod reason;",
        "pub mod record;",
        "pub mod readiness_extension;",
        "pub mod validation;",
        "pub use error::ImplementationEvidenceError;",
        "pub use reason::ImplementationEvidenceReason;",
        "ImplementationEvidenceRecord",
        "validate_evidence_record",
        "IMPLEMENTATIONS_COMMAND_ROOT",
        "IMPLEMENTATIONS_EVIDENCE_ROOT",
        "IMPLEMENTATIONS_TARGET_ROOT",
    ] {
        assert!(lib.contains(expected), "missing export {expected}");
    }
}

#[test]
fn product_deployment_exports_match_canonical_order() {
    let lib = read("product-implementation/deployment/src/lib.rs");
    assert_ordered(
        &lib,
        &[
            "pub mod error;",
            "pub mod live_endpoint;",
            "pub mod profile;",
            "pub mod production_profile;",
            "pub mod runtime;",
            "pub use error::ProductRuntimeError;",
            "pub use live_endpoint::",
            "pub use profile::",
            "pub use production_profile::build_product_production_profile;",
            "pub use runtime::",
        ],
        "product-implementation/deployment/src/lib.rs",
    );
}

#[test]
fn reference_and_product_libs_do_not_export_unlisted_private_modules() {
    for path in [
        "reference-implementation/signaling/src/lib.rs",
        "reference-implementation/turn/src/lib.rs",
        "reference-implementation/sfu/src/lib.rs",
        "reference-implementation/composition/src/lib.rs",
        "reference-implementation/ops/src/lib.rs",
        "product-implementation/signaling/src/lib.rs",
        "product-implementation/turn/src/lib.rs",
        "product-implementation/sfu/src/lib.rs",
        "product-implementation/product-policy/src/lib.rs",
        "product-implementation/persistence-topology/src/lib.rs",
        "product-implementation/deployment/src/lib.rs",
        "product-implementation/monitoring/src/lib.rs",
        "product-implementation/rollback/src/lib.rs",
    ] {
        let lib = read(path);
        assert!(
            !lib.contains("pub mod kernel_contract;")
                || path.contains("signaling")
                || path.contains("turn")
                || path.contains("sfu"),
            "{path} exposes unexpected kernel_contract module"
        );
        assert!(
            !lib.contains("pub use arcrtc_reference_output::*"),
            "{path} must not wildcard re-export reference output"
        );
    }
}
