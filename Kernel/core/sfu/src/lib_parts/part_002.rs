const SESSION_NOT_ACCEPTING: &[SfuFailureKind] = &[SfuFailureKind::SfuSessionNotAccepting];
const BEGIN_ENDPOINT_REJECTS: &[SfuFailureKind] = &[
    SfuFailureKind::SfuSessionNotAccepting,
    SfuFailureKind::EndpointCapacityExceeded,
];
const ENDPOINT_ADMISSION_REJECTS: &[SfuFailureKind] = &[
    SfuFailureKind::ParticipantNotAdmitted,
    SfuFailureKind::EndpointCapacityExceeded,
    SfuFailureKind::EndpointQualityNotAllowed,
];
const ENDPOINT_RECOVERY_REJECTS: &[SfuFailureKind] = &[SfuFailureKind::QualityRecoveryNotAllowed];
const TARGET_REJECTS: &[SfuFailureKind] = &[
    SfuFailureKind::SfuSessionNotAccepting,
    SfuFailureKind::TargetUnavailable,
];
const PUBLICATION_REJECTS: &[SfuFailureKind] = &[
    SfuFailureKind::SfuSessionNotAccepting,
    SfuFailureKind::PublicationNotAllowed,
    SfuFailureKind::PublicationQualityNotAllowed,
];
const ACCEPT_PUBLICATION_REJECTS: &[SfuFailureKind] = &[
    SfuFailureKind::SfuSessionNotAccepting,
    SfuFailureKind::TargetUnavailable,
    SfuFailureKind::PublicationNotAllowed,
    SfuFailureKind::PublicationQualityNotAllowed,
];
const REJECT_PUBLICATION_REJECTS: &[SfuFailureKind] = &[
    SfuFailureKind::PublicationNotAllowed,
    SfuFailureKind::PublicationQualityNotAllowed,
];
const SUPPRESS_PUBLICATION_REJECTS: &[SfuFailureKind] = &[
    SfuFailureKind::SfuSessionNotAccepting,
    SfuFailureKind::PublicationQualityNotAllowed,
];
const CLOSE_PUBLICATION_REJECTS: &[SfuFailureKind] = &[
    SfuFailureKind::SfuSessionNotAccepting,
    SfuFailureKind::PublicationNotAllowed,
];
const SUBSCRIPTION_REJECTS: &[SfuFailureKind] = &[
    SfuFailureKind::SfuSessionNotAccepting,
    SfuFailureKind::SubscriptionNotAllowed,
    SfuFailureKind::SubscriptionQualityNotAllowed,
];
const ACCEPT_SUBSCRIPTION_REJECTS: &[SfuFailureKind] = &[
    SfuFailureKind::SfuSessionNotAccepting,
    SfuFailureKind::TargetUnavailable,
    SfuFailureKind::SubscriptionNotAllowed,
    SfuFailureKind::SubscriptionQualityNotAllowed,
];
const REJECT_SUBSCRIPTION_REJECTS: &[SfuFailureKind] = &[
    SfuFailureKind::SubscriptionNotAllowed,
    SfuFailureKind::SubscriptionQualityNotAllowed,
];
const SUPPRESS_SUBSCRIPTION_REJECTS: &[SfuFailureKind] = &[
    SfuFailureKind::SfuSessionNotAccepting,
    SfuFailureKind::SubscriptionBackpressureSuppressed,
    SfuFailureKind::SubscriptionQualityNotAllowed,
];
const CLOSE_SUBSCRIPTION_REJECTS: &[SfuFailureKind] = &[
    SfuFailureKind::SfuSessionNotAccepting,
    SfuFailureKind::SubscriptionNotAllowed,
];
const ROUTE_REJECTS: &[SfuFailureKind] = &[
    SfuFailureKind::SfuSessionNotAccepting,
    SfuFailureKind::TargetUnavailable,
    SfuFailureKind::RouteConflict,
    SfuFailureKind::StreamNotFound,
    SfuFailureKind::RouteCandidateBoundExceeded,
];
const DELAY_REJECTS: &[SfuFailureKind] = &[
    SfuFailureKind::SfuSessionNotAccepting,
    SfuFailureKind::ActionDelayedByBackpressure,
];
const BACKPRESSURE_REJECTS: &[SfuFailureKind] = &[
    SfuFailureKind::SfuSessionNotAccepting,
    SfuFailureKind::RouteSuppressedByBackpressure,
];
const DEGRADE_BACKPRESSURE_REJECTS: &[SfuFailureKind] = &[
    SfuFailureKind::SfuSessionNotAccepting,
    SfuFailureKind::RouteDegradedByBackpressure,
];
const CLOSE_ENDPOINT_BACKPRESSURE_REJECTS: &[SfuFailureKind] = &[
    SfuFailureKind::SfuSessionNotAccepting,
    SfuFailureKind::EndpointClosedByBackpressure,
];
const QUALITY_SUPPRESSION_REJECTS: &[SfuFailureKind] = &[
    SfuFailureKind::SfuSessionNotAccepting,
    SfuFailureKind::RouteSuppressedByQuality,
];
const ROUTE_RECOVERY_REJECTS: &[SfuFailureKind] = &[
    SfuFailureKind::SfuSessionNotAccepting,
    SfuFailureKind::QualityRecoveryNotAllowed,
];
const BACKPRESSURE_RECOVERY_REJECTS: &[SfuFailureKind] = &[
    SfuFailureKind::SfuSessionNotAccepting,
    SfuFailureKind::BackpressureRecoveryNotAllowed,
];
const DROP_REJECTS: &[SfuFailureKind] = &[
    SfuFailureKind::SfuSessionNotAccepting,
    SfuFailureKind::RouteConflict,
    SfuFailureKind::TargetUnavailable,
];

