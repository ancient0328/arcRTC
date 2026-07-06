//! distro 全体で共有する evidence 型の公開境界です。
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

pub use error::DistroEvidenceError;
pub use readiness_extension::{
    validate_readiness_evidence_record, ReadinessAdmissionState, ReadinessClaim,
    ReadinessEvidenceRecord, ReadinessEvidenceValidationError, ReadinessValidationContext,
};
pub use reason::DistroEvidenceReason;
pub use record::{
    DistroCommandClass, DistroEnvironmentClass, DistroEvidenceRecord,
    DistroLayer, DistroNonClaimScope, DistroPlane,
};
pub use validation::{
    validate_evidence_record, EvidenceValidationError, DISTRO_COMMAND_ROOT,
    DISTRO_EVIDENCE_ROOT, DISTRO_TARGET_ROOT,
};
