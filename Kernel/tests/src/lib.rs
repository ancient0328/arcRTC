//! Kernel test support の共通検査ヘルパーです。
//!
//! production code へ責務を足さず、source shape とruntime behaviorを検査します。

use std::fs;
use std::path::{Path, PathBuf};

pub mod source_graph;

/// `Kernel` の実装 root です。
pub fn implementation_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("tests crate must live directly under implementation root")
        .to_path_buf()
}

/// implementation root 相対の UTF-8 ファイルを読みます。
pub fn read_impl(relative_path: &str) -> String {
    read_file(&implementation_root().join(relative_path))
}

/// 任意 path の UTF-8 ファイルを読みます。
pub fn read_file(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

/// implementation root 相対のファイルが存在し、必要 marker を含むことを検査します。
pub fn assert_impl_file_contains(relative_path: &str, markers: &[&str]) {
    let content = read_impl(relative_path);
    for marker in markers {
        assert!(
            content.contains(marker),
            "{relative_path} must contain marker `{marker}`"
        );
    }
}

/// implementation root 相対 directory 配下の Rust source set を結合して読みます。
///
/// 肥大化した `lib.rs` を責務別ファイルへ分割しても、検査対象が crate 全体からずれないようにします。
pub fn read_impl_rust_source_set(relative_dir: &str) -> String {
    read_impl_source_set_with_extension(relative_dir, "rs")
}

/// implementation root 相対 directory 配下の同一拡張子 source set を結合して読みます。
pub fn read_impl_source_set_with_extension(relative_dir: &str, extension: &str) -> String {
    source_files_with_extension(&implementation_root().join(relative_dir), extension)
        .into_iter()
        .map(|path| {
            let relative = impl_relative(&path);
            format!("\n// source: {relative}\n{}", read_file(&path))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// implementation root 相対 directory 配下の Rust source set に marker が存在することを検査します。
pub fn assert_impl_rust_source_set_contains(relative_dir: &str, markers: &[&str]) {
    let content = read_impl_rust_source_set(relative_dir);
    for marker in markers {
        assert!(
            content.contains(marker),
            "{relative_dir} source set must contain marker `{marker}`"
        );
    }
}

/// 指定名のファイルを再帰的に集めます。依存を増やさず標準ライブラリだけで行います。
pub fn files_named(root: &Path, file_name: &str) -> Vec<PathBuf> {
    let mut found = Vec::new();
    collect_files_named(root, file_name, &mut found);
    found.sort();
    found
}

fn collect_files_named(root: &Path, file_name: &str, found: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to list {}: {error}", root.display()));
    for entry in entries {
        let entry = entry.expect("directory entry must be readable");
        let path = entry.path();
        if path.is_dir() {
            if is_generated_or_build_directory(&path) {
                continue;
            }
            collect_files_named(&path, file_name, found);
        } else if path.file_name().and_then(|name| name.to_str()) == Some(file_name) {
            found.push(path);
        }
    }
}

/// 指定拡張子の source file を再帰的に集めます。
pub fn source_files_with_extension(root: &Path, extension: &str) -> Vec<PathBuf> {
    let mut found = Vec::new();
    collect_source_files_with_extension(root, extension, &mut found);
    found.sort();
    found
}

fn collect_source_files_with_extension(root: &Path, extension: &str, found: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to list {}: {error}", root.display()));
    for entry in entries {
        let entry = entry.expect("directory entry must be readable");
        let path = entry.path();
        if path.is_dir() {
            if is_generated_or_build_directory(&path) {
                continue;
            }
            collect_source_files_with_extension(&path, extension, found);
        } else if path.extension().and_then(|name| name.to_str()) == Some(extension) {
            found.push(path);
        }
    }
}

fn is_generated_or_build_directory(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some("target" | "build" | ".build" | ".gradle" | ".kotlin" | "node_modules")
    )
}

/// implementation root 相対 path へ変換します。
pub fn impl_relative(path: &Path) -> String {
    path.strip_prefix(implementation_root())
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// 指定文字列が含まれないことを検査します。
pub fn assert_not_contains(label: &str, content: &str, forbidden: &[&str]) {
    for marker in forbidden {
        assert!(
            !content.contains(marker),
            "{label} must not contain forbidden marker `{marker}`"
        );
    }
}
