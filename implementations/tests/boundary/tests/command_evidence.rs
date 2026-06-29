//! command evidence record の必須 field と非主張 scope を検査します。

use arcrtc_implementation_evidence::{
    validate_evidence_record, EvidenceValidationError, ImplementationCommandClass,
    ImplementationEnvironmentClass, ImplementationEvidenceReason, ImplementationEvidenceRecord,
    ImplementationLayer, ImplementationNonClaimScope, ImplementationPlane,
    IMPLEMENTATIONS_COMMAND_ROOT, IMPLEMENTATIONS_EVIDENCE_ROOT,
};

fn base_record(command_class: ImplementationCommandClass) -> ImplementationEvidenceRecord {
    let (target_package, workload) = match command_class {
        ImplementationCommandClass::Build => (Some("arcrtc-product-monitoring"), None),
        ImplementationCommandClass::Test
        | ImplementationCommandClass::Benchmark
        | ImplementationCommandClass::RealDevice
        | ImplementationCommandClass::ProductionReadiness
        | ImplementationCommandClass::LiveReadiness => (
            Some("arcrtc-implementation-boundary-tests"),
            Some("fixture"),
        ),
        _ => (None, None),
    };
    ImplementationEvidenceRecord {
        correlation_id: "boundary-command-evidence".to_owned(),
        command: "cargo test --manifest-path tests/boundary/Cargo.toml".to_owned(),
        working_directory: IMPLEMENTATIONS_COMMAND_ROOT.to_owned(),
        target_package: target_package.map(str::to_owned),
        target_scope: "tests/boundary".to_owned(),
        command_class,
        implementation_layer: ImplementationLayer::Reference,
        target_plane: ImplementationPlane::Ops,
        environment_class: ImplementationEnvironmentClass::LocalDocsOnly,
        toolchain_runtime_version: "rustc 1.96".to_owned(),
        input_fixture_or_workload: workload.map(str::to_owned),
        expected_outcome: "record validates".to_owned(),
        actual_outcome: "record validates".to_owned(),
        exit_status: Some(0),
        kernel_reason: None,
        implementation_reason: ImplementationEvidenceReason::ImplementationOk,
        non_claim_scope: required_non_claim_scope(command_class),
        rerun_condition: "rerun when evidence schema changes".to_owned(),
    }
}

fn required_non_claim_scope(
    command_class: ImplementationCommandClass,
) -> Vec<ImplementationNonClaimScope> {
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
fn command_evidence_required_fields_are_enforced_by_command_class() {
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
            validate_evidence_record(&base_record(class)),
            Ok(()),
            "{class:?}"
        );
    }
}

#[test]
fn command_evidence_rejects_wrong_working_directory_and_missing_fields() {
    let mut wrong_working_directory = base_record(ImplementationCommandClass::Test);
    wrong_working_directory.working_directory = "Kernel".to_owned();
    assert_eq!(
        validate_evidence_record(&wrong_working_directory),
        Err(EvidenceValidationError::WorkingDirectoryOutsideImplementations)
    );

    let mut missing_workload = base_record(ImplementationCommandClass::Benchmark);
    missing_workload.input_fixture_or_workload = None;
    assert_eq!(
        validate_evidence_record(&missing_workload),
        Err(EvidenceValidationError::MissingRequiredInputFixtureOrWorkload)
    );

    let mut missing_scope = base_record(ImplementationCommandClass::RealDevice);
    missing_scope.non_claim_scope.clear();
    assert_eq!(
        validate_evidence_record(&missing_scope),
        Err(EvidenceValidationError::EmptyNonClaimScope)
    );
}

#[test]
fn command_evidence_root_constants_are_bound_to_implementations_target() {
    assert_eq!(IMPLEMENTATIONS_COMMAND_ROOT, "implementations");
    assert_eq!(
        IMPLEMENTATIONS_EVIDENCE_ROOT,
        "implementations/target/implementation-evidence"
    );
    let absolute_home_prefix = ["/", "Users", "/"].concat();
    let kernel_segment = ["/", "Kernel", "/"].concat();
    assert!(!IMPLEMENTATIONS_COMMAND_ROOT.contains(&absolute_home_prefix));
    assert!(!IMPLEMENTATIONS_EVIDENCE_ROOT.contains(&absolute_home_prefix));
    assert!(!IMPLEMENTATIONS_COMMAND_ROOT.contains(&kernel_segment));
    assert!(!IMPLEMENTATIONS_EVIDENCE_ROOT.contains(&kernel_segment));
}
