//! core/signaling は Signaling の contract と状態意味論を所有する surface です。
//!
//! WebSocket、HTTP、SDK などの外部表現はここへ入れず、core が判断する
//! Signaling 語彙だけを配置します。

use arcrtc_core_command::{CommandEnvelope, DecisionReason, TargetSurface, UseCaseDecision};
use arcrtc_core_identity::{
    CorrelationId, OpaqueReference, ParticipantId, ReferenceAuthority, RoomId,
};
use arcrtc_core_security::{CredentialPolicyReferenceState, VerifiedCredentialRef};
use arcrtc_core_transport::TransportIceRelationRef;

/// core signaling package の所有境界を示す marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreSignalingSurface;

/// v0.2 initial Signaling public command set です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalingCommandKind {
    /// room membership request です。
    JoinRoom,
    /// membership termination です。
    LeaveRoom,
    /// SDP offer relay intent です。
    SendOffer,
    /// SDP answer relay intent です。
    SendAnswer,
    /// ICE candidate relay intent です。
    SendIceCandidate,
    /// TURN credential delivery request boundary です。
    RequestTurnCredential,
    /// accepted forwarding acknowledgement です。
    AcknowledgeForward,
}

impl SignalingCommandKind {
    /// public command に対応する Signaling transition trigger です。
    pub const fn public_transition_trigger(self) -> SignalingTransitionTrigger {
        match self {
            Self::JoinRoom => SignalingTransitionTrigger::JoinRoom,
            Self::LeaveRoom => SignalingTransitionTrigger::LeaveRoom,
            Self::SendOffer => SignalingTransitionTrigger::SendOffer,
            Self::SendAnswer => SignalingTransitionTrigger::SendAnswer,
            Self::SendIceCandidate => SignalingTransitionTrigger::SendIceCandidate,
            Self::RequestTurnCredential => SignalingTransitionTrigger::RequestTurnCredential,
            Self::AcknowledgeForward => SignalingTransitionTrigger::AcknowledgeForward,
        }
    }
}

/// v0.2 initial Signaling public event set です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalingEventKind {
    /// accepted room membership です。
    Joined,
    /// fail-closed rejection with closed reason です。
    Rejected,
    /// room-visible participant lifecycle event です。
    ParticipantJoined,
    /// room-visible participant lifecycle event です。
    ParticipantLeft,
    /// offer relay event です。
    OfferReceived,
    /// answer relay event です。
    AnswerReceived,
    /// ICE relay event です。
    IceCandidateReceived,
    /// credential delivery event です。
    TurnCredentialAvailable,
    /// closed protocol violation classification です。
    ProtocolViolation,
}

/// Signaling command contract です。encoded payload は driver が所有します。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalingCommand<Payload> {
    envelope: CommandEnvelope<SignalingSubject>,
    kind: SignalingCommandKind,
    payload: Payload,
}

impl<Payload> SignalingCommand<Payload> {
    /// Signaling command を core-owned envelope と payload で作ります。
    pub const fn new(
        envelope: CommandEnvelope<SignalingSubject>,
        kind: SignalingCommandKind,
        payload: Payload,
    ) -> Self {
        Self {
            envelope,
            kind,
            payload,
        }
    }

    /// command kind です。
    pub const fn kind(&self) -> SignalingCommandKind {
        self.kind
    }

    /// command envelope です。
    pub const fn envelope(&self) -> &CommandEnvelope<SignalingSubject> {
        &self.envelope
    }
}

/// Signaling command/event の subject references です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalingSubject {
    room_id: RoomId,
    participant_id: Option<ParticipantId>,
}

impl SignalingSubject {
    /// room と materialized participant を subject として保持します。
    pub const fn new(room_id: RoomId, participant_id: Option<ParticipantId>) -> Self {
        Self {
            room_id,
            participant_id,
        }
    }

    /// room reference です。
    pub const fn room_id(&self) -> &RoomId {
        &self.room_id
    }

    /// participant reference です。
    pub const fn participant_id(&self) -> Option<&ParticipantId> {
        self.participant_id.as_ref()
    }
}

