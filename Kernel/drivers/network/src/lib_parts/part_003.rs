impl NetworkIoFailureKind {
    /// cataloged reason code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::ExternalDecodeFailed => "external_decode_failed",
            Self::UnsupportedDriverWireVersion => "unsupported_driver_wire_version",
            Self::MissingRequiredWireField => "missing_required_wire_field",
            Self::ExternalEnumUnmapped => "external_enum_unmapped",
            Self::FrameSizeBoundExceeded => "frame_size_bound_exceeded",
            Self::ConnectionConcurrencyExceeded => "connection_concurrency_exceeded",
            Self::NetworkReceiveFailed => "network_receive_failed",
            Self::NetworkSendFailed => "network_send_failed",
            Self::DriverShutdown => "driver_shutdown",
        }
    }

    /// resource-bound canonical に接続する failure だけ resource kind を返します。
    pub const fn resource_bound_resource(self) -> Option<ResourceBoundKind> {
        match self {
            Self::FrameSizeBoundExceeded => Some(ResourceBoundKind::InboundFrameSize),
            Self::ConnectionConcurrencyExceeded => Some(ResourceBoundKind::ConnectionConcurrency),
            Self::ExternalDecodeFailed
            | Self::UnsupportedDriverWireVersion
            | Self::MissingRequiredWireField
            | Self::ExternalEnumUnmapped
            | Self::NetworkReceiveFailed
            | Self::NetworkSendFailed
            | Self::DriverShutdown => None,
        }
    }

    /// resource-bound failure に required audit mapping を接続します。
    pub fn resource_bound_decision(self) -> Option<ResourceBoundDecision> {
        let resource = self.resource_bound_resource()?;
        let closed_action = REQUIRED_RESOURCE_BOUND_CLOSED_ACTIONS
            .iter()
            .find(|action| {
                action.resource() == resource && action.reason_code() == self.reason_code()
            })
            .copied()
            .expect("network I/O resource bound must be present in core quality catalog");
        let references = match self {
            // inbound frame size は driver-local resource なので raw bytes ではなく静的 resource ref だけを残します。
            Self::FrameSizeBoundExceeded => {
                ResourceBoundReferenceSet::driver_resource("network_inbound_frame")
            }
            Self::ConnectionConcurrencyExceeded => ResourceBoundReferenceSet::none(),
            _ => return None,
        };

        Some(
            ResourceBoundDecision::try_new(closed_action, references)
                .expect("network I/O resource bound references must satisfy canonical shape"),
        )
    }
}

/// network I/O failure です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NetworkIoFailure {
    kind: NetworkIoFailureKind,
    reason: CatalogedReasonRef,
    resource_bound_decision: Option<Box<ResourceBoundDecision>>,
}

impl NetworkIoFailure {
    /// cataloged reason と接続した network I/O failure を作ります。
    pub fn from_kind(kind: NetworkIoFailureKind) -> Self {
        let reason = CatalogedReasonRef::from_code(kind.reason_code())
            .expect("network I/O failure reason code must be registered");
        let resource_bound_decision = kind.resource_bound_decision().map(Box::new);
        Self {
            kind,
            reason,
            resource_bound_decision,
        }
    }

    /// failure kind です。
    pub const fn kind(&self) -> NetworkIoFailureKind {
        self.kind
    }

    /// cataloged reason reference です。
    pub const fn reason(&self) -> CatalogedReasonRef {
        self.reason
    }

    /// resource-bound failure の場合だけ required closed action mapping を返します。
    pub fn resource_bound_decision(&self) -> Option<&ResourceBoundDecision> {
        self.resource_bound_decision.as_deref()
    }
}

/// inbound network flow の固定順序です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NetworkInboundFlowStep {
    /// driver receives external bytes/frame/request.
    ReceiveExternalFrame,
    /// driver enforces local shape and frame bounds.
    EnforceDriverShapeAndBounds,
    /// driver converts to core-owned command or packet view.
    ConvertToCoreOwnedInput,
    /// core evaluates domain/protocol semantics.
    CoreSemanticEvaluation,
    /// driver maps core result to external output.
    MapCoreResultToExternalOutput,
}

/// core entry path です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NetworkCoreEntryPath {
    /// application use case entry.
    ApplicationUseCase,
    /// core-owned port boundary.
    CoreOwnedPortBoundary,
    /// domain aggregate internals; forbidden.
    DomainAggregateInternal,
}

