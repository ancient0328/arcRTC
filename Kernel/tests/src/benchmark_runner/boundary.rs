use std::hint::black_box;

use arcrtc_core_command::{CommandType, CommandVersion, TargetSurface};
use arcrtc_core_cross_plane::{
    BindingExpiryBehavior, BindingLifecyclePrecondition, BindingReplayRelation, CrossPlane,
    CrossPlaneBindingClass, CrossPlaneBindingDecision, CrossPlaneBindingOutcome,
    CrossPlaneBindingPolicy, CrossPlaneReference,
};
use arcrtc_core_quality::{
    BackpressureDecisionKind, BackpressureOutcome, ResourceBoundDecision,
    ResourceBoundReferenceSet, REQUIRED_RESOURCE_BOUND_CLOSED_ACTIONS,
};
use arcrtc_core_recovery::{
    CommandRoutingRule, DistributedAuditRelation, DistributedConflictRule,
    DistributedStateAdmission, DistributedStateClass, DistributedStatePolicy,
    FailoverEvidenceShape, FailoverEvidenceShapeInput, OwnerNodeScope, PacketRoutingRule,
    RecoveryRestoreRelation,
};
use arcrtc_core_runtime::{
    CancellationPropagationRule, RuntimeFailureKind, RuntimeTaskClass,
    RuntimeTaskInputReferenceClass, RuntimeTaskLifecycleDecision, RuntimeTaskLifecycleOutcome,
    RuntimeTaskLifecyclePolicy, RuntimeTaskOutputObservation, RuntimeTaskOwningLayer,
    SupervisionScope,
};
use arcrtc_core_security::AuthorizationContextClass;
use arcrtc_core_state::StateFamily;
use arcrtc_core_turn::{TurnCommandKind, TurnFailureKind, TurnMessageClass, TurnModelKind};
use arcrtc_driver_network::{
    DriverCommandConversionInput, DriverIngressPreconditions, ExternalIngressKind,
    SemanticDelegationGuard, TurnWireDecodeInput, TurnWireDecodedAttributes, TurnWireMethodClass,
    TurnWirePreconditions,
};
use arcrtc_entrypoint_internal_control::{
    InternalServiceIdentityMappingGuard, InternalServiceRole, InternalServiceTrustClass,
    ServiceIdentityProofReferenceClass,
};
use arcrtc_entrypoint_topology::TopologyServiceDiscoveryRelationGuard;

use super::support::{
    correlation_id, endpoint_id, mix_bytes, opaque_ref, packet_id, participant_id, startup_run_id,
};

pub(super) fn driver_conversion_workload() -> u64 {
    let preconditions = DriverIngressPreconditions::new(
        ExternalIngressKind::WebSocketMessage,
        true,
        512,
        4_096,
        true,
        true,
        true,
        true,
        true,
        true,
    );
    let semantic_delegation =
        SemanticDelegationGuard::try_new(true, true, false, false).expect("valid guard");
    let subject = ("room-ref", "participant-ref");
    let mut state = 0u64;
    for index in 0..512 {
        let conversion = DriverCommandConversionInput::new(
            preconditions,
            semantic_delegation,
            true,
            Some(arcrtc_core_identity::UntrustedReference::new(format!(
                "corr-driver-{index}"
            ))),
            Some(CommandType::new("send_ice_candidate")),
            Some(CommandVersion::new(1)),
            Some(TargetSurface::Signaling),
            subject,
        )
        .into_core_command_envelope()
        .expect("driver conversion workload uses valid input");
        state = mix_bytes(state, conversion.correlation_id().as_str().as_bytes());
    }
    state
}

pub(super) fn service_discovery_workload() -> u64 {
    let mut state = 0u64;
    for index in 0..2_048 {
        let guard = TopologyServiceDiscoveryRelationGuard::try_new(
            true, true, true, true, true, true, true,
        )
        .expect("service discovery workload keeps helper relation bounded");
        state = mix_bytes(state, format!("{guard:?}:{index}").as_bytes());
    }
    state
}

pub(super) fn failover_diagnostic_workload() -> u64 {
    let mut state = 0u64;
    for index in 0..512 {
        let policy = DistributedStatePolicy::try_new(
            StateFamily::SfuForwardingState,
            DistributedStateClass::AffinityRequiredState,
            OwnerNodeScope::ServiceInstance,
            Some(opaque_ref(format!("owner-{index}"))),
            Some(opaque_ref(format!("affinity-{index}"))),
            CommandRoutingRule::OwnerAffinityRequired,
            PacketRoutingRule::OwnerAffinityRequired,
            RecoveryRestoreRelation::ExplicitRestoreEvidenceRequired,
            DistributedConflictRule::RejectConflictingOwner,
            true,
            DistributedAuditRelation::DistributedStateFailoverDecision,
        )
        .expect("valid distributed state policy");
        let admission = DistributedStateAdmission::admit_initial_v0_2(policy)
            .expect("affinity policy is admitted in initial v0.2");
        let failover = FailoverEvidenceShape::try_new(FailoverEvidenceShapeInput {
            failed_owner_observed: true,
            replacement_owner: Some(opaque_ref(format!("replacement-{index}"))),
            affected_state_family: StateFamily::SfuForwardingState,
            affinity_sticky_routing_updated: true,
            restore_replay_relation: RecoveryRestoreRelation::ExplicitRestoreEvidenceRequired,
            conflict_and_duplicate_handling_defined: true,
            resource_lifetime_revalidated: true,
            audit_continuity_or_close_not_claimed_scope_recorded: true,
        })
        .expect("valid failover evidence shape");
        state = mix_bytes(state, format!("{admission:?}:{failover:?}").as_bytes());
    }
    state
}

