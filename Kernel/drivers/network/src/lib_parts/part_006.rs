// Signaling wire codec は driver-owned byte 表現だけを扱い、core の状態判断は行いません。

/// Signaling inbound frame を core command へ渡す前の driver decode 結果です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalingWireDecoded {
    /// public Signaling command kind です。
    pub kind: arcrtc_core_signaling::SignalingCommandKind,
    /// external frame から得た room reference です。
    pub room: arcrtc_core_identity::UntrustedReference,
    /// external frame から得た correlation reference です。
    pub correlation: arcrtc_core_identity::UntrustedReference,
    /// 受信 frame の実長です。
    pub frame_len: usize,
}

/// Signaling inbound frame を driver-owned syntax として decode します。
pub fn decode_signaling_command_frame(
    bytes: &[u8],
) -> Result<SignalingWireDecoded, DriverConversionFailure> {
    if bytes.len() > 65536 {
        return Err(DriverConversionFailure::from_kind(
            DriverConversionFailureKind::FrameSizeBoundExceeded,
        ));
    }
    if bytes.len() < 3 {
        return Err(DriverConversionFailure::from_kind(
            DriverConversionFailureKind::ExternalDecodeFailed,
        ));
    }

    let kind = match bytes[0] {
        0 => arcrtc_core_signaling::SignalingCommandKind::JoinRoom,
        1 => arcrtc_core_signaling::SignalingCommandKind::LeaveRoom,
        2 => arcrtc_core_signaling::SignalingCommandKind::SendOffer,
        3 => arcrtc_core_signaling::SignalingCommandKind::SendAnswer,
        4 => arcrtc_core_signaling::SignalingCommandKind::SendIceCandidate,
        5 => arcrtc_core_signaling::SignalingCommandKind::RequestTurnCredential,
        6 => arcrtc_core_signaling::SignalingCommandKind::AcknowledgeForward,
        _ => {
            return Err(DriverConversionFailure::from_kind(
                DriverConversionFailureKind::ExternalEnumUnmapped,
            ));
        }
    };
    let room_len = u16::from_be_bytes([bytes[1], bytes[2]]) as usize;
    if room_len > bytes.len() - 3 {
        return Err(DriverConversionFailure::from_kind(
            DriverConversionFailureKind::ExternalDecodeFailed,
        ));
    }
    if room_len == 0 {
        return Err(DriverConversionFailure::from_kind(
            DriverConversionFailureKind::MissingRequiredWireField,
        ));
    }

    let room_end = 3 + room_len;
    let room_bytes = &bytes[3..room_end];
    let correlation_bytes = &bytes[room_end..];
    if correlation_bytes.is_empty() {
        return Err(DriverConversionFailure::from_kind(
            DriverConversionFailureKind::MissingCorrelationId,
        ));
    }

    let room_string = std::str::from_utf8(room_bytes)
        .map_err(|_error| {
            DriverConversionFailure::from_kind(DriverConversionFailureKind::ExternalDecodeFailed)
        })?
        .to_owned();
    let correlation_string = std::str::from_utf8(correlation_bytes)
        .map_err(|_error| {
            DriverConversionFailure::from_kind(DriverConversionFailureKind::ExternalDecodeFailed)
        })?
        .to_owned();

    Ok(SignalingWireDecoded {
        kind,
        room: arcrtc_core_identity::UntrustedReference::new(room_string),
        correlation: arcrtc_core_identity::UntrustedReference::new(correlation_string),
        frame_len: bytes.len(),
    })
}

/// Decode 済み Signaling frame から core-owned Signaling command を組み立てます。
pub fn build_signaling_command(
    decoded: SignalingWireDecoded,
) -> Result<arcrtc_core_signaling::SignalingCommand<()>, DriverConversionFailure> {
    let SignalingWireDecoded {
        kind,
        room,
        correlation,
        frame_len,
    } = decoded;

    let room_id = arcrtc_core_identity::OpaqueReference::accept_untrusted(
        room,
        arcrtc_core_identity::ReferenceAuthority::CoreValidatedUntrustedInput,
    )
    .map(arcrtc_core_identity::RoomId::new)
    .map_err(|_error: arcrtc_core_identity::OpaqueReferenceError| {
        DriverConversionFailure::from_kind(DriverConversionFailureKind::ExternalDecodeFailed)
    })?;
    let subject = arcrtc_core_signaling::SignalingSubject::new(room_id, None);
    let preconditions = DriverIngressPreconditions::new(
        ExternalIngressKind::TcpFrame,
        true,
        frame_len,
        65536,
        true,
        true,
        true,
        true,
        true,
        true,
    );
    let semantic_delegation =
        SemanticDelegationGuard::try_new(true, true, false, false)
            .expect("semantic delegation guard arguments are fixed");
    let envelope = DriverCommandConversionInput::new(
        preconditions,
        semantic_delegation,
        true,
        Some(correlation),
        Some(arcrtc_core_command::CommandType::new("signaling")),
        Some(arcrtc_core_command::CommandVersion::new(1)),
        Some(arcrtc_core_command::TargetSurface::Signaling),
        subject,
    )
    .into_core_command_envelope()?;

    Ok(arcrtc_core_signaling::SignalingCommand::new(
        envelope, kind, (),
    ))
}

/// Signaling event を driver-owned outbound frame へ encode します。
pub fn encode_signaling_event_frame(
    kind: arcrtc_core_signaling::SignalingEventKind,
    reason_code: Option<&str>,
) -> Vec<u8> {
    let discriminant = match kind {
        arcrtc_core_signaling::SignalingEventKind::Joined => 0,
        arcrtc_core_signaling::SignalingEventKind::Rejected => 1,
        arcrtc_core_signaling::SignalingEventKind::ParticipantJoined => 2,
        arcrtc_core_signaling::SignalingEventKind::ParticipantLeft => 3,
        arcrtc_core_signaling::SignalingEventKind::OfferReceived => 4,
        arcrtc_core_signaling::SignalingEventKind::AnswerReceived => 5,
        arcrtc_core_signaling::SignalingEventKind::IceCandidateReceived => 6,
        arcrtc_core_signaling::SignalingEventKind::TurnCredentialAvailable => 7,
        arcrtc_core_signaling::SignalingEventKind::ProtocolViolation => 8,
    };
    let mut frame = Vec::with_capacity(1 + reason_code.map(str::len).unwrap_or(0));
    frame.push(discriminant);
    if let Some(code) = reason_code {
        frame.extend_from_slice(code.as_bytes());
    }
    frame
}

/// driver conversion failure を Signaling Rejected frame へ encode します。
pub fn encode_signaling_rejection_frame(kind: DriverConversionFailureKind) -> Vec<u8> {
    encode_signaling_event_frame(
        arcrtc_core_signaling::SignalingEventKind::Rejected,
        Some(kind.reason_code()),
    )
}
