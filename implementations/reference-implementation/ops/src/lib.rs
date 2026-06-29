//! reference implementation ops の公開境界です。

pub mod error;
pub mod evidence;
pub mod reason;
pub mod runtime;

pub use arcrtc_implementation_evidence::{
    ImplementationEvidenceReason, ImplementationEvidenceRecord, IMPLEMENTATIONS_COMMAND_ROOT,
    IMPLEMENTATIONS_EVIDENCE_ROOT, IMPLEMENTATIONS_TARGET_ROOT,
};
pub use error::ReferenceRuntimeError;
pub use evidence::{write_implementation_evidence_record, ImplementationEvidenceWriteOutcome};
pub use reason::implementation_reason_closed_set;
pub use runtime::{
    build_reference_build_evidence_record, ImplementationRuntimePlane, ImplementationRuntimeState,
    ImplementationShutdownMode, ReferenceBuildEvidenceInput, ReferenceRuntime,
    ReferenceRuntimeOutcome,
};
