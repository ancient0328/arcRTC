use arcrtc_entrypoint_internal_control::{
    InternalControlPlaneFailureKind, InternalServiceTrustClass,
};

fn main() {
    let Some(token) = std::env::args().nth(1) else {
        eprintln!(
            "{}",
            InternalControlPlaneFailureKind::InternalControlMessageInvalid.reason_code()
        );
        std::process::exit(2);
    };

    // trust class の分類を表示するだけで、trust/auth 成立は主張しません。
    let trust_class = match token.as_str() {
        "service-identity-not-required" => InternalServiceTrustClass::ServiceIdentityNotRequired,
        "static-configured-service-identity" => {
            InternalServiceTrustClass::StaticConfiguredServiceIdentity
        }
        "mtls-peer-identity" => InternalServiceTrustClass::MtlsPeerIdentity,
        "signed-service-token-identity" => InternalServiceTrustClass::SignedServiceTokenIdentity,
        "mesh-asserted-service-identity" => InternalServiceTrustClass::MeshAssertedServiceIdentity,
        "test-service-identity" => InternalServiceTrustClass::TestServiceIdentity,
        "unauthenticated-internal-service-requested" => {
            InternalServiceTrustClass::UnauthenticatedInternalServiceRequested
        }
        _ => {
            eprintln!(
                "{}",
                InternalControlPlaneFailureKind::InternalControlMessageInvalid.reason_code()
            );
            std::process::exit(2);
        }
    };

    println!(
        "trust_class={token} credential_bearing={} test_only={} rejected_request_class={}",
        trust_class.is_credential_bearing(),
        trust_class.is_test_only(),
        trust_class.is_rejected_request_class()
    );
}
