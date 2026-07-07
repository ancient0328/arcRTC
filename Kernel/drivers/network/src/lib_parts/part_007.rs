// TURN raw datagram codec は STUN header の外形だけを見て、core 変換判断は既存入力型へ委譲します。

/// TURN raw datagram codec の固定 frame size 上限です。
pub const TURN_WIRE_FRAME_SIZE_BOUND_BYTES: usize = 2048;

/// TURN/STUN raw datagram を既存の TURN wire decode input へ変換します。
pub fn decode_turn_datagram(bytes: &[u8]) -> TurnWireDecodeInput {
    let header_ok = bytes.len() >= 20
        && (bytes[0] & 0xC0) == 0
        && bytes[4..8] == [0x21, 0x12, 0xA4, 0x42]
        && {
            let message_length = u16::from_be_bytes([bytes[2], bytes[3]]) as usize;
            message_length == bytes.len() - 20
        }
        && {
            let message_length = u16::from_be_bytes([bytes[2], bytes[3]]) as usize;
            message_length.is_multiple_of(4)
        };
    let preconditions = TurnWirePreconditions::new(
        bytes.len(),
        TURN_WIRE_FRAME_SIZE_BOUND_BYTES,
        header_ok,
        header_ok,
        true,
        false,
    );
    let method = if header_ok {
        Some(match u16::from_be_bytes([bytes[0], bytes[1]]) {
            0x0003 => TurnWireMethodClass::AllocateRequest,
            0x0004 => TurnWireMethodClass::RefreshRequest,
            0x0008 => TurnWireMethodClass::CreatePermissionRequest,
            0x0009 => TurnWireMethodClass::ChannelBindRequest,
            0x0016 => TurnWireMethodClass::SendIndication,
            0x0017 => TurnWireMethodClass::DataIndication,
            _ => TurnWireMethodClass::UnsupportedMethod,
        })
    } else {
        None
    };
    let transaction_id = if header_ok {
        Some(arcrtc_core_identity::UntrustedReference::new(hex_lower(
            &bytes[8..20],
        )))
    } else {
        None
    };
    let parsed_attributes = if header_ok {
        parse_turn_attributes(&bytes[20..])
    } else {
        ParsedTurnAttributes::default()
    };
    let attributes = TurnWireDecodedAttributes::new(
        transaction_id,
        parsed_attributes.allocation_ref,
        parsed_attributes.permission_ref,
        parsed_attributes.channel_bind_ref,
        parsed_attributes.credential_ref,
        parsed_attributes.peer_address,
        parsed_attributes.requested_lifetime_seconds,
        parsed_attributes.relay_packet_ref,
    );

    TurnWireDecodeInput::new(preconditions, method, attributes)
}

#[derive(Default)]
struct ParsedTurnAttributes {
    allocation_ref: Option<arcrtc_core_identity::UntrustedReference>,
    permission_ref: Option<arcrtc_core_identity::UntrustedReference>,
    channel_bind_ref: Option<arcrtc_core_identity::UntrustedReference>,
    credential_ref: Option<arcrtc_core_identity::UntrustedReference>,
    peer_address: Option<String>,
    requested_lifetime_seconds: Option<u32>,
    relay_packet_ref: Option<arcrtc_core_identity::UntrustedReference>,
}

fn parse_turn_attributes(bytes: &[u8]) -> ParsedTurnAttributes {
    let mut parsed = ParsedTurnAttributes::default();
    let mut offset = 0;
    while offset + 4 <= bytes.len() {
        let attribute_type = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]);
        let attribute_len = u16::from_be_bytes([bytes[offset + 2], bytes[offset + 3]]) as usize;
        let value_start = offset + 4;
        let value_end = value_start.saturating_add(attribute_len);
        if value_end > bytes.len() {
            break;
        }
        let value = &bytes[value_start..value_end];
        let reference = || arcrtc_core_identity::UntrustedReference::new(attribute_value(value));

        match attribute_type {
            0x0006 => parsed.credential_ref = Some(reference()),
            0x000c => parsed.peer_address = Some(attribute_value(value)),
            0x000d if value.len() == 4 => {
                parsed.requested_lifetime_seconds =
                    Some(u32::from_be_bytes([value[0], value[1], value[2], value[3]]));
            }
            0x0012 => parsed.channel_bind_ref = Some(reference()),
            0x0013 => parsed.relay_packet_ref = Some(reference()),
            0x8001 => parsed.allocation_ref = Some(reference()),
            0x8002 => parsed.permission_ref = Some(reference()),
            _ => {}
        }

        offset = value_start + ((attribute_len + 3) & !3);
    }
    parsed
}

