/**
 * TypeScript SDK は Signaling-only public surface です。
 *
 * media、auth issuance、regulated workflow、driver internal API はここで所有しません。
 */
export const arcrtcTypeScriptSdkSurface = "signaling-only" as const;

export const arcrtcTypeScriptSdkPlatform = "typescript" as const;

export const arcrtcSignalingContractVersion = "v0.2" as const;

export type ArcRtcSdkPlatform = typeof arcrtcTypeScriptSdkPlatform;

export type SignalingContractVersion = typeof arcrtcSignalingContractVersion;

export type CorrelationId = string & { readonly __brand: "CorrelationId" };

export type OpaqueReference = string & { readonly __brand: "OpaqueReference" };

export type OpaqueSignalingPayload = Readonly<Record<string, unknown>>;

declare const serverDecodedReasonBrand: unique symbol;

export type SignalingCommandKind =
  | "JoinRoom"
  | "LeaveRoom"
  | "SendOffer"
  | "SendAnswer"
  | "SendIceCandidate"
  | "RequestTurnCredential"
  | "AcknowledgeForward";

export type SignalingEventKind =
  | "Joined"
  | "Rejected"
  | "ParticipantJoined"
  | "ParticipantLeft"
  | "OfferReceived"
  | "AnswerReceived"
  | "IceCandidateReceived"
  | "TurnCredentialAvailable"
  | "ProtocolViolation";

export const signalingCommandKinds = [
  "JoinRoom",
  "LeaveRoom",
  "SendOffer",
  "SendAnswer",
  "SendIceCandidate",
  "RequestTurnCredential",
  "AcknowledgeForward",
] as const satisfies readonly SignalingCommandKind[];

export const signalingEventKinds = [
  "Joined",
  "Rejected",
  "ParticipantJoined",
  "ParticipantLeft",
  "OfferReceived",
  "AnswerReceived",
  "IceCandidateReceived",
  "TurnCredentialAvailable",
  "ProtocolViolation",
] as const satisfies readonly SignalingEventKind[];

export type CatalogedServerReason = {
  readonly category: string;
  readonly code: string;
  readonly [serverDecodedReasonBrand]: "server-decoded";
};

export type ServerReasonCarrierSdkErrorCode =
  | "server_protocol_violation"
  | "rejected_by_server"
  | "server_unsupported_version";

export type LocalOnlySdkErrorCode =
  | "connection_failed"
  | "local_protocol_violation"
  | "timeout"
  | "local_unsupported_version"
  | "malformed_event"
  | "local_serialization_error";

export type ClosedByRemoteSdkError =
  | {
      readonly code: "closed_by_remote";
      readonly origin: "server";
      readonly serverReason: CatalogedServerReason;
      readonly message?: string;
    }
  | {
      readonly code: "closed_by_remote";
      readonly origin: "local_transport";
      readonly serverReason?: never;
      readonly message?: string;
    };

export type ServerReasonCarrierSdkError = {
  readonly code: ServerReasonCarrierSdkErrorCode;
  readonly serverReason: CatalogedServerReason;
  readonly message?: string;
};

export type LocalOnlySdkError = {
  readonly code: LocalOnlySdkErrorCode;
  readonly serverReason?: never;
  readonly message?: string;
};

export type ArcRtcSdkError =
  | ServerReasonCarrierSdkError
  | LocalOnlySdkError
  | ClosedByRemoteSdkError;

export interface SignalingCommandBase<K extends SignalingCommandKind> {
  readonly kind: K;
  readonly correlationId: CorrelationId;
  readonly contractVersion: SignalingContractVersion;
}

export type JoinRoomCommand = SignalingCommandBase<"JoinRoom"> & {
  readonly roomRef?: OpaqueReference;
  readonly credentialRef?: OpaqueReference;
  readonly capabilities?: readonly string[];
};

export type LeaveRoomCommand = SignalingCommandBase<"LeaveRoom"> & {
  readonly participantRef?: OpaqueReference;
};

export type SendOfferCommand = SignalingCommandBase<"SendOffer"> & {
  readonly targetParticipantRef: OpaqueReference;
  readonly offer: OpaqueSignalingPayload;
};

export type SendAnswerCommand = SignalingCommandBase<"SendAnswer"> & {
  readonly targetParticipantRef: OpaqueReference;
  readonly answer: OpaqueSignalingPayload;
};

