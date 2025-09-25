//! Vim editing mode.

use crate::prelude::*;

pub const NORMAL: &str = "normal";
pub const N: &str = "n";
pub const VISUAL: &str = "visual";
pub const V: &str = "v";
pub const SELECT: &str = "select";
pub const S: &str = "s";
pub const OPERATOR_PENDING: &str = "operator-pending";
pub const OP_PENDING: &str = "op-pending";
pub const O: &str = "o";
pub const INSERT: &str = "insert";
pub const I: &str = "i";
pub const COMMAND_LINE: &str = "command-line";
pub const CMDLINE: &str = "cmdline";
pub const C: &str = "c";
pub const COMMAND_LINE_SEARCH_FORWARD: &str = "command-line-search-forward";
pub const COMMAND_LINE_SEARCH_BACKWARD: &str = "command-line-search-backward";

#[derive(
  Debug,
  Copy,
  Clone,
  PartialEq,
  Eq,
  PartialOrd,
  Ord,
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
}

impl Mode {
  pub fn name(&self) -> &str {
    match self {
      Mode::Normal => NORMAL,
      Mode::Visual => VISUAL,
      Mode::Select => SELECT,
      Mode::OperatorPending => OP_PENDING,
      Mode::Insert => INSERT,
      Mode::CommandLineEx => CMDLINE,
      Mode::CommandLineSearchForward => COMMAND_LINE_SEARCH_FORWARD,
      Mode::CommandLineSearchBackward => COMMAND_LINE_SEARCH_BACKWARD,
    }
  }

  pub fn short_name(&self) -> &str {
    match self {
      Mode::Normal => N,
      Mode::Visual => V,
      Mode::Select => S,
      Mode::OperatorPending => O,
      Mode::Insert => I,
      Mode::CommandLineEx => C,
      Mode::CommandLineSearchForward => COMMAND_LINE_SEARCH_FORWARD,
      Mode::CommandLineSearchBackward => COMMAND_LINE_SEARCH_BACKWARD,
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
      Mode::CommandLineSearchForward,
      Mode::CommandLineSearchBackward,
    ]
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// The modes collection.
pub struct Modes {
  values: BTreeSet<Mode>,
}

impl Modes {
  /// Make a new modes collection with no mode inside.
  pub fn new() -> Self {
    Modes {
      values: BTreeSet::new(),
    }
  }

  /// Make a new modes collection with all of current modes, and with a new mode.
  pub fn with(&mut self, mode: Mode) -> &Self {
    self.values.insert(mode);
    self
  }

  /// Make a new modes collection with all of current modes, but without the specified mode.
  pub fn without(&mut self, mode: Mode) -> &Self {
    self.values.remove(&mode);
    self
  }

  /// Add/set all the specified modes.
  pub fn extend(&mut self, modes: Modes) -> &Self {
    self.values.extend(modes.values.iter());
    self
  }

  /// Add/set the specified mode.
  pub fn set(&mut self, mode: Mode) -> bool {
    self.values.insert(mode)
  }

  /// Remove/unset the specified mode.
  pub fn unset(&mut self, mode: Mode) -> bool {
    self.values.remove(&mode)
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
  pub fn iter(&self) -> std::collections::btree_set::Iter<'_, Mode> {
    self.values.iter()
  }
}

impl From<Mode> for Modes {
  /// Create a collection from a mode.
  fn from(mode: Mode) -> Self {
    let mut values = BTreeSet::new();
    values.insert(mode);
    Modes { values }
  }
}

impl From<Vec<Mode>> for Modes {
  /// Create a collection from a mode vector.
  fn from(modes: Vec<Mode>) -> Self {
    let mut values = BTreeSet::new();
    for m in modes {
      values.insert(m);
    }
    Modes { values }
  }
}
