// drivers/network は socket、wire envelope、TURN wire などの外部 I/O surface です。
//
// 外部 byte / protocol 表現を core-owned type へ変換する境界であり、
// Signaling / SFU / TURN の判断意味論は定義しません。

use arcrtc_core_command::{
    CommandEnvelope, CommandType, CommandVersion, TargetSurface, UseCaseOutcome,
};
use arcrtc_core_identity::{
    AllocationId, ChannelBindId, CorrelationId, CredentialRef, OpaqueReference,
    OpaqueReferenceError, PacketId, PermissionId, ReferenceAuthority, UntrustedReference,
};
use arcrtc_core_ports::{CorePort, NetworkPort, NetworkPortInput, NetworkPortOutput, PortFamily};
use arcrtc_core_protocol::{
    ContractVersion, CoreSemanticEnvelope, CoreSemanticMessageType, CoreSemanticPayloadModel,
    SemanticEnvelopeMessageKind, WireEncodingVersion,
};
use arcrtc_core_quality::{
    ResourceBoundDecision, ResourceBoundKind, ResourceBoundReferenceSet,
    REQUIRED_RESOURCE_BOUND_CLOSED_ACTIONS,
};
use arcrtc_core_reason::{find_reason_definition, CatalogedReasonRef, ReasonDefinition};
use arcrtc_core_turn::{
    CorePeerAddress, TurnCommand, TurnCommandKind, TurnFailureKind, TurnReferenceSet,
    TurnRequestedLifetimeSeconds, TurnTransactionId,
};

/// network driver package の所有境界を示す marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetworkDriverSurface;

/// drivers/network が受け取る framework 非依存の外部 ingress class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExternalIngressKind {
    /// WebSocket message.
    WebSocketMessage,
    /// UDP datagram.
    UdpDatagram,
    /// TCP frame.
    TcpFrame,
    /// HTTP request body.
    HttpRequestBody,
}

/// driver conversion の事前検査結果です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DriverIngressPreconditions {
    ingress_kind: ExternalIngressKind,
    external_payload_decodable: bool,
    frame_size_bytes: usize,
    frame_size_bound_bytes: usize,
    required_wire_fields_present: bool,
    byte_buffer_parseable: bool,
    external_enum_mapped: bool,
    transport_connection_readable: bool,
    transport_connection_writable: bool,
    external_type_leak_blocked: bool,
}

impl DriverIngressPreconditions {
    /// core entry 前に driver が検査できる syntax / transport 条件だけを束ねます。
    pub const fn new(
        ingress_kind: ExternalIngressKind,
        external_payload_decodable: bool,
        frame_size_bytes: usize,
        frame_size_bound_bytes: usize,
        required_wire_fields_present: bool,
        byte_buffer_parseable: bool,
        external_enum_mapped: bool,
        transport_connection_readable: bool,
        transport_connection_writable: bool,
        external_type_leak_blocked: bool,
    ) -> Self {
        Self {
            ingress_kind,
            external_payload_decodable,
            frame_size_bytes,
            frame_size_bound_bytes,
            required_wire_fields_present,
            byte_buffer_parseable,
            external_enum_mapped,
            transport_connection_readable,
            transport_connection_writable,
            external_type_leak_blocked,
        }
    }

    /// syntax / transport 事前検査を fail-closed に評価します。
    pub fn validate_before_core_entry(&self) -> Result<(), DriverConversionFailure> {
        if !self.external_payload_decodable {
            return DriverConversionFailureKind::ExternalDecodeFailed.to_failure();
        }
        if self.frame_size_bytes > self.frame_size_bound_bytes {
            return DriverConversionFailureKind::FrameSizeBoundExceeded.to_failure();
        }
        if !self.required_wire_fields_present {
            return DriverConversionFailureKind::MissingRequiredWireField.to_failure();
        }
        if !self.byte_buffer_parseable {
            return DriverConversionFailureKind::ExternalDecodeFailed.to_failure();
        }
        if !self.external_enum_mapped {
            return DriverConversionFailureKind::ExternalEnumUnmapped.to_failure();
        }
        if !self.transport_connection_readable {
            return DriverConversionFailureKind::NetworkReceiveFailed.to_failure();
        }
        if !self.transport_connection_writable {
            return DriverConversionFailureKind::NetworkSendFailed.to_failure();
        }
        if !self.external_type_leak_blocked {
            return DriverConversionFailureKind::ExternalTypeLeakBlocked.to_failure();
        }

        Ok(())
    }
}

