//! drivers/browser は browser platform boundary の driver surface です。
//!
//! browser 固有型を core へ通さず、platform 表現から core-owned type への
//! 変換境界を後続 task で配置します。

use arcrtc_core_ports::PortFamily;
use arcrtc_core_reason::CatalogedReasonRef;

/// browser driver package の所有境界を示す marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrowserDriverSurface;

/// v0.2 initial scope で許可する browser driver responsibility です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BrowserDriverResponsibility {
    /// browser WebSocket / HTTP client binding.
    WebSocketHttpClientBinding,
    /// platform timer / clock binding.
    TimerClockBinding,
    /// platform cancellation/runtime binding.
    RuntimeCancellationBinding,
    /// browser randomness source binding.
    RandomnessBinding,
    /// browser logging / metrics sink binding.
    LoggingMetricsSinkBinding,
    /// browser storage as selected persistence implementation.
    StoragePersistenceBinding,
    /// platform-specific error conversion.
    PlatformErrorConversion,
}

impl BrowserDriverResponsibility {
    /// responsibility に対応する core-owned port family です。
    pub const fn approved_port_family(self) -> Option<PortFamily> {
        match self {
            Self::WebSocketHttpClientBinding => Some(PortFamily::Network),
            Self::TimerClockBinding => Some(PortFamily::Clock),
            Self::RuntimeCancellationBinding => Some(PortFamily::Runtime),
            Self::RandomnessBinding => Some(PortFamily::Random),
            Self::LoggingMetricsSinkBinding => Some(PortFamily::MetricsSink),
            Self::StoragePersistenceBinding => Some(PortFamily::Persistence),
            Self::PlatformErrorConversion => None,
        }
    }
}

/// browser concrete API type の分類です。core signature には出しません。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BrowserConcreteApiType {
    /// browser WebSocket object.
    WebSocket,
    /// browser MessageEvent.
    MessageEvent,
    /// Blob.
    Blob,
    /// ArrayBuffer.
    ArrayBuffer,
    /// ReadableStream.
    ReadableStream,
    /// DOM event object.
    DomEvent,
    /// platform permission result object.
    PlatformPermissionResult,
    /// browser lifecycle callback object.
    PlatformLifecycleCallback,
    /// media device/capture object. initial scope 外です。
    MediaDevice,
    /// PeerConnection object. SDK public abstraction にしません。
    PeerConnection,
}

/// browser platform boundary の admission guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BrowserPlatformBoundaryGuard {
    responsibility: BrowserDriverResponsibility,
    selected_port_family: Option<PortFamily>,
    core_port_defined_by_core: bool,
    concrete_browser_type_absent_from_core_signature: bool,
    platform_input_converted_before_core_call: bool,
    core_decision_semantic_category_preserved: bool,
    sdk_public_contract_not_owned: bool,
    regulated_workflow_not_owned: bool,
    media_ui_workflow_absent: bool,
}

/// browser platform boundary guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BrowserPlatformBoundaryError {
    /// driver が core port trait を定義しています。
    CorePortDefinedByDriver,
    /// responsibility と selected port family が一致しません。
    PortFamilyMismatch,
    /// browser concrete type が core signature に漏れています。
    BrowserConcreteTypeCrossesCore,
    /// platform input が core-owned type へ変換されていません。
    PlatformInputNotConverted,
    /// core decision の semantic category/code を変えています。
    CoreDecisionSemanticsChanged,
    /// SDK public contract を driver が所有しています。
    SdkPublicContractOwnedByDriver,
    /// regulated workflow を driver が所有しています。
    RegulatedWorkflowOwnedByDriver,
    /// media/UI workflow を initial scope に混入しています。
    MediaOrUiWorkflowMixed,
}

