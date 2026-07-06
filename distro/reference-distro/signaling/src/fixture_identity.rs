//! deterministic Signaling fixture identity / payload型です。

use arcrtc_core_identity::{ParticipantId, RoomId, SessionId};

/// reference local identity fixtureです。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixtureIdentity {
    /// fixture identity idです。
    pub identity_id: String,
    /// participant referenceです。
    pub participant_id: ParticipantId,
    /// room referenceです。
    pub room_id: RoomId,
}

/// fixture session descriptionの方向です。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FixtureSessionDescriptionDirection {
    /// offer fixtureです。
    Offer,
    /// answer fixtureです。
    Answer,
}

/// real SDP bodyを持たないsession description fixtureです。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixtureSessionDescription {
    /// session referenceです。
    pub session_id: SessionId,
    /// fixture SDP idです。
    pub fixture_sdp_id: String,
    /// fixture directionです。
    pub direction: FixtureSessionDescriptionDirection,
}

/// real ICE candidate bodyを持たないICE candidate fixtureです。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixtureIceCandidate {
    /// session referenceです。
    pub session_id: SessionId,
    /// fixture candidate idです。
    pub fixture_candidate_id: String,
}
