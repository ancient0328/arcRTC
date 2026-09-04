use arcrtc_core_command::{
    AuditProjectionRequirement, CommandEnvelope, CommandType, CommandVersion, DecisionReason,
    PortIntent, StateTransitionSummary, TargetSurface, UseCaseDecision, UseCaseDecisionInput,
    UseCaseOutcome,
};
use arcrtc_core_cross_plane::{
    BindingExpiryBehavior, BindingLifecyclePrecondition, BindingReplayRelation, CrossPlane,
    CrossPlaneBindingClass, CrossPlaneBindingDecision, CrossPlaneBindingFailureKind,
    CrossPlaneBindingOutcome, CrossPlaneBindingPolicy, CrossPlaneReference,
};
use arcrtc_core_identity::{
    ChannelBindId, CorrelationId, EndpointId, OpaqueReference, PacketId, ParticipantId,
    ReferenceAuthority, RoomId, RouteId, SessionId, StreamId,
};
use arcrtc_core_reason::CatalogedReasonRef;
use arcrtc_core_sfu::{SfuContractItem, SfuModelKind, SfuReferenceSet};
use arcrtc_core_signaling::{
    SignalingCommand, SignalingCommandKind, SignalingEvent, SignalingEventKind, SignalingSubject,
};
use arcrtc_core_turn::{
    CorePeerAddress, TurnCommand, TurnCommandKind, TurnReferenceSet, TurnRequestedLifetimeSeconds,
    TurnTransactionId, TURN_LIFECYCLE_RULES,
};
use arcrtc_driver_network::{
    DriverCommandConversionInput, DriverIngressPreconditions, ExternalIngressKind,
    SemanticDelegationGuard,
};

fn reference(value: &str) -> OpaqueReference {
    OpaqueReference::accept(value, ReferenceAuthority::CorePolicy).expect("reference must be valid")
}

fn correlation(value: &str) -> CorrelationId {
    CorrelationId::new(reference(value))
}

fn valid_preconditions() -> DriverIngressPreconditions {
    DriverIngressPreconditions::new(
        ExternalIngressKind::WebSocketMessage,
        true,
        8,
        1024,
        true,
        true,
        true,
        true,
        true,
        true,
    )
}

#[test]
fn driver_to_signaling_controlled_flow_keeps_core_decision_authority() {
    let envelope = DriverCommandConversionInput::new(
        valid_preconditions(),
        SemanticDelegationGuard::try_new(true, true, false, false)
            .expect("driver delegates semantic decision"),
        true,
        Some(arcrtc_core_identity::UntrustedReference::new(
            "corr-controlled-flow",
        )),
        Some(CommandType::new("join_room")),
        Some(CommandVersion::new(1)),
        Some(TargetSurface::Signaling),
        "room-reference",
    )
    .into_core_command_envelope()
    .expect("driver may only produce core-owned command envelope");

    let room = RoomId::new(reference("room-controlled-flow"));
    let subject = SignalingSubject::new(room.clone(), None);
    let command = SignalingCommand::new(
        CommandEnvelope::new(
            envelope.correlation_id().clone(),
            envelope.command_type(),
            envelope.version(),
            envelope.target_surface(),
            subject.clone(),
        ),
        SignalingCommandKind::JoinRoom,
        (),
    );
    assert_eq!(command.kind(), SignalingCommandKind::JoinRoom);
    assert_eq!(
        command.envelope().target_surface(),
        TargetSurface::Signaling
    );

    let event = SignalingEvent::new(
        envelope.correlation_id().clone(),
        SignalingEventKind::Rejected,
        subject,
        (),
    );
    assert_eq!(event.kind(), SignalingEventKind::Rejected);
}

