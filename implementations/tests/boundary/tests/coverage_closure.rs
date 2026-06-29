//! Five-item coverage closure tests for Kernel freeze consumption proof surfaces.

use arcrtc_implementation_evidence::{
    validate_evidence_record, EvidenceValidationError, ImplementationCommandClass,
    ImplementationEnvironmentClass, ImplementationEvidenceReason, ImplementationEvidenceRecord,
    ImplementationLayer, ImplementationNonClaimScope, ImplementationPlane,
    IMPLEMENTATIONS_COMMAND_ROOT, IMPLEMENTATIONS_EVIDENCE_ROOT, IMPLEMENTATIONS_TARGET_ROOT,
};

fn record(command_class: ImplementationCommandClass) -> ImplementationEvidenceRecord {
    ImplementationEvidenceRecord {
        correlation_id: "boundary-coverage-closure".to_owned(),
        command: "cargo test --manifest-path tests/boundary/Cargo.toml".to_owned(),
        working_directory: IMPLEMENTATIONS_COMMAND_ROOT.to_owned(),
        target_package: match command_class {
            ImplementationCommandClass::Build
            | ImplementationCommandClass::Test
            | ImplementationCommandClass::Benchmark
            | ImplementationCommandClass::RealDevice
            | ImplementationCommandClass::ProductionReadiness
            | ImplementationCommandClass::LiveReadiness => {
                Some("arcrtc-implementation-boundary-tests".to_owned())
            }
            _ => None,
        },
        target_scope: "tests/boundary".to_owned(),
        command_class,
        implementation_layer: ImplementationLayer::Readiness,
        target_plane: ImplementationPlane::Ops,
        environment_class: ImplementationEnvironmentClass::LocalDocsOnly,
        toolchain_runtime_version: "rustc 1.96".to_owned(),
        input_fixture_or_workload: match command_class {
            ImplementationCommandClass::Test
            | ImplementationCommandClass::Benchmark
            | ImplementationCommandClass::RealDevice
            | ImplementationCommandClass::ProductionReadiness
            | ImplementationCommandClass::LiveReadiness => Some("boundary-fixture".to_owned()),
            _ => None,
        },
        expected_outcome: "boundary evidence validation branches close".to_owned(),
        actual_outcome: "boundary evidence validation branches close".to_owned(),
        exit_status: Some(0),
        kernel_reason: None,
        implementation_reason: ImplementationEvidenceReason::ImplementationOk,
        non_claim_scope: required_scope(command_class),
        rerun_condition: "rerun when boundary coverage closure changes".to_owned(),
    }
}

fn required_scope(command_class: ImplementationCommandClass) -> Vec<ImplementationNonClaimScope> {
    match command_class {
        ImplementationCommandClass::Format => {
            vec![ImplementationNonClaimScope::CommandTargetSuccessNotClaimed]
        }
        ImplementationCommandClass::Build => vec![
            ImplementationNonClaimScope::BehaviorCorrectnessNotClaimed,
            ImplementationNonClaimScope::ProductionReadinessNotClaimed,
            ImplementationNonClaimScope::LiveReadinessNotClaimed,
        ],
        ImplementationCommandClass::Test => vec![
            ImplementationNonClaimScope::ProductionReadinessNotClaimed,
            ImplementationNonClaimScope::LiveReadinessNotClaimed,
        ],
        ImplementationCommandClass::Benchmark => vec![
            ImplementationNonClaimScope::BenchmarkThresholdNotClaimed,
            ImplementationNonClaimScope::ProductionReadinessNotClaimed,
            ImplementationNonClaimScope::LiveReadinessNotClaimed,
        ],
        ImplementationCommandClass::RealDevice => vec![
            ImplementationNonClaimScope::NativeApplicationReadinessNotClaimed,
            ImplementationNonClaimScope::PublicDistributionReadinessNotClaimed,
            ImplementationNonClaimScope::ProductionReadinessNotClaimed,
            ImplementationNonClaimScope::LiveReadinessNotClaimed,
        ],
        ImplementationCommandClass::ProductionReadiness => vec![
            ImplementationNonClaimScope::LiveReadinessNotClaimed,
            ImplementationNonClaimScope::KernelCompletionNotClaimed,
            ImplementationNonClaimScope::KernelFreezeNotClaimed,
        ],
        ImplementationCommandClass::LiveReadiness => vec![
            ImplementationNonClaimScope::KernelCompletionNotClaimed,
            ImplementationNonClaimScope::KernelFreezeNotClaimed,
        ],
    }
}

