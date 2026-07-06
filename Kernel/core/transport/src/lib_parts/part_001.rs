// core/transport は Sans-IO transport contract の所有 surface です。
//
// ここでは socket、runtime、str0m などの具象実装を持たず、core が扱う
// transport 境界の語彙だけを段階的に配置します。

use arcrtc_core_identity::{OpaqueReference, PacketId, SessionId};
use arcrtc_core_reason::CatalogedReasonRef;

/// core transport package の所有境界を示す marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreTransportSurface;

/// driver implementation に対する abstract transport command です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportCommandKind {
    /// transport session を開始します。
    StartSession,
    /// local description intent を適用します。
    ApplyLocalDescription,
    /// remote description intent を適用します。
    ApplyRemoteDescription,
    /// ICE candidate intent を追加します。
    AddIceCandidate,
    /// transport-level packet forwarding intent です。
    ForwardPacket,
    /// transport session を閉じます。
    CloseSession,
}

/// external transport event を core meaning へ変換した event です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportEventKind {
    /// transport session observed.
    SessionObserved,
    /// local description accepted.
    LocalDescriptionAccepted,
    /// remote description accepted.
    RemoteDescriptionAccepted,
    /// ICE candidate observed.
    IceCandidateObserved,
    /// packet semantic view observed.
    PacketSemanticViewObserved,
    /// transport closed.
    TransportClosed,
    /// conversion failure observed.
    ConversionFailed,
}

/// WebRTC transport capability の抽象表現です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransportCapability {
    name: &'static str,
}

impl TransportCapability {
    /// capability 名を作ります。
    pub const fn new(name: &'static str) -> Self {
        Self { name }
    }

    /// capability name です。
    pub const fn name(&self) -> &'static str {
        self.name
    }
}

/// SDP payload そのものではない opaque/validated semantic reference です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionDescriptionRef {
    session_id: SessionId,
    description_ref: String,
}

impl SessionDescriptionRef {
    /// validated semantic reference を作ります。
    pub fn new(
        session_id: SessionId,
        description_ref: impl Into<String>,
    ) -> Result<Self, TransportContractError> {
        let description_ref = description_ref.into();
        if description_ref.is_empty() || description_ref.chars().any(char::is_control) {
            return Err(TransportContractError::InvalidSemanticReference);
        }
        Ok(Self {
            session_id,
            description_ref,
        })
    }
}

/// candidate relay / negotiation intent の core-owned reference です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IceCandidateRef {
    session_id: SessionId,
    candidate_ref: String,
}

impl IceCandidateRef {
    /// validated semantic reference を作ります。
    pub fn new(
        session_id: SessionId,
        candidate_ref: impl Into<String>,
    ) -> Result<Self, TransportContractError> {
        let candidate_ref = candidate_ref.into();
        if candidate_ref.is_empty() || candidate_ref.chars().any(char::is_control) {
            return Err(TransportContractError::InvalidSemanticReference);
        }
        Ok(Self {
            session_id,
            candidate_ref,
        })
    }
}

/// routing に必要な borrowed packet view への reference です。raw bytes は持ちません。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PacketSemanticViewRef {
    packet_id: PacketId,
}

impl PacketSemanticViewRef {
    /// packet semantic view reference を作ります。
    pub const fn new(packet_id: PacketId) -> Self {
        Self { packet_id }
    }
}

/// transport command contract です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportCommand<Payload> {
    kind: TransportCommandKind,
    payload: Payload,
}

impl<Payload> TransportCommand<Payload> {
    /// transport command を作ります。
    pub const fn new(kind: TransportCommandKind, payload: Payload) -> Self {
        Self { kind, payload }
    }
}

/// transport event contract です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportEvent<Payload> {
    kind: TransportEventKind,
    payload: Payload,
}

impl<Payload> TransportEvent<Payload> {
    /// transport event を作ります。
    pub const fn new(kind: TransportEventKind, payload: Payload) -> Self {
        Self { kind, payload }
    }
}

/// transport failure mapping key です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportFailureKind {
    /// unsupported transport/media contract version.
    UnsupportedMediaContractVersion,
    /// external SDP/ICE/packet material cannot decode to core type.
    ExternalDecodeFailed,
    /// external response encoding failed.
    ExternalEncodeFailed,
    /// media payload mapping invalid.
    MediaPayloadMappingInvalid,
}

impl TransportFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::UnsupportedMediaContractVersion => "unsupported_media_contract_version",
            Self::ExternalDecodeFailed => "external_decode_failed",
            Self::ExternalEncodeFailed => "external_encode_failed",
            Self::MediaPayloadMappingInvalid => "media_payload_mapping_invalid",
        }
    }
}

/// WebRTC transport driver から core-facing port へ返せる failure の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportDriverFailureKind {
    /// external SDP/ICE/packet material cannot decode to core type.
    ExternalDecodeFailed,
    /// external response encoding failed.
    ExternalEncodeFailed,
    /// media payload mapping invalid.
    MediaPayloadMappingInvalid,
    /// driver shutdown.
    DriverShutdown,
}

