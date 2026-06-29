//! base evidence schema の validation 境界を検査します。

use arcrtc_implementation_evidence::{
    validate_evidence_record, EvidenceValidationError, ImplementationCommandClass,
    ImplementationEnvironmentClass, ImplementationEvidenceReason, ImplementationEvidenceRecord,
    ImplementationLayer, ImplementationNonClaimScope, ImplementationPlane,
    IMPLEMENTATIONS_COMMAND_ROOT,
};

fn valid_record() -> ImplementationEvidenceRecord {
    ImplementationEvidenceRecord {
        correlation_id: "schema-validation".to_owned(),
        command: "cargo test --manifest-path tests/boundary/Cargo.toml".to_owned(),
        working_directory: IMPLEMENTATIONS_COMMAND_ROOT.to_owned(),
        target_package: Some("arcrtc-implementation-boundary-tests".to_owned()),
        target_scope: "tests/boundary".to_owned(),
        command_class: ImplementationCommandClass::Test,
        implementation_layer: ImplementationLayer::Reference,
        target_plane: ImplementationPlane::Ops,
        environment_class: ImplementationEnvironmentClass::LocalDocsOnly,
        toolchain_runtime_version: "rustc 1.96".to_owned(),
        input_fixture_or_workload: Some("schema-fixture".to_owned()),
        expected_outcome: "schema accepted".to_owned(),
        actual_outcome: "schema accepted".to_owned(),
        exit_status: Some(0),
        kernel_reason: None,
        implementation_reason: ImplementationEvidenceReason::ImplementationOk,
        non_claim_scope: vec![
            ImplementationNonClaimScope::ProductionReadinessNotClaimed,
            ImplementationNonClaimScope::LiveReadinessNotClaimed,
        ],
        rerun_condition: "rerun when evidence schema changes".to_owned(),
    }
}

#[test]
fn evidence_schema_accepts_all_required_base_fields() {
    assert_eq!(validate_evidence_record(&valid_record()), Ok(()));
}

#[test]
fn evidence_schema_rejects_empty_required_fields() {
    let mut record = valid_record();
    record.correlation_id.clear();
    assert_eq!(
        validate_evidence_record(&record),
        Err(EvidenceValidationError::EmptyCorrelationId)
    );

    let mut record = valid_record();
    record.actual_outcome.clear();
    assert_eq!(
        validate_evidence_record(&record),
        Err(EvidenceValidationError::EmptyActualOutcome)
    );

    let mut record = valid_record();
    record.rerun_condition.clear();
    assert_eq!(
        validate_evidence_record(&record),
        Err(EvidenceValidationError::EmptyRerunCondition)
    );
}

#[test]
fn evidence_schema_rejects_unknown_style_and_secret_like_values() {
    let mut record = valid_record();
    record.kernel_reason = Some("UNKNOWN".to_owned());
    assert_eq!(
        validate_evidence_record(&record),
        Err(EvidenceValidationError::ForbiddenUnclassifiedReason)
    );

    let mut record = valid_record();
    record.kernel_reason = Some("Other".to_owned());
    assert_eq!(
        validate_evidence_record(&record),
        Err(EvidenceValidationError::ForbiddenUnclassifiedReason)
    );

    let mut record = valid_record();
    record.correlation_id = "token=abc".to_owned();
    assert_eq!(
        validate_evidence_record(&record),
        Err(EvidenceValidationError::RawSecretLikeValue)
    );

    let mut record = valid_record();
    record.input_fixture_or_workload = Some("raw_packet=010203".to_owned());
    assert_eq!(
        validate_evidence_record(&record),
        Err(EvidenceValidationError::RawSecretLikeValue)
    );
}