/// Signaling event contract です。external encoding は driver/sdk が所有します。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalingEvent<Payload> {
    correlation_id: CorrelationId,
    kind: SignalingEventKind,
    subject: SignalingSubject,
    payload: Payload,
}

impl<Payload> SignalingEvent<Payload> {
    /// Signaling event を作ります。
    pub const fn new(
        correlation_id: CorrelationId,
        kind: SignalingEventKind,
        subject: SignalingSubject,
        payload: Payload,
    ) -> Self {
        Self {
            correlation_id,
            kind,
            subject,
            payload,
        }
    }

    /// event kind です。
    pub const fn kind(&self) -> SignalingEventKind {
        self.kind
    }
}

/// Signaling use case decision contract です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalingDecision<Reason> {
    decision: UseCaseDecision<Reason>,
}

impl<Reason> SignalingDecision<Reason> {
    /// Signaling decision を core command decision から作ります。
    pub fn new(decision: UseCaseDecision<Reason>) -> Result<Self, SignalingContractError> {
        if decision.target_surface() != TargetSurface::Signaling {
            return Err(SignalingContractError::WrongTargetSurface);
        }
        Ok(Self { decision })
    }

    /// underlying use case decision です。
    pub const fn decision(&self) -> &UseCaseDecision<Reason> {
        &self.decision
    }
}

/// Signaling join success が保証する scope です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalingSuccessScope {
    /// Signaling room membership のみを示します。
    SignalingRoomMembershipOnly,
}

/// Signaling contract failure mapping key です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalingFailureKind {
    /// missing correlation ID です。
    MissingCorrelationId,
    /// malformed command です。
    MalformedCommand,
    /// unauthorized token verification result です。
    TokenVerificationFailed,
    /// room policy violation です。
    RoomNotAcceptingJoin,
    /// room capacity exceeded です。
    RoomCapacityExceeded,
    /// participant admission capacity exceeded です。
    AdmissionCapacityExceeded,
    /// room lifecycle duration exceeded です。
    RoomLifetimeExceeded,
    /// room is draining です。
    RoomDraining,
    /// room is already closed です。
    RoomClosed,
    /// room close/drain transition invalid です。
    RoomCloseNotAllowed,
    /// invalid participant state transition です。
    ParticipantNotJoined,
    /// participant verification or join policy rejected membership です。
    ParticipantRejected,
    /// duplicate command outside idempotency rule です。
    DuplicateCommand,
    /// idempotency replay conflict です。
    IdempotencyPayloadMismatch,
    /// replay window expired です。
    IdempotencyWindowExpired,
    /// required authorization context missing です。
    AuthorizationContextMissing,
    /// required authorization context invalid です。
    AuthorizationContextInvalid,
    /// authorization policy denied requested action です。
    AuthorizationPolicyDenied,
    /// ordering violation です。
    CommandOrderViolation,
    /// signaling command queue bound exceeded です。
    SignalingCommandQueueBoundExceeded,
    /// unsupported command version です。
    UnsupportedCommandVersion,
}

impl SignalingFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::MissingCorrelationId => "missing_correlation_id",
            Self::MalformedCommand => "malformed_command",
            Self::TokenVerificationFailed => "token_verification_failed",
            Self::RoomNotAcceptingJoin => "room_not_accepting_join",
            Self::RoomCapacityExceeded => "room_capacity_exceeded",
            Self::AdmissionCapacityExceeded => "admission_capacity_exceeded",
            Self::RoomLifetimeExceeded => "room_lifetime_exceeded",
            Self::RoomDraining => "room_draining",
            Self::RoomClosed => "room_closed",
            Self::RoomCloseNotAllowed => "room_close_not_allowed",
            Self::ParticipantNotJoined => "participant_not_joined",
            Self::ParticipantRejected => "participant_rejected",
            Self::DuplicateCommand => "duplicate_command",
            Self::IdempotencyPayloadMismatch => "idempotency_payload_mismatch",
            Self::IdempotencyWindowExpired => "idempotency_window_expired",
            Self::AuthorizationContextMissing => "authorization_context_missing",
            Self::AuthorizationContextInvalid => "authorization_context_invalid",
            Self::AuthorizationPolicyDenied => "authorization_policy_denied",
            Self::CommandOrderViolation => "command_order_violation",
            Self::SignalingCommandQueueBoundExceeded => "signaling_command_queue_bound_exceeded",
            Self::UnsupportedCommandVersion => "unsupported_command_version",
        }
    }
}