impl TransportDriverFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::ExternalDecodeFailed => TransportFailureKind::ExternalDecodeFailed.reason_code(),
            Self::ExternalEncodeFailed => TransportFailureKind::ExternalEncodeFailed.reason_code(),
            Self::MediaPayloadMappingInvalid => {
                TransportFailureKind::MediaPayloadMappingInvalid.reason_code()
            }
            Self::DriverShutdown => "driver_shutdown",
        }
    }
}

/// core-owned transport driver failure です。driver 固有 error object は含めません。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransportDriverFailure {
    kind: TransportDriverFailureKind,
    reason: CatalogedReasonRef,
}

impl TransportDriverFailure {
    /// cataloged reason と接続した failure を作ります。
    pub fn from_kind(kind: TransportDriverFailureKind) -> Self {
        let reason = CatalogedReasonRef::from_code(kind.reason_code())
            .expect("transport driver failure reason code must be registered");
        Self { kind, reason }
    }

    /// failure kind です。
    pub const fn kind(&self) -> TransportDriverFailureKind {
        self.kind
    }

    /// cataloged reason reference です。
    pub const fn reason(&self) -> CatalogedReasonRef {
        self.reason
    }
}

/// WebRTC transport port の core-owned command payload です。SDP/ICE/packet bytes は含めません。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebRtcTransportPayload {
    /// no additional payload.
    Empty,
    /// core-owned session description reference.
    SessionDescription(SessionDescriptionRef),
    /// core-owned ICE candidate reference.
    IceCandidate(IceCandidateRef),
    /// core-owned packet semantic view reference.
    PacketView(PacketSemanticViewRef),
    /// abstract WebRTC transport capability.
    Capability(TransportCapability),
}

/// WebRTC transport port の core-owned observation payload です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebRtcTransportObservation {
    /// no additional payload.
    Empty,
    /// core-owned session description reference.
    SessionDescription(SessionDescriptionRef),
    /// core-owned ICE candidate reference.
    IceCandidate(IceCandidateRef),
    /// core-owned packet semantic view reference.
    PacketView(PacketSemanticViewRef),
    /// driver conversion failure を cataloged reason 付き core failure として観測します。
    ConversionFailure(TransportDriverFailure),
}

/// WebRTC transport driver input の core-owned 型です。
pub type WebRtcTransportInput = TransportCommand<WebRtcTransportPayload>;

/// WebRTC transport driver output の core-owned 型です。
pub type WebRtcTransportOutput = TransportEvent<WebRtcTransportObservation>;

/// transport contract shape rule 違反です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportContractError {
    /// semantic reference が空または制御文字を含みます。
    InvalidSemanticReference,
}

/// SDP / ICE negotiation material class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NegotiationMaterialClass {
    /// SDP offer relay intent です。
    Offer,
    /// SDP answer relay intent です。
    Answer,
    /// ICE candidate relay intent です。
    IceCandidate,
}

/// inbound negotiation material の処理順序です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NegotiationFlowStep {
    /// driver/sdk boundary が external material を受けます。
    ReceiveExternalMaterial,
    /// driver が wire shape/size/version/decode を確認します。
    DriverDecodeAndValidate,
    /// driver が core-owned reference shape へ変換します。
    MapToCoreReference,
    /// core/signaling が room/participant/order/version/correlation を評価します。
    CoreSignalingEvaluation,
    /// core が relay accept/reject/protocol violation を返します。
    CoreRelayDecision,
    /// driver/sdk が reason を変えずに external projection へ写します。
    ExternalProjection,
}

/// SDP / ICE relay decision class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NegotiationDecisionKind {
    /// accepted relay です。
    AcceptedRelay,
    /// rejected relay です。
    RejectedRelay,
    /// protocol violation です。
    ProtocolViolation,
}

/// SDP / ICE version surface です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NegotiationVersionSurface {
    /// Signaling command/event version.
    SignalingCommandEvent,
    /// media-facing transport contract version.
    MediaFacingTransportContract,
    /// driver wire encoding version.
    DriverWireEncoding,
    /// SDK public API version.
    SdkPublicApi,
}

/// SDK parity requirement です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SdkNegotiationParityRequirement {
    /// offer/answer/ICE command/event set を揃えます。
    SameCommandEventSet,
    /// correlation propagation を揃えます。
    SameCorrelationPropagation,
    /// server reason category/code を保持します。
    PreserveServerReason,
    /// version negotiation behavior を揃えます。
    SameVersionNegotiation,
    /// Signaling-only boundary を維持します。
    SignalingOnlyBoundary,
}

