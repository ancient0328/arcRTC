//! entrypoints/composition-root は resident server loop の composition root です。
//!
//! ここでは published contract/ref を接続するだけで、domain rule、protocol semantics、
//! reason vocabulary、policy、ledger write は定義しません。

pub use arcrtc_entrypoint_admin::ShutdownDrainObservationRef;
pub use arcrtc_entrypoint_configuration::RuntimeProfileObservationRef;

/// resident server kind の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResidentServerKind {
    /// Signaling server.
    Signaling,
    /// TURN server.
    Turn,
    /// SFU server.
    Sfu,
}

/// resident loop が接続する driver binding reference の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResidentDriverBindingRef {
    /// network driver binding.
    Network,
    /// TURN relay driver binding.
    TurnRelay,
    /// SFU transport driver binding.
    SfuTransport,
    /// persistence driver binding.
    Persistence,
    /// observability driver binding.
    Observability,
    /// security driver binding.
    Security,
}

/// resident loop construction failure の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResidentLoopFailureKind {
    /// required config/ref is missing.
    ConfigMissing,
    /// required driver binding ref is missing.
    DriverBindingMissing,
    /// runtime start was rejected by published observation boundary.
    RuntimeStartRejected,
    /// shutdown drain observation rejected resident start.
    ShutdownDrainRejected,
    /// supervision observation is missing or failed.
    SupervisionObservationFailed,
}

/// resident server loop config です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResidentServerLoopConfig {
    server_kind: ResidentServerKind,
    runtime_profile_ref: RuntimeProfileObservationRef,
    driver_binding_refs: Vec<ResidentDriverBindingRef>,
    shutdown_ref: ShutdownDrainObservationRef,
    supervision_ref: ShutdownDrainObservationRef,
}

impl ResidentServerLoopConfig {
    /// resident server loop に必要な refs を束ねます。
    pub fn new(
        server_kind: ResidentServerKind,
        runtime_profile_ref: RuntimeProfileObservationRef,
        driver_binding_refs: Vec<ResidentDriverBindingRef>,
        shutdown_ref: ShutdownDrainObservationRef,
        supervision_ref: ShutdownDrainObservationRef,
    ) -> Self {
        Self {
            server_kind,
            runtime_profile_ref,
            driver_binding_refs,
            shutdown_ref,
            supervision_ref,
        }
    }

    /// server kind です。
    pub const fn server_kind(&self) -> ResidentServerKind {
        self.server_kind
    }

    /// driver binding refs です。
    pub fn driver_binding_refs(&self) -> &[ResidentDriverBindingRef] {
        &self.driver_binding_refs
    }
}

/// resident loop delegation observation です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResidentLoopObservation {
    server_kind: ResidentServerKind,
}

impl ResidentLoopObservation {
    /// resident loop delegation observation を作ります。
    pub const fn new(server_kind: ResidentServerKind) -> Self {
        Self { server_kind }
    }

    /// server kind です。
    pub const fn server_kind(&self) -> ResidentServerKind {
        self.server_kind
    }
}

/// resident loop の published ref 接続を行います。
///
/// 実 runtime start や driver policy selection は行わず、composition root の config/ref 境界だけを検査します。
pub fn run_resident_loop(
    config: ResidentServerLoopConfig,
) -> Result<ResidentLoopObservation, ResidentLoopFailureKind> {
    if config.runtime_profile_ref.as_str().is_empty() {
        return Err(ResidentLoopFailureKind::ConfigMissing);
    }
    if config.driver_binding_refs.is_empty() {
        return Err(ResidentLoopFailureKind::DriverBindingMissing);
    }
    if config.shutdown_ref.as_str().is_empty() {
        return Err(ResidentLoopFailureKind::ShutdownDrainRejected);
    }
    if config.supervision_ref.as_str().is_empty() {
        return Err(ResidentLoopFailureKind::SupervisionObservationFailed);
    }

    Ok(ResidentLoopObservation::new(config.server_kind))
}
