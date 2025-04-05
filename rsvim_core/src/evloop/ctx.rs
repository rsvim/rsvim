//! Event loop context.

use crate::ui::canvas::{CursorStyle, CursorStyleFormatter};

#[derive(Copy, Clone)]
/// Saved TUI context.
///
/// When rsvim begins, it saves the current TUI context, enter the raw-mode and take full control
/// of the terminal. Then when rsvim exit, it restores the TUI context and give the terminal back
/// to user's control.
pub struct TuiContext {
  is_raw_mode: bool,
  cursor_blinking: bool,
  cursor_hidden: bool,
  cursor_style: CursorStyle,
}

impl TuiContext {
  pub fn is_raw_mode(&self) -> bool {
    self.is_raw_mode
  }

  pub fn set_is_raw_mode(&mut self, value: bool) {
    self.is_raw_mode = value;
  }

  pub fn cursor_blinking(&self) -> bool {
    self.cursor_blinking
  }

  pub fn set_cursor_blinking(&mut self, value: bool) {
    self.cursor_blinking = value;
  }

  pub fn cursor_hidden(&self) -> bool {
    self.cursor_hidden
  }

  pub fn set_cursor_hidden(&mut self, value: bool) {
    self.cursor_hidden = value;
  }

  pub fn cursor_style(&self) -> &CursorStyle {
    &self.cursor_style
  }

  pub fn set_cursor_style(&mut self, cursor_style: &CursorStyle) {
    self.cursor_style = *cursor_style;
  }
}

impl std::fmt::Debug for TuiContext {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let cursor_style_formatter = CursorStyleFormatter::from(self.cursor_style);
    f.debug_struct("TuiContext")
      .field("is_raw_mode", &self.is_raw_mode)
      .field("cursor_blinking", &self.cursor_blinking)
      .field("cursor_hidden", &self.cursor_hidden)
      .field("cursor_style", &cursor_style_formatter)
      .finish()
  }
}
