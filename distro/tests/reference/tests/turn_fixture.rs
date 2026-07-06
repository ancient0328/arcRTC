//! reference TURN fixture credential 境界を検査します。

use std::{fs, path::PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("distro root must exist")
}

#[test]
fn turn_fixture_credential_rejects_empty_or_zero_lifetime_input() {
    let source =
        fs::read_to_string(root().join("reference-distro/turn/src/fixture_credential.rs"))
            .expect("turn fixture source must be readable");
    for expected in [
        "credential.credential_ref.as_str().is_empty()",
        "credential.allocation_id.as_str().is_empty()",
        "credential.requested_lifetime.as_u32() == 0",
        "ReferenceTurnError::InvalidFixtureCredential",
    ] {
        assert!(source.contains(expected), "missing {expected}");
    }
}
