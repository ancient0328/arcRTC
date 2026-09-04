#[test]
fn sfu_packet_transform_negotiation_and_congestion_contracts_are_core_owned() {
    let packet_id = PacketId::new(reference("packet-core-api-sfu"));
    let stream_id = StreamId::new(reference("stream-core-api-sfu"));
    let endpoint_id = EndpointId::new(reference("endpoint-core-api-sfu"));
    let raw_packet = [0_u8, 1, 2, 3];
    let payload = &raw_packet[2..];
    let packet_view = SfuPacketView::new(
        &packet_id,
        &stream_id,
        &endpoint_id,
        PacketHeaderSemanticView::new(PacketClass::Rtp, Some(10), Some(20), Some(30)),
        &raw_packet,
        payload,
    );
    assert_eq!(packet_view.packet_id(), &packet_id);
    assert_eq!(packet_view.payload_len(), 2);

    let metadata = PacketSemanticMetadata::new(
        Some(MediaKind::Video),
        PacketClass::Rtp,
        Some(10),
        Some(20),
        Some(SsrcRef::new(30)),
        Some(PayloadTypeRef::new(96)),
        Some(true),
        raw_packet.len(),
    );
    assert_eq!(metadata.packet_length(), raw_packet.len());

    let transform = PacketRewriteTransformIntent::new(
        RouteId::new(reference("route-core-api-transform")),
        Some(endpoint_id),
        PacketRewriteTransformClass::PayloadTransformRequested,
        packet_id,
        None,
        None,
        RewriteCopyAllowanceClass::CopyProhibited,
    );
    assert_eq!(
        transform.class(),
        PacketRewriteTransformClass::PayloadTransformRequested
    );
    assert_eq!(
        PacketRewriteTransformFailureKind::PayloadTransformNotAdmitted.reason_code(),
        "payload_transform_not_admitted"
    );

    let _mapping = MediaNegotiationMapping::new(
        "sfu-media-v1",
        MediaNegotiationClass::TrackSubscriptionRequest,
        Some(CodecProfileRef::new("vp8")),
        Some(TrackRef::new("camera")),
        Some(MediaLayerRef::new("mid")),
        Some(PayloadTypeRef::new(96)),
        Some(SsrcRef::new(30)),
    );
    assert_eq!(
        MediaNegotiationFailureKind::MediaLayerNotAvailable.reason_code(),
        "media_layer_not_available"
    );

    let _congestion = CongestionInput::new(CongestionObservationClass::TransmitQueueDepth, 128);
    assert_eq!(
        PacingExecutionBoundary::CoreIntentOnly,
        PacingExecutionBoundary::CoreIntentOnly
    );
    assert_eq!(
        RetransmissionBoundary::CacheMissFailClosed,
        RetransmissionBoundary::CacheMissFailClosed
    );
    assert_eq!(
        FeedbackReferenceClass::RetransmissionIntent,
        FeedbackReferenceClass::RetransmissionIntent
    );
    assert_eq!(
        CongestionPacingRetransmissionFailureKind::SfuTransmitQueueBoundExceeded.reason_code(),
        "sfu_transmit_queue_bound_exceeded"
    );
    assert_eq!(
        PacketReleaseReason::DroppedByTransmitQueueBound.reason_code(),
        Some("sfu_transmit_queue_bound_exceeded")
    );
}

#[test]
fn secure_media_token_and_clock_skew_boundaries_reject_untrusted_inputs() {
    let credential_ref = CredentialRef::new(reference("credential-core-api"));
    let request = TokenVerificationRequest::new(
        correlation(),
        credential_ref.clone(),
        vec![RequiredTokenClaim::Issuer, RequiredTokenClaim::Audience],
        IssuerPolicy::new(vec!["issuer-a"]),
        AudiencePolicy::new(vec!["audience-a"]),
        TokenAlgorithmPolicy::new(vec!["EdDSA"]),
    );
    assert_eq!(request.credential_ref(), &credential_ref);
    assert_eq!(request.required_claims().len(), 2);
    let verified = VerifiedCredential::new(
        correlation(),
        credential_ref,
        TokenTemporalDecision::Expired,
    );
    assert_eq!(verified.temporal_decision(), TokenTemporalDecision::Expired);
    assert_eq!(
        TokenVerificationFailureKind::TokenExpired.reason_code(),
        "token_expired"
    );

    assert_eq!(
        ClockSkewPolicy::try_new(
            TimeNodeScope::MultiNode,
            TimeTrustClass::SingleProcessMonotonic,
            None,
            PrecisionClass::DeclaredPrecisionLabel("ms"),
            SamplingWindow::Milliseconds(1000),
            TrustedTimeSourceClass::LocalMonotonicClock,
            ClockSkewImpact::Ordering,
        ),
        Err(ClockSkewPolicyError::TrustClassCannotSupportScope)
    );
    let policy = ClockSkewPolicy::try_new(
        TimeNodeScope::MultiNode,
        TimeTrustClass::MultiNodeBoundedSkew,
        Some(250),
        PrecisionClass::DeclaredPrecisionLabel("ms"),
        SamplingWindow::Milliseconds(1000),
        TrustedTimeSourceClass::ExternalTimeSourceObservation,
        ClockSkewImpact::Expiry,
    )
    .expect("bounded multi-node skew policy is explicit");
    assert_eq!(
        TimeSynchronizationDecision::try_new(
            StartupRunId::new(reference("startup-time-core-api")),
            Some(correlation()),
            policy,
            ObservedSkewClass::ExceedsPolicy,
            TimeSynchronizationOutcome::Rejected,
            None,
        ),
        Err(TimeSynchronizationDecisionError::ReasonRequired)
    );
    assert_eq!(
        TimeSynchronizationFailureKind::ClockSkewExceeded.reason_code(),
        "clock_skew_exceeded"
    );
}
