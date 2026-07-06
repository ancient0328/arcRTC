/// SFU endpoint admission の閉じた判定です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SfuEndpointAdmissionDecision {
    /// endpoint は SFU に admission 済みです。
    Admitted,
    /// endpoint admission は拒否されました。
    Rejected(SfuFailureKind),
    /// endpoint admission は quality/backpressure により抑止されました。
    Suppressed(SfuFailureKind),
}

/// publication の閉じた判定です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PublicationDecision {
    /// publication は routing 対象です。
    Accepted,
    /// publication は拒否されました。
    Rejected(SfuFailureKind),
    /// publication は抑止されました。
    Suppressed(SfuFailureKind),
}

/// subscription の閉じた判定です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubscriptionDecision {
    /// subscription は routing 対象です。
    Accepted,
    /// subscription は拒否されました。
    Rejected(SfuFailureKind),
    /// subscription は抑止されました。
    Suppressed(SfuFailureKind),
}

/// forwarding route selection outcome の閉集合です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForwardingDecision {
    /// core/sfu が route を選択しました。
    Selected(RouteId),
    /// forwarding は契約上拒否されました。
    Rejected(SfuFailureKind),
    /// forwarding は一時的に抑止されました。
    Suppressed(SfuFailureKind),
    /// forwarding 対象 packet/route は drop されました。
    Dropped(SfuFailureKind),
    /// forwarding 判定が fail-closed で失敗しました。
    Failed(SfuFailureKind),
}

/// SFU forwarding 判定に必要な core-owned state です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SfuForwardingState {
    session_state: SfuSessionState,
    source_endpoint_state: SfuEndpointState,
    target_endpoint_state: SfuEndpointState,
    route_state: SfuRouteState,
}

impl SfuForwardingState {
    /// SFU 内の session / endpoint / route state だけを束ねます。
    pub const fn new(
        session_state: SfuSessionState,
        source_endpoint_state: SfuEndpointState,
        target_endpoint_state: SfuEndpointState,
        route_state: SfuRouteState,
    ) -> Self {
        Self {
            session_state,
            source_endpoint_state,
            target_endpoint_state,
            route_state,
        }
    }

    /// session state です。
    pub const fn session_state(&self) -> SfuSessionState {
        self.session_state
    }
}

/// SFU forwarding 判定要求です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SfuForwardingRequest {
    route_id: RouteId,
    publication_decision: PublicationDecision,
    subscription_decision: SubscriptionDecision,
}

impl SfuForwardingRequest {
    /// route と publication/subscription 判定を束ねます。
    pub fn new(
        route_id: RouteId,
        publication_decision: PublicationDecision,
        subscription_decision: SubscriptionDecision,
    ) -> Self {
        Self {
            route_id,
            publication_decision,
            subscription_decision,
        }
    }

    /// route id です。
    pub const fn route_id(&self) -> &RouteId {
        &self.route_id
    }
}

/// SFU endpoint state から endpoint admission decision を得ます。
pub const fn classify_sfu_endpoint_admission(
    endpoint_state: SfuEndpointState,
) -> SfuEndpointAdmissionDecision {
    match endpoint_state {
        SfuEndpointState::Admitted | SfuEndpointState::Degraded => {
            SfuEndpointAdmissionDecision::Admitted
        }
        SfuEndpointState::Draining => {
            SfuEndpointAdmissionDecision::Suppressed(SfuFailureKind::EndpointClosedByBackpressure)
        }
        SfuEndpointState::Observed | SfuEndpointState::AdmissionPending => {
            SfuEndpointAdmissionDecision::Rejected(SfuFailureKind::ParticipantNotAdmitted)
        }
        SfuEndpointState::Removed => {
            SfuEndpointAdmissionDecision::Rejected(SfuFailureKind::TargetUnavailable)
        }
        SfuEndpointState::Rejected => {
            SfuEndpointAdmissionDecision::Rejected(SfuFailureKind::EndpointQualityNotAllowed)
        }
    }
}

/// forwarding の route selection authority を core/sfu 内に閉じます。
///
/// Signaling join success は SFU endpoint admission ではないため、この判定の入力に含めません。
pub fn apply_sfu_forwarding(
    state: SfuForwardingState,
    request: SfuForwardingRequest,
) -> ForwardingDecision {
    if !matches!(state.session_state, SfuSessionState::Open) {
        return ForwardingDecision::Rejected(SfuFailureKind::SfuSessionNotAccepting);
    }

    for endpoint_state in [state.source_endpoint_state, state.target_endpoint_state] {
        match classify_sfu_endpoint_admission(endpoint_state) {
            SfuEndpointAdmissionDecision::Admitted => {}
            SfuEndpointAdmissionDecision::Rejected(reason) => {
                return ForwardingDecision::Rejected(reason);
            }
            SfuEndpointAdmissionDecision::Suppressed(reason) => {
                return ForwardingDecision::Suppressed(reason);
            }
        }
    }

    match request.publication_decision {
        PublicationDecision::Accepted => {}
        PublicationDecision::Rejected(reason) => return ForwardingDecision::Rejected(reason),
        PublicationDecision::Suppressed(reason) => return ForwardingDecision::Suppressed(reason),
    }

    match request.subscription_decision {
        SubscriptionDecision::Accepted => {}
        SubscriptionDecision::Rejected(reason) => return ForwardingDecision::Rejected(reason),
        SubscriptionDecision::Suppressed(reason) => return ForwardingDecision::Suppressed(reason),
    }

    match state.route_state {
        SfuRouteState::Candidate | SfuRouteState::Selected | SfuRouteState::Degraded => {
            ForwardingDecision::Selected(request.route_id)
        }
        SfuRouteState::Delayed => {
            ForwardingDecision::Suppressed(SfuFailureKind::ActionDelayedByBackpressure)
        }
        SfuRouteState::SuppressedByBackpressure => {
            ForwardingDecision::Suppressed(SfuFailureKind::RouteSuppressedByBackpressure)
        }
        SfuRouteState::SuppressedByQuality => {
            ForwardingDecision::Suppressed(SfuFailureKind::RouteSuppressedByQuality)
        }
        SfuRouteState::Dropped => {
            ForwardingDecision::Dropped(SfuFailureKind::PacketDroppedByBackpressure)
        }
        SfuRouteState::Closed => ForwardingDecision::Failed(SfuFailureKind::TargetUnavailable),
    }
}
