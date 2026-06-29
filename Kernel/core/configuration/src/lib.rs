//! core/configuration は configuration と policy の意味論を core 側で保持する surface です。
//!
//! 環境変数、process args、外部ファイルの読み取りは entrypoints/drivers の責務とし、
//! ここでは core が解釈する設定値と policy 語彙を後続 task で配置します。

/// configuration boundary の所有 package が確定していることを示す marker です。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CoreConfigurationSurface;

/// configuration kind の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigurationKind {
    /// thresholds、bounds、accepted versions 等の core policy configuration です。
    CorePolicy,
    /// socket address、TLS files、DB DSN 等の driver runtime configuration です。
    DriverRuntime,
    /// selected drivers、process options 等の entrypoint composition configuration です。
    EntrypointComposition,
    /// signaling endpoint URL、reconnect behavior 等の SDK client configuration です。
    SdkClient,
    /// regulated-side mapping/enrichment configuration です。
    Regulated,
}

/// configuration owner です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigurationOwner {
    /// core owner です。
    Core,
    /// drivers owner です。
    Drivers,
    /// entrypoints owner です。
    Entrypoints,
    /// sdk owner です。
    Sdk,
    /// regulated owner です。
    Regulated,
}

/// configuration source class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigurationSourceClass {
    /// entrypoints が parse/wire した typed input です。
    TypedInput,
    /// environment variable です。core は直接読みません。
    EnvironmentVariable,
    /// external file です。core は直接読みません。
    File,
    /// process args です。core は直接読みません。
    ProcessArgs,
    /// OS settings です。core は直接読みません。
    OsSettings,
    /// cloud metadata です。core は直接読みません。
    CloudMetadata,
}

/// typed core policy configuration です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorePolicyConfiguration<Policy> {
    policy: Policy,
}

impl<Policy> CorePolicyConfiguration<Policy> {
    /// entrypoints が渡した typed policy input を core policy configuration として保持します。
    pub const fn new(policy: Policy) -> Self {
        Self { policy }
    }

    /// policy value です。
    pub const fn policy(&self) -> &Policy {
        &self.policy
    }
}

/// driver/entrypoint/sdk/regulated 側に属する configuration classification です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonCoreConfigurationBoundary {
    kind: ConfigurationKind,
    owner: ConfigurationOwner,
    source_class: ConfigurationSourceClass,
}

impl NonCoreConfigurationBoundary {
    /// non-core configuration の owner/source を明示します。
    pub const fn new(
        kind: ConfigurationKind,
        owner: ConfigurationOwner,
        source_class: ConfigurationSourceClass,
    ) -> Self {
        Self {
            kind,
            owner,
            source_class,
        }
    }

    /// configuration kind です。
    pub const fn kind(&self) -> ConfigurationKind {
        self.kind
    }

    /// owner です。
    pub const fn owner(&self) -> ConfigurationOwner {
        self.owner
    }
}

/// feature flag が許可される effect です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeatureFlagAllowedEffect {
    /// driver implementation selection です。
    SelectDriverImplementation,
    /// optional exporter enablement です。
    EnableOptionalExporter,
    /// external encoding selection です。
    ChooseExternalEncoding,
}

/// feature flag が暗黙に変更してはいけない core semantics です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeatureFlagProhibitedEffect {
    /// Signaling state machine の暗黙変更です。
    SignalingStateMachineChange,
    /// TURN lifecycle の暗黙変更です。
    TurnLifecycleChange,
    /// SFU routing semantics の暗黙変更です。
    SfuRoutingSemanticsChange,
    /// security verification bypass です。
    SecurityVerificationBypass,
    /// audit requirement bypass です。
    AuditRequirementBypass,
}

/// configuration failure mapping key です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigurationFailureKind {
    /// typed core policy configuration is invalid.
    CorePolicyConfigInvalid,
    /// required runtime configuration is missing.
    RuntimeConfigMissing,
    /// runtime configuration cannot initialize selected driver/entrypoint.
    RuntimeConfigInvalid,
    /// required secret source is unavailable.
    SecretUnavailable,
}

impl ConfigurationFailureKind {
    /// reason catalog へ接続する stable code です。
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::CorePolicyConfigInvalid => "core_policy_config_invalid",
            Self::RuntimeConfigMissing => "runtime_config_missing",
            Self::RuntimeConfigInvalid => "runtime_config_invalid",
            Self::SecretUnavailable => "secret_unavailable",
        }
    }
}

/// core が secrets を raw value として所有しないための境界です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecretBoundary {
    /// raw secret は driver/entrypoint concern です。
    RawSecretOutsideCore,
    /// core は opaque credential reference または verification result だけを扱います。
    OpaqueCredentialReferenceOnly,
}
