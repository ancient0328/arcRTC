//! product plane が参照できる reference output surface を検査します。

use std::{fs, path::PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("distro root must exist")
}

fn read(path: &str) -> String {
    fs::read_to_string(root().join(path)).expect("product source must be readable")
}

#[test]
fn product_plane_imports_only_three_reference_output_outcome_types() {
    assert_product_plane_consumes_only_three_reference_outputs();
}

#[test]
fn product_plane_does_not_reexport_reference_output() {
    for path in [
        "product-distro/signaling/src/lib.rs",
        "product-distro/turn/src/lib.rs",
        "product-distro/sfu/src/lib.rs",
    ] {
        let body = read(path);
        assert!(!body.contains("pub use arcrtc_reference_output"));
    }
}

fn assert_product_plane_consumes_only_three_reference_outputs() {
    for (path, allowed) in allowed_product_reference_outputs() {
        let body = read(path);
        assert!(body.contains(allowed), "{path} must import {allowed}");
        for forbidden in [
            "arcrtc_reference_output::*",
            "use arcrtc_reference_output;",
            "ReferenceCompositionOutcome",
            "arcrtc_reference_signaling",
            "arcrtc_reference_turn",
            "arcrtc_reference_sfu",
            "arcrtc_reference_composition",
            "arcrtc_reference_ops",
            "ReferenceSignalingState",
            "ReferenceTurnState",
            "ReferenceSfuState",
            "ReferenceSfuAction",
        ] {
            assert!(!body.contains(forbidden), "{path} contains {forbidden}");
        }
    }

    for path in product_rust_sources() {
        let path_string = path
            .strip_prefix(root())
            .expect("source must be under distro root")
            .to_string_lossy()
            .into_owned();
        if allowed_product_reference_outputs()
            .iter()
            .any(|(allowed_path, _)| *allowed_path == path_string)
        {
            continue;
        }

        let body = fs::read_to_string(&path).expect("product source must be readable");
        for forbidden in [
            "arcrtc_reference_output",
            "ReferenceSignalingOutcome",
            "ReferenceTurnOutcome",
            "ReferenceSfuOutcome",
            "ReferenceCompositionOutcome",
        ] {
            assert!(
                !body.contains(forbidden),
                "{path_string} must not consume reference output symbol {forbidden}"
            );
        }
    }
}

fn allowed_product_reference_outputs() -> &'static [(&'static str, &'static str)] {
    &[
        (
            "product-distro/signaling/src/kernel_contract.rs",
            "arcrtc_reference_output::ReferenceSignalingOutcome",
        ),
        (
            "product-distro/turn/src/kernel_contract.rs",
            "arcrtc_reference_output::ReferenceTurnOutcome",
        ),
        (
            "product-distro/sfu/src/kernel_contract.rs",
            "arcrtc_reference_output::ReferenceSfuOutcome",
        ),
    ]
}

fn product_rust_sources() -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rust_sources(root().join("product-distro"), &mut files);
    files
}

fn collect_rust_sources(path: PathBuf, files: &mut Vec<PathBuf>) {
    if path.is_file() {
        if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
        return;
    }
    for entry in fs::read_dir(path).expect("product directory must be readable") {
        collect_rust_sources(
            entry.expect("directory entry must be readable").path(),
            files,
        );
    }
}
