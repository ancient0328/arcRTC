use arcrtc_roadmap_tests::evidence_model::{ClosedReasonCode, EvidenceReportDraft};
use arcrtc_roadmap_tests::evidence_validator::{
    assert_rejected_with, validate_report_for_adoption,
};
use arcrtc_roadmap_tests::semantic_obligation_manifest::{ids_are_unique, CE_REPORTS, CI_GATES};

#[test]
fn ce9_required_ce_reports_and_ci_gates_are_closed_sets() {
    assert_eq!(CE_REPORTS.len(), 11);
    assert!(CE_REPORTS.iter().any(|item| item.id == "CE0"));
    assert!(CE_REPORTS.iter().any(|item| item.id == "CE10"));
    assert!(ids_are_unique(CE_REPORTS));

    assert_eq!(CI_GATES.len(), 31);
    assert!(CI_GATES.iter().any(|item| item.id == "CI-001"));
    assert!(CI_GATES.iter().any(|item| item.id == "CI-031"));
    assert!(ids_are_unique(CI_GATES));
}

#[test]
fn ce9_adoption_requires_rerunnable_evidence_shape() {
    let report = EvidenceReportDraft::adopted_minimal();
    validate_report_for_adoption(&report).expect("adoptable report must pass CE9 shape");

    let mut no_command = EvidenceReportDraft::adopted_minimal();
    no_command.command = None;
    assert_rejected_with(no_command, ClosedReasonCode::MissingRequiredField);

    let mut no_rerun = EvidenceReportDraft::adopted_minimal();
    no_rerun.rerun_condition = None;
    assert_rejected_with(no_rerun, ClosedReasonCode::MissingRequiredField);

    let mut no_obligation = EvidenceReportDraft::adopted_minimal();
    no_obligation.semantic_obligations.clear();
    assert_rejected_with(no_obligation, ClosedReasonCode::MissingRequiredField);
}

#[test]
fn ce9_rejects_evidence_class_mixing() {
    let mut report = EvidenceReportDraft::adopted_minimal();
    report.mixes_evidence_class = true;
    assert_rejected_with(report, ClosedReasonCode::EvidenceClassMixed);
}

#[test]
fn ce9_rejects_coverage_ci_and_checklist_completion_substitution() {
    let mut code_coverage = EvidenceReportDraft::adopted_minimal();
    code_coverage.claims_code_coverage_as_semantic_coverage = true;
    assert_rejected_with(code_coverage, ClosedReasonCode::CodeCoverageSubstitution);

    let mut ci_pass = EvidenceReportDraft::adopted_minimal();
    ci_pass.claims_ci_pass_as_completion = true;
    assert_rejected_with(ci_pass, ClosedReasonCode::CiPassCompletionSubstitution);

    let mut checklist = EvidenceReportDraft::adopted_minimal();
    checklist.claims_implementation_checklist_as_completion = true;
    assert_rejected_with(
        checklist,
        ClosedReasonCode::ImplementationChecklistCompletionSubstitution,
    );
}
