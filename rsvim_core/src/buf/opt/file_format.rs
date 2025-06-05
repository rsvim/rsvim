//! The "file-format" option for Vim buffer.

use std::fmt::Display;
use std::string::ToString;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum FileFormatOption {
  /// CRLF (<CR><NL>)
  Dos,

  /// LF (<NL>)
  Unix,

  /// LF (<NL>)
  ///
  /// NOTE: CR (<CR>) is deprecated in macos.
  Mac,

  /// CR (<CR>)
  ///
  /// NOTE: CR (<CR>) is deprecated in macos.
  ClassicMac,
}

impl Display for FileFormatOption {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      FileFormatOption::Dos => write!(f, "dos"),
      FileFormatOption::Unix => write!(f, "unix"),
      FileFormatOption::Mac => write!(f, "mac"),
      FileFormatOption::ClassicMac => write!(f, "classic_mac"),
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
      "classic_mac" => Ok(FileFormatOption::ClassicMac),
      _ => Err("Unknown FileFormat value".to_string()),
    }
  }
}

impl From<EndOfLineOption> for FileFormatOption {
  fn from(value: EndOfLineOption) -> Self {
    match value {
      EndOfLineOption::CRLF => FileFormatOption::Dos,
      EndOfLineOption::LF => FileFormatOption::Unix,
      EndOfLineOption::CR => FileFormatOption::ClassicMac,
    }
  }
}

impl Into<EndOfLineOption> for FileFormatOption {
  fn into(self) -> EndOfLineOption {
    match self {
      FileFormatOption::Dos => EndOfLineOption::CRLF,
      FileFormatOption::Unix => EndOfLineOption::LF,
      FileFormatOption::Mac => EndOfLineOption::LF,
      FileFormatOption::ClassicMac => EndOfLineOption::CR,
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
    match self {
      EndOfLineOption::CRLF => write!(f, "CRLF"),
      EndOfLineOption::LF => write!(f, "LF"),
      EndOfLineOption::CR => write!(f, "CR"),
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display1() {
    let actual1 = format!("{}", FileFormatOption::Dos);
    assert_eq!(actual1, "dos");
  }
}
