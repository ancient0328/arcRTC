use arcrtc_driver_security::{
    load_key_material_from_file, KeyCacheRefreshBounds, VerifierBackendFailureClass,
    VerifierDriverFailure,
};

struct DirGuard(std::path::PathBuf);

impl Drop for DirGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn temp_dir() -> DirGuard {
    let path = std::env::temp_dir().join(format!("arcrtc_key_material_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("temp dir must be created");
    DirGuard(path)
}

fn assert_key_unavailable(
    label: &str,
    result: Result<arcrtc_driver_security::LoadedKeyMaterial, VerifierDriverFailure>,
) {
    match result {
        Ok(material) => panic!("{label} unexpectedly loaded {} bytes", material.byte_len()),
        Err(failure) => {
            println!("label={label} failure={failure:?}");
            assert_eq!(
                failure,
                VerifierDriverFailure::from_backend_failure(
                    VerifierBackendFailureClass::KeyUnavailable
                )
            );
        }
    }
}

#[test]
fn key_material_is_loaded_from_explicit_file_with_bounds() {
    let dir = temp_dir();
    let path = dir.0.join("key-material.bin");
    std::fs::write(&path, b"test-key-material").expect("key material fixture must be written");

    let bounds =
        KeyCacheRefreshBounds::try_new(8, 65_536, 60_000, 3, 5_000).expect("bounds must be valid");
    let material = load_key_material_from_file(&path, &bounds).expect("key material must load");
    println!("key_path={path:?} byte_len={}", material.byte_len());
    assert_eq!(material.byte_len(), 17);

    let missing = load_key_material_from_file(&dir.0.join("missing.bin"), &bounds);
    assert_key_unavailable("missing", missing);

    let tight_bounds =
        KeyCacheRefreshBounds::try_new(8, 8, 60_000, 3, 5_000).expect("bounds must be valid");
    let over_bound = load_key_material_from_file(&path, &tight_bounds);
    assert_key_unavailable("over_bound", over_bound);
}
