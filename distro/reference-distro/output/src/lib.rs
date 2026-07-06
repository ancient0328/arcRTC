//! reference distro が外へ渡す outcome 型の境界です。
//!
//! product distro が消費できる reference surface は、このcrateのoutcome型だけです。

pub mod composition;
pub mod error;
pub mod sfu;
pub mod signaling;
pub mod turn;

pub use composition::ReferenceCompositionOutcome;
pub use error::ReferenceOutputError;
pub use sfu::ReferenceSfuOutcome;
pub use signaling::ReferenceSignalingOutcome;
pub use turn::ReferenceTurnOutcome;
