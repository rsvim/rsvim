//! Errors and results.

use crate::buf::BufferId;
use crate::js::err::JsError;
use crate::js::module::ModulePath;
use compact_str::CompactString;

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
  #[error("buffer `{0}` doesn't have a filename")]
  /// Buffer doesn't have a filename.
  BufferHaveNoFileName(BufferId),

  #[error("buffer `{0}` not exist")]
  /// Buffer not found
  BufferNotExist(BufferId),

  #[error("Failed to save buffer `{0}`: {1}")]
  /// Buffer failed to write file
  SaveBufferFailed(BufferId, IoErr),

  #[error("Failed to open file `{0}` for write: {1}")]
  /// Failed to open file for write
  OpenFileForWriteFailed(String, IoErr),
  // buf }

  // js {
  #[error("Command `{0}` not found")]
  /// Command not found
  CommandNotFound(CompactString),

  #[error("Command name `{0}` already exist")]
  /// Command not found
  CommandNameAlreadyExist(CompactString),

  #[error("Command alias `{0}` already exist")]
  /// Command not found
  CommandAliasAlreadyExist(CompactString),

  #[error("Js error: {0}")]
  /// JavaScript error/exception
  JsError(Box<JsError>),

  #[error("Failed to load module source `{0}`: {1}")]
  /// Failed to read script file when loading module
  LoadModuleFailed(ModulePath, IoErr),

  #[error("Module path `{0}` not found")]
  /// Failed to read script file when loading module
  ModulePathNotFound(ModulePath),

  #[error("Failed to compile typescript `{0}`")]
  /// Failed to compile typescript
  CompileTypeScriptFailed(String),
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