/// network driver の core entry guard です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NetworkCoreEntryGuard {
    entry_path: NetworkCoreEntryPath,
}

/// core entry guard の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NetworkCoreEntryGuardError {
    /// driver attempted direct domain aggregate entry.
    DomainAggregateEntryForbidden,
}

impl NetworkCoreEntryGuard {
    /// network driver が許可された entry path だけを使うことを確認します。
    pub const fn try_new(
        entry_path: NetworkCoreEntryPath,
    ) -> Result<Self, NetworkCoreEntryGuardError> {
        match entry_path {
            NetworkCoreEntryPath::ApplicationUseCase
            | NetworkCoreEntryPath::CoreOwnedPortBoundary => Ok(Self { entry_path }),
            NetworkCoreEntryPath::DomainAggregateInternal => {
                Err(NetworkCoreEntryGuardError::DomainAggregateEntryForbidden)
            }
        }
    }
}

/// drivers/network が core-owned NetworkPort を実装する marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NetworkIoDriverPort;

impl CorePort for NetworkIoDriverPort {
    const FAMILY: PortFamily = PortFamily::Network;
    type Input = NetworkPortInput;
    type Output = NetworkPortOutput;
    type Error = NetworkIoFailure;
}

impl NetworkPort for NetworkIoDriverPort {}

/// network I/O 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedNetworkIoBehavior {
    /// network driver owns protocol semantics.
    NetworkLayerOwnsProtocolSemantics,
    /// external wire format leaks into core.
    ExternalWireFormatLeaksIntoCore,
    /// core reason is replaced by driver-local status text.
    DriverStatusTextReplacesCoreReason,
    /// resource bound event lacks required audit mapping.
    ResourceBoundEventWithoutAuditMapping,
    /// network driver directly composes other driver implementations.
    DirectCrossDriverComposition,
    /// public/internal endpoint class inferred from route naming alone.
    EndpointClassInferredFromRouteName,
    /// source address, host, origin, or forwarded header becomes core identity.
    NetworkMetadataAsCoreIdentity,
    /// resolved network endpoint becomes semantic owner or authorization authority.
    ResolvedEndpointAsSemanticAuthority,
}

/// driver が decode した TURN/STUN method class です。wire number は core に渡しません。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnWireMethodClass {
    /// Allocate request.
    AllocateRequest,
    /// Refresh request.
    RefreshRequest,
    /// CreatePermission request.
    CreatePermissionRequest,
    /// ChannelBind request.
    ChannelBindRequest,
    /// Send indication.
    SendIndication,
    /// Data indication.
    DataIndication,
    /// driver parser は読めたが core TURN method として未対応です。
    UnsupportedMethod,
}

impl TurnWireMethodClass {
    /// TURN wire method を core-owned command kind へ写します。
    pub fn to_core_command_kind(self) -> Result<TurnCommandKind, TurnWireFailure> {
        match self {
            Self::AllocateRequest => Ok(TurnCommandKind::Allocate),
            Self::RefreshRequest => Ok(TurnCommandKind::Refresh),
            Self::CreatePermissionRequest => Ok(TurnCommandKind::CreatePermission),
            Self::ChannelBindRequest => Ok(TurnCommandKind::ChannelBind),
            Self::SendIndication | Self::DataIndication => Ok(TurnCommandKind::RelayData),
            Self::UnsupportedMethod => Err(TurnWireFailure::simple(
                TurnWireFailureKind::UnsupportedTurnMethod,
            )),
        }
    }
}

/// TURN wire decode 前に driver が判定できる syntax / socket 条件です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TurnWirePreconditions {
    frame_size_bytes: usize,
    frame_size_bound_bytes: usize,
    parser_decoded_message: bool,
    transaction_id_present: bool,
    socket_readable: bool,
    driver_shutting_down: bool,
}

impl TurnWirePreconditions {
    /// TURN wire precondition を作ります。
    pub const fn new(
        frame_size_bytes: usize,
        frame_size_bound_bytes: usize,
        parser_decoded_message: bool,
        transaction_id_present: bool,
        socket_readable: bool,
        driver_shutting_down: bool,
    ) -> Self {
        Self {
            frame_size_bytes,
            frame_size_bound_bytes,
            parser_decoded_message,
            transaction_id_present,
            socket_readable,
            driver_shutting_down,
        }
    }

