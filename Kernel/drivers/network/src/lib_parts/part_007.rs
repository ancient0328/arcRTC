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
    let attributes = TurnWireDecodedAttributes::new(
        transaction_id,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    TurnWireDecodeInput::new(preconditions, method, attributes)
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
