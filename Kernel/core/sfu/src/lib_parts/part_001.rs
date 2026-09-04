// core/sfu は SFU routing と media forwarding 意味論を所有する surface です。
//
// driver が保持する RTP/RTCP byte buffer や I/O queue はここへ持ち込まず、
// core-owned semantic view と判断語彙だけを配置します。

use arcrtc_core_command::{TargetSurface, UseCaseDecision};
use arcrtc_core_identity::{EndpointId, PacketId, RouteId, SessionId, StreamId};

/// core SFU package の所有境界を示す marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreSfuSurface;

/// SFU core model の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SfuModelKind {
    /// SFU session です。
    SfuSession,
    /// participant endpoint です。
    ParticipantEndpoint,
    /// media stream です。
    MediaStream,
    /// negotiated codec/track/layer reference です。
    MediaNegotiationReference,
    /// publication です。
    Publication,
    /// subscription です。
    Subscription,
    /// forwarding intent です。
    ForwardingIntent,
    /// route candidate です。
    RouteCandidate,
    /// borrowed packet abstract view です。
    BorrowedPacketAbstractView,
    /// core-owned quality observation です。
    QualityObservation,
    /// backpressure state です。
    BackpressureState,
    /// admission decision です。
    AdmissionDecision,
    /// rejection reason です。
    RejectionReason,
}

/// SFU decision の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SfuDecisionKind {
    /// participant admission / rejection です。
    ParticipantAdmission,
    /// publication accepted / rejected です。
    Publication,
    /// subscription accepted / rejected です。
    Subscription,
    /// route selected / not selected です。
    RouteSelection,
    /// forwarding allowed / suppressed です。
    Forwarding,
    /// degradation / recovery decision です。
    DegradationRecovery,
    /// backpressure action です。
    BackpressureAction,
    /// quality violation classification です。
    QualityViolation,
}

/// SFU contract が扱う core-owned references です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SfuReferenceSet {
    session_id: SessionId,
    endpoint_id: Option<EndpointId>,
    stream_id: Option<StreamId>,
    route_id: Option<RouteId>,
    packet_id: Option<PacketId>,
}

impl SfuReferenceSet {
    /// SFU reference set を作ります。
    pub const fn new(
        session_id: SessionId,
        endpoint_id: Option<EndpointId>,
        stream_id: Option<StreamId>,
        route_id: Option<RouteId>,
        packet_id: Option<PacketId>,
    ) -> Self {
        Self {
            session_id,
            endpoint_id,
            stream_id,
            route_id,
            packet_id,
        }
    }

    /// session reference です。
    pub const fn session_id(&self) -> &SessionId {
        &self.session_id
    }
}

/// publication / subscription / route / forwarding の contract item です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SfuContractItem<Payload> {
    model_kind: SfuModelKind,
    references: SfuReferenceSet,
    payload: Payload,
}

impl<Payload> SfuContractItem<Payload> {
    /// SFU contract item を作ります。
    pub const fn new(
        model_kind: SfuModelKind,
        references: SfuReferenceSet,
        payload: Payload,
    ) -> Self {
        Self {
            model_kind,
            references,
            payload,
        }
    }

    /// model kind です。
    pub const fn model_kind(&self) -> SfuModelKind {
        self.model_kind
    }
}

/// SFU decision contract です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SfuDecision<Reason> {
    kind: SfuDecisionKind,
    decision: UseCaseDecision<Reason>,
}

impl<Reason> SfuDecision<Reason> {
    /// SFU decision を core command decision から作ります。
    pub fn new(
        kind: SfuDecisionKind,
        decision: UseCaseDecision<Reason>,
    ) -> Result<Self, SfuContractError> {
        if decision.target_surface() != TargetSurface::Sfu {
            return Err(SfuContractError::WrongTargetSurface);
        }
        Ok(Self { kind, decision })
    }

    /// SFU decision kind です。
    pub const fn kind(&self) -> SfuDecisionKind {
        self.kind
    }
}

