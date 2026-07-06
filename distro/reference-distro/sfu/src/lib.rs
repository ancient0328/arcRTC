//! reference SFU distro の公開境界です。

pub mod error;
pub mod fixture_route_auth;
pub mod kernel_contract;
pub mod state;

pub use arcrtc_reference_output::ReferenceSfuOutcome;
pub use error::ReferenceSfuError;
pub use fixture_route_auth::{authorize_reference_route, FixtureRouteAdmission};
pub use kernel_contract::{
    build_borrowed_packet_view, build_kernel_sfu_item, ReferenceSfuContractInput,
};
pub use state::{
    apply_reference_sfu, validate_reference_sfu_state, ReferenceEndpointState, ReferenceRouteState,
    ReferenceSfuAction, ReferenceSfuSessionState, ReferenceSfuState, ReferenceSfuSuppressionSource,
};
