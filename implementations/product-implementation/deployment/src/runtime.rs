//! product runtime selection の境界です。

use arcrtc_core_identity::CorrelationId;
use arcrtc_implementation_evidence::{
    ImplementationEnvironmentClass, ImplementationEvidenceReason, ImplementationNonClaimScope,
};
use arcrtc_product_rollback::{ProductDrainPlan, ProductRestorePlan};

use crate::error::ProductRuntimeError;
use crate::profile::{ProductHostClass, ProductRuntimeProfile};

/// product runtime stateです。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImplementationRuntimeState {
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
    pub state: ImplementationRuntimeState,
}

/// product runtime selectionです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductRuntimeSelection {
    /// input profileです。
    pub profile: ProductRuntimeProfile,
    /// implementation reasonです。
    pub implementation_reason: ImplementationEvidenceReason,
}

/// product runtime lifecycle outcomeです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductRuntimeOutcome {
    /// command / report / span を接続する相関IDです。
    pub correlation_id: CorrelationId,
    /// runtime action 後のstateです。
    pub state: ImplementationRuntimeState,
    /// runtime action の実装側reasonです。
    pub implementation_reason: ImplementationEvidenceReason,
    /// runtime action が主張しない範囲です。
    pub non_claim_scope: Vec<ImplementationNonClaimScope>,
}

/// product runtimeを選択します。
pub fn select_product_runtime(profile: &ProductRuntimeProfile) -> ProductRuntimeSelection {
    ProductRuntimeSelection {
        profile: profile.clone(),
        implementation_reason: runtime_selection_reason(profile),
    }
}

impl ProductRuntime {
    /// product runtime descriptorを作ります。
    pub const fn new(profile: ProductRuntimeProfile) -> Self {
        Self {
            profile,
            state: ImplementationRuntimeState::Created,
        }
    }

    /// product runtimeをstartします。
    pub fn start(
        &mut self,
        correlation_id: CorrelationId,
    ) -> Result<ProductRuntimeOutcome, ProductRuntimeError> {
        if runtime_selection_reason(&self.profile)
            == ImplementationEvidenceReason::ReadinessNotAdmitted
        {
            self.state = ImplementationRuntimeState::Failed;
            return Err(ProductRuntimeError::ReadinessNotAdmitted);
        }
        if !matches!(
            self.state,
            ImplementationRuntimeState::Created | ImplementationRuntimeState::Stopped
        ) {
            self.state = ImplementationRuntimeState::Failed;
            return Err(ProductRuntimeError::RuntimeExecutorError);
        }
        self.state = ImplementationRuntimeState::Starting;
        self.state = ImplementationRuntimeState::Running;
        Ok(product_runtime_outcome(
            correlation_id,
            self.state,
            ImplementationEvidenceReason::ImplementationOk,
        ))
    }

    /// product runtimeをdrainします。
    pub fn drain(
        &mut self,
        plan: ProductDrainPlan,
    ) -> Result<ProductRuntimeOutcome, ProductRuntimeError> {
        if plan.implementation_reason == ImplementationEvidenceReason::ReadinessNotAdmitted {
            self.state = ImplementationRuntimeState::Failed;
            return Err(ProductRuntimeError::ReadinessNotAdmitted);
        }
        if self.state != ImplementationRuntimeState::Running {
            self.state = ImplementationRuntimeState::Failed;
            return Err(ProductRuntimeError::RuntimeExecutorError);
        }
        self.state = ImplementationRuntimeState::Draining;
        self.state = ImplementationRuntimeState::Stopped;
        Ok(product_runtime_outcome(
            plan.correlation_id,
            self.state,
            plan.implementation_reason,
        ))
    }

    /// product runtimeをrestoreします。
    pub fn restore(
        &mut self,
        plan: ProductRestorePlan,
    ) -> Result<ProductRuntimeOutcome, ProductRuntimeError> {
        if plan.implementation_reason == ImplementationEvidenceReason::ReadinessNotAdmitted {
            self.state = ImplementationRuntimeState::Failed;
            return Err(ProductRuntimeError::ReadinessNotAdmitted);
        }
        if !matches!(
            self.state,
            ImplementationRuntimeState::Stopped | ImplementationRuntimeState::Failed
        ) {
            self.state = ImplementationRuntimeState::Failed;
            return Err(ProductRuntimeError::RuntimeExecutorError);
        }
        self.state = ImplementationRuntimeState::Starting;
        self.state = ImplementationRuntimeState::Running;
        Ok(product_runtime_outcome(
            plan.correlation_id,
            self.state,
            plan.implementation_reason,
        ))
    }

    /// product runtime stateを返します。
    pub const fn state(&self) -> ImplementationRuntimeState {
        self.state
    }
}

fn runtime_selection_reason(profile: &ProductRuntimeProfile) -> ImplementationEvidenceReason {
    if profile.public_endpoint_claimed
        && !matches!(
            (profile.host_class, profile.environment_class),
            (
                ProductHostClass::LiveAdmitted,
                ImplementationEnvironmentClass::LiveDeferred,
            )
        )
    {
        return ImplementationEvidenceReason::ReadinessNotAdmitted;
    }
    match (profile.host_class, profile.environment_class) {
        (ProductHostClass::LocalSingleHost, ImplementationEnvironmentClass::LocalSingleHost)
        | (
            ProductHostClass::ControlledMultiProcess,
            ImplementationEnvironmentClass::ControlledProcess,
        )
        | (
            ProductHostClass::ProductionAdmitted,
            ImplementationEnvironmentClass::ProductionDeferred,
        )
        | (ProductHostClass::LiveAdmitted, ImplementationEnvironmentClass::LiveDeferred) => {
            ImplementationEvidenceReason::ImplementationOk
        }
        (
            ProductHostClass::ProductionDeferred,
            ImplementationEnvironmentClass::ProductionDeferred,
        )
        | (ProductHostClass::LiveDeferred, ImplementationEnvironmentClass::LiveDeferred) => {
            ImplementationEvidenceReason::ReadinessNotAdmitted
        }
        _ => ImplementationEvidenceReason::ReadinessNotAdmitted,
    }
}

fn product_runtime_outcome(
    correlation_id: CorrelationId,
    state: ImplementationRuntimeState,
    implementation_reason: ImplementationEvidenceReason,
) -> ProductRuntimeOutcome {
    ProductRuntimeOutcome {
        correlation_id,
        state,
        implementation_reason,
        non_claim_scope: vec![
            ImplementationNonClaimScope::BehaviorCorrectnessNotClaimed,
            ImplementationNonClaimScope::ProductionReadinessNotClaimed,
            ImplementationNonClaimScope::LiveReadinessNotClaimed,
        ],
    }
}
