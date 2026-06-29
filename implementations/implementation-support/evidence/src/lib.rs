//! implementations 全体で共有する evidence 型の公開境界です。
//!
//! Kernel reason catalog を所有せず、実装側の証跡 reason / record だけをここへ集約します。

pub mod error;
#[rustfmt::skip]
pub mod reason;
#[rustfmt::skip]
pub mod record;
#[rustfmt::skip]
pub mod readiness_extension;
pub mod validation;

pub use error::ImplementationEvidenceError;
pub use readiness_extension::{
    validate_readiness_evidence_record, ReadinessAdmissionState, ReadinessClaim,
    ReadinessEvidenceRecord, ReadinessEvidenceValidationError, ReadinessValidationContext,
};
pub use reason::ImplementationEvidenceReason;
pub use record::{
    ImplementationCommandClass, ImplementationEnvironmentClass, ImplementationEvidenceRecord,
    ImplementationLayer, ImplementationNonClaimScope, ImplementationPlane,
};
pub use validation::{
    validate_evidence_record, EvidenceValidationError, IMPLEMENTATIONS_COMMAND_ROOT,
    IMPLEMENTATIONS_EVIDENCE_ROOT, IMPLEMENTATIONS_TARGET_ROOT,
};
