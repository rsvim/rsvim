//! Editing mode.

use std::collections::HashSet;
use std::str::FromStr;
use std::string::ToString;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
  /// Convert enum to `String`.
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

  /// Parse `str` to enum.
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

  /// Parse `str` to enum.
  fn try_from(s: &str) -> Result<Self, Self::Error> {
    FromStr::from_str(s)
  }
}

impl TryFrom<String> for Mode {
  type Error = &'static str;

  /// Parse `String` to enum.
  fn try_from(s: String) -> Result<Self, Self::Error> {
    TryFrom::try_from(s.as_str())
  }
}

impl Mode {
  pub fn all() -> Vec<Mode> {
    vec![
      Mode::Normal,
      Mode::Visual,
      Mode::Select,
      Mode::OperatorPending,
      Mode::Insert,
      Mode::CommandLine,
      Mode::Terminal,
    ]
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Modes {
  values: HashSet<Mode>,
}

impl Modes {
  pub fn new() -> Self {
    Modes {
      values: HashSet::new(),
    }
  }

  pub fn contains(&self, value: &Mode) -> bool {
    self.values.contains(value)
  }

  pub fn insert(&mut self, value: Mode) -> bool {
    self.values.insert(value)
  }

  pub fn remove(&mut self, value: &Mode) -> bool {
    self.values.remove(&value)
  }

  pub fn all(&self) -> &HashSet<Mode> {
    &self.values
  }
}
