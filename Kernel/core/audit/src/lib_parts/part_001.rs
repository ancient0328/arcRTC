// core/audit は audit event と hash-chain contract を所有する surface です。
//
// file、HTTP、syslog、DB などの出力先は driver 側に隔離し、ここでは
// core が参照する audit event model だけを配置します。

use arcrtc_core_command::UseCaseOutcome;
use arcrtc_core_identity::{
    AllocationId, AuditEventId, ChannelBindId, ConfigurationScopeRef, CorrelationId, CredentialRef,
    EndpointId, PacketId, ParticipantId, PermissionId, RoomId, RouteId, SessionId, StartupRunId,
    StreamId,
};
use arcrtc_core_reason::Reason;

/// core audit package の所有境界を示す marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreAuditSurface;

/// closed audit event type code です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AuditEventType(&'static str);

impl AuditEventType {
    /// canonical event type code です。
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

/// audit event type definition です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AuditEventDefinition {
    event_type: AuditEventType,
}

impl AuditEventDefinition {
    const fn new(event_type: &'static str) -> Self {
        Self {
            event_type: AuditEventType(event_type),
        }
    }

    /// closed event type code です。
    pub const fn event_type(&self) -> AuditEventType {
        self.event_type
    }
}

/// canonical に登録された audit event type だけを検索します。
pub fn find_audit_event_definition(code: &str) -> Option<&'static AuditEventDefinition> {
    AUDIT_EVENT_DEFINITIONS
        .iter()
        .find(|definition| definition.event_type().as_str() == code)
}

/// audit event を発生させた component class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuditComponent {
    /// core component です。
    Core,
    /// driver component です。
    Driver,
    /// entrypoint composition component です。
    Entrypoints,
    /// SDK projection component です。
    Sdk,
    /// regulated optional enrichment component です。
    Regulated,
}

/// audit event reason presence です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuditReasonPresence {
    /// success outcome は reason を持ちません。
    None,
    /// non-success outcome は cataloged reason を持ちます。
    Cataloged,
}

/// audit event に添付する reason です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditReason<Details> {
    /// reason なしです。
    None,
    /// closed catalog reason です。
    Cataloged(Reason<Details>),
}

impl<Details> AuditReason<Details> {
    /// reason presence を返します。
    pub const fn presence(&self) -> AuditReasonPresence {
        match self {
            Self::None => AuditReasonPresence::None,
            Self::Cataloged(_) => AuditReasonPresence::Cataloged,
        }
    }
}

/// audit reference の分類です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuditReferenceKind {
    /// event ID です。
    Event,
    /// correlation ID です。
    Correlation,
    /// subject transport reference です。
    SubjectTransport,
    /// room reference です。
    Room,
    /// session reference です。
    Session,
    /// participant reference です。
    Participant,
    /// endpoint reference です。
    Endpoint,
    /// stream reference です。
    Stream,
    /// packet reference です。
    Packet,
    /// route reference です。
    Route,
    /// TURN allocation reference です。
    Allocation,
    /// TURN permission reference です。
    Permission,
    /// TURN channel binding reference です。
    ChannelBind,
    /// credential reference です。
    Credential,
    /// startup run reference です。
    StartupRun,
    /// configuration scope reference です。
    ConfigurationScope,
    /// authorization context class/reference です。
    AuthorizationContext,
    /// media negotiation class/reference です。
    MediaNegotiation,
    /// service topology class/reference です。
    ServiceTopology,
    /// observability signal class/reference です。
    ObservabilitySignal,
    /// dependency/toolchain reference です。
    DependencyToolchain,
    /// SDK platform/projection reference です。
    SdkPlatformProjection,
    /// internal control-plane class/reference です。
    InternalControlPlane,
    /// ICE candidate/connectivity class/reference です。
    IceCandidateConnectivity,
    /// secure media session class/reference です。
    SecureMediaSession,
    /// operator/admin authorization class/reference です。
    OperatorAdminAuthorization,
    /// out-of-scope feature class/reference です。
    OutOfScopeFeature,
    /// public endpoint and connection lifecycle class/reference です。
    PublicEndpointConnection,
    /// export/backup artifact class/reference です。
    ExportBackupArtifact,
    /// release artifact/provenance/distribution class/reference です。
    ReleaseArtifactDistribution,
    /// time synchronization/clock skew class/reference です。
    TimeSynchronizationClockSkew,
    /// edge/proxy trust class/reference です。
    EdgeProxyTrust,
    /// runtime reconfiguration class/reference です。
    RuntimeReconfiguration,
    /// packet rewrite/media transform class/reference です。
    PacketRewriteMediaTransform,
    /// service discovery/endpoint resolution class/reference です。
    ServiceDiscoveryEndpointResolution,
    /// distributed state/failover class/reference です。
    DistributedStateFailover,
    /// runtime task/worker class/reference です。
    RuntimeTaskWorker,
    /// internal service identity/trust class/reference です。
    InternalServiceIdentityTrust,
    /// cross-plane identity/session binding class/reference です。
    CrossPlaneBinding,
}

