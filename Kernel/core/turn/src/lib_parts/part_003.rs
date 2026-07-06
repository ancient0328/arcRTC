/// TURN refresh decision の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnRefreshDecision {
    /// refresh accepted.
    Accepted,
    /// refresh rejected.
    Rejected(TurnFailureKind),
    /// refresh expired.
    Expired(TurnFailureKind),
}

/// TURN permission decision の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnPermissionDecision {
    /// permission accepted.
    Accepted,
    /// permission rejected.
    Rejected(TurnFailureKind),
    /// permission revoked.
    Revoked(TurnFailureKind),
    /// permission expired.
    Expired(TurnFailureKind),
}

/// TURN channel bind decision の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnChannelBindDecision {
    /// channel bind accepted.
    Accepted,
    /// channel bind rejected.
    Rejected(TurnFailureKind),
    /// channel bind expired.
    Expired(TurnFailureKind),
}

/// TURN relay decision の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnRelayDecision {
    /// relay forwarding is selected.
    Forwarded,
    /// relay forwarding is denied.
    Denied(TurnFailureKind),
    /// relay forwarding failed.
    Failed(TurnFailureKind),
}

/// relay forwarding state です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RelayForwardingState {
    allocation_state: AllocationState,
    permission_state: PermissionState,
    channel_bind_state: Option<ChannelBindState>,
}

impl RelayForwardingState {
    /// allocation / permission / channel state を保持します。
    pub const fn new(
        allocation_state: AllocationState,
        permission_state: PermissionState,
        channel_bind_state: Option<ChannelBindState>,
    ) -> Self {
        Self {
            allocation_state,
            permission_state,
            channel_bind_state,
        }
    }
}

/// TURN relay-family command の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnRelayCommand {
    /// refresh command.
    Refresh,
    /// permission command.
    Permission,
    /// channel bind command.
    ChannelBind,
    /// relay command.
    Relay,
}

/// TURN relay-family command decision です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnRelayCommandDecision {
    /// refresh decision.
    Refresh(TurnRefreshDecision),
    /// permission decision.
    Permission(TurnPermissionDecision),
    /// channel bind decision.
    ChannelBind(TurnChannelBindDecision),
    /// relay decision.
    Relay(TurnRelayDecision),
}

/// TURN relay-family command を permission/channel/relay failure の閉集合へ写像します。
pub const fn apply_turn_relay_command(
    state: RelayForwardingState,
    command: TurnRelayCommand,
) -> TurnRelayCommandDecision {
    match command {
        TurnRelayCommand::Refresh => match state.allocation_state {
            AllocationState::Active | AllocationState::Refreshing => {
                TurnRelayCommandDecision::Refresh(TurnRefreshDecision::Accepted)
            }
            AllocationState::Expired => TurnRelayCommandDecision::Refresh(
                TurnRefreshDecision::Expired(TurnFailureKind::AllocationNotFound),
            ),
            AllocationState::Absent
            | AllocationState::Requested
            | AllocationState::Released
            | AllocationState::Rejected => TurnRelayCommandDecision::Refresh(
                TurnRefreshDecision::Rejected(TurnFailureKind::AllocationNotFound),
            ),
        },
        TurnRelayCommand::Permission => match state.permission_state {
            PermissionState::Active => {
                TurnRelayCommandDecision::Permission(TurnPermissionDecision::Accepted)
            }
            PermissionState::Expired => TurnRelayCommandDecision::Permission(
                TurnPermissionDecision::Expired(TurnFailureKind::PermissionNotFound),
            ),
            PermissionState::Revoked => TurnRelayCommandDecision::Permission(
                TurnPermissionDecision::Revoked(TurnFailureKind::PeerNotAllowed),
            ),
            PermissionState::Absent | PermissionState::Requested | PermissionState::Rejected => {
                TurnRelayCommandDecision::Permission(TurnPermissionDecision::Rejected(
                    TurnFailureKind::PermissionNotFound,
                ))
            }
        },
        TurnRelayCommand::ChannelBind => match state.channel_bind_state {
            Some(ChannelBindState::Bound) => {
                TurnRelayCommandDecision::ChannelBind(TurnChannelBindDecision::Accepted)
            }
            Some(ChannelBindState::Expired) => TurnRelayCommandDecision::ChannelBind(
                TurnChannelBindDecision::Expired(TurnFailureKind::ChannelBindLifetimeExceeded),
            ),
            Some(ChannelBindState::Unbound)
            | Some(ChannelBindState::Requested)
            | Some(ChannelBindState::Rejected)
            | None => TurnRelayCommandDecision::ChannelBind(TurnChannelBindDecision::Rejected(
                TurnFailureKind::PeerNotAllowed,
            )),
        },
        TurnRelayCommand::Relay => {
            if !matches!(state.allocation_state, AllocationState::Active) {
                return TurnRelayCommandDecision::Relay(TurnRelayDecision::Denied(
                    TurnFailureKind::AllocationNotFound,
                ));
            }
            if !matches!(state.permission_state, PermissionState::Active) {
                return TurnRelayCommandDecision::Relay(TurnRelayDecision::Denied(
                    TurnFailureKind::PermissionNotFound,
                ));
            }
            if matches!(
                state.channel_bind_state,
                Some(ChannelBindState::Expired | ChannelBindState::Rejected)
            ) {
                return TurnRelayCommandDecision::Relay(TurnRelayDecision::Denied(
                    TurnFailureKind::PeerNotAllowed,
                ));
            }

            TurnRelayCommandDecision::Relay(TurnRelayDecision::Forwarded)
        }
    }
}
