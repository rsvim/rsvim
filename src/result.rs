//! Results and errors.

use std::borrow::Cow;
use std::fmt::{Debug, Display};

/// A collection of common error codes.
pub enum ErrorCode {
  Dummy,
  Message(Cow<'static, str>),
  IoError(std::io::Error),
}

impl std::error::Error for ErrorCode {}

impl Debug for ErrorCode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ErrorCode::Dummy => write!(f, "ErrorCode::Dummy"),
      ErrorCode::Message(msg) => write!(f, "ErrorCode::Message({})", msg),
      ErrorCode::IoError(e) => write!(f, "ErrorCode::IoError({:?})", e),
    }
  }
}

impl Display for ErrorCode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ErrorCode::Dummy => write!(f, "Dummy error"),
      ErrorCode::Message(msg) => write!(f, "Error: {}", msg),
      ErrorCode::IoError(e) => write!(f, "Io Error: {:?}", e),
    }
  }
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
