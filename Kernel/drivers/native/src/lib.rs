//! drivers/native は native platform binding boundary の driver surface です。
//!
//! OS や mobile platform の具象型を core へ持ち込まず、後続 task で
//! core-owned type への変換境界だけを配置します。

use arcrtc_core_ports::PortFamily;
use arcrtc_core_reason::CatalogedReasonRef;

/// native driver package の所有境界を示す marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeDriverSurface;

/// v0.2 initial scope で許可する native driver responsibility です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeDriverResponsibility {
    /// native WebSocket / HTTP client binding.
    WebSocketHttpClientBinding,
    /// platform timer / clock binding.
    TimerClockBinding,
    /// platform cancellation/runtime binding.
    RuntimeCancellationBinding,
    /// platform randomness source binding.
    RandomnessBinding,
    /// native logging / metrics sink binding.
    LoggingMetricsSinkBinding,
    /// native storage as selected persistence implementation.
    StoragePersistenceBinding,
    /// platform-specific error conversion.
    PlatformErrorConversion,
}

impl NativeDriverResponsibility {
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

/// native concrete platform type の分類です。core signature には出しません。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeConcreteApiType {
    /// Android SDK / Kotlin platform type.
    AndroidSdkOrKotlinType,
    /// iOS Foundation concrete type.
    IosFoundationType,
    /// iOS AVFoundation concrete type.
    IosAvFoundationType,
    /// Apple Network framework concrete type.
    IosNetworkFrameworkType,
    /// platform permission result object.
    PlatformPermissionResult,
    /// mobile entrypoint lifecycle callback object.
    PlatformLifecycleCallback,
    /// media capture/rendering platform object. initial scope 外です。
    MediaDeviceOrTrackObject,
    /// native push notification object. initial scope 外です。
    PushNotificationObject,
}

/// native platform boundary の admission guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NativePlatformBoundaryGuard {
    responsibility: NativeDriverResponsibility,
    selected_port_family: Option<PortFamily>,
    core_port_defined_by_core: bool,
    concrete_native_type_absent_from_core_signature: bool,
    platform_input_converted_before_core_call: bool,
    core_decision_semantic_category_preserved: bool,
    sdk_public_contract_not_owned: bool,
    regulated_workflow_not_owned: bool,
    media_ui_workflow_absent: bool,
}

/// native platform boundary guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativePlatformBoundaryError {
    /// driver が core port trait を定義しています。
    CorePortDefinedByDriver,
    /// responsibility と selected port family が一致しません。
    PortFamilyMismatch,
    /// native concrete type が core signature に漏れています。
    NativeConcreteTypeCrossesCore,
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

impl NativePlatformBoundaryGuard {
    /// native driver が core-owned port への具象接続に留まることを確認します。
    pub const fn try_new(
        responsibility: NativeDriverResponsibility,
        selected_port_family: Option<PortFamily>,
        core_port_defined_by_core: bool,
        concrete_native_type_absent_from_core_signature: bool,
        platform_input_converted_before_core_call: bool,
        core_decision_semantic_category_preserved: bool,
        sdk_public_contract_not_owned: bool,
        regulated_workflow_not_owned: bool,
        media_ui_workflow_absent: bool,
    ) -> Result<Self, NativePlatformBoundaryError> {
        if !core_port_defined_by_core {
            return Err(NativePlatformBoundaryError::CorePortDefinedByDriver);
        }
        if !responsibility.accepts_port_family(selected_port_family) {
            return Err(NativePlatformBoundaryError::PortFamilyMismatch);
        }
        if !concrete_native_type_absent_from_core_signature {
            return Err(NativePlatformBoundaryError::NativeConcreteTypeCrossesCore);
        }
        if !platform_input_converted_before_core_call {
            return Err(NativePlatformBoundaryError::PlatformInputNotConverted);
        }
        if !core_decision_semantic_category_preserved {
            return Err(NativePlatformBoundaryError::CoreDecisionSemanticsChanged);
        }
        if !sdk_public_contract_not_owned {
            return Err(NativePlatformBoundaryError::SdkPublicContractOwnedByDriver);
        }
        if !regulated_workflow_not_owned {
            return Err(NativePlatformBoundaryError::RegulatedWorkflowOwnedByDriver);
        }
        if !media_ui_workflow_absent {
            return Err(NativePlatformBoundaryError::MediaOrUiWorkflowMixed);
        }

        Ok(Self {
            responsibility,
            selected_port_family,
            core_port_defined_by_core,
            concrete_native_type_absent_from_core_signature,
            platform_input_converted_before_core_call,
            core_decision_semantic_category_preserved,
            sdk_public_contract_not_owned,
            regulated_workflow_not_owned,
            media_ui_workflow_absent,
        })
    }
}