/// driver conversion failure の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DriverConversionFailureKind {
    /// external payload cannot decode.
    ExternalDecodeFailed,
    /// frame size exceeds driver bound.
    FrameSizeBoundExceeded,
    /// unsupported driver wire version.
    UnsupportedDriverWireVersion,
    /// correlation ID field missing.
    MissingCorrelationId,
    /// required wire field missing.
    MissingRequiredWireField,
    /// external enum has no core mapping.
    ExternalEnumUnmapped,
    /// transport connection is not readable.
    NetworkReceiveFailed,
    /// transport connection is not writable.
    NetworkSendFailed,
    /// external type would leak into core.
    ExternalTypeLeakBlocked,
    /// core event cannot encode.
    ExternalEncodeFailed,
}

impl DriverConversionFailureKind {
    /// canonical reason code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::ExternalDecodeFailed => "external_decode_failed",
            Self::FrameSizeBoundExceeded => "frame_size_bound_exceeded",
            Self::UnsupportedDriverWireVersion => "unsupported_driver_wire_version",
            Self::MissingCorrelationId => "missing_correlation_id",
            Self::MissingRequiredWireField => "missing_required_wire_field",
            Self::ExternalEnumUnmapped => "external_enum_unmapped",
            Self::NetworkReceiveFailed => "network_receive_failed",
            Self::NetworkSendFailed => "network_send_failed",
            Self::ExternalTypeLeakBlocked => "external_type_leak_blocked",
            Self::ExternalEncodeFailed => "external_encode_failed",
        }
    }

    /// failure kind を cataloged reason 付き failure に変換します。
    pub fn to_failure(self) -> Result<(), DriverConversionFailure> {
        Err(DriverConversionFailure::from_kind(self))
    }
}

/// conversion failure audit event の種類です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DriverConversionAuditEventType {
    /// driver_error_converted.
    DriverErrorConverted,
    /// driver_resource_bound_decision.
    DriverResourceBoundDecision,
}

/// driver conversion failure です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DriverConversionFailure {
    kind: DriverConversionFailureKind,
    reason: &'static ReasonDefinition,
    audit_event_type: DriverConversionAuditEventType,
    outcome: UseCaseOutcome,
}

impl DriverConversionFailure {
    /// cataloged reason と audit shape を接続します。
    pub fn from_kind(kind: DriverConversionFailureKind) -> Self {
        let reason = find_reason_definition(kind.reason_code())
            .expect("driver conversion reason code must be registered in core reason catalog");
        let (audit_event_type, outcome) = match kind {
            DriverConversionFailureKind::FrameSizeBoundExceeded => (
                DriverConversionAuditEventType::DriverResourceBoundDecision,
                UseCaseOutcome::Dropped,
            ),
            _ => (
                DriverConversionAuditEventType::DriverErrorConverted,
                UseCaseOutcome::ConvertedFailure,
            ),
        };

        Self {
            kind,
            reason,
            audit_event_type,
            outcome,
        }
    }

    /// failure kind です。
    pub const fn kind(&self) -> DriverConversionFailureKind {
        self.kind
    }

    /// cataloged reason definition です。
    pub const fn reason(&self) -> &'static ReasonDefinition {
        self.reason
    }

    /// audit event type です。
    pub const fn audit_event_type(&self) -> DriverConversionAuditEventType {
        self.audit_event_type
    }

    /// audit outcome です。
    pub const fn outcome(&self) -> UseCaseOutcome {
        self.outcome
    }
}

/// protocol / contract semantics を core へ渡すための guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SemanticDelegationGuard {
    preserves_protocol_version_fields: bool,
    preserves_method_identifiers: bool,
    driver_rejects_core_semantic_version: bool,
    driver_rejects_turn_method_semantics: bool,
}

/// semantic delegation guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticDelegationError {
    /// core が判断する version / method field が失われています。
    CoreRequiredSemanticFieldDropped,
    /// driver が core semantic decision を先取りしています。
    DriverPerformedCoreSemanticDecision,
}

