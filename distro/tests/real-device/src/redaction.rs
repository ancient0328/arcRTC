//! real-device identifier redaction 境界です。
#![allow(dead_code)]

/// redaction error の閉集合です。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RealDeviceRedactionError {
    /// raw identifier marker が残存しています。
    RawIdentifierMarker,
}

/// raw identifier marker の残存を拒否します。
pub fn reject_raw_real_device_identifier_marker(
    value: &str,
) -> Result<(), RealDeviceRedactionError> {
    if contains_raw_identifier_marker(value) {
        Err(RealDeviceRedactionError::RawIdentifierMarker)
    } else {
        Ok(())
    }
}

/// raw identifier を採用せず、証跡用の redacted sentinel に変換します。
pub fn redact_real_device_identifier(
    value: Option<&str>,
) -> Result<String, RealDeviceRedactionError> {
    if let Some(value) = value {
        reject_raw_real_device_identifier_marker(value)?;
    }
    Ok("redacted".to_owned())
}

fn contains_raw_identifier_marker(value: &str) -> bool {
    let normalized = value.trim().to_ascii_lowercase();
    [
        "serial:",
        "serial=",
        "udid:",
        "udid=",
        "android_id:",
        "android_id=",
        "device_id:",
        "device_id=",
        "imei:",
        "imei=",
        "meid:",
        "meid=",
        "account:",
        "account=",
        "token:",
        "token=",
        "private_key:",
        "private_key=",
        "device_name:",
        "device_name=",
    ]
    .iter()
    .any(|marker| normalized.contains(marker))
}