/// SFU fail-closed failure mapping key です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SfuFailureKind {
    /// unknown participant.
    ParticipantNotAdmitted,
    /// SFU session is not accepting.
    SfuSessionNotAccepting,
    /// endpoint admission capacity exceeded.
    EndpointCapacityExceeded,
    /// endpoint admission rejected by quality policy.
    EndpointQualityNotAllowed,
    /// endpoint degraded by quality policy.
    EndpointDegradedByQuality,
    /// endpoint or route recovery rejected by quality policy.
    QualityRecoveryNotAllowed,
    /// endpoint or route target unavailable.
    TargetUnavailable,
    /// invalid stream identity.
    StreamNotFound,
    /// unauthorized publication.
    PublicationNotAllowed,
    /// publication rejected or suppressed by quality policy.
    PublicationQualityNotAllowed,
    /// unauthorized subscription.
    SubscriptionNotAllowed,
    /// subscription rejected or suppressed by quality policy.
    SubscriptionQualityNotAllowed,
    /// route conflict.
    RouteConflict,
    /// route candidate bound exceeded.
    RouteCandidateBoundExceeded,
    /// route suppressed by quality policy.
    RouteSuppressedByQuality,
    /// route suppressed by backpressure policy.
    RouteSuppressedByBackpressure,
    /// quality policy violation.
    PacketSuppressedByQuality,
    /// backpressure threshold violation.
    PacketSuppressedByBackpressure,
    /// packet dropped by backpressure policy.
    PacketDroppedByBackpressure,
    /// subscription suppressed by backpressure policy.
    SubscriptionBackpressureSuppressed,
    /// action delayed by backpressure policy.
    ActionDelayedByBackpressure,
    /// route degraded by backpressure policy.
    RouteDegradedByBackpressure,
    /// endpoint closed by backpressure policy.
    EndpointClosedByBackpressure,
    /// recovery from backpressure state rejected.
    BackpressureRecoveryNotAllowed,
    /// SFU transmit queue bound exceeded.
    SfuTransmitQueueBoundExceeded,
    /// unsupported media contract version.
    UnsupportedMediaContractVersion,
    /// unsupported codec/profile.
    MediaCodecNotSupported,
    /// track publication/subscription not allowed.
    MediaTrackNotAllowed,
    /// requested media layer unavailable.
    MediaLayerNotAvailable,
    /// invalid payload/SSRC/RID/MID mapping.
    MediaPayloadMappingInvalid,
}

impl SfuFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::ParticipantNotAdmitted => "participant_not_admitted",
            Self::SfuSessionNotAccepting => "sfu_session_not_accepting",
            Self::EndpointCapacityExceeded => "endpoint_capacity_exceeded",
            Self::EndpointQualityNotAllowed => "endpoint_quality_not_allowed",
            Self::EndpointDegradedByQuality => "endpoint_degraded_by_quality",
            Self::QualityRecoveryNotAllowed => "quality_recovery_not_allowed",
            Self::TargetUnavailable => "target_unavailable",
            Self::StreamNotFound => "stream_not_found",
            Self::PublicationNotAllowed => "publication_not_allowed",
            Self::PublicationQualityNotAllowed => "publication_quality_not_allowed",
            Self::SubscriptionNotAllowed => "subscription_not_allowed",
            Self::SubscriptionQualityNotAllowed => "subscription_quality_not_allowed",
            Self::RouteConflict => "route_conflict",
            Self::RouteCandidateBoundExceeded => "route_candidate_bound_exceeded",
            Self::RouteSuppressedByQuality => "route_suppressed_by_quality",
            Self::RouteSuppressedByBackpressure => "route_suppressed_by_backpressure",
            Self::PacketSuppressedByQuality => "packet_suppressed_by_quality",
            Self::PacketSuppressedByBackpressure => "packet_suppressed_by_backpressure",
            Self::PacketDroppedByBackpressure => "packet_dropped_by_backpressure",
            Self::SubscriptionBackpressureSuppressed => "subscription_backpressure_suppressed",
            Self::ActionDelayedByBackpressure => "action_delayed_by_backpressure",
            Self::RouteDegradedByBackpressure => "route_degraded_by_backpressure",
            Self::EndpointClosedByBackpressure => "endpoint_closed_by_backpressure",
            Self::BackpressureRecoveryNotAllowed => "backpressure_recovery_not_allowed",
            Self::SfuTransmitQueueBoundExceeded => "sfu_transmit_queue_bound_exceeded",
            Self::UnsupportedMediaContractVersion => "unsupported_media_contract_version",
            Self::MediaCodecNotSupported => "media_codec_not_supported",
            Self::MediaTrackNotAllowed => "media_track_not_allowed",
            Self::MediaLayerNotAvailable => "media_layer_not_available",
            Self::MediaPayloadMappingInvalid => "media_payload_mapping_invalid",
        }
    }
}

/// SFU core に禁止される semantics です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedSfuSemantic {
    /// medical workflow priority.
    MedicalWorkflowPriority,
    /// application-specific room policy.
    ApplicationSpecificRoomPolicy,
    /// UI state.
    UiState,
    /// recording policy.
    RecordingPolicy,
    /// chat semantics.
    ChatSemantics,
    /// screen share workflow.
    ScreenShareWorkflow,
    /// DataChannel application semantics.
    DataChannelApplicationSemantics,
    /// regulated data classification.
    RegulatedDataClassification,
    /// concrete worker thread strategy.
    ConcreteWorkerThreadStrategy,
    /// packet bytes ownership.
    PacketBytesOwnership,
    /// packet cache.
    PacketCache,
    /// transmit queue.
    TransmitQueue,
    /// codec implementation / transcoding backend.
    CodecImplementationTranscodingBackend,
}