/// canonical 由来の SFU transition table です。
pub const SFU_TRANSITION_RULES: &[SfuTransitionRule] = &[
    SfuTransitionRule { trigger: SfuTransitionTrigger::ObserveEndpoint, allowed_pre_state: "session=sfu_session_open", success_effect: SfuStateEffect::Endpoint(SfuEndpointState::Observed), reject_reasons: SESSION_NOT_ACCEPTING },
    SfuTransitionRule { trigger: SfuTransitionTrigger::BeginEndpointAdmission, allowed_pre_state: "session=sfu_session_open; endpoint=endpoint_observed", success_effect: SfuStateEffect::Endpoint(SfuEndpointState::AdmissionPending), reject_reasons: BEGIN_ENDPOINT_REJECTS },
    SfuTransitionRule { trigger: SfuTransitionTrigger::AdmitEndpoint, allowed_pre_state: "endpoint=endpoint_admission_pending", success_effect: SfuStateEffect::Endpoint(SfuEndpointState::Admitted), reject_reasons: ENDPOINT_ADMISSION_REJECTS },
    SfuTransitionRule { trigger: SfuTransitionTrigger::RejectEndpoint, allowed_pre_state: "endpoint=endpoint_admission_pending", success_effect: SfuStateEffect::Endpoint(SfuEndpointState::Rejected), reject_reasons: ENDPOINT_ADMISSION_REJECTS },
    SfuTransitionRule { trigger: SfuTransitionTrigger::DegradeEndpoint, allowed_pre_state: "endpoint=endpoint_admitted", success_effect: SfuStateEffect::Endpoint(SfuEndpointState::Degraded), reject_reasons: &[SfuFailureKind::EndpointDegradedByQuality] },
    SfuTransitionRule { trigger: SfuTransitionTrigger::RecoverEndpoint, allowed_pre_state: "endpoint=endpoint_degraded", success_effect: SfuStateEffect::Endpoint(SfuEndpointState::Admitted), reject_reasons: ENDPOINT_RECOVERY_REJECTS },
    SfuTransitionRule { trigger: SfuTransitionTrigger::DrainEndpoint, allowed_pre_state: "session={sfu_session_open, sfu_session_draining}; endpoint={endpoint_admitted, endpoint_degraded}", success_effect: SfuStateEffect::Endpoint(SfuEndpointState::Draining), reject_reasons: TARGET_REJECTS },
    SfuTransitionRule { trigger: SfuTransitionTrigger::RemoveEndpoint, allowed_pre_state: "session={sfu_session_open, sfu_session_draining}; endpoint=endpoint_draining", success_effect: SfuStateEffect::Endpoint(SfuEndpointState::Removed), reject_reasons: TARGET_REJECTS },
    SfuTransitionRule { trigger: SfuTransitionTrigger::StartPublication, allowed_pre_state: "session=sfu_session_open; endpoint=endpoint_admitted; publication=publication_absent", success_effect: SfuStateEffect::Publication(SfuPublicationState::Requested), reject_reasons: PUBLICATION_REJECTS },
    SfuTransitionRule { trigger: SfuTransitionTrigger::AcceptPublication, allowed_pre_state: "session=sfu_session_open; endpoint={endpoint_admitted, endpoint_degraded}; publication=publication_requested", success_effect: SfuStateEffect::Publication(SfuPublicationState::Active), reject_reasons: ACCEPT_PUBLICATION_REJECTS },
    SfuTransitionRule { trigger: SfuTransitionTrigger::RejectPublication, allowed_pre_state: "publication=publication_requested", success_effect: SfuStateEffect::Publication(SfuPublicationState::Rejected), reject_reasons: REJECT_PUBLICATION_REJECTS },
    SfuTransitionRule { trigger: SfuTransitionTrigger::SuppressPublication, allowed_pre_state: "session={sfu_session_open, sfu_session_draining}; publication=publication_active", success_effect: SfuStateEffect::Publication(SfuPublicationState::Suppressed), reject_reasons: SUPPRESS_PUBLICATION_REJECTS },
    SfuTransitionRule { trigger: SfuTransitionTrigger::ClosePublication, allowed_pre_state: "session={sfu_session_open, sfu_session_draining}; publication={publication_active, publication_suppressed}", success_effect: SfuStateEffect::Publication(SfuPublicationState::Closed), reject_reasons: CLOSE_PUBLICATION_REJECTS },
    SfuTransitionRule { trigger: SfuTransitionTrigger::StartSubscription, allowed_pre_state: "session=sfu_session_open; endpoint=endpoint_admitted; subscription=subscription_absent", success_effect: SfuStateEffect::Subscription(SfuSubscriptionState::Requested), reject_reasons: SUBSCRIPTION_REJECTS },
    SfuTransitionRule { trigger: SfuTransitionTrigger::AcceptSubscription, allowed_pre_state: "session=sfu_session_open; endpoint={endpoint_admitted, endpoint_degraded}; subscription=subscription_requested", success_effect: SfuStateEffect::Subscription(SfuSubscriptionState::Active), reject_reasons: ACCEPT_SUBSCRIPTION_REJECTS },
    SfuTransitionRule { trigger: SfuTransitionTrigger::RejectSubscription, allowed_pre_state: "subscription=subscription_requested", success_effect: SfuStateEffect::Subscription(SfuSubscriptionState::Rejected), reject_reasons: REJECT_SUBSCRIPTION_REJECTS },
    SfuTransitionRule { trigger: SfuTransitionTrigger::SuppressSubscription, allowed_pre_state: "session={sfu_session_open, sfu_session_draining}; subscription=subscription_active", success_effect: SfuStateEffect::Subscription(SfuSubscriptionState::Suppressed), reject_reasons: SUPPRESS_SUBSCRIPTION_REJECTS },
    SfuTransitionRule { trigger: SfuTransitionTrigger::CloseSubscription, allowed_pre_state: "session={sfu_session_open, sfu_session_draining}; subscription={subscription_active, subscription_suppressed}", success_effect: SfuStateEffect::Subscription(SfuSubscriptionState::Closed), reject_reasons: CLOSE_SUBSCRIPTION_REJECTS },
    SfuTransitionRule { trigger: SfuTransitionTrigger::BuildRouteCandidate, allowed_pre_state: "session=sfu_session_open; endpoint={endpoint_admitted, endpoint_degraded}; publication=publication_active; subscription=subscription_active", success_effect: SfuStateEffect::Route(SfuRouteState::Candidate), reject_reasons: ROUTE_REJECTS },
    SfuTransitionRule { trigger: SfuTransitionTrigger::SelectRoute, allowed_pre_state: "session=sfu_session_open; endpoint={endpoint_admitted, endpoint_degraded}; publication=publication_active; subscription=subscription_active; route=route_candidate", success_effect: SfuStateEffect::Route(SfuRouteState::Selected), reject_reasons: ROUTE_REJECTS },
    SfuTransitionRule { trigger: SfuTransitionTrigger::DelayActionByBackpressure, allowed_pre_state: "session={sfu_session_open, sfu_session_draining}; route={route_candidate, route_selected}", success_effect: SfuStateEffect::Route(SfuRouteState::Delayed), reject_reasons: DELAY_REJECTS },
    SfuTransitionRule { trigger: SfuTransitionTrigger::ApplyBackpressure, allowed_pre_state: "session={sfu_session_open, sfu_session_draining}; route=route_selected", success_effect: SfuStateEffect::Route(SfuRouteState::SuppressedByBackpressure), reject_reasons: BACKPRESSURE_REJECTS },
    SfuTransitionRule { trigger: SfuTransitionTrigger::DegradeRouteByBackpressure, allowed_pre_state: "session={sfu_session_open, sfu_session_draining}; route={route_selected, route_delayed}", success_effect: SfuStateEffect::Route(SfuRouteState::Degraded), reject_reasons: DEGRADE_BACKPRESSURE_REJECTS },
    SfuTransitionRule { trigger: SfuTransitionTrigger::CloseEndpointByBackpressure, allowed_pre_state: "session={sfu_session_open, sfu_session_draining}; endpoint={endpoint_admitted, endpoint_degraded}", success_effect: SfuStateEffect::Endpoint(SfuEndpointState::Draining), reject_reasons: CLOSE_ENDPOINT_BACKPRESSURE_REJECTS },
    SfuTransitionRule { trigger: SfuTransitionTrigger::ApplyQualitySuppression, allowed_pre_state: "session={sfu_session_open, sfu_session_draining}; route=route_selected", success_effect: SfuStateEffect::Route(SfuRouteState::SuppressedByQuality), reject_reasons: QUALITY_SUPPRESSION_REJECTS },
    SfuTransitionRule { trigger: SfuTransitionTrigger::RecoverRoute, allowed_pre_state: "session={sfu_session_open, sfu_session_draining}; route=route_suppressed_by_quality", success_effect: SfuStateEffect::Route(SfuRouteState::Selected), reject_reasons: ROUTE_RECOVERY_REJECTS },
    SfuTransitionRule { trigger: SfuTransitionTrigger::RecoverRouteFromBackpressure, allowed_pre_state: "session={sfu_session_open, sfu_session_draining}; route={route_delayed, route_degraded, route_suppressed_by_backpressure}", success_effect: SfuStateEffect::Route(SfuRouteState::Selected), reject_reasons: BACKPRESSURE_RECOVERY_REJECTS },
    SfuTransitionRule { trigger: SfuTransitionTrigger::DropRoute, allowed_pre_state: "session={sfu_session_open, sfu_session_draining}; route={route_candidate, route_selected, route_delayed, route_suppressed_by_backpressure, route_suppressed_by_quality, route_degraded}", success_effect: SfuStateEffect::Route(SfuRouteState::Dropped), reject_reasons: DROP_REJECTS },
    SfuTransitionRule { trigger: SfuTransitionTrigger::CloseRoute, allowed_pre_state: "session={sfu_session_open, sfu_session_draining}; route={route_selected, route_delayed, route_suppressed_by_backpressure, route_suppressed_by_quality, route_degraded, route_dropped}", success_effect: SfuStateEffect::Route(SfuRouteState::Closed), reject_reasons: TARGET_REJECTS },
    SfuTransitionRule { trigger: SfuTransitionTrigger::BeginSessionDrain, allowed_pre_state: "session=sfu_session_open", success_effect: SfuStateEffect::Session(SfuSessionState::Draining), reject_reasons: SESSION_NOT_ACCEPTING },
    SfuTransitionRule { trigger: SfuTransitionTrigger::CloseSession, allowed_pre_state: "session={sfu_session_open, sfu_session_draining}", success_effect: SfuStateEffect::Session(SfuSessionState::Closed), reject_reasons: SESSION_NOT_ACCEPTING },
];

