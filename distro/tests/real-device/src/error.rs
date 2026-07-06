//! real-device wrapper local error です。

/// real-device wrapper error の閉集合です。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RealDeviceWrapperError {
    /// scope mismatch です。
    ScopeMismatch,
}