/// SFU contract shape rule 違反です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SfuContractError {
    /// decision target surface が SFU ではありません。
    WrongTargetSurface,
}

/// SFU session state の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SfuSessionState {
    /// endpoints and route decisions を受理します。
    Open,
    /// new admission を抑止し existing routes を drain します。
    Draining,
    /// all new SFU decisions を拒否します。
    Closed,
}

/// SFU endpoint state の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SfuEndpointState {
    /// driver observed endpoint.
    Observed,
    /// admission decision in progress.
    AdmissionPending,
    /// endpoint can publish or subscribe.
    Admitted,
    /// admitted with degraded quality state.
    Degraded,
    /// endpoint is leaving.
    Draining,
    /// endpoint is no longer routable.
    Removed,
    /// endpoint admission rejected.
    Rejected,
}

/// SFU publication state の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SfuPublicationState {
    /// no active publication.
    Absent,
    /// publication is being evaluated.
    Requested,
    /// stream can be routed.
    Active,
    /// stream exists but routing is suppressed.
    Suppressed,
    /// publication ended.
    Closed,
    /// publication rejected.
    Rejected,
}

/// SFU subscription state の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SfuSubscriptionState {
    /// no subscription.
    Absent,
    /// subscription is being evaluated.
    Requested,
    /// target can receive route.
    Active,
    /// subscription exists but forwarding is suppressed.
    Suppressed,
    /// subscription ended.
    Closed,
    /// subscription rejected.
    Rejected,
}

/// SFU route state の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SfuRouteState {
    /// route may be selected.
    Candidate,
    /// route selected by core.
    Selected,
    /// route action delayed by backpressure.
    Delayed,
    /// route suppressed by backpressure.
    SuppressedByBackpressure,
    /// route suppressed by quality.
    SuppressedByQuality,
    /// route remains usable only in degraded form.
    Degraded,
    /// route cannot be used.
    Dropped,
    /// route ended.
    Closed,
}

/// SFU state transition trigger の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SfuTransitionTrigger {
    /// endpoint observation.
    ObserveEndpoint,
    /// endpoint admission begin.
    BeginEndpointAdmission,
    /// endpoint admitted.
    AdmitEndpoint,
    /// endpoint rejected.
    RejectEndpoint,
    /// endpoint degraded.
    DegradeEndpoint,
    /// endpoint recovered.
    RecoverEndpoint,
    /// endpoint drain.
    DrainEndpoint,
    /// endpoint removal.
    RemoveEndpoint,
    /// publication start.
    StartPublication,
    /// publication accepted.
    AcceptPublication,
    /// publication rejected.
    RejectPublication,
    /// publication suppressed.
    SuppressPublication,
    /// publication closed.
    ClosePublication,
    /// subscription start.
    StartSubscription,
    /// subscription accepted.
    AcceptSubscription,
    /// subscription rejected.
    RejectSubscription,
    /// subscription suppressed.
    SuppressSubscription,
    /// subscription closed.
    CloseSubscription,
    /// route candidate build.
    BuildRouteCandidate,
    /// route select.
    SelectRoute,
    /// action delayed by backpressure.
    DelayActionByBackpressure,
    /// backpressure applied.
    ApplyBackpressure,
    /// route degraded by backpressure.
    DegradeRouteByBackpressure,
    /// endpoint closed by backpressure.
    CloseEndpointByBackpressure,
    /// quality suppression applied.
    ApplyQualitySuppression,
    /// quality-suppressed route recovered.
    RecoverRoute,
    /// backpressure route recovered.
    RecoverRouteFromBackpressure,
    /// route dropped.
    DropRoute,
    /// route closed.
    CloseRoute,
    /// session drain begin.
    BeginSessionDrain,
    /// session close.
    CloseSession,
}

/// SFU state transition の success effect です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SfuStateEffect {
    /// session state update です。
    Session(SfuSessionState),
    /// endpoint state update です。
    Endpoint(SfuEndpointState),
    /// publication state update です。
    Publication(SfuPublicationState),
    /// subscription state update です。
    Subscription(SfuSubscriptionState),
    /// route state update です。
    Route(SfuRouteState),
}

/// SFU state transition rule です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SfuTransitionRule {
    trigger: SfuTransitionTrigger,
    allowed_pre_state: &'static str,
    success_effect: SfuStateEffect,
    reject_reasons: &'static [SfuFailureKind],
}

impl SfuTransitionRule {
    /// transition trigger です。
    pub const fn trigger(&self) -> SfuTransitionTrigger {
        self.trigger
    }

    /// transition source contract の pre-state tuple notation です。
    pub const fn allowed_pre_state(&self) -> &'static str {
        self.allowed_pre_state
    }

    /// success effect です。
    pub const fn success_effect(&self) -> SfuStateEffect {
        self.success_effect
    }

    /// reject reason candidates です。
    pub const fn reject_reasons(&self) -> &'static [SfuFailureKind] {
        self.reject_reasons
    }
}
