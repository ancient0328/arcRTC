//! TURN server main は resident loop config を構築し、composition-root へ委譲します。

use arcrtc_entrypoint_composition_root::{
    serve_resident_loop, ResidentDriverBindingRef, ResidentLoopFailureKind, ResidentServerKind,
    ResidentServerLoopConfig, RuntimeProfileObservationRef, ShutdownDrainObservationRef,
};

fn main() -> Result<(), ResidentLoopFailureKind> {
    // server main は server kind 固定の refs を作るだけで、TURN lifecycle 判断を持ちません。
    let config = ResidentServerLoopConfig::new(
        ResidentServerKind::Turn,
        RuntimeProfileObservationRef::new("runtime-profile:turn"),
        vec![
            ResidentDriverBindingRef::Network,
            ResidentDriverBindingRef::TurnRelay,
            ResidentDriverBindingRef::Persistence,
            ResidentDriverBindingRef::Observability,
            ResidentDriverBindingRef::Security,
        ],
        ShutdownDrainObservationRef::new("shutdown:turn"),
        ShutdownDrainObservationRef::new("supervision:turn"),
    );

    serve_resident_loop(config)
}
