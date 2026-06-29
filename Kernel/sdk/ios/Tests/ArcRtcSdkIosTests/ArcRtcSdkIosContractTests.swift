import XCTest
@testable import ArcRtcSdkIos

final class ArcRtcSdkIosContractTests: XCTestCase {
    func testPublicSurfaceRemainsSignalingOnly() {
        XCTAssertEqual(ArcRtcIosSdkSurface.boundary, "signaling-only")
        XCTAssertEqual(ArcRtcIosSdkSurface.platform, .ios)
        XCTAssertEqual(ArcRtcIosSdkSurface.signalingContractVersion, .v0_2)
        XCTAssertEqual(ArcRtcIosSdkSurface.commandKinds, Set(SignalingCommandKind.allCases))
        XCTAssertEqual(ArcRtcIosSdkSurface.eventKinds, Set(SignalingEventKind.allCases))
        XCTAssertTrue(ArcRtcIosSdkSurface.outOfScopeFeatures.contains(.authIssuance))
        XCTAssertTrue(ArcRtcIosSdkSurface.outOfScopeFeatures.contains(.peerConnection))
    }

    func testProjectionCoversEveryCommandAndEventExactlyOnce() {
        let projection = ArcRtcIosSdkSurface.projection

        XCTAssertTrue(ArcRtcIosSdkSurface.assertSignalingOnlyProjection(projection))
        XCTAssertEqual(projection.platform, .ios)
        XCTAssertEqual(projection.sourceContractVersion, .v0_2)
        XCTAssertEqual(projection.sourceContract, .signalingContractCanonical)
        XCTAssertEqual(projection.projectionClass, .platformApiProjection)
        XCTAssertFalse(projection.generatedArtifactIsSemanticAuthority)
        XCTAssertEqual(projection.entries.count, SignalingCommandKind.allCases.count + SignalingEventKind.allCases.count)
    }

    func testReconnectPolicyDoesNotAdmitSessionResumptionAsSdkOwnedSemantics() {
        XCTAssertFalse(
            ArcRtcIosSdkSurface.reconnectPolicyAdmitsSessionResumption(
                SdkReconnectPolicy(
                    reconnectClass: .sessionResumptionRequested,
                    localRetryLimit: 0,
                    initialDelayMs: 0,
                    maxDelayMs: 0
                )
            )
        )
        XCTAssertTrue(
            ArcRtcIosSdkSurface.reconnectPolicyAdmitsSessionResumption(
                SdkReconnectPolicy(
                    reconnectClass: .eventStreamResubscribe,
                    localRetryLimit: 1,
                    initialDelayMs: 10,
                    maxDelayMs: 100
                )
            )
        )
    }

    func testCommandModelsExposeOnlySignalingContractFields() {
        let correlationId = CorrelationId("ios-command-correlation")
        let target = OpaqueReference("ios-target-participant")
        let room = OpaqueReference("ios-room")
        let credential = OpaqueReference("ios-credential")
        let payload = OpaqueSignalingPayload(fields: ["sdp": "opaque", "candidate": "opaque"])

        let commands: [SignalingCommand] = [
            .joinRoom(
                JoinRoomCommand(
                    correlationId: correlationId,
                    roomRef: room,
                    credentialRef: credential,
                    capabilities: ["audio", "video"]
                )
            ),
            .leaveRoom(LeaveRoomCommand(correlationId: correlationId, participantRef: target)),
            .relay(.offer(correlationId: correlationId, targetParticipantRef: target, offer: payload)),
            .relay(.answer(correlationId: correlationId, targetParticipantRef: target, answer: payload)),
            .relay(.iceCandidate(correlationId: correlationId, targetParticipantRef: target, candidate: payload)),
            .requestTurnCredential(RequestTurnCredentialCommand(correlationId: correlationId, allocationRef: credential)),
            .acknowledgeForward(AcknowledgeForwardCommand(correlationId: correlationId, forwardedEventRef: OpaqueReference("forwarded-event")))
        ]

        XCTAssertEqual(commands.map(\.kind), SignalingCommandKind.allCases)
        XCTAssertTrue(commands.allSatisfy { $0.correlationId == correlationId })
        XCTAssertTrue(commands.allSatisfy { $0.contractVersion == .v0_2 })

        if case .joinRoom(let join) = commands[0] {
            XCTAssertEqual(join.roomRef, room)
            XCTAssertEqual(join.credentialRef, credential)
            XCTAssertEqual(join.capabilities, ["audio", "video"])
        } else {
            XCTFail("join command must remain a signaling command")
        }
    }

