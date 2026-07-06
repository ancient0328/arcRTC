//! deterministic TURN credential fixtureです。

use arcrtc_core_identity::{AllocationId, CredentialRef};
use arcrtc_core_turn::TurnRequestedLifetimeSeconds;

use crate::error::ReferenceTurnError;

/// raw secretを持たないTURN credential fixtureです。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixtureTurnCredential {
    /// credential referenceです。
    pub credential_ref: CredentialRef,
    /// allocation referenceです。
    pub allocation_id: AllocationId,
    /// requested lifetimeです。
    pub requested_lifetime: TurnRequestedLifetimeSeconds,
}

/// TURN credential fixtureを検証します。
pub fn validate_fixture_turn_credential(
    credential: &FixtureTurnCredential,
) -> Result<(), ReferenceTurnError> {
    if credential.credential_ref.as_str().is_empty()
        || credential.allocation_id.as_str().is_empty()
        || credential.requested_lifetime.as_u32() == 0
    {
        return Err(ReferenceTurnError::InvalidFixtureCredential);
    }
    Ok(())
}