export type SendIceCandidateCommand = SignalingCommandBase<"SendIceCandidate"> & {
  readonly targetParticipantRef: OpaqueReference;
  readonly candidate: OpaqueSignalingPayload;
};

export type RequestTurnCredentialCommand =
  SignalingCommandBase<"RequestTurnCredential"> & {
    readonly allocationRef?: OpaqueReference;
  };

export type AcknowledgeForwardCommand =
  SignalingCommandBase<"AcknowledgeForward"> & {
    readonly forwardedEventRef: OpaqueReference;
  };

export type SignalingCommand =
  | JoinRoomCommand
  | LeaveRoomCommand
  | SendOfferCommand
  | SendAnswerCommand
  | SendIceCandidateCommand
  | RequestTurnCredentialCommand
  | AcknowledgeForwardCommand;

export type SignalingCommandResult =
  | {
      readonly accepted: true;
      readonly correlationId: CorrelationId;
      readonly serverReason?: never;
    }
  | {
      readonly accepted: false;
      readonly correlationId: CorrelationId;
      readonly serverReason: CatalogedServerReason;
    };

export interface SignalingEventBase<K extends SignalingEventKind> {
  readonly kind: K;
  readonly correlationId: CorrelationId;
  readonly contractVersion: SignalingContractVersion;
}

export type JoinedEvent = SignalingEventBase<"Joined"> & {
  readonly participantRef: OpaqueReference;
  readonly roomRef: OpaqueReference;
};

export type RejectedEvent = SignalingEventBase<"Rejected"> & {
  readonly serverReason: CatalogedServerReason;
};

export type ParticipantJoinedEvent = SignalingEventBase<"ParticipantJoined"> & {
  readonly participantRef: OpaqueReference;
};

export type ParticipantLeftEvent = SignalingEventBase<"ParticipantLeft"> & {
  readonly participantRef: OpaqueReference;
};

export type OfferReceivedEvent = SignalingEventBase<"OfferReceived"> & {
  readonly fromParticipantRef: OpaqueReference;
  readonly offer: OpaqueSignalingPayload;
};

export type AnswerReceivedEvent = SignalingEventBase<"AnswerReceived"> & {
  readonly fromParticipantRef: OpaqueReference;
  readonly answer: OpaqueSignalingPayload;
};

export type IceCandidateReceivedEvent =
  SignalingEventBase<"IceCandidateReceived"> & {
    readonly fromParticipantRef: OpaqueReference;
    readonly candidate: OpaqueSignalingPayload;
  };

export type TurnCredentialAvailableEvent =
  SignalingEventBase<"TurnCredentialAvailable"> & {
    readonly credentialRef: OpaqueReference;
  };

export type ProtocolViolationEvent = SignalingEventBase<"ProtocolViolation"> & {
  readonly serverReason: CatalogedServerReason;
};

export type SignalingEvent =
  | JoinedEvent
  | RejectedEvent
  | ParticipantJoinedEvent
  | ParticipantLeftEvent
  | OfferReceivedEvent
  | AnswerReceivedEvent
  | IceCandidateReceivedEvent
  | TurnCredentialAvailableEvent
  | ProtocolViolationEvent;

export type SdkReconnectClass =
  | "local_transport_reconnect"
  | "signaling_rejoin_command"
  | "event_stream_resubscribe"
  | "session_resumption_requested"
  | "reconnect_exhausted";

export interface SdkReconnectPolicy {
  readonly reconnectClass: SdkReconnectClass;
  readonly localRetryLimit: number;
  readonly backoff: {
    readonly initialDelayMs: number;
    readonly maxDelayMs: number;
  };
  readonly explicitServerCommand?: SignalingCommandKind;
}

export interface ArcRtcSignalingClient {
  readonly platform: ArcRtcSdkPlatform;
  readonly contractVersion: SignalingContractVersion;
  connect(options: {
    readonly endpoint: string;
    readonly correlationId: CorrelationId;
  }): Promise<void>;
  close(options?: { readonly correlationId?: CorrelationId }): Promise<void>;
  send(command: SignalingCommand): Promise<SignalingCommandResult>;
  events(): AsyncIterable<SignalingEvent>;
}

export type SdkOutOfScopeFeature =
  | "PeerConnection"
  | "media_capture"
  | "media_rendering"
  | "screen_share"
  | "recording"
  | "chat"
  | "data_channel_application_semantics"
  | "auth_issuance"
  | "user_account_management"
  | "regulated_workflow"
  | "medical_data_model";