    /// core TURN lifecycle へ入る前に wire/syntax failure を fail-closed にします。
    pub fn validate_before_core_entry(&self) -> Result<(), TurnWireFailure> {
        if self.driver_shutting_down {
            return Err(TurnWireFailure::simple(
                TurnWireFailureKind::DriverShutdown,
            ));
        }
        if !self.socket_readable {
            return Err(TurnWireFailure::simple(
                TurnWireFailureKind::NetworkReceiveFailed,
            ));
        }
        if self.frame_size_bytes > self.frame_size_bound_bytes {
            return Err(TurnWireFailure::simple(
                TurnWireFailureKind::FrameSizeBoundExceeded,
            ));
        }
        if !self.parser_decoded_message || !self.transaction_id_present {
            return Err(TurnWireFailure::simple(
                TurnWireFailureKind::MalformedTurnMessage,
            ));
        }

        Ok(())
    }
}

/// TURN wire decode 済み attribute の driver-internal field set です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnWireDecodedAttributes {
    transaction_id: Option<UntrustedReference>,
    allocation_ref: Option<UntrustedReference>,
    permission_ref: Option<UntrustedReference>,
    channel_bind_ref: Option<UntrustedReference>,
    credential_ref: Option<UntrustedReference>,
    peer_address: Option<String>,
    requested_lifetime_seconds: Option<u32>,
    relay_packet_ref: Option<UntrustedReference>,
}

impl TurnWireDecodedAttributes {
    /// raw attribute object ではなく、core 型へ変換可能な field だけを保持します。
    pub fn new(
        transaction_id: Option<UntrustedReference>,
        allocation_ref: Option<UntrustedReference>,
        permission_ref: Option<UntrustedReference>,
        channel_bind_ref: Option<UntrustedReference>,
        credential_ref: Option<UntrustedReference>,
        peer_address: Option<String>,
        requested_lifetime_seconds: Option<u32>,
        relay_packet_ref: Option<UntrustedReference>,
    ) -> Self {
        Self {
            transaction_id,
            allocation_ref,
            permission_ref,
            channel_bind_ref,
            credential_ref,
            peer_address,
            requested_lifetime_seconds,
            relay_packet_ref,
        }
    }
}

/// TURN wire decode input です。raw bytes / parser attribute object は保持しません。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnWireDecodeInput {
    preconditions: TurnWirePreconditions,
    method: Option<TurnWireMethodClass>,
    attributes: TurnWireDecodedAttributes,
}

impl TurnWireDecodeInput {
    /// TURN wire decode input を作ります。
    pub const fn new(
        preconditions: TurnWirePreconditions,
        method: Option<TurnWireMethodClass>,
        attributes: TurnWireDecodedAttributes,
    ) -> Self {
        Self {
            preconditions,
            method,
            attributes,
        }
    }

    /// TURN/STUN wire 表現を core-owned TURN command へ変換します。
    pub fn into_core_turn_command(self) -> Result<TurnCommand, TurnWireFailure> {
        self.preconditions.validate_before_core_entry()?;
        let command_kind = self
            .method
            .ok_or_else(|| TurnWireFailure::simple(TurnWireFailureKind::MalformedTurnMessage))?
            .to_core_command_kind()?;

        let transaction_id = self
            .attributes
            .transaction_id
            .ok_or_else(|| TurnWireFailure::simple(TurnWireFailureKind::MalformedTurnMessage))
            .and_then(|reference| {
                accept_reference(reference, ReferenceAuthority::CoreValidatedUntrustedInput)
            })
            .map(TurnTransactionId::new)?;
        let references = TurnReferenceSet::new(
            optional_allocation_id(self.attributes.allocation_ref)?,
            optional_permission_id(self.attributes.permission_ref)?,
            optional_channel_bind_id(self.attributes.channel_bind_ref)?,
            optional_credential_ref(self.attributes.credential_ref)?,
        );
        let peer_address = self
            .attributes
            .peer_address
            .map(CorePeerAddress::new)
            .transpose()
            .map_err(|_error| TurnWireFailure::simple(TurnWireFailureKind::MalformedTurnMessage))?;
        let requested_lifetime = self
            .attributes
            .requested_lifetime_seconds
            .map(TurnRequestedLifetimeSeconds::try_new)
            .transpose()
            .map_err(|_error| TurnWireFailure::simple(TurnWireFailureKind::MalformedTurnMessage))?;
        let relay_packet_id = optional_packet_id(self.attributes.relay_packet_ref)?;

        TurnCommand::try_new(
            command_kind,
            transaction_id,
            references,
            peer_address,
            requested_lifetime,
            relay_packet_id,
        )
        .map_err(|_error| TurnWireFailure::simple(TurnWireFailureKind::MalformedTurnMessage))
    }
}

