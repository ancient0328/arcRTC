//! core/cross-plane は Signaling / SFU / TURN / transport をまたぐ binding policy surface です。
//!
//! 個別 plane の成功を別 plane の許可として暗黙採用せず、
//! core-owned binding decision に必要な参照、lifecycle、audit 境界だけを定義します。

use arcrtc_core_identity::{
    AllocationId, ChannelBindId, CorrelationId, CredentialRef, EndpointId, OpaqueReference,
    ParticipantId, PermissionId, RoomId, SessionId, StreamId,
};
use arcrtc_core_security::AuthorizationContextClass;

/// core cross-plane package の所有境界を示す marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreCrossPlaneSurface;

/// v0.2 initial architecture が認める cross-plane binding class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CrossPlaneBindingClass {
    /// single-plane claim のため binding は不要です。
    NoCrossPlaneBindingRequired,
    /// Signaling room participant を target plane reference に結びます。
    SignalingParticipantBinding,
    /// SFU endpoint/session を Signaling participant/session context に結びます。
    SfuEndpointBinding,
    /// TURN allocation を許可済み communication context に結びます。
    TurnAllocationBinding,
    /// TURN permission/channel bind を relay context に結びます。
    TurnPermissionBinding,
    /// ICE candidate を room/participant と optional TURN relation に結びます。
    IceCandidateBinding,
    /// secure media observation を SFU endpoint/session scope に結びます。
    SecureMediaSessionBinding,
    /// deterministic/fake binding です。test evidence 以外には使いません。
    TestCrossPlaneBinding,
    /// 暗黙 binding 要求です。常に rejected 扱いです。
    ImplicitBindingRequested,
}

impl CrossPlaneBindingClass {
    /// implicit binding として fail-closed にする class です。
    pub const fn is_rejected_class(self) -> bool {
        match self {
            Self::ImplicitBindingRequested => true,
            Self::NoCrossPlaneBindingRequired
            | Self::SignalingParticipantBinding
            | Self::SfuEndpointBinding
            | Self::TurnAllocationBinding
            | Self::TurnPermissionBinding
            | Self::IceCandidateBinding
            | Self::SecureMediaSessionBinding
            | Self::TestCrossPlaneBinding => false,
        }
    }

    /// explicit binding materialization に source/target reference が必要な class です。
    pub const fn requires_materialized_references(self) -> bool {
        match self {
            Self::NoCrossPlaneBindingRequired => false,
            Self::SignalingParticipantBinding
            | Self::SfuEndpointBinding
            | Self::TurnAllocationBinding
            | Self::TurnPermissionBinding
            | Self::IceCandidateBinding
            | Self::SecureMediaSessionBinding
            | Self::TestCrossPlaneBinding
            | Self::ImplicitBindingRequested => true,
        }
    }
}

/// cross-plane が扱う plane の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CrossPlane {
    /// Signaling plane.
    Signaling,
    /// SFU plane.
    Sfu,
    /// TURN plane.
    Turn,
    /// ICE/transport plane.
    IceTransport,
    /// secure media plane.
    SecureMedia,
}

/// binding reference の lifecycle class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BindingReferenceLifecycle {
    /// room lifecycle.
    Room,
    /// room participant membership lifecycle.
    ParticipantMembership,
    /// transport/session lifecycle.
    Session,
    /// SFU endpoint lifecycle.
    SfuEndpoint,
    /// media stream lifecycle.
    MediaStream,
    /// TURN allocation lifecycle.
    TurnAllocation,
    /// TURN permission lifecycle.
    TurnPermission,
    /// TURN channel bind lifecycle.
    TurnChannelBind,
    /// credential verification lifecycle.
    CredentialVerification,
    /// materialized reference がまだ存在しません。
    NotMaterialized,
}

/// binding の source/target reference です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CrossPlaneReference {
    /// materialized reference がない、または不要です。
    NotMaterialized,
    /// Signaling room reference.
    Room(RoomId),
    /// communication session reference.
    Session(SessionId),
    /// Signaling participant reference.
    Participant(ParticipantId),
    /// SFU endpoint reference.
    Endpoint(EndpointId),
    /// media stream reference.
    Stream(StreamId),
    /// TURN allocation reference.
    Allocation(AllocationId),
    /// TURN permission reference.
    Permission(PermissionId),
    /// TURN channel bind reference.
    ChannelBind(ChannelBindId),
    /// credential relation reference.
    Credential(CredentialRef),
}

