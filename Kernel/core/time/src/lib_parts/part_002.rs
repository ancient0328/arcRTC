/// ordering claim の根拠 class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimestampOrderingBasis {
    /// correlation relation.
    Correlation,
    /// explicit sequence.
    Sequence,
    /// idempotency relation.
    Idempotency,
    /// aggregate version.
    AggregateVersion,
    /// protocol state.
    ProtocolState,
    /// bounded skew policy.
    BoundedSkewPolicy,
    /// audit hash-chain sequence.
    AuditHashChainSequence,
    /// audit/event sequence.
    EventSequence,
    /// timestamp presentation only, not ordering authority.
    PresentationOnlyTimestamp,
}

/// expiry/deadline comparison が使った time basis です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExpiryDeadlineTimeBasis {
    /// monotonic duration.
    MonotonicDuration,
    /// wall-clock timestamp.
    WallClockTimestamp,
    /// bounded skew class.
    BoundedSkew,
}

/// time synchronization failure mapping key です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeSynchronizationFailureKind {
    /// measured skew exceeds policy.
    ClockSkewExceeded,
    /// time source is not trusted for the target claim.
    TimeSourceUntrusted,
    /// required time synchronization observation unavailable.
    TimeSyncUnavailable,
    /// timestamp order cannot be trusted for target comparison.
    TimestampOrderUntrusted,
    /// local time observation unavailable.
    TimeObservationUnavailable,
}

impl TimeSynchronizationFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::ClockSkewExceeded => "clock_skew_exceeded",
            Self::TimeSourceUntrusted => "time_source_untrusted",
            Self::TimeSyncUnavailable => "time_sync_unavailable",
            Self::TimestampOrderUntrusted => "timestamp_order_untrusted",
            Self::TimeObservationUnavailable => "time_observation_unavailable",
        }
    }
}

/// time synchronization decision outcome です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeSynchronizationOutcome {
    /// accepted.
    Accepted,
    /// rejected.
    Rejected,
    /// unavailable.
    Unavailable,
    /// untrusted.
    Untrusted,
}

/// time_synchronization_decision の core 形状です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TimeSynchronizationDecision {
    startup_run_id: StartupRunId,
    correlation_id: Option<CorrelationId>,
    policy: ClockSkewPolicy,
    observed_skew_class: ObservedSkewClass,
    outcome: TimeSynchronizationOutcome,
    reason: Option<TimeSynchronizationFailureKind>,
}

/// time synchronization decision の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeSynchronizationDecisionError {
    /// non-success outcome requires cataloged reason.
    ReasonRequired,
    /// accepted outcome must not carry fake reason.
    ReasonMustBeAbsentForAccepted,
}

impl TimeSynchronizationDecision {
    /// time_synchronization_decision に投影できる decision を作ります。
    pub fn try_new(
        startup_run_id: StartupRunId,
        correlation_id: Option<CorrelationId>,
        policy: ClockSkewPolicy,
        observed_skew_class: ObservedSkewClass,
        outcome: TimeSynchronizationOutcome,
        reason: Option<TimeSynchronizationFailureKind>,
    ) -> Result<Self, TimeSynchronizationDecisionError> {
        match outcome {
            TimeSynchronizationOutcome::Accepted if reason.is_some() => {
                return Err(TimeSynchronizationDecisionError::ReasonMustBeAbsentForAccepted);
            }
            TimeSynchronizationOutcome::Rejected
            | TimeSynchronizationOutcome::Unavailable
            | TimeSynchronizationOutcome::Untrusted
                if reason.is_none() =>
            {
                return Err(TimeSynchronizationDecisionError::ReasonRequired);
            }
            _ => {}
        }

        Ok(Self {
            startup_run_id,
            correlation_id,
            policy,
            observed_skew_class,
            outcome,
            reason,
        })
    }

    /// audit event type code です。
    pub const fn audit_event_type(&self) -> &'static str {
        "time_synchronization_decision"
    }
}

/// time synchronization / clock skew 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedTimeSynchronizationBehavior {
    /// wall-clock timestamp is cross-node causal order without a bounded-skew observation.
    WallClockAsCrossNodeCausalOrder,
    /// report creation time is runtime observation time.
    ExternalRecordCreationTimeAsRuntimeObservation,
    /// deterministic test clock is trusted as a production clock.
    DeterministicTestClockAsProductionClockTrust,
    /// driver-local NTP status redefines core expiry/deadline policy.
    DriverNtpStatusRedefinesCorePolicy,
    /// timezone conversion is synchronization proof.
    TimezoneConversionAsSynchronizationProof,
    /// missing skew observation is accepted as within-bound observation.
    MissingSkewObservationAccepted,
}
