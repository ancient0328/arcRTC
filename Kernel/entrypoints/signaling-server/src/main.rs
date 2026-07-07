//! Signaling server main は resident loop config を構築し、composition-root へ委譲します。

use arcrtc_entrypoint_composition_root::{
    serve_resident_loop, ResidentDriverBindingRef, ResidentLoopFailureKind, ResidentServerKind,
    ResidentServerLoopConfig, RuntimeProfileObservationRef, ShutdownDrainObservationRef,
};

fn main() -> Result<(), ResidentLoopFailureKind> {
    // server main は server kind 固定の refs を作るだけで、domain 判断や driver policy を持ちません。
    let config = ResidentServerLoopConfig::new(
        ResidentServerKind::Signaling,
        RuntimeProfileObservationRef::new("runtime-profile:signaling"),
        vec![
            ResidentDriverBindingRef::Network,
            ResidentDriverBindingRef::Persistence,
            ResidentDriverBindingRef::Observability,
            ResidentDriverBindingRef::Security,
        ],
        ShutdownDrainObservationRef::new("shutdown:signaling"),
        ShutdownDrainObservationRef::new("supervision:signaling"),
    );

    serve_resident_loop(config)
}
