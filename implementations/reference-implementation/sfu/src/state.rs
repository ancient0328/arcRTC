//! reference SFU のin-memory state境界です。

use std::collections::BTreeMap;

use arcrtc_core_identity::{EndpointId, RouteId, SessionId, StreamId};
use arcrtc_core_sfu::SfuDecisionKind;
use arcrtc_implementation_evidence::ImplementationEvidenceReason;
use arcrtc_reference_output::ReferenceSfuOutcome;

use crate::error::ReferenceSfuError;

/// reference SFU suppression sourceです。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferenceSfuSuppressionSource {
    /// backpressure由来のsuppressionです。
    Backpressure,
    /// quality由来のsuppressionです。
    Quality,
}

/// reference SFU action の閉集合です。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReferenceSfuAction {
    /// participant endpointをadmitします。
    AdmitParticipant {
        /// session referenceです。
        session_id: SessionId,
        /// endpoint referenceです。
        endpoint_id: EndpointId,
    },
    /// participant endpointをrejectします。
    RejectParticipant {
        /// session referenceです。
        session_id: SessionId,
        /// endpoint referenceです。
        endpoint_id: EndpointId,
    },
    /// stream publicationです。
    PublishStream {
        /// session referenceです。
        session_id: SessionId,
        /// endpoint referenceです。
        endpoint_id: EndpointId,
        /// stream referenceです。
        stream_id: StreamId,
    },
    /// route subscriptionです。
    SubscribeRoute {
        /// session referenceです。
        session_id: SessionId,
        /// endpoint referenceです。
        endpoint_id: EndpointId,
        /// stream referenceです。
        stream_id: StreamId,
        /// route referenceです。
        route_id: RouteId,
    },
    /// route selectionです。
    SelectRoute {
        /// session referenceです。
        session_id: SessionId,
        /// endpoint referenceです。
        endpoint_id: EndpointId,
        /// stream referenceです。
        stream_id: StreamId,
        /// route referenceです。
        route_id: RouteId,
    },
    /// forwarding suppressionです。
    SuppressForwarding {
        /// route referenceです。
        route_id: RouteId,
        /// suppression sourceです。
        source: ReferenceSfuSuppressionSource,
    },
    /// forwarding dropです。
    DropForwarding {
        /// route referenceです。
        route_id: RouteId,
    },
    /// session closeです。
    CloseSession {
        /// session referenceです。
        session_id: SessionId,
    },
}

/// reference SFU session phaseです。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SfuSessionState {
    /// open sessionです。
    Open,
    /// draining sessionです。
    Draining,
    /// closed sessionです。
    Closed,
}

/// reference SFU endpoint phaseです。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SfuEndpointState {
    /// observed endpointです。
    Observed,
    /// admission pending endpointです。
    AdmissionPending,
    /// admitted endpointです。
    Admitted,
    /// degraded endpointです。
    Degraded,
    /// draining endpointです。
    Draining,
    /// removed endpointです。
    Removed,
    /// rejected endpointです。
    Rejected,
}

/// reference SFU route phaseです。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SfuRouteState {
    /// candidate routeです。
    Candidate,
    /// selected routeです。
    Selected,
    /// delayed routeです。
    Delayed,
    /// backpressureでsuppressedされたrouteです。
    SuppressedByBackpressure,
    /// qualityでsuppressedされたrouteです。
    SuppressedByQuality,
    /// degraded routeです。
    Degraded,
    /// dropped routeです。
    Dropped,
    /// closed routeです。
    Closed,
}

/// reference SFU session stateです。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceSfuSessionState {
    /// session referenceです。
    pub session_id: SessionId,
    /// session phaseです。
    pub phase: SfuSessionState,
}

/// reference SFU endpoint stateです。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceEndpointState {
    /// endpoint referenceです。
    pub endpoint_id: EndpointId,
    /// session referenceです。
    pub session_id: SessionId,
    /// endpoint phaseです。
    pub phase: SfuEndpointState,
}

/// reference SFU route stateです。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceRouteState {
    /// route referenceです。
    pub route_id: RouteId,
    /// session referenceです。
    pub session_id: SessionId,
    /// endpoint referenceです。
    pub endpoint_id: EndpointId,
    /// stream referenceです。
    pub stream_id: StreamId,
    /// route phaseです。
    pub phase: SfuRouteState,
}

/// reference SFU stateです。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReferenceSfuState {
    /// session_id文字列をkeyにしたsession stateです。
    pub sessions: BTreeMap<String, ReferenceSfuSessionState>,
    /// endpoint_id文字列をkeyにしたendpoint stateです。
    pub endpoints: BTreeMap<String, ReferenceEndpointState>,
    /// route_id文字列をkeyにしたroute stateです。
    pub routes: BTreeMap<String, ReferenceRouteState>,
}

