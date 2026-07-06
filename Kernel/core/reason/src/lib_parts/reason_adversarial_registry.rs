/// adversarial coverage が対象にする surface class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdversarialSurfaceClass {
    /// Signaling wire surface.
    SignalingWire,
    /// TURN packet surface.
    TurnPacket,
    /// SFU datagram surface.
    SfuDatagram,
    /// SDK command surface.
    SdkCommand,
    /// config input surface.
    ConfigInput,
}

/// protocol conformance class の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProtocolConformanceClass {
    /// valid protocol input.
    Valid,
    /// malformed protocol input.
    Malformed,
    /// unsupported protocol input.
    Unsupported,
}

/// fuzz target surface class の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FuzzSurfaceClass {
    /// parser fuzz surface.
    Parser,
    /// state transition fuzz surface.
    StateTransition,
    /// driver adapter fuzz surface.
    DriverAdapter,
}

/// closed reason coverage registry の 1 行です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClosedReasonCoverageRow {
    /// adversarial surface class です。
    pub surface_class: AdversarialSurfaceClass,
    /// closed reason catalog 上の reference です。free-text reason は受け取りません。
    pub closed_reason_ref: CatalogedReasonRef,
    /// assertion source への opaque reference です。
    pub assertion_ref: &'static str,
}

impl ClosedReasonCoverageRow {
    /// closed reason coverage row を作ります。
    pub const fn new(
        surface_class: AdversarialSurfaceClass,
        closed_reason_ref: CatalogedReasonRef,
        assertion_ref: &'static str,
    ) -> Self {
        Self {
            surface_class,
            closed_reason_ref,
            assertion_ref,
        }
    }
}

/// closed reason coverage row registration の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClosedReasonCoverageRegistrationDecision {
    /// registry row を受理しました。
    Registered,
    /// registry row を拒否しました。
    Rejected,
}

/// closed reason registry row だけを受け取り、free-text reason を registry に入れません。
pub const fn register_closed_reason_coverage(
    row: ClosedReasonCoverageRow,
) -> ClosedReasonCoverageRegistrationDecision {
    if row.assertion_ref.is_empty() {
        ClosedReasonCoverageRegistrationDecision::Rejected
    } else {
        ClosedReasonCoverageRegistrationDecision::Registered
    }
}
