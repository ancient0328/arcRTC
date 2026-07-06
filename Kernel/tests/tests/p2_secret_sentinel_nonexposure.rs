use arcrtc_core_identity::{
    CredentialRef, OpaqueReference, ReferenceAuthority, UntrustedReference,
};
use arcrtc_regulated::RegulatedHashChainRecordPointer;

const SENTINEL: &str = "ARCRTC-P2-SENTINEL-0001";

fn assert_debug_hides_value<T: std::fmt::Debug>(label: &str, value: &T) {
    let debug = format!("{value:?}");
    println!("label={label} debug={debug}");
    assert!(
        !debug.contains(SENTINEL),
        "{label} Debug output must not expose sentinel"
    );
}

#[test]
fn sentinel_is_not_exposed_by_debug_representations() {
    let untrusted = UntrustedReference::new(SENTINEL);
    assert_debug_hides_value("untrusted", &untrusted);
    assert_eq!(untrusted.as_str(), SENTINEL);

    let opaque = OpaqueReference::accept(SENTINEL, ReferenceAuthority::CoreValidatedUntrustedInput)
        .expect("sentinel must be accepted as opaque reference");
    assert_debug_hides_value("opaque", &opaque);
    assert_eq!(opaque.as_str(), SENTINEL);

    let credential = CredentialRef::new(opaque.clone());
    assert_debug_hides_value("credential", &credential);

    let regulated = RegulatedHashChainRecordPointer::accept(SENTINEL)
        .expect("sentinel must be accepted as regulated pointer");
    assert_debug_hides_value("regulated", &regulated);
    assert_eq!(regulated.as_str(), SENTINEL);
}
