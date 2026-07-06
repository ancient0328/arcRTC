use arcrtc_roadmap_tests::{assert_not_contains, read_impl_rust_source_set};

#[test]
fn grade_claim_words_are_absent_from_runtime_source_sets() {
    let forbidden = [
        "production-ready",
        "production ready",
        "defense-grade",
        "defense grade",
        "enterprise-grade",
        "enterprise grade",
    ];

    for relative_dir in ["core", "drivers", "entrypoints", "regulated"] {
        let source = read_impl_rust_source_set(relative_dir).to_ascii_lowercase();
        println!("scope={relative_dir} bytes={}", source.len());
        assert_not_contains(relative_dir, &source, &forbidden);
    }
}
