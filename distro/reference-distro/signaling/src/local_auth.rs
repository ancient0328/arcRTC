//! reference local auth の境界です。

use crate::{error::ReferenceSignalingError, fixture_identity::FixtureIdentity};

/// reference local auth decisionです。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferenceLocalAuthDecision {
    /// deterministic fixture identityを許可します。
    Allowed,
    /// deterministic fixture identityを拒否します。
    Denied,
}

/// reference Signaling用のlocal authを評価します。
pub fn authorize_reference_signaling(
    identity: &FixtureIdentity,
) -> Result<ReferenceLocalAuthDecision, ReferenceSignalingError> {
    if !identity.identity_id.starts_with("fixture-identity-")
        || identity.participant_id.as_str().is_empty()
        || identity.room_id.as_str().is_empty()
    {
        return Err(ReferenceSignalingError::InvalidFixtureIdentity);
    }
    Ok(ReferenceLocalAuthDecision::Allowed)
}
