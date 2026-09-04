package dev.arcrtc.sdk

/**
 * Android SDK は Signaling-only public surface です。
 *
 * media、auth issuance、regulated workflow、driver internal API はこの package で所有しません。
 */
object ArcRtcAndroidSdkSurface {
  const val boundary: String = "signaling-only"
  val platform: SdkPlatform = SdkPlatform.Android
  val signalingContractVersion: SignalingContractVersion = SignalingContractVersion.V0_2

  val commandKinds: Set<SignalingCommandKind> = SignalingCommandKind.entries.toSet()
  val eventKinds: Set<SignalingEventKind> = SignalingEventKind.entries.toSet()
  val outOfScopeFeatures: Set<SdkOutOfScopeFeature> = SdkOutOfScopeFeature.entries.toSet()

  val projection: SdkPublicApiProjection =
    SdkPublicApiProjection(
      platform = platform,
      sourceProtocolVersion = signalingContractVersion,
      sourceProtocol = SdkSourceProtocol.SignalingProtocol,
      projectionClass = SdkProjectionClass.PlatformApiProjection,
      generatedArtifactIsSemanticAuthority = false,
      commands = commandKinds,
      events = eventKinds,
      entries = androidProjectionEntries,
      outOfScopeFeatures = outOfScopeFeatures,
    )

  fun reconnectPolicyAdmitsSessionResumption(policy: SdkReconnectPolicy): Boolean =
    policy.reconnectClass != SdkReconnectClass.SessionResumptionRequested

  fun assertSignalingOnlyProjection(projection: SdkPublicApiProjection): Boolean {
    val commandProjectedKinds =
      projection.entries
        .filter { it.source is SdkProjectionSource.Command && it.sourceKindClass == SdkProjectionKindClass.Command }
        .map { it.source.name }
        .toSet()
    val eventProjectedKinds =
      projection.entries
        .filter { it.source is SdkProjectionSource.Event && it.sourceKindClass == SdkProjectionKindClass.Event }
        .map { it.source.name }
        .toSet()
    val expectedCommands = commandKinds.map { it.name }.toSet()
    val expectedEvents = eventKinds.map { it.name }.toSet()
    val entryClassesAreConsistent =
      projection.entries.all {
        when (it.source) {
          is SdkProjectionSource.Command -> it.sourceKindClass == SdkProjectionKindClass.Command
          is SdkProjectionSource.Event -> it.sourceKindClass == SdkProjectionKindClass.Event
        }
      }

    return projection.platform == SdkPlatform.Android &&
      projection.sourceProtocolVersion == SignalingContractVersion.V0_2 &&
      projection.sourceProtocol == SdkSourceProtocol.SignalingProtocol &&
      projection.projectionClass == SdkProjectionClass.PlatformApiProjection &&
      !projection.generatedArtifactIsSemanticAuthority &&
      projection.commands == commandKinds &&
      projection.events == eventKinds &&
      projection.entries.size == commandKinds.size + eventKinds.size &&
      entryClassesAreConsistent &&
      commandProjectedKinds == expectedCommands &&
      eventProjectedKinds == expectedEvents &&
      projection.outOfScopeFeatures == outOfScopeFeatures
  }
}

const val ARCRTC_SIGNALING_PROTOCOL_VERSION: String = "v0.2"
const val ARCRTC_SDK_SEMVER: String = "0.2.0"
const val ARCRTC_SDK_PACKAGE_NAME: String = "dev.arcrtc.sdk"

enum class SdkPlatform {
  Android,
}

enum class SignalingContractVersion {
  V0_2,
}

enum class SdkSourceProtocol {
  SignalingProtocol,
}

enum class SdkProjectionClass {
  PlatformApiProjection,
}

@JvmInline
value class CorrelationId(val value: String)

@JvmInline
value class OpaqueReference(val value: String)

typealias OpaqueSignalingPayload = Map<String, Any?>

