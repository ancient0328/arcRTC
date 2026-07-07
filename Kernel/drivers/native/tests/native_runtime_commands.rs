use arcrtc_driver_native::{
    map_native_platform_error, NativeDriverFailure, NativeDriverFailureKind,
    NativeLifecycleSurface, NativeMediaBridgeSurface, NativePlatformError,
    NativePlatformErrorMapping, NativeRuntimeCommand,
};

fn valid_native_platform_error() -> NativePlatformError {
    NativePlatformError::new(
        NativeRuntimeCommand::new("native-network-command", "native-lifecycle-command"),
        NativeLifecycleSurface::new("native-lifecycle-surface"),
        NativeMediaBridgeSurface::new("native-media-bridge"),
        true,
        true,
        true,
        true,
    )
}

#[test]
fn native_runtime_command_surface_accepts_valid_network_media_config_lifecycle() {
    // native driver は platform command の失敗分類だけを担い、domain acceptance は所有しません。
    assert_eq!(
        map_native_platform_error(valid_native_platform_error()),
        Ok(())
    );
}

#[test]
fn native_runtime_command_surface_rejects_network_failures_with_closed_mapping() {
    let missing_command_ref = NativePlatformError::new(
        NativeRuntimeCommand::new("", "native-lifecycle-command"),
        NativeLifecycleSurface::new("native-lifecycle-surface"),
        NativeMediaBridgeSurface::new("native-media-bridge"),
        true,
        true,
        true,
        true,
    );
    let network_unavailable = NativePlatformError::new(
        NativeRuntimeCommand::new("native-network-command", "native-lifecycle-command"),
        NativeLifecycleSurface::new("native-lifecycle-surface"),
        NativeMediaBridgeSurface::new("native-media-bridge"),
        false,
        true,
        true,
        true,
    );

    assert_eq!(
        map_native_platform_error(missing_command_ref),
        Err(NativePlatformErrorMapping::NetworkUnavailable)
    );
    assert_eq!(
        map_native_platform_error(network_unavailable),
        Err(NativePlatformErrorMapping::NetworkUnavailable)
    );
}

#[test]
fn native_runtime_command_surface_rejects_media_failures_with_closed_mapping() {
    let missing_media_bridge_ref = NativePlatformError::new(
        NativeRuntimeCommand::new("native-network-command", "native-lifecycle-command"),
        NativeLifecycleSurface::new("native-lifecycle-surface"),
        NativeMediaBridgeSurface::new(""),
        true,
        true,
        true,
        true,
    );
    let media_bridge_unavailable = NativePlatformError::new(
        NativeRuntimeCommand::new("native-network-command", "native-lifecycle-command"),
        NativeLifecycleSurface::new("native-lifecycle-surface"),
        NativeMediaBridgeSurface::new("native-media-bridge"),
        true,
        false,
        true,
        true,
    );

    assert_eq!(
        map_native_platform_error(missing_media_bridge_ref),
        Err(NativePlatformErrorMapping::MediaBridgeFailed)
    );
    assert_eq!(
        map_native_platform_error(media_bridge_unavailable),
        Err(NativePlatformErrorMapping::MediaBridgeFailed)
    );
}

#[test]
fn native_runtime_command_surface_rejects_config_failure_with_closed_mapping() {
    let config_rejected = NativePlatformError::new(
        NativeRuntimeCommand::new("native-network-command", "native-lifecycle-command"),
        NativeLifecycleSurface::new("native-lifecycle-surface"),
        NativeMediaBridgeSurface::new("native-media-bridge"),
        true,
        true,
        false,
        true,
    );

    assert_eq!(
        map_native_platform_error(config_rejected),
        Err(NativePlatformErrorMapping::ConfigRejected)
    );
}

#[test]
fn native_runtime_command_surface_rejects_lifecycle_failures_with_closed_mapping() {
    let missing_command_lifecycle = NativePlatformError::new(
        NativeRuntimeCommand::new("native-network-command", ""),
        NativeLifecycleSurface::new("native-lifecycle-surface"),
        NativeMediaBridgeSurface::new("native-media-bridge"),
        true,
        true,
        true,
        true,
    );
    let missing_lifecycle_surface = NativePlatformError::new(
        NativeRuntimeCommand::new("native-network-command", "native-lifecycle-command"),
        NativeLifecycleSurface::new(""),
        NativeMediaBridgeSurface::new("native-media-bridge"),
        true,
        true,
        true,
        true,
    );
    let lifecycle_ref_invalid = NativePlatformError::new(
        NativeRuntimeCommand::new("native-network-command", "native-lifecycle-command"),
        NativeLifecycleSurface::new("native-lifecycle-surface"),
        NativeMediaBridgeSurface::new("native-media-bridge"),
        true,
        true,
        true,
        false,
    );

    assert_eq!(
        map_native_platform_error(missing_command_lifecycle),
        Err(NativePlatformErrorMapping::LifecycleRejected)
    );
    assert_eq!(
        map_native_platform_error(missing_lifecycle_surface),
        Err(NativePlatformErrorMapping::LifecycleRejected)
    );
    assert_eq!(
        map_native_platform_error(lifecycle_ref_invalid),
        Err(NativePlatformErrorMapping::LifecycleRejected)
    );
}

#[test]
fn native_driver_failure_reason_codes_are_cataloged_closed_values() {
    let cataloged_kinds = [
        NativeDriverFailureKind::ExternalDecodeFailed,
        NativeDriverFailureKind::ExternalEncodeFailed,
        NativeDriverFailureKind::NetworkReceiveFailed,
        NativeDriverFailureKind::NetworkSendFailed,
        NativeDriverFailureKind::RuntimeConfigMissing,
        NativeDriverFailureKind::RuntimeConfigInvalid,
        NativeDriverFailureKind::DriverShutdown,
    ];

    for kind in cataloged_kinds {
        assert!(!kind.reason_code().is_empty());
        let _failure = NativeDriverFailure::from_kind(kind);
    }
}