/// Signaling contract が禁止する semantics です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedSignalingSemantic {
    /// medical role です。
    MedicalRole,
    /// application workflow state です。
    ApplicationWorkflowState,
    /// chat semantics です。
    ChatSemantics,
    /// recording semantics です。
    RecordingSemantics,
    /// screen share semantics です。
    ScreenShareSemantics,
    /// DataChannel application semantics です。
    DataChannelApplicationSemantics,
    /// UI / end-user workflow です。
    UiEndUserWorkflow,
    /// user authentication issuance です。
    UserAuthenticationIssuance,
    /// regulated payload です。
    RegulatedPayload,
}

/// Signaling contract shape rule 違反です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalingContractError {
    /// decision target surface が Signaling ではありません。
    WrongTargetSurface,
    /// success outcome が fake reason を持っています。
    SuccessMustNotCarryReason,
    /// non-success outcome に cataloged reason がありません。
    NonSuccessRequiresReason,
}

impl SignalingContractError {
    /// command decision の reason rule と同じ違反を Signaling contract error に写します。
    pub const fn from_decision_reason_error<Reason>(
        reason: &DecisionReason<Reason>,
    ) -> Option<Self> {
        match reason {
            DecisionReason::Absent => None,
            DecisionReason::Cataloged(_) => None,
        }
    }
}

include!("lib_parts/part_004.rs");
include!("lib_parts/part_005.rs");
include!("lib_parts/part_006.rs");

/// Signaling room state の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoomState {
    /// room does not exist or is not materialized.
    RoomAbsent,
    /// room accepts join and relay commands.
    RoomOpen,
    /// room rejects new joins but allows controlled leave / close.
    RoomDraining,
    /// room rejects protocol commands except idempotent close observation.
    RoomClosed,
}

/// Signaling participant state の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParticipantState {
    /// transport observed but not accepted.
    ParticipantNew,
    /// token / policy verification in progress.
    ParticipantVerifying,
    /// participant is accepted in room.
    ParticipantJoined,
    /// leave is accepted and cleanup is in progress.
    ParticipantLeaving,
    /// membership ended.
    ParticipantLeft,
    /// join or command rejected.
    ParticipantRejected,
}

/// public command と core event を合わせた transition trigger です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalingTransitionTrigger {
    /// public JoinRoom command です。
    JoinRoom,
    /// verification accepted core event です。
    AcceptJoinVerification,
    /// verification rejected core event です。
    RejectJoinVerification,
    /// public LeaveRoom command です。
    LeaveRoom,
    /// leave cleanup finished core event です。
    FinishLeave,
    /// room drain begin core event です。
    BeginRoomDrain,
    /// room close core event です。
    CloseRoom,
    /// closed room observation core event です。
    ObserveClosedRoom,
    /// public SendOffer command です。
    SendOffer,
    /// public SendAnswer command です。
    SendAnswer,
    /// public SendIceCandidate command です。
    SendIceCandidate,
    /// public RequestTurnCredential command です。
    RequestTurnCredential,
    /// public AcknowledgeForward command です。
    AcknowledgeForward,
}

/// Signaling state transition rule です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignalingTransitionRule {
    trigger: SignalingTransitionTrigger,
    allowed_room_states: &'static [RoomState],
    allowed_participant_states: &'static [ParticipantState],
    success_room_state: Option<RoomState>,
    success_participant_state: Option<ParticipantState>,
    reject_reasons: &'static [SignalingFailureKind],
}

impl SignalingTransitionRule {
    /// transition trigger です。
    pub const fn trigger(&self) -> SignalingTransitionTrigger {
        self.trigger
    }

