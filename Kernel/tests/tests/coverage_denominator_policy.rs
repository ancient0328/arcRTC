const POLICY: &str = include_str!("../../tools/coverage/coverage-denominator-policy.toml");

#[test]
fn coverage_policy_declares_denominator_scope_and_unit() {
    assert!(POLICY.contains("[denominator]"));
    assert!(POLICY.contains("policy_id = \"coverage-denominator\""));
    assert!(POLICY.contains(
        "scope = [\"kernel-rust-workspace\", \"sdk-typescript\", \"sdk-android\", \"sdk-ios\"]"
    ));
    assert!(POLICY.contains("unit = \"assertion\""));
}

#[test]
fn coverage_policy_declares_assertion_bearing_classification_closed_set() {
    assert!(POLICY.contains("[assertion_classification]"));
    assert!(POLICY.contains("allowed_classification = [\"assertion-bearing\", \"structural-only\", \"excluded-with-reason\"]"));
    assert!(POLICY.contains("default_classification = \"assertion-bearing\""));
    assert!(POLICY.contains("closed_set = true"));
}

#[test]
fn coverage_policy_declares_exclusion_policy_without_unknown_reason() {
    assert!(POLICY.contains("[exclusion_policy]"));
    assert!(POLICY.contains("allowed_exclusion_reasons = [\"generated-source\", \"non-executable-schema\", \"external-tool-wrapper\", \"documentation-only\"]"));
    assert!(POLICY.contains("reason_required = true"));
    assert!(POLICY.contains("unknown_reason_allowed = false"));
}

#[test]
fn coverage_policy_declares_core_workspace_floor_and_command_id() {
    assert!(POLICY.contains("[floor]"));
    assert!(POLICY.contains("line_floor_percent = 90"));
    assert!(POLICY.contains("branch_floor_percent = 85"));
    assert!(POLICY.contains("assertion_floor_percent = 100"));

    assert!(POLICY.contains("[command]"));
    assert!(POLICY.contains("coverage_command_id = \"coverage-denominator-check\""));
}