impl CrossPlaneReference {
    /// reference の lifecycle を返します。
    pub const fn lifecycle(&self) -> BindingReferenceLifecycle {
        match self {
            Self::NotMaterialized => BindingReferenceLifecycle::NotMaterialized,
            Self::Room(_) => BindingReferenceLifecycle::Room,
            Self::Session(_) => BindingReferenceLifecycle::Session,
            Self::Participant(_) => BindingReferenceLifecycle::ParticipantMembership,
            Self::Endpoint(_) => BindingReferenceLifecycle::SfuEndpoint,
            Self::Stream(_) => BindingReferenceLifecycle::MediaStream,
            Self::Allocation(_) => BindingReferenceLifecycle::TurnAllocation,
            Self::Permission(_) => BindingReferenceLifecycle::TurnPermission,
            Self::ChannelBind(_) => BindingReferenceLifecycle::TurnChannelBind,
            Self::Credential(_) => BindingReferenceLifecycle::CredentialVerification,
        }
    }
}

/// source/target plane の lifecycle precondition です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BindingLifecyclePrecondition {
    /// no cross-plane state is required.
    NotApplicable,
    /// participant joined.
    ParticipantJoined,
    /// endpoint admitted.
    EndpointAdmitted,
    /// allocation active.
    AllocationActive,
    /// permission active.
    PermissionActive,
    /// channel bind active.
    ChannelBindActive,
    /// candidate policy accepted.
    CandidatePolicyAccepted,
    /// secure media protection active.
    SecureMediaProtectionActive,
}

/// binding expiry/revocation behavior です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BindingExpiryBehavior {
    /// no expiry behavior is required.
    NotApplicable,
    /// new target plane action must be rejected.
    RejectNewTargetPlaneAction,
    /// target plane must close through its own state machine.
    TargetPlaneClosesThroughOwnStateMachine,
    /// expired relation must be recorded as close-not-claimed for affected claim.
    RecordCloseNotClaimed,
}

/// command-scoped binding の replay/idempotency relation です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BindingReplayRelation {
    /// command scoped ではありません。
    NotCommandScoped,
    /// same command identity may observe prior accepted binding.
    PriorAcceptedBindingObservable,
    /// replay/idempotency conflict must be rejected.
    ConflictRejected,
}

/// cross-plane binding rule の core-owned typed policy です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CrossPlaneBindingPolicy {
    binding_class: CrossPlaneBindingClass,
    source_plane: CrossPlane,
    target_plane: CrossPlane,
    source_reference: CrossPlaneReference,
    target_reference: CrossPlaneReference,
    authorization_context: Option<AuthorizationContextClass>,
    source_precondition: BindingLifecyclePrecondition,
    target_precondition: BindingLifecyclePrecondition,
    expiry_behavior: BindingExpiryBehavior,
    replay_relation: BindingReplayRelation,
}

impl CrossPlaneBindingPolicy {
    /// binding に必要な必須 field をすべて持つ policy を作ります。
    pub const fn new(
        binding_class: CrossPlaneBindingClass,
        source_plane: CrossPlane,
        target_plane: CrossPlane,
        source_reference: CrossPlaneReference,
        target_reference: CrossPlaneReference,
        authorization_context: Option<AuthorizationContextClass>,
        source_precondition: BindingLifecyclePrecondition,
        target_precondition: BindingLifecyclePrecondition,
        expiry_behavior: BindingExpiryBehavior,
        replay_relation: BindingReplayRelation,
    ) -> Self {
        Self {
            binding_class,
            source_plane,
            target_plane,
            source_reference,
            target_reference,
            authorization_context,
            source_precondition,
            target_precondition,
            expiry_behavior,
            replay_relation,
        }
    }

    /// binding class です。
    pub const fn binding_class(&self) -> CrossPlaneBindingClass {
        self.binding_class
    }
}

