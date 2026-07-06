use arcrtc_roadmap_integration_tests::{assert_not_contains, read_impl};

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
