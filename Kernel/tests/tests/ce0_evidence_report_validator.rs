use arcrtc_roadmap_tests::evidence_model::{
    ClosedReasonCode, EvidenceReportDraft, ReportStatus, TestDoubleClass,
};
use arcrtc_roadmap_tests::evidence_validator::{
    assert_rejected_with, validate_report_for_adoption,
};

#[test]
fn ce0_accepts_minimal_adoptable_report_shape() {
    let report = EvidenceReportDraft::adopted_minimal();
    validate_report_for_adoption(&report).expect("minimal adopted report shape must validate");
}

#[test]
fn ce0_rejects_missing_required_fields() {
    for mutate in [
        |report: &mut EvidenceReportDraft| report.title = None,
        |report: &mut EvidenceReportDraft| report.date_time = None,
        |report: &mut EvidenceReportDraft| report.correlation_id = None,
        |report: &mut EvidenceReportDraft| report.command = None,
        |report: &mut EvidenceReportDraft| report.working_directory = None,
        |report: &mut EvidenceReportDraft| report.target_package = None,
        |report: &mut EvidenceReportDraft| report.evidence_class = None,
        |report: &mut EvidenceReportDraft| report.input_fixture = None,
        |report: &mut EvidenceReportDraft| report.fixture_source = None,
        |report: &mut EvidenceReportDraft| report.expected_outcome = None,
        |report: &mut EvidenceReportDraft| report.actual_outcome = None,
        |report: &mut EvidenceReportDraft| report.coverage_classification = None,
        |report: &mut EvidenceReportDraft| report.rerun_condition = None,
        |report: &mut EvidenceReportDraft| report.close_not_claimed_scope = None,
    ] {
        let mut report = EvidenceReportDraft::adopted_minimal();
        mutate(&mut report);
        assert_rejected_with(report, ClosedReasonCode::MissingRequiredField);
    }
}

#[test]
fn ce0_rejects_unknown_reason_and_diagnostic_adoption() {
    let mut report = EvidenceReportDraft::adopted_minimal();
    report.reason_code = Some(ClosedReasonCode::UnknownReason);
    assert_rejected_with(report, ClosedReasonCode::UnknownReason);

    let mut diagnostic = EvidenceReportDraft::adopted_minimal();
    diagnostic.status = ReportStatus::DiagnosticOnly;
    assert_rejected_with(diagnostic, ClosedReasonCode::DiagnosticOnlyAdoption);
}

#[test]
fn ce0_rejects_sensitive_material_and_hidden_source_substitution() {
    for mutate in [
        |report: &mut EvidenceReportDraft| report.has_raw_secret = true,
        |report: &mut EvidenceReportDraft| report.has_raw_token = true,
        |report: &mut EvidenceReportDraft| report.has_raw_packet_payload = true,
        |report: &mut EvidenceReportDraft| report.has_unredacted_sdp_ice = true,
        |report: &mut EvidenceReportDraft| report.has_regulated_or_personal_data = true,
    ] {
        let mut report = EvidenceReportDraft::adopted_minimal();
        mutate(&mut report);
        assert_rejected_with(report, ClosedReasonCode::SensitiveMaterialPresent);
    }
}

#[test]
fn ce0_rejects_hidden_test_double_and_unknown_fixture_source() {
    let mut hidden_fake = EvidenceReportDraft::adopted_minimal();
    hidden_fake.test_double_class = Some(TestDoubleClass::Fake);
    hidden_fake.has_hidden_test_double = true;
    assert_rejected_with(hidden_fake, ClosedReasonCode::HiddenTestDouble);

    let mut unknown_source = EvidenceReportDraft::adopted_minimal();
    unknown_source.has_unknown_fixture_source = true;
    assert_rejected_with(unknown_source, ClosedReasonCode::UnknownFixtureSource);
}

#[test]
fn ce0_rejects_v01_and_benchmark_correctness_substitution() {
    let mut v01 = EvidenceReportDraft::adopted_minimal();
    v01.uses_v01_as_proof = true;
    assert_rejected_with(v01, ClosedReasonCode::V01ProofSubstitution);

    let mut benchmark = EvidenceReportDraft::adopted_minimal();
    benchmark.claims_benchmark_correctness = true;
    assert_rejected_with(
        benchmark,
        ClosedReasonCode::BenchmarkCorrectnessSubstitution,
    );
}
