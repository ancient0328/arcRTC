//! product runtime selection の境界です。

use arcrtc_core_identity::CorrelationId;
use arcrtc_distro_evidence::{
    DistroEnvironmentClass, DistroEvidenceReason, DistroNonClaimScope,
};
use arcrtc_product_rollback::{ProductDrainPlan, ProductRestorePlan};

use crate::error::ProductRuntimeError;
use crate::profile::{ProductHostClass, ProductRuntimeProfile};

/// product runtime stateです。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DistroRuntimeState {
    /// created stateです。
    Created,
    /// starting stateです。
    Starting,
    /// running stateです。
    Running,
    /// draining stateです。
    Draining,
    /// stopped stateです。
    Stopped,
    /// failed stateです。
    Failed,
}

/// product runtime descriptorです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductRuntime {
    /// runtime profileです。
    pub profile: ProductRuntimeProfile,
    /// runtime stateです。
    pub state: DistroRuntimeState,
}

/// product runtime selectionです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductRuntimeSelection {
    /// input profileです。
    pub profile: ProductRuntimeProfile,
    /// distro reasonです。
    pub distro_reason: DistroEvidenceReason,
}

/// product runtime lifecycle outcomeです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductRuntimeOutcome {
    /// command / report / span を接続する相関IDです。
    pub correlation_id: CorrelationId,
    /// runtime action 後のstateです。
    pub state: DistroRuntimeState,
    /// runtime action の実装側reasonです。
    pub distro_reason: DistroEvidenceReason,
    /// runtime action が主張しない範囲です。
    pub non_claim_scope: Vec<DistroNonClaimScope>,
}

/// product runtimeを選択します。
pub fn select_product_runtime(profile: &ProductRuntimeProfile) -> ProductRuntimeSelection {
    ProductRuntimeSelection {
        profile: profile.clone(),
        distro_reason: runtime_selection_reason(profile),
    }
}

impl ProductRuntime {
    /// product runtime descriptorを作ります。
    pub const fn new(profile: ProductRuntimeProfile) -> Self {
        Self {
            profile,
            state: DistroRuntimeState::Created,
        }
    }

    /// product runtimeをstartします。
    pub fn start(
        &mut self,
        correlation_id: CorrelationId,
    ) -> Result<ProductRuntimeOutcome, ProductRuntimeError> {
        if runtime_selection_reason(&self.profile)
            == DistroEvidenceReason::ReadinessNotAdmitted
        {
            self.state = DistroRuntimeState::Failed;
            return Err(ProductRuntimeError::ReadinessNotAdmitted);
        }
        if !matches!(
            self.state,
            DistroRuntimeState::Created | DistroRuntimeState::Stopped
        ) {
            self.state = DistroRuntimeState::Failed;
            return Err(ProductRuntimeError::RuntimeExecutorError);
        }
        self.state = DistroRuntimeState::Starting;
        self.state = DistroRuntimeState::Running;
        Ok(product_runtime_outcome(
            correlation_id,
            self.state,
            DistroEvidenceReason::DistroOk,
        ))
    }

    /// product runtimeをdrainします。
    pub fn drain(
        &mut self,
        plan: ProductDrainPlan,
    ) -> Result<ProductRuntimeOutcome, ProductRuntimeError> {
        if plan.distro_reason == DistroEvidenceReason::ReadinessNotAdmitted {
            self.state = DistroRuntimeState::Failed;
            return Err(ProductRuntimeError::ReadinessNotAdmitted);
        }
        if self.state != DistroRuntimeState::Running {
            self.state = DistroRuntimeState::Failed;
            return Err(ProductRuntimeError::RuntimeExecutorError);
        }
        self.state = DistroRuntimeState::Draining;
        self.state = DistroRuntimeState::Stopped;
        Ok(product_runtime_outcome(
            plan.correlation_id,
            self.state,
            plan.distro_reason,
        ))
    }

    /// product runtimeをrestoreします。
    pub fn restore(
        &mut self,
        plan: ProductRestorePlan,
    ) -> Result<ProductRuntimeOutcome, ProductRuntimeError> {
        if plan.distro_reason == DistroEvidenceReason::ReadinessNotAdmitted {
            self.state = DistroRuntimeState::Failed;
            return Err(ProductRuntimeError::ReadinessNotAdmitted);
        }
        if !matches!(
            self.state,
            DistroRuntimeState::Stopped | DistroRuntimeState::Failed
        ) {
            self.state = DistroRuntimeState::Failed;
            return Err(ProductRuntimeError::RuntimeExecutorError);
        }
        self.state = DistroRuntimeState::Starting;
        self.state = DistroRuntimeState::Running;
        Ok(product_runtime_outcome(
            plan.correlation_id,
            self.state,
            plan.distro_reason,
        ))
    }

    /// product runtime stateを返します。
    pub const fn state(&self) -> DistroRuntimeState {
        self.state
    }
}

fn runtime_selection_reason(profile: &ProductRuntimeProfile) -> DistroEvidenceReason {
    if profile.public_endpoint_claimed
        && !matches!(
            (profile.host_class, profile.environment_class),
            (
                ProductHostClass::LiveAdmitted,
                DistroEnvironmentClass::LiveDeferred,
            )
        )
    {
        return DistroEvidenceReason::ReadinessNotAdmitted;
    }
    match (profile.host_class, profile.environment_class) {
        (ProductHostClass::LocalSingleHost, DistroEnvironmentClass::LocalSingleHost)
        | (
            ProductHostClass::ControlledMultiProcess,
            DistroEnvironmentClass::ControlledProcess,
        )
        | (
            ProductHostClass::ProductionAdmitted,
            DistroEnvironmentClass::ProductionDeferred,
        )
        | (ProductHostClass::LiveAdmitted, DistroEnvironmentClass::LiveDeferred) => {
            DistroEvidenceReason::DistroOk
        }
        (
            ProductHostClass::ProductionDeferred,
            DistroEnvironmentClass::ProductionDeferred,
        )
        | (ProductHostClass::LiveDeferred, DistroEnvironmentClass::LiveDeferred) => {
            DistroEvidenceReason::ReadinessNotAdmitted
        }
        _ => DistroEvidenceReason::ReadinessNotAdmitted,
    }
}

fn product_runtime_outcome(
    correlation_id: CorrelationId,
    state: DistroRuntimeState,
    distro_reason: DistroEvidenceReason,
) -> ProductRuntimeOutcome {
    ProductRuntimeOutcome {
        correlation_id,
        state,
        distro_reason,
        non_claim_scope: vec![
            DistroNonClaimScope::BehaviorCorrectnessNotClaimed,
            DistroNonClaimScope::ProductionReadinessNotClaimed,
            DistroNonClaimScope::LiveReadinessNotClaimed,
        ],
    }
}
