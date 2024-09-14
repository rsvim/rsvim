//! Results and errors.

/// A collection of common error codes.
pub enum ErrorCode {
  Dummy,
  Message(String),
}

/// Void value result, error uses [`ErrorCode`].
pub type VoidResult = Result<(), ErrorCode>;

/// Boolean value result, error uses [`ErrorCode`].
pub type BoolResult = Result<bool, ErrorCode>;

/// String value result, error uses [`ErrorCode`].
pub type StrResult = Result<String, ErrorCode>;

/// Void value result for [`std::io`].
pub type VoidIoResult = std::io::Result<()>;

/// Boolean value result for [`std::io`].
pub type BoolIoResult = std::io::Result<bool>;
/// String value result for [`std::io`].
pub type StrIoResult = std::io::Result<String>;

/// Generic result for [`std::io`].
pub type IoResult<T> = std::io::Result<T>;