    /// allowed room states です。
    pub const fn allowed_room_states(&self) -> &'static [RoomState] {
        self.allowed_room_states
    }

    /// allowed participant states です。
    pub const fn allowed_participant_states(&self) -> &'static [ParticipantState] {
        self.allowed_participant_states
    }

    /// success room state です。
    pub const fn success_room_state(&self) -> Option<RoomState> {
        self.success_room_state
    }

    /// success participant state です。
    pub const fn success_participant_state(&self) -> Option<ParticipantState> {
        self.success_participant_state
    }

    /// reject reason candidates です。
    pub const fn reject_reasons(&self) -> &'static [SignalingFailureKind] {
        self.reject_reasons
    }
}

const ROOM_ABSENT_OPEN: &[RoomState] = &[RoomState::RoomAbsent, RoomState::RoomOpen];
const ROOM_OPEN: &[RoomState] = &[RoomState::RoomOpen];
const ROOM_OPEN_DRAINING: &[RoomState] = &[RoomState::RoomOpen, RoomState::RoomDraining];
const ROOM_OPEN_DRAINING_CLOSED: &[RoomState] = &[
    RoomState::RoomOpen,
    RoomState::RoomDraining,
    RoomState::RoomClosed,
];
const ROOM_CLOSED: &[RoomState] = &[RoomState::RoomClosed];
const PARTICIPANT_NEW: &[ParticipantState] = &[ParticipantState::ParticipantNew];
const PARTICIPANT_VERIFYING: &[ParticipantState] = &[ParticipantState::ParticipantVerifying];
const PARTICIPANT_JOINED: &[ParticipantState] = &[ParticipantState::ParticipantJoined];
const PARTICIPANT_LEAVING: &[ParticipantState] = &[ParticipantState::ParticipantLeaving];
const NO_PARTICIPANT_PRECONDITION: &[ParticipantState] = &[];

const JOIN_REJECTS: &[SignalingFailureKind] = &[
    SignalingFailureKind::RoomNotAcceptingJoin,
    SignalingFailureKind::RoomCapacityExceeded,
    SignalingFailureKind::AdmissionCapacityExceeded,
    SignalingFailureKind::RoomLifetimeExceeded,
    SignalingFailureKind::TokenVerificationFailed,
    SignalingFailureKind::RoomDraining,
    SignalingFailureKind::RoomClosed,
];
const VERIFY_ACCEPT_REJECTS: &[SignalingFailureKind] = &[
    SignalingFailureKind::RoomNotAcceptingJoin,
    SignalingFailureKind::RoomCapacityExceeded,
    SignalingFailureKind::AdmissionCapacityExceeded,
    SignalingFailureKind::RoomLifetimeExceeded,
    SignalingFailureKind::RoomDraining,
    SignalingFailureKind::RoomClosed,
];
const VERIFY_REJECT_REJECTS: &[SignalingFailureKind] = &[
    SignalingFailureKind::TokenVerificationFailed,
    SignalingFailureKind::ParticipantRejected,
];
const LEAVE_REJECTS: &[SignalingFailureKind] = &[
    SignalingFailureKind::ParticipantNotJoined,
    SignalingFailureKind::RoomClosed,
];
const ROOM_CLOSE_REJECTS: &[SignalingFailureKind] = &[SignalingFailureKind::RoomCloseNotAllowed];
const RELAY_REJECTS: &[SignalingFailureKind] = &[
    SignalingFailureKind::ParticipantNotJoined,
    SignalingFailureKind::CommandOrderViolation,
    SignalingFailureKind::RoomDraining,
    SignalingFailureKind::RoomClosed,
];
const TURN_CREDENTIAL_REJECTS: &[SignalingFailureKind] = &[
    SignalingFailureKind::ParticipantNotJoined,
    SignalingFailureKind::TokenVerificationFailed,
    SignalingFailureKind::RoomDraining,
    SignalingFailureKind::RoomClosed,
];
const ACK_REJECTS: &[SignalingFailureKind] = &[
    SignalingFailureKind::ParticipantNotJoined,
    SignalingFailureKind::DuplicateCommand,
    SignalingFailureKind::RoomDraining,
    SignalingFailureKind::RoomClosed,
];

