//! reference output境界のtyped errorです。

/// reference output package のerror閉集合です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceOutputError {
    /// output boundary外の型を要求されました。
    BoundaryViolation,
}