// SDK は core reason の権威を持たず、公開 failure code を閉集合で示すだけです。
enum class ArcRtcSdkFailureCode {
  CredentialRejected,
  JoinRejected,
  MembershipConflict,
  NegotiationRejected,
  IceCandidateRejected,
  Timeout,
  TransportUnavailable,
  ProtocolViolation,
  VersionMismatch,
  InternalInvariantViolation,
}

data class ArcRtcJoinCommand(
  val roomId: String,
  val participantId: String,
  val credentialRef: String,
  val sessionRef: String? = null,
)

data class ArcRtcLeaveCommand(
  val roomId: String,
  val participantId: String,
  val sessionRef: String? = null,
)

data class ArcRtcOfferCommand(
  val roomId: String,
  val participantId: String,
  val sdpRef: String,
  val sessionRef: String? = null,
)

data class ArcRtcAnswerCommand(
  val roomId: String,
  val participantId: String,
  val sdpRef: String,
  val sessionRef: String? = null,
)

data class ArcRtcIceCandidateCommand(
  val roomId: String,
  val participantId: String,
  val candidateRef: String,
  val sessionRef: String? = null,
)

data class ArcRtcReconnectCommand(
  val roomId: String,
  val participantId: String,
  val sessionRef: String,
)

data class ArcRtcJoinAcceptedEvent(
  val roomId: String,
  val participantId: String,
  val sessionRef: String,
)

data class ArcRtcJoinRejectedEvent(
  val roomId: String,
  val reasonCode: String,
  val participantId: String? = null,
)

data class ArcRtcParticipantLeftEvent(
  val roomId: String,
  val participantId: String,
  val sessionRef: String? = null,
)

data class ArcRtcNegotiationRequiredEvent(
  val roomId: String,
  val participantId: String,
  val sessionRef: String,
  val sdpRef: String? = null,
)

data class ArcRtcIceCandidateReceivedEvent(
  val roomId: String,
  val participantId: String,
  val candidateRef: String,
  val sessionRef: String,
)

data class ArcRtcSessionTimedOutEvent(
  val roomId: String,
  val participantId: String,
  val sessionRef: String,
  val reasonCode: String,
)

enum class SignalingCommandKind {
  JoinRoom,
  LeaveRoom,
  SendOffer,
  SendAnswer,
  SendIceCandidate,
  RequestTurnCredential,
  AcknowledgeForward,
}

enum class SignalingEventKind {
  Joined,
  Rejected,
  ParticipantJoined,
  ParticipantLeft,
  OfferReceived,
  AnswerReceived,
  IceCandidateReceived,
  TurnCredentialAvailable,
  ProtocolViolation,
}

class CatalogedServerReason private constructor(
  val category: String,
  val code: String,
) {
  companion object {
    internal fun fromServerDecoded(category: String, code: String): CatalogedServerReason =
      CatalogedServerReason(category, code)
  }
}

enum class ServerReasonCarrierSdkErrorCode {
  ServerProtocolViolation,
  RejectedByServer,
  ServerUnsupportedVersion,
}

enum class LocalOnlySdkErrorCode {
  ConnectionFailed,
  LocalProtocolViolation,
  Timeout,
  LocalUnsupportedVersion,
  MalformedEvent,
  LocalSerializationError,
}

sealed interface ArcRtcSdkError {
  val message: String?
}

data class ServerReasonCarrierSdkError(
  val code: ServerReasonCarrierSdkErrorCode,
  val serverReason: CatalogedServerReason,
  override val message: String? = null,
) : ArcRtcSdkError

data class LocalOnlySdkError(
  val code: LocalOnlySdkErrorCode,
  override val message: String? = null,
) : ArcRtcSdkError

sealed interface ClosedByRemoteSdkError : ArcRtcSdkError {
  data class Server(
    val serverReason: CatalogedServerReason,
    override val message: String? = null,
  ) : ClosedByRemoteSdkError

  data class LocalTransport(
    override val message: String? = null,
  ) : ClosedByRemoteSdkError
}

sealed interface SignalingCommand {
  val kind: SignalingCommandKind
  val correlationId: CorrelationId
  val contractVersion: SignalingContractVersion
}