/// cross-plane binding decision outcome です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CrossPlaneBindingOutcome {
    /// accepted.
    Accepted,
    /// rejected.
    Rejected,
    /// expired.
    Expired,
    /// failed.
    Failed,
    /// claim does not adopt cross-plane evidence.
    CloseNotClaimed,
}

impl CrossPlaneBindingOutcome {
    /// audit outcome code です。
    pub const fn code(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Expired => "expired",
            Self::Failed => "failed",
            Self::CloseNotClaimed => "close_not_claimed",
        }
    }

    /// cataloged reason が必須の outcome です。
    pub const fn requires_reason(self) -> bool {
        match self {
            Self::Accepted => false,
            Self::Rejected | Self::Expired | Self::Failed | Self::CloseNotClaimed => true,
        }
    }
}

/// cross-plane binding failure/reason mapping key です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CrossPlaneBindingFailureKind {
    /// binding class is not admitted.
    BindingClassNotAdmitted,
    /// required binding is absent.
    RequiredBindingAbsent,
    /// binding material cannot map to declared references.
    BindingMaterialInvalid,
    /// source/target scope conflicts.
    SourceTargetScopeConflict,
    /// source or target lifecycle state is not valid for binding.
    LifecycleConflict,
    /// binding lifetime or source relation expired.
    BindingExpired,
    /// binding replay/idempotency conflict detected.
    BindingReplayDetected,
    /// target SFU participant/endpoint is not admitted.
    ParticipantNotAdmitted,
    /// target unavailable.
    TargetUnavailable,
    /// target TURN allocation absent.
    AllocationNotFound,
    /// target TURN permission absent.
    PermissionNotFound,
    /// target secure media protection is not active.
    SecureMediaProtectionNotActive,
}

impl CrossPlaneBindingFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::BindingClassNotAdmitted => "cross_plane_binding_not_admitted",
            Self::RequiredBindingAbsent => "cross_plane_binding_missing",
            Self::BindingMaterialInvalid => "cross_plane_binding_invalid",
            Self::SourceTargetScopeConflict => "cross_plane_binding_scope_conflict",
            Self::LifecycleConflict => "cross_plane_binding_lifecycle_conflict",
            Self::BindingExpired => "cross_plane_binding_expired",
            Self::BindingReplayDetected => "cross_plane_binding_replay_detected",
            Self::ParticipantNotAdmitted => "participant_not_admitted",
            Self::TargetUnavailable => "target_unavailable",
            Self::AllocationNotFound => "allocation_not_found",
            Self::PermissionNotFound => "permission_not_found",
            Self::SecureMediaProtectionNotActive => "secure_media_protection_not_active",
        }
    }
}

/// cross-plane binding materialization の入力です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CrossPlaneBindingMaterializationInput {
    source_plane_ref: CrossPlaneReference,
    target_plane_ref: CrossPlaneReference,
    binding_class: CrossPlaneBindingClass,
    authorization_ref: Option<AuthorizationContextClass>,
}

impl CrossPlaneBindingMaterializationInput {
    /// source/target plane reference と binding class を束ねます。
    pub fn new(
        source_plane_ref: CrossPlaneReference,
        target_plane_ref: CrossPlaneReference,
        binding_class: CrossPlaneBindingClass,
        authorization_ref: Option<AuthorizationContextClass>,
    ) -> Self {
        Self {
            source_plane_ref,
            target_plane_ref,
            binding_class,
            authorization_ref,
        }
    }

    /// binding class です。
    pub const fn binding_class(&self) -> CrossPlaneBindingClass {
        self.binding_class
    }

    /// source plane reference です。
    pub const fn source_plane_ref(&self) -> &CrossPlaneReference {
        &self.source_plane_ref
    }

    /// target plane reference です。
    pub const fn target_plane_ref(&self) -> &CrossPlaneReference {
        &self.target_plane_ref
    }

    /// authorization reference です。
    pub const fn authorization_ref(&self) -> Option<AuthorizationContextClass> {
        self.authorization_ref
    }
}

/// cross-plane binding decision の payload です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CrossPlaneBindingDecisionRecord {
    correlation_id: Option<CorrelationId>,
    policy: Option<CrossPlaneBindingPolicy>,
    materialization_input: Option<CrossPlaneBindingMaterializationInput>,
    outcome: CrossPlaneBindingOutcome,
    reason: Option<CrossPlaneBindingFailureKind>,
}

