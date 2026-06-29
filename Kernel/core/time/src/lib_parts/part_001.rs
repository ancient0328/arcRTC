// core/time は unit、measurement、time normalization の core-owned surface です。
//
// driver/platform 固有の raw measurement を policy decision に直接入れず、
// 正規化済み単位、丸め、比較条件、window を明示した値だけを扱います。

use arcrtc_core_identity::{CorrelationId, StartupRunId};

/// core time package の所有境界を示す marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreTimeSurface;

/// v0.2 initial architecture が認める normalized quantity です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NormalizedQuantity {
    /// duration.
    Duration,
    /// timestamp.
    Timestamp,
    /// bytes.
    Bytes,
    /// packet count.
    PacketCount,
    /// rate.
    Rate,
    /// ratio.
    Ratio,
    /// bitrate.
    Bitrate,
    /// jitter / RTT.
    JitterRtt,
}

/// normalized unit の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NormalizedUnit {
    /// milliseconds as integer.
    MillisecondsInteger,
    /// UTC epoch milliseconds for evidence only.
    UtcEpochMillisecondsEvidenceOnly,
    /// bytes as integer.
    BytesInteger,
    /// integer count.
    IntegerCount,
    /// units per second with explicit numerator.
    UnitsPerSecond,
    /// rational value with declared precision.
    RationalDeclaredPrecision,
    /// fixed decimal value with declared precision.
    FixedDecimalDeclaredPrecision,
    /// bits per second.
    BitsPerSecond,
    /// milliseconds with declared precision.
    MillisecondsDeclaredPrecision,
}

impl NormalizedQuantity {
    /// Canonical で許可された unit です。
    pub const fn default_unit(self) -> NormalizedUnit {
        match self {
            Self::Duration => NormalizedUnit::MillisecondsInteger,
            Self::Timestamp => NormalizedUnit::UtcEpochMillisecondsEvidenceOnly,
            Self::Bytes => NormalizedUnit::BytesInteger,
            Self::PacketCount => NormalizedUnit::IntegerCount,
            Self::Rate => NormalizedUnit::UnitsPerSecond,
            Self::Ratio => NormalizedUnit::RationalDeclaredPrecision,
            Self::Bitrate => NormalizedUnit::BitsPerSecond,
            Self::JitterRtt => NormalizedUnit::MillisecondsDeclaredPrecision,
        }
    }
}

/// hidden float comparison を避ける normalized value です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NormalizedValue {
    /// integer value.
    Integer(i128),
    /// rational value.
    Rational { numerator: i128, denominator: i128 },
    /// fixed decimal value.
    FixedDecimal { value: i128, scale: u32 },
}

/// normalized value の構築 error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NormalizedValueError {
    /// rational denominator must not be zero.
    ZeroDenominator,
}

impl NormalizedValue {
    /// rational value を作ります。
    pub const fn rational(
        numerator: i128,
        denominator: i128,
    ) -> Result<Self, NormalizedValueError> {
        if denominator == 0 {
            return Err(NormalizedValueError::ZeroDenominator);
        }
        Ok(Self::Rational {
            numerator,
            denominator,
        })
    }
}

/// measurement precision class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrecisionClass {
    /// integer exact.
    IntegerExact,
    /// declared decimal scale.
    DeclaredDecimalScale(u32),
    /// declared sampling precision label.
    DeclaredPrecisionLabel(&'static str),
}

/// rounding direction です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoundingDirection {
    /// exact value; no rounding.
    Exact,
    /// floor.
    Floor,
    /// ceiling.
    Ceiling,
    /// nearest with declared tie rule.
    Nearest,
    /// toward zero.
    TowardZero,
}

/// comparison operator です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComparisonOperator {
    /// less than.
    LessThan,
    /// less than or equal.
    LessThanOrEqual,
    /// equal.
    Equal,
    /// greater than or equal.
    GreaterThanOrEqual,
    /// greater than.
    GreaterThan,
}

/// inclusive/exclusive boundary です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoundaryInclusivity {
    /// inclusive.
    Inclusive,
    /// exclusive.
    Exclusive,
}

/// sampling window の正規化表現です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SamplingWindow {
    /// no sampling window.
    None,
    /// milliseconds.
    Milliseconds(u64),
    /// per second.
    PerSecond,
    /// explicitly declared policy label.
    PolicyLabel(&'static str),
}

/// raw measurement owner です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RawMeasurementOwner {
    /// driver owns raw platform measurement.
    Driver,
    /// entrypoints supply wall-clock timestamp observation.
    Entrypoints,
    /// benchmark docs/reports own benchmark reporting unit.
    BenchmarkDocsReports,
}

/// policy decision owner です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PolicyDecisionOwner {
    /// core owns policy comparison.
    Core,
    /// reports own evidence/benchmark claim adoption.
    Reports,
}

