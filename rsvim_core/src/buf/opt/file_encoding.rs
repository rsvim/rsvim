//! The "file-encoding" option for Vim buffer.

use std::fmt::Display;
use std::string::ToString;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum FileEncodingOption {
  Utf8,
  // Utf16,
  // Utf32,
}

impl Display for FileEncodingOption {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      FileEncodingOption::Utf8 => write!(f, "utf-8"),
      // FileEncoding::Utf16 => "utf-16".to_string(),
      // FileEncoding::Utf32 => "utf-32".to_string(),
    }
  }
}

impl TryFrom<&str> for FileEncodingOption {
  type Error = String;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    let lower_value = value.to_lowercase();
    match lower_value.as_str() {
      "utf-8" | "utf8" => Ok(FileEncodingOption::Utf8),
      // "utf-16" | "utf16" => Ok(FileEncoding::Utf16),
      // "utf-32" | "utf32" => Ok(FileEncoding::Utf32),
      _ => Err("Unknown FileEncoding value".to_string()),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display1() {
    let actual1 = format!("{}", FileEncodingOption::Utf8);
    assert_eq!(actual1, "utf-8");
  }
}
