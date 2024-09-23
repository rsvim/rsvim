//! Results and errors.

use std::fmt::{Debug, Display};

#[derive(Debug)]
/// Error kind.
pub enum AnyErrorKind {
  DUMMY,
  MESSAGE,
}

/// Common error.
pub struct AnyError {
  kind: AnyErrorKind,
  message: String,
}

impl AnyError {
  pub fn dummy() -> Self {
    AnyError {
      kind: AnyErrorKind::DUMMY,
      message: "".into(),
    }
  }

  pub fn with_message(message: String) -> Self {
    AnyError {
      kind: AnyErrorKind::MESSAGE,
      message,
    }
  }

  pub fn kind(&self) -> AnyErrorKind {
    self.kind
  }

  pub fn message(&self) -> &String {
    &self.message
  }
}

impl Debug for AnyError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("AnyError")
      .field("kind", &self.kind)
      .field("message", &self.message)
      .finish()
  }
}

impl Display for AnyError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Error type:{:?}, message:{}", self.kind, self.message)
  }
}

impl std::error::Error for AnyError {}

/// Void value result, error uses [`ErrorCode`].
pub type VoidResult = Result<(), AnyError>;

/// Boolean value result, error uses [`ErrorCode`].
pub type BoolResult = Result<bool, AnyError>;

/// String value result, error uses [`ErrorCode`].
pub type StrResult = Result<String, AnyError>;

/// Void value result for [`std::io`].
pub type VoidIoResult = std::io::Result<()>;

/// Boolean value result for [`std::io`].
pub type BoolIoResult = std::io::Result<bool>;
/// String value result for [`std::io`].
pub type StrIoResult = std::io::Result<String>;

/// Generic result for [`std::io`].
pub type IoResult<T> = std::io::Result<T>;
