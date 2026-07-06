//! SFU reference input と Kernel contract item builder の境界です。

use arcrtc_core_identity::{EndpointId, PacketId, StreamId};
use arcrtc_core_sfu::{
    PacketHeaderSemanticView, SfuContractItem, SfuModelKind, SfuPacketView, SfuReferenceSet,
};

/// reference SFU contract inputです。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceSfuContractInput<Payload> {
    /// Kernel SFU model kindです。
    pub model_kind: SfuModelKind,
    /// Kernel SFU reference setです。
    pub references: SfuReferenceSet,
    /// payloadです。
    pub payload: Payload,
}

impl<Payload> ReferenceSfuContractInput<Payload> {
    /// reference SFU contract inputを作ります。
    pub const fn new(
        model_kind: SfuModelKind,
        references: SfuReferenceSet,
        payload: Payload,
    ) -> Self {
        Self {
            model_kind,
            references,
            payload,
        }
    }
}

/// Kernel SFU contract itemを構築します。
pub fn build_kernel_sfu_item<Payload>(
    input: ReferenceSfuContractInput<Payload>,
) -> SfuContractItem<Payload> {
    SfuContractItem::new(input.model_kind, input.references, input.payload)
}

/// driver-owned byte sliceをcopyせずにborrowed packet viewとして束ねます。
pub fn build_borrowed_packet_view<'packet>(
    packet_id: &'packet PacketId,
    stream_id: &'packet StreamId,
    source_endpoint_id: &'packet EndpointId,
    header: PacketHeaderSemanticView,
    raw_packet: &'packet [u8],
    payload: &'packet [u8],
) -> SfuPacketView<'packet> {
    SfuPacketView::new(
        packet_id,
        stream_id,
        source_endpoint_id,
        header,
        raw_packet,
        payload,
    )
}
