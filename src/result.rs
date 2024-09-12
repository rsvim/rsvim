//! Results and errors.

pub enum ErrorCode {
  Dummy,
  Message(String),
}

pub type VoidResult = Result<(), ErrorCode>;
pub type BoolResult = Result<bool, ErrorCode>;
pub type StrResult = Result<String, ErrorCode>;

pub type VoidIoResult = std::io::Result<()>;
pub type BoolIoResult = std::io::Result<bool>;
pub type StrIoResult = std::io::Result<String>;
pub type IoResult<T> = std::io::Result<T>;
