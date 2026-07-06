//! command evidence record の必須 field と非主張 scope を検査します。

use arcrtc_distro_evidence::{
    validate_evidence_record, EvidenceValidationError, DistroCommandClass,
    DistroEnvironmentClass, DistroEvidenceReason, DistroEvidenceRecord,
    DistroLayer, DistroNonClaimScope, DistroPlane,
    DISTRO_COMMAND_ROOT, DISTRO_EVIDENCE_ROOT,
};

fn base_record(command_class: DistroCommandClass) -> DistroEvidenceRecord {
    let (target_package, workload) = match command_class {
        DistroCommandClass::Build => (Some("arcrtc-product-monitoring"), None),
        DistroCommandClass::Test
        | DistroCommandClass::Benchmark
        | DistroCommandClass::RealDevice
        | DistroCommandClass::ProductionReadiness
        | DistroCommandClass::LiveReadiness => (
            Some("arcrtc-distro-boundary-tests"),
            Some("fixture"),
        ),
        _ => (None, None),
    };
    DistroEvidenceRecord {
        correlation_id: "boundary-command-evidence".to_owned(),
        command: "cargo test --manifest-path tests/boundary/Cargo.toml".to_owned(),
        working_directory: DISTRO_COMMAND_ROOT.to_owned(),
        target_package: target_package.map(str::to_owned),
        target_scope: "tests/boundary".to_owned(),
        command_class,
        distro_layer: DistroLayer::Reference,
        target_plane: DistroPlane::Ops,
        environment_class: DistroEnvironmentClass::LocalDocsOnly,
        toolchain_runtime_version: "rustc 1.96".to_owned(),
        input_fixture_or_workload: workload.map(str::to_owned),
        expected_outcome: "record validates".to_owned(),
        actual_outcome: "record validates".to_owned(),
        exit_status: Some(0),
        kernel_reason: None,
        distro_reason: DistroEvidenceReason::DistroOk,
        non_claim_scope: required_non_claim_scope(command_class),
        rerun_condition: "rerun when evidence schema changes".to_owned(),
    }
}

fn required_non_claim_scope(
    command_class: DistroCommandClass,
) -> Vec<DistroNonClaimScope> {
    match command_class {
        DistroCommandClass::Format => {
            vec![DistroNonClaimScope::CommandTargetSuccessNotClaimed]
        }
        DistroCommandClass::Build => vec![
            DistroNonClaimScope::BehaviorCorrectnessNotClaimed,
            DistroNonClaimScope::ProductionReadinessNotClaimed,
            DistroNonClaimScope::LiveReadinessNotClaimed,
        ],
        DistroCommandClass::Test => vec![
            DistroNonClaimScope::ProductionReadinessNotClaimed,
            DistroNonClaimScope::LiveReadinessNotClaimed,
        ],
        DistroCommandClass::Benchmark => vec![
            DistroNonClaimScope::BenchmarkThresholdNotClaimed,
            DistroNonClaimScope::ProductionReadinessNotClaimed,
            DistroNonClaimScope::LiveReadinessNotClaimed,
        ],
        DistroCommandClass::RealDevice => vec![
            DistroNonClaimScope::NativeApplicationReadinessNotClaimed,
            DistroNonClaimScope::PublicDistributionReadinessNotClaimed,
            DistroNonClaimScope::ProductionReadinessNotClaimed,
            DistroNonClaimScope::LiveReadinessNotClaimed,
        ],
        DistroCommandClass::ProductionReadiness => vec![
            DistroNonClaimScope::LiveReadinessNotClaimed,
            DistroNonClaimScope::KernelCompletionNotClaimed,
            DistroNonClaimScope::KernelFreezeNotClaimed,
        ],
        DistroCommandClass::LiveReadiness => vec![
            DistroNonClaimScope::KernelCompletionNotClaimed,
            DistroNonClaimScope::KernelFreezeNotClaimed,
        ],
    }
}

#[test]
fn command_evidence_required_fields_are_enforced_by_command_class() {
    for class in [
        DistroCommandClass::Format,
        DistroCommandClass::Build,
        DistroCommandClass::Test,
        DistroCommandClass::Benchmark,
        DistroCommandClass::RealDevice,
        DistroCommandClass::ProductionReadiness,
        DistroCommandClass::LiveReadiness,
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
    let mut wrong_working_directory = base_record(DistroCommandClass::Test);
    wrong_working_directory.working_directory = "Kernel".to_owned();
    assert_eq!(
        validate_evidence_record(&wrong_working_directory),
        Err(EvidenceValidationError::WorkingDirectoryOutsideDistro)
    );

    let mut missing_workload = base_record(DistroCommandClass::Benchmark);
    missing_workload.input_fixture_or_workload = None;
    assert_eq!(
        validate_evidence_record(&missing_workload),
        Err(EvidenceValidationError::MissingRequiredInputFixtureOrWorkload)
    );

    let mut missing_scope = base_record(DistroCommandClass::RealDevice);
    missing_scope.non_claim_scope.clear();
    assert_eq!(
        validate_evidence_record(&missing_scope),
        Err(EvidenceValidationError::EmptyNonClaimScope)
    );
}

#[test]
fn command_evidence_root_constants_are_bound_to_distro_target() {
    assert_eq!(DISTRO_COMMAND_ROOT, "distro");
    assert_eq!(
        DISTRO_EVIDENCE_ROOT,
        "distro/target/distro-evidence"
    );
    let absolute_home_prefix = ["/", "Users", "/"].concat();
    let kernel_segment = ["/", "Kernel", "/"].concat();
    assert!(!DISTRO_COMMAND_ROOT.contains(&absolute_home_prefix));
    assert!(!DISTRO_EVIDENCE_ROOT.contains(&absolute_home_prefix));
    assert!(!DISTRO_COMMAND_ROOT.contains(&kernel_segment));
    assert!(!DISTRO_EVIDENCE_ROOT.contains(&kernel_segment));
}
