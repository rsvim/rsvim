//! Errors and results.

use crate::buf::BufferId;
use crate::js::err::JsError;
use crate::js::module::ModulePath;
use compact_str::CompactString;

// // anyhow {
//
// /// [`anyhow::Error`]
// pub type AnyError = anyhow::Error;
//
// /// [`anyhow::Result`] with `T` if ok, [`AnyErr`]([`anyhow::Error`]) if error.
// pub type AnyResult<T> = anyhow::Result<T>;
//
// // anyhow {

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

  #[error("Failed to write buffer {0}: {1}")]
  /// Buffer failed to write file
  WriteBufferFailed(BufferId, IoErr),

  #[error("Failed to open(w) file {0}: {1}")]
  /// Buffer failed to open(w) file
  WriteBufferOpenFailed(String, IoErr),
  // buf }

  // js {
  #[error("Command {0} not found")]
  /// Command not found
  CommandNotFound(CompactString),

  #[error("Command name {0} already exist")]
  /// Command not found
  CommandNameAlreadyExist(CompactString),

  #[error("Command alias {0} already exist")]
  /// Command not found
  CommandAliasAlreadyExist(CompactString),

  #[error("Js error: {0}")]
  /// JavaScript error/exception
  JsErr(Box<JsError>),

  #[error("Failed to read module path {0}: {1}")]
  /// Failed to read script file when loading module
  LoadModuleSourceFailed(ModulePath, IoErr),

  #[error("Module path {0} not found")]
  /// Failed to read script file when loading module
  ModulePathNotFound(ModulePath),

  #[error("Failed to compile typescript: {0}")]
  /// Failed to compile typescript
  CompileTypeScriptFailed(String),
  // js }
}

/// [`Result`] with `T` if ok, [`TheError`] if error.
pub type TheResult<T> = Result<T, TheError>;

// thiserror }

#[macro_export]
macro_rules! bail {
  ($e:expr) => {
    return Err($e)
  };
}
