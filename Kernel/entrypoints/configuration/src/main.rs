use arcrtc_entrypoint_configuration::{ConfigurationBundleFailureKind, ConfigurationProfileClass};

fn main() {
    let Some(token) = std::env::args().nth(1) else {
        eprintln!(
            "{}",
            ConfigurationBundleFailureKind::RuntimeConfigMissing.reason_code()
        );
        std::process::exit(2);
    };

    // profile token の source-owned runtime selection を表示します。
    let class = match token.as_str() {
        "development-local" => ConfigurationProfileClass::DevelopmentLocal,
        "test-deterministic" => ConfigurationProfileClass::TestDeterministic,
        "integration-controlled" => ConfigurationProfileClass::IntegrationControlled,
        "benchmark-controlled" => ConfigurationProfileClass::BenchmarkControlled,
        "production-candidate" => ConfigurationProfileClass::ProductionCandidate,
        _ => {
            eprintln!(
                "{}",
                ConfigurationBundleFailureKind::RuntimeConfigInvalid.reason_code()
            );
            std::process::exit(2);
        }
    };

    println!("profile_class={token} selected_profile={class:?}");
}
