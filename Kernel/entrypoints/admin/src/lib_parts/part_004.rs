/// shutdown / drain の admin command 観測です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShutdownDrainCommand {
    /// shutdown/drain 状態の観測要求です。
    Observe,
    /// bounded drain の開始要求です。
    BeginDrain,
    /// bounded cancellation の要求です。
    CancelOutstanding,
    /// bounded join/wait の要求です。
    JoinOutstanding,
}

/// supervision の admin-owned observation です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupervisionObservation {
    /// supervisor は未観測です。
    NotObserved,
    /// supervisor は running を観測しました。
    Running,
    /// supervisor は draining を観測しました。
    Draining,
    /// supervisor は restart を観測しました。
    RestartObserved,
    /// supervisor observation は失敗しました。
    ObservationFailed,
}

/// shutdown / supervision observation の typed reference です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShutdownDrainObservationRef {
    value: &'static str,
}

impl ShutdownDrainObservationRef {
    /// typed observation ref を作ります。
    pub const fn new(value: &'static str) -> Self {
        Self { value }
    }

    /// observation ref value です。runtime lifecycle success を決定しません。
    pub const fn as_str(&self) -> &'static str {
        self.value
    }
}

/// shutdown / supervision observation の入力です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShutdownSupervisionObservationInput {
    command: ShutdownDrainCommand,
    supervision: SupervisionObservation,
    observation_ref: ShutdownDrainObservationRef,
}

impl ShutdownSupervisionObservationInput {
    /// admin command と supervision observation を束ねます。
    pub const fn new(
        command: ShutdownDrainCommand,
        supervision: SupervisionObservation,
        observation_ref: ShutdownDrainObservationRef,
    ) -> Self {
        Self {
            command,
            supervision,
            observation_ref,
        }
    }

    /// admin command です。
    pub const fn command(&self) -> ShutdownDrainCommand {
        self.command
    }

    /// supervision observation です。
    pub const fn supervision(&self) -> SupervisionObservation {
        self.supervision
    }
}

/// shutdown / supervision を typed observation ref として返します。
///
/// この関数は runtime lifecycle decision、runtime lifecycle success、domain state transition を生成しません。
pub const fn observe_shutdown_and_supervision(
    input: ShutdownSupervisionObservationInput,
) -> ShutdownDrainObservationRef {
    input.observation_ref
}
