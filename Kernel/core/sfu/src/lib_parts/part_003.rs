/// packet rewrite / media transform failure mapping key です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacketRewriteTransformFailureKind {
    /// rewrite/transform class is not admitted.
    PacketRewriteClassNotAdmitted,
    /// core rewrite intent invalid.
    PacketRewriteIntentInvalid,
    /// rewrite path tries to own routing/quality semantics.
    PacketRewriteOwnerViolation,
    /// payload transform requested without admitted Canonical.
    PayloadTransformNotAdmitted,
    /// codec transcode requested in initial v0.2.
    MediaTranscodeNotSupported,
    /// driver transform execution failed.
    PayloadTransformFailed,
    /// copy/allocation bound exceeded.
    RewriteCopyBoundExceeded,
}

impl PacketRewriteTransformFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::PacketRewriteClassNotAdmitted => "packet_rewrite_class_not_admitted",
            Self::PacketRewriteIntentInvalid => "packet_rewrite_intent_invalid",
            Self::PacketRewriteOwnerViolation => "packet_rewrite_owner_violation",
            Self::PayloadTransformNotAdmitted => "payload_transform_not_admitted",
            Self::MediaTranscodeNotSupported => "media_transcode_not_supported",
            Self::PayloadTransformFailed => "payload_transform_failed",
            Self::RewriteCopyBoundExceeded => "rewrite_copy_bound_exceeded",
        }
    }
}

/// media negotiation class の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediaNegotiationClass {
    /// offered codec/profile capability.
    CodecCapabilityOffer,
    /// endpoint wants to publish a media track.
    TrackPublicationOffer,
    /// endpoint wants to subscribe to media track/layer.
    TrackSubscriptionRequest,
    /// external RTP payload type to accepted media reference.
    PayloadTypeMapping,
    /// RID/layer selection intent.
    SimulcastLayerSelection,
    /// SVC dependency/layer selection intent.
    SvcLayerSelection,
    /// RTX/FEC/RTCP feedback support.
    RtxFecFeedbackSupport,
}

/// accepted codec/profile reference です。codec implementation ではありません。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CodecProfileRef(&'static str);

impl CodecProfileRef {
    /// codec/profile semantic reference を作ります。
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }
}

/// track reference です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TrackRef(&'static str);

impl TrackRef {
    /// track semantic reference を作ります。
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }
}

/// media layer reference です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MediaLayerRef(&'static str);

impl MediaLayerRef {
    /// layer semantic reference を作ります。
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }
}

/// media negotiation mapping declaration です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaNegotiationMapping {
    contract_version: &'static str,
    negotiation_class: MediaNegotiationClass,
    codec_profile: Option<CodecProfileRef>,
    track_ref: Option<TrackRef>,
    layer_ref: Option<MediaLayerRef>,
    payload_type_ref: Option<PayloadTypeRef>,
    ssrc_ref: Option<SsrcRef>,
}

impl MediaNegotiationMapping {
    /// media negotiation mapping を作ります。
    pub const fn new(
        contract_version: &'static str,
        negotiation_class: MediaNegotiationClass,
        codec_profile: Option<CodecProfileRef>,
        track_ref: Option<TrackRef>,
        layer_ref: Option<MediaLayerRef>,
        payload_type_ref: Option<PayloadTypeRef>,
        ssrc_ref: Option<SsrcRef>,
    ) -> Self {
        Self {
            contract_version,
            negotiation_class,
            codec_profile,
            track_ref,
            layer_ref,
            payload_type_ref,
            ssrc_ref,
        }
    }
}

/// media negotiation failure mapping key です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediaNegotiationFailureKind {
    /// codec/profile not supported.
    MediaCodecNotSupported,
    /// track publication/subscription not allowed.
    MediaTrackNotAllowed,
    /// requested media layer unavailable.
    MediaLayerNotAvailable,
    /// payload type / SSRC / RID / MID mapping invalid.
    MediaPayloadMappingInvalid,
    /// RTCP feedback / RTX / FEC support not available.
    MediaFeedbackNotSupported,
    /// transcoding or media transform requested but not supported.
    MediaTranscodeNotSupported,
    /// media-facing contract version unsupported.
    UnsupportedMediaContractVersion,
    /// external negotiation material cannot decode.
    ExternalDecodeFailed,
}

