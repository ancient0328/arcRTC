//! product source set と dev-docs の依存分離を機械的に確認します。
//!
//! 検査対象は core / drivers / entrypoints / regulated の product source です。
//! 開発記録ディレクトリを product source の仕様根拠へ混ぜないため、指示文だけに頼らず
//! ビルド／テストで機械的に検出します。

use arcrtc_roadmap_tests::{assert_not_contains, read_impl_rust_source_set};

#[test]
fn dd1_core_source_does_not_reference_dev_docs() {
    let content = read_impl_rust_source_set("core");
    assert_not_contains("core", &content, &["dev-docs", "dev_docs"]);
}

#[test]
fn dd1_drivers_source_does_not_reference_dev_docs() {
    let content = read_impl_rust_source_set("drivers");
    assert_not_contains("drivers", &content, &["dev-docs", "dev_docs"]);
}

#[test]
fn dd1_entrypoints_source_does_not_reference_dev_docs() {
    let content = read_impl_rust_source_set("entrypoints");
    assert_not_contains("entrypoints", &content, &["dev-docs", "dev_docs"]);
}

#[test]
fn dd1_regulated_source_does_not_reference_dev_docs() {
    let content = read_impl_rust_source_set("regulated");
    assert_not_contains("regulated", &content, &["dev-docs", "dev_docs"]);
}
