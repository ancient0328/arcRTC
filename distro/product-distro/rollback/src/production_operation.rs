//! production rollback / drain operation plan の境界です。

use arcrtc_core_identity::CorrelationId;
use arcrtc_distro_evidence::{DistroEvidenceReason, DistroPlane};

use crate::{
    drain::{plan_drain, ProductDrainMode, ProductDrainPlan},
    error::ProductRollbackError,
    restore::{plan_restore, ProductRestorePlan, ProductRestoreSource},
};

/// production drain plan を生成します。
pub fn plan_production_drain(
    correlation_id: CorrelationId,
    planes: Vec<DistroPlane>,
) -> Result<ProductDrainPlan, ProductRollbackError> {
    let plan = plan_drain(correlation_id, planes, ProductDrainMode::ControlledProduct);
    match plan.distro_reason {
        DistroEvidenceReason::DistroOk => Ok(plan),
        DistroEvidenceReason::RuntimeExecutorError => {
            Err(ProductRollbackError::RuntimeExecutorError)
        }
        _ => Err(ProductRollbackError::ReadinessNotAdmitted),
    }
}

/// production restore plan を生成します。
pub fn plan_production_restore(
    correlation_id: CorrelationId,
) -> Result<ProductRestorePlan, ProductRollbackError> {
    let plan = plan_restore(correlation_id, ProductRestoreSource::EvidenceReport);
    match plan.distro_reason {
        DistroEvidenceReason::DistroOk => Ok(plan),
        _ => Err(ProductRollbackError::ReadinessNotAdmitted),
    }
}