/// RTP / RTCP packet class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacketClass {
    /// RTP packet semantic view です。
    Rtp,
    /// RTCP packet semantic view です。
    Rtcp,
}

/// routing に必要な semantic header view です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PacketHeaderSemanticView {
    packet_class: PacketClass,
    sequence_number: Option<u64>,
    timestamp: Option<u64>,
    synchronization_source: Option<u32>,
}

impl PacketHeaderSemanticView {
    /// packet header semantic view を作ります。
    pub const fn new(
        packet_class: PacketClass,
        sequence_number: Option<u64>,
        timestamp: Option<u64>,
        synchronization_source: Option<u32>,
    ) -> Self {
        Self {
            packet_class,
            sequence_number,
            timestamp,
            synchronization_source,
        }
    }
}

/// driver-owned buffer を routing call 中だけ借用する packet view です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SfuPacketView<'packet> {
    packet_id: &'packet PacketId,
    stream_id: &'packet StreamId,
    source_endpoint_id: &'packet EndpointId,
    header: PacketHeaderSemanticView,
    raw_packet: &'packet [u8],
    payload: &'packet [u8],
}

impl<'packet> SfuPacketView<'packet> {
    /// borrowed packet semantic view を作ります。
    pub const fn new(
        packet_id: &'packet PacketId,
        stream_id: &'packet StreamId,
        source_endpoint_id: &'packet EndpointId,
        header: PacketHeaderSemanticView,
        raw_packet: &'packet [u8],
        payload: &'packet [u8],
    ) -> Self {
        Self {
            packet_id,
            stream_id,
            source_endpoint_id,
            header,
            raw_packet,
            payload,
        }
    }

