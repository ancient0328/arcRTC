//! distro dependency admission と workspace 境界を検査します。

use std::{fs, path::PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("distro root must exist")
}

fn read(path: &str) -> String {
    fs::read_to_string(root().join(path)).expect("source file must be readable")
}

#[test]
fn distro_is_not_a_kernel_workspace_member() {
    let kernel_workspace = fs::read_to_string(root().join("../Kernel/Cargo.toml"))
        .expect("Kernel workspace manifest must be readable");
    assert!(
        !kernel_workspace.contains("distro"),
        "Kernel workspace must not include distro members"
    );
}

#[test]
fn workspace_members_exclude_tests_and_profile_data() {
    let manifest = read("Cargo.toml");
    assert!(!manifest.contains("\"tests/"));
    assert!(!manifest.contains("reference-distro/deployment-profiles"));
}

#[test]
fn product_plane_manifests_depend_only_on_reference_output() {
    for path in [
        "product-distro/signaling/Cargo.toml",
        "product-distro/turn/Cargo.toml",
        "product-distro/sfu/Cargo.toml",
    ] {
        let body = read(path);
        assert!(body.contains("arcrtc-reference-output"));
        for forbidden in [
            "arcrtc-reference-signaling",
            "arcrtc-reference-turn",
            "arcrtc-reference-sfu",
            "arcrtc-reference-composition",
            "arcrtc-reference-ops",
        ] {
            assert!(
                !body.contains(forbidden),
                "{path} must not depend on {forbidden}"
            );
        }
    }
}

#[test]
fn distro_packages_do_not_admit_criterion_dependency() {
    for entry in collect_cargo_toml(root()) {
        let path = entry.strip_prefix(root()).expect("path must be under root");
        let path_string = path.to_string_lossy();
        if path_string.starts_with("tests/benchmark") {
            continue;
        }
        let body = fs::read_to_string(&entry).expect("manifest must be readable");
        assert!(
            !body.contains("criterion"),
            "{} must not contain criterion",
            path_string
        );
    }
}

#[test]
fn kernel_production_implementability_kernel_imports_are_allow_listed() {
    for manifest_path in collect_cargo_toml(root()) {
        let relative_path = manifest_path
            .strip_prefix(root())
            .expect("manifest must be under distro root")
            .to_string_lossy()
            .into_owned();
        if relative_path == "Cargo.toml" || relative_path.starts_with("tests/") {
            continue;
        }

        let body = fs::read_to_string(&manifest_path).expect("manifest must be readable");
        let package_name = package_name_from_manifest(&body)
            .unwrap_or_else(|| panic!("{relative_path} must define a package name"));

        for (dependency_name, dependency_path) in kernel_local_path_dependencies(&body) {
            assert!(
                allowed_kernel_dependencies(package_name).contains(&dependency_name),
                "{package_name} must not admit unlisted Kernel dependency {dependency_name}"
            );
            assert_eq!(
                dependency_path,
                expected_kernel_path(dependency_name),
                "{package_name} uses wrong Kernel local path for {dependency_name}"
            );
        }
    }
}

#[test]
fn distro_does_not_vendor_kernel_source_tree() {
    for forbidden_root in [
        "Kernel",
        "core",
        "drivers",
        "entrypoints",
        "sdk",
        "regulated",
    ] {
        assert!(
            !root().join(forbidden_root).exists(),
            "distro must not vendor Kernel source root {forbidden_root}"
        );
    }

    for manifest_path in collect_cargo_toml(root()) {
        let relative_path = manifest_path
            .strip_prefix(root())
            .expect("manifest must be under distro root")
            .to_string_lossy()
            .into_owned();
        if relative_path == "Cargo.toml" {
            continue;
        }
        let body = fs::read_to_string(&manifest_path).expect("manifest must be readable");
        let package_name = package_name_from_manifest(&body)
            .unwrap_or_else(|| panic!("{relative_path} must define a package name"));

        // Kernel source copy は local path dependency とは別の境界侵食なので、
        // distro 側で arcrtc-core-* package を再定義しないことを検査します。
        assert!(
            !package_name.starts_with("arcrtc-core-"),
            "{relative_path} must not define Kernel package {package_name}"
        );
    }
}

fn collect_cargo_toml(path: PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect(path, &mut files);
    files
}

