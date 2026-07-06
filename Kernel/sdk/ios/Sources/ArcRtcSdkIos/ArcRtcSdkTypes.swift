// iOS SDK signaling-only surface の責務別 source shard です。
public enum ArcRtcSdkPlatform: String, Sendable {
    case ios
}

public enum SignalingContractVersion: String, Sendable {
    case v0_2 = "v0.2"
}

public enum SdkSourceContract: String, Sendable {
    case signalingContractCanonical = "SIGNALING_CONTRACT_CANONICAL"
}

public enum SdkProjectionClass: String, Sendable {
    case platformApiProjection = "platform_api_projection"
}

public struct CorrelationId: Hashable, Sendable {
    public let value: String

    public init(_ value: String) {
        self.value = value
    }
}

public struct OpaqueReference: Hashable, Sendable {
    public let value: String

    public init(_ value: String) {
        self.value = value
    }
}

public struct OpaqueSignalingPayload: Equatable, Sendable {
    public let fields: [String: String]

    public init(fields: [String: String]) {
        self.fields = fields
    }
}

// SDK は core reason の権威を持たず、公開 failure code を閉集合で示すだけです。
public enum ArcRtcSdkFailureCode: String, CaseIterable, Hashable, Sendable {
    case CredentialRejected = "CredentialRejected"
    case JoinRejected = "JoinRejected"
    case MembershipConflict = "MembershipConflict"
    case NegotiationRejected = "NegotiationRejected"
    case IceCandidateRejected = "IceCandidateRejected"
    case Timeout = "Timeout"
    case TransportUnavailable = "TransportUnavailable"
    case ProtocolViolation = "ProtocolViolation"
    case VersionMismatch = "VersionMismatch"
    case InternalInvariantViolation = "InternalInvariantViolation"
}

public enum SignalingCommandKind: String, CaseIterable, Hashable, Sendable {
    case joinRoom = "JoinRoom"
    case leaveRoom = "LeaveRoom"
    case sendOffer = "SendOffer"
    case sendAnswer = "SendAnswer"
    case sendIceCandidate = "SendIceCandidate"
    case requestTurnCredential = "RequestTurnCredential"
    case acknowledgeForward = "AcknowledgeForward"
}

public enum SignalingEventKind: String, CaseIterable, Hashable, Sendable {
    case joined = "Joined"
    case rejected = "Rejected"
    case participantJoined = "ParticipantJoined"
    case participantLeft = "ParticipantLeft"
    case offerReceived = "OfferReceived"
    case answerReceived = "AnswerReceived"
    case iceCandidateReceived = "IceCandidateReceived"
    case turnCredentialAvailable = "TurnCredentialAvailable"
    case protocolViolation = "ProtocolViolation"
}

public struct CatalogedServerReason: Equatable, Sendable {
    public let category: String
    public let code: String

    private init(category: String, code: String) {
        self.category = category
        self.code = code
    }

    internal static func fromServerDecoded(category: String, code: String) -> CatalogedServerReason {
        CatalogedServerReason(category: category, code: code)
    }
}

public enum ServerReasonCarrierSdkErrorCode: String, Sendable {
    case serverProtocolViolation = "server_protocol_violation"
    case rejectedByServer = "rejected_by_server"
    case serverUnsupportedVersion = "server_unsupported_version"
}

public enum LocalOnlySdkErrorCode: String, Sendable {
    case connectionFailed = "connection_failed"
    case localProtocolViolation = "local_protocol_violation"
    case timeout
    case localUnsupportedVersion = "local_unsupported_version"
    case malformedEvent = "malformed_event"
    case localSerializationError = "local_serialization_error"
}

public enum ArcRtcSdkError: Error, Equatable, Sendable {
    case serverReasonCarrier(ServerReasonCarrierSdkErrorCode, CatalogedServerReason, message: String?)
    case localOnly(LocalOnlySdkErrorCode, message: String?)
    case closedByRemoteServer(CatalogedServerReason, message: String?)
    case closedByRemoteLocalTransport(message: String?)
}

public protocol SignalingCommandProtocol: Sendable {
    var kind: SignalingCommandKind { get }
    var correlationId: CorrelationId { get }
    var contractVersion: SignalingContractVersion { get }
}