/// SDP / ICE negotiation failure mapping key です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NegotiationFailureKind {
    /// external SDP/ICE material cannot decode to reference shape.
    ExternalDecodeFailed,
    /// candidate class or exposure policy rejected.
    IceCandidatePolicyViolation,
    /// candidate connectivity check failed.
    IceConnectivityCheckFailed,
    /// ICE consent expired.
    IceConsentExpired,
    /// required wire field absent.
    MissingRequiredWireField,
    /// external enum or attribute cannot map.
    ExternalEnumUnmapped,
    /// Signaling command version unsupported.
    UnsupportedCommandVersion,
    /// media-facing contract version unsupported.
    UnsupportedMediaContractVersion,
    /// media codec/profile unsupported.
    MediaCodecNotSupported,
    /// media payload/track/layer mapping invalid.
    MediaPayloadMappingInvalid,
    /// participant not joined.
    ParticipantNotJoined,
    /// command arrives in invalid order.
    CommandOrderViolation,
    /// room is draining.
    RoomDraining,
    /// room is closed.
    RoomClosed,
    /// network send failed.
    NetworkSendFailed,
    /// network receive failed.
    NetworkReceiveFailed,
    /// driver shutdown.
    DriverShutdown,
}

impl NegotiationFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::ExternalDecodeFailed => "external_decode_failed",
            Self::IceCandidatePolicyViolation => "ice_candidate_policy_violation",
            Self::IceConnectivityCheckFailed => "ice_connectivity_check_failed",
            Self::IceConsentExpired => "ice_consent_expired",
            Self::MissingRequiredWireField => "missing_required_wire_field",
            Self::ExternalEnumUnmapped => "external_enum_unmapped",
            Self::UnsupportedCommandVersion => "unsupported_command_version",
            Self::UnsupportedMediaContractVersion => "unsupported_media_contract_version",
            Self::MediaCodecNotSupported => "media_codec_not_supported",
            Self::MediaPayloadMappingInvalid => "media_payload_mapping_invalid",
            Self::ParticipantNotJoined => "participant_not_joined",
            Self::CommandOrderViolation => "command_order_violation",
            Self::RoomDraining => "room_draining",
            Self::RoomClosed => "room_closed",
            Self::NetworkSendFailed => "network_send_failed",
            Self::NetworkReceiveFailed => "network_receive_failed",
            Self::DriverShutdown => "driver_shutdown",
        }
    }
}

/// ICE candidate / connectivity class の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IceCandidateConnectivityClass {
    /// host candidate reference after policy.
    HostCandidateRef,
    /// server-reflexive candidate reference.
    SrflxCandidateRef,
    /// TURN relay candidate reference.
    RelayCandidateRef,
    /// mDNS-obfuscated candidate reference.
    MdnsCandidateRef,
    /// incremental candidate relay.
    TrickleCandidateRef,
    /// restart negotiation intent.
    IceRestartIntent,
    /// driver observed connectivity result.
    ConnectivityObservation,
    /// driver observed consent state.
    ConsentFreshnessObservation,
}

/// ICE address exposure policy です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IceAddressExposurePolicy {
    /// relay-only policy です。
    RelayOnly,
    /// host candidate exposure is allowed by explicit policy.
    HostAllowed,
    /// srflx candidate exposure is allowed by explicit policy.
    SrflxAllowed,
    /// mDNS obfuscation is required.
    MdnsObfuscationRequired,
    /// raw address redaction is required.
    RawAddressRedactionRequired,
}

/// ICE policy declaration です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IceCandidatePolicy {
    accepted_classes: Vec<IceCandidateConnectivityClass>,
    exposure_policy: IceAddressExposurePolicy,
    restart_allowed: bool,
}

impl IceCandidatePolicy {
    /// accepted classes、exposure policy、restart allowance を固定します。
    pub fn new(
        accepted_classes: Vec<IceCandidateConnectivityClass>,
        exposure_policy: IceAddressExposurePolicy,
        restart_allowed: bool,
    ) -> Self {
        Self {
            accepted_classes,
            exposure_policy,
            restart_allowed,
        }
    }

    /// accepted candidate/connectivity classes です。
    pub fn accepted_classes(&self) -> &[IceCandidateConnectivityClass] {
        &self.accepted_classes
    }
}

/// ICE observation は relay acceptance ではないことを示す class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IceObservationMeaning {
    /// diagnostic/evidence class です。
    DiagnosticEvidenceOnly,
    /// Signaling success ではありません。
    NotSignalingRelaySuccess,
    /// TURN/SFU/secure media binding ではありません。
    NotCrossPlaneBinding,
}

/// ICE candidate/connectivity failure mapping key です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IceFailureKind {
    /// candidate class not allowed by policy.
    IceCandidatePolicyViolation,
    /// candidate cannot map to core reference.
    IceCandidateMappingInvalid,
    /// candidate material requires redaction.
    IceCandidateRedactionRequired,
    /// candidate gathering failed before relay evidence.
    IceGatheringFailed,
    /// connectivity check failed.
    IceConnectivityCheckFailed,
    /// consent freshness expired or failed.
    IceConsentExpired,
    /// ICE restart not allowed.
    IceRestartNotAllowed,
    /// external ICE material cannot decode.
    ExternalDecodeFailed,
    /// command ordering invalid.
    CommandOrderViolation,
    /// network receive failed.
    NetworkReceiveFailed,
    /// network send failed.
    NetworkSendFailed,
}
