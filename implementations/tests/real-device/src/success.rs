//! KPI-011 real-device success verdict の判定境界です。
#![allow(dead_code)]

use crate::{
    evidence::{
        validate_real_device_evidence_record, RealDeviceClass, RealDeviceEvidenceRecord,
        RealDeviceEvidenceValidationError,
    },
    redaction::{reject_raw_real_device_identifier_marker, RealDeviceRedactionError},
};

/// real-device success verdict です。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RealDeviceSuccessVerdict {
    /// six required row がすべて success evidence として成立しています。
    Pass,
}

/// real-device success validation error の閉集合です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RealDeviceSuccessValidationError {
    /// evidence extension validation error です。
    Evidence(RealDeviceEvidenceValidationError),
    /// 必須 row が不足しています。
    MissingRequiredRow,
    /// row が重複しています。
    DuplicateRequiredRow,
    /// exit status が success ではありません。
    NonZeroExitStatus,
    /// redacted identifier が不足しています。
    MissingRedactedIdentifier,
    /// raw identifier marker が残存しています。
    RawIdentifierMarker(RealDeviceRedactionError),
}

/// six required row の success evidence を検証します。
pub fn validate_real_device_success_evidence(
    records: &[RealDeviceEvidenceRecord],
) -> Result<RealDeviceSuccessVerdict, RealDeviceSuccessValidationError> {
    let mut classes = Vec::new();
    for record in records {
        if record.base.exit_status != Some(0) {
            validate_real_device_evidence_record(record)
                .map_err(RealDeviceSuccessValidationError::Evidence)?;
            return Err(RealDeviceSuccessValidationError::NonZeroExitStatus);
        }
        let Some(identifier) = record.redacted_device_identifier.as_deref() else {
            return Err(RealDeviceSuccessValidationError::MissingRedactedIdentifier);
        };
        reject_raw_real_device_identifier_marker(identifier)
            .map_err(RealDeviceSuccessValidationError::RawIdentifierMarker)?;
        validate_real_device_evidence_record(record)
            .map_err(RealDeviceSuccessValidationError::Evidence)?;
        if classes.contains(&record.device_class) {
            return Err(RealDeviceSuccessValidationError::DuplicateRequiredRow);
        }
        classes.push(record.device_class);
    }
    if required_device_classes()
        .iter()
        .all(|required| classes.contains(required))
    {
        Ok(RealDeviceSuccessVerdict::Pass)
    } else {
        Err(RealDeviceSuccessValidationError::MissingRequiredRow)
    }
}

fn required_device_classes() -> [RealDeviceClass; 6] {
    [
        RealDeviceClass::AndroidPhysical,
        RealDeviceClass::AndroidEmulator,
        RealDeviceClass::IosPhysical,
        RealDeviceClass::IosSimulator,
        RealDeviceClass::DesktopBrowser,
        RealDeviceClass::MobileBrowser,
    ]
}
