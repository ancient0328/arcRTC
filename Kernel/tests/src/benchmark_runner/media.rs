use arcrtc_core_sfu::{
    MediaKind, PacketClass, PacketCopyPolicy, PacketHeaderSemanticView,
    PacketRewriteTransformClass, PacketRewriteTransformIntent, PacketSemanticField,
    PacketSemanticMetadata, PayloadTypeRef, RewriteCopyAllowanceClass, SfuContractItem,
    SfuDecisionKind, SfuEndpointState, SfuFailureKind, SfuModelKind, SfuPacketView,
    SfuReferenceSet, SfuRouteState, SsrcRef,
};

use super::support::{
    endpoint_id, mix_bytes, mix_usize, packet_id, route_id, session_id, stream_id,
    synthetic_packet_bytes,
};

pub(super) fn sfu_route_transition_workload() -> u64 {
    let references = SfuReferenceSet::new(
        session_id("sfu-session"),
        Some(endpoint_id("sfu-endpoint")),
        Some(stream_id("media-stream")),
        Some(route_id("route-selected")),
        Some(packet_id("packet-current")),
    );
    let route_item = SfuContractItem::new(
        SfuModelKind::RouteCandidate,
        references,
        (SfuEndpointState::Admitted, SfuRouteState::Selected),
    );
    let decisions = [
        SfuDecisionKind::RouteSelection,
        SfuDecisionKind::Forwarding,
        SfuDecisionKind::BackpressureAction,
        SfuDecisionKind::DegradationRecovery,
    ];
    let failures = [
        SfuFailureKind::RouteConflict,
        SfuFailureKind::RouteSuppressedByBackpressure,
        SfuFailureKind::PacketDroppedByBackpressure,
        SfuFailureKind::TargetUnavailable,
    ];

    let mut state = mix_bytes(0, format!("{:?}", route_item.model_kind()).as_bytes());
    for index in 0..2_048 {
        let decision = decisions[index % decisions.len()];
        let failure = failures[index % failures.len()];
        state = mix_bytes(
            state,
            format!("{:?}:{}", decision, failure.reason_code()).as_bytes(),
        );
    }
    state
}

pub(super) fn packet_semantic_view_workload() -> u64 {
    let raw_packet = synthetic_packet_bytes(1_200);
    let payload = &raw_packet[12..];
    let packet = packet_id("packet-view");
    let stream = stream_id("stream-view");
    let endpoint = endpoint_id("endpoint-view");
    let header =
        PacketHeaderSemanticView::new(PacketClass::Rtp, Some(345), Some(123_456), Some(42));
    let view = SfuPacketView::new(&packet, &stream, &endpoint, header, &raw_packet, payload);
    let metadata = PacketSemanticMetadata::new(
        Some(MediaKind::Video),
        PacketClass::Rtp,
        Some(345),
        Some(123_456),
        Some(SsrcRef::new(42)),
        Some(PayloadTypeRef::new(96)),
        Some(true),
        raw_packet.len(),
    );
    let fields = [
        PacketSemanticField::PacketId,
        PacketSemanticField::SourceEndpointId,
        PacketSemanticField::StreamId,
        PacketSemanticField::SequenceNumber,
        PacketSemanticField::Timestamp,
        PacketSemanticField::PacketLength,
    ];

    let mut state = mix_usize(0, view.payload_len());
    state = mix_usize(state, metadata.packet_length());
    for index in 0..4_096 {
        state = mix_bytes(
            state,
            format!("{:?}:{}", fields[index % fields.len()], index).as_bytes(),
        );
    }
    std::hint::black_box(view.packet_id());
    state
}

pub(super) fn packet_rewrite_transform_workload() -> u64 {
    let route = route_id("rewrite-route");
    let endpoint = endpoint_id("rewrite-target");
    let source_packet = packet_id("rewrite-source");
    let target_packet = packet_id("rewrite-target-packet");
    let classes = [
        PacketRewriteTransformClass::NoRewriteForward,
        PacketRewriteTransformClass::HeaderRewriteOnly,
        PacketRewriteTransformClass::SequenceNumberMapping,
        PacketRewriteTransformClass::SsrcRewrite,
        PacketRewriteTransformClass::RtcpFeedbackRewrite,
    ];
    let policies = [
        PacketCopyPolicy::NoCopySharedLease,
        PacketCopyPolicy::HeaderOnlyAllocationPreferred,
        PacketCopyPolicy::TargetSpecificHeaderBufferAllowed,
    ];

    let mut state = 0u64;
    for index in 0..2_048 {
        let intent = PacketRewriteTransformIntent::new(
            route.clone(),
            Some(endpoint.clone()),
            classes[index % classes.len()],
            source_packet.clone(),
            Some(target_packet.clone()),
            Some("rtp-seq-map-v0-2"),
            RewriteCopyAllowanceClass::TargetSpecificHeaderBufferAllowed,
        );
        state = mix_bytes(
            state,
            format!(
                "{:?}:{:?}:{}",
                intent.class(),
                policies[index % policies.len()],
                index
            )
            .as_bytes(),
        );
    }
    state
}
