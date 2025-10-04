//! Errors and results.

use crate::buf::BufferId;
use crate::js::err::JsError;
use compact_str::CompactString;

// anyhow {

/// [`anyhow::Error`]
pub type AnyError = anyhow::Error;

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

// thiserror {

/// All error codes.
#[derive(Debug, thiserror::Error)]
pub enum TheError {
  // buf {
  #[error("Buffer {0} doesn't have a filename")]
  /// Buffer doesn't have a filename.
  BufferHaveNoFileName(BufferId),

  #[error("Buffer {0} not found")]
  /// Buffer not found
  BufferNotFound(BufferId),

  #[error("Buffer {0} failed to write file: {1}")]
  /// Buffer failed to write file
  BufferWriteFileFailed(BufferId, IoErr),

  #[error("Buffer {0} failed to open(w) file: {1}")]
  /// Buffer failed to open(w) file
  BufferOpenwFileFailed(BufferId, IoErr),
  // buf }

  // js {
  #[error("Command {0} not found")]
  /// Command not found
  CommandNotFound(CompactString),

  #[error("Js error: {0}")]
  /// JavaScript error/exception
  JsErr(JsError),
  // js }
}

/// [`Result`] with `T` if ok, [`TheError`] if error.
pub type TheResult<T> = Result<T, TheError>;

// thiserror }

#[macro_export]
macro_rules! bail {
  ($e:expr) => {
    return Err($e);
  };
}
