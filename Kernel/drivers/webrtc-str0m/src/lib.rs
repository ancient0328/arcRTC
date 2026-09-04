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

    /// この bound が許可する最大 item 数です。
    pub const fn maximum_items(&self) -> usize {
        self.maximum_items
    }
}

impl Str0mOwnedResourceClass {
    /// core quality source contract に対応する resource kind です。
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

    /// core quality source contract に対応する exceeded reason code です。
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

/// str0m の Sans I/O engine を driver 境界内に閉じ込める wrapper です。
pub struct Str0mMediaEngine {
    rtc: str0m::Rtc,
    transmit_queue_bound: Str0mDriverResourceBound,
}

impl Str0mMediaEngine {
    /// str0m engine と driver-owned transmit queue bound を初期化します。
    pub fn new(now: std::time::Instant) -> Self {
        Self {
            rtc: str0m::Rtc::new(now),
            transmit_queue_bound: Str0mDriverResourceBound::try_new(
                Str0mOwnedResourceClass::TransmitQueue,
                1024,
                true,
            )
            .expect("transmit queue bound arguments are fixed"),
        }
    }

    /// UDP datagram を str0m の受信入力へ変換して投入します。
    pub fn ingest_udp_datagram(
        &mut self,
        received_at: std::time::Instant,
        source: std::net::SocketAddr,
        local_addr: std::net::SocketAddr,
        datagram: &[u8],
    ) -> Result<(), TransportDriverFailure> {
        let receive =
            str0m::net::Receive::new(str0m::net::Protocol::Udp, source, local_addr, datagram)
                .map_err(|_error| {
                    TransportDriverFailure::from_kind(
                        arcrtc_core_transport::TransportDriverFailureKind::ExternalDecodeFailed,
                    )
                })?;

        self.rtc
            .handle_input(str0m::Input::Receive(received_at, receive))
            .map_err(|_error| {
                TransportDriverFailure::from_kind(
                    arcrtc_core_transport::TransportDriverFailureKind::ExternalDecodeFailed,
                )
            })?;

        Ok(())
    }

    /// str0m の出力を Timeout まで drain し、送信可能 datagram だけを返します。
    pub fn drain_outbound(&mut self) -> Result<Str0mDrainReport, TransportDriverFailure> {
        let mut outbound = Vec::new();
        let mut dropped_over_bound = 0usize;

        for _ in 0..4096 {
            match self.rtc.poll_output() {
                Ok(str0m::Output::Timeout(_timeout)) => {
                    return Ok(Str0mDrainReport {
                        outbound,
                        dropped_over_bound,
                    });
                }
                Ok(str0m::Output::Transmit(transmit)) => {
                    if outbound.len() < self.transmit_queue_bound.maximum_items() {
                        outbound.push(Str0mOutboundDatagram {
                            destination: transmit.destination,
                            payload: Vec::from(&transmit.contents[..]),
                        });
                    } else {
                        dropped_over_bound += 1;
                    }
                }
                Ok(str0m::Output::Event(_event)) => {}
                Err(_error) => {
                    return Err(TransportDriverFailure::from_kind(
                        arcrtc_core_transport::TransportDriverFailureKind::ExternalEncodeFailed,
                    ));
                }
            }
        }

        Err(TransportDriverFailure::from_kind(
            arcrtc_core_transport::TransportDriverFailureKind::DriverShutdown,
        ))
    }
}

/// entrypoint が実送信する driver-owned outbound datagram です。
pub struct Str0mOutboundDatagram {
    destination: std::net::SocketAddr,
    payload: Vec<u8>,
}

impl Str0mOutboundDatagram {
    /// 実送信先の socket address です。
    pub const fn destination(&self) -> std::net::SocketAddr {
        self.destination
    }

    /// str0m が生成した送信 payload です。
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

/// str0m drain の結果です。payload bytes は driver-owned のまま保持します。
pub struct Str0mDrainReport {
    outbound: Vec<Str0mOutboundDatagram>,
    dropped_over_bound: usize,
}

impl Str0mDrainReport {
    /// transmit queue bound 内で収集できた outbound datagram です。
    pub fn outbound(&self) -> &[Str0mOutboundDatagram] {
        &self.outbound
    }

