impl<Metadata> WireEnvelopeDecodeInput<Metadata> {
    /// wire envelope decode input を作ります。
    pub const fn new(
        external_envelope: ExternalWireEnvelope<Metadata>,
        preconditions: DriverIngressPreconditions,
        semantic_delegation: SemanticDelegationGuard,
        driver_wire_version_supported: bool,
        decoded_fields: DecodedWireEnvelopeFields,
    ) -> Self {
        Self {
            external_envelope,
            preconditions,
            semantic_delegation,
            driver_wire_version_supported,
            decoded_fields,
        }
    }

    /// external wire envelope を core semantic envelope へ変換します。
    pub fn into_core_semantic_envelope(
        self,
    ) -> Result<CoreSemanticEnvelope, DriverConversionFailure> {
        self.preconditions.validate_before_core_entry()?;
        if !self.driver_wire_version_supported {
            return Err(DriverConversionFailure::from_kind(
                DriverConversionFailureKind::UnsupportedDriverWireVersion,
            ));
        }

        let surface = self.decoded_fields.surface.ok_or_else(|| {
            DriverConversionFailure::from_kind(
                DriverConversionFailureKind::MissingRequiredWireField,
            )
        })?;
        let contract_version = self.decoded_fields.contract_version.ok_or_else(|| {
            DriverConversionFailure::from_kind(
                DriverConversionFailureKind::MissingRequiredWireField,
            )
        })?;
        let correlation_id = self
            .decoded_fields
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
        let message_kind = self.decoded_fields.message_kind.ok_or_else(|| {
            DriverConversionFailure::from_kind(
                DriverConversionFailureKind::MissingRequiredWireField,
            )
        })?;
        let message_type = self.decoded_fields.message_type.ok_or_else(|| {
            DriverConversionFailure::from_kind(
                DriverConversionFailureKind::MissingRequiredWireField,
            )
        })?;

        CoreSemanticEnvelope::try_new(
            surface,
            contract_version,
            correlation_id,
            message_kind,
            message_type,
            self.decoded_fields.subject_reference_materialized,
            self.decoded_fields.payload_model,
            None,
            None,
        )
        .map_err(|_error| {
            DriverConversionFailure::from_kind(DriverConversionFailureKind::ExternalDecodeFailed)
        })
    }
}

/// core decision / failure を external response envelope へ encode する前の semantic input です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireEnvelopeEncodeInput<ResponseModel> {
    correlation_id: CorrelationId,
    outcome: UseCaseOutcome,
    reason: Option<CatalogedReasonRef>,
    response_model: ResponseModel,
}

/// wire envelope encode の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WireEnvelopeEncodeError {
    /// non-success response なのに cataloged reason がありません。
    MissingCatalogedReasonForFailure,
    /// success response に driver-created reason を載せています。
    SuccessReasonMustNotBeInvented,
}

impl<ResponseModel> WireEnvelopeEncodeInput<ResponseModel> {
    /// external response encode 前の semantic input を作ります。
    pub fn try_new(
        correlation_id: CorrelationId,
        outcome: UseCaseOutcome,
        reason: Option<CatalogedReasonRef>,
        response_model: ResponseModel,
    ) -> Result<Self, WireEnvelopeEncodeError> {
        if outcome.requires_reason() && reason.is_none() {
            return Err(WireEnvelopeEncodeError::MissingCatalogedReasonForFailure);
        }
        if !outcome.requires_reason() && reason.is_some() {
            return Err(WireEnvelopeEncodeError::SuccessReasonMustNotBeInvented);
        }

        Ok(Self {
            correlation_id,
            outcome,
            reason,
            response_model,
        })
    }
}

/// external wire encoding と canonical encoding の関係です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WireCanonicalEncodingRelation {
    /// external wire format is not canonical encoding.
    ExternalWireIsNotCanonicalEncoding,
    /// deterministic canonical serialization rule is required before digest use.
    RequiresDeterministicCanonicalRule,
}

/// wire envelope 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedWireEnvelopeBehavior {
    /// wire frame shape becomes core API.
    WireFrameShapeAsCoreApi,
    /// core reason category/code is lost in external error mapping.
    CoreReasonLostInExternalErrorMapping,
    /// pre-core conversion failure enters domain state machine.
    PreCoreFailureEntersDomainStateMachine,
    /// correlation rule differs from shared source policy.
    DriverSpecificCorrelationRule,
    /// protocol version semantics are owned by driver.
    DriverOwnsProtocolVersionSemantics,
    /// canonical digest depends on driver wire formatting.
    CanonicalDigestDependsOnWireFormatting,
}

/// external error を投影する surface です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExternalErrorSurface {
    /// HTTP response surface.
    Http,
    /// WebSocket close/error event surface.
    WebSocket,
    /// STUN/TURN error representation surface.
    StunTurn,
    /// SDK wrapper surface.
    Sdk,
    /// CLI exit classification surface.
    Cli,
    /// log/trace diagnostic surface.
    LogsTraces,
}

