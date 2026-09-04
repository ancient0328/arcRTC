/// production profile parsing が扱う profile class です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProductionProfileClass {
    /// development profile observation.
    Development,
    /// staging profile observation.
    Staging,
    /// production profile observation.
    Production,
}

/// secret source の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecretSourceClass {
    /// file reference source.
    FileRef,
    /// environment variable reference source.
    EnvRef,
    /// external secret manager reference source.
    ExternalSecretRef,
}

/// rollback profile の閉集合です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RollbackProfileClass {
    /// rollback disabled observation.
    Disabled,
    /// manual rollback observation.
    Manual,
    /// automatic rollback observation.
    Automatic,
}

/// typed config validation observation です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypedConfigValidationObservation {
    /// parsed profile class です。
    pub profile_class: ProductionProfileClass,
    /// secret source class です。
    pub secret_source_class: SecretSourceClass,
    /// rollback profile class です。
    pub rollback_profile_class: RollbackProfileClass,
}

/// production profile parser input です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProductionProfileInput {
    /// parsed profile class です。
    pub profile_class: ProductionProfileClass,
    /// secret source class です。
    pub secret_source_class: SecretSourceClass,
    /// rollback profile class です。
    pub rollback_profile_class: RollbackProfileClass,
}

impl ProductionProfileInput {
    /// config parser が観測した profile / secret / rollback class を束ねます。
    pub const fn new(
        profile_class: ProductionProfileClass,
        secret_source_class: SecretSourceClass,
        rollback_profile_class: RollbackProfileClass,
    ) -> Self {
        Self {
            profile_class,
            secret_source_class,
            rollback_profile_class,
        }
    }
}

/// production profile を typed config observation に変換します。
///
/// entrypoint configuration は domain decision、readiness outcome、secret material を生成しません。
pub const fn parse_production_profile(
    input: ProductionProfileInput,
) -> TypedConfigValidationObservation {
    TypedConfigValidationObservation {
        profile_class: input.profile_class,
        secret_source_class: input.secret_source_class,
        rollback_profile_class: input.rollback_profile_class,
    }
}
