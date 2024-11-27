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
pub type IoError = std::io::Error;

/// [`std::io::ErrorKind`]
pub type IoErrorKind = std::io::ErrorKind;

/// [`std::io::Result`] with `T` if ok.
pub type IoResult<T> = std::io::Result<T>;

// std::io }

// Js Runtime {

#[derive(Debug, Clone, ThisError)]
/// Error code implemented by [`thiserror::Error`].
pub enum TheJsRuntimeErr {
  #[error("Error: {0}")]
  Message(String),
}

/// [`std::result::Result`] with `T` if ok, [`TheJsRuntimeErr`] if error.
pub type TheJsRuntimeResult<T> = std::result::Result<T, TheJsRuntimeErr>;

// Js Runtime }

// Buffer {

#[derive(Debug, ThisError)]
/// Vim buffer error code implemented by [`thiserror::Error`].
pub enum TheBufferErr {
  // #[error("File already exists: {0}")]
  // FileAlreadyExists(String),
  //
  // #[error("Buffer already binded: {0}")]
  // BufferAlreadyBinded(String),
  //
  // #[error("Io error: {0}")]
  // IoErr(IoError),
}

/// [`std::result::Result`] with `T` if ok, [`TheBufferErr`] if error.
pub type TheBufferResult<T> = std::result::Result<T, TheBufferErr>;

// Buffer }
