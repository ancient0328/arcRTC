//! product runtime selection の境界です。

use std::net::SocketAddr;

use arcrtc_core_identity::CorrelationId;
use arcrtc_distro_evidence::{DistroEnvironmentClass, DistroEvidenceReason, DistroNonClaimScope};
use arcrtc_product_rollback::{ProductDrainPlan, ProductRestorePlan};
use tokio::net::TcpListener;

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
#[derive(Debug)]
pub struct ProductRuntime {
    /// runtime profileです。
    pub profile: ProductRuntimeProfile,
    /// runtime stateです。
    pub state: DistroRuntimeState,
    /// socket probe の listener です。production/live endpoint ではなく、local runtime の生存性だけを保持します。
    socket_probe_listener: Option<TcpListener>,
}

/// product runtime socket probe の観測結果です。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProductRuntimeSocketProbe {
    /// 実際にbindされたlocal addressです。
    local_addr: SocketAddr,
}

impl ProductRuntimeSocketProbe {
    /// 実際にbindされたlocal addressを返します。
    pub const fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }
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
            socket_probe_listener: None,
        }
    }

    /// product runtimeをstartします。
    pub fn start(
        &mut self,
        correlation_id: CorrelationId,
    ) -> Result<ProductRuntimeOutcome, ProductRuntimeError> {
        if runtime_selection_reason(&self.profile) == DistroEvidenceReason::ReadinessNotAdmitted {
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

    /// 実socket bindを伴ってproduct runtimeをstartします。
    pub async fn start_with_socket_probe(
        &mut self,
        correlation_id: CorrelationId,
        bind_addr: SocketAddr,
    ) -> Result<(ProductRuntimeOutcome, ProductRuntimeSocketProbe), ProductRuntimeError> {
        if runtime_selection_reason(&self.profile) == DistroEvidenceReason::ReadinessNotAdmitted {
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
        let listener = TcpListener::bind(bind_addr).await.map_err(|_| {
            self.state = DistroRuntimeState::Failed;
            ProductRuntimeError::RuntimeExecutorError
        })?;
        let local_addr = listener.local_addr().map_err(|_| {
            self.state = DistroRuntimeState::Failed;
            ProductRuntimeError::RuntimeExecutorError
        })?;
        // listener を保持する範囲は local runtime probe に限定し、readiness 証明には転用しません。
        self.socket_probe_listener = Some(listener);
        self.state = DistroRuntimeState::Running;
        Ok((
            product_runtime_outcome(correlation_id, self.state, DistroEvidenceReason::DistroOk),
            ProductRuntimeSocketProbe { local_addr },
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
        self.socket_probe_listener = None;
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
        | (ProductHostClass::ControlledMultiProcess, DistroEnvironmentClass::ControlledProcess)
        | (ProductHostClass::ProductionAdmitted, DistroEnvironmentClass::ProductionDeferred)
        | (ProductHostClass::LiveAdmitted, DistroEnvironmentClass::LiveDeferred) => {
            DistroEvidenceReason::DistroOk
        }
        (ProductHostClass::ProductionDeferred, DistroEnvironmentClass::ProductionDeferred)
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
