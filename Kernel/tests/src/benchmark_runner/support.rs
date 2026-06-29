use arcrtc_core_identity::{
    CorrelationId, EndpointId, OpaqueReference, PacketId, ParticipantId, ReferenceAuthority,
    RouteId, SessionId, StartupRunId, StreamId,
};

pub(super) fn synthetic_packet_bytes(len: usize) -> Vec<u8> {
    (0..len)
        .map(|index| ((index * 31 + 17) % u8::MAX as usize) as u8)
        .collect()
}

pub(super) fn deterministic_digest_bytes(left: u64, right: u64) -> Vec<u8> {
    (0..32)
        .map(|index| {
            left.rotate_left(index)
                .wrapping_add(right.rotate_right(index))
                .wrapping_add(index as u64) as u8
        })
        .collect()
}

pub(super) fn mix_usize(state: u64, value: usize) -> u64 {
    state
        .wrapping_mul(1_099_511_628_211)
        .wrapping_add(value as u64)
}

pub(super) fn mix_bytes(mut state: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        state = state
            .wrapping_mul(1_099_511_628_211)
            .wrapping_add(*byte as u64);
    }
    state
}

pub(super) fn opaque_ref(value: impl Into<String>) -> OpaqueReference {
    OpaqueReference::accept(value, ReferenceAuthority::CorePolicy)
        .expect("benchmark opaque reference must be valid")
}

pub(super) fn correlation_id(value: impl Into<String>) -> CorrelationId {
    CorrelationId::new(opaque_ref(value))
}

pub(super) fn session_id(value: impl Into<String>) -> SessionId {
    SessionId::new(opaque_ref(value))
}

pub(super) fn participant_id(value: impl Into<String>) -> ParticipantId {
    ParticipantId::new(opaque_ref(value))
}

pub(super) fn endpoint_id(value: impl Into<String>) -> EndpointId {
    EndpointId::new(opaque_ref(value))
}

pub(super) fn stream_id(value: impl Into<String>) -> StreamId {
    StreamId::new(opaque_ref(value))
}

pub(super) fn packet_id(value: impl Into<String>) -> PacketId {
    PacketId::new(opaque_ref(value))
}

pub(super) fn route_id(value: impl Into<String>) -> RouteId {
    RouteId::new(opaque_ref(value))
}

pub(super) fn startup_run_id(value: impl Into<String>) -> StartupRunId {
    StartupRunId::new(opaque_ref(value))
}
