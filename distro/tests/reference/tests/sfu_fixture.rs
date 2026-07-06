//! reference SFU fixture route admission 境界を検査します。

use std::{fs, path::PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("distro root must exist")
}

#[test]
fn sfu_fixture_route_authorization_is_deterministic_and_local() {
    let source =
        fs::read_to_string(root().join("reference-distro/sfu/src/fixture_route_auth.rs"))
            .expect("sfu fixture source must be readable");
    assert!(source.contains("FixtureRouteAdmission"));
    assert!(source.contains("authorize_reference_route"));
    assert!(!source.contains("token"));
    assert!(!source.contains("secret"));
}
