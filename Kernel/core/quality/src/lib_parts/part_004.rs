impl BackpressureDecision {
    /// Canonical mapping 済みの decision を作ります。
    pub fn try_new(
        kind: BackpressureDecisionKind,
        target: BackpressureTargetRef,
    ) -> Result<Self, BackpressureDecisionShapeError> {
        match kind {
            BackpressureDecisionKind::Accept => {
                if !matches!(target, BackpressureTargetRef::NotApplicable) {
                    return Err(BackpressureDecisionShapeError::AcceptTargetMustBeNotApplicable);
                }
            }
            BackpressureDecisionKind::DelayAction
            | BackpressureDecisionKind::SuppressRouteState
            | BackpressureDecisionKind::DegradeRoute
            | BackpressureDecisionKind::RejectRecoveryFromBackpressureState => {
                if !matches!(target, BackpressureTargetRef::Route(_)) {
                    return Err(BackpressureDecisionShapeError::RouteTargetRequired);
                }
            }
            BackpressureDecisionKind::SuppressForwarding
            | BackpressureDecisionKind::DropPacket
            | BackpressureDecisionKind::StopRetainingPacketByPressurePolicy => {
                if !matches!(target, BackpressureTargetRef::Packet(_)) {
                    return Err(BackpressureDecisionShapeError::PacketTargetRequired);
                }
            }
            BackpressureDecisionKind::SuppressSubscription => {
                if !matches!(target, BackpressureTargetRef::Subscription(_)) {
                    return Err(BackpressureDecisionShapeError::SubscriptionTargetRequired);
                }
            }
            BackpressureDecisionKind::CloseEndpoint => {
                if !matches!(target, BackpressureTargetRef::Endpoint(_)) {
                    return Err(BackpressureDecisionShapeError::EndpointTargetRequired);
                }
            }
        }

        Ok(Self { kind, target })
    }
}

/// audit backlog overflow 時の非再帰 overflow path を固定する rule です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuditBacklogOverflowRule {
    /// reserved non-recursive overflow record path に 1 件だけ記録します。
    EmitSingleReservedOverflowRecord,
    /// audit-required path を reject/stop します。
    RejectNewAuditRequiredPath,
    /// audit failed decision を closeout evidence に使うことを禁止します。
    ProhibitCloseoutEvidence,
}

/// resource/backpressure 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedResourceBoundaryBehavior {
    /// unbounded queue.
    UnboundedQueue,
    /// unbounded cache.
    UnboundedCache,
    /// unbounded retry.
    UnboundedRetry,
    /// unbounded packet retention.
    UnboundedPacketRetention,
    /// reason なしの best-effort fallback.
    BestEffortFallbackWithoutReason,
    /// core reason mapping なしの driver-local drop.
    DriverLocalDropWithoutCoreReasonMapping,
    /// entrypoints による core bound policy override.
    EntrypointsOverrideCoreBoundPolicy,
    /// runtime implementation に task/worker bound を隠すこと。
    HiddenRuntimeTaskWorkerBound,
}
