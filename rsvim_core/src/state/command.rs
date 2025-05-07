//! An abstract layer between terminal events and editor operations.
//!
//! This is the low-level commands between terminal keyboard/mouse events and the behaviors that we
//! want editor to do.

#[derive(Debug, Copy, Clone)]
/// Editor operations.
///
/// NOTE:
/// 1. The enum name is following the `Subject-Predicate-Object` English grammar.
/// 2. For those enums starts with `__`, they are private enums and should not used by users.
pub enum Command {
  /// Move cursor by offset `(x,y)` relatively, based on current cursor position.
  ///
  /// - The `x` is chars count, when negative it moves to left, when positive it moves to right.
  /// - The `y` is lines count, when negative it moves to up, when positive it moves to down.
  __CursorMoveBy((isize, isize)),

  /// Move cursor left by `n` chars relatively, based on current cursor position.
  CursorMoveLeftBy(usize),

  /// Move cursor right by `n` chars relatively, based on current cursor position.
  CursorMoveRightBy(usize),

  /// Move cursor up by `n` lines relatively, based on current cursor position.
  CursorMoveUpBy(usize),

  /// Move cursor down by `n` lines relatively, based on current cursor position.
  CursorMoveDownBy(usize),

  /// Similar to [`Command::__CursorMoveBy`], except it moves cursor to an absolute position based on
  /// current buffer.
  ///
  /// - The `x` is char index on the buffer.
  /// - The `y` is line index on the buffer.
  __CursorMoveTo((usize, usize)),

  /// Scroll buffer by offset `(x,y)` relatively, based on current window.
  ///
  /// - The `x` is columns (not chars) count, when negative it moves to left, when positive it
  ///   moves to right.
  /// - The `y` is lines count, when negative it moves to up, when positive it moves to down.
  __WindowScrollBy((isize, isize)),

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

  /// Similar to [`Command::__WindowScrollBy`], except it scrolls window to an absolute position
  /// based on current buffer.
  ///
  /// - The `x` is column (not char) index on the buffer.
  /// - The `y` is line index on the buffer.
  /// - The `(x,y)` is the top-left anchor of the window viewport.
  __WindowScrollTo((usize, usize)),

  /// Quit editor
  EditorQuit,
}