data class JoinRoomCommand(
  override val correlationId: CorrelationId,
  override val contractVersion: SignalingContractVersion = ArcRtcAndroidSdkSurface.signalingContractVersion,
  val roomRef: OpaqueReference? = null,
  val credentialRef: OpaqueReference? = null,
  val capabilities: List<String> = emptyList(),
) : SignalingCommand {
  override val kind: SignalingCommandKind = SignalingCommandKind.JoinRoom
}

data class LeaveRoomCommand(
  override val correlationId: CorrelationId,
  override val contractVersion: SignalingContractVersion = ArcRtcAndroidSdkSurface.signalingContractVersion,
  val participantRef: OpaqueReference? = null,
) : SignalingCommand {
  override val kind: SignalingCommandKind = SignalingCommandKind.LeaveRoom
}

data class SendOfferCommand(
  override val correlationId: CorrelationId,
  val targetParticipantRef: OpaqueReference,
  val offer: OpaqueSignalingPayload,
  override val contractVersion: SignalingContractVersion = ArcRtcAndroidSdkSurface.signalingContractVersion,
) : SignalingCommand {
  override val kind: SignalingCommandKind = SignalingCommandKind.SendOffer
}

data class SendAnswerCommand(
  override val correlationId: CorrelationId,
  val targetParticipantRef: OpaqueReference,
  val answer: OpaqueSignalingPayload,
  override val contractVersion: SignalingContractVersion = ArcRtcAndroidSdkSurface.signalingContractVersion,
) : SignalingCommand {
  override val kind: SignalingCommandKind = SignalingCommandKind.SendAnswer
}

data class SendIceCandidateCommand(
  override val correlationId: CorrelationId,
  val targetParticipantRef: OpaqueReference,
  val candidate: OpaqueSignalingPayload,
  override val contractVersion: SignalingContractVersion = ArcRtcAndroidSdkSurface.signalingContractVersion,
) : SignalingCommand {
  override val kind: SignalingCommandKind = SignalingCommandKind.SendIceCandidate
}

data class RequestTurnCredentialCommand(
  override val correlationId: CorrelationId,
  override val contractVersion: SignalingContractVersion = ArcRtcAndroidSdkSurface.signalingContractVersion,
  val allocationRef: OpaqueReference? = null,
) : SignalingCommand {
  override val kind: SignalingCommandKind = SignalingCommandKind.RequestTurnCredential
}

data class AcknowledgeForwardCommand(
  override val correlationId: CorrelationId,
  val forwardedEventRef: OpaqueReference,
  override val contractVersion: SignalingContractVersion = ArcRtcAndroidSdkSurface.signalingContractVersion,
) : SignalingCommand {
  override val kind: SignalingCommandKind = SignalingCommandKind.AcknowledgeForward
}

sealed interface SignalingCommandResult {
  val correlationId: CorrelationId

  data class Accepted(
    override val correlationId: CorrelationId,
  ) : SignalingCommandResult

  data class Rejected(
    override val correlationId: CorrelationId,
    val serverReason: CatalogedServerReason,
  ) : SignalingCommandResult
}

sealed interface SignalingEvent {
  val kind: SignalingEventKind
  val correlationId: CorrelationId
  val contractVersion: SignalingContractVersion
}

data class JoinedEvent(
  override val correlationId: CorrelationId,
  val participantRef: OpaqueReference,
  val roomRef: OpaqueReference,
  override val contractVersion: SignalingContractVersion = ArcRtcAndroidSdkSurface.signalingContractVersion,
) : SignalingEvent {
  override val kind: SignalingEventKind = SignalingEventKind.Joined
}

data class RejectedEvent(
  override val correlationId: CorrelationId,
  val serverReason: CatalogedServerReason,
  override val contractVersion: SignalingContractVersion = ArcRtcAndroidSdkSurface.signalingContractVersion,
) : SignalingEvent {
  override val kind: SignalingEventKind = SignalingEventKind.Rejected
}