#[test]
fn sfu_turn_and_cross_plane_flow_uses_references_not_raw_payloads() {
    let session = SessionId::new(reference("session-controlled-flow"));
    let sfu_refs = SfuReferenceSet::new(
        session,
        Some(EndpointId::new(reference("endpoint-controlled-flow"))),
        Some(StreamId::new(reference("stream-controlled-flow"))),
        Some(RouteId::new(reference("route-controlled-flow"))),
        Some(PacketId::new(reference("packet-controlled-flow"))),
    );
    let packet_view = SfuContractItem::new(SfuModelKind::BorrowedPacketAbstractView, sfu_refs, ());
    assert_eq!(
        packet_view.model_kind(),
        SfuModelKind::BorrowedPacketAbstractView
    );

    let turn_command = TurnCommand::try_new(
        TurnCommandKind::CreatePermission,
        TurnTransactionId::new(reference("turn-tx-controlled-flow")),
        TurnReferenceSet::new(None, None, None, None),
        Some(CorePeerAddress::new("192.0.2.10:3478").expect("valid peer address")),
        Some(TurnRequestedLifetimeSeconds::try_new(60).expect("valid lifetime")),
        None,
    )
    .expect("CreatePermission flow has peer address");
    assert_eq!(turn_command.kind(), TurnCommandKind::CreatePermission);
    assert_eq!(
        TurnCommandKind::CreatePermission.decision_kind(),
        arcrtc_core_turn::TurnDecisionKind::Permission
    );

    let policy = CrossPlaneBindingPolicy::new(
        CrossPlaneBindingClass::SfuEndpointBinding,
        CrossPlane::Signaling,
        CrossPlane::Sfu,
        CrossPlaneReference::Participant(ParticipantId::new(reference(
            "participant-controlled-flow",
        ))),
        CrossPlaneReference::Endpoint(EndpointId::new(reference("endpoint-controlled-flow-bind"))),
        None,
        BindingLifecyclePrecondition::ParticipantJoined,
        BindingLifecyclePrecondition::EndpointAdmitted,
        BindingExpiryBehavior::RejectNewTargetPlaneAction,
        BindingReplayRelation::PriorAcceptedBindingObservable,
    );
    let decision = CrossPlaneBindingDecision::new(
        correlation("corr-controlled-flow-binding-decision"),
        policy,
        CrossPlaneBindingOutcome::Accepted,
        None,
    );
    assert_eq!(decision.audit_event_type(), "cross_plane_binding_decision");
}

#[test]
fn flow_result_cannot_turn_driver_status_into_domain_reason() {
    let reason = CatalogedReasonRef::from_code("participant_not_admitted")
        .expect("cataloged SFU reason must exist");
    let decision = UseCaseDecision::new(UseCaseDecisionInput {
        correlation_id: correlation("corr-controlled-flow-result"),
        command_type: CommandType::new("route_selection"),
        target_surface: TargetSurface::Sfu,
        outcome: UseCaseOutcome::Rejected,
        reason: DecisionReason::Cataloged(reason),
        state_transition: StateTransitionSummary::NoStateChange,
        port_intents: vec![PortIntent::new("observe-driver-status", TargetSurface::Sfu)],
        audit_projection: AuditProjectionRequirement::Required,
    })
    .expect("core-owned non-success decision carries cataloged reason");
    assert_eq!(decision.target_surface(), TargetSurface::Sfu);
    assert!(matches!(decision.reason(), DecisionReason::Cataloged(_)));
}

