//! command / behavior evidence の共通 record 型です。

use crate::reason::ImplementationEvidenceReason;

/// implementations command evidence の閉集合です。
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum ImplementationCommandClass {
    /// format command です。
    Format,
    /// build command です。
    Build,
    /// test command です。
    Test,
    /// benchmark command です。
    Benchmark,
    /// real-device command です。
    RealDevice,
    /// production readiness command です。
    ProductionReadiness,
    /// live readiness command です。
    LiveReadiness,
}

/// implementations evidence のlayer閉集合です。
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum ImplementationLayer {
    /// reference implementation layer です。
    Reference,
    /// product implementation layer です。
    Product,
    /// benchmark layer です。
    Benchmark,
    /// real-device layer です。
    RealDevice,
    /// readiness layer です。
    Readiness,
}

/// implementations evidence の対象plane閉集合です。
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum ImplementationPlane {
    /// Signaling plane です。
    Signaling,
    /// TURN plane です。
    Turn,
    /// SFU plane です。
    Sfu,
    /// composition plane です。
    Composition,
    /// ops plane です。
    Ops,
    /// policy plane です。
    Policy,
    /// persistence plane です。
    Persistence,
    /// deployment plane です。
    Deployment,
    /// monitoring plane です。
    Monitoring,
    /// rollback plane です。
    Rollback,
}

/// implementations evidence の環境class閉集合です。
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum ImplementationEnvironmentClass {
    /// local docs only environment です。
    LocalDocsOnly,
    /// local single host environment です。
    LocalSingleHost,
    /// controlled process environment です。
    ControlledProcess,
    /// benchmark host environment です。
    BenchmarkHost,
    /// bounded real-device environment です。
    RealDeviceBounded,
    /// deferred production environment です。
    ProductionDeferred,
    /// deferred live environment です。
    LiveDeferred,
}

/// evidence が主張しない範囲の閉集合です。
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum ImplementationNonClaimScope {
    /// source implementation completion を主張しないことを示します。
    SourceImplementationCompletionNotClaimed,
    /// product completion を主張しないことを示します。
    ProductCompletionNotClaimed,
    /// test pass を主張しないことを示します。
    TestPassNotClaimed,
    /// benchmark threshold satisfaction を主張しないことを示します。
    BenchmarkThresholdNotClaimed,
    /// command target success を主張しないことを示します。
    CommandTargetSuccessNotClaimed,
    /// behavior correctness を主張しないことを示します。
    BehaviorCorrectnessNotClaimed,
    /// native application readiness を主張しないことを示します。
    NativeApplicationReadinessNotClaimed,
    /// public distribution readiness を主張しないことを示します。
    PublicDistributionReadinessNotClaimed,
    /// production readiness を主張しないことを示します。
    ProductionReadinessNotClaimed,
    /// live readiness を主張しないことを示します。
    LiveReadinessNotClaimed,
    /// Kernel completion を主張しないことを示します。
    KernelCompletionNotClaimed,
    /// Kernel freeze を主張しないことを示します。
    KernelFreezeNotClaimed,
}

/// implementations evidence の共通recordです。
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ImplementationEvidenceRecord {
    /// command / report / span を接続する相関IDです。
    pub correlation_id: String,
    /// 実行または検証されたcommand文字列です。
    pub command: String,
    /// command を実行したworking directoryです。
    pub working_directory: String,
    /// package単位commandの場合の対象packageです。
    pub target_package: Option<String>,
    /// 証跡が対象にするscopeです。
    pub target_scope: String,
    /// command classです。
    pub command_class: ImplementationCommandClass,
    /// implementation layerです。
    pub implementation_layer: ImplementationLayer,
    /// target planeです。
    pub target_plane: ImplementationPlane,
    /// environment classです。
    pub environment_class: ImplementationEnvironmentClass,
    /// toolchain / runtime version表記です。
    pub toolchain_runtime_version: String,
    /// fixtureまたはworkloadの識別子です。
    pub input_fixture_or_workload: Option<String>,
    /// 期待結果です。
    pub expected_outcome: String,
    /// 実結果です。
    pub actual_outcome: String,
    /// process exit statusです。
    pub exit_status: Option<i32>,
    /// Kernel reason を参照する場合の文字列表現です。
    pub kernel_reason: Option<String>,
    /// implementations-local evidence reasonです。
    pub implementation_reason: ImplementationEvidenceReason,
    /// この証跡が主張しない範囲です。
    pub non_claim_scope: Vec<ImplementationNonClaimScope>,
    /// 再実行条件です。
    pub rerun_condition: String,
}
