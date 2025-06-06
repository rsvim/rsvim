//! The "file-format" option for Vim buffer.

use std::fmt::Display;
use std::string::ToString;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum FileFormatOption {
  /// CRLF (`<CR><NL>`)
  Dos,

  /// LF (`<NL>`)
  Unix,

  /// CR (`<CR>`)
  ///
  /// NOTE: This is deprecated and should never used in today's computer.
  Mac,
}

impl Display for FileFormatOption {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      FileFormatOption::Dos => write!(f, "dos"),
      FileFormatOption::Unix => write!(f, "unix"),
      FileFormatOption::Mac => write!(f, "mac"),
    }
  }
}

impl TryFrom<&str> for FileFormatOption {
  type Error = String;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    let lower_value = value.to_lowercase();
    match lower_value.as_str() {
      "dos" => Ok(FileFormatOption::Dos),
      "unix" => Ok(FileFormatOption::Unix),
      "mac" => Ok(FileFormatOption::Mac),
      _ => Err("Unknown FileFormat value".to_string()),
    }
  }
}

impl From<EndOfLineOption> for FileFormatOption {
  fn from(value: EndOfLineOption) -> Self {
    match value {
      EndOfLineOption::CRLF => FileFormatOption::Dos,
      EndOfLineOption::LF => FileFormatOption::Unix,
      EndOfLineOption::CR => FileFormatOption::Mac,
    }
  }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum EndOfLineOption {
  /// Windows
  CRLF,

  /// Unix
  LF,

  /// Classic Mac, not used today
  CR,
}

impl Display for EndOfLineOption {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use crate::defaults::ascii::end_of_line as eol;

    match self {
      EndOfLineOption::CRLF => write!(f, "{}", eol::CRLF),
      EndOfLineOption::LF => write!(f, "{}", eol::LF),
      EndOfLineOption::CR => write!(f, "{}", eol::CR),
    }
  }
}

impl TryFrom<&str> for EndOfLineOption {
  type Error = String;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    let lower_value = value.to_lowercase();
    match lower_value.as_str() {
      "CRLF" => Ok(EndOfLineOption::CRLF),
      "LF" => Ok(EndOfLineOption::LF),
      "CR" => Ok(EndOfLineOption::CR),
      _ => Err("Unknown EndOfLine value".to_string()),
    }
  }
}

impl From<FileFormatOption> for EndOfLineOption {
  fn from(value: FileFormatOption) -> Self {
    match value {
      FileFormatOption::Dos => EndOfLineOption::CRLF,
      FileFormatOption::Unix => EndOfLineOption::LF,
      FileFormatOption::Mac => EndOfLineOption::CR,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display1() {
    let actual1 = format!("{}", FileFormatOption::Dos);
    assert_eq!(actual1, "dos");
  }
}