impl SemanticDelegationGuard {
    /// driver conversion が core semantic decision を先取りしないことを確認します。
    pub const fn try_new(
        preserves_protocol_version_fields: bool,
        preserves_method_identifiers: bool,
        driver_rejects_core_semantic_version: bool,
        driver_rejects_turn_method_semantics: bool,
    ) -> Result<Self, SemanticDelegationError> {
        if !preserves_protocol_version_fields || !preserves_method_identifiers {
            return Err(SemanticDelegationError::CoreRequiredSemanticFieldDropped);
        }
        if driver_rejects_core_semantic_version || driver_rejects_turn_method_semantics {
            return Err(SemanticDelegationError::DriverPerformedCoreSemanticDecision);
        }

        Ok(Self {
            preserves_protocol_version_fields,
            preserves_method_identifiers,
            driver_rejects_core_semantic_version,
            driver_rejects_turn_method_semantics,
        })
    }
}

/// external field から core command envelope を作る入力です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriverCommandConversionInput<SubjectReferences> {
    preconditions: DriverIngressPreconditions,
    semantic_delegation: SemanticDelegationGuard,
    driver_wire_version_supported: bool,
    correlation_id: Option<UntrustedReference>,
    command_type: Option<CommandType>,
    command_version: Option<CommandVersion>,
    target_surface: Option<TargetSurface>,
    subject_references: SubjectReferences,
}

impl<SubjectReferences> DriverCommandConversionInput<SubjectReferences> {
    /// command conversion input を作ります。
    pub const fn new(
        preconditions: DriverIngressPreconditions,
        semantic_delegation: SemanticDelegationGuard,
        driver_wire_version_supported: bool,
        correlation_id: Option<UntrustedReference>,
        command_type: Option<CommandType>,
        command_version: Option<CommandVersion>,
        target_surface: Option<TargetSurface>,
        subject_references: SubjectReferences,
    ) -> Self {
        Self {
            preconditions,
            semantic_delegation,
            driver_wire_version_supported,
            correlation_id,
            command_type,
            command_version,
            target_surface,
            subject_references,
        }
    }

    /// external ingress を core-owned command envelope に変換します。
    pub fn into_core_command_envelope(
        self,
    ) -> Result<CommandEnvelope<SubjectReferences>, DriverConversionFailure> {
        self.preconditions.validate_before_core_entry()?;
        if !self.driver_wire_version_supported {
            return Err(DriverConversionFailure::from_kind(
                DriverConversionFailureKind::UnsupportedDriverWireVersion,
            ));
        }

        let correlation_id = self
            .correlation_id
            .ok_or_else(|| {
                DriverConversionFailure::from_kind(
                    DriverConversionFailureKind::MissingCorrelationId,
                )
            })
            .and_then(|reference| {
                OpaqueReference::accept_untrusted(
                    reference,
                    ReferenceAuthority::CoreValidatedUntrustedInput,
                )
                .map(CorrelationId::new)
                .map_err(|_error: OpaqueReferenceError| {
                    DriverConversionFailure::from_kind(
                        DriverConversionFailureKind::ExternalDecodeFailed,
                    )
                })
            })?;
        let command_type = self.command_type.ok_or_else(|| {
            DriverConversionFailure::from_kind(
                DriverConversionFailureKind::MissingRequiredWireField,
            )
        })?;
        let command_version = self.command_version.ok_or_else(|| {
            DriverConversionFailure::from_kind(
                DriverConversionFailureKind::MissingRequiredWireField,
            )
        })?;
        let target_surface = self.target_surface.ok_or_else(|| {
            DriverConversionFailure::from_kind(
                DriverConversionFailureKind::MissingRequiredWireField,
            )
        })?;

        Ok(CommandEnvelope::new(
            correlation_id,
            command_type,
            command_version,
            target_surface,
            self.subject_references,
        ))
    }
}

/// driver conversion 後に core へ入れてよい ingress です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreOwnedIngress<SubjectReferences, PacketView> {
    /// command envelope.
    Command(CommandEnvelope<SubjectReferences>),
    /// core-owned packet semantic view.
    PacketView(PacketView),
}