fn attribute_value(bytes: &[u8]) -> String {
    std::str::from_utf8(bytes)
        .map(str::to_owned)
        .unwrap_or_else(|_error| hex_lower(bytes))
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

/// TURN error response 専用の outbound datagram を encode します。
pub fn encode_turn_error_datagram(reason_code: &str) -> Vec<u8> {
    let mut datagram = Vec::with_capacity(1 + reason_code.len());
    datagram.push(0x05);
    datagram.extend_from_slice(reason_code.as_bytes());
    datagram
}

/// TURN success response / forwarding を固定の driver-owned datagram に encode します。
pub fn encode_turn_success_datagram(output_class: TurnWireOutputClass) -> Vec<u8> {
    let discriminant = match output_class {
        TurnWireOutputClass::AllocationSuccessResponse => 0x00,
        TurnWireOutputClass::RefreshSuccessResponse => 0x01,
        TurnWireOutputClass::PermissionSuccessResponse => 0x02,
        TurnWireOutputClass::ChannelBindSuccessResponse => 0x03,
        TurnWireOutputClass::RelayDataForwarding => 0x04,
        TurnWireOutputClass::ErrorResponse | TurnWireOutputClass::DropOrDeny => 0x05,
    };
    vec![discriminant]
}

/// SFU media datagram の固定 frame size 上限です。
pub const SFU_MEDIA_FRAME_SIZE_BOUND_BYTES: usize = 2048;

/// SFU UDP datagram を driver-owned wire token として decode した結果です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SfuMediaDatagramClass {
    /// resident control token 0 です。
    ResidentControl0,
    /// resident control token 1 です。
    ResidentControl1,
    /// resident control token 2 です。
    ResidentControl2,
    /// resident control token 3 です。
    ResidentControl3,
    /// RTP/RTCP に見える payload token です。
    RtpOrRtcpLikePayload,
    /// DTLS record に見える payload token です。
    DtlsLikeRecord,
}

/// SFU media datagram decode の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SfuMediaDecodeFailureKind {
    /// external datagram を driver syntax として decode できません。
    ExternalDecodeFailed,
    /// datagram が固定上限を超えています。
    MediaPayloadMappingInvalid,
}

impl SfuMediaDecodeFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::ExternalDecodeFailed => "external_decode_failed",
            Self::MediaPayloadMappingInvalid => "media_payload_mapping_invalid",
        }
    }
}

/// SFU media datagram decode 結果です。raw payload ownership は driver 側に閉じます。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SfuMediaDatagram {
    class: SfuMediaDatagramClass,
}

impl SfuMediaDatagram {
    /// datagram class を保持します。
    pub const fn new(class: SfuMediaDatagramClass) -> Self {
        Self { class }
    }

    /// decoded wire token です。
    pub const fn class(&self) -> SfuMediaDatagramClass {
        self.class
    }

    /// wire token を core/sfu resident input へ投影します。
    ///
    /// driver は byte token の decode だけを所有し、state 更新と成功判定は core/sfu へ委譲します。
    pub const fn into_resident_sfu_input(self) -> arcrtc_core_sfu::ResidentSfuInputClass {
        match self.class {
            SfuMediaDatagramClass::ResidentControl0 => {
                arcrtc_core_sfu::ResidentSfuInputClass::IceConnectivityObservation
            }
            SfuMediaDatagramClass::ResidentControl1 | SfuMediaDatagramClass::DtlsLikeRecord => {
                arcrtc_core_sfu::ResidentSfuInputClass::DtlsHandshakeObservation
            }
            SfuMediaDatagramClass::ResidentControl2 => {
                arcrtc_core_sfu::ResidentSfuInputClass::SrtpProtectionObservation
            }
            SfuMediaDatagramClass::ResidentControl3
            | SfuMediaDatagramClass::RtpOrRtcpLikePayload => {
                arcrtc_core_sfu::ResidentSfuInputClass::ProtectedMediaPacket
            }
        }
    }
}

