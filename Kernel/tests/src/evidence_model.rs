//! CE0-CE10 証跡 report を test 側で検査するための最小 model です。
//!
//! production code の意味論ではなく、evidence 採用条件だけを表現します。

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EvidenceClass {
    SourceShape,
    UnitTest,
    ContractTest,
    CompositionTest,
    RuntimeInTest,
    PublicContract,
    OptionalSupport,
    BenchmarkMeasurement,
    DocsGovernance,
}

impl EvidenceClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SourceShape => "source-shape",
            Self::UnitTest => "unit-test",
            Self::ContractTest => "contract-test",
            Self::CompositionTest => "composition-test",
            Self::RuntimeInTest => "runtime-in-test",
            Self::PublicContract => "public-contract",
            Self::OptionalSupport => "optional-support",
            Self::BenchmarkMeasurement => "benchmark-measurement",
            Self::DocsGovernance => "docs-governance",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoverageClassification {
    CoveredBySourceShape,
    CoveredByUnitTest,
    CoveredByContractTest,
    CoveredByCompositionTest,
    CoveredByRuntimeInTest,
    CoveredByBenchmarkMeasurement,
    ClosedAsNonGoal,
    RejectedAsOutOfScope,
}

impl CoverageClassification {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CoveredBySourceShape => "covered-by-source-shape",
            Self::CoveredByUnitTest => "covered-by-unit-test",
            Self::CoveredByContractTest => "covered-by-contract-test",
            Self::CoveredByCompositionTest => "covered-by-composition-test",
            Self::CoveredByRuntimeInTest => "covered-by-runtime-in-test",
            Self::CoveredByBenchmarkMeasurement => "covered-by-benchmark-measurement",
            Self::ClosedAsNonGoal => "closed-as-non-goal",
            Self::RejectedAsOutOfScope => "rejected-as-out-of-scope",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReportStatus {
    Adopted,
    NonAdopted,
    DiagnosticOnly,
}

impl ReportStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Adopted => "adopted",
            Self::NonAdopted => "non-adopted",
            Self::DiagnosticOnly => "diagnostic-only",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClosedReasonCode {
    MissingRequiredField,
    UnknownReason,
    SensitiveMaterialPresent,
    HiddenTestDouble,
    UnknownFixtureSource,
    V01ProofSubstitution,
    DiagnosticOnlyAdoption,
    BenchmarkCorrectnessSubstitution,
    EvidenceClassMixed,
    MissingSemanticObligation,
    MissingClosedGate,
    UnimportablePublicTestSurface,
    CommandUnavailable,
    CodeCoverageSubstitution,
    CiPassCompletionSubstitution,
    ImplementationChecklistCompletionSubstitution,
    NonGoalCompletionClaim,
    CoreSemanticSubstitution,
    FakeOnlyConcreteDriverProof,
}

impl ClosedReasonCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingRequiredField => "missing_required_field",
            Self::UnknownReason => "unknown_reason",
            Self::SensitiveMaterialPresent => "sensitive_material_present",
            Self::HiddenTestDouble => "hidden_test_double",
            Self::UnknownFixtureSource => "unknown_fixture_source",
            Self::V01ProofSubstitution => "v0_1_proof_substitution",
            Self::DiagnosticOnlyAdoption => "diagnostic_only_adoption",
            Self::BenchmarkCorrectnessSubstitution => "benchmark_correctness_substitution",
            Self::EvidenceClassMixed => "evidence_class_mixed",
            Self::MissingSemanticObligation => "missing_semantic_obligation",
            Self::MissingClosedGate => "missing_closed_gate",
            Self::UnimportablePublicTestSurface => "unimportable_public_test_surface",
            Self::CommandUnavailable => "command_unavailable",
            Self::CodeCoverageSubstitution => "code_coverage_substitution",
            Self::CiPassCompletionSubstitution => "ci_pass_completion_substitution",
            Self::ImplementationChecklistCompletionSubstitution => {
                "implementation_checklist_completion_substitution"
            }
            Self::NonGoalCompletionClaim => "non_goal_completion_claim",
            Self::CoreSemanticSubstitution => "core_semantic_substitution",
            Self::FakeOnlyConcreteDriverProof => "fake_only_concrete_driver_proof",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FixtureSourceClass {
    Synthetic,
    Generated,
    CapturedRedacted,
    ManuallyAuthored,
    V01DerivedHistoricalInput,
}

impl FixtureSourceClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Synthetic => "synthetic",
            Self::Generated => "generated",
            Self::CapturedRedacted => "captured-redacted",
            Self::ManuallyAuthored => "manually-authored",
            Self::V01DerivedHistoricalInput => "v0.1-derived-historical-input",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TestDoubleClass {
    Fake,
    Stub,
    Mock,
    DeterministicClock,
    DeterministicRng,
    InMemoryPersistence,
    SimulatedNetwork,
}

#[derive(Debug, Clone)]
pub struct EvidenceReportDraft {
    pub title: Option<&'static str>,
    pub date_time: Option<&'static str>,
    pub correlation_id: Option<&'static str>,
    pub command: Option<&'static str>,
    pub working_directory: Option<&'static str>,
    pub target_package: Option<&'static str>,
    pub evidence_class: Option<EvidenceClass>,
    pub input_fixture: Option<&'static str>,
    pub fixture_source: Option<FixtureSourceClass>,
    pub expected_outcome: Option<&'static str>,
    pub actual_outcome: Option<&'static str>,
    pub coverage_classification: Option<CoverageClassification>,
    pub semantic_obligations: Vec<&'static str>,
    pub rerun_condition: Option<&'static str>,
    pub close_not_claimed_scope: Option<&'static str>,
    pub reason_code: Option<ClosedReasonCode>,
    pub status: ReportStatus,
    pub has_raw_secret: bool,
    pub has_raw_token: bool,
    pub has_raw_packet_payload: bool,
    pub has_unredacted_sdp_ice: bool,
    pub has_regulated_or_personal_data: bool,
    pub has_hidden_test_double: bool,
    pub has_unknown_fixture_source: bool,
    pub test_double_class: Option<TestDoubleClass>,
    pub uses_v01_as_proof: bool,
    pub claims_benchmark_correctness: bool,
    pub mixes_evidence_class: bool,
    pub claims_code_coverage_as_semantic_coverage: bool,
    pub claims_ci_pass_as_completion: bool,
    pub claims_implementation_checklist_as_completion: bool,
    pub claims_non_goal_as_completion: bool,
    pub proves_core_semantics_from_non_core_evidence: bool,
    pub proves_concrete_driver_from_fake_only: bool,
}

impl EvidenceReportDraft {
    pub fn adopted_minimal() -> Self {
        Self {
            title: Some("CE report"),
            date_time: Some("2026-06-15 13:51:05 JST"),
            correlation_id: Some("ARCRTC-V02-TEST-EVIDENCE-001"),
            command: Some("cargo test --workspace --all-targets"),
            working_directory: Some("v0.2/Kernel"),
            target_package: Some("arcrtc-roadmap-tests"),
            evidence_class: Some(EvidenceClass::DocsGovernance),
            input_fixture: Some("synthetic-fixture"),
            fixture_source: Some(FixtureSourceClass::Synthetic),
            expected_outcome: Some("validator accepts adopted evidence shape"),
            actual_outcome: Some("validator accepted adopted evidence shape"),
            coverage_classification: Some(CoverageClassification::CoveredByUnitTest),
            semantic_obligations: vec!["OBL-T0.1-001"],
            rerun_condition: Some("same command, same working directory, same source snapshot"),
            close_not_claimed_scope: Some("does not claim v0.2 completion"),
            reason_code: None,
            status: ReportStatus::Adopted,
            has_raw_secret: false,
            has_raw_token: false,
            has_raw_packet_payload: false,
            has_unredacted_sdp_ice: false,
            has_regulated_or_personal_data: false,
            has_hidden_test_double: false,
            has_unknown_fixture_source: false,
            test_double_class: None,
            uses_v01_as_proof: false,
            claims_benchmark_correctness: false,
            mixes_evidence_class: false,
            claims_code_coverage_as_semantic_coverage: false,
            claims_ci_pass_as_completion: false,
            claims_implementation_checklist_as_completion: false,
            claims_non_goal_as_completion: false,
            proves_core_semantics_from_non_core_evidence: false,
            proves_concrete_driver_from_fake_only: false,
        }
    }
}
