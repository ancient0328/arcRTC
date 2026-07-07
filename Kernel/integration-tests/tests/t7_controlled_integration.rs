use arcrtc_roadmap_integration_tests::{assert_not_contains, read_impl};

fn production_dependencies_section(content: &str) -> &str {
    let start = content
        .find("[dependencies]")
        .expect("entrypoint manifest must declare production dependencies");
    let body = &content[start + "[dependencies]".len()..];
    match body.find("\n[") {
        Some(end) => &body[..end],
        None => body,
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
        // server main は複数 capability を編成せず、composition-root への委譲だけを保持します。
        assert!(
            content.contains("arcrtc-entrypoint-composition-root"),
            "{path}"
        );
        let production_dependencies = production_dependencies_section(&content);
        assert_not_contains(
            path,
            production_dependencies,
            &[
                "arcrtc-core-",
                "arcrtc-driver-",
                "arcrtc-regulated",
                "../../regulated",
                "../../sdk",
            ],
        );
    }
}
