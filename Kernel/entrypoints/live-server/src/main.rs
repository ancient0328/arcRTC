//! live public probe server binary.

use arcrtc_entrypoint_live_server::{
    serve_live_public_probe_server, LivePublicProbeFailureKind, LivePublicProbeServerConfig,
};

fn main() -> Result<(), LivePublicProbeFailureKind> {
    // main は config construction と serve 委譲だけを持ち、domain 判断を行いません。
    serve_live_public_probe_server(LivePublicProbeServerConfig::local_default())
}
