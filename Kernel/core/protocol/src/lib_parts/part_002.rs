/// semantic envelope の message kind です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticEnvelopeMessageKind {
    /// command.
    Command,
    /// event.
    Event,
}

/// Signaling semantic command type の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalingSemanticCommandType {
    /// JoinRoom.
    JoinRoom,
    /// LeaveRoom.
    LeaveRoom,
    /// SendOffer.
    SendOffer,
    /// SendAnswer.
    SendAnswer,
    /// SendIceCandidate.
    SendIceCandidate,
    /// RequestTurnCredential.
    RequestTurnCredential,
    /// AcknowledgeForward.
    AcknowledgeForward,
}

/// Signaling semantic event type の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalingSemanticEventType {
    /// Joined.
    Joined,
    /// Rejected.
    Rejected,
    /// ParticipantJoined.
    ParticipantJoined,
    /// ParticipantLeft.
    ParticipantLeft,
    /// OfferReceived.
    OfferReceived,
    /// AnswerReceived.
    AnswerReceived,
    /// IceCandidateReceived.
    IceCandidateReceived,
    /// TurnCredentialAvailable.
    TurnCredentialAvailable,
    /// ProtocolViolation.
    ProtocolViolation,
}

/// SFU semantic model type の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SfuSemanticModelType {
    /// SFU session.
    SfuSession,
    /// participant endpoint.
    ParticipantEndpoint,
    /// media stream.
    MediaStream,
    /// media negotiation reference.
    MediaNegotiationReference,
    /// publication.
    Publication,
    /// subscription.
    Subscription,
    /// forwarding intent.
    ForwardingIntent,
    /// route candidate.
    RouteCandidate,
    /// borrowed packet abstract view.
    BorrowedPacketAbstractView,
    /// quality observation.
    QualityObservation,
    /// backpressure state.
    BackpressureState,
    /// admission decision.
    AdmissionDecision,
    /// rejection reason.
    RejectionReason,
}

/// SFU semantic decision type の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SfuSemanticDecisionType {
    /// participant admission/rejection.
    ParticipantAdmission,
    /// publication accepted/rejected.
    Publication,
    /// subscription accepted/rejected.
    Subscription,
    /// route selected/not selected.
    RouteSelection,
    /// forwarding allowed/suppressed.
    Forwarding,
    /// degradation/recovery.
    DegradationRecovery,
    /// backpressure action.
    BackpressureAction,
    /// quality violation.
    QualityViolation,
}

/// TURN semantic model type の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnSemanticModelType {
    /// transaction ID.
    TransactionId,
    /// request message.
    Request,
    /// response message.
    Response,
    /// indication message.
    Indication,
    /// error message.
    Error,
    /// allocation.
    Allocation,
    /// permission.
    Permission,
    /// channel binding.
    ChannelBinding,
    /// channel binding reference.
    ChannelBindingReference,
    /// peer address.
    PeerAddress,
    /// relay decision.
    RelayDecision,
    /// credential verification outcome.
    CredentialVerificationOutcome,
    /// lifetime / expiry.
    LifetimeExpiry,
    /// closed error reason.
    ClosedErrorReason,
}

/// TURN semantic decision type の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnSemanticDecisionType {
    /// allocation accepted/rejected.
    Allocation,
    /// allocation released/expired.
    AllocationLifecycle,
    /// refresh accepted/rejected/expired.
    Refresh,
    /// permission accepted/rejected/revoked/expired.
    Permission,
    /// channel bind accepted/rejected/expired.
    ChannelBind,
    /// relay allowed/denied.
    Relay,
    /// malformed message classification.
    MalformedMessage,
    /// expired credential classification.
    ExpiredCredential,
    /// unauthorized request classification.
    UnauthorizedRequest,
}

/// transport semantic command type の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportSemanticCommandType {
    /// start session.
    StartSession,
    /// apply local description.
    ApplyLocalDescription,
    /// apply remote description.
    ApplyRemoteDescription,
    /// add ICE candidate.
    AddIceCandidate,
    /// forward packet.
    ForwardPacket,
    /// close session.
    CloseSession,
}

/// transport semantic event type の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportSemanticEventType {
    /// session observed.
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

/// core semantic envelope で許可する command/event type の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoreSemanticMessageType {
    /// Signaling command.
    SignalingCommand(SignalingSemanticCommandType),
    /// Signaling event.
    SignalingEvent(SignalingSemanticEventType),
    /// SFU model.
    SfuModel(SfuSemanticModelType),
    /// SFU decision.
    SfuDecision(SfuSemanticDecisionType),
    /// TURN model.
    TurnModel(TurnSemanticModelType),
    /// TURN decision.
    TurnDecision(TurnSemanticDecisionType),
    /// transport command.
    TransportCommand(TransportSemanticCommandType),
    /// transport event.
    TransportEvent(TransportSemanticEventType),
    /// driver conversion failure observation.
    DriverErrorConverted,
}

/// core semantic envelope に載せられる payload model class の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoreSemanticPayloadClass {
    /// Signaling subject reference set.
    SignalingSubject,
    /// SFU reference set.
    SfuReferenceSet,
    /// TURN reference set.
    TurnReferenceSet,
    /// packet semantic view reference.
    PacketSemanticView,
    /// accepted opaque core reference.
    OpaqueCoreReference,
}

/// core semantic envelope に載せられる payload model です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CoreSemanticPayloadModel {
    class: CoreSemanticPayloadClass,
    reference: Option<OpaqueReference>,
}

impl CoreSemanticPayloadModel {
    /// payload class と core-owned reference を保持します。
    pub const fn new(class: CoreSemanticPayloadClass, reference: Option<OpaqueReference>) -> Self {
        Self { class, reference }
    }
}

/// core boundary を跨げる semantic envelope です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreSemanticEnvelope {
    surface: TargetSurface,
    contract_version: ContractVersion,
    correlation_id: CorrelationId,
    message_kind: SemanticEnvelopeMessageKind,
    message_type: CoreSemanticMessageType,
    subject_reference_materialized: bool,
    payload_model: Option<CoreSemanticPayloadModel>,
    reason: Option<CatalogedReasonRef>,
}

/// semantic envelope construction の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticEnvelopeError {
    /// non-success outcome に cataloged reason がありません。
    RequiredReasonMissing,
    /// success outcome に driver-created reason を載せています。
    SuccessReasonMustNotBeInvented,
}

impl CoreSemanticEnvelope {
    /// core-owned semantic envelope を作ります。
    pub fn try_new(
        surface: TargetSurface,
        contract_version: ContractVersion,
        correlation_id: CorrelationId,
        message_kind: SemanticEnvelopeMessageKind,
        message_type: CoreSemanticMessageType,
        subject_reference_materialized: bool,
        payload_model: Option<CoreSemanticPayloadModel>,
        outcome: Option<UseCaseOutcome>,
        reason: Option<CatalogedReasonRef>,
    ) -> Result<Self, SemanticEnvelopeError> {
        if matches!(outcome, Some(outcome) if outcome.requires_reason()) && reason.is_none() {
            return Err(SemanticEnvelopeError::RequiredReasonMissing);
        }
        if matches!(outcome, Some(outcome) if !outcome.requires_reason()) && reason.is_some() {
            return Err(SemanticEnvelopeError::SuccessReasonMustNotBeInvented);
        }

        Ok(Self {
            surface,
            contract_version,
            correlation_id,
            message_kind,
            message_type,
            subject_reference_materialized,
            payload_model,
            reason,
        })
    }
}