#[test]
fn turn_lifecycle_trace_rejects_missing_channel_and_packet_references() {
    let transaction = TurnTransactionId::new(reference("turn-tx-controlled-flow-lifecycle"));
    let peer = Some(CorePeerAddress::new("192.0.2.20:3478").expect("valid peer"));
    let lifetime = Some(TurnRequestedLifetimeSeconds::try_new(60).expect("valid lifetime"));

    let allocate = TurnCommand::try_new(
        TurnCommandKind::Allocate,
        transaction.clone(),
        TurnReferenceSet::new(None, None, None, None),
        None,
        lifetime,
        None,
    )
    .expect("Allocate does not require peer/channel/packet reference");
    assert_eq!(allocate.kind(), TurnCommandKind::Allocate);

    let create_permission = TurnCommand::try_new(
        TurnCommandKind::CreatePermission,
        transaction.clone(),
        TurnReferenceSet::new(None, None, None, None),
        peer.clone(),
        lifetime,
        None,
    )
    .expect("CreatePermission requires peer address");
    assert_eq!(create_permission.kind(), TurnCommandKind::CreatePermission);

    assert_eq!(
        TurnCommand::try_new(
            TurnCommandKind::ChannelBind,
            transaction.clone(),
            TurnReferenceSet::new(None, None, None, None),
            peer.clone(),
            lifetime,
            None,
        ),
        Err(arcrtc_core_turn::TurnContractError::MissingChannelBindReference)
    );

    let channel_bound = TurnCommand::try_new(
        TurnCommandKind::ChannelBind,
        transaction.clone(),
        TurnReferenceSet::new(
            None,
            None,
            Some(ChannelBindId::new(reference(
                "channel-bind-controlled-flow",
            ))),
            None,
        ),
        peer.clone(),
        lifetime,
        None,
    )
    .expect("ChannelBind requires peer address and channel bind reference");
    assert_eq!(channel_bound.kind(), TurnCommandKind::ChannelBind);

    assert_eq!(
        TurnCommand::try_new(
            TurnCommandKind::RelayData,
            transaction.clone(),
            TurnReferenceSet::new(None, None, None, None),
            peer.clone(),
            lifetime,
            None,
        ),
        Err(arcrtc_core_turn::TurnContractError::MissingRelayPacketReference)
    );

    let relay = TurnCommand::try_new(
        TurnCommandKind::RelayData,
        transaction,
        TurnReferenceSet::new(None, None, None, None),
        peer,
        lifetime,
        Some(PacketId::new(reference("packet-controlled-flow-relay"))),
    )
    .expect("RelayData requires peer address and packet reference");
    assert_eq!(relay.kind(), TurnCommandKind::RelayData);
}

