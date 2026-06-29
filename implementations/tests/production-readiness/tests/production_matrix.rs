//! production readiness matrix 境界を検査します。

use arcrtc_implementation_evidence::{
    validate_readiness_evidence_record, ImplementationCommandClass, ImplementationEnvironmentClass,
    ImplementationEvidenceReason, ImplementationEvidenceRecord, ImplementationLayer,
    ImplementationNonClaimScope, ImplementationPlane, ReadinessAdmissionState, ReadinessClaim,
    ReadinessEvidenceRecord, ReadinessValidationContext, IMPLEMENTATIONS_COMMAND_ROOT,
};

fn base() -> ImplementationEvidenceRecord {
    ImplementationEvidenceRecord {
        correlation_id: "production-readiness-matrix".to_owned(),
        command: "cargo test --manifest-path tests/production-readiness/Cargo.toml".to_owned(),
        working_directory: IMPLEMENTATIONS_COMMAND_ROOT.to_owned(),
        target_package: Some("arcrtc-implementation-production-readiness-tests".to_owned()),
        target_scope: "tests/production-readiness".to_owned(),
        command_class: ImplementationCommandClass::ProductionReadiness,
        implementation_layer: ImplementationLayer::Readiness,
        target_plane: ImplementationPlane::Deployment,
        environment_class: ImplementationEnvironmentClass::ProductionDeferred,
        toolchain_runtime_version: "rustc 1.96".to_owned(),
        input_fixture_or_workload: Some("production-readiness-matrix".to_owned()),
        expected_outcome: "readiness extension validates".to_owned(),
        actual_outcome: "readiness extension validates".to_owned(),
        exit_status: Some(0),
        kernel_reason: None,
        implementation_reason: ImplementationEvidenceReason::ImplementationOk,
        non_claim_scope: vec![
            ImplementationNonClaimScope::ProductionReadinessNotClaimed,
            ImplementationNonClaimScope::LiveReadinessNotClaimed,
            ImplementationNonClaimScope::KernelCompletionNotClaimed,
            ImplementationNonClaimScope::KernelFreezeNotClaimed,
        ],
        rerun_condition: "rerun when readiness matrix changes".to_owned(),
    }
}

fn record(gate_id: &str, required_ref: &str) -> ReadinessEvidenceRecord {
    let mut record = ReadinessEvidenceRecord {
        base: base(),
        readiness_claim: ReadinessClaim::ProductionReadiness,
        readiness_gate_id: gate_id.to_owned(),
        readiness_adr_ref:
            "READINESS_CLAIM_BOUNDARY"
                .to_owned(),
        readiness_canonical_ref: "READINESS_MATRIX"
            .to_owned(),
        build_evidence_ref: None,
        behavior_test_evidence_ref: None,
        auth_provider_admission_ref: None,
        persistence_provider_admission_ref: None,
        deployment_profile_ref: None,
        monitoring_probe_ref: None,
        rollback_plan_ref: None,
        security_scan_ref: None,
        production_readiness_report_ref: None,
        live_endpoint_evidence_ref: None,
        public_traversal_evidence_ref: None,
        rollback_drain_execution_ref: None,
        shutdown_drain_evidence_ref: None,
        restore_evidence_ref: None,
        closed_gate_report_ref: None,
    };
    match required_ref {
        "build_evidence_ref" => record.build_evidence_ref = Some("report:build".to_owned()),
        "behavior_test_evidence_ref" => {
            record.behavior_test_evidence_ref = Some("report:behavior".to_owned());
        }
        "auth_provider_admission_ref" => {
            record.auth_provider_admission_ref =
                Some("canonical:production-auth-provider-admission".to_owned());
        }
        "persistence_provider_admission_ref" => {
            record.persistence_provider_admission_ref =
                Some("canonical:production-persistence-provider-admission".to_owned());
        }
        "deployment_profile_ref" => {
            record.deployment_profile_ref = Some("profile:deployment".to_owned());
        }
        "monitoring_probe_ref" => {
            record.monitoring_probe_ref = Some("report:monitoring".to_owned())
        }
        "rollback_plan_ref" => record.rollback_plan_ref = Some("report:rollback".to_owned()),
        "security_scan_ref" => record.security_scan_ref = Some("report:security".to_owned()),
        "closed_gate_report_ref" => {
            record.closed_gate_report_ref = Some("report:closed-gate".to_owned());
        }
        _ => unreachable!("unknown production readiness ref"),
    }
    record
}

fn admitted_context() -> ReadinessValidationContext {
    ReadinessValidationContext {
        auth_provider_authority: ReadinessAdmissionState::Admitted,
        persistence_provider_authority: ReadinessAdmissionState::Admitted,
        live_endpoint_authority: ReadinessAdmissionState::Absent,
        production_readiness_report: ReadinessAdmissionState::Absent,
        public_traversal_authority: ReadinessAdmissionState::Absent,
    }
}

#[test]
fn kpi_production_readiness_matrix_requires_prd_001_through_prd_009() {
    for (gate_id, required_ref) in [
        ("PRD-001", "build_evidence_ref"),
        ("PRD-002", "behavior_test_evidence_ref"),
        ("PRD-003", "auth_provider_admission_ref"),
        ("PRD-004", "persistence_provider_admission_ref"),
        ("PRD-005", "deployment_profile_ref"),
        ("PRD-006", "monitoring_probe_ref"),
        ("PRD-007", "rollback_plan_ref"),
        ("PRD-008", "security_scan_ref"),
        ("PRD-009", "closed_gate_report_ref"),
    ] {
        assert_eq!(
            validate_readiness_evidence_record(&record(gate_id, required_ref), &admitted_context()),
            Ok(()),
            "{gate_id}"
        );
    }
}