/// reference SFU stateへactionを適用します。
pub fn apply_reference_sfu(
    state: &mut ReferenceSfuState,
    action: &ReferenceSfuAction,
) -> Result<ReferenceSfuOutcome, ReferenceSfuError> {
    match action {
        ReferenceSfuAction::AdmitParticipant {
            session_id,
            endpoint_id,
        } => {
            ensure_session_mutable(state, session_id)?;
            admit_endpoint(state, session_id, endpoint_id)?;
        }
        ReferenceSfuAction::RejectParticipant {
            session_id,
            endpoint_id,
        } => {
            ensure_session_mutable(state, session_id)?;
            reject_endpoint(state, session_id, endpoint_id)?;
        }
        ReferenceSfuAction::PublishStream {
            session_id,
            endpoint_id,
            ..
        } => {
            ensure_endpoint_admitted(state, session_id, endpoint_id)?;
        }
        ReferenceSfuAction::SubscribeRoute {
            session_id,
            endpoint_id,
            stream_id,
            route_id,
        } => {
            ensure_endpoint_admitted(state, session_id, endpoint_id)?;
            subscribe_route(state, session_id, endpoint_id, stream_id, route_id)?;
        }
        ReferenceSfuAction::SelectRoute {
            session_id,
            endpoint_id,
            stream_id,
            route_id,
        } => {
            ensure_endpoint_admitted(state, session_id, endpoint_id)?;
            let route = state
                .routes
                .get_mut(route_id.as_str())
                .ok_or(ReferenceSfuError::StateBoundaryViolation)?;
            if route.session_id.as_str() != session_id.as_str()
                || route.endpoint_id.as_str() != endpoint_id.as_str()
                || route.stream_id.as_str() != stream_id.as_str()
                || !matches!(
                    route.phase,
                    SfuRouteState::Candidate | SfuRouteState::Selected
                )
            {
                return Err(ReferenceSfuError::StateBoundaryViolation);
            }
            route.phase = SfuRouteState::Selected;
        }
        ReferenceSfuAction::SuppressForwarding { route_id, source } => {
            let route = state
                .routes
                .get_mut(route_id.as_str())
                .ok_or(ReferenceSfuError::StateBoundaryViolation)?;
            if route.phase != SfuRouteState::Selected {
                return Err(ReferenceSfuError::StateBoundaryViolation);
            }
            route.phase = match source {
                ReferenceSfuSuppressionSource::Backpressure => {
                    SfuRouteState::SuppressedByBackpressure
                }
                ReferenceSfuSuppressionSource::Quality => SfuRouteState::SuppressedByQuality,
            };
        }
        ReferenceSfuAction::DropForwarding { route_id } => {
            let route = state
                .routes
                .get_mut(route_id.as_str())
                .ok_or(ReferenceSfuError::StateBoundaryViolation)?;
            if matches!(route.phase, SfuRouteState::Dropped | SfuRouteState::Closed) {
                return Err(ReferenceSfuError::StateBoundaryViolation);
            }
            route.phase = SfuRouteState::Dropped;
        }
        ReferenceSfuAction::CloseSession { session_id } => {
            let session = state
                .sessions
                .get_mut(session_id.as_str())
                .ok_or(ReferenceSfuError::StateBoundaryViolation)?;
            if session.phase == SfuSessionState::Closed {
                return Err(ReferenceSfuError::StateBoundaryViolation);
            }
            session.phase = SfuSessionState::Closed;
            for endpoint in state.endpoints.values_mut() {
                if endpoint.session_id.as_str() == session_id.as_str() {
                    endpoint.phase = SfuEndpointState::Removed;
                }
            }
            for route in state.routes.values_mut() {
                if route.session_id.as_str() == session_id.as_str() {
                    route.phase = SfuRouteState::Closed;
                }
            }
        }
    }
    Ok(outcome(action_decision_kind(action)))
}

/// reference SFU stateを検証します。
pub fn validate_reference_sfu_state(state: &ReferenceSfuState) -> Result<(), ReferenceSfuError> {
    for endpoint in state.endpoints.values() {
        if !state.sessions.contains_key(endpoint.session_id.as_str()) {
            return Err(ReferenceSfuError::StateBoundaryViolation);
        }
    }
    for route in state.routes.values() {
        let endpoint = state
            .endpoints
            .get(route.endpoint_id.as_str())
            .ok_or(ReferenceSfuError::StateBoundaryViolation)?;
        if endpoint.session_id.as_str() != route.session_id.as_str()
            || !state.sessions.contains_key(route.session_id.as_str())
        {
            return Err(ReferenceSfuError::StateBoundaryViolation);
        }
    }
    Ok(())
}

fn ensure_session_mutable(
    state: &mut ReferenceSfuState,
    session_id: &SessionId,
) -> Result<(), ReferenceSfuError> {
    match state.sessions.get(session_id.as_str()) {
        Some(session) if session.phase == SfuSessionState::Open => Ok(()),
        Some(_) => Err(ReferenceSfuError::StateBoundaryViolation),
        None => {
            state.sessions.insert(
                session_id.as_str().to_owned(),
                ReferenceSfuSessionState {
                    session_id: session_id.clone(),
                    phase: SfuSessionState::Open,
                },
            );
            Ok(())
        }
    }
}