    /// packet identity です。
    pub const fn packet_id(&self) -> &'packet PacketId {
        self.packet_id
    }

    /// payload length です。payload bytes の所有は driver のままです。
    pub const fn payload_len(&self) -> usize {
        self.payload.len()
    }
}

/// fan-out / rewrite 時の copy policy です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacketCopyPolicy {
    /// rewrite 不要。copy 禁止で shared lease を使います。
    NoCopySharedLease,
    /// header-only allocation または scatter/gather を優先します。
    HeaderOnlyAllocationPreferred,
    /// target-specific header buffer は許可、payload copy は避けます。
    TargetSpecificHeaderBufferAllowed,
    /// payload transform 必須時の copy-on-write / new buffer allocation です。
    PayloadCopyOnWriteAllowed,
    /// encryption/framing backend が要求する driver-local copy です。
    DriverLocalBackendCopyAllowed,
}

/// packet cache bound requirement です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacketCacheBoundKind {
    /// maximum packets.
    MaximumPackets,
    /// maximum bytes.
    MaximumBytes,
    /// maximum retention duration.
    MaximumRetentionDuration,
    /// eviction reason.
    EvictionReason,
}

/// driver packet lifecycle release reason の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacketReleaseReason {
    /// packet was forwarded to selected target.
    Forwarded,
    /// core route decision suppressed forwarding.
    SuppressedByRoutingDecision,
    /// core quality decision suppressed packet forwarding.
    SuppressedByQualityDecision,
    /// core backpressure decision suppressed packet forwarding.
    SuppressedByBackpressure,
    /// core backpressure decision dropped packet.
    DroppedByBackpressure,
    /// SFU transmit queue bound prevented scheduling.
    DroppedByTransmitQueueBound,
    /// target endpoint cannot receive packet.
    TargetUnavailable,
    /// driver send operation failed.
    SendFailed,
    /// packet retention duration ended.
    ExpiredFromPacketCache,
    /// packet cache bound forced eviction.
    EvictedByCacheBound,
    /// driver transform or encode step failed.
    TransformFailed,
    /// driver failed to release buffer lease.
    ReleaseFailed,
    /// driver shutdown ended packet lifecycle.
    DriverShutdown,
}

