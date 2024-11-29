//! Results and errors.

use std::path::PathBuf;
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

/// [`std::result::Result`] with `T` if ok, [`TheJsRuntimeErr`] if error.
pub type JsRuntimeResult<T> = std::result::Result<T, JsRuntimeErr>;

// Js Runtime }

// Buffer {

// #[derive(Debug, ThisError)]
// /// Vim buffer error code implemented by [`thiserror::Error`].
// pub enum BufferErr {
//   #[error("File path already exists: {0}")]
//   FilePathAlreadyExists(PathBuf),
//
//   #[error("Io error: {0}")]
//   IoErr(IoErr),
// }
//
// /// [`std::result::Result`] with `T` if ok, [`TheBufferErr`] if error.
// pub type BufferResult<T> = std::result::Result<T, BufferErr>;

// Buffer }