impl CrossPlaneBindingDecisionRecord {
    /// legacy audit projection と materialization projection の共通 payload を作ります。
    pub fn new(
        correlation_id: Option<CorrelationId>,
        policy: Option<CrossPlaneBindingPolicy>,
        materialization_input: Option<CrossPlaneBindingMaterializationInput>,
        outcome: CrossPlaneBindingOutcome,
        reason: Option<CrossPlaneBindingFailureKind>,
    ) -> Self {
        Self {
            correlation_id,
            policy,
            materialization_input,
            outcome,
            reason,
        }
    }

    /// outcome です。
    pub const fn outcome(&self) -> CrossPlaneBindingOutcome {
        self.outcome
    }

    /// rejected/failed reason です。
    pub const fn reason(&self) -> Option<CrossPlaneBindingFailureKind> {
        self.reason
    }

    /// legacy audit projection correlation id です。
    pub const fn correlation_id(&self) -> Option<&CorrelationId> {
        self.correlation_id.as_ref()
    }

    /// legacy binding policy です。
    pub const fn policy(&self) -> Option<&CrossPlaneBindingPolicy> {
        self.policy.as_ref()
    }

    /// materialization input です。
    pub const fn materialization_input(&self) -> Option<&CrossPlaneBindingMaterializationInput> {
        self.materialization_input.as_ref()
    }
}

/// cross-plane binding decision の閉集合です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CrossPlaneBindingDecision {
    /// explicit binding materialization が成立しました。
    Materialized(CrossPlaneBindingDecisionRecord),
    /// binding materialization は閉じた failure reason で拒否されました。
    Rejected(CrossPlaneBindingDecisionRecord),
}

impl CrossPlaneBindingDecision {
    /// cross_plane_binding_decision に投影できる decision を作ります。
    pub fn new(
        correlation_id: CorrelationId,
        policy: CrossPlaneBindingPolicy,
        outcome: CrossPlaneBindingOutcome,
        reason: Option<CrossPlaneBindingFailureKind>,
    ) -> Self {
        let record = CrossPlaneBindingDecisionRecord::new(
            Some(correlation_id),
            Some(policy),
            None,
            outcome,
            reason,
        );
        match outcome {
            CrossPlaneBindingOutcome::Accepted => Self::Materialized(record),
            CrossPlaneBindingOutcome::Rejected
            | CrossPlaneBindingOutcome::Expired
            | CrossPlaneBindingOutcome::Failed
            | CrossPlaneBindingOutcome::CloseNotClaimed => Self::Rejected(record),
        }
    }

    /// audit event type code です。
    pub const fn audit_event_type(&self) -> &'static str {
        "cross_plane_binding_decision"
    }

    /// decision payload です。
    pub const fn record(&self) -> &CrossPlaneBindingDecisionRecord {
        match self {
            Self::Materialized(record) | Self::Rejected(record) => record,
        }
    }
}

/// explicit binding だけを materialize し、implicit binding は fail-closed で拒否します。
///
/// この関数は cross-plane relation の決定だけを返し、各 plane の state machine は変更しません。
pub fn materialize_cross_plane_binding(
    input: CrossPlaneBindingMaterializationInput,
) -> CrossPlaneBindingDecision {
    if input.binding_class.is_rejected_class() {
        return CrossPlaneBindingDecision::Rejected(CrossPlaneBindingDecisionRecord::new(
            None,
            None,
            Some(input),
            CrossPlaneBindingOutcome::Rejected,
            Some(CrossPlaneBindingFailureKind::BindingClassNotAdmitted),
        ));
    }

    if input.binding_class.requires_materialized_references()
        && (matches!(input.source_plane_ref, CrossPlaneReference::NotMaterialized)
            || matches!(input.target_plane_ref, CrossPlaneReference::NotMaterialized))
    {
        return CrossPlaneBindingDecision::Rejected(CrossPlaneBindingDecisionRecord::new(
            None,
            None,
            Some(input),
            CrossPlaneBindingOutcome::Rejected,
            Some(CrossPlaneBindingFailureKind::RequiredBindingAbsent),
        ));
    }

    CrossPlaneBindingDecision::Materialized(CrossPlaneBindingDecisionRecord::new(
        None,
        None,
        Some(input),
        CrossPlaneBindingOutcome::Accepted,
        None,
    ))
}