/// native concrete type conversion の guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NativeTypeConversionGuard {
    concrete_api_type: NativeConcreteApiType,
    converted_to_core_owned_command_event_reference_or_port_result: bool,
    concrete_type_absent_from_core_signature: bool,
    output_encoded_without_changing_semantic_category: bool,
    platform_specific_error_text_not_authoritative_reason: bool,
}

/// native type conversion guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeTypeConversionError {
    /// native concrete type が core に渡っています。
    NativeConcreteTypeCrossesCore,
    /// core-owned type への変換がありません。
    CoreOwnedConversionMissing,
    /// core decision の semantic category/code を変えています。
    SemanticCategoryChanged,
    /// platform-specific error text が authoritative reason になっています。
    PlatformErrorTextAsReason,
}

impl NativeTypeConversionGuard {
    /// native platform 型を core-owned type へ変換済みか検査します。
    pub const fn try_new(
        concrete_api_type: NativeConcreteApiType,
        converted_to_core_owned_command_event_reference_or_port_result: bool,
        concrete_type_absent_from_core_signature: bool,
        output_encoded_without_changing_semantic_category: bool,
        platform_specific_error_text_not_authoritative_reason: bool,
    ) -> Result<Self, NativeTypeConversionError> {
        if !converted_to_core_owned_command_event_reference_or_port_result {
            return Err(NativeTypeConversionError::CoreOwnedConversionMissing);
        }
        if !concrete_type_absent_from_core_signature {
            return Err(NativeTypeConversionError::NativeConcreteTypeCrossesCore);
        }
        if !output_encoded_without_changing_semantic_category {
            return Err(NativeTypeConversionError::SemanticCategoryChanged);
        }
        if !platform_specific_error_text_not_authoritative_reason {
            return Err(NativeTypeConversionError::PlatformErrorTextAsReason);
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

/// native platform lifecycle observation の分類です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeLifecycleObservation {
    /// mobile entrypoint foreground/background.
    EntrypointForegroundBackground,
    /// native network availability change.
    NetworkAvailabilityChange,
    /// platform cancellation.
    PlatformCancellation,
    /// platform shutdown.
    PlatformShutdown,
    /// generic platform lifecycle callback.
    PlatformLifecycleCallback,
}

/// native lifecycle observation の mapping guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NativeLifecycleMappingGuard {
    observation: NativeLifecycleObservation,
    mapped_only_through_approved_core_event_or_port_result: bool,
    platform_local_policy_does_not_close_room: bool,
    platform_local_policy_does_not_drop_route: bool,
    platform_local_policy_does_not_revoke_permission: bool,
    platform_local_policy_does_not_reject_command: bool,
    driver_shutdown_maps_to_cataloged_reason: bool,
}

/// native lifecycle mapping guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeLifecycleMappingError {
    /// approved core event/port result 以外で lifecycle を core に入れています。
    ApprovedMappingMissing,
    /// platform lifecycle callback が domain transition を所有しています。
    PlatformLifecycleOwnsDomainTransition,
    /// driver shutdown が cataloged reason に写像されません。
    DriverShutdownReasonMissing,
}