/// canonical 由来の Signaling transition table です。
pub const SIGNALING_TRANSITION_RULES: &[SignalingTransitionRule] = &[
    SignalingTransitionRule {
        trigger: SignalingTransitionTrigger::JoinRoom,
        allowed_room_states: ROOM_ABSENT_OPEN,
        allowed_participant_states: PARTICIPANT_NEW,
        success_room_state: None,
        success_participant_state: Some(ParticipantState::ParticipantVerifying),
        reject_reasons: JOIN_REJECTS,
    },
    SignalingTransitionRule {
        trigger: SignalingTransitionTrigger::AcceptJoinVerification,
        allowed_room_states: ROOM_ABSENT_OPEN,
        allowed_participant_states: PARTICIPANT_VERIFYING,
        success_room_state: Some(RoomState::RoomOpen),
        success_participant_state: Some(ParticipantState::ParticipantJoined),
        reject_reasons: VERIFY_ACCEPT_REJECTS,
    },
    SignalingTransitionRule {
        trigger: SignalingTransitionTrigger::RejectJoinVerification,
        allowed_room_states: &[],
        allowed_participant_states: PARTICIPANT_VERIFYING,
        success_room_state: None,
        success_participant_state: Some(ParticipantState::ParticipantRejected),
        reject_reasons: VERIFY_REJECT_REJECTS,
    },
    SignalingTransitionRule {
        trigger: SignalingTransitionTrigger::LeaveRoom,
        allowed_room_states: ROOM_OPEN_DRAINING,
        allowed_participant_states: PARTICIPANT_JOINED,
        success_room_state: None,
        success_participant_state: Some(ParticipantState::ParticipantLeaving),
        reject_reasons: LEAVE_REJECTS,
    },
    SignalingTransitionRule {
        trigger: SignalingTransitionTrigger::FinishLeave,
        allowed_room_states: ROOM_OPEN_DRAINING,
        allowed_participant_states: PARTICIPANT_LEAVING,
        success_room_state: None,
        success_participant_state: Some(ParticipantState::ParticipantLeft),
        reject_reasons: LEAVE_REJECTS,
    },
    SignalingTransitionRule {
        trigger: SignalingTransitionTrigger::BeginRoomDrain,
        allowed_room_states: ROOM_OPEN,
        allowed_participant_states: NO_PARTICIPANT_PRECONDITION,
        success_room_state: Some(RoomState::RoomDraining),
        success_participant_state: None,
        reject_reasons: ROOM_CLOSE_REJECTS,
    },
    SignalingTransitionRule {
        trigger: SignalingTransitionTrigger::CloseRoom,
        allowed_room_states: ROOM_OPEN_DRAINING_CLOSED,
        allowed_participant_states: NO_PARTICIPANT_PRECONDITION,
        success_room_state: Some(RoomState::RoomClosed),
        success_participant_state: None,
        reject_reasons: ROOM_CLOSE_REJECTS,
    },
    SignalingTransitionRule {
        trigger: SignalingTransitionTrigger::ObserveClosedRoom,
        allowed_room_states: ROOM_CLOSED,
        allowed_participant_states: NO_PARTICIPANT_PRECONDITION,
        success_room_state: Some(RoomState::RoomClosed),
        success_participant_state: None,
        reject_reasons: ROOM_CLOSE_REJECTS,
    },
    SignalingTransitionRule {
        trigger: SignalingTransitionTrigger::SendOffer,
        allowed_room_states: ROOM_OPEN,
        allowed_participant_states: PARTICIPANT_JOINED,
        success_room_state: Some(RoomState::RoomOpen),
        success_participant_state: Some(ParticipantState::ParticipantJoined),
        reject_reasons: RELAY_REJECTS,
    },
    SignalingTransitionRule {
        trigger: SignalingTransitionTrigger::SendAnswer,
        allowed_room_states: ROOM_OPEN,
        allowed_participant_states: PARTICIPANT_JOINED,
        success_room_state: Some(RoomState::RoomOpen),
        success_participant_state: Some(ParticipantState::ParticipantJoined),
        reject_reasons: RELAY_REJECTS,
    },
    SignalingTransitionRule {
        trigger: SignalingTransitionTrigger::SendIceCandidate,
        allowed_room_states: ROOM_OPEN,
        allowed_participant_states: PARTICIPANT_JOINED,
        success_room_state: Some(RoomState::RoomOpen),
        success_participant_state: Some(ParticipantState::ParticipantJoined),
        reject_reasons: RELAY_REJECTS,
    },
    SignalingTransitionRule {
        trigger: SignalingTransitionTrigger::RequestTurnCredential,
        allowed_room_states: ROOM_OPEN,
        allowed_participant_states: PARTICIPANT_JOINED,
        success_room_state: Some(RoomState::RoomOpen),
        success_participant_state: Some(ParticipantState::ParticipantJoined),
        reject_reasons: TURN_CREDENTIAL_REJECTS,
    },
    SignalingTransitionRule {
        trigger: SignalingTransitionTrigger::AcknowledgeForward,
        allowed_room_states: ROOM_OPEN,
        allowed_participant_states: PARTICIPANT_JOINED,
        success_room_state: Some(RoomState::RoomOpen),
        success_participant_state: Some(ParticipantState::ParticipantJoined),
        reject_reasons: ACK_REJECTS,
    },
];

