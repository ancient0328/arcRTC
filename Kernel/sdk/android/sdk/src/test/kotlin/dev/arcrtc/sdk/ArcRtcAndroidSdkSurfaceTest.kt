package dev.arcrtc.sdk

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class ArcRtcAndroidSdkSurfaceTest {
  @Test
  fun signalingOnlyProjectionMatchesAndroidPublicSurface() {
    assertTrue(
      ArcRtcAndroidSdkSurface.assertSignalingOnlyProjection(ArcRtcAndroidSdkSurface.projection),
    )
  }

  @Test
  fun reconnectPolicyRejectsSessionResumptionForCurrentSignalingOnlyScope() {
    assertFalse(
      ArcRtcAndroidSdkSurface.reconnectPolicyAdmitsSessionResumption(
        SdkReconnectPolicy(
          reconnectClass = SdkReconnectClass.SessionResumptionRequested,
          localRetryLimit = 0,
          initialDelayMs = 0,
          maxDelayMs = 0,
        ),
      ),
    )
    assertTrue(
      ArcRtcAndroidSdkSurface.reconnectPolicyAdmitsSessionResumption(
        SdkReconnectPolicy(
          reconnectClass = SdkReconnectClass.LocalTransportReconnect,
          localRetryLimit = 3,
          initialDelayMs = 100,
          maxDelayMs = 1_000,
        ),
      ),
    )
  }

  @Test
  fun commandModelsExposeOnlySignalingContractFields() {
    val correlationId = CorrelationId("android-command-correlation")
    val target = OpaqueReference("android-target-participant")
    val room = OpaqueReference("android-room")
    val credential = OpaqueReference("android-credential")
    val payload = mapOf("opaque" to "payload")

    val commands =
      listOf<SignalingCommand>(
        JoinRoomCommand(
          correlationId = correlationId,
          roomRef = room,
          credentialRef = credential,
          capabilities = listOf("audio", "video"),
        ),
        LeaveRoomCommand(correlationId = correlationId, participantRef = target),
        SendOfferCommand(correlationId = correlationId, targetParticipantRef = target, offer = payload),
        SendAnswerCommand(correlationId = correlationId, targetParticipantRef = target, answer = payload),
        SendIceCandidateCommand(correlationId = correlationId, targetParticipantRef = target, candidate = payload),
        RequestTurnCredentialCommand(correlationId = correlationId, allocationRef = credential),
        AcknowledgeForwardCommand(correlationId = correlationId, forwardedEventRef = OpaqueReference("forwarded-event")),
      )

    assertEquals(SignalingCommandKind.entries, commands.map { it.kind })
    assertTrue(commands.all { it.correlationId == correlationId })
    assertTrue(commands.all { it.contractVersion == SignalingContractVersion.V0_2 })
    assertEquals(listOf("audio", "video"), (commands[0] as JoinRoomCommand).capabilities)
  }

  @Test
  fun eventAndOutcomeModelsPreserveOpaqueReferencesAndServerReasons() {
    val correlationId = CorrelationId("android-event-correlation")
    val participant = OpaqueReference("android-participant")
    val room = OpaqueReference("android-room")
    val credential = OpaqueReference("android-turn-credential")
    val reason = CatalogedServerReason.fromServerDecoded(category = "protocol", code = "unsupported_version")
    val payload = mapOf("opaque" to "payload")

    val events =
      listOf<SignalingEvent>(
        JoinedEvent(correlationId = correlationId, participantRef = participant, roomRef = room),
        RejectedEvent(correlationId = correlationId, serverReason = reason),
        ParticipantJoinedEvent(correlationId = correlationId, participantRef = participant),
        ParticipantLeftEvent(correlationId = correlationId, participantRef = participant),
        OfferReceivedEvent(correlationId = correlationId, fromParticipantRef = participant, offer = payload),
        AnswerReceivedEvent(correlationId = correlationId, fromParticipantRef = participant, answer = payload),
        IceCandidateReceivedEvent(correlationId = correlationId, fromParticipantRef = participant, candidate = payload),
        TurnCredentialAvailableEvent(correlationId = correlationId, credentialRef = credential),
        ProtocolViolationEvent(correlationId = correlationId, serverReason = reason),
      )

    assertEquals(SignalingEventKind.entries, events.map { it.kind })
    assertTrue(events.all { it.correlationId == correlationId })
    assertTrue(events.all { it.contractVersion == SignalingContractVersion.V0_2 })
    assertEquals("protocol", reason.category)
    assertEquals("unsupported_version", reason.code)

    val accepted = SignalingCommandResult.Accepted(correlationId)
    val rejected = SignalingCommandResult.Rejected(correlationId, reason)
    assertEquals(correlationId, accepted.correlationId)
    assertEquals(reason, rejected.serverReason)
    assertEquals(SignalingSendOutcome.Result(accepted), SignalingSendOutcome.Result(accepted))
    assertEquals(
      SignalingSendOutcome.Error(ServerReasonCarrierSdkError(ServerReasonCarrierSdkErrorCode.ServerProtocolViolation, reason, "server rejected")),
      SignalingSendOutcome.Error(ServerReasonCarrierSdkError(ServerReasonCarrierSdkErrorCode.ServerProtocolViolation, reason, "server rejected")),
    )
    assertEquals(SignalingEventCallbackItem.Event(events[0]), SignalingEventCallbackItem.Event(events[0]))
    assertEquals(SignalingEventCallbackItem.Error(LocalOnlySdkError(LocalOnlySdkErrorCode.MalformedEvent, "bad event")), SignalingEventCallbackItem.Error(LocalOnlySdkError(LocalOnlySdkErrorCode.MalformedEvent, "bad event")))
  }

  @Test
  fun sdkErrorAndProjectionValueTypesRemainLocalToSdkSurface() {
    val reason = CatalogedServerReason.fromServerDecoded(category = "policy", code = "room_closed")
    val errors =
      listOf<ArcRtcSdkError>(
        ServerReasonCarrierSdkError(ServerReasonCarrierSdkErrorCode.RejectedByServer, reason, "rejected"),
        LocalOnlySdkError(LocalOnlySdkErrorCode.LocalSerializationError, "serialization"),
        ClosedByRemoteSdkError.Server(reason, "closed"),
        ClosedByRemoteSdkError.LocalTransport("socket closed"),
      )
    assertEquals(listOf("rejected", "serialization", "closed", "socket closed"), errors.map { it.message })

    // SDK projection は public shape の写像であり、core/drivers の意味論を所有しません。
    for (entry in ArcRtcAndroidSdkSurface.projection.entries) {
      assertEquals(SdkCorrelationPropagation.Required, entry.correlationPropagation)
      assertTrue(entry.platformApiSymbol.isNotEmpty())
      assertTrue(entry.publicShape.isNotEmpty())
      when (val source = entry.source) {
        is SdkProjectionSource.Command -> {
          assertEquals(SdkProjectionKindClass.Command, entry.sourceKindClass)
          assertTrue(ArcRtcAndroidSdkSurface.commandKinds.contains(source.kind))
          assertEquals(SdkVersionCapabilityBehavior.SendReceiveServerResult, entry.versionCapabilityBehavior)
        }
        is SdkProjectionSource.Event -> {
          assertEquals(SdkProjectionKindClass.Event, entry.sourceKindClass)
          assertTrue(ArcRtcAndroidSdkSurface.eventKinds.contains(source.kind))
          assertEquals(SdkVersionCapabilityBehavior.PreserveServerResult, entry.versionCapabilityBehavior)
        }
      }
    }
  }
}
