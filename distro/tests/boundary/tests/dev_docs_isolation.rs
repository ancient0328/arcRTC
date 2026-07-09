//! implementation source が開発文書をruntime境界へ持ち込まないことを検査します。

use std::{fs, path::PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("distro root must exist")
}

#[test]
fn implementation_source_does_not_reference_development_documents() {
    let mut scanned = 0usize;
    let mut findings = Vec::new();
    for file in implementation_sources() {
        scanned += 1;
        let relative_path = file
            .strip_prefix(root())
            .expect("implementation source must be under distro root")
            .to_string_lossy()
            .into_owned();
        let body = fs::read_to_string(&file).expect("implementation source must be readable");
        for forbidden in ["dev-docs", "90-reports", "ROADMAP", "Roadmap"] {
            if body.contains(forbidden) {
                findings.push(format!("{relative_path} contains {forbidden}"));
            }
        }
    }
    println!("dev docs isolation scanned_files={scanned}");
    assert!(scanned > 0, "implementation source scan must not be empty");
    assert!(findings.is_empty(), "{}", findings.join("\n"));
}

fn implementation_sources() -> Vec<PathBuf> {
    let mut files = Vec::new();
    for path in ["reference-distro", "product-distro", "distro-support"] {
        collect_rust_sources(root().join(path), &mut files);
    }
    files
}

fn collect_rust_sources(path: PathBuf, files: &mut Vec<PathBuf>) {
    if path.ends_with("target") {
        return;
    }
    if path.is_file() {
        if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
        return;
    }
    for entry in fs::read_dir(path).expect("implementation directory must be readable") {
        collect_rust_sources(
            entry.expect("directory entry must be readable").path(),
            files,
        );
    }
}
