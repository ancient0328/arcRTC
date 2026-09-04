use serde_json::Value;

const POLICY: &str = include_str!("../../scripts/release/supply-chain-policy.toml");
const MANIFEST_SCHEMA: &str =
    include_str!("../../scripts/release/release-artifact-manifest.schema.json");

fn required_manifest_schema() -> Value {
    serde_json::from_str(MANIFEST_SCHEMA).expect("release artifact manifest schema must be JSON")
}

#[test]
fn supply_chain_policy_declares_sbom_license_and_vulnerability_gates() {
    for section in [
        "[dependency_admission]",
        "[license_admission]",
        "[vulnerability_gate]",
        "[sbom]",
    ] {
        assert!(POLICY.contains(section), "{section}");
    }

    assert!(POLICY.contains("required_manifest_sources"));
    assert!(POLICY.contains("allowed_license_classes"));
    assert!(POLICY.contains("denied_license_classes"));
    assert!(POLICY.contains("release_blocking_severities"));
    assert!(POLICY.contains("required_formats"));
    assert!(POLICY.contains("spdx-json"));
    assert!(POLICY.contains("cyclonedx-json"));
}

#[test]
fn supply_chain_policy_declares_digest_signature_and_provenance_fields() {
    for section in ["[artifact_digest]", "[artifact_signature]", "[provenance]"] {
        assert!(POLICY.contains(section), "{section}");
    }

    assert!(POLICY.contains("digest_algorithms = [\"sha256\"]"));
    assert!(POLICY.contains("required_digest_fields"));
    assert!(POLICY.contains("signature_ref_required = true"));
    assert!(POLICY.contains("signature_ref_classes"));
    assert!(POLICY.contains("required_fields = [\"source_snapshot\", \"provenance_ref\"]"));
    assert!(POLICY.contains("provenance_ref_classes"));
}

#[test]
fn release_artifact_manifest_schema_requires_digest_signature_and_provenance() {
    let schema = required_manifest_schema();
    let required = schema["required"]
        .as_array()
        .expect("schema.required must be an array")
        .iter()
        .map(|value| value.as_str().expect("required field must be string"))
        .collect::<Vec<_>>();

    assert_eq!(
        required,
        [
            "artifact_name",
            "artifact_digest",
            "digest_algorithm",
            "signature_ref",
            "source_snapshot",
            "provenance_ref",
        ]
    );
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(
        schema["properties"]["digest_algorithm"]["enum"][0],
        "sha256"
    );
}
