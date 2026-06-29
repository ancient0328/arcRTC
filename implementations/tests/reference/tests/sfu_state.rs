//! reference SFU state mutation 境界を直接実行して検査します。

use arcrtc_core_identity::{
    EndpointId, OpaqueReference, PacketId, ReferenceAuthority, RouteId, SessionId, StreamId,
};
use arcrtc_core_sfu::{
    PacketClass, PacketHeaderSemanticView, SfuDecisionKind, SfuModelKind, SfuReferenceSet,
};
use arcrtc_reference_sfu::{
    apply_reference_sfu, build_borrowed_packet_view, build_kernel_sfu_item,
    validate_reference_sfu_state, ReferenceSfuAction, ReferenceSfuContractInput, ReferenceSfuError,
    ReferenceSfuState, ReferenceSfuSuppressionSource,
};

fn accepted(value: &str) -> OpaqueReference {
    OpaqueReference::accept(value, ReferenceAuthority::CoreValidatedUntrustedInput)
        .expect("test reference must be accepted")
}

fn session() -> SessionId {
    SessionId::new(accepted("sfu-session"))
}

fn endpoint() -> EndpointId {
    EndpointId::new(accepted("sfu-endpoint"))
}

fn stream() -> StreamId {
    StreamId::new(accepted("sfu-stream"))
}

fn route() -> RouteId {
    RouteId::new(accepted("sfu-route"))
}

fn packet() -> PacketId {
    PacketId::new(accepted("sfu-packet"))
}

fn references() -> SfuReferenceSet {
    SfuReferenceSet::new(
        session(),
        Some(endpoint()),
        Some(stream()),
        Some(route()),
        Some(packet()),
    )
}

#[test]
fn kpi_reference_sfu_executes_kernel_contract_driven_state_transition() {
    let mut state = ReferenceSfuState::default();
    let packet_id = packet();
    let stream_id = stream();
    let endpoint_id = endpoint();
    let raw_packet = [
        0x80, 0x60, 0x00, 0x2a, 0x00, 0x00, 0x03, 0xe8, 0, 0, 0, 7, 1, 2, 3, 4,
    ];
    let payload = &raw_packet[12..];
    let packet_view = build_borrowed_packet_view(
        &packet_id,
        &stream_id,
        &endpoint_id,
        PacketHeaderSemanticView::new(PacketClass::Rtp, Some(42), Some(1000), Some(7)),
        &raw_packet,
        payload,
    );

    // KPI-T3-3 は Kernel contract item と borrowed packet view を reference transition の起点として固定する。
    let packet_item = build_kernel_sfu_item(ReferenceSfuContractInput::new(
        SfuModelKind::BorrowedPacketAbstractView,
        references(),
        packet_view,
    ));
    assert_eq!(
        packet_item.model_kind(),
        SfuModelKind::BorrowedPacketAbstractView
    );
    assert_eq!(packet_view.packet_id(), &packet_id);
    assert_eq!(packet_view.payload_len(), payload.len());

    let admitted = ReferenceSfuAction::AdmitParticipant {
        session_id: session(),
        endpoint_id: endpoint(),
    };
    let admission_item = build_kernel_sfu_item(ReferenceSfuContractInput::new(
        SfuModelKind::AdmissionDecision,
        references(),
        admitted.clone(),
    ));
    assert_eq!(admission_item.model_kind(), SfuModelKind::AdmissionDecision);
    assert_eq!(
        apply_reference_sfu(&mut state, &admitted)
            .expect("Kernel-derived admission must succeed")
            .kind,
        SfuDecisionKind::ParticipantAdmission
    );

    let subscribed = ReferenceSfuAction::SubscribeRoute {
        session_id: session(),
        endpoint_id: endpoint(),
        stream_id: stream(),
        route_id: route(),
    };
    let subscription_item = build_kernel_sfu_item(ReferenceSfuContractInput::new(
        SfuModelKind::Subscription,
        references(),
        subscribed.clone(),
    ));
    assert_eq!(subscription_item.model_kind(), SfuModelKind::Subscription);
    assert_eq!(
        apply_reference_sfu(&mut state, &subscribed)
            .expect("Kernel-derived subscription must succeed")
            .kind,
        SfuDecisionKind::Subscription
    );

    let selected = ReferenceSfuAction::SelectRoute {
        session_id: session(),
        endpoint_id: endpoint(),
        stream_id: stream(),
        route_id: route(),
    };
    let route_item = build_kernel_sfu_item(ReferenceSfuContractInput::new(
        SfuModelKind::RouteCandidate,
        references(),
        selected.clone(),
    ));
    assert_eq!(route_item.model_kind(), SfuModelKind::RouteCandidate);
    assert_eq!(
        apply_reference_sfu(&mut state, &selected)
            .expect("Kernel-derived route selection must succeed")
            .kind,
        SfuDecisionKind::RouteSelection
    );

    let forwarding = ReferenceSfuAction::SuppressForwarding {
        route_id: route(),
        source: ReferenceSfuSuppressionSource::Backpressure,
    };
    let forwarding_item = build_kernel_sfu_item(ReferenceSfuContractInput::new(
        SfuModelKind::ForwardingIntent,
        references(),
        forwarding.clone(),
    ));
    assert_eq!(forwarding_item.model_kind(), SfuModelKind::ForwardingIntent);
    assert_eq!(
        apply_reference_sfu(&mut state, &forwarding)
            .expect("Kernel-derived forwarding suppression must succeed")
            .kind,
        SfuDecisionKind::Forwarding
    );
    assert_eq!(validate_reference_sfu_state(&state), Ok(()));
}

