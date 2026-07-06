use arcrtc_entrypoint_endpoints::{PublicEndpointClass, PublicEndpointFailureKind};

fn main() {
    let Some(token) = std::env::args().nth(1) else {
        eprintln!(
            "{}",
            PublicEndpointFailureKind::PublicEndpointNotAllowed.reason_code()
        );
        std::process::exit(2);
    };

    // endpoint class の分類を表示するだけで、listener bind や公開成立は主張しません。
    let endpoint_class = match token.as_str() {
        "signaling-public" => PublicEndpointClass::SignalingPublic,
        "turn-public-relay" => PublicEndpointClass::TurnPublicRelay,
        "sfu-media-public" => PublicEndpointClass::SfuMediaPublic,
        "health-public-readonly" => PublicEndpointClass::HealthPublicReadonly,
        "admin-private" => PublicEndpointClass::AdminPrivate,
        "internal-control-private" => PublicEndpointClass::InternalControlPrivate,
        "test-only-endpoint" => PublicEndpointClass::TestOnlyEndpoint,
        _ => {
            eprintln!(
                "{}",
                PublicEndpointFailureKind::PublicEndpointNotAllowed.reason_code()
            );
            std::process::exit(2);
        }
    };

    println!(
        "endpoint_class={token} public_surface={} test_only={} private_control_or_admin={}",
        endpoint_class.is_public_surface(),
        endpoint_class.is_test_only(),
        endpoint_class.is_private_control_or_admin()
    );
}
