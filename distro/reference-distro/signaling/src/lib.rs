//! reference Signaling distro の公開境界です。

pub mod error;
pub mod fixture_identity;
pub mod kernel_contract;
pub mod local_auth;
pub mod state;

pub use arcrtc_reference_output::ReferenceSignalingOutcome;
pub use error::ReferenceSignalingError;
pub use fixture_identity::{FixtureIceCandidate, FixtureIdentity, FixtureSessionDescription};
pub use kernel_contract::{
    build_kernel_signaling_command, ReferenceSignalingCommandInput, ReferenceSignalingPayload,
};
pub use local_auth::{authorize_reference_signaling, ReferenceLocalAuthDecision};
pub use state::{
    apply_reference_signaling, project_reference_signaling_event,
    validate_reference_signaling_state, ReferenceParticipantPhase, ReferenceParticipantState,
    ReferenceRoomPhase, ReferenceRoomState, ReferenceSignalingState,
};