/// external status / wrapper class です。具体 status number はここでは固定しません。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExternalStatusWrapperClass {
    /// HTTP status family or response wrapper.
    HttpStatusWrapper,
    /// WebSocket close/error wrapper.
    WebSocketErrorWrapper,
    /// STUN/TURN error wrapper.
    StunTurnErrorWrapper,
    /// SDK error wrapper.
    SdkErrorWrapper,
    /// CLI exit wrapper.
    CliExitWrapper,
    /// redacted diagnostic wrapper.
    RedactedDiagnosticWrapper,
}

/// external surface へ露出する reason 形態です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExternalReasonExposure {
    /// safe_to_expose = true の場合だけ category/code を露出します。
    CatalogedCategoryCode {
        /// cataloged core reason reference.
        reason: CatalogedReasonRef,
    },
    /// safe_to_expose = false の場合は opaque reference のみを露出します。
    GenericFailureWithOpaqueReference {
        /// public reason detail ではない opaque error reference.
        opaque_error_ref: OpaqueReference,
    },
}

/// retry hint exposure です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RetryHintExposure {
    /// retry hint を出しません。
    NotExposed,
    /// reason metadata が retryable のときだけ retry hint を出します。
    Exposed,
}

/// audit event relation です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExternalErrorAuditRelation {
    /// audit event relation recorded.
    Recorded,
    /// audit event relation is not required.
    NotRequired,
}

/// external error projection です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternalErrorProjection {
    surface: ExternalErrorSurface,
    wrapper_class: ExternalStatusWrapperClass,
    correlation_id: Option<CorrelationId>,
    authoritative_reason: CatalogedReasonRef,
    exposure: ExternalReasonExposure,
    retry_hint: RetryHintExposure,
    audit_relation: ExternalErrorAuditRelation,
}

/// external error projection 生成時の未検査入力です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternalErrorProjectionInput {
    pub surface: ExternalErrorSurface,
    pub wrapper_class: ExternalStatusWrapperClass,
    pub correlation_id: Option<CorrelationId>,
    pub outcome: UseCaseOutcome,
    pub authoritative_reason: CatalogedReasonRef,
    pub opaque_error_ref: Option<OpaqueReference>,
    pub retry_hint_requested: bool,
    pub audit_relation_recorded: bool,
}

/// external error projection の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExternalErrorProjectionError {
    /// success outcome を external error として投影しようとしています。
    SuccessOutcomeCannotBeExternalError,
    /// unsafe reason に opaque reference がありません。
    UnsafeReasonRequiresOpaqueReference,
    /// non-retryable reason に retry hint を出そうとしています。
    RetryHintNotAllowed,
    /// audit_required reason なのに audit relation がありません。
    AuditRelationRequired,
}

impl ExternalErrorProjection {
    /// core reason metadata に従って external error projection を作ります。
    pub fn try_new(
        input: ExternalErrorProjectionInput,
    ) -> Result<Self, ExternalErrorProjectionError> {
        let ExternalErrorProjectionInput {
            surface,
            wrapper_class,
            correlation_id,
            outcome,
            authoritative_reason,
            opaque_error_ref,
            retry_hint_requested,
            audit_relation_recorded,
        } = input;

        if !outcome.requires_reason() {
            return Err(ExternalErrorProjectionError::SuccessOutcomeCannotBeExternalError);
        }

        let metadata = authoritative_reason.metadata();
        let exposure = if metadata.safe_to_expose() {
            ExternalReasonExposure::CatalogedCategoryCode {
                reason: authoritative_reason,
            }
        } else {
            ExternalReasonExposure::GenericFailureWithOpaqueReference {
                opaque_error_ref: opaque_error_ref
                    .ok_or(ExternalErrorProjectionError::UnsafeReasonRequiresOpaqueReference)?,
            }
        };

        let retry_hint = if retry_hint_requested {
            if !metadata.retryable() {
                return Err(ExternalErrorProjectionError::RetryHintNotAllowed);
            }
            RetryHintExposure::Exposed
        } else {
            RetryHintExposure::NotExposed
        };

        let audit_relation = if metadata.audit_required() {
            if !audit_relation_recorded {
                return Err(ExternalErrorProjectionError::AuditRelationRequired);
            }
            ExternalErrorAuditRelation::Recorded
        } else {
            ExternalErrorAuditRelation::NotRequired
        };

        Ok(Self {
            surface,
            wrapper_class,
            correlation_id,
            authoritative_reason,
            exposure,
            retry_hint,
            audit_relation,
        })
    }
}

/// external error response emission failure の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExternalErrorEmissionFailureKind {
    /// core event cannot be encoded externally.
    ExternalEncodeFailed,
    /// network send failed while emitting error response.
    NetworkSendFailed,
    /// driver shutdown before response emission.
    DriverShutdown,
}

