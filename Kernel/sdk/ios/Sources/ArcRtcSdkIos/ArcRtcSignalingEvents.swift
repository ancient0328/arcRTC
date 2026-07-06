// iOS SDK signaling-only surface の責務別 source shard です。
public struct ArcRtcJoinAcceptedEvent: Equatable, Sendable {
    public let roomId: String
    public let participantId: String
    public let sessionRef: String

    public init(roomId: String, participantId: String, sessionRef: String) {
        self.roomId = roomId
        self.participantId = participantId
        self.sessionRef = sessionRef
    }
}

public struct ArcRtcJoinRejectedEvent: Equatable, Sendable {
    public let roomId: String
    public let participantId: String?
    public let reasonCode: String

    public init(roomId: String, participantId: String? = nil, reasonCode: String) {
        self.roomId = roomId
        self.participantId = participantId
        self.reasonCode = reasonCode
    }
}

public struct ArcRtcParticipantLeftEvent: Equatable, Sendable {
    public let roomId: String
    public let participantId: String
    public let sessionRef: String?

    public init(roomId: String, participantId: String, sessionRef: String? = nil) {
        self.roomId = roomId
        self.participantId = participantId
        self.sessionRef = sessionRef
    }
}

public struct ArcRtcNegotiationRequiredEvent: Equatable, Sendable {
    public let roomId: String
    public let participantId: String
    public let sdpRef: String?
    public let sessionRef: String

    public init(roomId: String, participantId: String, sdpRef: String? = nil, sessionRef: String) {
        self.roomId = roomId
        self.participantId = participantId
        self.sdpRef = sdpRef
        self.sessionRef = sessionRef
    }
}

public struct ArcRtcIceCandidateReceivedEvent: Equatable, Sendable {
    public let roomId: String
    public let participantId: String
    public let candidateRef: String
    public let sessionRef: String

    public init(roomId: String, participantId: String, candidateRef: String, sessionRef: String) {
        self.roomId = roomId
        self.participantId = participantId
        self.candidateRef = candidateRef
        self.sessionRef = sessionRef
    }
}

public struct ArcRtcSessionTimedOutEvent: Equatable, Sendable {
    public let roomId: String
    public let participantId: String
    public let sessionRef: String
    public let reasonCode: String

    public init(roomId: String, participantId: String, sessionRef: String, reasonCode: String) {
        self.roomId = roomId
        self.participantId = participantId
        self.sessionRef = sessionRef
        self.reasonCode = reasonCode
    }
}

public enum SignalingEvent: Equatable, Sendable {
    case joined(correlationId: CorrelationId, participantRef: OpaqueReference, roomRef: OpaqueReference, contractVersion: SignalingContractVersion = .v0_2)
    case rejected(correlationId: CorrelationId, serverReason: CatalogedServerReason, contractVersion: SignalingContractVersion = .v0_2)
    case participantJoined(correlationId: CorrelationId, participantRef: OpaqueReference, contractVersion: SignalingContractVersion = .v0_2)
    case participantLeft(correlationId: CorrelationId, participantRef: OpaqueReference, contractVersion: SignalingContractVersion = .v0_2)
    case offerReceived(correlationId: CorrelationId, fromParticipantRef: OpaqueReference, offer: OpaqueSignalingPayload, contractVersion: SignalingContractVersion = .v0_2)
    case answerReceived(correlationId: CorrelationId, fromParticipantRef: OpaqueReference, answer: OpaqueSignalingPayload, contractVersion: SignalingContractVersion = .v0_2)
    case iceCandidateReceived(correlationId: CorrelationId, fromParticipantRef: OpaqueReference, candidate: OpaqueSignalingPayload, contractVersion: SignalingContractVersion = .v0_2)
    case turnCredentialAvailable(correlationId: CorrelationId, credentialRef: OpaqueReference, contractVersion: SignalingContractVersion = .v0_2)
    case protocolViolation(correlationId: CorrelationId, serverReason: CatalogedServerReason, contractVersion: SignalingContractVersion = .v0_2)

