//! An abstract layer between terminal events and editor operations.
//!
//! The terminal keyboard/mouse events, for example, both h/j/k/l and left/up/down/right keys in
//! vim editor (normal mode) means cursor move left/up/down/right. But they have different
//! behaviors in different scenarios.

#[derive(Debug, Copy, Clone)]
/// Editor operation commands.
pub enum Command {
  /// Move cursor by offset `(x,y)`, relatively based on current cursor position.
  /// The `x` is chars count, when negative it moves to left, when positive it moves to right. The
  /// `y` is lines count, when negative it moves to up, when positive it moves to down.
  ///
  /// NOTE: When the cursor moves to a position out of current window, it scrolls the window's
  /// viewport.
  CursorMoveBy((isize, isize)),

  /// Similar to [`Command::CursorMoveBy`], but it moves cursor to an absolute position based on
  /// current buffer.
  /// The `x` is char index, `y` is line index.
  ///
  /// NOTE: When the cursor moves to a position out of current window, it scrolls the window's
  /// viewport.
  CursorMoveTo((usize, usize)),

  /// Scroll buffer by offset `(x,y)`, relatively based on current window.
  /// The `x` is columns count, when negative it moves to left, when positive it moves to right.
  /// The `y` is lines count, when negative it moves to up, when positive it moves to down.
  ///
  /// NOTE: When the cursor is at the window border and scrolling will move the cursor out of
  /// window, the cursor position will be drag to keep it still inside the window.
  WindowSrollBy((isize, isize)),

  /// Similar to [`Command::WindowSrollBy`], but it srolls window to an absolute position based on
  /// current buffer.
  /// The `x` is columns index, `y` is line index. The anchor is at the top-left of window
  /// viewport.
  ///
  /// NOTE: When the cursor is at the window border and scrolling will move the cursor out of
  /// window, the cursor position will be drag to keep it still inside the window.
  WindowSrollTo((isize, isize)),

  /// Quit editor
  EditorQuit,
}
