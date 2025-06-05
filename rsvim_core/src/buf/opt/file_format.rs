//! The "file-format" option for Vim buffer.

use std::fmt::Display;
use std::string::ToString;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum FileFormatOption {
  /// CRLF (<CR><NL>)
  Dos,
  /// LF (<NL>)
  Unix,
  /// CR (<CR>)
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display1() {
    let actual1 = format!("{}", FileFormatOption::Dos);
    assert_eq!(actual1, "dos");
  }
}
