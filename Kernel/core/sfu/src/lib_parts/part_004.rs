use arcrtc_core_identity::{OpaqueReference, ReferenceAuthority};
use arcrtc_core_transport::{
    decide_secure_media_session, DtlsHandshakeDecision, IceCandidateRef,
    IceConnectivityDecision, IceFailureKind, SecureMediaFailureKind,
    SecureMediaSessionDecision, SecureMediaSessionFailureReason, SecureMediaSessionInput,
    SrtpProtectionDecision, TransportCandidateRelationDecision, TransportCandidateRelationInput,
    TransportIceRelationRef, TransportPolicyRef, TransportPolicyState,
};

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

/// resident SFU loop が core に渡す入力分類です。
///
/// driver は wire token を decode し、secure media / forwarding の状態更新は core/sfu が所有します。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResidentSfuInputClass {
    /// ICE connectivity observation が届いたことを示します。
    IceConnectivityObservation,
    /// DTLS handshake observation が届いたことを示します。
    DtlsHandshakeObservation,
    /// SRTP protection observation が届いたことを示します。
    SrtpProtectionObservation,
    /// media forwarding packet observation が届いたことを示します。
    ProtectedMediaPacket,
}

/// resident SFU loop の core-owned state です。
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ResidentSfuState {
    ice_relation: Option<TransportIceRelationRef>,
    dtls_established: bool,
    srtp_active: bool,
    forwarded_packets: u64,
}

/// resident SFU が外部へ投影できる success 種別です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResidentSfuSuccessKind {
    /// ICE connectivity accepted.
    IceConnectivityAccepted,
    /// DTLS handshake accepted.
    DtlsHandshakeAccepted,
    /// SRTP protection accepted.
    SrtpProtectionAccepted,
    /// protected media forwarding accepted.
    ProtectedMediaForwarded,
}

/// resident SFU command の core-owned outcome です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResidentSfuOutcome {
    /// core/sfu が success outcome を返しました。
    Accepted {
        /// success 種別です。
        success_kind: ResidentSfuSuccessKind,
        /// observation detail code です。
        detail_code: &'static str,
        /// transmitted packet count です。
        transmits: usize,
        /// dropped packet count です。
        dropped: usize,
        /// stable transmit digest です。
        digest: u64,
    },
    /// core/sfu または core/transport が closed reason で拒否しました。
    Rejected {
        /// closed reason code です。
        reason_code: &'static str,
    },
}

impl ResidentSfuOutcome {
    /// success kind を返します。
    pub const fn success_kind(&self) -> Option<ResidentSfuSuccessKind> {
        match self {
            Self::Accepted { success_kind, .. } => Some(*success_kind),
            Self::Rejected { .. } => None,
        }
    }
}

/// resident SFU input を secure media / forwarding state machine として適用します。
pub fn apply_resident_sfu_input(
    input_class: ResidentSfuInputClass,
    state: &mut ResidentSfuState,
) -> ResidentSfuOutcome {
    match input_class {
        ResidentSfuInputClass::IceConnectivityObservation => resident_sfu_ice_response(state),
        ResidentSfuInputClass::DtlsHandshakeObservation => resident_sfu_dtls_response(state),
        ResidentSfuInputClass::SrtpProtectionObservation => resident_sfu_srtp_response(state),
        ResidentSfuInputClass::ProtectedMediaPacket => resident_sfu_media_response(state),
    }
}

fn resident_sfu_ice_response(state: &mut ResidentSfuState) -> ResidentSfuOutcome {
    let session_id = session_ref();
    let candidate_ref = IceCandidateRef::new(session_id, "candidate:relay:resident-sfu")
        .expect("resident SFU candidate reference is fixed");
    let relation_ref = TransportIceRelationRef::new(accepted_reference(
        "ice-relation:resident-sfu",
        ReferenceAuthority::CoreValidatedUntrustedInput,
    ));
    let policy_ref = TransportPolicyRef::new(
        accepted_reference(
            "transport-policy:resident-sfu",
            ReferenceAuthority::CorePolicy,
        ),
        TransportPolicyState::AllowsCandidateRelation,
    );
    let input = TransportCandidateRelationInput::new(relation_ref, policy_ref, candidate_ref);

    match arcrtc_core_transport::relate_ice_candidate_to_transport(input) {
        TransportCandidateRelationDecision::Accepted(relation_ref) => {
            state.ice_relation = Some(relation_ref);
            accepted_resident_sfu_outcome(
                ResidentSfuSuccessKind::IceConnectivityAccepted,
                "secure=ice_connected",
                0,
                0,
                0,
            )
        }
        TransportCandidateRelationDecision::Rejected(rejection) => {
            rejected_resident_sfu_outcome(rejection.reason().reason_code())
        }
    }
}

