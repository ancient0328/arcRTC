//! product drain plan の境界です。

use arcrtc_core_identity::CorrelationId;
use arcrtc_distro_evidence::{DistroEvidenceReason, DistroPlane};

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
    pub planes: Vec<DistroPlane>,
    /// drain modeです。
    pub mode: ProductDrainMode,
    /// distro reasonです。
    pub distro_reason: DistroEvidenceReason,
}

/// product drain planを作成します。
pub fn plan_drain(
    correlation_id: CorrelationId,
    planes: Vec<DistroPlane>,
    mode: ProductDrainMode,
) -> ProductDrainPlan {
    let distro_reason = if planes.is_empty() {
        DistroEvidenceReason::RuntimeExecutorError
    } else if mode == ProductDrainMode::ProductionDeferred {
        DistroEvidenceReason::ReadinessNotAdmitted
    } else {
        DistroEvidenceReason::DistroOk
    };
    ProductDrainPlan {
        correlation_id,
        planes,
        mode,
        distro_reason,
    }
}