/// cross-plane audit relation の opaque typed reference です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CrossPlaneAuditRelationRef(OpaqueReference);

impl CrossPlaneAuditRelationRef {
    /// accepted opaque reference から audit relation reference を作ります。
    pub fn new(value: OpaqueReference) -> Self {
        Self(value)
    }

    /// opaque value です。document path ではありません。
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// cross-plane evidence relation の opaque typed reference です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CrossPlaneEvidenceRelationRef(OpaqueReference);

impl CrossPlaneEvidenceRelationRef {
    /// accepted opaque reference から evidence relation reference を作ります。
    pub fn new(value: OpaqueReference) -> Self {
        Self(value)
    }

    /// opaque value です。document path ではありません。
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// cross-plane decision event の opaque typed reference です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CrossPlaneDecisionEventRef(OpaqueReference);

impl CrossPlaneDecisionEventRef {
    /// accepted opaque reference から decision event reference を作ります。
    pub fn new(value: OpaqueReference) -> Self {
        Self(value)
    }

    /// opaque value です。document path ではありません。
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// cross-plane binding に attach できる relation reference の閉集合です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CrossPlaneRelationRef {
    /// audit relation reference です。
    Audit(CrossPlaneAuditRelationRef),
    /// evidence relation reference です。
    Evidence(CrossPlaneEvidenceRelationRef),
    /// decision event reference です。
    DecisionEvent(CrossPlaneDecisionEventRef),
}

/// cross-plane binding relation attachment です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CrossPlaneRelationAttachment {
    binding_ref: CrossPlaneEvidenceRelationRef,
    relation_ref: CrossPlaneRelationRef,
}

impl CrossPlaneRelationAttachment {
    /// binding reference と attached relation を保持します。
    pub fn new(
        binding_ref: CrossPlaneEvidenceRelationRef,
        relation_ref: CrossPlaneRelationRef,
    ) -> Self {
        Self {
            binding_ref,
            relation_ref,
        }
    }

    /// binding reference です。
    pub const fn binding_ref(&self) -> &CrossPlaneEvidenceRelationRef {
        &self.binding_ref
    }

    /// relation reference です。
    pub const fn relation_ref(&self) -> &CrossPlaneRelationRef {
        &self.relation_ref
    }
}

/// cross-plane binding と audit/evidence/decision relation を opaque ref だけで接続します。
pub fn attach_cross_plane_relation(
    binding_ref: CrossPlaneEvidenceRelationRef,
    relation_ref: CrossPlaneRelationRef,
) -> CrossPlaneRelationAttachment {
    CrossPlaneRelationAttachment::new(binding_ref, relation_ref)
}

/// cross-plane evidence に必要な採用 class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CrossPlaneEvidenceAdoption {
    /// single-plane evidence だけであり cross-plane evidence には採用しません。
    SinglePlaneOnly,
    /// source/target reference と関連 decision event を持つため採用可能です。
    CrossPlaneBindingEvidence,
    /// affected claim は close-not-claimed として扱います。
    CloseNotClaimed,
}

/// cross-plane で禁止する暗黙同一視です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedCrossPlaneEquivalence {
    /// Signaling join success is SFU endpoint admission.
    SignalingJoinAsSfuAdmission,
    /// token verification or authorization mapping is TURN/SFU binding.
    TokenOrAuthorizationAsPlaneBinding,
    /// same correlation ID is same participant/session/endpoint binding.
    SameCorrelationIdAsBinding,
    /// ICE candidate relay is TURN allocation or connectivity proof.
    IceRelayAsTurnAllocationOrConnectivityProof,
    /// TURN credential delivery is active allocation or permission.
    TurnCredentialDeliveryAsActiveTurnState,
    /// secure media protection is communication authorization.
    SecureMediaProtectionAsAuthorization,
    /// driver-local map is binding authority.
    DriverLocalMapAsBindingAuthority,
    /// lifecycle expiry in one plane silently mutates another plane.
    SilentCrossPlaneLifecycleMutation,
}
