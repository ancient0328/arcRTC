package io.arcrtc.sdk

import dev.arcrtc.sdk.ARCRTC_SDK_PACKAGE_NAME
import dev.arcrtc.sdk.ARCRTC_SDK_SEMVER
import dev.arcrtc.sdk.ARCRTC_SIGNALING_PROTOCOL_VERSION
import dev.arcrtc.sdk.ArcRtcAnswerCommand
import dev.arcrtc.sdk.ArcRtcIceCandidateCommand
import dev.arcrtc.sdk.ArcRtcIceCandidateReceivedEvent
import dev.arcrtc.sdk.ArcRtcJoinAcceptedEvent
import dev.arcrtc.sdk.ArcRtcJoinCommand
import dev.arcrtc.sdk.ArcRtcJoinRejectedEvent
import dev.arcrtc.sdk.ArcRtcLeaveCommand
import dev.arcrtc.sdk.ArcRtcNegotiationRequiredEvent
import dev.arcrtc.sdk.ArcRtcOfferCommand
import dev.arcrtc.sdk.ArcRtcParticipantLeftEvent
import dev.arcrtc.sdk.ArcRtcReconnectCommand
import dev.arcrtc.sdk.ArcRtcSdkFailureCode
import dev.arcrtc.sdk.ArcRtcSessionTimedOutEvent
import kotlin.test.Test
import kotlin.test.assertEquals

class ArcRtcSdkContractTest {
  @Test
  fun androidSdkExposesFixedPublicCommandNamesAndFields() {
    val commands =
      listOf(
        ArcRtcJoinCommand(
          roomId = "room-a",
          participantId = "participant-a",
          credentialRef = "credential-a",
          sessionRef = "session-a",
        ),
        ArcRtcLeaveCommand(
          roomId = "room-a",
          participantId = "participant-a",
          sessionRef = "session-a",
        ),
        ArcRtcOfferCommand(
          roomId = "room-a",
          participantId = "participant-a",
          sdpRef = "sdp-a",
          sessionRef = "session-a",
        ),
        ArcRtcAnswerCommand(
          roomId = "room-a",
          participantId = "participant-a",
          sdpRef = "sdp-a",
          sessionRef = "session-a",
        ),
        ArcRtcIceCandidateCommand(
          roomId = "room-a",
          participantId = "participant-a",
          candidateRef = "candidate-a",
          sessionRef = "session-a",
        ),
        ArcRtcReconnectCommand(
          roomId = "room-a",
          participantId = "participant-a",
          sessionRef = "session-a",
        ),
      )

    // SDK は公開 Signaling DTO の名前と field を固定し、domain 権威は持ちません。
    assertEquals(
      listOf(
        "ArcRtcJoinCommand",
        "ArcRtcLeaveCommand",
        "ArcRtcOfferCommand",
        "ArcRtcAnswerCommand",
        "ArcRtcIceCandidateCommand",
        "ArcRtcReconnectCommand",
      ),
      commands.map { it::class.java.simpleName },
    )
    assertEquals("credential-a", (commands[0] as ArcRtcJoinCommand).credentialRef)
    assertEquals("sdp-a", (commands[2] as ArcRtcOfferCommand).sdpRef)
    assertEquals("sdp-a", (commands[3] as ArcRtcAnswerCommand).sdpRef)
    assertEquals("candidate-a", (commands[4] as ArcRtcIceCandidateCommand).candidateRef)
    assertEquals("session-a", (commands[5] as ArcRtcReconnectCommand).sessionRef)
  }

  @Test
  fun androidSdkExposesFixedPublicEventNamesAndFields() {
    val events =
      listOf(
        ArcRtcJoinAcceptedEvent(
          roomId = "room-a",
          participantId = "participant-a",
          sessionRef = "session-a",
        ),
        ArcRtcJoinRejectedEvent(
          roomId = "room-a",
          participantId = "participant-a",
          reasonCode = "join_rejected",
        ),
        ArcRtcParticipantLeftEvent(
          roomId = "room-a",
          participantId = "participant-a",
          sessionRef = "session-a",
        ),
        ArcRtcNegotiationRequiredEvent(
          roomId = "room-a",
          participantId = "participant-a",
          sdpRef = "sdp-a",
          sessionRef = "session-a",
        ),
        ArcRtcIceCandidateReceivedEvent(
          roomId = "room-a",
          participantId = "participant-a",
          candidateRef = "candidate-a",
          sessionRef = "session-a",
        ),
        ArcRtcSessionTimedOutEvent(
          roomId = "room-a",
          participantId = "participant-a",
          sessionRef = "session-a",
          reasonCode = "session_timed_out",
        ),
      )

    assertEquals(
      listOf(
        "ArcRtcJoinAcceptedEvent",
        "ArcRtcJoinRejectedEvent",
        "ArcRtcParticipantLeftEvent",
        "ArcRtcNegotiationRequiredEvent",
        "ArcRtcIceCandidateReceivedEvent",
        "ArcRtcSessionTimedOutEvent",
      ),
      events.map { it::class.java.simpleName },
    )
    assertEquals("join_rejected", (events[1] as ArcRtcJoinRejectedEvent).reasonCode)
    assertEquals("sdp-a", (events[3] as ArcRtcNegotiationRequiredEvent).sdpRef)
    assertEquals("candidate-a", (events[4] as ArcRtcIceCandidateReceivedEvent).candidateRef)
    assertEquals("session_timed_out", (events[5] as ArcRtcSessionTimedOutEvent).reasonCode)
  }

  @Test
  fun androidSdkExposesFixedFailureCodeAndVersionConstants() {
    assertEquals(
      listOf(
        "CredentialRejected",
        "JoinRejected",
        "MembershipConflict",
        "NegotiationRejected",
        "IceCandidateRejected",
        "Timeout",
        "TransportUnavailable",
        "ProtocolViolation",
        "VersionMismatch",
        "InternalInvariantViolation",
      ),
      ArcRtcSdkFailureCode.entries.map { it.name },
    )
    assertEquals("v0.2", ARCRTC_SIGNALING_PROTOCOL_VERSION)
    assertEquals("0.2.0", ARCRTC_SDK_SEMVER)
    assertEquals("dev.arcrtc.sdk", ARCRTC_SDK_PACKAGE_NAME)
  }
}