/// audit reference の値です。generic label は raw secret や raw payload を含めません。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AuditReferenceValue {
    /// correlation ID です。
    CorrelationId(CorrelationId),
    /// room ID です。
    RoomId(RoomId),
    /// session ID です。
    SessionId(SessionId),
    /// participant ID です。
    ParticipantId(ParticipantId),
    /// endpoint ID です。
    EndpointId(EndpointId),
    /// stream ID です。
    StreamId(StreamId),
    /// packet ID です。
    PacketId(PacketId),
    /// route ID です。
    RouteId(RouteId),
    /// TURN allocation ID です。
    AllocationId(AllocationId),
    /// TURN permission ID です。
    PermissionId(PermissionId),
    /// TURN channel binding ID です。
    ChannelBindId(ChannelBindId),
    /// audit event ID です。
    AuditEventId(AuditEventId),
    /// credential reference です。
    CredentialRef(CredentialRef),
    /// startup run ID です。
    StartupRunId(StartupRunId),
    /// configuration scope reference です。
    ConfigurationScopeRef(ConfigurationScopeRef),
    /// non-sensitive class/reference label です。
    Generic(&'static str),
}

/// audit reference presence の closed field です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AuditReferencePresence {
    /// reference が自然に materialized されています。
    Present {
        /// reference kind です。
        kind: AuditReferenceKind,
        /// reference value です。
        value: AuditReferenceValue,
    },
    /// reference が対象 event で自然に存在しないことを示します。
    AbsentNotApplicable(AuditReferenceKind),
}

/// audit event が持つ reference 群です。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AuditReferences {
    entries: Vec<AuditReferencePresence>,
}

impl AuditReferences {
    /// 空の reference set を作ります。
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// reference presence を追加します。
    pub fn push(&mut self, entry: AuditReferencePresence) {
        self.entries.push(entry);
    }

    /// reference presence 一覧です。
    pub fn entries(&self) -> &[AuditReferencePresence] {
        &self.entries
    }
}

/// resource-bound audit event の owner tuple です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceOwnerTuple {
    resource_policy_owner: AuditComponent,
    physical_resource_owner: AuditComponent,
}

impl ResourceOwnerTuple {
    /// resource policy owner と physical resource owner を固定します。
    pub const fn new(
        resource_policy_owner: AuditComponent,
        physical_resource_owner: AuditComponent,
    ) -> Self {
        Self {
            resource_policy_owner,
            physical_resource_owner,
        }
    }

    /// resource policy owner です。
    pub const fn resource_policy_owner(&self) -> AuditComponent {
        self.resource_policy_owner
    }

    /// physical resource owner です。
    pub const fn physical_resource_owner(&self) -> AuditComponent {
        self.physical_resource_owner
    }
}

/// non-sensitive tag です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NonSensitiveTag {
    key: &'static str,
    value: &'static str,
}