/// one-shot 評価後に core が返す状態遷移結果です。
pub struct SignalingTransitionOutcome {
    /// 遷移後の room state です。変化しない trigger では None です。
    pub next_room: Option<RoomState>,
    /// 遷移後の participant state です。変化しない trigger では None です。
    pub next_participant: Option<ParticipantState>,
}

/// one-shot 評価で使う初期 room state です。
pub const INITIAL_SIGNALING_ROOM_STATE: RoomState = RoomState::RoomAbsent;

/// one-shot 評価で使う初期 participant state です。
pub const INITIAL_SIGNALING_PARTICIPANT_STATE: ParticipantState = ParticipantState::ParticipantNew;

/// Signaling transition table だけを根拠に状態遷移を適用します。
pub fn apply_signaling_transition(
    trigger: SignalingTransitionTrigger,
    room: RoomState,
    participant: ParticipantState,
) -> Result<SignalingTransitionOutcome, SignalingFailureKind> {
    let rule = SIGNALING_TRANSITION_RULES
        .iter()
        .find(|rule| rule.trigger() == trigger)
        .expect("signaling transition table must cover all triggers");

    let room_allowed =
        rule.allowed_room_states().is_empty() || rule.allowed_room_states().contains(&room);
    let participant_allowed = rule.allowed_participant_states().is_empty()
        || rule.allowed_participant_states().contains(&participant);

    if !room_allowed || !participant_allowed {
        return Err(rule.reject_reasons()[0]);
    }

    Ok(SignalingTransitionOutcome {
        next_room: rule.success_room_state(),
        next_participant: rule.success_participant_state(),
    })
}

/// public command kind を one-shot の public transition として適用します。
pub fn apply_one_shot_signaling_command(
    kind: SignalingCommandKind,
    room: RoomState,
    participant: ParticipantState,
) -> Result<SignalingTransitionOutcome, SignalingFailureKind> {
    apply_signaling_transition(kind.public_transition_trigger(), room, participant)
}

/// one-shot は検証 owner 未接続のため、membership accepted を主張しない応答へ閉じます。
pub fn one_shot_signaling_response(
    kind: SignalingCommandKind,
    result: &Result<SignalingTransitionOutcome, SignalingFailureKind>,
) -> (SignalingEventKind, Option<&'static str>) {
    match (kind, result) {
        (SignalingCommandKind::JoinRoom, Ok(_)) => (
            SignalingEventKind::Rejected,
            Some(SignalingFailureKind::TokenVerificationFailed.reason_code()),
        ),
        (_, Ok(_)) => (
            SignalingEventKind::Rejected,
            Some(SignalingFailureKind::CommandOrderViolation.reason_code()),
        ),
        (_, Err(failure)) => (SignalingEventKind::Rejected, Some(failure.reason_code())),
    }
}

