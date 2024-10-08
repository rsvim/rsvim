//! Results and errors.

use thiserror::Error as ThisError;

/// [`anyhow::Error`]
pub type AnyErr = anyhow::Error;

/// [`anyhow::Result`] with `T` if ok, [`AnyErr`]([`anyhow::Error`]) if error.
pub type AnyResult<T> = anyhow::Result<T>;

/// [`std::io::Error`]
pub type IoError = std::io::Error;

/// [`std::io::ErrorKind`]
pub type IoErrorKind = std::io::ErrorKind;

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