impl NativeLifecycleMappingGuard {
    /// lifecycle observation が domain transition を所有しないことを確認します。
    pub const fn try_new(
        observation: NativeLifecycleObservation,
        mapped_only_through_approved_core_event_or_port_result: bool,
        platform_local_policy_does_not_close_room: bool,
        platform_local_policy_does_not_drop_route: bool,
        platform_local_policy_does_not_revoke_permission: bool,
        platform_local_policy_does_not_reject_command: bool,
        driver_shutdown_maps_to_cataloged_reason: bool,
    ) -> Result<Self, NativeLifecycleMappingError> {
        if !mapped_only_through_approved_core_event_or_port_result {
            return Err(NativeLifecycleMappingError::ApprovedMappingMissing);
        }
        if !platform_local_policy_does_not_close_room
            || !platform_local_policy_does_not_drop_route
            || !platform_local_policy_does_not_revoke_permission
            || !platform_local_policy_does_not_reject_command
        {
            return Err(NativeLifecycleMappingError::PlatformLifecycleOwnsDomainTransition);
        }
        if matches!(observation, NativeLifecycleObservation::PlatformShutdown)
            && !driver_shutdown_maps_to_cataloged_reason
        {
            return Err(NativeLifecycleMappingError::DriverShutdownReasonMissing);
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

/// native driver failure の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeDriverFailureKind {
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

impl NativeDriverFailureKind {
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

/// native driver failure です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NativeDriverFailure {
    kind: NativeDriverFailureKind,
    reason: CatalogedReasonRef,
}

impl NativeDriverFailure {
    /// native driver failure を cataloged reason に接続します。
    pub fn from_kind(kind: NativeDriverFailureKind) -> Self {
        let reason = CatalogedReasonRef::from_code(kind.reason_code())
            .expect("native driver reason code must be registered");
        Self { kind, reason }
    }
}

/// native driver initial scope 外の feature です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeOutOfScopeFeature {
    /// camera / microphone capture.
    CameraMicrophoneCapture,
    /// media track rendering.
    MediaTrackRendering,
    /// PeerConnection public SDK abstraction.
    PeerConnectionPublicSdkAbstraction,
    /// screen sharing.
    ScreenSharing,
    /// recording.
    Recording,
    /// chat.
    Chat,
    /// DataChannel application semantics.
    DataChannelApplicationSemantics,
    /// UI / end-user workflow.
    UserInterfaceWorkflow,
    /// push notification workflow.
    PushNotificationWorkflow,
    /// regulated workflow ownership.
    RegulatedWorkflowOwnership,
    /// user account or auth issuance.
    UserAccountOrAuthIssuance,
}

/// native initial scope の out-of-scope admission guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NativeOutOfScopeAdmissionGuard {
    feature: NativeOutOfScopeFeature,
    adr_or_canonical_admission_present: bool,
    feature_absent_from_initial_scope: bool,
}

/// native out-of-scope admission guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeOutOfScopeAdmissionError {
    /// initial scope 外 feature が ADR/Canonical なしで入っています。
    OutOfScopeFeatureAdmitted,
}

impl NativeOutOfScopeAdmissionGuard {
    /// out-of-scope feature を v0.2 initial native driver に混入させない guard です。
    pub const fn try_new(
        feature: NativeOutOfScopeFeature,
        adr_or_canonical_admission_present: bool,
        feature_absent_from_initial_scope: bool,
    ) -> Result<Self, NativeOutOfScopeAdmissionError> {
        if !feature_absent_from_initial_scope && !adr_or_canonical_admission_present {
            return Err(NativeOutOfScopeAdmissionError::OutOfScopeFeatureAdmitted);
        }

        Ok(Self {
            feature,
            adr_or_canonical_admission_present,
            feature_absent_from_initial_scope,
        })
    }
}

/// native driver 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedNativeDriverBehavior {
    /// native driver defines core port trait.
    NativeDriverDefinesCorePortTrait,
    /// native platform API type crosses into core.
    NativePlatformTypeCrossesCore,
    /// platform lifecycle callback owns domain transition.
    PlatformLifecycleCallbackOwnsDomainTransition,
    /// native driver owns SDK public contract semantics.
    NativeDriverOwnsSdkPublicContract,
    /// native driver owns regulated workflow.
    NativeDriverOwnsRegulatedWorkflow,
    /// platform permission/media device state becomes generic core state.
    PlatformPermissionOrMediaDeviceAsCoreState,
    /// platform-specific error text becomes authoritative reason.
    PlatformSpecificErrorTextAsAuthoritativeReason,
    /// out-of-scope native feature is admitted without ADR/Canonical.
    OutOfScopeFeatureWithoutCanonicalAdmission,
}
