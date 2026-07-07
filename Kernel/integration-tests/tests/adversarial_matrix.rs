use arcrtc_core_reason::{
    register_closed_reason_coverage, AdversarialSurfaceClass, CatalogedReasonRef,
    ClosedReasonCoverageRegistrationDecision, ClosedReasonCoverageRow, FuzzSurfaceClass,
    ProtocolConformanceClass,
};

fn cataloged_reason(code: &str) -> CatalogedReasonRef {
    CatalogedReasonRef::from_code(code).expect("adversarial matrix reason must be cataloged")
}

#[test]
fn adversarial_surfaces_register_only_closed_reason_coverage_rows() {
    let rows = [
        ClosedReasonCoverageRow::new(
            AdversarialSurfaceClass::SignalingWire,
            cataloged_reason("malformed_command"),
            "protocol-conformance:signaling-wire",
        ),
        ClosedReasonCoverageRow::new(
            AdversarialSurfaceClass::TurnPacket,
            cataloged_reason("malformed_turn_message"),
            "malformed-packet:turn",
        ),
        ClosedReasonCoverageRow::new(
            AdversarialSurfaceClass::SfuDatagram,
            cataloged_reason("media_payload_mapping_invalid"),
            "fuzz:parser:sfu-datagram",
        ),
        ClosedReasonCoverageRow::new(
            AdversarialSurfaceClass::SdkCommand,
            cataloged_reason("sdk_contract_drift_detected"),
            "property:sdk-command",
        ),
        ClosedReasonCoverageRow::new(
            AdversarialSurfaceClass::ConfigInput,
            cataloged_reason("runtime_config_invalid"),
            "fuzz:driver-adapter:config",
        ),
    ];

    for row in rows {
        // adversarial coverage は free-text reason ではなく、closed reason ref だけを登録します。
        assert_eq!(
            register_closed_reason_coverage(row),
            ClosedReasonCoverageRegistrationDecision::Registered
        );
        assert!(!row
            .closed_reason_ref
            .definition()
            .code()
            .as_str()
            .is_empty());
        assert!(!row.assertion_ref.is_empty());
    }
}

#[test]
fn malformed_and_unsupported_protocol_classes_are_explicit_closed_values() {
    let protocol_classes = [
        ProtocolConformanceClass::Valid,
        ProtocolConformanceClass::Malformed,
        ProtocolConformanceClass::Unsupported,
    ];

    assert_eq!(
        protocol_classes,
        [
            ProtocolConformanceClass::Valid,
            ProtocolConformanceClass::Malformed,
            ProtocolConformanceClass::Unsupported,
        ]
    );
}

#[test]
fn fuzz_and_property_entrypoint_classes_are_explicit_closed_values() {
    let fuzz_classes = [
        FuzzSurfaceClass::Parser,
        FuzzSurfaceClass::StateTransition,
        FuzzSurfaceClass::DriverAdapter,
    ];

    assert_eq!(
        fuzz_classes,
        [
            FuzzSurfaceClass::Parser,
            FuzzSurfaceClass::StateTransition,
            FuzzSurfaceClass::DriverAdapter,
        ]
    );
}

#[test]
fn adversarial_registry_rejects_empty_assertion_and_unknown_reason() {
    let empty_assertion = ClosedReasonCoverageRow::new(
        AdversarialSurfaceClass::SignalingWire,
        cataloged_reason("malformed_command"),
        "",
    );

    assert_eq!(
        register_closed_reason_coverage(empty_assertion),
        ClosedReasonCoverageRegistrationDecision::Rejected
    );
    assert!(CatalogedReasonRef::from_code("UNKNOWN").is_err());
}