export const sdkOutOfScopeFeatures = [
  "PeerConnection",
  "media_capture",
  "media_rendering",
  "screen_share",
  "recording",
  "chat",
  "data_channel_application_semantics",
  "auth_issuance",
  "user_account_management",
  "regulated_workflow",
  "medical_data_model",
] as const satisfies readonly SdkOutOfScopeFeature[];

export interface SdkProjectionEntry {
  readonly sourceSignalingKind: SignalingCommandKind | SignalingEventKind;
  readonly sourceKindClass: "command" | "event";
  readonly platformApiSymbol: string;
  readonly publicShape: string;
  readonly correlationPropagation: "required";
  readonly serverReasonPreservation: "preserve_category_code" | "not_applicable";
  readonly localSdkErrorWrapper:
    | LocalOnlySdkErrorCode
    | ServerReasonCarrierSdkErrorCode
    | "closed_by_remote"
    | "not_applicable";
  readonly versionCapabilityBehavior:
    | "send_receive_server_result"
    | "preserve_server_result"
    | "not_applicable";
  readonly reconnectRelation: SdkReconnectClass | "not_applicable";
  readonly unsupportedSurfaceBehavior: "sdk_local_error" | "server_rejection";
}

export interface SdkPublicApiProjection {
  readonly platform: ArcRtcSdkPlatform;
  readonly sourceContractVersion: SignalingContractVersion;
  readonly sourceContract: "SIGNALING_CONTRACT_CANONICAL";
  readonly projectionClass: "platform_api_projection";
  readonly generatedArtifactIsSemanticAuthority: false;
  readonly commands: readonly SignalingCommandKind[];
  readonly events: readonly SignalingEventKind[];
  readonly entries: readonly SdkProjectionEntry[];
  readonly outOfScopeFeatures: readonly SdkOutOfScopeFeature[];
}

