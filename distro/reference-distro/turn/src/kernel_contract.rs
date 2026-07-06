//! TURN reference input と Kernel command builder の境界です。

use arcrtc_core_identity::{AllocationId, ChannelBindId, CredentialRef, PacketId, PermissionId};
use arcrtc_core_turn::{
    CorePeerAddress, TurnCommand, TurnCommandKind, TurnReferenceSet, TurnRequestedLifetimeSeconds,
    TurnTransactionId,
};

use crate::error::ReferenceTurnError;

/// reference TURN command inputです。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceTurnCommandInput {
    /// TURN command kindです。
    pub kind: TurnCommandKind,
    /// transaction referenceです。
    pub transaction_id: TurnTransactionId,
    /// TURN reference setです。
    pub references: TurnReferenceSet,
    /// reference state mutation用のallocation referenceです。
    pub allocation_id: Option<AllocationId>,
    /// reference state mutation用のpermission referenceです。
    pub permission_id: Option<PermissionId>,
    /// reference state mutation用のchannel bind referenceです。
    pub channel_bind_id: Option<ChannelBindId>,
    /// reference state mutation用のcredential referenceです。
    pub credential_ref: Option<CredentialRef>,
    /// peer addressです。
    pub peer_address: Option<CorePeerAddress>,
    /// requested lifetimeです。
    pub requested_lifetime: Option<TurnRequestedLifetimeSeconds>,
    /// relay packet referenceです。
    pub relay_packet_id: Option<PacketId>,
}

/// Kernel TURN commandを構築します。
pub fn build_kernel_turn_command(
    input: &ReferenceTurnCommandInput,
) -> Result<TurnCommand, ReferenceTurnError> {
    TurnCommand::try_new(
        input.kind,
        input.transaction_id.clone(),
        input.references.clone(),
        input.peer_address.clone(),
        input.requested_lifetime,
        input.relay_packet_id.clone(),
    )
    .map_err(|_| ReferenceTurnError::KernelContractMismatch)
}
