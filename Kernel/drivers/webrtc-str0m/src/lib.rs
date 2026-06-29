//! drivers/webrtc-str0m は str0m を用いる WebRTC transport driver surface です。
//!
//! core transport contract の意味論を定義せず、後続 task で core-owned port の
//! 具象実装だけを配置します。

use arcrtc_core_ports::{CorePort, PortFamily, WebRtcTransportPort};
use arcrtc_core_quality::{
    ResourceBoundClosedAction, ResourceBoundKind, REQUIRED_RESOURCE_BOUND_CLOSED_ACTIONS,
};
use arcrtc_core_transport::{TransportDriverFailure, WebRtcTransportInput, WebRtcTransportOutput};

/// str0m driver package の所有境界を示す marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Str0mDriverSurface;

/// str0m driver が扱う driver-owned implementation detail です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Str0mOwnedResourceClass {
    /// external library initialization.
    ExternalLibraryInitialization,
    /// byte buffer codec.
    ByteBufferCodec,
    /// buffer pool.
    BufferPool,
    /// buffer lease.
    BufferLease,
    /// bounded packet cache.
    BoundedPacketCache,
    /// transmit queue.
    TransmitQueue,
    /// retry transport detail.
    RetryTransportDetail,
    /// serialization format.
    SerializationFormat,
    /// TLS / platform-specific transport setting.
    TlsPlatformTransportSetting,
}

/// driver-owned resource bound です。unbounded resource を許可しません。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Str0mDriverResourceBound {
    resource_class: Str0mOwnedResourceClass,
    maximum_items: usize,
    audit_owner_tuple_recorded: bool,
    required_closed_action: Option<ResourceBoundClosedAction>,
}

/// driver resource bound の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Str0mDriverResourceBoundError {
    /// resource bound が 0 または未設定です。
    UnboundedResource,
    /// audit owner tuple が記録されていません。
    AuditOwnerTupleMissing,
}

impl Str0mDriverResourceBound {
    /// bounded resource と audit owner tuple を確認して作ります。
    pub fn try_new(
        resource_class: Str0mOwnedResourceClass,
        maximum_items: usize,
        audit_owner_tuple_recorded: bool,
    ) -> Result<Self, Str0mDriverResourceBoundError> {
        if maximum_items == 0 {
            return Err(Str0mDriverResourceBoundError::UnboundedResource);
        }
        if !audit_owner_tuple_recorded {
            return Err(Str0mDriverResourceBoundError::AuditOwnerTupleMissing);
        }
        let required_closed_action = resource_class.required_closed_action();
        Ok(Self {
            resource_class,
            maximum_items,
            audit_owner_tuple_recorded,
            required_closed_action,
        })
    }

    /// core quality catalog 上の required closed action です。
    pub const fn required_closed_action(&self) -> Option<ResourceBoundClosedAction> {
        self.required_closed_action
    }
}

impl Str0mOwnedResourceClass {
    /// core quality canonical に対応する resource kind です。
    pub const fn required_resource_bound_kind(self) -> Option<ResourceBoundKind> {
        match self {
            Self::BoundedPacketCache => Some(ResourceBoundKind::SfuPacketCache),
            Self::TransmitQueue => Some(ResourceBoundKind::SfuTransmitQueue),
            Self::BufferPool | Self::BufferLease => {
                Some(ResourceBoundKind::DriverReceiveBufferPool)
            }
            Self::ExternalLibraryInitialization
            | Self::ByteBufferCodec
            | Self::RetryTransportDetail
            | Self::SerializationFormat
            | Self::TlsPlatformTransportSetting => None,
        }
    }

