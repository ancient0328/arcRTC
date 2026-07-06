//! SFU server main は resident loop config を構築し、composition-root へ委譲します。

use arcrtc_entrypoint_composition_root::{
    run_resident_loop, ResidentDriverBindingRef, ResidentLoopFailureKind, ResidentServerKind,
    ResidentServerLoopConfig, RuntimeProfileObservationRef, ShutdownDrainObservationRef,
};

fn main() -> Result<(), ResidentLoopFailureKind> {
    // server main は server kind 固定の refs を作るだけで、SFU route selection を持ちません。
    let config = ResidentServerLoopConfig::new(
        ResidentServerKind::Sfu,
        RuntimeProfileObservationRef::new("runtime-profile:sfu"),
        vec![
            ResidentDriverBindingRef::Network,
            ResidentDriverBindingRef::SfuTransport,
            ResidentDriverBindingRef::Persistence,
            ResidentDriverBindingRef::Observability,
        ],
        ShutdownDrainObservationRef::new("shutdown:sfu"),
        ShutdownDrainObservationRef::new("supervision:sfu"),
    );

    let _resident_loop = run_resident_loop(config)?;
    Ok(())
}
