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
  #[error("Buffer `{0}` doesn't have a filename.")]
  BufferNoName(BufferId),

  #[error("Buffer `{0}` not exist.")]
  BufferNotExist(BufferId),

  #[error("Failed to save buffer `{0}`({1}): {2}.")]
  SaveBufferFailed(BufferId, CompactString, IoErr),

  #[error("Undo commit `{0}` not exist on buffer `{1}`.")]
  UndoCommitNotExist(usize, BufferId),

  #[error("Failed to normalize path `{0}`: {1}.")]
  NormalizePathFailed(CompactString, IoErr),

  #[error("Command `{0}` not found.")]
  CommandNotFound(CompactString),

  #[error("Command `{0}` already exist.")]
  CommandAlreadyExist(CompactString),

  #[error("{0}")]
  JsError(Box<JsError>),

  #[error("Failed to load module `{0}`: {1}.")]
  LoadModuleFailed(ModulePath, IoErr),

  #[error("Module `{0}` not found.")]
  ModuleNotFound(ModulePath),

  #[error("Failed to compile typescript: {0}.")]
  CompileTypeScriptFailed(Cow<'static, str>),

  #[error("Not enough arguments specified.")]
  ArgumentsNotEnough,

  #[error("File `{0}` not found: {1}.")]
  FileNotFound(CompactString, IoErr),

  #[error("Failed to open file `{0}`: {1}.")]
  OpenFileFailed(CompactString, IoErr),

  #[error("Failed to read file `{0}`: {1}.")]
  ReadFileFailed(usize, IoErr),

  #[error("Failed to write file `{0}`: {1}.")]
  WriteFileFailed(usize, IoErr),

  #[error("Invalid data.")]
  DataInvalid,

  #[error("Value too large: `{0}`.")]
  ValueTooLarge(usize),

  #[error("Buffer too small: `{0}`.")]
  BufferTooSmall(usize),
}

/// [`Result`] with `T` if ok, [`TheErr`] if error.
pub type TheResult<T> = Result<T, TheErr>;

// thiserror }