fn ensure_endpoint_admitted(
    state: &ReferenceSfuState,
    session_id: &SessionId,
    endpoint_id: &EndpointId,
) -> Result<(), ReferenceSfuError> {
    let session = state
        .sessions
        .get(session_id.as_str())
        .ok_or(ReferenceSfuError::StateBoundaryViolation)?;
    if session.phase != SfuSessionState::Open {
        return Err(ReferenceSfuError::StateBoundaryViolation);
    }
    let endpoint = state
        .endpoints
        .get(endpoint_id.as_str())
        .ok_or(ReferenceSfuError::StateBoundaryViolation)?;
    if endpoint.session_id.as_str() != session_id.as_str()
        || endpoint.phase != SfuEndpointState::Admitted
    {
        return Err(ReferenceSfuError::StateBoundaryViolation);
    }
    Ok(())
}

fn admit_endpoint(
    state: &mut ReferenceSfuState,
    session_id: &SessionId,
    endpoint_id: &EndpointId,
) -> Result<(), ReferenceSfuError> {
    if let Some(endpoint) = state.endpoints.get(endpoint_id.as_str()) {
        if endpoint.session_id.as_str() != session_id.as_str()
            || matches!(
                endpoint.phase,
                SfuEndpointState::Rejected | SfuEndpointState::Removed
            )
        {
            return Err(ReferenceSfuError::StateBoundaryViolation);
        }
        if endpoint.phase == SfuEndpointState::Admitted {
            return Ok(());
        }
    }
    state.endpoints.insert(
        endpoint_id.as_str().to_owned(),
        ReferenceEndpointState {
            endpoint_id: endpoint_id.clone(),
            session_id: session_id.clone(),
            phase: SfuEndpointState::Admitted,
        },
    );
    Ok(())
}

fn reject_endpoint(
    state: &mut ReferenceSfuState,
    session_id: &SessionId,
    endpoint_id: &EndpointId,
) -> Result<(), ReferenceSfuError> {
    if let Some(endpoint) = state.endpoints.get(endpoint_id.as_str()) {
        if endpoint.session_id.as_str() != session_id.as_str()
            || matches!(
                endpoint.phase,
                SfuEndpointState::Admitted
                    | SfuEndpointState::Degraded
                    | SfuEndpointState::Draining
                    | SfuEndpointState::Removed
            )
        {
            return Err(ReferenceSfuError::StateBoundaryViolation);
        }
        if endpoint.phase == SfuEndpointState::Rejected {
            return Ok(());
        }
    }
    state.endpoints.insert(
        endpoint_id.as_str().to_owned(),
        ReferenceEndpointState {
            endpoint_id: endpoint_id.clone(),
            session_id: session_id.clone(),
            phase: SfuEndpointState::Rejected,
        },
    );
    Ok(())
}

fn subscribe_route(
    state: &mut ReferenceSfuState,
    session_id: &SessionId,
    endpoint_id: &EndpointId,
    stream_id: &StreamId,
    route_id: &RouteId,
) -> Result<(), ReferenceSfuError> {
    if let Some(route) = state.routes.get(route_id.as_str()) {
        if route.session_id.as_str() != session_id.as_str()
            || route.endpoint_id.as_str() != endpoint_id.as_str()
            || route.stream_id.as_str() != stream_id.as_str()
            || matches!(route.phase, SfuRouteState::Closed | SfuRouteState::Dropped)
        {
            return Err(ReferenceSfuError::StateBoundaryViolation);
        }
        if route.phase == SfuRouteState::Candidate {
            return Ok(());
        }
        return Err(ReferenceSfuError::StateBoundaryViolation);
    }
    state.routes.insert(
        route_id.as_str().to_owned(),
        ReferenceRouteState {
            route_id: route_id.clone(),
            session_id: session_id.clone(),
            endpoint_id: endpoint_id.clone(),
            stream_id: stream_id.clone(),
            phase: SfuRouteState::Candidate,
        },
    );
    Ok(())
}

fn action_decision_kind(action: &ReferenceSfuAction) -> SfuDecisionKind {
    match action {
        ReferenceSfuAction::AdmitParticipant { .. }
        | ReferenceSfuAction::RejectParticipant { .. } => SfuDecisionKind::ParticipantAdmission,
        ReferenceSfuAction::PublishStream { .. } => SfuDecisionKind::Publication,
        ReferenceSfuAction::SubscribeRoute { .. } => SfuDecisionKind::Subscription,
        ReferenceSfuAction::SelectRoute { .. } => SfuDecisionKind::RouteSelection,
        ReferenceSfuAction::SuppressForwarding { .. } => SfuDecisionKind::Forwarding,
        ReferenceSfuAction::DropForwarding { .. } => SfuDecisionKind::BackpressureAction,
        ReferenceSfuAction::CloseSession { .. } => SfuDecisionKind::DegradationRecovery,
    }
}

fn outcome(kind: SfuDecisionKind) -> ReferenceSfuOutcome {
    ReferenceSfuOutcome::new(kind, ImplementationEvidenceReason::ImplementationOk)
}