pub(super) fn runtime_task_lifecycle_workload() -> u64 {
    let mut state = 0u64;
    for index in 0..512 {
        let policy = RuntimeTaskLifecyclePolicy::try_new(
            RuntimeTaskClass::DriverPacketWorker,
            Some(SupervisionScope::DriverComponent),
            RuntimeTaskOwningLayer::Driver,
            RuntimeTaskInputReferenceClass::OpaqueTaskReference,
            RuntimeTaskOutputObservation::JoinObserved,
            CancellationPropagationRule::ParentScopeEndsThenBoundedJoinOrCancel,
            true,
            true,
        )
        .expect("valid runtime task policy");
        let decision = RuntimeTaskLifecycleDecision::try_new(
            startup_run_id(format!("startup-{index}")),
            Some(correlation_id(format!("runtime-corr-{index}"))),
            policy,
            Some("driver-packet-worker"),
            RuntimeTaskLifecycleOutcome::Joined,
            None,
        )
        .expect("joined outcome has no failure reason");
        state = mix_bytes(state, decision.audit_event_type().as_bytes());
        state = mix_bytes(
            state,
            format!("{:?}", RuntimeFailureKind::RuntimeTaskQueueBoundExceeded).as_bytes(),
        );
    }
    state
}

pub(super) fn internal_service_trust_workload() -> u64 {
    let mut state = 0u64;
    for index in 0..512 {
        let guard = InternalServiceIdentityMappingGuard::try_new(
            InternalServiceTrustClass::SignedServiceTokenIdentity,
            ServiceIdentityProofReferenceClass::SignedServiceTokenReference,
            InternalServiceRole::Signaling,
            InternalServiceRole::Sfu,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
        )
        .expect("valid internal service identity mapping");
        state = mix_bytes(state, format!("{guard:?}:{index}").as_bytes());
    }
    state
}

pub(super) fn cross_plane_binding_workload() -> u64 {
    let mut state = 0u64;
    for index in 0..512 {
        let policy = CrossPlaneBindingPolicy::new(
            CrossPlaneBindingClass::SfuEndpointBinding,
            CrossPlane::Signaling,
            CrossPlane::Sfu,
            CrossPlaneReference::Participant(participant_id(format!("participant-{index}"))),
            CrossPlaneReference::Endpoint(endpoint_id(format!("endpoint-{index}"))),
            Some(AuthorizationContextClass::PublicationPolicyContext),
            BindingLifecyclePrecondition::ParticipantJoined,
            BindingLifecyclePrecondition::EndpointAdmitted,
            BindingExpiryBehavior::RejectNewTargetPlaneAction,
            BindingReplayRelation::ConflictRejected,
        );
        let decision = CrossPlaneBindingDecision::new(
            correlation_id(format!("cross-corr-{index}")),
            policy,
            CrossPlaneBindingOutcome::Accepted,
            None,
        );
        state = mix_bytes(state, decision.audit_event_type().as_bytes());
    }
    state
}

pub(super) fn turn_wire_workload() -> u64 {
    let mut state = 0u64;
    for index in 0..512 {
        let attributes = TurnWireDecodedAttributes::new(
            Some(arcrtc_core_identity::UntrustedReference::new(format!(
                "tx-{index}"
            ))),
            Some(arcrtc_core_identity::UntrustedReference::new(format!(
                "alloc-{index}"
            ))),
            Some(arcrtc_core_identity::UntrustedReference::new(format!(
                "perm-{index}"
            ))),
            Some(arcrtc_core_identity::UntrustedReference::new(format!(
                "chan-{index}"
            ))),
            Some(arcrtc_core_identity::UntrustedReference::new(format!(
                "cred-{index}"
            ))),
            Some(format!(
                "198.51.100.{}:{}",
                (index % 200) + 1,
                40_000 + index
            )),
            Some(600),
            Some(arcrtc_core_identity::UntrustedReference::new(format!(
                "packet-{index}"
            ))),
        );
        let command = TurnWireDecodeInput::new(
            TurnWirePreconditions::new(256, 1_500, true, true, true, false),
            Some(TurnWireMethodClass::SendIndication),
            attributes,
        )
        .into_core_turn_command()
        .expect("valid TURN wire decode workload");
        state = mix_bytes(
            state,
            format!(
                "{:?}:{:?}:{:?}",
                command.kind(),
                TurnCommandKind::RelayData,
                TurnFailureKind::RelayDenied
            )
            .as_bytes(),
        );
    }
    black_box(TurnModelKind::Message(TurnMessageClass::Indication));
    state
}

pub(super) fn resource_saturation_workload() -> u64 {
    let mut state = 0u64;
    for index in 0..2_048 {
        let action = REQUIRED_RESOURCE_BOUND_CLOSED_ACTIONS
            [index % REQUIRED_RESOURCE_BOUND_CLOSED_ACTIONS.len()];
        state = mix_bytes(
            state,
            format!(
                "{:?}:{:?}:{:?}:{}",
                action.resource(),
                action.action(),
                action.outcome(),
                BackpressureOutcome::Dropped.code()
            )
            .as_bytes(),
        );
        state = mix_bytes(
            state,
            format!("{:?}", BackpressureDecisionKind::DropPacket).as_bytes(),
        );
    }
    let packet_decision = ResourceBoundDecision::try_new(
        REQUIRED_RESOURCE_BOUND_CLOSED_ACTIONS
            .iter()
            .find(|action| action.resource().resource_name() == "SFU packet cache")
            .copied()
            .expect("SFU packet cache action exists"),
        ResourceBoundReferenceSet::packet(packet_id("resource-packet")),
    )
    .expect("packet scoped resource decision has packet reference");
    black_box(packet_decision);
    state
}
