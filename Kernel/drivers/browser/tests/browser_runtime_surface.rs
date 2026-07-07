use arcrtc_driver_browser::{
    map_browser_platform_event, BrowserDriverFailure, BrowserDriverFailureKind,
    BrowserDriverFailureMapping, BrowserPlatformEvent, BrowserPlatformEventAdapter,
    BrowserRuntimeIoSurface,
};

#[test]
fn browser_runtime_surface_maps_valid_event_to_driver_success() {
    let event = BrowserPlatformEvent::new(
        BrowserRuntimeIoSurface::new("browser-runtime", "websocket-event-source"),
        BrowserPlatformEventAdapter::new("browser-event-adapter"),
        true,
        true,
        true,
    );

    // browser driver は platform I/O の写像だけを担い、domain decision を所有しません。
    assert_eq!(map_browser_platform_event(event), Ok(()));
}

#[test]
fn browser_runtime_surface_rejects_unavailable_runtime_with_closed_mapping() {
    let missing_runtime_ref = BrowserPlatformEvent::new(
        BrowserRuntimeIoSurface::new("", "websocket-event-source"),
        BrowserPlatformEventAdapter::new("browser-event-adapter"),
        true,
        true,
        true,
    );
    let unavailable_runtime = BrowserPlatformEvent::new(
        BrowserRuntimeIoSurface::new("browser-runtime", "websocket-event-source"),
        BrowserPlatformEventAdapter::new("browser-event-adapter"),
        false,
        true,
        true,
    );

    assert_eq!(
        map_browser_platform_event(missing_runtime_ref),
        Err(BrowserDriverFailureMapping::RuntimeUnavailable)
    );
    assert_eq!(
        map_browser_platform_event(unavailable_runtime),
        Err(BrowserDriverFailureMapping::RuntimeUnavailable)
    );
}

#[test]
fn browser_runtime_surface_rejects_decode_failure_with_closed_mapping() {
    let missing_adapter_ref = BrowserPlatformEvent::new(
        BrowserRuntimeIoSurface::new("browser-runtime", "websocket-event-source"),
        BrowserPlatformEventAdapter::new(""),
        true,
        true,
        true,
    );
    let undecoded_event = BrowserPlatformEvent::new(
        BrowserRuntimeIoSurface::new("browser-runtime", "websocket-event-source"),
        BrowserPlatformEventAdapter::new("browser-event-adapter"),
        true,
        false,
        true,
    );

    assert_eq!(
        map_browser_platform_event(missing_adapter_ref),
        Err(BrowserDriverFailureMapping::EventDecodeFailed)
    );
    assert_eq!(
        map_browser_platform_event(undecoded_event),
        Err(BrowserDriverFailureMapping::EventDecodeFailed)
    );
}

#[test]
fn browser_runtime_surface_rejects_permission_denial_with_closed_mapping() {
    let permission_denied = BrowserPlatformEvent::new(
        BrowserRuntimeIoSurface::new("browser-runtime", "websocket-event-source"),
        BrowserPlatformEventAdapter::new("browser-event-adapter"),
        true,
        true,
        false,
    );

    assert_eq!(
        map_browser_platform_event(permission_denied),
        Err(BrowserDriverFailureMapping::PermissionDenied)
    );
}

#[test]
fn browser_driver_failure_reason_codes_are_cataloged_closed_values() {
    let cataloged_kinds = [
        BrowserDriverFailureKind::ExternalDecodeFailed,
        BrowserDriverFailureKind::ExternalEncodeFailed,
        BrowserDriverFailureKind::NetworkReceiveFailed,
        BrowserDriverFailureKind::NetworkSendFailed,
        BrowserDriverFailureKind::RuntimeConfigMissing,
        BrowserDriverFailureKind::RuntimeConfigInvalid,
        BrowserDriverFailureKind::DriverShutdown,
    ];

    for kind in cataloged_kinds {
        assert!(!kind.reason_code().is_empty());
        let _failure = BrowserDriverFailure::from_kind(kind);
    }
}
