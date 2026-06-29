use arcrtc_roadmap_tests::{
    assert_impl_file_contains, assert_not_contains, files_named, impl_relative,
    implementation_root, read_file, read_impl_rust_source_set,
};

#[test]
fn entrypoint_composition_asset_surfaces_exist() {
    assert_impl_file_contains(
        "tests/entrypoints/servers/SERVER_COMPOSITION_ASSET.md",
        &[
            "T4.1",
            "entrypoint composition test asset",
            "Signaling",
            "SFU",
            "TURN",
            "must not own domain semantics",
        ],
    );
    assert_impl_file_contains(
        "tests/entrypoints/operational-surfaces/OPERATIONAL_ENTRYPOINT_SURFACE_ASSET.md",
        &[
            "T4.2",
            "configuration",
            "endpoint",
            "topology",
            "internal-control",
            "health",
            "admin authorization",
        ],
    );
}

#[test]
fn entrypoints_compose_core_and_drivers_without_regulated_dependency() {
    for manifest in files_named(&implementation_root().join("entrypoints"), "Cargo.toml") {
        let relative = impl_relative(&manifest);
        let content = read_file(&manifest);
        assert!(
            content.contains("owner_layer = \"entrypoints\""),
            "{relative}"
        );
        assert!(content.contains("arcrtc-core-"), "{relative}");
        assert_not_contains(
            &relative,
            &content,
            &["arcrtc-regulated", "../../regulated", "../../sdk"],
        );
    }
}

#[test]
fn entrypoint_sources_keep_composition_marker_visible() {
    for source in files_named(&implementation_root().join("entrypoints"), "lib.rs")
        .into_iter()
        .chain(files_named(
            &implementation_root().join("entrypoints"),
            "main.rs",
        ))
    {
        let relative = impl_relative(&source);
        let source_dir = relative
            .rsplit_once('/')
            .map(|(dir, _)| dir)
            .expect("entrypoint source path must have a parent directory");
        let content = read_impl_rust_source_set(source_dir);
        let lower = content.to_ascii_lowercase();
        assert!(
            lower.contains("composition")
                || lower.contains("wiring")
                || content.contains("Surface"),
            "{source_dir} must remain a composition or wiring surface"
        );
    }
}

#[test]
fn cli_and_demo_surfaces_have_specific_fail_closed_composition_guards() {
    assert_impl_file_contains(
        "entrypoints/cli/src/main.rs",
        &[
            "CliCompositionGuard",
            "CliCompositionError",
            "OperatorAuthorizationMissing",
            "CliOwnsDomainDecision",
            "CliDefinesReasonVocabulary",
            "CliDefinesPortTrait",
            "OutOfScopeFeatureAdmitted",
        ],
    );
    assert_impl_file_contains(
        "entrypoints/demo/src/main.rs",
        &[
            "DemoCompositionGuard",
            "DemoCompositionError",
            "DemoOwnsDomainDecision",
            "DemoDefinesReasonVocabulary",
            "DemoDefinesPortTrait",
            "ListenerStartupAsReadiness",
            "DemoAsCloseOrReadyEvidence",
            "UiOrEndUserWorkflowOwnedByDemo",
            "OutOfScopeFeatureAdmitted",
        ],
    );
}
