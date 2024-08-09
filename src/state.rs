//! The global editing state of the editor.

/// Editing mode enums.
pub enum EditingMode {
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

/// The global editing state.
pub struct State {}
