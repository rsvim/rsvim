//! Results and errors.

use thiserror::Error as ThisError;

/// [`anyhow::Error`](anyhow::Error)
pub type AnyErr = anyhow::Error;

/// [`Result`](std::result::Result) with `T` if ok, [`AnyErr`] if error.
pub type AnyResult<T> = std::result::Result<T, AnyErr>;

/// [`std::io::Error`](std::io::Error)
pub type IoError = std::io::Error;

/// [`std::io::Result`](std::io::Result) with `T` if ok.
pub type IoResult<T> = std::io::Result<T>;

#[derive(Debug, Clone, ThisError)]
/// Error code implemented by [`thiserror::Error`](thiserror::Error).
pub enum TheErr {
  #[error("message error: {0}")]
  Message(String),
}

/// [`Result`](std::result::Result) with `T` if ok, [`TheErr`] if error.
pub type TheResult<T> = std::result::Result<T, TheErr>;
