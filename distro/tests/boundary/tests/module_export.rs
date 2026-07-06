//! public module export surface が Canonical 境界から逸脱していないことを検査します。

use std::{fs, path::PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("distro root must exist")
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
    let lib = read("distro-support/evidence/src/lib.rs");
    assert_ordered(
        &lib,
        &[
            "pub mod error;",
            "pub mod reason;",
            "pub mod record;",
            "pub mod readiness_extension;",
            "pub mod validation;",
            "pub use error::DistroEvidenceError;",
            "pub use readiness_extension::",
            "pub use reason::DistroEvidenceReason;",
            "pub use record::",
            "pub use validation::",
        ],
        "distro-support/evidence/src/lib.rs",
    );
    for expected in [
        "pub mod error;",
        "pub mod reason;",
        "pub mod record;",
        "pub mod readiness_extension;",
        "pub mod validation;",
        "pub use error::DistroEvidenceError;",
        "pub use reason::DistroEvidenceReason;",
        "DistroEvidenceRecord",
        "validate_evidence_record",
        "DISTRO_COMMAND_ROOT",
        "DISTRO_EVIDENCE_ROOT",
        "DISTRO_TARGET_ROOT",
    ] {
        assert!(lib.contains(expected), "missing export {expected}");
    }
}

#[test]
fn product_deployment_exports_match_canonical_order() {
    let lib = read("product-distro/deployment/src/lib.rs");
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
        "product-distro/deployment/src/lib.rs",
    );
}

#[test]
fn reference_and_product_libs_do_not_export_unlisted_private_modules() {
    for path in [
        "reference-distro/signaling/src/lib.rs",
        "reference-distro/turn/src/lib.rs",
        "reference-distro/sfu/src/lib.rs",
        "reference-distro/composition/src/lib.rs",
        "reference-distro/ops/src/lib.rs",
        "product-distro/signaling/src/lib.rs",
        "product-distro/turn/src/lib.rs",
        "product-distro/sfu/src/lib.rs",
        "product-distro/product-policy/src/lib.rs",
        "product-distro/persistence-topology/src/lib.rs",
        "product-distro/deployment/src/lib.rs",
        "product-distro/monitoring/src/lib.rs",
        "product-distro/rollback/src/lib.rs",
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