/// driver が判断してはならない semantic decision の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DriverProhibitedSemanticDecision {
    /// Signaling join accepted/rejected.
    SignalingJoinDecision,
    /// TURN allocation accepted/rejected.
    TurnAllocationDecision,
    /// TURN refresh accepted/rejected/expired.
    TurnRefreshDecision,
    /// TURN permission accepted/rejected/revoked.
    TurnPermissionDecision,
    /// TURN channel bind accepted/rejected/expired.
    TurnChannelBindDecision,
    /// SFU route selected/suppressed.
    SfuRouteDecision,
    /// quality violation semantics.
    QualityViolationSemantics,
    /// backpressure policy.
    BackpressurePolicy,
    /// protocol/contract version accept-reject semantics.
    ProtocolContractVersionDecision,
    /// TURN method support semantics.
    TurnMethodSupportDecision,
    /// regulated domain meaning.
    RegulatedDomainMeaning,
    /// SDK public contract semantics.
    SdkPublicContractSemantics,
}

/// core boundary へ露出してはいけない external concrete type です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedExternalTypeExposure {
    /// framework request / response.
    FrameworkRequestResponse,
    /// WebSocket frame type.
    WebSocketFrameType,
    /// tokio socket type.
    TokioSocketType,
    /// str0m event type.
    Str0mEventType,
    /// sqlx row / pool.
    SqlxRowPool,
    /// AWS SDK type.
    AwsSdkType,
    /// browser API type.
    BrowserApiType,
    /// Android / iOS platform type.
    MobilePlatformType,
    /// driver buffer handle.
    DriverBufferHandle,
}

/// driver-owned external wire encoding class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExternalWireEncodingClass {
    /// JSON object/bytes.
    Json,
    /// binary frame.
    Binary,
    /// HTTP method/path/body/status envelope.
    Http,
    /// WebSocket message/close envelope.
    WebSocket,
    /// UDP datagram envelope.
    Udp,
    /// TCP frame envelope.
    Tcp,
    /// STUN/TURN wire message.
    StunTurn,
}

/// external wire envelope は driver-owned であり core API ではありません。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternalWireEnvelope<Metadata> {
    encoding_class: ExternalWireEncodingClass,
    wire_version: WireEncodingVersion,
    metadata: Metadata,
    frame_size_bytes: usize,
}

impl<Metadata> ExternalWireEnvelope<Metadata> {
    /// external wire envelope を driver 内部表現として保持します。
    pub const fn new(
        encoding_class: ExternalWireEncodingClass,
        wire_version: WireEncodingVersion,
        metadata: Metadata,
        frame_size_bytes: usize,
    ) -> Self {
        Self {
            encoding_class,
            wire_version,
            metadata,
            frame_size_bytes,
        }
    }
}

/// decoded wire envelope から semantic envelope へ渡す field set です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedWireEnvelopeFields {
    surface: Option<TargetSurface>,
    contract_version: Option<ContractVersion>,
    correlation_id: Option<UntrustedReference>,
    message_kind: Option<SemanticEnvelopeMessageKind>,
    message_type: Option<CoreSemanticMessageType>,
    subject_reference_materialized: bool,
    payload_model: Option<CoreSemanticPayloadModel>,
}

impl DecodedWireEnvelopeFields {
    /// wire decode 後、core semantic envelope に必要な field set を保持します。
    pub const fn new(
        surface: Option<TargetSurface>,
        contract_version: Option<ContractVersion>,
        correlation_id: Option<UntrustedReference>,
        message_kind: Option<SemanticEnvelopeMessageKind>,
        message_type: Option<CoreSemanticMessageType>,
        subject_reference_materialized: bool,
        payload_model: Option<CoreSemanticPayloadModel>,
    ) -> Self {
        Self {
            surface,
            contract_version,
            correlation_id,
            message_kind,
            message_type,
            subject_reference_materialized,
            payload_model,
        }
    }
}

/// wire envelope decode boundary です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireEnvelopeDecodeInput<Metadata> {
    external_envelope: ExternalWireEnvelope<Metadata>,
    preconditions: DriverIngressPreconditions,
    semantic_delegation: SemanticDelegationGuard,
    driver_wire_version_supported: bool,
    decoded_fields: DecodedWireEnvelopeFields,
}

