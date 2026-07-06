// iOS SDK signaling-only surface の責務別 source shard です。
public let ARCRTC_SIGNALING_PROTOCOL_VERSION = "v0.2"
public let ARCRTC_SDK_SEMVER: String = "0.2.0"
public let ARCRTC_SDK_PACKAGE_NAME: String = "ArcRtcSdkIos"

public enum SdkOutOfScopeFeature: String, CaseIterable, Hashable, Sendable {
    case peerConnection = "PeerConnection"
    case mediaCapture = "media_capture"
    case mediaRendering = "media_rendering"
    case screenShare = "screen_share"
    case recording
    case chat
    case dataChannelApplicationSemantics = "data_channel_application_semantics"
    case authIssuance = "auth_issuance"
    case userAccountManagement = "user_account_management"
    case regulatedWorkflow = "regulated_workflow"
    case medicalDataModel = "medical_data_model"
}

public enum SdkProjectionSource: Hashable, Sendable {
    case command(SignalingCommandKind)
    case event(SignalingEventKind)

    public var name: String {
        switch self {
        case .command(let kind): kind.rawValue
        case .event(let kind): kind.rawValue
        }
    }
}

public enum SdkProjectionKindClass: String, Sendable {
    case command
    case event
}

public enum SdkCorrelationPropagation: String, Sendable {
    case required
}

public enum SdkServerReasonPreservation: String, Sendable {
    case preserveCategoryCode = "preserve_category_code"
    case notApplicable = "not_applicable"
}

public enum SdkProjectionLocalErrorWrapper: String, Sendable {
    case rejectedByServer = "rejected_by_server"
    case serverProtocolViolation = "server_protocol_violation"
    case malformedEvent = "malformed_event"
}

public enum SdkVersionCapabilityBehavior: String, Sendable {
    case sendReceiveServerResult = "send_receive_server_result"
    case preserveServerResult = "preserve_server_result"
}

public enum SdkReconnectRelation: String, Sendable {
    case signalingRejoinCommand = "signaling_rejoin_command"
    case eventStreamResubscribe = "event_stream_resubscribe"
    case notApplicable = "not_applicable"
}

public enum SdkUnsupportedSurfaceBehavior: String, Sendable {
    case sdkLocalError = "sdk_local_error"
    case serverRejection = "server_rejection"
}

public struct SdkProjectionEntry: Equatable, Sendable {
    public let source: SdkProjectionSource
    public let sourceKindClass: SdkProjectionKindClass
    public let platformApiSymbol: String
    public let publicShape: String
    public let correlationPropagation: SdkCorrelationPropagation
    public let serverReasonPreservation: SdkServerReasonPreservation
    public let localSdkErrorWrapper: SdkProjectionLocalErrorWrapper
    public let versionCapabilityBehavior: SdkVersionCapabilityBehavior
    public let reconnectRelation: SdkReconnectRelation
    public let unsupportedSurfaceBehavior: SdkUnsupportedSurfaceBehavior
}

public struct SdkPublicApiProjection: Equatable, Sendable {
    public let platform: ArcRtcSdkPlatform
    public let sourceContractVersion: SignalingContractVersion
    public let sourceContract: SdkSourceContract
    public let projectionClass: SdkProjectionClass
    public let generatedArtifactIsSemanticAuthority: Bool
    public let commands: Set<SignalingCommandKind>
    public let events: Set<SignalingEventKind>
    public let entries: [SdkProjectionEntry]
    public let outOfScopeFeatures: Set<SdkOutOfScopeFeature>
}

public let iosProjectionEntries: [SdkProjectionEntry] = [
    commandProjection(.joinRoom, "JoinRoomCommand -> SignalingSendOutcome", .signalingRejoinCommand),
    commandProjection(.leaveRoom, "LeaveRoomCommand -> SignalingSendOutcome", .notApplicable),
    commandProjection(.sendOffer, "RelayPayloadCommand.offer -> SignalingSendOutcome", .notApplicable),
    commandProjection(.sendAnswer, "RelayPayloadCommand.answer -> SignalingSendOutcome", .notApplicable),
    commandProjection(.sendIceCandidate, "RelayPayloadCommand.iceCandidate -> SignalingSendOutcome", .notApplicable),
    commandProjection(.requestTurnCredential, "RequestTurnCredentialCommand -> SignalingSendOutcome", .notApplicable),
    commandProjection(.acknowledgeForward, "AcknowledgeForwardCommand -> SignalingSendOutcome", .notApplicable),
    eventProjection(.joined, "handler(.event(.joined(..., contractVersion)))", .notApplicable, .malformedEvent),
    eventProjection(.rejected, "handler(.event(.rejected(..., contractVersion)))", .preserveCategoryCode, .rejectedByServer),
    eventProjection(.participantJoined, "handler(.event(.participantJoined(..., contractVersion)))", .notApplicable, .malformedEvent),
    eventProjection(.participantLeft, "handler(.event(.participantLeft(..., contractVersion)))", .notApplicable, .malformedEvent),
    eventProjection(.offerReceived, "handler(.event(.offerReceived(..., contractVersion)))", .notApplicable, .malformedEvent),
    eventProjection(.answerReceived, "handler(.event(.answerReceived(..., contractVersion)))", .notApplicable, .malformedEvent),
    eventProjection(.iceCandidateReceived, "handler(.event(.iceCandidateReceived(..., contractVersion)))", .notApplicable, .malformedEvent),
    eventProjection(.turnCredentialAvailable, "handler(.event(.turnCredentialAvailable(..., contractVersion)))", .notApplicable, .malformedEvent),
    eventProjection(.protocolViolation, "handler(.event(.protocolViolation(..., contractVersion)))", .preserveCategoryCode, .serverProtocolViolation)
]

private func commandProjection(
    _ kind: SignalingCommandKind,
    _ shape: String,
    _ reconnectRelation: SdkReconnectRelation
) -> SdkProjectionEntry {
    SdkProjectionEntry(
        source: .command(kind),
        sourceKindClass: .command,
        platformApiSymbol: "ArcRtcSignalingClient.send",
        publicShape: shape,
        correlationPropagation: .required,
        serverReasonPreservation: .preserveCategoryCode,
        localSdkErrorWrapper: .rejectedByServer,
        versionCapabilityBehavior: .sendReceiveServerResult,
        reconnectRelation: reconnectRelation,
        unsupportedSurfaceBehavior: .serverRejection
    )
}

private func eventProjection(
    _ kind: SignalingEventKind,
    _ shape: String,
    _ reasonPreservation: SdkServerReasonPreservation,
    _ errorWrapper: SdkProjectionLocalErrorWrapper
) -> SdkProjectionEntry {
    SdkProjectionEntry(
        source: .event(kind),
        sourceKindClass: .event,
        platformApiSymbol: "ArcRtcSignalingClient.setEventHandler",
        publicShape: shape,
        correlationPropagation: .required,
        serverReasonPreservation: reasonPreservation,
        localSdkErrorWrapper: errorWrapper,
        versionCapabilityBehavior: .preserveServerResult,
        reconnectRelation: .eventStreamResubscribe,
        unsupportedSurfaceBehavior: reasonPreservation == .preserveCategoryCode ? .serverRejection : .sdkLocalError
    )
}