impl BrowserPlatformBoundaryGuard {
    /// browser driver が core-owned port への具象接続に留まることを確認します。
    pub const fn try_new(
        responsibility: BrowserDriverResponsibility,
        selected_port_family: Option<PortFamily>,
        core_port_defined_by_core: bool,
        concrete_browser_type_absent_from_core_signature: bool,
        platform_input_converted_before_core_call: bool,
        core_decision_semantic_category_preserved: bool,
        sdk_public_contract_not_owned: bool,
        regulated_workflow_not_owned: bool,
        media_ui_workflow_absent: bool,
    ) -> Result<Self, BrowserPlatformBoundaryError> {
        if !core_port_defined_by_core {
            return Err(BrowserPlatformBoundaryError::CorePortDefinedByDriver);
        }
        if !responsibility.accepts_port_family(selected_port_family) {
            return Err(BrowserPlatformBoundaryError::PortFamilyMismatch);
        }
        if !concrete_browser_type_absent_from_core_signature {
            return Err(BrowserPlatformBoundaryError::BrowserConcreteTypeCrossesCore);
        }
        if !platform_input_converted_before_core_call {
            return Err(BrowserPlatformBoundaryError::PlatformInputNotConverted);
        }
        if !core_decision_semantic_category_preserved {
            return Err(BrowserPlatformBoundaryError::CoreDecisionSemanticsChanged);
        }
        if !sdk_public_contract_not_owned {
            return Err(BrowserPlatformBoundaryError::SdkPublicContractOwnedByDriver);
        }
        if !regulated_workflow_not_owned {
            return Err(BrowserPlatformBoundaryError::RegulatedWorkflowOwnedByDriver);
        }
        if !media_ui_workflow_absent {
            return Err(BrowserPlatformBoundaryError::MediaOrUiWorkflowMixed);
        }

        Ok(Self {
            responsibility,
            selected_port_family,
            core_port_defined_by_core,
            concrete_browser_type_absent_from_core_signature,
            platform_input_converted_before_core_call,
            core_decision_semantic_category_preserved,
            sdk_public_contract_not_owned,
            regulated_workflow_not_owned,
            media_ui_workflow_absent,
        })
    }
}

impl BrowserDriverResponsibility {
    /// responsibility と selected port family の互換を確認します。
    pub const fn accepts_port_family(self, selected_port_family: Option<PortFamily>) -> bool {
        matches!(
            (self, selected_port_family),
            (Self::WebSocketHttpClientBinding, Some(PortFamily::Network))
                | (Self::TimerClockBinding, Some(PortFamily::Clock))
                | (Self::RuntimeCancellationBinding, Some(PortFamily::Runtime))
                | (Self::RandomnessBinding, Some(PortFamily::Random))
                | (
                    Self::LoggingMetricsSinkBinding,
                    Some(PortFamily::MetricsSink)
                )
                | (
                    Self::StoragePersistenceBinding,
                    Some(PortFamily::Persistence)
                )
                | (Self::PlatformErrorConversion, None)
        )
    }
}

/// browser concrete type conversion の guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BrowserTypeConversionGuard {
    concrete_api_type: BrowserConcreteApiType,
    converted_to_core_owned_command_event_reference_or_port_result: bool,
    concrete_type_absent_from_core_signature: bool,
    output_encoded_without_changing_semantic_category: bool,
    platform_specific_error_text_not_authoritative_reason: bool,
}

/// browser type conversion guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BrowserTypeConversionError {
    /// browser concrete type が core に渡っています。
    BrowserConcreteTypeCrossesCore,
    /// core-owned type への変換がありません。
    CoreOwnedConversionMissing,
    /// core decision の semantic category/code を変えています。
    SemanticCategoryChanged,
    /// platform-specific error text が authoritative reason になっています。
    PlatformErrorTextAsReason,
}

impl BrowserTypeConversionGuard {
    /// browser Web API 型を core-owned type へ変換済みか検査します。
    pub const fn try_new(
        concrete_api_type: BrowserConcreteApiType,
        converted_to_core_owned_command_event_reference_or_port_result: bool,
        concrete_type_absent_from_core_signature: bool,
        output_encoded_without_changing_semantic_category: bool,
        platform_specific_error_text_not_authoritative_reason: bool,
    ) -> Result<Self, BrowserTypeConversionError> {
        if !converted_to_core_owned_command_event_reference_or_port_result {
            return Err(BrowserTypeConversionError::CoreOwnedConversionMissing);
        }
        if !concrete_type_absent_from_core_signature {
            return Err(BrowserTypeConversionError::BrowserConcreteTypeCrossesCore);
        }
        if !output_encoded_without_changing_semantic_category {
            return Err(BrowserTypeConversionError::SemanticCategoryChanged);
        }
        if !platform_specific_error_text_not_authoritative_reason {
            return Err(BrowserTypeConversionError::PlatformErrorTextAsReason);
        }

        Ok(Self {
            concrete_api_type,
            converted_to_core_owned_command_event_reference_or_port_result,
            concrete_type_absent_from_core_signature,
            output_encoded_without_changing_semantic_category,
            platform_specific_error_text_not_authoritative_reason,
        })
    }
}