impl NonSensitiveTag {
    /// non-sensitive tag を作ります。
    pub const fn new(key: &'static str, value: &'static str) -> Self {
        Self { key, value }
    }

    /// tag key です。
    pub const fn key(&self) -> &'static str {
        self.key
    }

    /// tag value です。
    pub const fn value(&self) -> &'static str {
        self.value
    }
}

/// audit event の timestamp 表現です。時刻正規化の詳細は core/time が所有します。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AuditTimestamp(String);

impl AuditTimestamp {
    /// canonical timestamp label を保持します。
    pub fn new(value: impl Into<String>) -> Result<Self, AuditEventShapeError> {
        let value = value.into();
        if value.is_empty() {
            return Err(AuditEventShapeError::EmptyTimestamp);
        }
        Ok(Self(value))
    }

    /// timestamp value です。
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// audit event model です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEvent<Details> {
    event_id: AuditEventId,
    correlation: AuditReferencePresence,
    timestamp: AuditTimestamp,
    component: AuditComponent,
    event_type: &'static AuditEventDefinition,
    outcome: UseCaseOutcome,
    reason: AuditReason<Details>,
    references: AuditReferences,
    resource_owner: Option<ResourceOwnerTuple>,
    tags: Vec<NonSensitiveTag>,
}

/// audit event 生成時の未検査入力です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEventInput<Details> {
    pub event_id: AuditEventId,
    pub correlation: AuditReferencePresence,
    pub timestamp: AuditTimestamp,
    pub component: AuditComponent,
    pub event_type: &'static AuditEventDefinition,
    pub outcome: UseCaseOutcome,
    pub reason: AuditReason<Details>,
    pub references: AuditReferences,
    pub resource_owner: Option<ResourceOwnerTuple>,
    pub tags: Vec<NonSensitiveTag>,
}

impl<Details> AuditEvent<Details> {
    /// audit event を reason presence rule に従って生成します。
    pub fn new(input: AuditEventInput<Details>) -> Result<Self, AuditEventShapeError> {
        let AuditEventInput {
            event_id,
            correlation,
            timestamp,
            component,
            event_type,
            outcome,
            reason,
            references,
            resource_owner,
            tags,
        } = input;

        match (outcome.requires_reason(), reason.presence()) {
            (false, AuditReasonPresence::Cataloged) => {
                return Err(AuditEventShapeError::SuccessMustNotCarryReason);
            }
            (true, AuditReasonPresence::None) => {
                return Err(AuditEventShapeError::NonSuccessRequiresReason);
            }
            _ => {}
        }

        Ok(Self {
            event_id,
            correlation,
            timestamp,
            component,
            event_type,
            outcome,
            reason,
            references,
            resource_owner,
            tags,
        })
    }

    /// audit event ID です。
    pub const fn event_id(&self) -> &AuditEventId {
        &self.event_id
    }

    /// correlation reference presence です。
    pub const fn correlation(&self) -> &AuditReferencePresence {
        &self.correlation
    }

    /// event type definition です。
    pub const fn event_type(&self) -> &'static AuditEventDefinition {
        self.event_type
    }

    /// decision outcome です。
    pub const fn outcome(&self) -> UseCaseOutcome {
        self.outcome
    }

    /// reason です。
    pub const fn reason(&self) -> &AuditReason<Details> {
        &self.reason
    }
}

/// audit event shape rule 違反です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuditEventShapeError {
    /// success outcome が fake reason を持っています。
    SuccessMustNotCarryReason,
    /// non-success outcome に cataloged reason がありません。
    NonSuccessRequiresReason,
    /// timestamp が空です。
    EmptyTimestamp,
}

