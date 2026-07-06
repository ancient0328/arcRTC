//! Five-item coverage closure tests for Kernel freeze consumption proof surfaces.

use arcrtc_distro_evidence::{
    validate_evidence_record, EvidenceValidationError, DistroCommandClass,
    DistroEnvironmentClass, DistroEvidenceReason, DistroEvidenceRecord,
    DistroLayer, DistroNonClaimScope, DistroPlane,
    DISTRO_COMMAND_ROOT, DISTRO_EVIDENCE_ROOT, DISTRO_TARGET_ROOT,
};

fn record(command_class: DistroCommandClass) -> DistroEvidenceRecord {
    DistroEvidenceRecord {
        correlation_id: "boundary-coverage-closure".to_owned(),
        command: "cargo test --manifest-path tests/boundary/Cargo.toml".to_owned(),
        working_directory: DISTRO_COMMAND_ROOT.to_owned(),
        target_package: match command_class {
            DistroCommandClass::Build
            | DistroCommandClass::Test
            | DistroCommandClass::Benchmark
            | DistroCommandClass::RealDevice
            | DistroCommandClass::ProductionReadiness
            | DistroCommandClass::LiveReadiness => {
                Some("arcrtc-distro-boundary-tests".to_owned())
            }
            _ => None,
        },
        target_scope: "tests/boundary".to_owned(),
        command_class,
        distro_layer: DistroLayer::Readiness,
        target_plane: DistroPlane::Ops,
        environment_class: DistroEnvironmentClass::LocalDocsOnly,
        toolchain_runtime_version: "rustc 1.96".to_owned(),
        input_fixture_or_workload: match command_class {
            DistroCommandClass::Test
            | DistroCommandClass::Benchmark
            | DistroCommandClass::RealDevice
            | DistroCommandClass::ProductionReadiness
            | DistroCommandClass::LiveReadiness => Some("boundary-fixture".to_owned()),
            _ => None,
        },
        expected_outcome: "boundary evidence validation branches close".to_owned(),
        actual_outcome: "boundary evidence validation branches close".to_owned(),
        exit_status: Some(0),
        kernel_reason: None,
        distro_reason: DistroEvidenceReason::DistroOk,
        non_claim_scope: required_scope(command_class),
        rerun_condition: "rerun when boundary coverage closure changes".to_owned(),
    }
}

fn required_scope(command_class: DistroCommandClass) -> Vec<DistroNonClaimScope> {
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
fn boundary_evidence_validation_covers_empty_and_required_field_branches() {
    for (record, expected) in [
        {
            let mut record = record(DistroCommandClass::Test);
            record.command.clear();
            (record, EvidenceValidationError::EmptyCommand)
        },
        {
            let mut record = record(DistroCommandClass::Test);
            record.working_directory.clear();
            (record, EvidenceValidationError::EmptyWorkingDirectory)
        },
        {
            let mut record = record(DistroCommandClass::Test);
            record.target_scope.clear();
            (record, EvidenceValidationError::EmptyTargetScope)
        },
        {
            let mut record = record(DistroCommandClass::Test);
            record.toolchain_runtime_version.clear();
            (
                record,
                EvidenceValidationError::EmptyToolchainRuntimeVersion,
            )
        },
        {
            let mut record = record(DistroCommandClass::Test);
            record.expected_outcome.clear();
            (record, EvidenceValidationError::EmptyExpectedOutcome)
        },
        {
            let mut record = record(DistroCommandClass::Build);
            record.target_package = None;
            (
                record,
                EvidenceValidationError::MissingRequiredTargetPackage,
            )
        },
        {
            let mut record = record(DistroCommandClass::Test);
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
        DistroCommandClass::Format,
        DistroCommandClass::Build,
        DistroCommandClass::Test,
        DistroCommandClass::Benchmark,
        DistroCommandClass::RealDevice,
        DistroCommandClass::ProductionReadiness,
        DistroCommandClass::LiveReadiness,
    ] {
        assert_eq!(
            validate_evidence_record(&record(class)),
            Ok(()),
            "{class:?}"
        );

        let mut missing_required_scope = record(class);
        missing_required_scope.non_claim_scope =
            vec![DistroNonClaimScope::KernelCompletionNotClaimed];
        if required_scope(class).contains(&DistroNonClaimScope::KernelCompletionNotClaimed)
        {
            missing_required_scope.non_claim_scope =
                vec![DistroNonClaimScope::CommandTargetSuccessNotClaimed];
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
        let mut evidence = record(DistroCommandClass::Format);
        evidence.actual_outcome = marker.to_owned();
        assert_eq!(
            validate_evidence_record(&evidence),
            Err(EvidenceValidationError::RawSecretLikeValue),
            "{marker}"
        );
    }
}

#[test]
fn boundary_coverage_keeps_roots_on_distro_side() {
    assert_eq!(DISTRO_COMMAND_ROOT, "distro");
    assert_eq!(DISTRO_TARGET_ROOT, "distro/target");
    assert_eq!(
        DISTRO_EVIDENCE_ROOT,
        "distro/target/distro-evidence"
    );
    let kernel_segment = ["/", "Kernel", "/"].concat();
    let absolute_home_prefix = ["/", "Users", "/"].concat();
    assert!(!DISTRO_COMMAND_ROOT.contains(&kernel_segment));
    assert!(!DISTRO_TARGET_ROOT.contains(&kernel_segment));
    assert!(!DISTRO_EVIDENCE_ROOT.contains(&kernel_segment));
    assert!(!DISTRO_COMMAND_ROOT.contains(&absolute_home_prefix));
    assert!(!DISTRO_TARGET_ROOT.contains(&absolute_home_prefix));
    assert!(!DISTRO_EVIDENCE_ROOT.contains(&absolute_home_prefix));
}
