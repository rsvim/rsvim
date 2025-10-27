//! Errors and results.

use crate::buf::BufferId;
use crate::js::err::JsError;
use crate::js::module::ModulePath;
use compact_str::CompactString;
use std::borrow::Cow;

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
pub enum TheErr {
  // buf {
  #[error("Buffer `{0}` doesn't have a filename.")]
  BufferNoName(BufferId),

  #[error("Buffer `{0}` not exist.")]
  BufferNotExist(BufferId),

  #[error("Failed to save buffer `{0}`({1}): {2}.")]
  BufferSaveFailed(BufferId, String, IoErr),
  // buf }

  // js {
  //
  #[error("Command `{0}` not found.")]
  CommandNotFound(CompactString),

  #[error("Command `{0}` already exist.")]
  CommandAlreadyExist(CompactString),

  #[error("{0}")]
  JsError(Box<JsError>),

  #[error("Module `{0}` not found.")]
  ModuleNotFound(ModulePath),

  #[error("Failed to compile typescript: {0}.")]
  CompileTypeScriptFailed(Cow<'static, str>),

  #[error("Not enough arguments specified.")]
  ArgumentsNotEnough,

  #[error("Invalid data.")]
  DataInvalid,

  #[error("Value too large: `{0}`.")]
  ValueTooLarge(usize),

  #[error("Buffer too small: `{0}`.")]
  BufferTooSmall(usize),
  // js }
}

/// [`Result`] with `T` if ok, [`TheErr`] if error.
pub type TheResult<T> = Result<T, TheErr>;

// thiserror }

#[macro_export]
macro_rules! bail {
  ($e:expr) => {
    return Err($e)
  };
}
