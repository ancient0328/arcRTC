use arcrtc_roadmap_tests::{
    assert_not_contains, files_named, impl_relative, implementation_root, read_file, read_impl,
    read_impl_rust_source_set, source_files_with_extension,
};

#[test]
fn ce1_workspace_membership_matches_declared_v0_2_layer_surface() {
    let manifest = read_impl("Cargo.toml");
    for member in [
        "core/transport",
        "core/signaling",
        "core/sfu",
        "core/turn",
        "core/security",
        "core/audit",
        "core/quality",
        "core/ports",
        "core/domain",
        "core/identity",
        "core/command",
        "core/reason",
        "core/protocol",
        "core/configuration",
        "core/features",
        "core/cross-plane",
        "core/operation",
        "core/runtime",
        "core/time",
        "core/state",
        "core/recovery",
        "drivers/webrtc-str0m",
        "drivers/network",
        "drivers/persistence",
        "drivers/observability",
        "drivers/security",
        "drivers/browser",
        "drivers/native",
        "entrypoints/signaling-server",
        "entrypoints/sfu-server",
        "entrypoints/turn-server",
        "entrypoints/cli",
        "entrypoints/demo",
        "entrypoints/configuration",
        "entrypoints/endpoints",
        "entrypoints/topology",
        "entrypoints/internal-control",
        "entrypoints/admin",
        "regulated",
        "tests",
        "integration-tests",
    ] {
        assert!(
            manifest.contains(&format!("\"{member}\"")),
            "workspace manifest must include {member}"
        );
    }
}

#[test]
fn ce1_required_directories_are_not_replaced_by_docs_or_generated_targets() {
    let root = implementation_root();
    for directory in [
        "core",
        "drivers",
        "entrypoints",
        "sdk",
        "regulated",
        "tests",
        "integration-tests",
    ] {
        assert!(
            root.join(directory).is_dir(),
            "implementation directory {directory} must exist"
        );
    }
}

#[test]
fn ce1_semantic_modular_monolith_keeps_line_count_as_review_signal() {
    let mut inspected_sources = 0usize;
    let mut max_line_count = 0usize;
    for extension in ["rs", "kt", "swift", "ts", "mjs"] {
        for source in source_files_with_extension(&implementation_root(), extension) {
            let line_count = read_file(&source).lines().count();
            inspected_sources += 1;
            max_line_count = max_line_count.max(line_count);
        }
    }
    assert!(
        inspected_sources > 0 && max_line_count > 0,
        "source-size review signal must inspect the implementation source set"
    );
}

#[test]
fn ce1_semantic_modular_monolith_rejects_source_shape_collapse() {
    let root = implementation_root();
    let mut inspected_core_sources = 0usize;
    for source in source_files_with_extension(&root.join("core"), "rs") {
        inspected_core_sources += 1;
        let relative = impl_relative(&source);
        let content = read_file(&source);
        assert_not_contains(
            &relative,
            &content,
            &[
                "#[allow(clippy::too_many_arguments)]",
                "static mut",
                "lazy_static!",
                "once_cell",
                "Mutex<",
                "RwLock<",
                "arcrtc_driver_",
                "arcrtc_entrypoint_",
                "arcrtc_regulated",
            ],
        );

        if let Some((crate_src, shard_file)) = relative.split_once("/lib_parts/") {
            let lib_rs = root.join(crate_src).join("lib.rs");
            let lib_content = read_file(&lib_rs);
            assert!(
                lib_content.contains(&format!("include!(\"lib_parts/{shard_file}\");")),
                "{relative} must be visible through its crate source-set include map"
            );
            assert_not_contains(
                &relative,
                &content,
                &[
                    "pub mod ",
                    "mod drivers",
                    "mod entrypoints",
                    "mod regulated",
                ],
            );
        }
    }
    assert!(inspected_core_sources > 0);
}

#[test]
fn ce1_production_sources_do_not_hide_local_too_many_arguments_suppression() {
    for source in source_files_with_extension(&implementation_root(), "rs") {
        let relative = impl_relative(&source);
        if relative.starts_with("tests/") || relative.starts_with("integration-tests/") {
            continue;
        }
        let content = read_file(&source);
        assert_not_contains(
            &relative,
            &content,
            &["#[allow(clippy::too_many_arguments)]"],
        );
    }
}

#[test]
fn ce1_production_manifests_do_not_depend_on_test_or_integration_crates() {
    for manifest in files_named(&implementation_root(), "Cargo.toml") {
        let relative = impl_relative(&manifest);
        if relative.starts_with("tests/")
            || relative.starts_with("integration-tests/")
            || relative.starts_with("target/")
        {
            continue;
        }
        let content = read_file(&manifest);
        assert_not_contains(
            &relative,
            &content,
            &[
                "arcrtc-roadmap-tests",
                "arcrtc-roadmap-integration-tests",
                "../tests",
                "../integration-tests",
            ],
        );
    }
}

#[test]
fn ce1_core_source_shape_rejects_concrete_runtime_io_and_feature_inversion() {
    let root = implementation_root();
    for source in files_named(&root.join("core"), "lib.rs") {
        let relative = impl_relative(&source);
        let crate_dir = relative
            .strip_suffix("/src/lib.rs")
            .expect("core source path must be a crate lib.rs");
        let content = read_impl_rust_source_set(&format!("{crate_dir}/src"));
        assert_not_contains(
            crate_dir,
            &content,
            &[
                "tokio::",
                "std::net::",
                "web_sys::",
                "wasm_bindgen",
                "sqlx::",
                "rusqlite::",
                "aws_sdk",
                "reqwest::",
                "hyper::",
                "axum::",
                "cfg(feature",
            ],
        );
    }
}

#[test]
fn ce1_v01_material_is_not_v0_2_source_authority() {
    for manifest in files_named(&implementation_root(), "Cargo.toml") {
        let relative = impl_relative(&manifest);
        if relative.starts_with("target/") {
            continue;
        }
        let content = read_file(&manifest);
        assert_not_contains(
            &relative,
            &content,
            &["../v0.1", "../../v0.1", "CRarc/v0.1", "arcRTC/v0.1"],
        );
    }
}