    // Signaling event の共通 envelope は platform 間 parity のため公開 accessor で固定します。
    public var kind: SignalingEventKind {
        switch self {
        case .joined: .joined
        case .rejected: .rejected
        case .participantJoined: .participantJoined
        case .participantLeft: .participantLeft
        case .offerReceived: .offerReceived
        case .answerReceived: .answerReceived
        case .iceCandidateReceived: .iceCandidateReceived
        case .turnCredentialAvailable: .turnCredentialAvailable
        case .protocolViolation: .protocolViolation
        }
    }

    public var correlationId: CorrelationId {
        switch self {
        case .joined(let correlationId, _, _, _): correlationId
        case .rejected(let correlationId, _, _): correlationId
        case .participantJoined(let correlationId, _, _): correlationId
        case .participantLeft(let correlationId, _, _): correlationId
        case .offerReceived(let correlationId, _, _, _): correlationId
        case .answerReceived(let correlationId, _, _, _): correlationId
        case .iceCandidateReceived(let correlationId, _, _, _): correlationId
        case .turnCredentialAvailable(let correlationId, _, _): correlationId
        case .protocolViolation(let correlationId, _, _): correlationId
        }
    }

    public var contractVersion: SignalingContractVersion {
        switch self {
        case .joined(_, _, _, let contractVersion): contractVersion
        case .rejected(_, _, let contractVersion): contractVersion
        case .participantJoined(_, _, let contractVersion): contractVersion
        case .participantLeft(_, _, let contractVersion): contractVersion
        case .offerReceived(_, _, _, let contractVersion): contractVersion
        case .answerReceived(_, _, _, let contractVersion): contractVersion
        case .iceCandidateReceived(_, _, _, let contractVersion): contractVersion
        case .turnCredentialAvailable(_, _, let contractVersion): contractVersion
        case .protocolViolation(_, _, let contractVersion): contractVersion
        }
    }
}

public enum SignalingEventCallbackItem: Equatable, Sendable {
    case event(SignalingEvent)
    case error(ArcRtcSdkError)
}

public enum SdkReconnectClass: String, Sendable {
    case localTransportReconnect = "local_transport_reconnect"
    case signalingRejoinCommand = "signaling_rejoin_command"
    case eventStreamResubscribe = "event_stream_resubscribe"
    case sessionResumptionRequested = "session_resumption_requested"
    case reconnectExhausted = "reconnect_exhausted"
}

public struct SdkReconnectPolicy: Equatable, Sendable {
    public let reconnectClass: SdkReconnectClass
    public let localRetryLimit: Int
    public let initialDelayMs: Int
    public let maxDelayMs: Int
    public let explicitServerCommand: SignalingCommandKind?

    public init(
        reconnectClass: SdkReconnectClass,
        localRetryLimit: Int,
        initialDelayMs: Int,
        maxDelayMs: Int,
        explicitServerCommand: SignalingCommandKind? = nil
    ) {
        self.reconnectClass = reconnectClass
        self.localRetryLimit = localRetryLimit
        self.initialDelayMs = initialDelayMs
        self.maxDelayMs = maxDelayMs
        self.explicitServerCommand = explicitServerCommand
    }
}

public protocol ArcRtcSignalingClient {
    var platform: ArcRtcSdkPlatform { get }
    var contractVersion: SignalingContractVersion { get }

    func connect(endpoint: String, correlationId: CorrelationId, completion: @escaping (ArcRtcSdkError?) -> Void)
    func close(correlationId: CorrelationId?, completion: @escaping (ArcRtcSdkError?) -> Void)
    func send(_ command: SignalingCommand, completion: @escaping (SignalingSendOutcome) -> Void)
    func setEventHandler(_ handler: @escaping (SignalingEventCallbackItem) -> Void)
}