/// SFU media datagram を core contract へ渡す前に driver-owned wire token として decode します。
pub fn decode_sfu_media_datagram(
    bytes: &[u8],
) -> Result<SfuMediaDatagram, SfuMediaDecodeFailureKind> {
    if bytes.is_empty() {
        return Err(SfuMediaDecodeFailureKind::ExternalDecodeFailed);
    }
    if bytes.len() > SFU_MEDIA_FRAME_SIZE_BOUND_BYTES {
        return Err(SfuMediaDecodeFailureKind::MediaPayloadMappingInvalid);
    }

    match bytes {
        b"ARCRTC-SFU/ICE/CONNECTED" => Ok(SfuMediaDatagram::new(
            SfuMediaDatagramClass::ResidentControl0,
        )),
        b"ARCRTC-SFU/DTLS/ESTABLISHED" => Ok(SfuMediaDatagram::new(
            SfuMediaDatagramClass::ResidentControl1,
        )),
        b"ARCRTC-SFU/SRTP/ACTIVE" => Ok(SfuMediaDatagram::new(
            SfuMediaDatagramClass::ResidentControl2,
        )),
        b"ARCRTC-SFU/RTP/FORWARD" => Ok(SfuMediaDatagram::new(
            SfuMediaDatagramClass::ResidentControl3,
        )),
        _ if looks_like_rtp_or_rtcp(bytes) => Ok(SfuMediaDatagram::new(
            SfuMediaDatagramClass::RtpOrRtcpLikePayload,
        )),
        _ if looks_like_dtls(bytes) => Ok(SfuMediaDatagram::new(
            SfuMediaDatagramClass::DtlsLikeRecord,
        )),
        _ => Err(SfuMediaDecodeFailureKind::ExternalDecodeFailed),
    }
}

fn looks_like_rtp_or_rtcp(bytes: &[u8]) -> bool {
    bytes.len() >= 12 && (bytes[0] & 0b1100_0000) == 0b1000_0000
}

fn looks_like_dtls(bytes: &[u8]) -> bool {
    bytes.len() >= 4 && matches!(bytes[0], 0x14..=0x17)
}

/// SFU error response 専用の outbound datagram を encode します。
pub fn encode_sfu_error_datagram(reason_code: &str) -> Vec<u8> {
    let mut datagram = Vec::with_capacity(1 + reason_code.len());
    datagram.push(0x15);
    datagram.extend_from_slice(reason_code.as_bytes());
    datagram
}

/// SFU success response / forwarding を固定の driver-owned datagram に encode します。
pub fn encode_sfu_success_datagram(
    success_kind: arcrtc_core_sfu::ResidentSfuSuccessKind,
) -> Vec<u8> {
    let discriminant = match success_kind {
        arcrtc_core_sfu::ResidentSfuSuccessKind::IceConnectivityAccepted => 0x20,
        arcrtc_core_sfu::ResidentSfuSuccessKind::DtlsHandshakeAccepted => 0x21,
        arcrtc_core_sfu::ResidentSfuSuccessKind::SrtpProtectionAccepted => 0x22,
        arcrtc_core_sfu::ResidentSfuSuccessKind::ProtectedMediaForwarded => 0x23,
    };
    vec![discriminant]
}

/// core/turn の resident success を TURN wire success datagram へ投影します。
pub fn encode_resident_turn_success_datagram(
    success_kind: arcrtc_core_turn::ResidentTurnSuccessKind,
) -> Vec<u8> {
    let output_class = match success_kind {
        arcrtc_core_turn::ResidentTurnSuccessKind::AllocationSuccessResponse => {
            TurnWireOutputClass::AllocationSuccessResponse
        }
        arcrtc_core_turn::ResidentTurnSuccessKind::RefreshSuccessResponse => {
            TurnWireOutputClass::RefreshSuccessResponse
        }
        arcrtc_core_turn::ResidentTurnSuccessKind::PermissionSuccessResponse => {
            TurnWireOutputClass::PermissionSuccessResponse
        }
        arcrtc_core_turn::ResidentTurnSuccessKind::ChannelBindSuccessResponse => {
            TurnWireOutputClass::ChannelBindSuccessResponse
        }
        arcrtc_core_turn::ResidentTurnSuccessKind::RelayDataForwarding => {
            TurnWireOutputClass::RelayDataForwarding
        }
    };
    encode_turn_success_datagram(output_class)
}
