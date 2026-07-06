use arcrtc_roadmap_tests::{
    assert_not_contains, files_named, impl_relative, implementation_root, read_file,
};

#[test]
fn cargo_manifests_preserve_dependency_direction() {
    let root = implementation_root();
    for manifest in files_named(&root, "Cargo.toml") {
        let relative = impl_relative(&manifest);
        let content = read_file(&manifest);

        if relative.starts_with("core/") {
            assert_not_contains(
                &relative,
                &content,
                &[
                    "arcrtc-driver-",
                    "arcrtc-entrypoint-",
                    "arcrtc-regulated",
                    "../../drivers",
                    "../../entrypoints",
                ],
            );
        }

        if relative.starts_with("drivers/") {
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

        if relative.starts_with("entrypoints/") {
            assert_not_contains(
                &relative,
                &content,
                &["arcrtc-regulated", "../../regulated"],
            );
        }

        if relative.starts_with("regulated/") {
            assert_not_contains(
                &relative,
                &content,
                &[
                    "arcrtc-driver-",
                    "arcrtc-entrypoint-",
                    "../../drivers",
                    "../../entrypoints",
                    "../../sdk",
                ],
            );
        }
    }
}

#[test]
fn package_owner_metadata_is_present_on_rust_package_surfaces() {
    let root = implementation_root();
    for manifest in files_named(&root, "Cargo.toml") {
        let relative = impl_relative(&manifest);
        let content = read_file(&manifest);
        if relative == "Cargo.toml" {
            assert!(content.contains("[workspace.metadata.arcrtc]"));
        } else {
            assert!(content.contains("[package.metadata.arcrtc]"), "{relative}");
            assert!(content.contains("owner_layer"), "{relative}");
            assert!(content.contains("package_role"), "{relative}");
            assert!(
                content.contains("allowed_dependency_direction"),
                "{relative}"
            );
            assert!(content.contains("dependency_class"), "{relative}");
        }
    }
}

fn declared_core_dependency_names(manifest: &str) -> Vec<String> {
    manifest
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            let rest = trimmed.strip_prefix("arcrtc-core-")?;
            let name = rest.split([' ', '=']).next()?;
            Some(format!("arcrtc-core-{name}"))
        })
        .collect()
}

fn allowed_core_dependencies(package_path: &str) -> &'static [&'static str] {
    match package_path {
        "core/audit" => &[
            "arcrtc-core-command",
            "arcrtc-core-identity",
            "arcrtc-core-reason",
        ],
        "core/command" => &["arcrtc-core-identity"],
        "core/configuration" => &[],
        "core/cross-plane" => &["arcrtc-core-identity", "arcrtc-core-security"],
        "core/domain" => &[],
        "core/features" => &["arcrtc-core-identity"],
        "core/identity" => &[],
        "core/operation" => &["arcrtc-core-command", "arcrtc-core-identity"],
        "core/ports" => &[
            "arcrtc-core-identity",
            "arcrtc-core-protocol",
            "arcrtc-core-quality",
            "arcrtc-core-reason",
            "arcrtc-core-state",
        ],
        "core/protocol" => &[
            "arcrtc-core-command",
            "arcrtc-core-identity",
            "arcrtc-core-reason",
        ],
        "core/quality" => &["arcrtc-core-identity"],
        "core/reason" => &[],
        "core/recovery" => &["arcrtc-core-identity", "arcrtc-core-state"],
        "core/runtime" => &["arcrtc-core-identity"],
        "core/security" => &["arcrtc-core-command", "arcrtc-core-identity"],
        "core/sfu" => &["arcrtc-core-command", "arcrtc-core-identity"],
        "core/signaling" => &["arcrtc-core-command", "arcrtc-core-identity"],
        "core/state" => &[],
        "core/time" => &["arcrtc-core-identity"],
        "core/transport" => &["arcrtc-core-identity", "arcrtc-core-reason"],
        "core/turn" => &["arcrtc-core-command", "arcrtc-core-identity"],
        _ => panic!("unmapped core package path: {package_path}"),
    }
}

#[test]
fn core_internal_dependencies_match_semantic_peer_closure() {
    for manifest in files_named(&implementation_root().join("core"), "Cargo.toml") {
        let relative = impl_relative(&manifest);
        let package_path = relative
            .strip_suffix("/Cargo.toml")
            .expect("core manifest must end with Cargo.toml");
        let content = read_file(&manifest);
        let allowed = allowed_core_dependencies(package_path);
        for dependency in declared_core_dependency_names(&content) {
            assert!(
                allowed.contains(&dependency.as_str()),
                "{relative} declares undeclared core peer dependency {dependency}"
            );
        }
    }
}
