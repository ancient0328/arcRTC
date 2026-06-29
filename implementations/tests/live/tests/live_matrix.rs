//! live readiness matrix 境界を検査します。

use arcrtc_implementation_evidence::{
    validate_readiness_evidence_record, ImplementationCommandClass, ImplementationEnvironmentClass,
    ImplementationEvidenceReason, ImplementationEvidenceRecord, ImplementationLayer,
    ImplementationNonClaimScope, ImplementationPlane, ReadinessAdmissionState, ReadinessClaim,
    ReadinessEvidenceRecord, ReadinessValidationContext, IMPLEMENTATIONS_COMMAND_ROOT,
};

fn base() -> ImplementationEvidenceRecord {
    ImplementationEvidenceRecord {
        correlation_id: "live-readiness-matrix".to_owned(),
        command: "cargo test --manifest-path tests/live/Cargo.toml".to_owned(),
        working_directory: IMPLEMENTATIONS_COMMAND_ROOT.to_owned(),
        target_package: Some("arcrtc-implementation-live-tests".to_owned()),
        target_scope: "tests/live".to_owned(),
        command_class: ImplementationCommandClass::LiveReadiness,
        implementation_layer: ImplementationLayer::Readiness,
        target_plane: ImplementationPlane::Deployment,
        environment_class: ImplementationEnvironmentClass::LiveDeferred,
        toolchain_runtime_version: "rustc 1.96".to_owned(),
        input_fixture_or_workload: Some("live-readiness-matrix".to_owned()),
        expected_outcome: "live readiness extension validates".to_owned(),
        actual_outcome: "live readiness extension validates".to_owned(),
        exit_status: Some(0),
        kernel_reason: None,
        implementation_reason: ImplementationEvidenceReason::ImplementationOk,
        non_claim_scope: vec![
            ImplementationNonClaimScope::LiveReadinessNotClaimed,
            ImplementationNonClaimScope::KernelCompletionNotClaimed,
            ImplementationNonClaimScope::KernelFreezeNotClaimed,
        ],
        rerun_condition: "rerun when live readiness matrix changes".to_owned(),
    }
}

fn record(gate_id: &str, required_ref: &str) -> ReadinessEvidenceRecord {
    let mut record = ReadinessEvidenceRecord {
        base: base(),
        readiness_claim: ReadinessClaim::LiveReadiness,
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
        "production_readiness_report_ref" => {
            record.production_readiness_report_ref = Some("report:production".to_owned());
        }
        "live_endpoint_evidence_ref" => {
            record.live_endpoint_evidence_ref = Some("report:endpoint".to_owned());
        }
        "public_traversal_evidence_ref" => {
            record.public_traversal_evidence_ref = Some("report:traversal".to_owned());
        }
        "monitoring_probe_ref" => {
            record.monitoring_probe_ref = Some("report:monitoring".to_owned())
        }
        "rollback_drain_execution_ref" => {
            record.rollback_drain_execution_ref = Some("report:rollback-drain".to_owned());
        }
        "shutdown_drain_evidence_ref" => {
            record.shutdown_drain_evidence_ref = Some("report:shutdown".to_owned());
        }
        "restore_evidence_ref" => record.restore_evidence_ref = Some("report:restore".to_owned()),
        "closed_gate_report_ref" => {
            record.closed_gate_report_ref = Some("CLOSED_GATE_REPORT".to_owned());
        }
        _ => unreachable!("unknown live readiness ref"),
    }
    record
}

fn admitted_context() -> ReadinessValidationContext {
    ReadinessValidationContext {
        auth_provider_authority: ReadinessAdmissionState::Admitted,
        persistence_provider_authority: ReadinessAdmissionState::Admitted,
        live_endpoint_authority: ReadinessAdmissionState::Admitted,
        production_readiness_report: ReadinessAdmissionState::Admitted,
        public_traversal_authority: ReadinessAdmissionState::Admitted,
    }
}

#[test]
fn kpi_live_readiness_matrix_requires_live_001_through_live_008() {
    for (gate_id, required_ref) in [
        ("LIVE-001", "production_readiness_report_ref"),
        ("LIVE-002", "live_endpoint_evidence_ref"),
        ("LIVE-003", "public_traversal_evidence_ref"),
        ("LIVE-004", "monitoring_probe_ref"),
        ("LIVE-005", "rollback_drain_execution_ref"),
        ("LIVE-006", "shutdown_drain_evidence_ref"),
        ("LIVE-007", "restore_evidence_ref"),
        ("LIVE-008", "closed_gate_report_ref"),
    ] {
        assert_eq!(
            validate_readiness_evidence_record(&record(gate_id, required_ref), &admitted_context()),
            Ok(()),
            "{gate_id}"
        );
    }
}
