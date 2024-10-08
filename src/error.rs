//! Errors.

use thiserror::Error as ThisError;

/// [`anyhow::Error`](anyhow::Error)
pub type AhErr = anyhow::Error;

/// [`std::io::Error`](std::io::Error)
pub type IoError = std::io::Error;

/// [`std::io::Result`](std::io::Result)
pub type IoResult<T> = std::io::Result<T>;

#[derive(Debug, Clone, ThisError)]
/// Error code implemented by [`thiserror::Error`](thiserror::Error).
pub enum TeErr {
  #[error("message error: {0}")]
  MESSAGE(String),
}
