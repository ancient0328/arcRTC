/// runtime profile selection の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeProfileSelection {
    /// local development profile の観測です。
    DevelopmentLocal,
    /// deterministic test profile の観測です。
    TestDeterministic,
    /// controlled integration profile の観測です。
    IntegrationControlled,
    /// benchmark controlled profile の観測です。
    BenchmarkControlled,
    /// production candidate profile の観測です。
    ProductionCandidate,
}

impl RuntimeProfileSelection {
    /// configuration profile class から runtime profile selection を作ります。
    pub const fn from_profile_class(profile_class: ConfigurationProfileClass) -> Self {
        match profile_class {
            ConfigurationProfileClass::DevelopmentLocal => Self::DevelopmentLocal,
            ConfigurationProfileClass::TestDeterministic => Self::TestDeterministic,
            ConfigurationProfileClass::IntegrationControlled => Self::IntegrationControlled,
            ConfigurationProfileClass::BenchmarkControlled => Self::BenchmarkControlled,
            ConfigurationProfileClass::ProductionCandidate => Self::ProductionCandidate,
        }
    }
}

/// runtime profile selection observation の typed reference です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RuntimeProfileObservationRef {
    value: &'static str,
}

impl RuntimeProfileObservationRef {
    /// typed observation ref を作ります。
    pub const fn new(value: &'static str) -> Self {
        Self { value }
    }

    /// observation ref value です。runtime success claim ではありません。
    pub const fn as_str(&self) -> &'static str {
        self.value
    }
}

/// runtime profile selection の入力です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RuntimeProfileSelectionInput {
    profile_class: ConfigurationProfileClass,
    selection: RuntimeProfileSelection,
    observation_ref: RuntimeProfileObservationRef,
}

impl RuntimeProfileSelectionInput {
    /// profile class と observation ref を束ねます。
    pub const fn new(
        profile_class: ConfigurationProfileClass,
        observation_ref: RuntimeProfileObservationRef,
    ) -> Self {
        Self {
            profile_class,
            selection: RuntimeProfileSelection::from_profile_class(profile_class),
            observation_ref,
        }
    }

    /// runtime profile selection です。
    pub const fn selection(&self) -> RuntimeProfileSelection {
        self.selection
    }

    /// configuration profile class です。
    pub const fn profile_class(&self) -> ConfigurationProfileClass {
        self.profile_class
    }
}

/// runtime profile selection を typed observation ref として返します。
///
/// この関数は runtime success、domain state transition、driver selection policy を生成しません。
pub const fn select_runtime_profile(
    input: RuntimeProfileSelectionInput,
) -> RuntimeProfileObservationRef {
    input.observation_ref
}