/// time source class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeSourceClass {
    /// monotonic observation through ClockPort.
    MonotonicClockPort,
    /// wall-clock timestamp for audit/evidence display only.
    WallClockEvidenceOnly,
}

/// normalized measurement comparison shape です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NormalizedMeasurement {
    quantity: NormalizedQuantity,
    unit: NormalizedUnit,
    value: NormalizedValue,
    precision: PrecisionClass,
    rounding: RoundingDirection,
    comparison_operator: ComparisonOperator,
    boundary: BoundaryInclusivity,
    sampling_window: SamplingWindow,
    raw_owner: RawMeasurementOwner,
    policy_owner: PolicyDecisionOwner,
}

impl NormalizedMeasurement {
    /// policy comparison に使える normalized measurement を作ります。
    pub const fn new(
        quantity: NormalizedQuantity,
        unit: NormalizedUnit,
        value: NormalizedValue,
        precision: PrecisionClass,
        rounding: RoundingDirection,
        comparison_operator: ComparisonOperator,
        boundary: BoundaryInclusivity,
        sampling_window: SamplingWindow,
        raw_owner: RawMeasurementOwner,
        policy_owner: PolicyDecisionOwner,
    ) -> Self {
        Self {
            quantity,
            unit,
            value,
            precision,
            rounding,
            comparison_operator,
            boundary,
            sampling_window,
            raw_owner,
            policy_owner,
        }
    }
}

/// time normalization failure mapping key です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeNormalizationFailureKind {
    /// measurement cannot be normalized.
    MeasurementNormalizationFailed,
    /// required time observation unavailable.
    TimeObservationUnavailable,
    /// operation deadline exceeded.
    OperationDeadlineExceeded,
    /// retention duration exceeded.
    RetentionDurationExceeded,
    /// runtime configuration invalid for time/measurement source.
    RuntimeConfigInvalid,
    /// driver shutdown during measurement.
    DriverShutdown,
}

impl TimeNormalizationFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::MeasurementNormalizationFailed => "measurement_normalization_failed",
            Self::TimeObservationUnavailable => "time_observation_unavailable",
            Self::OperationDeadlineExceeded => "operation_deadline_exceeded",
            Self::RetentionDurationExceeded => "retention_duration_exceeded",
            Self::RuntimeConfigInvalid => "runtime_config_invalid",
            Self::DriverShutdown => "driver_shutdown",
        }
    }
}

/// evidence/report が unit/window を持つかどうかの採用 class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EvidenceUnitWindowAdoption {
    /// raw source and normalized value are both recorded.
    RawAndNormalizedRecorded,
    /// normalized value is recorded for claim.
    NormalizedValueRecorded,
    /// unit/window missing, not adoptable for claim.
    MissingUnitWindowNotAdoptable,
}

/// unit/time normalization 境界で禁止する fail-open 動作です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProhibitedUnitMeasurementBehavior {
    /// driver-exported metric label becomes policy unit.
    DriverMetricLabelAsPolicyUnit,
    /// wall-clock order replaces monotonic deadline/expiry policy.
    WallClockOrderAsMonotonicPolicy,
    /// float comparison uses hidden precision.
    HiddenFloatPrecisionComparison,
    /// bitrate and byte-rate are conflated.
    BitrateByteRateConflated,
    /// benchmark report omits unit/window/aggregation.
    BenchmarkWithoutUnitWindowAggregation,
    /// raw platform-specific stats object enters core policy.
    RawPlatformStatsInCorePolicy,
    /// timezone conversion is treated as synchronization evidence.
    TimezoneConversionAsSynchronizationEvidence,
}

/// time synchronization / clock skew concern の owner です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeSynchronizationOwner {
    /// driver/runtime observes time source.
    DriverRuntimeObservation,
    /// core owns expiry/deadline/skew/timestamp trust policy.
    CorePolicy,
    /// entrypoints wire selected clock/time-source implementation.
    EntrypointsWiring,
    /// evidence Canonical owns evidence timestamp claim adoption.
    EvidenceCanonical,
}

/// time synchronization / skew boundary concern です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeSynchronizationConcern {
    /// local monotonic duration.
    LocalMonotonicDuration,
    /// wall-clock timestamp.
    WallClockTimestamp,
    /// cross-node skew policy.
    CrossNodeSkewPolicy,
    /// external time source.
    ExternalTimeSource,
    /// timestamp normalization.
    TimestampNormalization,
    /// evidence timestamp claim.
    EvidenceTimestampClaim,
}

impl TimeSynchronizationConcern {
    /// concern owner です。
    pub const fn owner(self) -> TimeSynchronizationOwner {
        match self {
            Self::LocalMonotonicDuration | Self::CrossNodeSkewPolicy => {
                TimeSynchronizationOwner::CorePolicy
            }
            Self::WallClockTimestamp | Self::ExternalTimeSource => {
                TimeSynchronizationOwner::DriverRuntimeObservation
            }
            Self::TimestampNormalization => TimeSynchronizationOwner::CorePolicy,
            Self::EvidenceTimestampClaim => TimeSynchronizationOwner::EvidenceCanonical,
        }
    }
}

