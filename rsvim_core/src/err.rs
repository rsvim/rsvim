//! Errors and results.

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
#[derive(Debug, Clone, thiserror::Error)]
pub enum TheError {
  #[error("Buffer {0} doesn't have a filename")]
  BufferHaveNoFileName(crate::buf::BufferId),
}

/// [`Result`] with `T` if ok, [`TheError`] if error.
pub type TheResult<T, TheError> = Result<T, TheError>;

// thiserror }
