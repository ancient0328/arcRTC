use arcrtc_roadmap_integration_tests::{
    assert_impl_file_contains, assert_not_contains, file_exists, read_impl,
};

#[test]
fn controlled_integration_asset_surfaces_exist() {
    for (path, task) in [
        (
            "integration-tests/signaling/SIGNALING_CONTROLLED_INTEGRATION_ASSET.md",
            "T7.1",
        ),
        (
            "integration-tests/turn-relay/TURN_RELAY_GATE_INTEGRATION_ASSET.md",
            "T7.2",
        ),
        (
            "integration-tests/turn-blocked-network/TURN_BLOCKED_NETWORK_OBSERVATION_ASSET.md",
            "T7.3",
        ),
        (
            "integration-tests/sfu-three-party/SFU_THREE_PARTY_ROUTING_INTEGRATION_ASSET.md",
            "T7.4",
        ),
        (
            "integration-tests/cross-plane-binding/CROSS_PLANE_BINDING_INTEGRATION_ASSET.md",
            "T7.5",
        ),
    ] {
        assert!(file_exists(path), "{path} must exist");
        assert_impl_file_contains(
            path,
            &[
                task,
                "controlled integration",
                "runtime-in-test",
                "Close-not-claimed",
            ],
        );
    }
}

#[test]
fn server_composition_manifests_wire_only_allowed_layers() {
    for path in [
        "entrypoints/signaling-server/Cargo.toml",
        "entrypoints/sfu-server/Cargo.toml",
        "entrypoints/turn-server/Cargo.toml",
    ] {
        let content = read_impl(path);
        assert!(content.contains("owner_layer = \"entrypoints\""), "{path}");
        assert!(content.contains("arcrtc-core-"), "{path}");
        assert!(content.contains("arcrtc-driver-"), "{path}");
        assert_not_contains(
            path,
            &content,
            &["arcrtc-regulated", "../../regulated", "../../sdk"],
        );
    }
}

#[test]
fn controlled_integration_assets_do_not_claim_live_network_or_production_readiness() {
    for path in [
        "integration-tests/signaling/SIGNALING_CONTROLLED_INTEGRATION_ASSET.md",
        "integration-tests/turn-relay/TURN_RELAY_GATE_INTEGRATION_ASSET.md",
        "integration-tests/turn-blocked-network/TURN_BLOCKED_NETWORK_OBSERVATION_ASSET.md",
        "integration-tests/sfu-three-party/SFU_THREE_PARTY_ROUTING_INTEGRATION_ASSET.md",
        "integration-tests/cross-plane-binding/CROSS_PLANE_BINDING_INTEGRATION_ASSET.md",
    ] {
        let content = read_impl(path);
        assert!(content.contains("not live production"));
        assert!(content.contains("not public internet traversal proof"));
    }
}