/// TURN wire driver failure の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnWireFailureKind {
    /// malformed STUN/TURN message or invalid transaction ID.
    MalformedTurnMessage,
    /// unsupported TURN method.
    UnsupportedTurnMethod,
    /// concrete receive failed.
    NetworkReceiveFailed,
    /// concrete send failed.
    NetworkSendFailed,
    /// frame size bound exceeded.
    FrameSizeBoundExceeded,
    /// TURN relay queue bound exceeded.
    TurnRelayQueueBoundExceeded,
    /// driver shutdown.
    DriverShutdown,
}

impl TurnWireFailureKind {
    /// cataloged reason code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::MalformedTurnMessage => TurnFailureKind::MalformedTurnMessage.reason_code(),
            Self::UnsupportedTurnMethod => TurnFailureKind::UnsupportedTurnMethod.reason_code(),
            Self::NetworkReceiveFailed => "network_receive_failed",
            Self::NetworkSendFailed => "network_send_failed",
            Self::FrameSizeBoundExceeded => "frame_size_bound_exceeded",
            Self::TurnRelayQueueBoundExceeded => {
                TurnFailureKind::TurnRelayQueueBoundExceeded.reason_code()
            }
            Self::DriverShutdown => "driver_shutdown",
        }
    }
}

/// TURN wire failure shape error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnWireFailureShapeError {
    /// relay queue bound には active AllocationId / PermissionId が必要です。
    RelayQueueReferencesRequired,
}

/// TURN wire driver failure です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TurnWireFailure {
    kind: TurnWireFailureKind,
    reason: CatalogedReasonRef,
    resource_bound_decision: Option<Box<ResourceBoundDecision>>,
}

impl TurnWireFailure {
    /// relay queue bound 以外の failure を作ります。
    fn simple(kind: TurnWireFailureKind) -> Self {
        let reason = CatalogedReasonRef::from_code(kind.reason_code())
            .expect("TURN wire failure reason code must be registered");
        let resource_bound_decision = match kind {
            TurnWireFailureKind::FrameSizeBoundExceeded => Some(Box::new(resource_bound_decision(
                ResourceBoundKind::InboundFrameSize,
                kind.reason_code(),
                ResourceBoundReferenceSet::driver_resource("turn_wire_frame"),
            ))),
            _ => None,
        };

        Self {
            kind,
            reason,
            resource_bound_decision,
        }
    }

    /// public constructor です。relay queue bound は専用 constructor を使わせます。
    pub fn try_from_kind(kind: TurnWireFailureKind) -> Result<Self, TurnWireFailureShapeError> {
        match kind {
            TurnWireFailureKind::TurnRelayQueueBoundExceeded => {
                Err(TurnWireFailureShapeError::RelayQueueReferencesRequired)
            }
            _ => Ok(Self::simple(kind)),
        }
    }

    /// TURN relay queue bound failure を required active references 付きで作ります。
    pub fn relay_queue_bound(allocation_id: AllocationId, permission_id: PermissionId) -> Self {
        let kind = TurnWireFailureKind::TurnRelayQueueBoundExceeded;
        let reason = CatalogedReasonRef::from_code(kind.reason_code())
            .expect("TURN relay queue bound reason code must be registered");
        let resource_bound_decision = Some(Box::new(resource_bound_decision(
            ResourceBoundKind::TurnRelayQueue,
            kind.reason_code(),
            ResourceBoundReferenceSet::turn_relay_queue(allocation_id, permission_id),
        )));

        Self {
            kind,
            reason,
            resource_bound_decision,
        }
    }

    /// failure kind です。
    pub const fn kind(&self) -> TurnWireFailureKind {
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

/// TURN external response / indication へ encode する外部形状です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnWireOutputClass {
    /// allocation success response.
    AllocationSuccessResponse,
    /// refresh success response.
    RefreshSuccessResponse,
    /// permission success response.
    PermissionSuccessResponse,
    /// channel bind success response.
    ChannelBindSuccessResponse,
    /// relay data forwarding.
    RelayDataForwarding,
    /// TURN error response.
    ErrorResponse,
    /// deny/drop behavior preserving core reason in audit.
    DropOrDeny,
}
