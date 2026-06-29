//! shared evidence package のtyped errorです。

use crate::validation::EvidenceValidationError;

/// evidence helper が返す package-local errorです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImplementationEvidenceError {
    /// base evidence validation に失敗しました。
    Validation(EvidenceValidationError),
}