/// v0.2 initial architecture が認める time trust class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeTrustClass {
    /// one process monotonic elapsed comparison.
    SingleProcessMonotonic,
    /// one node wall-clock timestamp.
    SingleNodeWallClock,
    /// nodes have measured skew within policy.
    MultiNodeBoundedSkew,
    /// accepted external time source observation.
    ExternalTrustedTimeSource,
    /// deterministic test clock.
    TestDeterministicClock,
    /// time source cannot be trusted for target claim.
    TimeUntrusted,
}

impl TimeTrustClass {
    /// production/runtime evidence として採用できる class です。
    pub const fn runtime_evidence_allowed(self) -> bool {
        match self {
            Self::TestDeterministicClock | Self::TimeUntrusted => false,
            Self::SingleProcessMonotonic
            | Self::SingleNodeWallClock
            | Self::MultiNodeBoundedSkew
            | Self::ExternalTrustedTimeSource => true,
        }
    }

    /// cross-node causal comparison を支えられる class です。
    pub const fn supports_cross_node_comparison(self) -> bool {
        match self {
            Self::MultiNodeBoundedSkew | Self::ExternalTrustedTimeSource => true,
            Self::SingleProcessMonotonic
            | Self::SingleNodeWallClock
            | Self::TestDeterministicClock
            | Self::TimeUntrusted => false,
        }
    }
}

/// node/process scope です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeNodeScope {
    /// single process.
    SingleProcess,
    /// single node.
    SingleNode,
    /// multiple nodes.
    MultiNode,
    /// multiple processes.
    MultiProcess,
}

/// time source class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrustedTimeSourceClass {
    /// local monotonic clock.
    LocalMonotonicClock,
    /// local wall clock.
    LocalWallClock,
    /// NTP/PTP/cloud metadata observation.
    ExternalTimeSourceObservation,
    /// deterministic test clock.
    DeterministicTestClock,
    /// source is unavailable or untrusted.
    UntrustedOrUnavailable,
}

/// skew observation class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObservedSkewClass {
    /// skew was not required for the claim.
    NotRequired,
    /// observed within policy.
    WithinPolicy,
    /// observed exceeding policy.
    ExceedsPolicy,
    /// required observation unavailable.
    ObservationUnavailable,
}

/// clock skew policy が claim に与える impact です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClockSkewImpact {
    /// expiry comparison.
    Expiry,
    /// ordering comparison.
    Ordering,
    /// audit timestamp claim.
    Audit,
    /// evidence timestamp claim.
    Evidence,
}

/// clock skew policy shape です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClockSkewPolicy {
    node_scope: TimeNodeScope,
    trust_class: TimeTrustClass,
    max_accepted_skew_ms: Option<u64>,
    observation_precision: PrecisionClass,
    measurement_window: SamplingWindow,
    time_source_class: TrustedTimeSourceClass,
    impact: ClockSkewImpact,
}

/// clock skew policy の fail-closed error です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClockSkewPolicyError {
    /// bounded skew class requires max skew.
    MaxSkewRequired,
    /// cross-node/process policy requires bounded or trusted class.
    TrustClassCannotSupportScope,
    /// external trusted source requires external source class.
    ExternalSourceClassRequired,
}

impl ClockSkewPolicy {
    /// skew policy を作ります。
    pub fn try_new(
        node_scope: TimeNodeScope,
        trust_class: TimeTrustClass,
        max_accepted_skew_ms: Option<u64>,
        observation_precision: PrecisionClass,
        measurement_window: SamplingWindow,
        time_source_class: TrustedTimeSourceClass,
        impact: ClockSkewImpact,
    ) -> Result<Self, ClockSkewPolicyError> {
        if matches!(trust_class, TimeTrustClass::MultiNodeBoundedSkew)
            && max_accepted_skew_ms.is_none()
        {
            return Err(ClockSkewPolicyError::MaxSkewRequired);
        }

        if matches!(
            node_scope,
            TimeNodeScope::MultiNode | TimeNodeScope::MultiProcess
        ) && !trust_class.supports_cross_node_comparison()
        {
            return Err(ClockSkewPolicyError::TrustClassCannotSupportScope);
        }

        if matches!(trust_class, TimeTrustClass::ExternalTrustedTimeSource)
            && !matches!(
                time_source_class,
                TrustedTimeSourceClass::ExternalTimeSourceObservation
            )
        {
            return Err(ClockSkewPolicyError::ExternalSourceClassRequired);
        }

        Ok(Self {
            node_scope,
            trust_class,
            max_accepted_skew_ms,
            observation_precision,
            measurement_window,
            time_source_class,
            impact,
        })
    }
}

