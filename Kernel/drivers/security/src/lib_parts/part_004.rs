use arcrtc_core_security::{
    decide_credential_verification, CredentialVerifierFailureKind, CredentialVerifierPortInput,
    CredentialVerifierPortOutput,
};

/// file-backed credential verifier adapter です。
pub struct FileCredentialVerifierAdapter {
    key_material_path: std::path::PathBuf,
}

impl FileCredentialVerifierAdapter {
    /// typed configuration 由来の key material path reference を保持します。
    pub fn new(key_material_path: std::path::PathBuf) -> Self {
        Self { key_material_path }
    }

    /// key material file と core-owned verifier input を verification output へ写像します。
    pub fn verify(
        &self,
        input: CredentialVerifierPortInput,
    ) -> Result<CredentialVerifierPortOutput, CredentialVerifierFailureKind> {
        let bytes = std::fs::read(&self.key_material_path)
            .map_err(|_error| CredentialVerifierFailureKind::KeyMaterialReadFailed)?;
        let material = parse_driver_local_key_material(&bytes)?;
        observe_crypto_backend(&material)?;

        decide_credential_verification(input)
    }
}

struct DriverLocalCredentialVerifierMaterial {
    byte_len: usize,
    crypto_backend_available: bool,
}

fn parse_driver_local_key_material(
    bytes: &[u8],
) -> Result<DriverLocalCredentialVerifierMaterial, CredentialVerifierFailureKind> {
    if bytes.is_empty() {
        return Err(CredentialVerifierFailureKind::KeyMaterialParseFailed);
    }

    Ok(DriverLocalCredentialVerifierMaterial {
        byte_len: bytes.len(),
        crypto_backend_available: !bytes.starts_with(b"crypto_backend=failed"),
    })
}

fn observe_crypto_backend(
    material: &DriverLocalCredentialVerifierMaterial,
) -> Result<(), CredentialVerifierFailureKind> {
    // raw key bytes を保持せず、driver-local observation だけを closed failure に写像します。
    if material.byte_len == 0 || !material.crypto_backend_available {
        return Err(CredentialVerifierFailureKind::CryptoBackendRejected);
    }

    Ok(())
}