impl PacketReleaseReason {
    /// catalog reason code for non-success, if required.
    pub const fn reason_code(self) -> Option<&'static str> {
        match self {
            Self::Forwarded => None,
            Self::SuppressedByRoutingDecision => Some("route_conflict"),
            Self::SuppressedByQualityDecision => Some("packet_suppressed_by_quality"),
            Self::SuppressedByBackpressure => Some("packet_suppressed_by_backpressure"),
            Self::DroppedByBackpressure => Some("packet_dropped_by_backpressure"),
            Self::DroppedByTransmitQueueBound => Some("sfu_transmit_queue_bound_exceeded"),
            Self::TargetUnavailable => Some("target_unavailable"),
            Self::SendFailed => Some("network_send_failed"),
            Self::ExpiredFromPacketCache => Some("retention_duration_exceeded"),
            Self::EvictedByCacheBound => Some("packet_cache_bound_exceeded"),
            Self::TransformFailed => Some("payload_transform_failed"),
            Self::ReleaseFailed => Some("buffer_release_failed"),
            Self::DriverShutdown => Some("driver_shutdown"),
        }
    }
}

/// packet semantic view field の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacketSemanticField {
    /// core-owned packet reference.
    PacketId,
    /// correlated flow reference.
    CorrelationId,
    /// core-owned source endpoint reference.
    SourceEndpointId,
    /// core-owned stream reference.
    StreamId,
    /// media kind.
    MediaKind,
    /// RTP or RTCP semantic class.
    PacketKind,
    /// routing/retransmission semantic sequence number.
    SequenceNumber,
    /// RTP timestamp semantic.
    Timestamp,
    /// opaque SSRC reference.
    SsrcRef,
    /// payload type reference.
    PayloadTypeRef,
    /// routing/quality marker hint.
    Marker,
    /// resource/quality packet length.
    PacketLength,
    /// core time observation.
    ArrivalTime,
}