/// browser platform lifecycle observation の分類です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BrowserLifecycleObservation {
    /// browser tab/page visibility.
    TabOrPageVisibility,
    /// browser network availability change.
    NetworkAvailabilityChange,
    /// platform cancellation.
    PlatformCancellation,
    /// platform shutdown.
    PlatformShutdown,
    /// generic DOM/lifecycle callback.
    PlatformLifecycleCallback,
}

/// browser lifecycle observation の mapping guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BrowserLifecycleMappingGuard {
    observation: BrowserLifecycleObservation,
    mapped_only_through_approved_core_event_or_port_result: bool,
    platform_local_policy_does_not_close_room: bool,
    platform_local_policy_does_not_drop_route: bool,
    platform_local_policy_does_not_revoke_permission: bool,
    platform_local_policy_does_not_reject_command: bool,
    driver_shutdown_maps_to_cataloged_reason: bool,
}

/// browser lifecycle mapping guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BrowserLifecycleMappingError {
    /// approved core event/port result 以外で lifecycle を core に入れています。
    ApprovedMappingMissing,
    /// platform lifecycle callback が domain transition を所有しています。
    PlatformLifecycleOwnsDomainTransition,
    /// driver shutdown が cataloged reason に写像されません。
    DriverShutdownReasonMissing,
}

impl BrowserLifecycleMappingGuard {
    /// lifecycle observation が domain transition を所有しないことを確認します。
    pub const fn try_new(
        observation: BrowserLifecycleObservation,
        mapped_only_through_approved_core_event_or_port_result: bool,
        platform_local_policy_does_not_close_room: bool,
        platform_local_policy_does_not_drop_route: bool,
        platform_local_policy_does_not_revoke_permission: bool,
        platform_local_policy_does_not_reject_command: bool,
        driver_shutdown_maps_to_cataloged_reason: bool,
    ) -> Result<Self, BrowserLifecycleMappingError> {
        if !mapped_only_through_approved_core_event_or_port_result {
            return Err(BrowserLifecycleMappingError::ApprovedMappingMissing);
        }
        if !platform_local_policy_does_not_close_room
            || !platform_local_policy_does_not_drop_route
            || !platform_local_policy_does_not_revoke_permission
            || !platform_local_policy_does_not_reject_command
        {
            return Err(BrowserLifecycleMappingError::PlatformLifecycleOwnsDomainTransition);
        }
        if matches!(observation, BrowserLifecycleObservation::PlatformShutdown)
            && !driver_shutdown_maps_to_cataloged_reason
        {
            return Err(BrowserLifecycleMappingError::DriverShutdownReasonMissing);
        }

        Ok(Self {
            observation,
            mapped_only_through_approved_core_event_or_port_result,
            platform_local_policy_does_not_close_room,
            platform_local_policy_does_not_drop_route,
            platform_local_policy_does_not_revoke_permission,
            platform_local_policy_does_not_reject_command,
            driver_shutdown_maps_to_cataloged_reason,
        })
    }
}

/// browser driver failure の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BrowserDriverFailureKind {
    /// platform input cannot map to core type.
    ExternalDecodeFailed,
    /// platform output cannot be encoded.
    ExternalEncodeFailed,
    /// concrete network receive failed.
    NetworkReceiveFailed,
    /// concrete network send failed.
    NetworkSendFailed,
    /// required runtime/platform configuration missing.
    RuntimeConfigMissing,
    /// selected driver cannot initialize.
    RuntimeConfigInvalid,
    /// driver shutdown ended operation.
    DriverShutdown,
}