    func testEventModelsExposeKindCorrelationAndContractVersionForEveryCase() {
        let correlationId = CorrelationId("ios-event-correlation")
        let participant = OpaqueReference("ios-participant")
        let room = OpaqueReference("ios-room")
        let credential = OpaqueReference("ios-turn-credential")
        let reason = CatalogedServerReason.fromServerDecoded(category: "protocol", code: "unsupported_version")
        let payload = OpaqueSignalingPayload(fields: ["opaque": "payload"])

        let events: [SignalingEvent] = [
            .joined(correlationId: correlationId, participantRef: participant, roomRef: room),
            .rejected(correlationId: correlationId, serverReason: reason),
            .participantJoined(correlationId: correlationId, participantRef: participant),
            .participantLeft(correlationId: correlationId, participantRef: participant),
            .offerReceived(correlationId: correlationId, fromParticipantRef: participant, offer: payload),
            .answerReceived(correlationId: correlationId, fromParticipantRef: participant, answer: payload),
            .iceCandidateReceived(correlationId: correlationId, fromParticipantRef: participant, candidate: payload),
            .turnCredentialAvailable(correlationId: correlationId, credentialRef: credential),
            .protocolViolation(correlationId: correlationId, serverReason: reason)
        ]

        XCTAssertEqual(events.map(\.kind), SignalingEventKind.allCases)
        XCTAssertTrue(events.allSatisfy { $0.correlationId == correlationId })
        XCTAssertTrue(events.allSatisfy { $0.contractVersion == .v0_2 })

        XCTAssertEqual(SignalingEventCallbackItem.event(events[0]), .event(events[0]))
        XCTAssertEqual(
            SignalingEventCallbackItem.error(.serverReasonCarrier(.serverProtocolViolation, reason, message: "server rejected")),
            .error(.serverReasonCarrier(.serverProtocolViolation, reason, message: "server rejected"))
        )
    }

    func testErrorAndProjectionValueTypesPreserveServerReasonWithoutOwningSemantics() {
        let correlationId = CorrelationId("ios-result-correlation")
        let reason = CatalogedServerReason.fromServerDecoded(category: "policy", code: "room_closed")

        XCTAssertEqual(reason.category, "policy")
        XCTAssertEqual(reason.code, "room_closed")
        XCTAssertEqual(
            SignalingCommandResult.accepted(correlationId: correlationId),
            .accepted(correlationId: correlationId)
        )
        XCTAssertEqual(
            SignalingCommandResult.rejected(correlationId: correlationId, serverReason: reason),
            .rejected(correlationId: correlationId, serverReason: reason)
        )
        XCTAssertEqual(
            SignalingSendOutcome.result(.accepted(correlationId: correlationId)),
            .result(.accepted(correlationId: correlationId))
        )
        XCTAssertEqual(
            SignalingSendOutcome.error(.closedByRemoteServer(reason, message: "closed")),
            .error(.closedByRemoteServer(reason, message: "closed"))
        )
        XCTAssertEqual(
            ArcRtcSdkError.localOnly(.localSerializationError, message: "bad payload"),
            .localOnly(.localSerializationError, message: "bad payload")
        )
        XCTAssertEqual(
            ArcRtcSdkError.closedByRemoteLocalTransport(message: "socket closed"),
            .closedByRemoteLocalTransport(message: "socket closed")
        )

        // SDK projection は core/drivers の意味論ではなく public shape の写像だけを保持します。
        for entry in ArcRtcIosSdkSurface.projection.entries {
            XCTAssertEqual(entry.correlationPropagation, .required)
            XCTAssertFalse(entry.platformApiSymbol.isEmpty)
            XCTAssertFalse(entry.publicShape.isEmpty)
            switch entry.source {
            case .command(let kind):
                XCTAssertEqual(entry.sourceKindClass, .command)
                XCTAssertTrue(ArcRtcIosSdkSurface.commandKinds.contains(kind))
                XCTAssertEqual(entry.versionCapabilityBehavior, .sendReceiveServerResult)
            case .event(let kind):
                XCTAssertEqual(entry.sourceKindClass, .event)
                XCTAssertTrue(ArcRtcIosSdkSurface.eventKinds.contains(kind))
                XCTAssertEqual(entry.versionCapabilityBehavior, .preserveServerResult)
            }
        }
    }
}
