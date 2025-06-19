//! Vim editing mode.

use crate::prelude::*;

use std::fmt::Display;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// Editing mode.
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
  /// Command-line mode, ex-command variant.
  CommandLineEx,
  /// Command-line mode, search forward variant.
  CommandLineSearchForward,
  /// Command-line mode, search backward variant.
  CommandLineSearchBackward,
  /// Terminal mode.
  Terminal,
}

impl Display for Mode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Mode::Normal => write!(f, "Normal"),
      Mode::Visual => write!(f, "Visual"),
      Mode::Select => write!(f, "Select"),
      Mode::OperatorPending => write!(f, "Operator-pending"),
      Mode::Insert => write!(f, "Insert"),
      Mode::CommandLineEx => write!(f, "Command-line (ex)"),
      Mode::CommandLineSearchForward => write!(f, "Command-line (search forward)"),
      Mode::CommandLineSearchBackward => write!(f, "Command-line (search backward)"),
      Mode::Terminal => write!(f, "Terminal"),
    }
  }
}

impl Mode {
  /// Get all modes.
  pub fn all() -> Vec<Mode> {
    vec![
      Mode::Normal,
      Mode::Visual,
      Mode::Select,
      Mode::OperatorPending,
      Mode::Insert,
      Mode::CommandLineEx,
      Mode::Terminal,
    ]
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// The modes collection.
pub struct Modes {
  values: HashSet<Mode>,
}

impl Modes {
  /// Make a new modes collection with no mode inside.
  pub fn new() -> Self {
    Modes {
      values: HashSet::new(),
    }
  }

  /// Make a new modes collection with all of current modes, and with a new mode.
  pub fn with(&self, mode: Mode) -> Self {
    let mut values = self.values.clone();
    values.insert(mode);
    Modes { values }
  }

  /// Make a new modes collection with all of current modes, but without the specified mode.
  pub fn without(&self, mode: Mode) -> Self {
    let mut values = self.values.clone();
    values.remove(&mode);
    Modes { values }
  }

  /// Add/set the specified mode.
  pub fn set(&mut self, mode: Mode) -> bool {
    self.values.insert(mode)
  }

  /// Remove/unset the specified mode.
  pub fn unset(&mut self, mode: Mode) -> bool {
    self.values.remove(&mode)
  }

  /// Add/set all the specified modes.
  pub fn extend(&mut self, modes: Modes) {
    self.values.extend(modes.values.iter())
  }

  /// Whether current collection is empty.
  pub fn is_empty(&self) -> bool {
    self.values.is_empty()
  }

  /// Current collection's mode count.
  pub fn len(&self) -> usize {
    self.values.len()
  }

  /// Whether current collection contains a mode.
  pub fn contains(&self, mode: &Mode) -> bool {
    self.values.contains(mode)
  }

  /// Get the iterator of current collection.
  ///
  /// NOTE: The internal collection is [`HashSet`] and the iterator is non-ordered.
  pub fn iter(&self) -> std::collections::hash_set::Iter<'_, Mode> {
    self.values.iter()
  }
}

impl From<Mode> for Modes {
  /// Create a collection from a mode.
  fn from(mode: Mode) -> Self {
    let mut values = HashSet::new();
    values.insert(mode);
    Modes { values }
  }
}

impl From<Vec<Mode>> for Modes {
  /// Create a collection from a mode vector.
  fn from(modes: Vec<Mode>) -> Self {
    let mut values = HashSet::new();
    for m in modes.iter() {
      values.insert(*m);
    }
    Modes { values }
  }
}
