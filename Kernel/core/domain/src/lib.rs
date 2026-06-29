//! core/domain は DDD domain model と application use case の core surface です。
//!
//! aggregate、value object、domain service、use case はここで所有し、
//! driver implementation や composition root の判断を持ち込みません。

use core::marker::PhantomData;

/// core domain package の所有境界を示す marker です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreDomainSurface;

/// v0.2 初期 architecture で許可された aggregate family です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AggregateFamily {
    /// Signaling room / participant の状態と受理判定を扱う family です。
    SignalingRoomParticipant,
    /// SFU session / endpoint / route の media routing 状態を扱う family です。
    SfuSessionEndpointRoute,
    /// TURN allocation / permission / channel bind の relay lifecycle を扱う family です。
    TurnAllocationPermissionChannelBind,
    /// Signaling / SFU / TURN / ICE / secure media の参照関係を扱う family です。
    CrossPlaneBindingScope,
    /// audit event ordering と hash-chain semantics を扱う family です。
    AuditChainScope,
    /// startup / wiring policy acceptance を扱う family です。
    ConfigurationScope,
}

/// immutable semantics と equality を持つ core value object の境界です。
pub trait ValueObject: Clone + Eq {}

/// state transition と invariant を持つ aggregate root の境界です。
pub trait AggregateRoot {
    /// aggregate を識別する core-owned value object です。
    type Id: ValueObject;

    /// aggregate が属する許可済み family を返します。
    fn family(&self) -> AggregateFamily;
}

/// single aggregate に閉じない pure domain rule を表す domain service 境界です。
pub trait DomainService {
    /// service が責務を持つ aggregate family 群を返します。
    fn aggregate_families(&self) -> &'static [AggregateFamily];
}

/// application use case が辿る orchestration step です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UseCaseStep {
    /// driver 変換後の core-owned command を受け取る段階です。
    ReceiveCoreOwnedCommand,
    /// identity / correlation / idempotency / replay / version を確認する段階です。
    VerifyCoreGuards,
    /// aggregate または domain service に decision を委譲する段階です。
    DelegateDomainDecision,
    /// decision reason を closed reason vocabulary に接続する段階です。
    ConnectDecisionReason,
    /// port command、audit event、external response model の生成へ進む段階です。
    EmitCoreEffects,
}

/// application use case の入力型と出力型を core-owned 型に限定する境界です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UseCaseBoundary<Command, Result> {
    name: &'static str,
    steps: &'static [UseCaseStep],
    _command: PhantomData<Command>,
    _result: PhantomData<Result>,
}

impl<Command, Result> UseCaseBoundary<Command, Result> {
    /// use case の責務名と orchestration order を固定します。
    pub const fn new(name: &'static str, steps: &'static [UseCaseStep]) -> Self {
        Self {
            name,
            steps,
            _command: PhantomData,
            _result: PhantomData,
        }
    }

    /// use case の責務名です。
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// use case が従う orchestration order です。
    pub const fn steps(&self) -> &'static [UseCaseStep] {
        self.steps
    }
}

/// Canonical が定める application use case の標準順序です。
pub const ENTRYPOINTLICATION_USE_CASE_ORDER: &[UseCaseStep] = &[
    UseCaseStep::ReceiveCoreOwnedCommand,
    UseCaseStep::VerifyCoreGuards,
    UseCaseStep::DelegateDomainDecision,
    UseCaseStep::ConnectDecisionReason,
    UseCaseStep::EmitCoreEffects,
];
