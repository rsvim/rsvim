//! An abstract layer between terminal events and editor operations.
//!
//! The terminal keyboard/mouse events, for example, both h/j/k/l and left/up/down/right keys in
//! vim editor (normal mode) means cursor move left/up/down/right. But they have different
//! behaviors in different scenarios.
//!
//! And if one day, we support other editor modes such as emacs, vscode, etc, different key codes
//! will indicate different editor operations. Thus this layer will help to maintain different
//! editor modes and internal core logics.

#[derive(Debug, Copy, Clone)]
// Editor operation commands.
pub enum Command {
  /// Move cursor by offset `(x,y)`, relatively based on current cursor position.
  /// The `x` is chars count, `y` is lines count.
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

  /// Quit editor
  EditorQuit,
}
