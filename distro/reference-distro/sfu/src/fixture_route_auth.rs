//! deterministic SFU route admission fixtureです。

use arcrtc_core_identity::{EndpointId, RouteId, StreamId};

use crate::error::ReferenceSfuError;

/// reference route admission fixtureです。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixtureRouteAdmission {
    /// route referenceです。
    pub route_id: RouteId,
    /// stream referenceです。
    pub stream_id: StreamId,
    /// endpoint referenceです。
    pub endpoint_id: EndpointId,
    /// deterministic local admission flagです。
    pub allowed: bool,
}

/// reference route admissionを評価します。
pub fn authorize_reference_route(
    admission: &FixtureRouteAdmission,
) -> Result<(), ReferenceSfuError> {
    if !admission.allowed
        || admission.route_id.as_str().is_empty()
        || admission.stream_id.as_str().is_empty()
        || admission.endpoint_id.as_str().is_empty()
    {
        return Err(ReferenceSfuError::InvalidFixtureRouteAdmission);
    }
    Ok(())
}