impl ExternalErrorEmissionFailureKind {
    /// emission failure 自体の cataloged reason code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::ExternalEncodeFailed => "external_encode_failed",
            Self::NetworkSendFailed => "network_send_failed",
            Self::DriverShutdown => "driver_shutdown",
        }
    }
}

/// error response emission failure は元の domain reason を置換しません。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExternalErrorEmissionObservation {
    original_reason: CatalogedReasonRef,
    emission_failure: CatalogedReasonRef,
    failure_kind: ExternalErrorEmissionFailureKind,
}

impl ExternalErrorEmissionObservation {
    /// response emission failure を driver observation として記録します。
    pub fn from_kind(
        original_reason: CatalogedReasonRef,
        failure_kind: ExternalErrorEmissionFailureKind,
    ) -> Self {
        let emission_failure = CatalogedReasonRef::from_code(failure_kind.reason_code())
            .expect("external error emission reason code must be registered");
        Self {
            original_reason,
            emission_failure,
            failure_kind,
        }
    }
}

/// external error mapping 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedExternalErrorMappingBehavior {
    /// HTTP status or WebSocket close code replaces core reason.
    ExternalStatusReplacesCoreReason,
    /// unsafe reason detail is exposed because wrapper expects text.
    UnsafeReasonDetailExposed,
    /// retry hint is exposed for non-retryable reason.
    RetryHintForNonRetryableReason,
    /// external response claims success after core non-success decision.
    SuccessProjectionAfterCoreNonSuccess,
    /// SDK invents server reason for local decode failure.
    SdkInventsServerReason,
    /// logs/traces are used as external error authority.
    LogsTracesAsErrorAuthority,
    /// driver exception text becomes public reason.
    DriverExceptionTextAsPublicReason,
}

/// network driver が所有する concrete I/O surface class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NetworkIoSurfaceClass {
    /// UDP socket.
    UdpSocket,
    /// TCP listener or stream.
    TcpSocket,
    /// HTTP endpoint.
    HttpEndpoint,
    /// WebSocket endpoint.
    WebSocketEndpoint,
}

/// network I/O operation の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NetworkIoOperationKind {
    /// listener bind.
    BindListener,
    /// accept connection.
    AcceptConnection,
    /// receive frame.
    ReceiveFrame,
    /// send frame.
    SendFrame,
    /// close connection.
    CloseConnection,
}

/// network I/O local bound です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NetworkIoLocalBound {
    frame_size_bound_bytes: usize,
    connection_concurrency_bound: usize,
}

impl NetworkIoLocalBound {
    /// driver-local frame / concurrency bound を作ります。
    pub const fn new(frame_size_bound_bytes: usize, connection_concurrency_bound: usize) -> Self {
        Self {
            frame_size_bound_bytes,
            connection_concurrency_bound,
        }
    }
}

/// network I/O admission precondition です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NetworkIoPreconditions {
    driver_shutting_down: bool,
    current_connection_count: usize,
    local_bound: NetworkIoLocalBound,
}

impl NetworkIoPreconditions {
    /// network I/O admission precondition を作ります。
    pub const fn new(
        driver_shutting_down: bool,
        current_connection_count: usize,
        local_bound: NetworkIoLocalBound,
    ) -> Self {
        Self {
            driver_shutting_down,
            current_connection_count,
            local_bound,
        }
    }

    /// receive / accept 前に driver-local bound を fail-closed に評価します。
    pub fn validate_connection_admission(&self) -> Result<(), NetworkIoFailure> {
        if self.driver_shutting_down {
            return Err(NetworkIoFailure::from_kind(
                NetworkIoFailureKind::DriverShutdown,
            ));
        }
        if self.current_connection_count >= self.local_bound.connection_concurrency_bound {
            return Err(NetworkIoFailure::from_kind(
                NetworkIoFailureKind::ConnectionConcurrencyExceeded,
            ));
        }
        Ok(())
    }

    /// frame size bound を fail-closed に評価します。
    pub fn validate_frame_size(&self, frame_size_bytes: usize) -> Result<(), NetworkIoFailure> {
        if frame_size_bytes > self.local_bound.frame_size_bound_bytes {
            return Err(NetworkIoFailure::from_kind(
                NetworkIoFailureKind::FrameSizeBoundExceeded,
            ));
        }
        Ok(())
    }
}

/// network I/O failure の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NetworkIoFailureKind {
    /// external payload cannot decode to core type.
    ExternalDecodeFailed,
    /// external wire version unsupported.
    UnsupportedDriverWireVersion,
    /// required wire field absent.
    MissingRequiredWireField,
    /// external enum has no mapping.
    ExternalEnumUnmapped,
    /// inbound frame size bound exceeded.
    FrameSizeBoundExceeded,
    /// connection concurrency bound exceeded.
    ConnectionConcurrencyExceeded,
    /// concrete receive failed.
    NetworkReceiveFailed,
    /// concrete send failed.
    NetworkSendFailed,
    /// driver is shutting down.
    DriverShutdown,
}
