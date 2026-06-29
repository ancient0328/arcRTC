//! Controlled integration test 用の共通検査ヘルパーです。
//!
//! 実ネットワークを起動せず、composition boundary と統合シナリオ資産を検査します。

use std::fs;
use std::path::{Path, PathBuf};

/// `Kernel` の実装 root です。
pub fn implementation_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("integration-tests crate must live directly under implementation root")
        .to_path_buf()
}

pub fn read_impl(relative_path: &str) -> String {
    let path = implementation_root().join(relative_path);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

pub fn assert_impl_file_contains(relative_path: &str, markers: &[&str]) {
    let content = read_impl(relative_path);
    for marker in markers {
        assert!(
            content.contains(marker),
            "{relative_path} must contain marker `{marker}`"
        );
    }
}

pub fn assert_not_contains(label: &str, content: &str, forbidden: &[&str]) {
    for marker in forbidden {
        assert!(
            !content.contains(marker),
            "{label} must not contain forbidden marker `{marker}`"
        );
    }
}

pub fn file_exists(relative_path: &str) -> bool {
    Path::new(&implementation_root().join(relative_path)).exists()
}