impl BrowserDriverFailureKind {
    /// cataloged reason code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::ExternalDecodeFailed => "external_decode_failed",
            Self::ExternalEncodeFailed => "external_encode_failed",
            Self::NetworkReceiveFailed => "network_receive_failed",
            Self::NetworkSendFailed => "network_send_failed",
            Self::RuntimeConfigMissing => "runtime_config_missing",
            Self::RuntimeConfigInvalid => "runtime_config_invalid",
            Self::DriverShutdown => "driver_shutdown",
        }
    }
}

/// browser driver failure です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BrowserDriverFailure {
    kind: BrowserDriverFailureKind,
    reason: CatalogedReasonRef,
}

impl BrowserDriverFailure {
    /// browser driver failure を cataloged reason に接続します。
    pub fn from_kind(kind: BrowserDriverFailureKind) -> Self {
        let reason = CatalogedReasonRef::from_code(kind.reason_code())
            .expect("browser driver reason code must be registered");
        Self { kind, reason }
    }
}

/// browser runtime I/O surface です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BrowserRuntimeIoSurface {
    runtime_ref: &'static str,
    event_source_ref: &'static str,
}

impl BrowserRuntimeIoSurface {
    /// browser runtime ref と event source ref を束ねます。
    pub const fn new(runtime_ref: &'static str, event_source_ref: &'static str) -> Self {
        Self {
            runtime_ref,
            event_source_ref,
        }
    }
}

/// browser platform event adapter です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BrowserPlatformEventAdapter {
    adapter_ref: &'static str,
}

impl BrowserPlatformEventAdapter {
    /// adapter ref を保持します。domain command semantics は所有しません。
    pub const fn new(adapter_ref: &'static str) -> Self {
        Self { adapter_ref }
    }
}

/// browser driver failure mapping の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BrowserDriverFailureMapping {
    /// browser runtime が利用できません。
    RuntimeUnavailable,
    /// browser platform event の decode に失敗しました。
    EventDecodeFailed,
    /// browser permission が拒否されました。
    PermissionDenied,
}

/// browser platform event mapping input です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BrowserPlatformEvent {
    surface: BrowserRuntimeIoSurface,
    adapter: BrowserPlatformEventAdapter,
    runtime_available: bool,
    event_decoded: bool,
    permission_granted: bool,
}

impl BrowserPlatformEvent {
    /// browser platform event の mapping 材料を束ねます。
    pub const fn new(
        surface: BrowserRuntimeIoSurface,
        adapter: BrowserPlatformEventAdapter,
        runtime_available: bool,
        event_decoded: bool,
        permission_granted: bool,
    ) -> Self {
        Self {
            surface,
            adapter,
            runtime_available,
            event_decoded,
            permission_granted,
        }
    }
}

/// browser platform event の failure 条件を driver-local closed enum へ写像します。
///
/// browser driver は room/session/domain admission や auth issuance を所有しません。
pub const fn map_browser_platform_event(
    event: BrowserPlatformEvent,
) -> Result<(), BrowserDriverFailureMapping> {
    if event.surface.runtime_ref.is_empty()
        || event.surface.event_source_ref.is_empty()
        || !event.runtime_available
    {
        return Err(BrowserDriverFailureMapping::RuntimeUnavailable);
    }
    if event.adapter.adapter_ref.is_empty() || !event.event_decoded {
        return Err(BrowserDriverFailureMapping::EventDecodeFailed);
    }
    if !event.permission_granted {
        return Err(BrowserDriverFailureMapping::PermissionDenied);
    }
    Ok(())
}

/// browser driver 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedBrowserDriverBehavior {
    /// browser driver defines core port trait.
    BrowserDriverDefinesCorePortTrait,
    /// browser platform API type crosses into core.
    BrowserPlatformTypeCrossesCore,
    /// platform lifecycle callback owns domain transition.
    PlatformLifecycleCallbackOwnsDomainTransition,
    /// browser driver owns SDK public contract semantics.
    BrowserDriverOwnsSdkPublicContract,
    /// browser driver owns regulated workflow.
    BrowserDriverOwnsRegulatedWorkflow,
    /// platform permission/media device state becomes generic core state.
    PlatformPermissionOrMediaDeviceAsCoreState,
    /// platform-specific error text becomes authoritative reason.
    PlatformSpecificErrorTextAsAuthoritativeReason,
    /// out-of-scope browser feature executes without a core admission decision.
    OutOfScopeFeatureExecutedWithoutAdmissionDecision,
}
