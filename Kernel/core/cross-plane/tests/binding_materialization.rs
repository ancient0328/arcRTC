use arcrtc_core_cross_plane::{
    materialize_cross_plane_binding, CrossPlaneBindingClass, CrossPlaneBindingDecision,
    CrossPlaneBindingFailureKind, CrossPlaneBindingMaterializationInput, CrossPlaneBindingOutcome,
    CrossPlaneReference,
};
use arcrtc_core_identity::{
    AllocationId, ChannelBindId, CredentialRef, EndpointId, OpaqueReference, ParticipantId,
    PermissionId, ReferenceAuthority, RoomId, SessionId, StreamId,
};

fn reference(value: &'static str) -> OpaqueReference {
    OpaqueReference::accept(value, ReferenceAuthority::CorePolicy).expect("test reference is fixed")
}

fn materialized_input(
    binding_class: CrossPlaneBindingClass,
) -> CrossPlaneBindingMaterializationInput {
    match binding_class {
        CrossPlaneBindingClass::NoCrossPlaneBindingRequired => {
            CrossPlaneBindingMaterializationInput::new(
                CrossPlaneReference::NotMaterialized,
                CrossPlaneReference::NotMaterialized,
                binding_class,
                None,
            )
        }
        CrossPlaneBindingClass::SignalingParticipantBinding => {
            CrossPlaneBindingMaterializationInput::new(
                CrossPlaneReference::Room(RoomId::new(reference("room:cross-plane"))),
                CrossPlaneReference::Participant(ParticipantId::new(reference(
                    "participant:cross-plane",
                ))),
                binding_class,
                None,
            )
        }
        CrossPlaneBindingClass::SfuEndpointBinding => CrossPlaneBindingMaterializationInput::new(
            CrossPlaneReference::Participant(ParticipantId::new(reference(
                "participant:sfu-binding",
            ))),
            CrossPlaneReference::Endpoint(EndpointId::new(reference("endpoint:sfu-binding"))),
            binding_class,
            None,
        ),
        CrossPlaneBindingClass::TurnAllocationBinding => {
            CrossPlaneBindingMaterializationInput::new(
                CrossPlaneReference::Credential(CredentialRef::new(reference(
                    "credential:turn-allocation",
                ))),
                CrossPlaneReference::Allocation(AllocationId::new(reference(
                    "allocation:turn-allocation",
                ))),
                binding_class,
                None,
            )
        }
        CrossPlaneBindingClass::TurnPermissionBinding => {
            CrossPlaneBindingMaterializationInput::new(
                CrossPlaneReference::Allocation(AllocationId::new(reference(
                    "allocation:turn-permission",
                ))),
                CrossPlaneReference::Permission(PermissionId::new(reference(
                    "permission:turn-permission",
                ))),
                binding_class,
                None,
            )
        }
        CrossPlaneBindingClass::IceCandidateBinding => CrossPlaneBindingMaterializationInput::new(
            CrossPlaneReference::Participant(ParticipantId::new(reference(
                "participant:ice-binding",
            ))),
            CrossPlaneReference::Session(SessionId::new(reference("session:ice-binding"))),
            binding_class,
            None,
        ),
        CrossPlaneBindingClass::SecureMediaSessionBinding => {
            CrossPlaneBindingMaterializationInput::new(
                CrossPlaneReference::Endpoint(EndpointId::new(reference("endpoint:secure-media"))),
                CrossPlaneReference::Stream(StreamId::new(reference("stream:secure-media"))),
                binding_class,
                None,
            )
        }
        CrossPlaneBindingClass::TestCrossPlaneBinding => {
            CrossPlaneBindingMaterializationInput::new(
                CrossPlaneReference::ChannelBind(ChannelBindId::new(reference(
                    "channel:test-binding",
                ))),
                CrossPlaneReference::Session(SessionId::new(reference("session:test-binding"))),
                binding_class,
                None,
            )
        }
        CrossPlaneBindingClass::ImplicitBindingRequested => {
            CrossPlaneBindingMaterializationInput::new(
                CrossPlaneReference::NotMaterialized,
                CrossPlaneReference::NotMaterialized,
                binding_class,
                None,
            )
        }
    }
}

#[test]
fn assert_all_admitted_binding_classes() {
    for binding_class in [
        CrossPlaneBindingClass::NoCrossPlaneBindingRequired,
        CrossPlaneBindingClass::SignalingParticipantBinding,
        CrossPlaneBindingClass::SfuEndpointBinding,
        CrossPlaneBindingClass::TurnAllocationBinding,
        CrossPlaneBindingClass::TurnPermissionBinding,
        CrossPlaneBindingClass::IceCandidateBinding,
        CrossPlaneBindingClass::SecureMediaSessionBinding,
        CrossPlaneBindingClass::TestCrossPlaneBinding,
    ] {
        let decision = materialize_cross_plane_binding(materialized_input(binding_class));
        match decision {
            CrossPlaneBindingDecision::Materialized(record) => {
                assert_eq!(record.outcome(), CrossPlaneBindingOutcome::Accepted);
                assert_eq!(record.reason(), None);
            }
            CrossPlaneBindingDecision::Rejected(record) => {
                panic!("{binding_class:?} must materialize, got {record:?}");
            }
        }
    }
}

#[test]
fn assert_implicit_binding_rejection() {
    let decision = materialize_cross_plane_binding(materialized_input(
        CrossPlaneBindingClass::ImplicitBindingRequested,
    ));

    match decision {
        CrossPlaneBindingDecision::Rejected(record) => {
            assert_eq!(record.outcome(), CrossPlaneBindingOutcome::Rejected);
            assert_eq!(
                record.reason(),
                Some(CrossPlaneBindingFailureKind::BindingClassNotAdmitted)
            );
        }
        CrossPlaneBindingDecision::Materialized(record) => {
            panic!("implicit binding must be rejected, got {record:?}");
        }
    }
}

#[test]
fn assert_missing_binding_rejection() {
    let decision = materialize_cross_plane_binding(CrossPlaneBindingMaterializationInput::new(
        CrossPlaneReference::Participant(ParticipantId::new(reference(
            "participant:missing-target",
        ))),
        CrossPlaneReference::NotMaterialized,
        CrossPlaneBindingClass::SfuEndpointBinding,
        None,
    ));

    match decision {
        CrossPlaneBindingDecision::Rejected(record) => {
            assert_eq!(record.outcome(), CrossPlaneBindingOutcome::Rejected);
            assert_eq!(
                record.reason(),
                Some(CrossPlaneBindingFailureKind::RequiredBindingAbsent)
            );
        }
        CrossPlaneBindingDecision::Materialized(record) => {
            panic!("missing binding material must be rejected, got {record:?}");
        }
    }
}
