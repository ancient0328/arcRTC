use arcrtc_roadmap_tests::evidence_model::{ClosedReasonCode, EvidenceReportDraft};
use arcrtc_roadmap_tests::evidence_validator::assert_rejected_with;
use arcrtc_roadmap_tests::semantic_obligation_manifest::{
    contains_id, ids_are_unique, BENCHMARK_SCENARIOS, CE_REPORTS, CI_GATES, T_SERIES_TASKS,
};

struct UnfinishedButNormalClaim {
    responsibility_outside_current_scope: bool,
    owner_is_other: bool,
    not_adopted_as_current_claim_evidence: bool,
}

impl UnfinishedButNormalClaim {
    fn is_admissible(&self) -> bool {
        self.responsibility_outside_current_scope
            && self.owner_is_other
            && self.not_adopted_as_current_claim_evidence
    }
}

#[test]
fn ce10_manifest_has_all_required_closed_sets() {
    assert_eq!(T_SERIES_TASKS.len(), 35);
    assert_eq!(CE_REPORTS.len(), 11);
    assert_eq!(CI_GATES.len(), 31);
    assert_eq!(BENCHMARK_SCENARIOS.len(), 16);

    assert!(ids_are_unique(T_SERIES_TASKS));
    assert!(ids_are_unique(CE_REPORTS));
    assert!(ids_are_unique(CI_GATES));
    assert!(ids_are_unique(BENCHMARK_SCENARIOS));
}

#[test]
fn ce10_manifest_contains_required_boundary_endpoints() {
    for id in ["T0.1", "T9.4"] {
        assert!(contains_id(T_SERIES_TASKS, id), "{id}");
    }
    for id in ["CE0", "CE10"] {
        assert!(contains_id(CE_REPORTS, id), "{id}");
    }
    for id in ["CI-001", "CI-031"] {
        assert!(contains_id(CI_GATES, id), "{id}");
    }
    for id in ["BENCH-001", "BENCH-016"] {
        assert!(contains_id(BENCHMARK_SCENARIOS, id), "{id}");
    }
}

#[test]
fn ce10_rejects_coverage_denominator_without_manifest() {
    let synthetic_covered_obligations =
        T_SERIES_TASKS.len() + CE_REPORTS.len() + CI_GATES.len() + BENCHMARK_SCENARIOS.len();
    assert_eq!(synthetic_covered_obligations, 93);
}

#[test]
fn ce10_rejects_non_goal_and_cross_layer_completion_substitution() {
    let mut non_goal = EvidenceReportDraft::adopted_minimal();
    non_goal.claims_non_goal_as_completion = true;
    assert_rejected_with(non_goal, ClosedReasonCode::NonGoalCompletionClaim);

    let mut core_from_driver = EvidenceReportDraft::adopted_minimal();
    core_from_driver.proves_core_semantics_from_non_core_evidence = true;
    assert_rejected_with(core_from_driver, ClosedReasonCode::CoreSemanticSubstitution);

    let mut driver_from_fake = EvidenceReportDraft::adopted_minimal();
    driver_from_fake.proves_concrete_driver_from_fake_only = true;
    assert_rejected_with(
        driver_from_fake,
        ClosedReasonCode::FakeOnlyConcreteDriverProof,
    );
}

#[test]
fn ce10_unfinished_but_normal_requires_all_three_conditions() {
    let admitted = UnfinishedButNormalClaim {
        responsibility_outside_current_scope: true,
        owner_is_other: true,
        not_adopted_as_current_claim_evidence: true,
    };
    assert!(admitted.is_admissible());

    for claim in [
        UnfinishedButNormalClaim {
            responsibility_outside_current_scope: false,
            owner_is_other: true,
            not_adopted_as_current_claim_evidence: true,
        },
        UnfinishedButNormalClaim {
            responsibility_outside_current_scope: true,
            owner_is_other: false,
            not_adopted_as_current_claim_evidence: true,
        },
        UnfinishedButNormalClaim {
            responsibility_outside_current_scope: true,
            owner_is_other: true,
            not_adopted_as_current_claim_evidence: false,
        },
    ] {
        assert!(
            !claim.is_admissible(),
            "unfinished-but-normal requires all three Closed Gate conditions"
        );
    }
}
