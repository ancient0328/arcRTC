use arcrtc_entrypoint_configuration::{
    parse_production_profile, ProductionProfileClass, ProductionProfileInput, RollbackProfileClass,
    SecretSourceClass,
};

#[test]
fn production_profile_parses_typed_config_validation_observation() {
    let observation = parse_production_profile(ProductionProfileInput::new(
        ProductionProfileClass::Production,
        SecretSourceClass::ExternalSecretRef,
        RollbackProfileClass::Automatic,
    ));

    // configuration entrypoint は typed observation を作るだけで、readiness decision は所有しません。
    assert_eq!(
        observation.profile_class,
        ProductionProfileClass::Production
    );
    assert_eq!(
        observation.secret_source_class,
        SecretSourceClass::ExternalSecretRef
    );
    assert_eq!(
        observation.rollback_profile_class,
        RollbackProfileClass::Automatic
    );
}

#[test]
fn production_profile_covers_development_staging_and_rollback_classes() {
    let cases = [
        (
            ProductionProfileClass::Development,
            SecretSourceClass::FileRef,
            RollbackProfileClass::Disabled,
        ),
        (
            ProductionProfileClass::Staging,
            SecretSourceClass::EnvRef,
            RollbackProfileClass::Manual,
        ),
        (
            ProductionProfileClass::Production,
            SecretSourceClass::ExternalSecretRef,
            RollbackProfileClass::Automatic,
        ),
    ];

    for (profile_class, secret_source_class, rollback_profile_class) in cases {
        let observation = parse_production_profile(ProductionProfileInput::new(
            profile_class,
            secret_source_class,
            rollback_profile_class,
        ));
        assert_eq!(observation.profile_class, profile_class);
        assert_eq!(observation.secret_source_class, secret_source_class);
        assert_eq!(observation.rollback_profile_class, rollback_profile_class);
    }
}
