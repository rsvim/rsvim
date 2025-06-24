//! The low-level editor operations.

use compact_str::CompactString;

pub mod cursor_move_ops;
pub mod cursor_ops;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// A set of low-level editor operations between terminal keyboard/mouse events and editor
/// operations.
///
/// NOTE: The enum name follows the `Subject-Predicate-Object` English grammar.
pub enum Operation {
  /// Move cursor by offset `(chars,lines)` relatively, based on current cursor position.
  ///
  /// - For `chars`, when negative it moves to left, when positive it moves to right.
  /// - For `lines`, when negative it moves to up, when positive it moves to down.
  CursorMoveBy((/* chars */ isize, /* lines */ isize)),

  /// Move cursor left by `n` chars relatively, based on current cursor position.
  CursorMoveLeftBy(usize),

  /// Move cursor right by `n` chars relatively, based on current cursor position.
  CursorMoveRightBy(usize),

  /// Move cursor up by `n` lines relatively, based on current cursor position.
  CursorMoveUpBy(usize),

  /// Move cursor down by `n` lines relatively, based on current cursor position.
  CursorMoveDownBy(usize),

  /// Similar to [`Operation::CursorMoveBy`], except it moves cursor to absolute position
  /// `(char_idx,line_idx)`, based on current buffer.
  CursorMoveTo((/* char_idx */ usize, /* lines_idx */ usize)),

  /// Scroll buffer by offset `(columns,lines)` relatively, based on current window.
  ///
  /// - For `columns` (not chars!), when negative it moves to left, when positive it moves to right.
  /// - For `lines`, when negative it moves to up, when positive it moves to down.
  WindowScrollBy((/* columns */ isize, /* lines */ isize)),

  /// Scroll buffer left by `n` columns relatively, based on current window viewport.
  ///
  /// NOTE: The offset `n` is columns, not chars.
  WindowScrollLeftBy(usize),

  /// Scroll buffer right by `n` columns relatively, based on current window viewport.
  ///
  /// NOTE: The offset `n` is columns, not chars.
  WindowScrollRightBy(usize),

  /// Scroll buffer up by `n` lines relatively, based on current window viewport.
  WindowScrollUpBy(usize),

  /// Scroll buffer down by `n` lines relatively, based on current window viewport.
  WindowScrollDownBy(usize),

  /// Similar to [`Operation::WindowScrollBy`], except it scrolls window to an absolute position
  /// `(column_idx,line_idx)` based on current buffer.
  WindowScrollTo((/* column_idx */ usize, /* line_idx */ usize)),

  /// Goto insert mode.
  GotoInsertMode,

  /// Goto normal mode.
  GotoNormalMode,

  /// Insert text at cursor.
  InsertAtCursor(/* text */ CompactString),

  /// Delete N-chars text, to the left of cursor if negative, to the right of cursor if positive.
  DeleteAtCursor(/* N-chars */ isize),

  /// Goto command-line ex mode.
  GotoCommandLineExMode,

  /// Goto command-line search forward mode.
  GotoCommandLineSearchForwardMode,

  /// Goto command-line search backward mode.
  GotoCommandLineSearchBackwardMode,

  /// Similar to [`CursorMoveBy`](Operation::CursorMoveBy), but for command-line ex mode.
  CursorMoveByCommandLineEx((/* chars */ isize, /* lines */ isize)),

  /// Similar to [`CursorMoveLeftBy`](Operation::CursorMoveLeftBy), but for command-line ex mode.
  CursorMoveLeftByCommandLineEx(usize),

  /// Similar to [`CursorMoveRightBy`](Operation::CursorMoveRightBy), but for command-line ex mode.
  CursorMoveRightByCommandLineEx(usize),

  /// Similar to [`CursorMoveUpBy`](Operation::CursorMoveUpBy), but for command-line ex mode.
  CursorMoveUpByCommandLineEx(usize),

  /// Similar to [`CursorMoveDownBy`](Operation::CursorMoveDownBy), but for command-line ex mode.
  CursorMoveDownByCommandLineEx(usize),

  /// Similar to [`CursorMoveTo`](Operation::CursorMoveTo), but for command-line ex mode.
  CursorMoveToCommandLineEx((/* char_idx */ usize, /* lines_idx */ usize)),

  /// Similar to [`InsertAtCursor`](Operation::InsertAtCursor), but for command-line ex mode.
  InsertAtCursorCommandLineEx(/* text */ CompactString),

  /// Similar to [`DeleteAtCursor`](Operation::DeleteAtCursor), but for command-line ex mode.
  DeleteAtCursorCommandLineEx(/* N-chars */ isize),

  /// Quit editor
  EditorQuit,
}
