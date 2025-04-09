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
  CursorMoveUp(usize),
  CursorMoveDown(usize),
  CursorMoveLeft(usize),
  CursorMoveRight(usize),
  QuitEditor,
}