/// canonical 由来の closed audit event type definition table です。
pub const AUDIT_EVENT_DEFINITIONS: &[AuditEventDefinition] = &[
    AuditEventDefinition::new("signaling_join_decision"),
    AuditEventDefinition::new("signaling_participant_lifecycle_decision"),
    AuditEventDefinition::new("signaling_room_lifecycle_decision"),
    AuditEventDefinition::new("signaling_protocol_violation"),
    AuditEventDefinition::new("signaling_relay_event"),
    AuditEventDefinition::new("turn_allocation_decision"),
    AuditEventDefinition::new("turn_refresh_decision"),
    AuditEventDefinition::new("turn_permission_decision"),
    AuditEventDefinition::new("turn_channel_bind_decision"),
    AuditEventDefinition::new("turn_relay_decision"),
    AuditEventDefinition::new("sfu_session_lifecycle_decision"),
    AuditEventDefinition::new("sfu_admission_decision"),
    AuditEventDefinition::new("sfu_endpoint_lifecycle_decision"),
    AuditEventDefinition::new("sfu_publication_decision"),
    AuditEventDefinition::new("sfu_subscription_decision"),
    AuditEventDefinition::new("sfu_forwarding_decision"),
    AuditEventDefinition::new("backpressure_decision"),
    AuditEventDefinition::new("resource_bound_decision"),
    AuditEventDefinition::new("driver_resource_bound_decision"),
    AuditEventDefinition::new("configuration_decision"),
    AuditEventDefinition::new("quality_violation_decision"),
    AuditEventDefinition::new("token_verification_decision"),
    AuditEventDefinition::new("driver_error_converted"),
    AuditEventDefinition::new("operational_probe_observation"),
    AuditEventDefinition::new("admin_maintenance_decision"),
    AuditEventDefinition::new("process_lifecycle_observation"),
    AuditEventDefinition::new("atomicity_compensation_decision"),
    AuditEventDefinition::new("canonical_serialization_verification"),
    AuditEventDefinition::new("sdk_reconnect_observation"),
    AuditEventDefinition::new("test_fixture_decision"),
    AuditEventDefinition::new("command_idempotency_decision"),
    AuditEventDefinition::new("authorization_context_decision"),
    AuditEventDefinition::new("deployment_topology_decision"),
    AuditEventDefinition::new("media_negotiation_decision"),
    AuditEventDefinition::new("observability_signal_decision"),
    AuditEventDefinition::new("secret_rotation_decision"),
    AuditEventDefinition::new("supply_chain_decision"),
    AuditEventDefinition::new("sdk_public_api_contract_decision"),
    AuditEventDefinition::new("internal_control_plane_decision"),
    AuditEventDefinition::new("ice_candidate_connectivity_decision"),
    AuditEventDefinition::new("secure_media_session_decision"),
    AuditEventDefinition::new("operator_admin_authorization_decision"),
    AuditEventDefinition::new("out_of_scope_feature_decision"),
    AuditEventDefinition::new("public_endpoint_connection_decision"),
    AuditEventDefinition::new("export_backup_artifact_decision"),
    AuditEventDefinition::new("release_artifact_distribution_decision"),
    AuditEventDefinition::new("time_synchronization_decision"),
    AuditEventDefinition::new("edge_proxy_trust_decision"),
    AuditEventDefinition::new("runtime_reconfiguration_decision"),
    AuditEventDefinition::new("packet_rewrite_transform_decision"),
    AuditEventDefinition::new("service_discovery_resolution_decision"),
    AuditEventDefinition::new("distributed_state_failover_decision"),
    AuditEventDefinition::new("runtime_task_lifecycle_decision"),
    AuditEventDefinition::new("internal_service_trust_decision"),
    AuditEventDefinition::new("cross_plane_binding_decision"),
];

/// v0.2 initial architecture で許可された hash-chain scope です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HashChainScope {
    /// startup / configuration / wiring decisions の chain です。
    Startup,
    /// Signaling decisions and protocol violations の chain です。
    Signaling,
    /// SFU decisions の chain です。
    Sfu,
    /// TURN decisions の chain です。
    Turn,
    /// driver conversion/resource/failure events の chain です。
    Driver,
}

/// hash-chain sequence number です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HashChainSequence(u64);