#[test]
fn sfu_state_applies_route_lifecycle_and_projection_in_order() {
    let mut state = ReferenceSfuState::default();

    let admitted = apply_reference_sfu(
        &mut state,
        &ReferenceSfuAction::AdmitParticipant {
            session_id: session(),
            endpoint_id: endpoint(),
        },
    )
    .expect("admit");
    assert_eq!(admitted.kind, SfuDecisionKind::ParticipantAdmission);

    let subscribed = apply_reference_sfu(
        &mut state,
        &ReferenceSfuAction::SubscribeRoute {
            session_id: session(),
            endpoint_id: endpoint(),
            stream_id: stream(),
            route_id: route(),
        },
    )
    .expect("subscribe");
    assert_eq!(subscribed.kind, SfuDecisionKind::Subscription);

    let selected = apply_reference_sfu(
        &mut state,
        &ReferenceSfuAction::SelectRoute {
            session_id: session(),
            endpoint_id: endpoint(),
            stream_id: stream(),
            route_id: route(),
        },
    )
    .expect("select");
    assert_eq!(selected.kind, SfuDecisionKind::RouteSelection);

    let suppressed = apply_reference_sfu(
        &mut state,
        &ReferenceSfuAction::SuppressForwarding {
            route_id: route(),
            source: ReferenceSfuSuppressionSource::Backpressure,
        },
    )
    .expect("suppress");
    assert_eq!(suppressed.kind, SfuDecisionKind::Forwarding);

    let dropped = apply_reference_sfu(
        &mut state,
        &ReferenceSfuAction::DropForwarding { route_id: route() },
    )
    .expect("drop");
    assert_eq!(dropped.kind, SfuDecisionKind::BackpressureAction);
    assert_eq!(validate_reference_sfu_state(&state), Ok(()));
}

#[test]
fn sfu_state_rejects_route_selection_before_subscription() {
    let mut state = ReferenceSfuState::default();
    apply_reference_sfu(
        &mut state,
        &ReferenceSfuAction::AdmitParticipant {
            session_id: session(),
            endpoint_id: endpoint(),
        },
    )
    .expect("admit");

    assert_eq!(
        apply_reference_sfu(
            &mut state,
            &ReferenceSfuAction::SelectRoute {
                session_id: session(),
                endpoint_id: endpoint(),
                stream_id: stream(),
                route_id: route(),
            },
        ),
        Err(ReferenceSfuError::StateBoundaryViolation)
    );
}