export const arcrtcTypeScriptSdkProjectionEntries = [
  {
    sourceSignalingKind: "JoinRoom",
    sourceKindClass: "command",
    platformApiSymbol: "ArcRtcSignalingClient.send",
    publicShape: "JoinRoomCommand -> SignalingCommandResult",
    correlationPropagation: "required",
    serverReasonPreservation: "preserve_category_code",
    localSdkErrorWrapper: "rejected_by_server",
    versionCapabilityBehavior: "send_receive_server_result",
    reconnectRelation: "signaling_rejoin_command",
    unsupportedSurfaceBehavior: "server_rejection",
  },
  {
    sourceSignalingKind: "LeaveRoom",
    sourceKindClass: "command",
    platformApiSymbol: "ArcRtcSignalingClient.send",
    publicShape: "LeaveRoomCommand -> SignalingCommandResult",
    correlationPropagation: "required",
    serverReasonPreservation: "preserve_category_code",
    localSdkErrorWrapper: "rejected_by_server",
    versionCapabilityBehavior: "send_receive_server_result",
    reconnectRelation: "not_applicable",
    unsupportedSurfaceBehavior: "server_rejection",
  },
  {
    sourceSignalingKind: "SendOffer",
    sourceKindClass: "command",
    platformApiSymbol: "ArcRtcSignalingClient.send",
    publicShape: "SendOfferCommand -> SignalingCommandResult",
    correlationPropagation: "required",
    serverReasonPreservation: "preserve_category_code",
    localSdkErrorWrapper: "rejected_by_server",
    versionCapabilityBehavior: "send_receive_server_result",
    reconnectRelation: "not_applicable",
    unsupportedSurfaceBehavior: "server_rejection",
  },
  {
    sourceSignalingKind: "SendAnswer",
    sourceKindClass: "command",
    platformApiSymbol: "ArcRtcSignalingClient.send",
    publicShape: "SendAnswerCommand -> SignalingCommandResult",
    correlationPropagation: "required",
    serverReasonPreservation: "preserve_category_code",
    localSdkErrorWrapper: "rejected_by_server",
    versionCapabilityBehavior: "send_receive_server_result",
    reconnectRelation: "not_applicable",
    unsupportedSurfaceBehavior: "server_rejection",
  },
  {
    sourceSignalingKind: "SendIceCandidate",
    sourceKindClass: "command",
    platformApiSymbol: "ArcRtcSignalingClient.send",
    publicShape: "SendIceCandidateCommand -> SignalingCommandResult",
    correlationPropagation: "required",
    serverReasonPreservation: "preserve_category_code",
    localSdkErrorWrapper: "rejected_by_server",
    versionCapabilityBehavior: "send_receive_server_result",
    reconnectRelation: "not_applicable",
    unsupportedSurfaceBehavior: "server_rejection",
  },
  {
    sourceSignalingKind: "RequestTurnCredential",
    sourceKindClass: "command",
    platformApiSymbol: "ArcRtcSignalingClient.send",
    publicShape: "RequestTurnCredentialCommand -> SignalingCommandResult",
    correlationPropagation: "required",
    serverReasonPreservation: "preserve_category_code",
    localSdkErrorWrapper: "rejected_by_server",
    versionCapabilityBehavior: "send_receive_server_result",
    reconnectRelation: "not_applicable",
    unsupportedSurfaceBehavior: "server_rejection",
  },
  {
    sourceSignalingKind: "AcknowledgeForward",
    sourceKindClass: "command",
    platformApiSymbol: "ArcRtcSignalingClient.send",
    publicShape: "AcknowledgeForwardCommand -> SignalingCommandResult",
    correlationPropagation: "required",
    serverReasonPreservation: "preserve_category_code",
    localSdkErrorWrapper: "rejected_by_server",
    versionCapabilityBehavior: "send_receive_server_result",
    reconnectRelation: "not_applicable",
    unsupportedSurfaceBehavior: "server_rejection",
  },
  {
    sourceSignalingKind: "Joined",
    sourceKindClass: "event",
    platformApiSymbol: "ArcRtcSignalingClient.events",
    publicShape: "AsyncIterable<JoinedEvent>",
    correlationPropagation: "required",
    serverReasonPreservation: "not_applicable",
    localSdkErrorWrapper: "malformed_event",
    versionCapabilityBehavior: "preserve_server_result",
    reconnectRelation: "event_stream_resubscribe",
    unsupportedSurfaceBehavior: "sdk_local_error",
  },
  {
    sourceSignalingKind: "Rejected",
    sourceKindClass: "event",
    platformApiSymbol: "ArcRtcSignalingClient.events",
    publicShape: "AsyncIterable<RejectedEvent>",
    correlationPropagation: "required",
    serverReasonPreservation: "preserve_category_code",
    localSdkErrorWrapper: "rejected_by_server",
    versionCapabilityBehavior: "preserve_server_result",
    reconnectRelation: "event_stream_resubscribe",
    unsupportedSurfaceBehavior: "server_rejection",
  },
  {
    sourceSignalingKind: "ParticipantJoined",
    sourceKindClass: "event",
    platformApiSymbol: "ArcRtcSignalingClient.events",
    publicShape: "AsyncIterable<ParticipantJoinedEvent>",
    correlationPropagation: "required",
    serverReasonPreservation: "not_applicable",
    localSdkErrorWrapper: "malformed_event",
    versionCapabilityBehavior: "preserve_server_result",
    reconnectRelation: "event_stream_resubscribe",
    unsupportedSurfaceBehavior: "sdk_local_error",
  },
  {
    sourceSignalingKind: "ParticipantLeft",
    sourceKindClass: "event",
    platformApiSymbol: "ArcRtcSignalingClient.events",
    publicShape: "AsyncIterable<ParticipantLeftEvent>",
    correlationPropagation: "required",
    serverReasonPreservation: "not_applicable",
    localSdkErrorWrapper: "malformed_event",
    versionCapabilityBehavior: "preserve_server_result",
    reconnectRelation: "event_stream_resubscribe",
    unsupportedSurfaceBehavior: "sdk_local_error",
  },
  {
    sourceSignalingKind: "OfferReceived",
    sourceKindClass: "event",
    platformApiSymbol: "ArcRtcSignalingClient.events",
    publicShape: "AsyncIterable<OfferReceivedEvent>",
    correlationPropagation: "required",
    serverReasonPreservation: "not_applicable",
    localSdkErrorWrapper: "malformed_event",
    versionCapabilityBehavior: "preserve_server_result",
    reconnectRelation: "event_stream_resubscribe",
    unsupportedSurfaceBehavior: "sdk_local_error",
  },
  {
    sourceSignalingKind: "AnswerReceived",
    sourceKindClass: "event",
    platformApiSymbol: "ArcRtcSignalingClient.events",
    publicShape: "AsyncIterable<AnswerReceivedEvent>",
    correlationPropagation: "required",
    serverReasonPreservation: "not_applicable",
    localSdkErrorWrapper: "malformed_event",
    versionCapabilityBehavior: "preserve_server_result",
    reconnectRelation: "event_stream_resubscribe",
    unsupportedSurfaceBehavior: "sdk_local_error",
  },
  {
    sourceSignalingKind: "IceCandidateReceived",
    sourceKindClass: "event",
    platformApiSymbol: "ArcRtcSignalingClient.events",
    publicShape: "AsyncIterable<IceCandidateReceivedEvent>",
    correlationPropagation: "required",
    serverReasonPreservation: "not_applicable",
    localSdkErrorWrapper: "malformed_event",
    versionCapabilityBehavior: "preserve_server_result",
    reconnectRelation: "event_stream_resubscribe",
    unsupportedSurfaceBehavior: "sdk_local_error",
  },
  {
    sourceSignalingKind: "TurnCredentialAvailable",
    sourceKindClass: "event",
    platformApiSymbol: "ArcRtcSignalingClient.events",
    publicShape: "AsyncIterable<TurnCredentialAvailableEvent>",
    correlationPropagation: "required",
    serverReasonPreservation: "not_applicable",
    localSdkErrorWrapper: "malformed_event",
    versionCapabilityBehavior: "preserve_server_result",
    reconnectRelation: "event_stream_resubscribe",
    unsupportedSurfaceBehavior: "sdk_local_error",
  },
  {
    sourceSignalingKind: "ProtocolViolation",
    sourceKindClass: "event",
    platformApiSymbol: "ArcRtcSignalingClient.events",
    publicShape: "AsyncIterable<ProtocolViolationEvent>",
    correlationPropagation: "required",
    serverReasonPreservation: "preserve_category_code",
    localSdkErrorWrapper: "server_protocol_violation",
    versionCapabilityBehavior: "preserve_server_result",
    reconnectRelation: "event_stream_resubscribe",
    unsupportedSurfaceBehavior: "server_rejection",
  },
] as const satisfies readonly SdkProjectionEntry[];

