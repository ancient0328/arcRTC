//! reference composition distro の公開境界です。

pub mod composition_state;
pub mod error;
pub mod runtime_bridge;
pub mod step_input;

pub use arcrtc_reference_output::ReferenceCompositionOutcome;
pub use composition_state::{
    bind_signaling_to_sfu, bind_signaling_to_turn, validate_reference_composition_state,
    ReferenceCompositionState, ReferenceCrossPlaneBinding,
};
pub use error::ReferenceCompositionError;
pub use runtime_bridge::{
    run_reference_composition_step, ReferenceCompositionPlaneOutcome,
    ReferenceCompositionStepOutcome,
};
pub use step_input::{ReferenceCompositionStep, ReferenceCompositionStepInput};
