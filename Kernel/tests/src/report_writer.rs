//! CE report の生成形を test で検査するための renderer です。

use crate::evidence_model::EvidenceReportDraft;

pub fn render_report(report: &EvidenceReportDraft) -> String {
    let evidence_class = report
        .evidence_class
        .map(|value| value.as_str())
        .unwrap_or("MISSING");
    let fixture_source = report
        .fixture_source
        .map(|value| value.as_str())
        .unwrap_or("MISSING");
    let coverage_classification = report
        .coverage_classification
        .map(|value| value.as_str())
        .unwrap_or("MISSING");
    let semantic_obligations = if report.semantic_obligations.is_empty() {
        "MISSING".to_string()
    } else {
        report.semantic_obligations.join(", ")
    };
    let reason_code = report
        .reason_code
        .map(|value| value.as_str())
        .unwrap_or("none");

    format!(
        "# {}\n\nDate/Time: `{}`\n\nCorrelation ID: `{}`\n\nCommand: `{}`\n\nWorking directory: `{}`\n\nTarget: `{}`\n\nEvidence class: `{}`\n\nInput fixture: `{}`\n\nFixture source: `{}`\n\nExpected outcome: {}\n\nActual outcome: {}\n\nCoverage classification: `{}`\n\nSemantic obligations: `{}`\n\nRerun condition: {}\n\nClose-not-claimed: {}\n\nStatus: `{}`\n\nClosed reason: `{}`\n",
        report.title.unwrap_or("MISSING"),
        report.date_time.unwrap_or("MISSING"),
        report.correlation_id.unwrap_or("MISSING"),
        report.command.unwrap_or("MISSING"),
        report.working_directory.unwrap_or("MISSING"),
        report.target_package.unwrap_or("MISSING"),
        evidence_class,
        report.input_fixture.unwrap_or("MISSING"),
        fixture_source,
        report.expected_outcome.unwrap_or("MISSING"),
        report.actual_outcome.unwrap_or("MISSING"),
        coverage_classification,
        semantic_obligations,
        report.rerun_condition.unwrap_or("MISSING"),
        report.close_not_claimed_scope.unwrap_or("MISSING"),
        report.status.as_str(),
        reason_code,
    )
}