fn resident_sfu_dtls_response(state: &mut ResidentSfuState) -> ResidentSfuOutcome {
    if state.ice_relation.is_none() {
        return rejected_resident_sfu_outcome(IceFailureKind::CommandOrderViolation.reason_code());
    }
    state.dtls_established = true;
    accepted_resident_sfu_outcome(
        ResidentSfuSuccessKind::DtlsHandshakeAccepted,
        "secure=dtls_established",
        0,
        0,
        0,
    )
}

fn resident_sfu_srtp_response(state: &mut ResidentSfuState) -> ResidentSfuOutcome {
    if !state.dtls_established {
        return rejected_resident_sfu_outcome(
            SecureMediaFailureKind::SecureMediaHandshakeFailed.reason_code(),
        );
    }
    state.srtp_active = true;
    accepted_resident_sfu_outcome(
        ResidentSfuSuccessKind::SrtpProtectionAccepted,
        "secure=srtp_active",
        0,
        0,
        0,
    )
}

fn resident_sfu_media_response(state: &mut ResidentSfuState) -> ResidentSfuOutcome {
    let secure_media = decide_secure_media_session(secure_media_input(state));
    if let SecureMediaSessionDecision::Protected = secure_media {
        let forwarding = apply_sfu_forwarding(
            SfuForwardingState::new(
                SfuSessionState::Open,
                SfuEndpointState::Admitted,
                SfuEndpointState::Admitted,
                SfuRouteState::Selected,
            ),
            SfuForwardingRequest::new(
                RouteId::new(accepted_reference(
                    "route:resident-sfu",
                    ReferenceAuthority::CorePolicy,
                )),
                PublicationDecision::Accepted,
                SubscriptionDecision::Accepted,
            ),
        );
        return match forwarding {
            ForwardingDecision::Selected(_) => {
                state.forwarded_packets = state.forwarded_packets.saturating_add(1);
                let digest = sfu_transmit_digest(state.forwarded_packets);
                accepted_resident_sfu_outcome(
                    ResidentSfuSuccessKind::ProtectedMediaForwarded,
                    "secure=protected",
                    1,
                    0,
                    digest,
                )
            }
            ForwardingDecision::Rejected(reason)
            | ForwardingDecision::Suppressed(reason)
            | ForwardingDecision::Dropped(reason)
            | ForwardingDecision::Failed(reason) => rejected_resident_sfu_outcome(reason.reason_code()),
        };
    }

    rejected_resident_sfu_outcome(secure_media_reason_code(secure_media))
}

fn secure_media_input(state: &ResidentSfuState) -> SecureMediaSessionInput {
    let ice_connectivity = match &state.ice_relation {
        Some(relation_ref) => IceConnectivityDecision::Connected(relation_ref.clone()),
        None => IceConnectivityDecision::Rejected(IceFailureKind::CommandOrderViolation),
    };
    let dtls_handshake = if state.dtls_established {
        DtlsHandshakeDecision::Established
    } else {
        DtlsHandshakeDecision::Rejected(SecureMediaFailureKind::SecureMediaHandshakeFailed)
    };
    let srtp_protection = if state.srtp_active {
        SrtpProtectionDecision::Active
    } else {
        SrtpProtectionDecision::Rejected(SecureMediaFailureKind::SecureMediaProtectionNotActive)
    };

    SecureMediaSessionInput::new(ice_connectivity, dtls_handshake, srtp_protection)
}

fn secure_media_reason_code(decision: SecureMediaSessionDecision) -> &'static str {
    match decision {
        SecureMediaSessionDecision::Protected => "protected",
        SecureMediaSessionDecision::Rejected(reason)
        | SecureMediaSessionDecision::Failed(reason) => match reason {
            SecureMediaSessionFailureReason::Ice(reason) => reason.reason_code(),
            SecureMediaSessionFailureReason::Dtls(reason)
            | SecureMediaSessionFailureReason::Srtp(reason) => reason.reason_code(),
        },
    }
}

fn accepted_resident_sfu_outcome(
    success_kind: ResidentSfuSuccessKind,
    detail_code: &'static str,
    transmits: usize,
    dropped: usize,
    digest: u64,
) -> ResidentSfuOutcome {
    ResidentSfuOutcome::Accepted {
        success_kind,
        detail_code,
        transmits,
        dropped,
        digest,
    }
}

fn rejected_resident_sfu_outcome(reason_code: &'static str) -> ResidentSfuOutcome {
    ResidentSfuOutcome::Rejected { reason_code }
}

fn accepted_reference(value: &'static str, authority: ReferenceAuthority) -> OpaqueReference {
    OpaqueReference::accept(value, authority).expect("resident SFU reference is fixed")
}

fn session_ref() -> SessionId {
    SessionId::new(accepted_reference(
        "session:resident-sfu",
        ReferenceAuthority::CorePolicy,
    ))
}

fn sfu_transmit_digest(packet_index: u64) -> u64 {
    0xcbf2_9ce4_8422_2325_u64
        .wrapping_mul(0x0000_0100_0000_01b3)
        .wrapping_add(packet_index)
}