#[test]
fn boundary_evidence_validation_covers_empty_and_required_field_branches() {
    for (record, expected) in [
        {
            let mut record = record(ImplementationCommandClass::Test);
            record.command.clear();
            (record, EvidenceValidationError::EmptyCommand)
        },
        {
            let mut record = record(ImplementationCommandClass::Test);
            record.working_directory.clear();
            (record, EvidenceValidationError::EmptyWorkingDirectory)
        },
        {
            let mut record = record(ImplementationCommandClass::Test);
            record.target_scope.clear();
            (record, EvidenceValidationError::EmptyTargetScope)
        },
        {
            let mut record = record(ImplementationCommandClass::Test);
            record.toolchain_runtime_version.clear();
            (
                record,
                EvidenceValidationError::EmptyToolchainRuntimeVersion,
            )
        },
        {
            let mut record = record(ImplementationCommandClass::Test);
            record.expected_outcome.clear();
            (record, EvidenceValidationError::EmptyExpectedOutcome)
        },
        {
            let mut record = record(ImplementationCommandClass::Build);
            record.target_package = None;
            (
                record,
                EvidenceValidationError::MissingRequiredTargetPackage,
            )
        },
        {
            let mut record = record(ImplementationCommandClass::Test);
            record.exit_status = None;
            (record, EvidenceValidationError::MissingRequiredExitStatus)
        },
    ] {
        assert_eq!(validate_evidence_record(&record), Err(expected));
    }
}

#[test]
fn boundary_evidence_validation_covers_each_command_class_required_scope() {
    for class in [
        ImplementationCommandClass::Format,
        ImplementationCommandClass::Build,
        ImplementationCommandClass::Test,
        ImplementationCommandClass::Benchmark,
        ImplementationCommandClass::RealDevice,
        ImplementationCommandClass::ProductionReadiness,
        ImplementationCommandClass::LiveReadiness,
    ] {
        assert_eq!(
            validate_evidence_record(&record(class)),
            Ok(()),
            "{class:?}"
        );

        let mut missing_required_scope = record(class);
        missing_required_scope.non_claim_scope =
            vec![ImplementationNonClaimScope::KernelCompletionNotClaimed];
        if required_scope(class).contains(&ImplementationNonClaimScope::KernelCompletionNotClaimed)
        {
            missing_required_scope.non_claim_scope =
                vec![ImplementationNonClaimScope::CommandTargetSuccessNotClaimed];
        }
        assert_eq!(
            validate_evidence_record(&missing_required_scope),
            Err(EvidenceValidationError::MissingRequiredNonClaimScope),
            "{class:?}"
        );
    }
}

#[test]
fn boundary_evidence_validation_covers_secret_markers_without_kernel_mutation() {
    for marker in [
        "secret=abc",
        "token=abc",
        "private_key=abc",
        "-----BEGIN PRIVATE KEY-----",
        "packet_payload=abc",
        "raw_packet=abc",
        "Authorization: Bearer abc",
        "bearer abc",
    ] {
        let mut evidence = record(ImplementationCommandClass::Format);
        evidence.actual_outcome = marker.to_owned();
        assert_eq!(
            validate_evidence_record(&evidence),
            Err(EvidenceValidationError::RawSecretLikeValue),
            "{marker}"
        );
    }
}

#[test]
fn boundary_coverage_keeps_roots_on_implementations_side() {
    assert_eq!(IMPLEMENTATIONS_COMMAND_ROOT, "implementations");
    assert_eq!(IMPLEMENTATIONS_TARGET_ROOT, "implementations/target");
    assert_eq!(
        IMPLEMENTATIONS_EVIDENCE_ROOT,
        "implementations/target/implementation-evidence"
    );
    let kernel_segment = ["/", "Kernel", "/"].concat();
    let absolute_home_prefix = ["/", "Users", "/"].concat();
    assert!(!IMPLEMENTATIONS_COMMAND_ROOT.contains(&kernel_segment));
    assert!(!IMPLEMENTATIONS_TARGET_ROOT.contains(&kernel_segment));
    assert!(!IMPLEMENTATIONS_EVIDENCE_ROOT.contains(&kernel_segment));
    assert!(!IMPLEMENTATIONS_COMMAND_ROOT.contains(&absolute_home_prefix));
    assert!(!IMPLEMENTATIONS_TARGET_ROOT.contains(&absolute_home_prefix));
    assert!(!IMPLEMENTATIONS_EVIDENCE_ROOT.contains(&absolute_home_prefix));
}