data class ParticipantJoinedEvent(
  override val correlationId: CorrelationId,
  val participantRef: OpaqueReference,
  override val contractVersion: SignalingContractVersion = ArcRtcAndroidSdkSurface.signalingContractVersion,
) : SignalingEvent {
  override val kind: SignalingEventKind = SignalingEventKind.ParticipantJoined
}

data class ParticipantLeftEvent(
  override val correlationId: CorrelationId,
  val participantRef: OpaqueReference,
  override val contractVersion: SignalingContractVersion = ArcRtcAndroidSdkSurface.signalingContractVersion,
) : SignalingEvent {
  override val kind: SignalingEventKind = SignalingEventKind.ParticipantLeft
}

data class OfferReceivedEvent(
  override val correlationId: CorrelationId,
  val fromParticipantRef: OpaqueReference,
  val offer: OpaqueSignalingPayload,
  override val contractVersion: SignalingContractVersion = ArcRtcAndroidSdkSurface.signalingContractVersion,
) : SignalingEvent {
  override val kind: SignalingEventKind = SignalingEventKind.OfferReceived
}

data class AnswerReceivedEvent(
  override val correlationId: CorrelationId,
  val fromParticipantRef: OpaqueReference,
  val answer: OpaqueSignalingPayload,
  override val contractVersion: SignalingContractVersion = ArcRtcAndroidSdkSurface.signalingContractVersion,
) : SignalingEvent {
  override val kind: SignalingEventKind = SignalingEventKind.AnswerReceived
}

data class IceCandidateReceivedEvent(
  override val correlationId: CorrelationId,
  val fromParticipantRef: OpaqueReference,
  val candidate: OpaqueSignalingPayload,
  override val contractVersion: SignalingContractVersion = ArcRtcAndroidSdkSurface.signalingContractVersion,
) : SignalingEvent {
  override val kind: SignalingEventKind = SignalingEventKind.IceCandidateReceived
}

data class TurnCredentialAvailableEvent(
  override val correlationId: CorrelationId,
  val credentialRef: OpaqueReference,
  override val contractVersion: SignalingContractVersion = ArcRtcAndroidSdkSurface.signalingContractVersion,
) : SignalingEvent {
  override val kind: SignalingEventKind = SignalingEventKind.TurnCredentialAvailable
}

data class ProtocolViolationEvent(
  override val correlationId: CorrelationId,
  val serverReason: CatalogedServerReason,
  override val contractVersion: SignalingContractVersion = ArcRtcAndroidSdkSurface.signalingContractVersion,
) : SignalingEvent {
  override val kind: SignalingEventKind = SignalingEventKind.ProtocolViolation
}

enum class SdkReconnectClass {
  LocalTransportReconnect,
  SignalingRejoinCommand,
  EventStreamResubscribe,
  SessionResumptionRequested,
  ReconnectExhausted,
}

data class SdkReconnectPolicy(
  val reconnectClass: SdkReconnectClass,
  val localRetryLimit: Int,
  val initialDelayMs: Long,
  val maxDelayMs: Long,
  val explicitServerCommand: SignalingCommandKind? = null,
)

interface ArcRtcSignalingClient {
  val platform: SdkPlatform
  val contractVersion: SignalingContractVersion

  fun connect(endpoint: String, correlationId: CorrelationId, callback: (ArcRtcSdkError?) -> Unit)

  fun close(correlationId: CorrelationId? = null, callback: (ArcRtcSdkError?) -> Unit)

  fun send(command: SignalingCommand, callback: (SignalingSendOutcome) -> Unit)

  fun setEventHandler(handler: (SignalingEventCallbackItem) -> Unit)
}

sealed interface SignalingSendOutcome {
  data class Result(val result: SignalingCommandResult) : SignalingSendOutcome
  data class Error(val error: ArcRtcSdkError) : SignalingSendOutcome
}

sealed interface SignalingEventCallbackItem {
  data class Event(val event: SignalingEvent) : SignalingEventCallbackItem
  data class Error(val error: ArcRtcSdkError) : SignalingEventCallbackItem
}

