//! Kernel substitute 語彙が implementation source に残存しないことを検査します。

use std::{fs, path::PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("implementations root must exist")
}

#[test]
fn kernel_production_implementability_rejects_kernel_substitute_vocabulary() {
    let forbidden_path_stems = [
        "core".to_owned(),
        ["kernel", "_", "core"].concat(),
        ["kernel", "_", "port"].concat(),
        ["port", "_", "authority"].concat(),
        ["reason", "_", "catalog"].concat(),
        ["semantic", "_", "authority"].concat(),
        ["kernel", "_", "state"].concat(),
        ["kernel", "_", "runtime"].concat(),
        ["kernel", "_", "freeze"].concat(),
    ];
    let mut findings = Vec::new();

    for file in implementation_rust_files() {
        let stem = file
            .file_stem()
            .and_then(|name| name.to_str())
            .expect("Rust source file stem must be UTF-8");
        if forbidden_path_stems
            .iter()
            .any(|forbidden| stem == forbidden)
        {
            findings.push(format!(
                "{} uses forbidden file stem {stem}",
                file.display()
            ));
        }
    }

    assert!(findings.is_empty(), "{}", findings.join("\n"));
}

#[test]
fn kernel_production_implementability_rejects_unlisted_kernel_substitute_symbols() {
    let forbidden_type_suffixes = [
        ["Reason", "Catalog"].concat(),
        ["Port", "Authority"].concat(),
        ["Semantic", "Authority"].concat(),
        ["Kernel", "State"].concat(),
        ["Kernel", "Runtime"].concat(),
        ["Kernel", "Freeze"].concat(),
    ];
    let forbidden_function_prefixes = [
        ["own", "_", "kernel", "_"].concat(),
        ["replace", "_", "kernel", "_"].concat(),
        ["shadow", "_", "kernel", "_"].concat(),
        ["duplicate", "_", "core", "_"].concat(),
        ["admit", "_", "kernel", "_", "freeze"].concat(),
    ];
    let mut findings = Vec::new();

    for file in implementation_rust_files() {
        let body = fs::read_to_string(&file).expect("source file must be readable");
        for line in body.lines().map(str::trim) {
            if let Some(type_name) = declared_type_name(line) {
                if forbidden_type_suffixes
                    .iter()
                    .any(|suffix| type_name.ends_with(suffix))
                {
                    findings.push(format!(
                        "{} declares forbidden type symbol {type_name}",
                        file.display()
                    ));
                }
            }
            if let Some(function_name) = declared_function_name(line) {
                if forbidden_function_prefixes
                    .iter()
                    .any(|prefix| function_name.starts_with(prefix))
                {
                    findings.push(format!(
                        "{} declares forbidden function symbol {function_name}",
                        file.display()
                    ));
                }
            }
        }
    }

    assert!(findings.is_empty(), "{}", findings.join("\n"));
}

fn implementation_rust_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    for dir in [
        "implementation-support",
        "reference-implementation",
        "product-implementation",
        "tests",
    ] {
        collect_rs(root().join(dir), &mut files);
    }
    files.sort();
    files
}

fn collect_rs(path: PathBuf, files: &mut Vec<PathBuf>) {
    if path.is_file() {
        if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
        return;
    }
    if path.ends_with("target") {
        return;
    }
    for entry in fs::read_dir(path).expect("source path must be readable") {
        collect_rs(
            entry.expect("directory entry must be readable").path(),
            files,
        );
    }
}

fn declared_type_name(line: &str) -> Option<&str> {
    let line = line.strip_prefix("pub ").unwrap_or(line);
    for keyword in ["struct ", "enum ", "trait "] {
        if let Some(rest) = line.strip_prefix(keyword) {
            return rest
                .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
                .next();
        }
    }
    None
}

fn declared_function_name(line: &str) -> Option<&str> {
    let line = line.strip_prefix("pub ").unwrap_or(line);
    let line = line.strip_prefix("async ").unwrap_or(line);
    let rest = line.strip_prefix("fn ")?;
    rest.split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .next()
}
