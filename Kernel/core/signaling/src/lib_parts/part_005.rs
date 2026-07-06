/// participant lifecycle state の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParticipantLifecycleState {
    /// join admission is in progress.
    Joining,
    /// participant is active in the room.
    Active,
    /// leave processing is in progress.
    Leaving,
    /// participant has left.
    Left,
    /// participant membership timed out.
    TimedOut,
}

/// room membership state です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoomMembershipState {
    room_ref: RoomId,
    participant_ref: ParticipantId,
    lifecycle_state: ParticipantLifecycleState,
    last_idempotency_decision: Option<IdempotentCommandDecision>,
}

impl RoomMembershipState {
    /// room / participant / lifecycle state を保持します。
    pub const fn new(
        room_ref: RoomId,
        participant_ref: ParticipantId,
        lifecycle_state: ParticipantLifecycleState,
        last_idempotency_decision: Option<IdempotentCommandDecision>,
    ) -> Self {
        Self {
            room_ref,
            participant_ref,
            lifecycle_state,
            last_idempotency_decision,
        }
    }

    /// lifecycle state です。
    pub const fn lifecycle_state(&self) -> ParticipantLifecycleState {
        self.lifecycle_state
    }
}

/// idempotent command decision の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdempotentCommandDecision {
    /// first command is accepted for evaluation.
    FirstCommand,
    /// duplicate command is observed and not re-applied.
    DuplicateObserved,
    /// same scope command conflicts with prior payload.
    ConflictRejected,
}

/// leave decision の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LeaveDecision {
    /// leave is accepted.
    Accepted,
    /// participant is already leaving.
    AlreadyLeaving,
    /// participant already left or timed out.
    AlreadyEnded,
    /// participant is not active enough to leave.
    Rejected,
}

/// timeout decision の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeoutDecision {
    /// membership timed out.
    TimedOut,
    /// timeout is ignored because membership already ended.
    AlreadyEnded,
    /// timeout is rejected for current lifecycle state.
    Rejected,
}

/// membership command の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MembershipCommand {
    /// first command observation.
    FirstCommand,
    /// duplicate command observation.
    DuplicateCommand,
    /// conflicting duplicate command observation.
    IdempotencyConflict,
    /// leave command.
    Leave,
    /// timeout observation.
    Timeout,
}

/// membership command outcome の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MembershipCommandDecision {
    /// idempotency decision.
    Idempotent(IdempotentCommandDecision),
    /// leave decision.
    Leave(LeaveDecision),
    /// timeout decision.
    Timeout(TimeoutDecision),
}

/// membership command application result です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipCommandResult {
    state: RoomMembershipState,
    decision: MembershipCommandDecision,
    rejected_reason: Option<SignalingFailureKind>,
}

impl MembershipCommandResult {
    /// resulting state と closed decision/reason を保持します。
    pub const fn new(
        state: RoomMembershipState,
        decision: MembershipCommandDecision,
        rejected_reason: Option<SignalingFailureKind>,
    ) -> Self {
        Self {
            state,
            decision,
            rejected_reason,
        }
    }

    /// resulting membership state です。
    pub const fn state(&self) -> &RoomMembershipState {
        &self.state
    }

    /// command decision です。
    pub const fn decision(&self) -> MembershipCommandDecision {
        self.decision
    }
}

/// membership command を duplicate / leave / timeout の closed outcome に写像します。
pub fn apply_membership_command(
    state: RoomMembershipState,
    command: MembershipCommand,
) -> MembershipCommandResult {
    match command {
        MembershipCommand::FirstCommand => {
            let next = RoomMembershipState::new(
                state.room_ref,
                state.participant_ref,
                state.lifecycle_state,
                Some(IdempotentCommandDecision::FirstCommand),
            );
            MembershipCommandResult::new(
                next,
                MembershipCommandDecision::Idempotent(IdempotentCommandDecision::FirstCommand),
                None,
            )
        }
        MembershipCommand::DuplicateCommand => {
            let next = RoomMembershipState::new(
                state.room_ref,
                state.participant_ref,
                state.lifecycle_state,
                Some(IdempotentCommandDecision::DuplicateObserved),
            );
            MembershipCommandResult::new(
                next,
                MembershipCommandDecision::Idempotent(
                    IdempotentCommandDecision::DuplicateObserved,
                ),
                None,
            )
        }
        MembershipCommand::IdempotencyConflict => {
            let next = RoomMembershipState::new(
                state.room_ref,
                state.participant_ref,
                state.lifecycle_state,
                Some(IdempotentCommandDecision::ConflictRejected),
            );
            MembershipCommandResult::new(
                next,
                MembershipCommandDecision::Idempotent(
                    IdempotentCommandDecision::ConflictRejected,
                ),
                Some(SignalingFailureKind::IdempotencyPayloadMismatch),
            )
        }
        MembershipCommand::Leave => apply_leave_command(state),
        MembershipCommand::Timeout => apply_timeout_command(state),
    }
}

fn apply_leave_command(state: RoomMembershipState) -> MembershipCommandResult {
    match state.lifecycle_state {
        ParticipantLifecycleState::Joining | ParticipantLifecycleState::Active => {
            let next = RoomMembershipState::new(
                state.room_ref,
                state.participant_ref,
                ParticipantLifecycleState::Leaving,
                state.last_idempotency_decision,
            );
            MembershipCommandResult::new(
                next,
                MembershipCommandDecision::Leave(LeaveDecision::Accepted),
                None,
            )
        }
        ParticipantLifecycleState::Leaving => MembershipCommandResult::new(
            state,
            MembershipCommandDecision::Leave(LeaveDecision::AlreadyLeaving),
            None,
        ),
        ParticipantLifecycleState::Left | ParticipantLifecycleState::TimedOut => {
            MembershipCommandResult::new(
                state,
                MembershipCommandDecision::Leave(LeaveDecision::AlreadyEnded),
                Some(SignalingFailureKind::ParticipantNotJoined),
            )
        }
    }
}

fn apply_timeout_command(state: RoomMembershipState) -> MembershipCommandResult {
    match state.lifecycle_state {
        ParticipantLifecycleState::Joining
        | ParticipantLifecycleState::Active
        | ParticipantLifecycleState::Leaving => {
            let next = RoomMembershipState::new(
                state.room_ref,
                state.participant_ref,
                ParticipantLifecycleState::TimedOut,
                state.last_idempotency_decision,
            );
            MembershipCommandResult::new(
                next,
                MembershipCommandDecision::Timeout(TimeoutDecision::TimedOut),
                None,
            )
        }
        ParticipantLifecycleState::Left | ParticipantLifecycleState::TimedOut => {
            MembershipCommandResult::new(
                state,
                MembershipCommandDecision::Timeout(TimeoutDecision::AlreadyEnded),
                Some(SignalingFailureKind::ParticipantNotJoined),
            )
        }
    }
}
