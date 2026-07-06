/// system-wide continuity 判定が扱う component failure class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentFailureClass {
    /// process が終了しました。単体 process の観測だけでは system-wide 継続を証明しません。
    ProcessExit,
    /// driver failure が component 境界で観測されました。
    DriverFailure,
    /// durable state corruption が疑われます。
    StateCorruption,
    /// network partition が疑われます。
    NetworkPartition,
}

/// split-brain admission の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SplitBrainAdmissionDecision {
    /// split-brain admission を許可できます。
    Accepted,
    /// split-brain risk を理由に admission を拒否します。
    Rejected,
}

/// readiness transition の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReadinessTransitionDecision {
    /// readiness を維持できます。
    Ready,
    /// readiness を満たしません。
    NotReady,
    /// drain 状態へ遷移します。
    Draining,
}

/// system-wide failure continuity の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemWideFailureContinuityDecision {
    /// system-wide continuity を継続できます。
    Continues,
    /// system-wide continuity を degraded として扱います。
    Degrades,
    /// system-wide continuity を拒否します。
    Rejects,
}

/// system-wide failure continuity 判定の入力です。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SystemWideFailureContinuityInput {
    /// system-wide component を示す opaque reference です。
    pub component_ref: Option<OpaqueReference>,
    /// component failure class です。
    pub failure_class: ComponentFailureClass,
    /// readiness observation を示す opaque reference です。
    pub readiness_ref: Option<OpaqueReference>,
}

impl SystemWideFailureContinuityInput {
    /// component/ref と failure class を束ねます。
    pub const fn new(
        component_ref: Option<OpaqueReference>,
        failure_class: ComponentFailureClass,
        readiness_ref: Option<OpaqueReference>,
    ) -> Self {
        Self {
            component_ref,
            failure_class,
            readiness_ref,
        }
    }
}

impl ComponentFailureClass {
    /// failure class から split-brain admission を閉集合へ写像します。
    pub const fn split_brain_admission(self) -> SplitBrainAdmissionDecision {
        match self {
            Self::NetworkPartition | Self::StateCorruption => SplitBrainAdmissionDecision::Rejected,
            Self::ProcessExit | Self::DriverFailure => SplitBrainAdmissionDecision::Accepted,
        }
    }

    /// failure class と readiness ref から readiness transition を閉集合へ写像します。
    pub const fn readiness_transition(
        self,
        readiness_ref: Option<&OpaqueReference>,
    ) -> ReadinessTransitionDecision {
        match (self, readiness_ref.is_some()) {
            (Self::StateCorruption, true)
            | (Self::StateCorruption, false)
            | (Self::NetworkPartition, true)
            | (Self::NetworkPartition, false) => {
                ReadinessTransitionDecision::NotReady
            }
            (Self::ProcessExit, true) | (Self::ProcessExit, false) => {
                ReadinessTransitionDecision::Draining
            }
            (Self::DriverFailure, true) => ReadinessTransitionDecision::Ready,
            (Self::DriverFailure, false) => ReadinessTransitionDecision::NotReady,
        }
    }
}

/// split-brain/readiness transition を system-wide continuity の closed outcome へ写像します。
pub fn decide_failure_continuity(
    input: SystemWideFailureContinuityInput,
) -> SystemWideFailureContinuityDecision {
    if input.component_ref.is_none() {
        return SystemWideFailureContinuityDecision::Rejects;
    }

    match (
        input.failure_class.split_brain_admission(),
        input
            .failure_class
            .readiness_transition(input.readiness_ref.as_ref()),
    ) {
        (SplitBrainAdmissionDecision::Rejected, ReadinessTransitionDecision::Ready) => {
            SystemWideFailureContinuityDecision::Rejects
        }
        (SplitBrainAdmissionDecision::Rejected, ReadinessTransitionDecision::Draining) => {
            SystemWideFailureContinuityDecision::Rejects
        }
        (SplitBrainAdmissionDecision::Rejected, ReadinessTransitionDecision::NotReady) => {
            SystemWideFailureContinuityDecision::Rejects
        }
        (SplitBrainAdmissionDecision::Accepted, ReadinessTransitionDecision::Ready) => {
            SystemWideFailureContinuityDecision::Continues
        }
        (SplitBrainAdmissionDecision::Accepted, ReadinessTransitionDecision::Draining) => {
            SystemWideFailureContinuityDecision::Degrades
        }
        (SplitBrainAdmissionDecision::Accepted, ReadinessTransitionDecision::NotReady) => {
            SystemWideFailureContinuityDecision::Degrades
        }
    }
}
