use arcrtc_kernel_test_suite::{
    assert_not_contains, read_impl, read_impl_source_set_with_extension,
};

const COMMANDS: [&str; 7] = [
    "JoinRoom",
    "LeaveRoom",
    "SendOffer",
    "SendAnswer",
    "SendIceCandidate",
    "RequestTurnCredential",
    "AcknowledgeForward",
];

const EVENTS: [&str; 9] = [
    "Joined",
    "Rejected",
    "ParticipantJoined",
    "ParticipantLeft",
    "OfferReceived",
    "AnswerReceived",
    "IceCandidateReceived",
    "TurnCredentialAvailable",
    "ProtocolViolation",
];

#[test]
fn typescript_sdk_exports_only_signaling_projection() {
    let source = read_impl("sdk/typescript/src/index.ts");
    assert!(source.contains("signaling-only"));
    assert!(source.contains("generatedArtifactIsSemanticAuthority: false"));
    assert!(source.contains("sdkOutOfScopeFeatures"));
    assert!(source.contains("CatalogedServerReason"));

    for command in COMMANDS {
        assert!(
            source.contains(command),
            "TypeScript SDK must project command {command}"
        );
    }
    for event in EVENTS {
        assert!(
            source.contains(event),
            "TypeScript SDK must project event {event}"
        );
    }
    assert_not_contains(
        "sdk/typescript/src/index.ts",
        &source,
        &[
            "from \"../../drivers",
            "from \"../drivers",
            "arcrtc-driver",
            "RegulatedEnrichmentGuard",
            "SfuDecision",
            "TurnDecision",
            "AuthIssuer",
        ],
    );
}

#[test]
fn android_and_ios_sdk_sources_preserve_projection_invariants() {
    for (path, extension, platform_marker) in [
        (
            "sdk/android/sdk/src/main/kotlin/dev/arcrtc/sdk",
            "kt",
            "SdkPlatform.Android",
        ),
        (
            "sdk/ios/Sources/ArcRtcSdkIos",
            "swift",
            "ArcRtcSdkPlatform.ios",
        ),
    ] {
        let source = read_impl_source_set_with_extension(path, extension);
        assert!(source.contains("signaling-only"), "{path}");
        assert!(source.contains(platform_marker), "{path}");
        assert!(
            source.contains("generatedArtifactIsSemanticAuthority") && source.contains("false"),
            "{path} must not make generated SDK artifact semantic authority"
        );
        assert!(
            source.contains("outOfScopeFeatures") || source.contains("OutOfScopeFeature"),
            "{path} must declare out-of-scope feature vocabulary"
        );
        for command in COMMANDS {
            assert!(
                source.contains(command),
                "{path} must project command {command}"
            );
        }
        for event in EVENTS {
            assert!(source.contains(event), "{path} must project event {event}");
        }
        assert_not_contains(
            path,
            &source,
            &[
                "arcrtc-driver",
                "DriverPort",
                "RegulatedEnrichmentGuard",
                "SfuDecision",
                "TurnDecision",
                "AuthIssuer",
            ],
        );
    }
}

#[test]
fn sdk_contract_tests_are_platform_specific_not_v0_1_substitutes() {
    let package_json = read_impl("sdk/typescript/package.json");
    let node_test = read_impl("sdk/typescript/test/sdk-contract.test.mjs");
    assert!(package_json.contains("\"test\""));
    assert!(node_test.contains("TypeScript SDK remains a Signaling-only public surface"));
    assert_not_contains(
        "sdk/typescript/test/sdk-contract.test.mjs",
        &node_test,
        &["v0.1", "v0_1", "previous version proof"],
    );
}
