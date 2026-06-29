use arcrtc_roadmap_tests::{
    assert_impl_file_contains, assert_not_contains, files_named, impl_relative,
    implementation_root, read_file, read_impl_rust_source_set,
};

#[test]
fn driver_contract_asset_surfaces_exist() {
    for (path, task) in [
        (
            "tests/drivers/network/NETWORK_CONVERSION_CONTRACT_ASSET.md",
            "T3.1",
        ),
        (
            "tests/drivers/turn-wire/TURN_WIRE_CONTRACT_ASSET.md",
            "T3.2",
        ),
        (
            "tests/drivers/webrtc-str0m/WEBRTC_STR0M_CONTRACT_ASSET.md",
            "T3.3",
        ),
        (
            "tests/drivers/platform/PLATFORM_DRIVER_CONTRACT_ASSET.md",
            "T3.4",
        ),
    ] {
        assert_impl_file_contains(
            path,
            &[
                task,
                "contract-test asset",
                "driver-owned conversion",
                "forbidden core semantic ownership",
            ],
        );
    }
}

#[test]
fn drivers_depend_on_core_without_depending_on_entrypoints_or_regulated() {
    let root = implementation_root().join("drivers");
    for manifest in files_named(&root, "Cargo.toml") {
        let relative = impl_relative(&manifest);
        let content = read_file(&manifest);
        assert!(content.contains("owner_layer = \"drivers\""), "{relative}");
        assert!(content.contains("arcrtc-core-"), "{relative}");
        assert_not_contains(
            &relative,
            &content,
            &[
                "arcrtc-entrypoint-",
                "arcrtc-regulated",
                "../../entrypoints",
                "../../regulated",
            ],
        );
    }
}

#[test]
fn drivers_expose_ports_guards_failures_or_prohibited_behavior_markers() {
    for source in files_named(&implementation_root().join("drivers"), "lib.rs") {
        let relative = impl_relative(&source);
        let crate_dir = relative
            .strip_suffix("/src/lib.rs")
            .expect("driver source path must be a crate lib.rs");
        let content = read_impl_rust_source_set(&format!("{crate_dir}/src"));
        assert!(
            content.contains("Prohibited")
                || content.contains("Guard")
                || content.contains("Failure")
                || content.contains("impl CorePort"),
            "{crate_dir} must expose contract boundary markers"
        );
    }
}