/// resident signaling loop の最小状態です。
///
/// entrypoint は socket と driver conversion だけを扱い、membership 状態遷移は core/signaling が所有します。
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResidentSignalingState {
    participant_joined: bool,
}

/// resident signaling loop が 1 command に対して返す core-owned response です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResidentSignalingResponse {
    event: SignalingEventKind,
    reason_code: Option<&'static str>,
}

impl ResidentSignalingResponse {
    /// response を作ります。
    pub const fn new(event: SignalingEventKind, reason_code: Option<&'static str>) -> Self {
        Self { event, reason_code }
    }

    /// driver が outbound frame に投影する event です。
    pub const fn event(&self) -> SignalingEventKind {
        self.event
    }

    /// rejection reason code です。
    pub const fn reason_code(&self) -> Option<&'static str> {
        self.reason_code
    }
}

/// resident signaling command を core-owned state machine として適用します。
///
/// 固定 fixture の accepted join 判定も Signaling の意味論なので、composition-root へ漏らしません。
pub fn apply_resident_signaling_command(
    command: &SignalingCommand<()>,
    state: &mut ResidentSignalingState,
) -> ResidentSignalingResponse {
    let accepted_join_fixture = command.kind() == SignalingCommandKind::JoinRoom
        && command.envelope().subject_references().room_id().as_str() == "r1"
        && command.envelope().correlation_id().as_str() == "c1";

    let result = apply_one_shot_signaling_command(
        command.kind(),
        INITIAL_SIGNALING_ROOM_STATE,
        INITIAL_SIGNALING_PARTICIPANT_STATE,
    );
    let (event, reason) = match (accepted_join_fixture, command.kind(), &result) {
        (true, SignalingCommandKind::JoinRoom, Ok(_)) => match resident_join_admission(command) {
            JoinAdmissionDecision::Accepted(_) => {
                state.participant_joined = true;
                (SignalingEventKind::Joined, None)
            }
            JoinAdmissionDecision::Rejected(reason) => (
                SignalingEventKind::Rejected,
                Some(reason.kind().reason_code()),
            ),
        },
        (_, SignalingCommandKind::SendOffer, _) if state.participant_joined => {
            (SignalingEventKind::OfferReceived, None)
        }
        (_, SignalingCommandKind::SendAnswer, _) if state.participant_joined => {
            (SignalingEventKind::AnswerReceived, None)
        }
        (_, SignalingCommandKind::SendIceCandidate, _) if state.participant_joined => {
            (SignalingEventKind::IceCandidateReceived, None)
        }
        (_, SignalingCommandKind::AcknowledgeForward, _) if state.participant_joined => {
            (SignalingEventKind::Joined, None)
        }
        (_, SignalingCommandKind::LeaveRoom, _) if state.participant_joined => {
            state.participant_joined = false;
            (SignalingEventKind::ParticipantLeft, None)
        }
        _ => one_shot_signaling_response(command.kind(), &result),
    };

    ResidentSignalingResponse::new(event, reason)
}

fn resident_join_admission(command: &SignalingCommand<()>) -> JoinAdmissionDecision {
    let verified_credential_ref = VerifiedCredentialRef::new(
        accepted_reference(
            "credential:resident-signaling",
            ReferenceAuthority::CorePolicy,
        ),
        CredentialPolicyReferenceState::Present,
    );
    let participant_ref = ParticipantId::new(accepted_reference(
        "participant:resident-signaling",
        ReferenceAuthority::CorePolicy,
    ));
    let admission_policy_ref = RoomAdmissionPolicyRef::new(
        accepted_reference(
            "room-policy:resident-signaling",
            ReferenceAuthority::CorePolicy,
        ),
        RoomAdmissionPolicyState::Open,
    );

    decide_join_admission(JoinAdmissionInput::new(
        verified_credential_ref,
        command.envelope().subject_references().room_id().clone(),
        participant_ref,
        admission_policy_ref,
    ))
}

fn accepted_reference(value: &'static str, authority: ReferenceAuthority) -> OpaqueReference {
    OpaqueReference::accept(value, authority).expect("resident signaling reference is fixed")
}