enum class SdkOutOfScopeFeature {
  PeerConnection,
  MediaCapture,
  MediaRendering,
  ScreenShare,
  Recording,
  Chat,
  DataChannelApplicationSemantics,
  AuthIssuance,
  UserAccountManagement,
  RegulatedWorkflow,
  MedicalDataModel,
}

data class SdkProjectionEntry(
  val source: SdkProjectionSource,
  val sourceKindClass: SdkProjectionKindClass,
  val platformApiSymbol: String,
  val publicShape: String,
  val correlationPropagation: SdkCorrelationPropagation,
  val serverReasonPreservation: SdkServerReasonPreservation,
  val localSdkErrorWrapper: SdkProjectionLocalErrorWrapper,
  val versionCapabilityBehavior: SdkVersionCapabilityBehavior,
  val reconnectRelation: SdkReconnectRelation,
  val unsupportedSurfaceBehavior: SdkUnsupportedSurfaceBehavior,
)

sealed interface SdkProjectionSource {
  val name: String

  data class Command(val kind: SignalingCommandKind) : SdkProjectionSource {
    override val name: String = kind.name
  }

  data class Event(val kind: SignalingEventKind) : SdkProjectionSource {
    override val name: String = kind.name
  }
}

enum class SdkProjectionKindClass {
  Command,
  Event,
}

enum class SdkCorrelationPropagation {
  Required,
}

enum class SdkServerReasonPreservation {
  PreserveCategoryCode,
  NotApplicable,
}

enum class SdkProjectionLocalErrorWrapper {
  RejectedByServer,
  ServerProtocolViolation,
  MalformedEvent,
}

enum class SdkVersionCapabilityBehavior {
  SendReceiveServerResult,
  PreserveServerResult,
}

enum class SdkReconnectRelation {
  SignalingRejoinCommand,
  EventStreamResubscribe,
  NotApplicable,
}

enum class SdkUnsupportedSurfaceBehavior {
  SdkLocalError,
  ServerRejection,
}

data class SdkPublicApiProjection(
  val platform: SdkPlatform,
  val sourceProtocolVersion: SignalingContractVersion,
  val sourceProtocol: SdkSourceProtocol,
  val projectionClass: SdkProjectionClass,
  val generatedArtifactIsSemanticAuthority: Boolean,
  val commands: Set<SignalingCommandKind>,
  val events: Set<SignalingEventKind>,
  val entries: List<SdkProjectionEntry>,
  val outOfScopeFeatures: Set<SdkOutOfScopeFeature>,
)

