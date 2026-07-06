//! reference local / benchmark profile literal 境界を検査します。

use std::{fs, path::PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("distro root must exist")
}

#[test]
fn reference_profiles_exist_without_production_or_live_claims() {
    for path in [
        "reference-distro/deployment-profiles/local.toml",
        "reference-distro/deployment-profiles/benchmark.toml",
    ] {
        let source = fs::read_to_string(root().join(path)).expect("profile must be readable");
        assert!(source.contains("profile"));
        assert!(!source.contains("production_ready = true"));
        assert!(!source.contains("live_ready = true"));
    }
}
