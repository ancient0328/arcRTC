use arcrtc_kernel_test_suite::{assert_impl_file_contains, assert_not_contains, read_impl};

#[test]
fn regulated_depends_only_on_admitted_core_references() {
    let manifest = read_impl("regulated/Cargo.toml");
    assert!(manifest.contains("owner_layer = \"regulated\""));
    assert!(manifest.contains("arcrtc-core-identity"));
    assert_not_contains(
        "regulated/Cargo.toml",
        &manifest,
        &[
            "arcrtc-driver-",
            "arcrtc-entrypoint-",
            "../../drivers",
            "../../entrypoints",
            "../../sdk",
        ],
    );
}

#[test]
fn regulated_source_keeps_optional_support_and_prohibited_behavior_markers() {
    assert_impl_file_contains(
        "regulated/src/lib.rs",
        &[
            "RegulatedOptionalSupportClass",
            "RegulatedEnrichmentLifecycleStage",
            "RegulatedDependencyGuard",
            "ProhibitedRegulatedBehavior",
        ],
    );
}
