use arcrtc_roadmap_tests::assert_impl_file_contains;

#[test]
fn fake_register_declares_allowed_and_forbidden_substitution_boundaries() {
    assert_impl_file_contains(
        "tests/fakes/FAKE_TEST_DOUBLE_REGISTER.md",
        &[
            "T0.2",
            "fake/test-double ownership",
            "allowed substitution surfaces",
            "forbidden semantic substitution surfaces",
            "report disclosure fields",
            "must not own core semantics",
        ],
    );
}

#[test]
fn fixture_register_declares_provenance_identity_and_mutation_policy() {
    assert_impl_file_contains(
        "tests/fixtures/FIXTURE_SCENARIO_REGISTER.md",
        &[
            "T0.3",
            "fixture provenance",
            "scenario identity",
            "mutation policy",
            "golden-data admission",
            "report linkage fields",
        ],
    );
}
