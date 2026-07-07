// Roadmap の assertion 名をそのまま残すため、このテストファイルだけ許可します。
#![allow(non_snake_case)]

use arcrtc_core_identity::{OpaqueReference, ReferenceAuthority, RouteId};
use arcrtc_core_sfu::{
    apply_sfu_forwarding, ForwardingDecision, PublicationDecision, SfuEndpointState,
    SfuFailureKind, SfuForwardingRequest, SfuForwardingState, SfuRouteState, SfuSessionState,
    SubscriptionDecision,
};

fn route_id(value: &'static str) -> RouteId {
    RouteId::new(
        OpaqueReference::accept(value, ReferenceAuthority::CorePolicy)
            .expect("test route reference is fixed"),
    )
}

fn state(route_state: SfuRouteState) -> SfuForwardingState {
    // source/target endpoint を分けて渡し、multi-peer route の判断を core/sfu に閉じる。
    SfuForwardingState::new(
        SfuSessionState::Open,
        SfuEndpointState::Admitted,
        SfuEndpointState::Admitted,
        route_state,
    )
}

fn request(
    publication_decision: PublicationDecision,
    subscription_decision: SubscriptionDecision,
) -> SfuForwardingRequest {
    SfuForwardingRequest::new(
        route_id("route:test:multi-peer"),
        publication_decision,
        subscription_decision,
    )
}

#[test]
fn assert_t_sfu_02__publication() {
    let decision = apply_sfu_forwarding(
        state(SfuRouteState::Selected),
        request(
            PublicationDecision::Rejected(SfuFailureKind::PublicationNotAllowed),
            SubscriptionDecision::Accepted,
        ),
    );

    assert_eq!(
        decision,
        ForwardingDecision::Rejected(SfuFailureKind::PublicationNotAllowed)
    );
}

#[test]
fn assert_t_sfu_02__subscription() {
    let decision = apply_sfu_forwarding(
        state(SfuRouteState::Selected),
        request(
            PublicationDecision::Accepted,
            SubscriptionDecision::Rejected(SfuFailureKind::SubscriptionNotAllowed),
        ),
    );

    assert_eq!(
        decision,
        ForwardingDecision::Rejected(SfuFailureKind::SubscriptionNotAllowed)
    );
}

#[test]
fn assert_t_sfu_02__forwarding_selected() {
    let decision = apply_sfu_forwarding(
        state(SfuRouteState::Selected),
        request(
            PublicationDecision::Accepted,
            SubscriptionDecision::Accepted,
        ),
    );

    match decision {
        ForwardingDecision::Selected(route) => {
            assert_eq!(route.as_str(), "route:test:multi-peer");
        }
        other => panic!("forwarding must be selected, got {other:?}"),
    }
}

#[test]
fn assert_t_sfu_02__forwarding_dropped() {
    let decision = apply_sfu_forwarding(
        state(SfuRouteState::Dropped),
        request(
            PublicationDecision::Accepted,
            SubscriptionDecision::Accepted,
        ),
    );

    assert_eq!(
        decision,
        ForwardingDecision::Dropped(SfuFailureKind::PacketDroppedByBackpressure)
    );
}

#[test]
fn assert_t_sfu_02__forwarding_failed() {
    let decision = apply_sfu_forwarding(
        state(SfuRouteState::Closed),
        request(
            PublicationDecision::Accepted,
            SubscriptionDecision::Accepted,
        ),
    );

    assert_eq!(
        decision,
        ForwardingDecision::Failed(SfuFailureKind::TargetUnavailable)
    );
}
