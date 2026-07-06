// core/ports は primary / secondary port の所有 surface です。
//
// driver はここで定義される port を実装しますが、port の所有権は常に
// core 側に残します。

use arcrtc_core_identity::{CorrelationId, OpaqueReference};
use arcrtc_core_protocol::CoreSemanticEnvelope;
use arcrtc_core_quality::{
    QualityMetric, ResourceBoundDecision, ResourceBoundKind, ResourceBoundReferenceSet,
    REQUIRED_RESOURCE_BOUND_CLOSED_ACTIONS,
};
use arcrtc_core_reason::{CatalogedReasonRef, Reason};
use arcrtc_core_state::{StateClass, StateFamily, StatePersistenceFailureKind};

/// core ports package の所有境界を示す marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CorePortsSurface;

/// v0.2 initial architecture で要求される port family です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortFamily {
    /// now、deadline comparison、monotonic time を扱う port です。
    Clock,
    /// nonce、opaque ID、challenge entropy を扱う port です。
    Random,
    /// external token verification result を扱う port です。
    TokenVerifier,
    /// abstract datagram/stream send/receive を扱う port です。
    Network,
    /// WebRTC transport event / command exchange を扱う port です。
    WebRtcTransport,
    /// borrowed packet abstract view / routing input を扱う port です。
    PacketView,
    /// state / audit persistence boundary を扱う port です。
    Persistence,
    /// audit event export を扱う port です。
    AuditSink,
    /// metrics export を扱う port です。
    MetricsSink,
    /// timer / spawn / cancellation abstraction を扱う port です。
    Runtime,
}

/// port call に必ず伝搬する core-owned context です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortCallContext {
    correlation_id: CorrelationId,
}

impl PortCallContext {
    /// correlation ID を持つ call context を作ります。
    pub const fn new(correlation_id: CorrelationId) -> Self {
        Self { correlation_id }
    }

    /// port call の correlation ID です。
    pub const fn correlation_id(&self) -> &CorrelationId {
        &self.correlation_id
    }
}

/// core-owned port interface の共通境界です。
pub trait CorePort {
    /// port family です。
    const FAMILY: PortFamily;

    /// core-owned input type です。
    type Input;

    /// core-owned output type です。
    type Output;

    /// closed error classification へ接続する error type です。
    type Error;
}

/// ClockPort boundary です。
pub trait ClockPort: CorePort {}

/// RandomPort boundary です。
pub trait RandomPort: CorePort {}

/// TokenVerifierPort boundary です。
pub trait TokenVerifierPort: CorePort {}

/// NetworkPort boundary です。
pub trait NetworkPort: CorePort {}

/// core が所有する NetworkPort input です。driver の raw frame / socket 型は含めません。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkPortInput {
    /// core semantic envelope を外部 peer へ送信する intent です。
    SendSemanticEnvelope(CoreSemanticEnvelope),
    /// driver 変換済みの inbound observation を core-owned envelope として渡します。
    ObserveInbound(CoreSemanticEnvelope),
    /// opaque connection reference に対する close intent です。
    CloseConnection(OpaqueReference),
}

/// core が所有する NetworkPort output です。外部 wire 表現は含めません。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkPortOutput {
    /// external delivery の core-facing observation です。
    DeliveryObservation(NetworkDeliveryObservation),
    /// driver 変換済み inbound observation です。
    ConvertedInbound(CoreSemanticEnvelope),
}

/// network delivery observation です。peer は opaque reference としてのみ扱います。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NetworkDeliveryObservation {
    correlation_id: CorrelationId,
    peer_ref: Option<OpaqueReference>,
    delivered: bool,
}

impl NetworkDeliveryObservation {
    /// network delivery observation を作ります。
    pub const fn new(
        correlation_id: CorrelationId,
        peer_ref: Option<OpaqueReference>,
        delivered: bool,
    ) -> Self {
        Self {
            correlation_id,
            peer_ref,
            delivered,
        }
    }

    /// delivery に対応する correlation ID です。
    pub const fn correlation_id(&self) -> &CorrelationId {
        &self.correlation_id
    }

    /// delivery 先 peer の opaque reference です。
    pub const fn peer_ref(&self) -> Option<&OpaqueReference> {
        self.peer_ref.as_ref()
    }

    /// concrete network write が delivery observation として成立したかです。
    pub const fn delivered(&self) -> bool {
        self.delivered
    }
}

/// WebRtcTransportPort boundary です。
pub trait WebRtcTransportPort: CorePort {}

/// PacketViewPort boundary です。driver packet bytes の所有権は受け取りません。
pub trait PacketViewPort: CorePort {}

/// PersistencePort boundary です。
pub trait PersistencePort: CorePort {}

/// AuditSinkPort boundary です。
pub trait AuditSinkPort: CorePort {}