impl MediaNegotiationFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::MediaCodecNotSupported => "media_codec_not_supported",
            Self::MediaTrackNotAllowed => "media_track_not_allowed",
            Self::MediaLayerNotAvailable => "media_layer_not_available",
            Self::MediaPayloadMappingInvalid => "media_payload_mapping_invalid",
            Self::MediaFeedbackNotSupported => "media_feedback_not_supported",
            Self::MediaTranscodeNotSupported => "media_transcode_not_supported",
            Self::UnsupportedMediaContractVersion => "unsupported_media_contract_version",
            Self::ExternalDecodeFailed => "external_decode_failed",
        }
    }
}

/// congestion/backpressure observation class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CongestionObservationClass {
    /// transmit queue depth.
    TransmitQueueDepth,
    /// send latency.
    SendLatency,
    /// packet cache pressure.
    PacketCachePressure,
    /// packet loss signal or NACK-like event.
    PacketLossSignal,
    /// transport/backend send failure.
    SendFailure,
    /// memory pressure.
    MemoryPressure,
    /// receive buffer pressure.
    ReceiveBufferPressure,
}

/// core-owned congestion input です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CongestionInput {
    observation_class: CongestionObservationClass,
    normalized_value: u64,
}

impl CongestionInput {
    /// driver observation を normalized core input として保持します。
    pub const fn new(observation_class: CongestionObservationClass, normalized_value: u64) -> Self {
        Self {
            observation_class,
            normalized_value,
        }
    }
}

/// congestion/backpressure policy action です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CongestionPolicyAction {
    /// forwarding を delay します。
    Delay,
    /// forwarding を degrade します。
    Degrade,
    /// forwarding を suppress します。
    Suppress,
    /// packet/route を drop します。
    Drop,
    /// endpoint/route を close by policy します。
    CloseByPolicy,
    /// recovery を reject します。
    RejectRecovery,
}

/// pacing は execution であり policy ではないことを示す境界です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacingExecutionBoundary {
    /// core returns intent only.
    CoreIntentOnly,
    /// driver owns queue/timer execution.
    DriverQueueTimerExecution,
    /// queue bound overflow maps to transmit queue reason.
    BoundedQueueRequired,
}

/// retransmission execution boundary です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RetransmissionBoundary {
    /// driver-owned bounded packet cache からのみ retransmission できます。
    DriverOwnedBoundedPacketCacheOnly,
    /// core は PacketId と semantic allowance だけを返します。
    CorePacketIdAllowanceOnly,
    /// cache absence/expiry/eviction は fail-closed です。
    CacheMissFailClosed,
}

/// NACK-like feedback の core-owned reference class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeedbackReferenceClass {
    /// retransmission intent input.
    RetransmissionIntent,
    /// suppression input.
    SuppressionInput,
    /// quality/backpressure signal.
    QualityBackpressureSignal,
}

/// congestion/pacing/retransmission failure mapping key です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CongestionPacingRetransmissionFailureKind {
    /// packet cache cannot retain packet.
    PacketCacheBoundExceeded,
    /// packet cache retention duration exceeded.
    RetentionDurationExceeded,
    /// transmit queue bound exceeded.
    SfuTransmitQueueBoundExceeded,
    /// backpressure suppresses packet forwarding.
    PacketSuppressedByBackpressure,
    /// backpressure drops packet forwarding.
    PacketDroppedByBackpressure,
    /// route degraded by backpressure.
    RouteDegradedByBackpressure,
    /// endpoint closed by backpressure.
    EndpointClosedByBackpressure,
    /// recovery rejected.
    BackpressureRecoveryNotAllowed,
    /// target unavailable.
    TargetUnavailable,
    /// concrete send failed.
    NetworkSendFailed,
    /// driver shutdown.
    DriverShutdown,
}

impl CongestionPacingRetransmissionFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::PacketCacheBoundExceeded => "packet_cache_bound_exceeded",
            Self::RetentionDurationExceeded => "retention_duration_exceeded",
            Self::SfuTransmitQueueBoundExceeded => "sfu_transmit_queue_bound_exceeded",
            Self::PacketSuppressedByBackpressure => "packet_suppressed_by_backpressure",
            Self::PacketDroppedByBackpressure => "packet_dropped_by_backpressure",
            Self::RouteDegradedByBackpressure => "route_degraded_by_backpressure",
            Self::EndpointClosedByBackpressure => "endpoint_closed_by_backpressure",
            Self::BackpressureRecoveryNotAllowed => "backpressure_recovery_not_allowed",
            Self::TargetUnavailable => "target_unavailable",
            Self::NetworkSendFailed => "network_send_failed",
            Self::DriverShutdown => "driver_shutdown",
        }
    }
}
