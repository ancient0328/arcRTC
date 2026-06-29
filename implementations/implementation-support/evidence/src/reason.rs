//! implementations-local evidence reason の閉集合です。

/// 実装側証跡で採用できる reason の閉集合です。
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ImplementationEvidenceReason {
    /// 期待された実装コマンドまたは bounded behavior が成功した状態です。
    ImplementationOk,
    /// 必要な Kernel public contract が存在しない状態です。
    KernelContractUnavailable,
    /// Kernel contract の形状が implementations 正典と一致しない状態です。
    KernelContractMismatch,
    /// dependency admission が存在しない状態です。
    DependencyNotAdmitted,
    /// bounded runtime executor が失敗した状態です。
    RuntimeExecutorError,
    /// state / persistence 境界の違反です。
    StateBoundaryViolation,
    /// fixture identity または credential が不正な状態です。
    FixtureIdentityInvalid,
    /// evidence field が不足している状態です。
    EvidenceFieldsIncomplete,
    /// command の working directory / target scope が一致しない状態です。
    CommandScopeMismatch,
    /// benchmark workload / environment scope が一致しない状態です。
    BenchmarkScopeMismatch,
    /// real-device command scope が一致しない状態です。
    RealDeviceScopeMismatch,
    /// readiness claim に必要な authority が採用されていない状態です。
    ReadinessNotAdmitted,
}