/// MetricsSinkPort boundary です。
pub trait MetricsSinkPort: CorePort {}

/// MetricsSinkPort に送る core-owned input です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetricsSinkInput {
    /// core quality/resource policy に関係する normalized metric です。
    SubmitQualityMetric(QualityMetric),
}

/// metrics export acknowledgement です。domain decision evidence ではありません。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MetricsExportAcknowledgement {
    accepted_by_sink: bool,
    exported: bool,
}

impl MetricsExportAcknowledgement {
    /// metrics sink acknowledgement を作ります。
    pub const fn new(accepted_by_sink: bool, exported: bool) -> Self {
        Self {
            accepted_by_sink,
            exported,
        }
    }
}

/// MetricsSinkPort から返す core-owned output です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricsSinkOutput {
    /// metrics export acknowledgement.
    Acknowledgement(MetricsExportAcknowledgement),
}

/// MetricsSinkPort failure の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricsSinkFailureKind {
    /// metrics export failed.
    MetricsExportFailed,
    /// metrics backlog bound exceeded.
    MetricsBacklogBoundExceeded,
    /// signal taxonomy invalid.
    ObservabilitySignalInvalid,
    /// metric label cardinality exceeded.
    MetricCardinalityExceeded,
    /// required telemetry sampling policy absent.
    TelemetrySamplingPolicyMissing,
    /// driver shutdown.
    DriverShutdown,
}

impl MetricsSinkFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::MetricsExportFailed => "metrics_export_failed",
            Self::MetricsBacklogBoundExceeded => "metrics_backlog_bound_exceeded",
            Self::ObservabilitySignalInvalid => "observability_signal_invalid",
            Self::MetricCardinalityExceeded => "metric_cardinality_exceeded",
            Self::TelemetrySamplingPolicyMissing => "telemetry_sampling_policy_missing",
            Self::DriverShutdown => "driver_shutdown",
        }
    }
}

/// closed reason と resource-bound mapping に接続した MetricsSinkPort failure です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MetricsSinkFailure {
    kind: MetricsSinkFailureKind,
    reason: CatalogedReasonRef,
    resource_bound_decision: Option<Box<ResourceBoundDecision>>,
}

impl MetricsSinkFailure {
    /// metrics sink failure を作ります。
    pub fn from_kind(kind: MetricsSinkFailureKind) -> Self {
        let reason = CatalogedReasonRef::from_code(kind.reason_code())
            .expect("metrics sink failure reason code must be registered");
        let resource_bound_decision = match kind {
            MetricsSinkFailureKind::MetricsBacklogBoundExceeded => {
                Some(Box::new(metrics_resource_bound_decision(kind.reason_code())))
            }
            MetricsSinkFailureKind::MetricsExportFailed
            | MetricsSinkFailureKind::ObservabilitySignalInvalid
            | MetricsSinkFailureKind::MetricCardinalityExceeded
            | MetricsSinkFailureKind::TelemetrySamplingPolicyMissing
            | MetricsSinkFailureKind::DriverShutdown => None,
        };

        Self {
            kind,
            reason,
            resource_bound_decision,
        }
    }

    /// failure kind です。
    pub const fn kind(&self) -> MetricsSinkFailureKind {
        self.kind
    }

    /// cataloged reason reference です。
    pub const fn reason(&self) -> CatalogedReasonRef {
        self.reason
    }

    /// resource-bound failure の required audit mapping です。
    pub fn resource_bound_decision(&self) -> Option<&ResourceBoundDecision> {
        self.resource_bound_decision.as_deref()
    }
}

fn metrics_resource_bound_decision(reason_code: &'static str) -> ResourceBoundDecision {
    let closed_action = REQUIRED_RESOURCE_BOUND_CLOSED_ACTIONS
        .iter()
        .find(|action| {
            action.resource() == ResourceBoundKind::MetricsExportBacklog
                && action.reason_code() == reason_code
        })
        .copied()
        .expect("metrics resource bound must be present in core quality catalog");

    ResourceBoundDecision::try_new(closed_action, ResourceBoundReferenceSet::none())
        .expect("metrics resource bound references must satisfy canonical shape")
}

/// RuntimePort boundary です。concrete task handle は core semantic state に渡しません。
pub trait RuntimePort: CorePort {}

/// port call shape の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortCallShape {
    /// driver に副作用のある command を依頼します。
    Command,
    /// core-owned observation を問い合わせます。
    Query,
    /// sink へ append/submit します。
    SinkSubmit,
    /// driver から core-owned observation stream を受けます。
    StreamObservation,
    /// timer / spawn / cancellation scheduling を依頼します。
    Scheduler,
}

/// port ownership rule の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortOwnershipRule {
    /// external concrete type を signature に含めません。
    CoreOwnedTypesOnly,
    /// driver buffer ownership を core に移しません。
    NoDriverBufferOwnershipTransfer,
    /// concrete runtime handle を domain state に入れません。
    NoConcreteRuntimeHandleTransfer,
}