export const arcrtcTypeScriptSdkProjection: SdkPublicApiProjection = {
  platform: arcrtcTypeScriptSdkPlatform,
  sourceContractVersion: arcrtcSignalingContractVersion,
  sourceContract: "SIGNALING_CONTRACT_CANONICAL",
  projectionClass: "platform_api_projection",
  generatedArtifactIsSemanticAuthority: false,
  commands: signalingCommandKinds,
  events: signalingEventKinds,
  entries: arcrtcTypeScriptSdkProjectionEntries,
  outOfScopeFeatures: sdkOutOfScopeFeatures,
};

export function isServerReasonCarrierSdkErrorCode(
  code: ArcRtcSdkError["code"],
): code is ServerReasonCarrierSdkErrorCode {
  return (
    code === "server_protocol_violation" ||
    code === "rejected_by_server" ||
    code === "server_unsupported_version"
  );
}

export function createServerBackedSdkError(
  code: ServerReasonCarrierSdkErrorCode,
  serverReason: CatalogedServerReason,
  message?: string,
): ServerReasonCarrierSdkError {
  return { code, serverReason, message };
}

export function createLocalSdkError(
  code: LocalOnlySdkErrorCode,
  message?: string,
): LocalOnlySdkError {
  return { code, message };
}

export function reconnectPolicyAdmitsSessionResumption(
  policy: SdkReconnectPolicy,
): boolean {
  return policy.reconnectClass !== "session_resumption_requested";
}

export function assertSignalingOnlyProjection(
  projection: SdkPublicApiProjection,
): boolean {
  const projectedKinds = projection.entries.map((entry) => entry.sourceSignalingKind);

  return (
    projection.sourceContract === "SIGNALING_CONTRACT_CANONICAL" &&
    projection.generatedArtifactIsSemanticAuthority === false &&
    sameStringSet(projection.commands, signalingCommandKinds) &&
    sameStringSet(projection.events, signalingEventKinds) &&
    sameStringSet(projectedKinds, [...signalingCommandKinds, ...signalingEventKinds]) &&
    projection.entries.length === signalingCommandKinds.length + signalingEventKinds.length &&
    sameStringSet(projection.outOfScopeFeatures, sdkOutOfScopeFeatures)
  );
}

function sameStringSet(
  actual: readonly string[],
  expected: readonly string[],
): boolean {
  const actualSet = new Set(actual);
  return (
    actual.length === expected.length &&
    actualSet.size === expected.length &&
    expected.every((value) => actualSet.has(value))
  );
}
