/// driver 内でのみ保持する key material です。raw bytes は外部へ公開しません。
pub struct LoadedKeyMaterial {
    bytes: Vec<u8>,
}

impl LoadedKeyMaterial {
    /// key material の byte 長だけを公開します。
    pub fn byte_len(&self) -> usize {
        self.bytes.len()
    }
}

/// typed configuration 由来の path から key material を読み込みます。
pub fn load_key_material_from_file(
    path: &std::path::Path,
    bounds: &KeyCacheRefreshBounds,
) -> Result<LoadedKeyMaterial, VerifierDriverFailure> {
    let bytes = std::fs::read(path).map_err(|_| {
        VerifierDriverFailure::from_backend_failure(VerifierBackendFailureClass::KeyUnavailable)
    })?;
    if bytes.len() > bounds.maximum_key_material_bytes {
        return Err(VerifierDriverFailure::from_backend_failure(
            VerifierBackendFailureClass::KeyUnavailable,
        ));
    }

    Ok(LoadedKeyMaterial { bytes })
}
