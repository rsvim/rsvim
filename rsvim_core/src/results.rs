//! Results and errors.

use thiserror::Error as ThisError;

// anyhow {

/// [`anyhow::Error`]
pub type AnyErr = anyhow::Error;

/// [`anyhow::Result`] with `T` if ok, [`AnyErr`]([`anyhow::Error`]) if error.
pub type AnyResult<T> = anyhow::Result<T>;

// anyhow {

// std::io {

/// [`std::io::Error`]
pub type IoErr = std::io::Error;

/// [`std::io::ErrorKind`]
pub type IoErrKind = std::io::ErrorKind;

/// [`std::io::Result`] with `T` if ok.
pub type IoResult<T> = std::io::Result<T>;

// std::io }

// Js Runtime {

#[derive(Debug, Clone, ThisError)]
/// Error code implemented by [`thiserror::Error`].
pub enum JsRuntimeErr {
  #[error("Error: {0}")]
  Message(String),
}

/// [`std::result::Result`] with `T` if ok, [`JsRuntimeErr`] if error.
pub type JsRuntimeResult<T> = std::result::Result<T, JsRuntimeErr>;

// Js Runtime }
