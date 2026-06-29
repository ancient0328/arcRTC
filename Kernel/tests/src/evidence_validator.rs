//! evidence 採用条件を fail-closed で検査する validator です。

use crate::evidence_model::{ClosedReasonCode, EvidenceReportDraft, ReportStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceValidationFailure {
    pub code: ClosedReasonCode,
    pub field: &'static str,
}

pub fn validate_report_for_adoption(
    report: &EvidenceReportDraft,
) -> Result<(), Vec<EvidenceValidationFailure>> {
    let mut failures = Vec::new();
    require(report.title.is_some(), "title", &mut failures);
    require(report.date_time.is_some(), "date_time", &mut failures);
    require(
        report.correlation_id.is_some(),
        "correlation_id",
        &mut failures,
    );
    require(report.command.is_some(), "command", &mut failures);
    require(
        report.working_directory.is_some(),
        "working_directory",
        &mut failures,
    );
    require(
        report.target_package.is_some(),
        "target_package",
        &mut failures,
    );
    require(
        report.evidence_class.is_some(),
        "evidence_class",
        &mut failures,
    );
    require(
        report.input_fixture.is_some(),
        "input_fixture",
        &mut failures,
    );
    require(
        report.fixture_source.is_some(),
        "fixture_source",
        &mut failures,
    );
    require(
        report.expected_outcome.is_some(),
        "expected_outcome",
        &mut failures,
    );
    require(
        report.actual_outcome.is_some(),
        "actual_outcome",
        &mut failures,
    );
    require(
        report.coverage_classification.is_some(),
        "coverage_classification",
        &mut failures,
    );
    require(
        !report.semantic_obligations.is_empty(),
        "semantic_obligations",
        &mut failures,
    );
    require(
        report.rerun_condition.is_some(),
        "rerun_condition",
        &mut failures,
    );
    require(
        report.close_not_claimed_scope.is_some(),
        "close_not_claimed_scope",
        &mut failures,
    );

    if report.status != ReportStatus::Adopted {
        failures.push(EvidenceValidationFailure {
            code: ClosedReasonCode::DiagnosticOnlyAdoption,
            field: "status",
        });
    }

    if report.reason_code == Some(ClosedReasonCode::UnknownReason) {
        failures.push(EvidenceValidationFailure {
            code: ClosedReasonCode::UnknownReason,
            field: "reason_code",
        });
    }

    for (present, field) in [
        (report.has_raw_secret, "raw_secret"),
        (report.has_raw_token, "raw_token"),
        (report.has_raw_packet_payload, "raw_packet_payload"),
        (report.has_unredacted_sdp_ice, "unredacted_sdp_ice"),
        (
            report.has_regulated_or_personal_data,
            "regulated_or_personal_data",
        ),
    ] {
        if present {
            failures.push(EvidenceValidationFailure {
                code: ClosedReasonCode::SensitiveMaterialPresent,
                field,
            });
        }
    }

    if report.uses_v01_as_proof {
        failures.push(EvidenceValidationFailure {
            code: ClosedReasonCode::V01ProofSubstitution,
            field: "v0_1_proof",
        });
    }

    if report.claims_benchmark_correctness {
        failures.push(EvidenceValidationFailure {
            code: ClosedReasonCode::BenchmarkCorrectnessSubstitution,
            field: "benchmark_correctness",
        });
    }

    if report.mixes_evidence_class {
        failures.push(EvidenceValidationFailure {
            code: ClosedReasonCode::EvidenceClassMixed,
            field: "evidence_class",
        });
    }

    if report.has_hidden_test_double {
        failures.push(EvidenceValidationFailure {
            code: ClosedReasonCode::HiddenTestDouble,
            field: "test_double_class",
        });
    }

    if report.has_unknown_fixture_source {
        failures.push(EvidenceValidationFailure {
            code: ClosedReasonCode::UnknownFixtureSource,
            field: "fixture_source",
        });
    }

    for (active, code, field) in [
        (
            report.claims_code_coverage_as_semantic_coverage,
            ClosedReasonCode::CodeCoverageSubstitution,
            "coverage_classification",
        ),
        (
            report.claims_ci_pass_as_completion,
            ClosedReasonCode::CiPassCompletionSubstitution,
            "command",
        ),
        (
            report.claims_implementation_checklist_as_completion,
            ClosedReasonCode::ImplementationChecklistCompletionSubstitution,
            "actual_outcome",
        ),
        (
            report.claims_non_goal_as_completion,
            ClosedReasonCode::NonGoalCompletionClaim,
            "close_not_claimed_scope",
        ),
        (
            report.proves_core_semantics_from_non_core_evidence,
            ClosedReasonCode::CoreSemanticSubstitution,
            "evidence_class",
        ),
        (
            report.proves_concrete_driver_from_fake_only,
            ClosedReasonCode::FakeOnlyConcreteDriverProof,
            "fixture_source",
        ),
    ] {
        if active {
            failures.push(EvidenceValidationFailure { code, field });
        }
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures)
    }
}

fn require(condition: bool, field: &'static str, failures: &mut Vec<EvidenceValidationFailure>) {
    if !condition {
        failures.push(EvidenceValidationFailure {
            code: ClosedReasonCode::MissingRequiredField,
            field,
        });
    }
}

pub fn assert_rejected_with(report: EvidenceReportDraft, expected: ClosedReasonCode) {
    let failures = validate_report_for_adoption(&report).expect_err("report must be rejected");
    assert!(
        failures.iter().any(|failure| failure.code == expected),
        "expected rejection {expected:?}, got {failures:?}"
    );
}
