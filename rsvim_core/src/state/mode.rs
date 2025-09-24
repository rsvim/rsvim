//! Vim editing mode.

use crate::prelude::*;

#[derive(
  Debug,
  Copy,
  Clone,
  PartialEq,
  Eq,
  Hash,
  strum_macros::Display,
  strum_macros::EnumString,
)]
/// Editing mode.
pub enum Mode {
  #[strum(serialize = "normal", serialize = "n")]
  /// Normal mode.
  Normal,

  #[strum(serialize = "visual", serialize = "v")]
  /// Visual mode.
  Visual,

  #[strum(serialize = "select", serialize = "s")]
  /// Select mode.
  Select,

  #[strum(
    serialize = "operator-pending",
    serialize = "op-pending",
    serialize = "o"
  )]
  /// Operator-pending mode.
  OperatorPending,

  #[strum(serialize = "insert", serialize = "i")]
  /// Insert mode.
  Insert,

  #[strum(serialize = "command-line", serialize = "cmdline", serialize = "c")]
  /// Command-line mode, ex-command variant.
  CommandLineEx,

  #[strum(serialize = "command-line-search-forward")]
  /// Command-line mode, search forward variant.
  CommandLineSearchForward,

  #[strum(serialize = "command-line-search-backward")]
  /// Command-line mode, search backward variant.
  CommandLineSearchBackward,

  #[strum(serialize = "terminal", serialize = "t")]
  /// Terminal mode.
  Terminal,
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
      Mode::CommandLineSearchForward,
      Mode::CommandLineSearchBackward,
      Mode::Terminal,
    ]
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// The modes collection.
pub struct Modes {
  values: FoldSet<Mode>,
}

impl Modes {
  /// Make a new modes collection with no mode inside.
  pub fn new() -> Self {
    Modes {
      values: FoldSet::new(),
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
  /// NOTE: The internal collection is [`FoldSet`] and the iterator is non-ordered.
  pub fn iter(&self) -> std::collections::hash_set::Iter<'_, Mode> {
    self.values.iter()
  }
}

impl From<Mode> for Modes {
  /// Create a collection from a mode.
  fn from(mode: Mode) -> Self {
    let mut values = FoldSet::new();
    values.insert(mode);
    Modes { values }
  }
}

impl From<Vec<Mode>> for Modes {
  /// Create a collection from a mode vector.
  fn from(modes: Vec<Mode>) -> Self {
    let mut values = FoldSet::new();
    for m in modes.iter() {
      values.insert(*m);
    }
    Modes { values }
  }
}
