//! Editing mode.

use std::str::FromStr;
use std::string::ToString;

/// Editing mode enums.
pub enum Mode {
  /// Normal mode.
  Normal,
  /// Visual mode.
  Visual,
  /// Select mode.
  Select,
  /// Operator-pending mode.
  OperatorPending,
  /// Insert mode.
  Insert,
  /// Command-line mode.
  CommandLine,
  /// Terminal mode.
  Terminal,
}

impl ToString for Mode {
  fn to_string(&self) -> String {
    match self {
      Mode::Normal => "Normal".to_string(),
      Mode::Visual => "Visual".to_string(),
      Mode::Select => "Select".to_string(),
      Mode::OperatorPending => "Operator-pending".to_string(),
      Mode::Insert => "Insert".to_string(),
      Mode::CommandLine => "Command-line".to_string(),
      Mode::Terminal => "Terminal".to_string(),
    }
  }
}

impl FromStr for Mode {
  type Err = &'static str;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "Normal" => Ok(Mode::Normal),
      "Visual" => Ok(Mode::Visual),
      "Select" => Ok(Mode::Visual),
      "Operator-pending" => Ok(Mode::OperatorPending),
      "Insert" => Ok(Mode::Insert),
      "Command-line" => Ok(Mode::CommandLine),
      "Terminal" => Ok(Mode::Terminal),
      _ => Err("Invalid Mode name"),
    }
  }
}

impl TryFrom<&str> for Mode {
  type Error = &'static str;

  fn try_from(s: &str) -> Result<Self, Self::Error> {
    FromStr::from_str(s)
  }
}