/// media kind の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediaKind {
    /// audio media.
    Audio,
    /// video media.
    Video,
    /// data-like media is not admitted as application semantics.
    DataLikeTransport,
}

/// opaque SSRC reference です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SsrcRef(u32);

impl SsrcRef {
    /// SSRC semantic reference を作ります。
    pub const fn new(value: u32) -> Self {
        Self(value)
    }
}

/// payload type reference です。codec negotiation authority ではありません。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PayloadTypeRef(u8);

impl PayloadTypeRef {
    /// payload type semantic reference を作ります。
    pub const fn new(value: u8) -> Self {
        Self(value)
    }
}

/// core が参照できる packet semantic metadata です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PacketSemanticMetadata {
    media_kind: Option<MediaKind>,
    packet_kind: PacketClass,
    sequence_number: Option<u64>,
    timestamp: Option<u64>,
    ssrc_ref: Option<SsrcRef>,
    payload_type_ref: Option<PayloadTypeRef>,
    marker: Option<bool>,
    packet_length: usize,
}

impl PacketSemanticMetadata {
    /// packet semantic metadata を作ります。
    pub const fn new(
        media_kind: Option<MediaKind>,
        packet_kind: PacketClass,
        sequence_number: Option<u64>,
        timestamp: Option<u64>,
        ssrc_ref: Option<SsrcRef>,
        payload_type_ref: Option<PayloadTypeRef>,
        marker: Option<bool>,
        packet_length: usize,
    ) -> Self {
        Self {
            media_kind,
            packet_kind,
            sequence_number,
            timestamp,
            ssrc_ref,
            payload_type_ref,
            marker,
            packet_length,
        }
    }