val androidProjectionEntries: List<SdkProjectionEntry> =
  listOf(
    commandProjection(SignalingCommandKind.JoinRoom, "JoinRoomCommand -> SignalingSendOutcome", SdkReconnectRelation.SignalingRejoinCommand),
    commandProjection(SignalingCommandKind.LeaveRoom, "LeaveRoomCommand -> SignalingSendOutcome", SdkReconnectRelation.NotApplicable),
    commandProjection(SignalingCommandKind.SendOffer, "SendOfferCommand -> SignalingSendOutcome", SdkReconnectRelation.NotApplicable),
    commandProjection(SignalingCommandKind.SendAnswer, "SendAnswerCommand -> SignalingSendOutcome", SdkReconnectRelation.NotApplicable),
    commandProjection(SignalingCommandKind.SendIceCandidate, "SendIceCandidateCommand -> SignalingSendOutcome", SdkReconnectRelation.NotApplicable),
    commandProjection(SignalingCommandKind.RequestTurnCredential, "RequestTurnCredentialCommand -> SignalingSendOutcome", SdkReconnectRelation.NotApplicable),
    commandProjection(SignalingCommandKind.AcknowledgeForward, "AcknowledgeForwardCommand -> SignalingSendOutcome", SdkReconnectRelation.NotApplicable),
    eventProjection(SignalingEventKind.Joined, "callback(JoinedEvent)", SdkServerReasonPreservation.NotApplicable, SdkProjectionLocalErrorWrapper.MalformedEvent),
    eventProjection(SignalingEventKind.Rejected, "callback(RejectedEvent)", SdkServerReasonPreservation.PreserveCategoryCode, SdkProjectionLocalErrorWrapper.RejectedByServer),
    eventProjection(SignalingEventKind.ParticipantJoined, "callback(ParticipantJoinedEvent)", SdkServerReasonPreservation.NotApplicable, SdkProjectionLocalErrorWrapper.MalformedEvent),
    eventProjection(SignalingEventKind.ParticipantLeft, "callback(ParticipantLeftEvent)", SdkServerReasonPreservation.NotApplicable, SdkProjectionLocalErrorWrapper.MalformedEvent),
    eventProjection(SignalingEventKind.OfferReceived, "callback(OfferReceivedEvent)", SdkServerReasonPreservation.NotApplicable, SdkProjectionLocalErrorWrapper.MalformedEvent),
    eventProjection(SignalingEventKind.AnswerReceived, "callback(AnswerReceivedEvent)", SdkServerReasonPreservation.NotApplicable, SdkProjectionLocalErrorWrapper.MalformedEvent),
    eventProjection(SignalingEventKind.IceCandidateReceived, "callback(IceCandidateReceivedEvent)", SdkServerReasonPreservation.NotApplicable, SdkProjectionLocalErrorWrapper.MalformedEvent),
    eventProjection(SignalingEventKind.TurnCredentialAvailable, "callback(TurnCredentialAvailableEvent)", SdkServerReasonPreservation.NotApplicable, SdkProjectionLocalErrorWrapper.MalformedEvent),
    eventProjection(SignalingEventKind.ProtocolViolation, "callback(ProtocolViolationEvent)", SdkServerReasonPreservation.PreserveCategoryCode, SdkProjectionLocalErrorWrapper.ServerProtocolViolation),
  )

private fun commandProjection(
  kind: SignalingCommandKind,
  shape: String,
  reconnectRelation: SdkReconnectRelation,
): SdkProjectionEntry =
  SdkProjectionEntry(
    source = SdkProjectionSource.Command(kind),
    sourceKindClass = SdkProjectionKindClass.Command,
    platformApiSymbol = "ArcRtcSignalingClient.send",
    publicShape = shape,
    correlationPropagation = SdkCorrelationPropagation.Required,
    serverReasonPreservation = SdkServerReasonPreservation.PreserveCategoryCode,
    localSdkErrorWrapper = SdkProjectionLocalErrorWrapper.RejectedByServer,
    versionCapabilityBehavior = SdkVersionCapabilityBehavior.SendReceiveServerResult,
    reconnectRelation = reconnectRelation,
    unsupportedSurfaceBehavior = SdkUnsupportedSurfaceBehavior.ServerRejection,
  )

private fun eventProjection(
  kind: SignalingEventKind,
  shape: String,
  serverReasonPreservation: SdkServerReasonPreservation,
  localSdkErrorWrapper: SdkProjectionLocalErrorWrapper,
): SdkProjectionEntry =
  SdkProjectionEntry(
    source = SdkProjectionSource.Event(kind),
    sourceKindClass = SdkProjectionKindClass.Event,
    platformApiSymbol = "ArcRtcSignalingClient.setEventHandler",
    publicShape = shape,
    correlationPropagation = SdkCorrelationPropagation.Required,
    serverReasonPreservation = serverReasonPreservation,
    localSdkErrorWrapper = localSdkErrorWrapper,
    versionCapabilityBehavior = SdkVersionCapabilityBehavior.PreserveServerResult,
    reconnectRelation = SdkReconnectRelation.EventStreamResubscribe,
    unsupportedSurfaceBehavior =
      if (serverReasonPreservation == SdkServerReasonPreservation.PreserveCategoryCode) {
        SdkUnsupportedSurfaceBehavior.ServerRejection
      } else {
        SdkUnsupportedSurfaceBehavior.SdkLocalError
      },
  )