/// port contract shape summary です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortContractShape {
    family: PortFamily,
    call_shape: PortCallShape,
    ownership_rules: Vec<PortOwnershipRule>,
}

impl PortContractShape {
    /// port family、call shape、ownership rule を固定します。
    pub fn new(
        family: PortFamily,
        call_shape: PortCallShape,
        ownership_rules: Vec<PortOwnershipRule>,
    ) -> Self {
        Self {
            family,
            call_shape,
            ownership_rules,
        }
    }

    /// port family です。
    pub const fn family(&self) -> PortFamily {
        self.family
    }

    /// call shape です。
    pub const fn call_shape(&self) -> PortCallShape {
        self.call_shape
    }

    /// ownership rule 群です。
    pub fn ownership_rules(&self) -> &[PortOwnershipRule] {
        &self.ownership_rules
    }
}

/// port error の分類です。正規 reason は `Reason` が保持します。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortErrorClass {
    /// driver failure reason へ接続します。
    DriverFailure,
    /// bounded resource / backpressure reason へ接続します。
    BoundOrBackpressure,
    /// malformed external observation reason へ接続します。
    ConversionFailure,
    /// runtime config / driver shutdown reason へ接続します。
    RuntimeOrShutdown,
    /// token verification reason へ接続します。
    TokenVerification,
    /// persistence reason へ接続します。
    Persistence,
}

/// closed reason catalog に接続した port error です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortError<Details> {
    class: PortErrorClass,
    reason: Reason<Details>,
}

impl<Details> PortError<Details> {
    /// port error class と cataloged reason を束ねます。
    pub const fn new(class: PortErrorClass, reason: Reason<Details>) -> Self {
        Self { class, reason }
    }

    /// port error class です。
    pub const fn class(&self) -> PortErrorClass {
        self.class
    }

    /// cataloged reason です。
    pub const fn reason(&self) -> &Reason<Details> {
        &self.reason
    }
}

/// port call result です。
pub type PortResult<Output, Details> = Result<Output, PortError<Details>>;

/// PersistencePort intent の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PersistenceIntentClass {
    /// state checkpoint intent です。
    StateCheckpoint,
    /// audit persistence intent です。
    AuditPersistence,
    /// hash-chain record persistence intent です。
    HashChainRecordPersistence,
    /// retry store intent です。
    RetryStore,
}

/// PersistencePort で扱う operation の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PersistenceOperationKind {
    /// checkpoint を保存します。
    PersistCheckpoint,
    /// checkpoint を読み出します。
    LoadCheckpoint,
    /// audit event を append します。
    AppendAuditEvent,
    /// audit hash-chain record を append します。
    AppendHashChainRecord,
    /// retry store に投入します。
    EnqueueRetry,
    /// retry store から読み出します。
    DequeueRetry,
    /// retry store entry を ack します。
    AcknowledgeRetry,
}

/// persistence intent が要求する consistency / retention class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PersistenceConsistencyRequirement {
    /// idempotency semantics を保持する必要があります。
    PreserveIdempotency,
    /// ordered append を保持する必要があります。
    OrderedAppend,
    /// retention policy intent を保持する必要があります。
    RetentionPolicy,
    /// retry store は bounded である必要があります。
    BoundedRetryStore,
}

/// PersistencePort に渡す core-owned intent です。schema/table/key は含めません。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistencePortIntent {
    intent_class: PersistenceIntentClass,
    operation: PersistenceOperationKind,
    state_family: StateFamily,
    state_class: StateClass,
    consistency_requirements: Vec<PersistenceConsistencyRequirement>,
    correlation_id: Option<CorrelationId>,
}

/// PersistencePort intent shape error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PersistencePortIntentError {
    /// state family と state class が state policy と一致していません。
    StateFamilyClassPolicyMismatch,
    /// checkpoint intent なのに checkpoint-eligible state ではありません。
    CheckpointRequiresCheckpointEligibleState,
    /// audit intent なのに audit-only state ではありません。
    AuditIntentRequiresAuditOnlyState,
    /// audit persistence は audit event / compensation evidence に限ります。
    AuditPersistenceRequiresAuditEventState,
    /// hash-chain persistence は audit hash-chain record に限ります。
    HashChainPersistenceRequiresHashChainState,
    /// retry intent なのに driver-local state ではありません。
    RetryIntentRequiresDriverLocalState,
    /// retry intent は driver retry store state family に限ります。
    RetryIntentRequiresDriverRetryStore,
    /// retry store intent に bounded retry requirement がありません。
    RetryIntentRequiresBoundedRetry,
    /// operation と intent class が一致していません。
    OperationClassMismatch,
}
