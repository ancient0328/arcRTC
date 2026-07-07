use arcrtc_core_identity::{OpaqueReference, ReferenceAuthority};
use arcrtc_core_recovery::{
    decide_failure_continuity, ComponentFailureClass, ReadinessTransitionDecision,
    SplitBrainAdmissionDecision, SystemWideFailureContinuityDecision,
    SystemWideFailureContinuityInput,
};

fn accepted_ref(value: &str) -> OpaqueReference {
    OpaqueReference::accept(value, ReferenceAuthority::CorePolicy)
        .expect("system-wide P1 test references must be accepted")
}

fn input_for(
    failure_class: ComponentFailureClass,
    readiness_ref: Option<OpaqueReference>,
) -> SystemWideFailureContinuityInput {
    SystemWideFailureContinuityInput::new(
        Some(accepted_ref("component-ref")),
        failure_class,
        readiness_ref,
    )
}

#[test]
fn component_failure_matrix_maps_closed_continuity_outcomes() {
    let readiness_ref = Some(accepted_ref("readiness-ref"));
    let matrix = [
        (
            ComponentFailureClass::ProcessExit,
            SplitBrainAdmissionDecision::Accepted,
            ReadinessTransitionDecision::Draining,
            SystemWideFailureContinuityDecision::Degrades,
        ),
        (
            ComponentFailureClass::DriverFailure,
            SplitBrainAdmissionDecision::Accepted,
            ReadinessTransitionDecision::Ready,
            SystemWideFailureContinuityDecision::Continues,
        ),
        (
            ComponentFailureClass::StateCorruption,
            SplitBrainAdmissionDecision::Rejected,
            ReadinessTransitionDecision::NotReady,
            SystemWideFailureContinuityDecision::Rejects,
        ),
        (
            ComponentFailureClass::NetworkPartition,
            SplitBrainAdmissionDecision::Rejected,
            ReadinessTransitionDecision::NotReady,
            SystemWideFailureContinuityDecision::Rejects,
        ),
    ];

    for (failure_class, split_brain, readiness, continuity) in matrix {
        assert_eq!(failure_class.split_brain_admission(), split_brain);
        assert_eq!(
            failure_class.readiness_transition(readiness_ref.as_ref()),
            readiness
        );
        assert_eq!(
            decide_failure_continuity(input_for(failure_class, readiness_ref.clone())),
            continuity
        );
    }
}

#[test]
fn split_brain_risk_rejects_continuity_without_fail_open_path() {
    for failure_class in [
        ComponentFailureClass::StateCorruption,
        ComponentFailureClass::NetworkPartition,
    ] {
        // split-brain risk は readiness ref の有無に関係なく fail-closed に拒否されます。
        assert_eq!(
            failure_class.split_brain_admission(),
            SplitBrainAdmissionDecision::Rejected
        );
        assert_eq!(
            decide_failure_continuity(input_for(
                failure_class,
                Some(accepted_ref("readiness-ref"))
            )),
            SystemWideFailureContinuityDecision::Rejects
        );
        assert_eq!(
            decide_failure_continuity(input_for(failure_class, None)),
            SystemWideFailureContinuityDecision::Rejects
        );
    }
}

#[test]
fn readiness_transition_rejects_missing_component_and_degrades_driver_without_readiness() {
    let missing_component = SystemWideFailureContinuityInput::new(
        None,
        ComponentFailureClass::DriverFailure,
        Some(accepted_ref("readiness-ref")),
    );
    let driver_without_readiness = input_for(ComponentFailureClass::DriverFailure, None);

    assert_eq!(
        decide_failure_continuity(missing_component),
        SystemWideFailureContinuityDecision::Rejects
    );
    assert_eq!(
        ComponentFailureClass::DriverFailure.readiness_transition(None),
        ReadinessTransitionDecision::NotReady
    );
    assert_eq!(
        decide_failure_continuity(driver_without_readiness),
        SystemWideFailureContinuityDecision::Degrades
    );
}
