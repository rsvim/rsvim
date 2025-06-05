//! The low-level editor operations.

pub mod cursor_ops;

use compact_str::CompactString;

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

  /// Insert line-wise text at cursor.
  InsertLineWiseTextAtCursor(CompactString),

  /// Delete line-wise N-chars text, to the left of the cursor.
  DeleteLineWiseTextToLeftAtCursor(usize),

  /// Delete line-wise N-chars text, to the right of the cursor.
  DeleteLineWiseTextToRightAtCursor(usize),

  /// Quit editor
  EditorQuit,
}
