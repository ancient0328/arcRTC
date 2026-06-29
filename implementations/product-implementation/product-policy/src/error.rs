//! product policy のtyped errorです。

/// product policy error の閉集合です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProductPolicyError {
    /// fixture identity が不正です。
    InvalidFixtureIdentity,
    /// security reason mapping に失敗しました。
    SecurityReasonMappingFailed,
    /// readiness は採用されていません。
    ReadinessNotAdmitted,
}

impl ProductPolicyError {
    /// implementation evidence reasonへ変換します。
    pub const fn implementation_reason(
        &self,
    ) -> arcrtc_implementation_evidence::ImplementationEvidenceReason {
        match self {
            Self::InvalidFixtureIdentity => {
                arcrtc_implementation_evidence::ImplementationEvidenceReason::FixtureIdentityInvalid
            }
            Self::SecurityReasonMappingFailed => {
                arcrtc_implementation_evidence::ImplementationEvidenceReason::StateBoundaryViolation
            }
            Self::ReadinessNotAdmitted => {
                arcrtc_implementation_evidence::ImplementationEvidenceReason::ReadinessNotAdmitted
            }
        }
    }
}