fn collect(path: PathBuf, files: &mut Vec<PathBuf>) {
    if path.is_file() {
        if path.file_name().is_some_and(|name| name == "Cargo.toml") {
            files.push(path);
        }
        return;
    }
    if path.ends_with("target") {
        return;
    }
    for entry in fs::read_dir(path).expect("directory must be readable") {
        collect(
            entry.expect("directory entry must be readable").path(),
            files,
        );
    }
}

fn package_name_from_manifest(body: &str) -> Option<&str> {
    body.lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix("name = \"")?.strip_suffix('"'))
}

fn kernel_local_path_dependencies(body: &str) -> Vec<(&str, &str)> {
    body.lines()
        .map(str::trim)
        .filter(|line| line.contains("../../../Kernel/"))
        .map(|line| {
            let (dependency_name, rest) = line
                .split_once('=')
                .expect("dependency line must contain assignment");
            let path_value = rest
                .split("path = \"")
                .nth(1)
                .and_then(|value| value.split('"').next())
                .expect("Kernel dependency line must contain local path");
            (dependency_name.trim(), path_value)
        })
        .collect()
}

fn allowed_kernel_dependencies(package_name: &str) -> &'static [&'static str] {
    match package_name {
        "arcrtc-reference-output" => &[
            "arcrtc-core-signaling",
            "arcrtc-core-turn",
            "arcrtc-core-sfu",
            "arcrtc-core-identity",
        ],
        "arcrtc-reference-signaling" => &[
            "arcrtc-core-signaling",
            "arcrtc-core-command",
            "arcrtc-core-identity",
            "arcrtc-core-reason",
            "arcrtc-core-ports",
            "arcrtc-core-protocol",
            "arcrtc-core-configuration",
        ],
        "arcrtc-reference-turn" => &[
            "arcrtc-core-turn",
            "arcrtc-core-identity",
            "arcrtc-core-transport",
            "arcrtc-core-security",
            "arcrtc-core-time",
            "arcrtc-core-reason",
            "arcrtc-core-ports",
            "arcrtc-core-configuration",
        ],
        "arcrtc-reference-sfu" => &[
            "arcrtc-core-sfu",
            "arcrtc-core-identity",
            "arcrtc-core-transport",
            "arcrtc-core-quality",
            "arcrtc-core-state",
            "arcrtc-core-reason",
            "arcrtc-core-ports",
            "arcrtc-core-configuration",
        ],
        "arcrtc-reference-composition" => &[
            "arcrtc-core-cross-plane",
            "arcrtc-core-operation",
            "arcrtc-core-runtime",
            "arcrtc-core-reason",
            "arcrtc-core-identity",
        ],
        "arcrtc-reference-ops" => &[
            "arcrtc-core-command",
            "arcrtc-core-operation",
            "arcrtc-core-runtime",
            "arcrtc-core-identity",
        ],
        "arcrtc-product-signaling" | "arcrtc-product-turn" | "arcrtc-product-sfu" => {
            // Product plane は reference output を入力にするため、
            // Kernel communication semantic ではなく共通 identity だけを直接許可します。
            &["arcrtc-core-identity"]
        }
        "arcrtc-product-policy" => &[
            "arcrtc-core-command",
            "arcrtc-core-identity",
            "arcrtc-core-reason",
        ],
        "arcrtc-product-persistence-topology" => &["arcrtc-core-state", "arcrtc-core-reason"],
        "arcrtc-product-deployment" => &[
            "arcrtc-core-configuration",
            "arcrtc-core-operation",
            "arcrtc-core-runtime",
            "arcrtc-core-identity",
        ],
        "arcrtc-product-monitoring" => &[
            "arcrtc-core-audit",
            "arcrtc-core-quality",
            "arcrtc-core-identity",
            "arcrtc-core-reason",
        ],
        "arcrtc-product-rollback" => &[
            "arcrtc-core-recovery",
            "arcrtc-core-operation",
            "arcrtc-core-identity",
            "arcrtc-core-reason",
        ],
        _ => &[],
    }
}

fn expected_kernel_path(dependency_name: &str) -> String {
    format!(
        "../../../Kernel/core/{}",
        dependency_name
            .strip_prefix("arcrtc-core-")
            .expect("Kernel dependency must use arcrtc-core prefix")
    )
}