    /// transmit queue bound 超過で enqueue しなかった datagram 件数です。
    pub const fn dropped_over_bound(&self) -> usize {
        self.dropped_over_bound
    }
}

/// multi-datagram outbound polling の入力です。
pub struct Str0mOutboundBatchInput<'session> {
    session: &'session mut Str0mMediaEngine,
}

impl<'session> Str0mOutboundBatchInput<'session> {
    /// driver-owned str0m session wrapper を batch polling 対象にします。
    pub const fn new(session: &'session mut Str0mMediaEngine) -> Self {
        Self { session }
    }
}

/// outbound datagram collection の安定 digest です。
pub type Str0mOutboundBatchDigest = u64;

/// driver-owned outbound datagram batch です。
pub struct Str0mOutboundBatch {
    datagrams: Vec<Str0mOutboundDatagram>,
    digest: Str0mOutboundBatchDigest,
}

impl Str0mOutboundBatch {
    /// collected datagrams と digest を保持します。
    pub fn new(datagrams: Vec<Str0mOutboundDatagram>, digest: Str0mOutboundBatchDigest) -> Self {
        Self { datagrams, digest }
    }

    /// entrypoint が実送信する datagram collection です。
    pub fn datagrams(&self) -> &[Str0mOutboundDatagram] {
        &self.datagrams
    }

    /// collection 内容から計算した driver-local digest です。
    pub const fn digest(&self) -> Str0mOutboundBatchDigest {
        self.digest
    }
}

/// str0m session polling failure の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Str0mDriverSessionFailure {
    /// str0m polling が core-owned transport failure に写像されました。
    PollFailed(TransportDriverFailure),
    /// driver-owned outbound collection が bound を超過しました。
    OutboundBoundExceeded {
        dropped_over_bound: usize,
        transport_failure: TransportDriverFailure,
    },
}

impl Str0mDriverSessionFailure {
    /// driver-local failure を core-facing transport failure へ戻します。
    pub const fn transport_failure(self) -> TransportDriverFailure {
        match self {
            Self::PollFailed(failure) => failure,
            Self::OutboundBoundExceeded {
                transport_failure, ..
            } => transport_failure,
        }
    }

    /// bound 超過で drop された datagram 件数です。
    pub const fn dropped_over_bound(self) -> usize {
        match self {
            Self::PollFailed(_) => 0,
            Self::OutboundBoundExceeded {
                dropped_over_bound, ..
            } => dropped_over_bound,
        }
    }
}

/// str0m session から複数 outbound datagram を一括 polling する driver contract です。
pub struct Str0mMultiDatagramSessionDriver;

impl Str0mMultiDatagramSessionDriver {
    /// outbound datagram batch を取得し、bound 超過や polling 失敗を core transport failure に写像します。
    ///
    /// driver は datagram 収集だけを担当し、media authorization / participant admission / route selection は判断しません。
    pub fn poll_outbound_batch(
        input: Str0mOutboundBatchInput<'_>,
    ) -> Result<Str0mOutboundBatch, Str0mDriverSessionFailure> {
        let report = input
            .session
            .drain_outbound()
            .map_err(Str0mDriverSessionFailure::PollFailed)?;

        if report.dropped_over_bound() > 0 {
            return Err(Str0mDriverSessionFailure::OutboundBoundExceeded {
                dropped_over_bound: report.dropped_over_bound(),
                transport_failure: TransportDriverFailure::from_kind(
                    arcrtc_core_transport::TransportDriverFailureKind::MediaPayloadMappingInvalid,
                ),
            });
        }

        let digest = calculate_outbound_batch_digest(&report.outbound);
        Ok(Str0mOutboundBatch::new(report.outbound, digest))
    }
}

fn calculate_outbound_batch_digest(
    datagrams: &[Str0mOutboundDatagram],
) -> Str0mOutboundBatchDigest {
    let mut digest = 0xcbf2_9ce4_8422_2325_u64;
    digest = mix_outbound_digest(digest, &(datagrams.len() as u64).to_be_bytes());

    for datagram in datagrams {
        digest = match datagram.destination.ip() {
            std::net::IpAddr::V4(addr) => mix_outbound_digest(digest, &addr.octets()),
            std::net::IpAddr::V6(addr) => mix_outbound_digest(digest, &addr.octets()),
        };
        digest = mix_outbound_digest(digest, &datagram.destination.port().to_be_bytes());
        digest = mix_outbound_digest(digest, &datagram.payload);
    }

    digest
}

fn mix_outbound_digest(mut digest: Str0mOutboundBatchDigest, bytes: &[u8]) -> u64 {
    for byte in bytes {
        digest ^= u64::from(*byte);
        digest = digest.wrapping_mul(0x0000_0100_0000_01b3);
    }
    digest
}
