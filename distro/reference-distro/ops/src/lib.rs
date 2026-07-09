//! reference distro ops の公開境界です。

pub mod error;
pub mod evidence;
pub mod reason;
pub mod runtime;

pub use arcrtc_distro_evidence::{
    DistroEvidenceReason, DistroEvidenceRecord, DISTRO_COMMAND_ROOT, DISTRO_EVIDENCE_ROOT,
    DISTRO_TARGET_ROOT,
};
pub use error::ReferenceRuntimeError;
pub use evidence::{write_distro_evidence_record, DistroEvidenceWriteOutcome};
pub use reason::distro_reason_closed_set;
pub use runtime::{
    build_reference_build_evidence_record, DistroRuntimePlane, DistroRuntimeState,
    DistroShutdownMode, ReferenceBuildEvidenceInput, ReferenceRuntime, ReferenceRuntimeOutcome,
    ReferenceRuntimeSocketProbe,
};
