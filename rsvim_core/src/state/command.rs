//! An abstract layer between terminal events and editor operations.
//!
//! This is the low-level commands between terminal keyboard/mouse events and the behaviors that we
//! want editor to do.

#[derive(Debug, Copy, Clone)]
/// Editor operations.
///
/// NOTE: The enum name is following the `Subject-Predicate-Object` English grammar.
pub enum Command {
  /// Move cursor by offset `(x,y)` relatively, based on current cursor position.
  ///
  /// - The `x` is chars count, when negative it moves to left, when positive it moves to right.
  /// - The `y` is lines count, when negative it moves to up, when positive it moves to down.
  ///
  /// NOTE: When the cursor moves to a position not showing in the window, it tries to scroll the
  /// window.
  CursorMoveBy((isize, isize)),

  /// Move cursor left by offset `n` relatively, based on current cursor position.
  ///
  /// Same to [`Command::CursorMoveBy((-n,0))`](Command::CursorMoveBy).
  CursorMoveLeftBy(usize),

  /// Move cursor right by offset `n` relatively, based on current cursor position.
  ///
  /// Same to [`Command::CursorMoveBy((n,0))`](Command::CursorMoveBy).
  CursorMoveRightBy(usize),

  /// Move cursor up by offset `n` relatively, based on current cursor position.
  ///
  /// Same to [`Command::CursorMoveBy((0,-n))`](Command::CursorMoveBy).
  CursorMoveUpBy(usize),

  /// Move cursor down by offset `n` relatively, based on current cursor position.
  ///
  /// Same to [`Command::CursorMoveBy((0,n))`](Command::CursorMoveBy).
  CursorMoveDownBy(usize),

  /// Similar to [`Command::CursorMoveBy`], except it moves cursor to an absolute position based on
  /// current buffer.
  ///
  /// - The `x` is char index on the buffer.
  /// - The `y` is line index on the buffer.
  ///
  /// NOTE: When the cursor moves to a position not showing in the window, it tries to scroll the
  /// window.
  CursorMoveTo((usize, usize)),

  /// Scroll buffer by offset `(x,y)` relatively, based on current window.
  ///
  /// - The `x` is columns (not chars) count, when negative it moves to left, when positive it
  ///   moves to right.
  /// - The `y` is lines count, when negative it moves to up, when positive it moves to down.
  ///
  /// NOTE: When scrolling will move the cursor out of window, the cursor position will be drag to
  /// keep it still inside the window.
  WindowScrollBy((isize, isize)),

  /// Similar to [`Command::WindowScrollBy`], except it scrolls window to an absolute position
  /// based on current buffer.
  ///
  /// - The `x` is column (not char) index on the buffer.
  /// - The `y` is line index on the buffer.
  /// - The `(x,y)` is the top-left anchor of the window viewport.
  ///
  /// NOTE: When scrolling will move the cursor out of window, the cursor position will be drag to
  /// keep it still inside the window.
  WindowScrollTo((usize, usize)),

  /// Quit editor
  EditorQuit,
}