    /// core quality canonical に対応する exceeded reason code です。
    pub const fn required_resource_reason_code(self) -> Option<&'static str> {
        match self {
            Self::BoundedPacketCache => Some("packet_cache_bound_exceeded"),
            Self::TransmitQueue => Some("sfu_transmit_queue_bound_exceeded"),
            Self::BufferPool | Self::BufferLease => Some("buffer_pool_bound_exceeded"),
            Self::ExternalLibraryInitialization
            | Self::ByteBufferCodec
            | Self::RetryTransportDetail
            | Self::SerializationFormat
            | Self::TlsPlatformTransportSetting => None,
        }
    }

    /// resource-bound 対象 class を core quality catalog の closed action に接続します。
    pub fn required_closed_action(self) -> Option<ResourceBoundClosedAction> {
        let resource = self.required_resource_bound_kind()?;
        let reason_code = self.required_resource_reason_code()?;
        Some(
            REQUIRED_RESOURCE_BOUND_CLOSED_ACTIONS
                .iter()
                .find(|action| action.resource() == resource && action.reason_code() == reason_code)
                .copied()
                .expect("str0m resource bound must be present in core quality catalog"),
        )
    }
}

/// str0m concrete type exposure の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedStr0mTypeExposure {
    /// str0m event type crosses core boundary.
    Str0mEventAsCoreApi,
    /// str0m state type crosses core boundary.
    Str0mStateAsCoreApi,
    /// str0m error type crosses core boundary.
    Str0mErrorAsCoreApi,
    /// SDP / ICE concrete type crosses core boundary.
    SdpIceConcreteTypeAsCoreApi,
    /// driver-owned packet bytes or buffer lease moves to core ownership.
    PacketBytesOrBufferLeaseMovedToCore,
}

/// str0m conversion boundary guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Str0mConversionBoundary {
    external_type_not_exposed: bool,
    core_owned_event_command_only: bool,
    driver_error_mapped_to_catalog: bool,
}

/// str0m conversion boundary の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Str0mConversionBoundaryError {
    /// external concrete type exposure が残っています。
    ExternalTypeExposure,
    /// core-owned event/command 以外を使っています。
    NonCoreOwnedEventCommand,
    /// driver-local error が cataloged reason に接続していません。
    DriverErrorNotMapped,
}

impl Str0mConversionBoundary {
    /// str0m concrete type を core API に出さない conversion boundary を作ります。
    pub const fn try_new(
        external_type_not_exposed: bool,
        core_owned_event_command_only: bool,
        driver_error_mapped_to_catalog: bool,
    ) -> Result<Self, Str0mConversionBoundaryError> {
        if !external_type_not_exposed {
            return Err(Str0mConversionBoundaryError::ExternalTypeExposure);
        }
        if !core_owned_event_command_only {
            return Err(Str0mConversionBoundaryError::NonCoreOwnedEventCommand);
        }
        if !driver_error_mapped_to_catalog {
            return Err(Str0mConversionBoundaryError::DriverErrorNotMapped);
        }
        Ok(Self {
            external_type_not_exposed,
            core_owned_event_command_only,
            driver_error_mapped_to_catalog,
        })
    }
}

/// drivers/webrtc-str0m が core-owned WebRtcTransportPort を実装する marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Str0mTransportDriverPort;

impl CorePort for Str0mTransportDriverPort {
    const FAMILY: PortFamily = PortFamily::WebRtcTransport;
    type Input = WebRtcTransportInput;
    type Output = WebRtcTransportOutput;
    type Error = TransportDriverFailure;
}

impl WebRtcTransportPort for Str0mTransportDriverPort {}

/// WebRTC transport driver 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedStr0mDriverBehavior {
    /// driver reimplements core-owned rule.
    DriverReimplementsCoreRule,
    /// external concrete type crosses core boundary.
    ExternalConcreteTypeCrossesCoreBoundary,
    /// driver-owned packet bytes/cache/queue move to core ownership.
    DriverResourceMovesToCoreOwnership,
    /// direct dependency on other drivers bypasses entrypoints composition.
    DirectCrossDriverDependency,
    /// driver-local error bypasses cataloged reason.
    DriverLocalErrorBypassesCatalogReason,
    /// observability/persistence driver is treated as transport driver.
    NonTransportDriverTreatedAsTransport,
}
