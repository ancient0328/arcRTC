//! live rollback / drain operation の境界です。

use arcrtc_core_identity::CorrelationId;
use arcrtc_distro_evidence::{DistroEvidenceReason, DistroPlane};

use crate::{
    drain::{plan_drain, ProductDrainMode, ProductDrainPlan},
    error::ProductRollbackError,
    restore::{plan_restore, ProductRestorePlan, ProductRestoreSource},
};

/// live shutdown drain を実行証跡入力として生成します。
pub fn execute_live_shutdown_drain(
    correlation_id: CorrelationId,
    planes: Vec<DistroPlane>,
) -> Result<ProductDrainPlan, ProductRollbackError> {
    let plan = plan_drain(correlation_id, planes, ProductDrainMode::LiveAdmitted);
    match plan.distro_reason {
        DistroEvidenceReason::DistroOk => Ok(plan),
        DistroEvidenceReason::RuntimeExecutorError => {
            Err(ProductRollbackError::RuntimeExecutorError)
        }
        _ => Err(ProductRollbackError::ReadinessNotAdmitted),
    }
}

/// live restore を実行証跡入力として生成します。
pub fn execute_live_restore(
    correlation_id: CorrelationId,
) -> Result<ProductRestorePlan, ProductRollbackError> {
    let plan = plan_restore(correlation_id, ProductRestoreSource::LiveEvidenceReport);
    match plan.distro_reason {
        DistroEvidenceReason::DistroOk => Ok(plan),
        _ => Err(ProductRollbackError::ReadinessNotAdmitted),
    }
}
