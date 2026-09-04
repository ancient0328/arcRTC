/// iOS SDK は Signaling-only public surface です。
///
/// media、auth issuance、regulated workflow、driver internal API はこの package で所有しません。
public enum ArcRtcIosSdkSurface {
    public static let boundary = "signaling-only"
    public static let platform = ArcRtcSdkPlatform.ios
    public static let signalingContractVersion = SignalingContractVersion.v0_2

    public static let commandKinds: Set<SignalingCommandKind> = Set(SignalingCommandKind.allCases)
    public static let eventKinds: Set<SignalingEventKind> = Set(SignalingEventKind.allCases)
    public static let outOfScopeFeatures: Set<SdkOutOfScopeFeature> = Set(SdkOutOfScopeFeature.allCases)

    public static let projection = SdkPublicApiProjection(
        platform: .ios,
        sourceProtocolVersion: .v0_2,
        sourceProtocol: .signalingProtocol,
        projectionClass: .platformApiProjection,
        generatedArtifactIsSemanticAuthority: false,
        commands: commandKinds,
        events: eventKinds,
        entries: iosProjectionEntries,
        outOfScopeFeatures: outOfScopeFeatures
    )

    public static func reconnectPolicyAdmitsSessionResumption(_ policy: SdkReconnectPolicy) -> Bool {
        policy.reconnectClass != .sessionResumptionRequested
    }

    public static func assertSignalingOnlyProjection(_ projection: SdkPublicApiProjection) -> Bool {
        let commandProjected = Set(
            projection.entries.compactMap { entry -> String? in
                guard case .command(let kind) = entry.source,
                      entry.sourceKindClass == .command else { return nil }
                return kind.rawValue
            }
        )
        let eventProjected = Set(
            projection.entries.compactMap { entry -> String? in
                guard case .event(let kind) = entry.source,
                      entry.sourceKindClass == .event else { return nil }
                return kind.rawValue
            }
        )
        let entryClassesAreConsistent = projection.entries.allSatisfy { entry in
            switch entry.source {
            case .command:
                return entry.sourceKindClass == .command
            case .event:
                return entry.sourceKindClass == .event
            }
        }

        return projection.platform == .ios
            && projection.sourceProtocolVersion == .v0_2
            && projection.sourceProtocol == .signalingProtocol
            && projection.projectionClass == .platformApiProjection
            && !projection.generatedArtifactIsSemanticAuthority
            && projection.commands == commandKinds
            && projection.events == eventKinds
            && projection.entries.count == commandKinds.count + eventKinds.count
            && entryClassesAreConsistent
            && commandProjected == Set(commandKinds.map(\.rawValue))
            && eventProjected == Set(eventKinds.map(\.rawValue))
            && projection.outOfScopeFeatures == outOfScopeFeatures
    }
}