#[test]
fn turn_relay_gate_trace_includes_refresh_and_release_lifecycle_rows() {
    for event in ["Refresh", "AcceptRefresh", "Release"] {
        assert!(
            TURN_LIFECYCLE_RULES
                .iter()
                .any(|rule| rule.event() == event),
            "TURN lifecycle trace must include {event}"
        );
    }

    let refresh = TurnCommand::try_new(
        TurnCommandKind::Refresh,
        TurnTransactionId::new(reference("turn-tx-controlled-flow-refresh")),
        TurnReferenceSet::new(None, None, None, None),
        None,
        Some(TurnRequestedLifetimeSeconds::try_new(120).expect("valid lifetime")),
        None,
    )
    .expect("Refresh command is a core-owned lifecycle command");
    assert_eq!(refresh.kind(), TurnCommandKind::Refresh);
    assert_eq!(
        TurnCommandKind::Refresh.decision_kind(),
        arcrtc_core_turn::TurnDecisionKind::Refresh
    );
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ThreePartySfuRoutingTrace {
    publisher: ParticipantId,
    subscribers: [ParticipantId; 2],
    packet_id: PacketId,
    route_id: RouteId,
    raw_payload_owned_by_driver: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ThreePartySfuRoutingTraceError {
    PublisherEqualsSubscriber,
    MissingSubscriber,
    RawPayloadOwnershipMovedToCore,
}

fn validate_three_party_sfu_routing_trace(
    trace: &ThreePartySfuRoutingTrace,
) -> Result<(), ThreePartySfuRoutingTraceError> {
    if trace.subscribers[0] == trace.subscribers[1] {
        return Err(ThreePartySfuRoutingTraceError::MissingSubscriber);
    }
    if trace.subscribers.contains(&trace.publisher) {
        return Err(ThreePartySfuRoutingTraceError::PublisherEqualsSubscriber);
    }
    if !trace.raw_payload_owned_by_driver {
        return Err(ThreePartySfuRoutingTraceError::RawPayloadOwnershipMovedToCore);
    }
    Ok(())
}

#[test]
fn sfu_three_party_routing_trace_keeps_packet_view_borrowed() {
    let trace = ThreePartySfuRoutingTrace {
        publisher: ParticipantId::new(reference("publisher-controlled-flow")),
        subscribers: [
            ParticipantId::new(reference("subscriber-a-controlled-flow")),
            ParticipantId::new(reference("subscriber-b-controlled-flow")),
        ],
        packet_id: PacketId::new(reference("packet-controlled-flow-three-party")),
        route_id: RouteId::new(reference("route-controlled-flow-three-party")),
        raw_payload_owned_by_driver: true,
    };
    validate_three_party_sfu_routing_trace(&trace).expect("three-party route trace is complete");

    let invalid = ThreePartySfuRoutingTrace {
        publisher: ParticipantId::new(reference("publisher-controlled-flow-invalid")),
        subscribers: [
            ParticipantId::new(reference("subscriber-controlled-flow-invalid")),
            ParticipantId::new(reference("subscriber-controlled-flow-invalid")),
        ],
        packet_id: PacketId::new(reference("packet-controlled-flow-invalid")),
        route_id: RouteId::new(reference("route-controlled-flow-invalid")),
        raw_payload_owned_by_driver: true,
    };
    assert_eq!(
        validate_three_party_sfu_routing_trace(&invalid),
        Err(ThreePartySfuRoutingTraceError::MissingSubscriber)
    );
    assert_eq!(
        trace.packet_id.as_str(),
        "packet-controlled-flow-three-party"
    );
    assert_eq!(trace.route_id.as_str(), "route-controlled-flow-three-party");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CrossPlaneTraceRequiredFields {
    source_plane_present: bool,
    target_plane_present: bool,
    source_reference_present: bool,
    target_reference_present: bool,
    lifecycle_present: bool,
    authorization_relation_present: bool,
}

fn validate_cross_plane_required_fields(
    fields: CrossPlaneTraceRequiredFields,
) -> Result<(), CrossPlaneBindingFailureKind> {
    if !fields.source_plane_present
        || !fields.target_plane_present
        || !fields.source_reference_present
        || !fields.target_reference_present
    {
        return Err(CrossPlaneBindingFailureKind::BindingMaterialInvalid);
    }
    if !fields.lifecycle_present {
        return Err(CrossPlaneBindingFailureKind::LifecycleConflict);
    }
    if !fields.authorization_relation_present {
        return Err(CrossPlaneBindingFailureKind::RequiredBindingAbsent);
    }
    Ok(())
}

#[test]
fn cross_plane_binding_missing_any_required_field_is_rejected() {
    assert_eq!(
        validate_cross_plane_required_fields(CrossPlaneTraceRequiredFields {
            source_plane_present: true,
            target_plane_present: true,
            source_reference_present: true,
            target_reference_present: false,
            lifecycle_present: true,
            authorization_relation_present: true,
        }),
        Err(CrossPlaneBindingFailureKind::BindingMaterialInvalid)
    );
    assert_eq!(
        validate_cross_plane_required_fields(CrossPlaneTraceRequiredFields {
            source_plane_present: true,
            target_plane_present: true,
            source_reference_present: true,
            target_reference_present: true,
            lifecycle_present: false,
            authorization_relation_present: true,
        }),
        Err(CrossPlaneBindingFailureKind::LifecycleConflict)
    );
    assert_eq!(
        validate_cross_plane_required_fields(CrossPlaneTraceRequiredFields {
            source_plane_present: true,
            target_plane_present: true,
            source_reference_present: true,
            target_reference_present: true,
            lifecycle_present: true,
            authorization_relation_present: false,
        }),
        Err(CrossPlaneBindingFailureKind::RequiredBindingAbsent)
    );
}
