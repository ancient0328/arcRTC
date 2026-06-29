// iOS SDK signaling-only surface の責務別 source shard です。
public struct JoinRoomCommand: SignalingCommandProtocol, Equatable {
    public let kind: SignalingCommandKind = .joinRoom
    public let correlationId: CorrelationId
    public let contractVersion: SignalingContractVersion
    public let roomRef: OpaqueReference?
    public let credentialRef: OpaqueReference?
    public let capabilities: [String]

    public init(
        correlationId: CorrelationId,
        contractVersion: SignalingContractVersion = .v0_2,
        roomRef: OpaqueReference? = nil,
        credentialRef: OpaqueReference? = nil,
        capabilities: [String] = []
    ) {
        self.correlationId = correlationId
        self.contractVersion = contractVersion
        self.roomRef = roomRef
        self.credentialRef = credentialRef
        self.capabilities = capabilities
    }
}

public struct LeaveRoomCommand: SignalingCommandProtocol, Equatable {
    public let kind: SignalingCommandKind = .leaveRoom
    public let correlationId: CorrelationId
    public let contractVersion: SignalingContractVersion
    public let participantRef: OpaqueReference?

    public init(
        correlationId: CorrelationId,
        contractVersion: SignalingContractVersion = .v0_2,
        participantRef: OpaqueReference? = nil
    ) {
        self.correlationId = correlationId
        self.contractVersion = contractVersion
        self.participantRef = participantRef
    }
}

public struct RelayPayloadCommand: SignalingCommandProtocol, Equatable {
    public let kind: SignalingCommandKind
    public let correlationId: CorrelationId
    public let contractVersion: SignalingContractVersion
    public let targetParticipantRef: OpaqueReference
    public let payload: OpaqueSignalingPayload

    private init(
        kind: SignalingCommandKind,
        correlationId: CorrelationId,
        targetParticipantRef: OpaqueReference,
        payload: OpaqueSignalingPayload,
        contractVersion: SignalingContractVersion = .v0_2
    ) {
        self.kind = kind
        self.correlationId = correlationId
        self.targetParticipantRef = targetParticipantRef
        self.payload = payload
        self.contractVersion = contractVersion
    }

    public static func offer(
        correlationId: CorrelationId,
        targetParticipantRef: OpaqueReference,
        offer: OpaqueSignalingPayload,
        contractVersion: SignalingContractVersion = .v0_2
    ) -> RelayPayloadCommand {
        RelayPayloadCommand(
            kind: .sendOffer,
            correlationId: correlationId,
            targetParticipantRef: targetParticipantRef,
            payload: offer,
            contractVersion: contractVersion
        )
    }

    public static func answer(
        correlationId: CorrelationId,
        targetParticipantRef: OpaqueReference,
        answer: OpaqueSignalingPayload,
        contractVersion: SignalingContractVersion = .v0_2
    ) -> RelayPayloadCommand {
        RelayPayloadCommand(
            kind: .sendAnswer,
            correlationId: correlationId,
            targetParticipantRef: targetParticipantRef,
            payload: answer,
            contractVersion: contractVersion
        )
    }

    public static func iceCandidate(
        correlationId: CorrelationId,
        targetParticipantRef: OpaqueReference,
        candidate: OpaqueSignalingPayload,
        contractVersion: SignalingContractVersion = .v0_2
    ) -> RelayPayloadCommand {
        RelayPayloadCommand(
            kind: .sendIceCandidate,
            correlationId: correlationId,
            targetParticipantRef: targetParticipantRef,
            payload: candidate,
            contractVersion: contractVersion
        )
    }
}

public struct RequestTurnCredentialCommand: SignalingCommandProtocol, Equatable {
    public let kind: SignalingCommandKind = .requestTurnCredential
    public let correlationId: CorrelationId
    public let contractVersion: SignalingContractVersion
    public let allocationRef: OpaqueReference?

    public init(
        correlationId: CorrelationId,
        contractVersion: SignalingContractVersion = .v0_2,
        allocationRef: OpaqueReference? = nil
    ) {
        self.correlationId = correlationId
        self.contractVersion = contractVersion
        self.allocationRef = allocationRef
    }
}

public struct AcknowledgeForwardCommand: SignalingCommandProtocol, Equatable {
    public let kind: SignalingCommandKind = .acknowledgeForward
    public let correlationId: CorrelationId
    public let contractVersion: SignalingContractVersion
    public let forwardedEventRef: OpaqueReference

    public init(
        correlationId: CorrelationId,
        forwardedEventRef: OpaqueReference,
        contractVersion: SignalingContractVersion = .v0_2
    ) {
        self.correlationId = correlationId
        self.forwardedEventRef = forwardedEventRef
        self.contractVersion = contractVersion
    }
}

public enum SignalingCommand: Equatable, Sendable {
    case joinRoom(JoinRoomCommand)
    case leaveRoom(LeaveRoomCommand)
    case relay(RelayPayloadCommand)
    case requestTurnCredential(RequestTurnCredentialCommand)
    case acknowledgeForward(AcknowledgeForwardCommand)

    public var kind: SignalingCommandKind {
        switch self {
        case .joinRoom(let command): command.kind
        case .leaveRoom(let command): command.kind
        case .relay(let command): command.kind
        case .requestTurnCredential(let command): command.kind
        case .acknowledgeForward(let command): command.kind
        }
    }

    public var correlationId: CorrelationId {
        switch self {
        case .joinRoom(let command): command.correlationId
        case .leaveRoom(let command): command.correlationId
        case .relay(let command): command.correlationId
        case .requestTurnCredential(let command): command.correlationId
        case .acknowledgeForward(let command): command.correlationId
        }
    }

    public var contractVersion: SignalingContractVersion {
        switch self {
        case .joinRoom(let command): command.contractVersion
        case .leaveRoom(let command): command.contractVersion
        case .relay(let command): command.contractVersion
        case .requestTurnCredential(let command): command.contractVersion
        case .acknowledgeForward(let command): command.contractVersion
        }
    }
}

public enum SignalingCommandResult: Equatable, Sendable {
    case accepted(correlationId: CorrelationId)
    case rejected(correlationId: CorrelationId, serverReason: CatalogedServerReason)
}

public enum SignalingSendOutcome: Equatable, Sendable {
    case result(SignalingCommandResult)
    case error(ArcRtcSdkError)
}