    /// packet length です。
    pub const fn packet_length(&self) -> usize {
        self.packet_length
    }
}

/// packet semantic view failure mapping key です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacketSemanticViewFailureKind {
    /// parser cannot produce required semantic view.
    ExternalDecodeFailed,
    /// frame size bound exceeded.
    FrameSizeBoundExceeded,
    /// unsupported media contract version.
    UnsupportedMediaContractVersion,
    /// buffer release failed.
    BufferReleaseFailed,
}

impl PacketSemanticViewFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::ExternalDecodeFailed => "external_decode_failed",
            Self::FrameSizeBoundExceeded => "frame_size_bound_exceeded",
            Self::UnsupportedMediaContractVersion => "unsupported_media_contract_version",
            Self::BufferReleaseFailed => "buffer_release_failed",
        }
    }
}

/// packet rewrite / media transform class の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacketRewriteTransformClass {
    /// original packet lease can be forwarded unchanged.
    NoRewriteForward,
    /// driver rewrites RTP/RTCP header fields only.
    HeaderRewriteOnly,
    /// driver applies core-owned sequence mapping intent.
    SequenceNumberMapping,
    /// driver rewrites SSRC based on accepted route/stream reference.
    SsrcRewrite,
    /// driver maps admitted RTCP feedback reference.
    RtcpFeedbackRewrite,
    /// encryption/framing backend requires driver-local copy.
    FramingSecurityBackendCopy,
    /// payload bytes must be modified; rejected unless admitted.
    PayloadTransformRequested,
    /// codec decode/encode/transcode is required; rejected in initial v0.2.
    CodecTranscodeRequested,
}

/// rewrite/transform copy allowance class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RewriteCopyAllowanceClass {
    /// copy prohibited by default.
    CopyProhibited,
    /// header-only allocation or scatter/gather preferred.
    HeaderOnlyAllocationPreferred,
    /// target-specific header buffer allowed.
    TargetSpecificHeaderBufferAllowed,
    /// driver-local copy allowed by backend requirement.
    DriverLocalCopyAllowed,
    /// copy-on-write/new buffer allowed only for admitted transform.
    PayloadCopyOnWriteAllowedWhenAdmitted,
}

/// core が返せる rewrite/transform intent です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketRewriteTransformIntent {
    forwarding_reference: RouteId,
    target_endpoint_id: Option<EndpointId>,
    class: PacketRewriteTransformClass,
    source_packet_id: PacketId,
    target_packet_id: Option<PacketId>,
    mapping_table_ref: Option<&'static str>,
    copy_allowance: RewriteCopyAllowanceClass,
}

impl PacketRewriteTransformIntent {
    /// rewrite/transform intent を作ります。
    pub const fn new(
        forwarding_reference: RouteId,
        target_endpoint_id: Option<EndpointId>,
        class: PacketRewriteTransformClass,
        source_packet_id: PacketId,
        target_packet_id: Option<PacketId>,
        mapping_table_ref: Option<&'static str>,
        copy_allowance: RewriteCopyAllowanceClass,
    ) -> Self {
        Self {
            forwarding_reference,
            target_endpoint_id,
            class,
            source_packet_id,
            target_packet_id,
            mapping_table_ref,
            copy_allowance,
        }
    }

    /// rewrite/transform class です。
    pub const fn class(&self) -> PacketRewriteTransformClass {
        self.class
    }
}

