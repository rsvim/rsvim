//! The global editing state of the editor.

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

impl From<&str> for Mode {
  fn from(name: &str) -> Self {
    match name {
      "Normal" => Mode::Normal,
      "Visual" => Mode::Visual,
      "Select" => Mode::Visual,
      "Operator-pending" => Mode::OperatorPending,
      "Insert" => Mode::Insert,
      "Command-line" => Mode::CommandLine,
      "Terminal" => Mode::Terminal,
      _ => unreachable!("Invalid Mode name"),
    }
  }
}

/// The global editing state.
pub struct State {}
