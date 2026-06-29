//! product drain plan の境界です。

use arcrtc_core_identity::CorrelationId;
use arcrtc_implementation_evidence::{ImplementationEvidenceReason, ImplementationPlane};

/// product drain modeです。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProductDrainMode {
    /// reference local drainです。
    ReferenceLocal,
    /// controlled product drainです。
    ControlledProduct,
    /// production drain は未採用です。
    ProductionDeferred,
    /// live readiness admitted drainです。
    LiveAdmitted,
}

/// product drain planです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductDrainPlan {
    /// correlation idです。
    pub correlation_id: CorrelationId,
    /// drain対象planeです。
    pub planes: Vec<ImplementationPlane>,
    /// drain modeです。
    pub mode: ProductDrainMode,
    /// implementation reasonです。
    pub implementation_reason: ImplementationEvidenceReason,
}

/// product drain planを作成します。
pub fn plan_drain(
    correlation_id: CorrelationId,
    planes: Vec<ImplementationPlane>,
    mode: ProductDrainMode,
) -> ProductDrainPlan {
    let implementation_reason = if planes.is_empty() {
        ImplementationEvidenceReason::RuntimeExecutorError
    } else if mode == ProductDrainMode::ProductionDeferred {
        ImplementationEvidenceReason::ReadinessNotAdmitted
    } else {
        ImplementationEvidenceReason::ImplementationOk
    };
    ProductDrainPlan {
        correlation_id,
        planes,
        mode,
        implementation_reason,
    }
}
