//! Cursor of canvas frame.

use crate::prelude::*;

pub type CursorStyle = crossterm::cursor::SetCursorStyle;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Terminal cursor.
pub struct Cursor {
  pos: U16Pos,
  blinking: bool,
  hidden: bool,
  style: CursorStyle,
}

impl Cursor {
  /// Make new terminal cursor.
  pub fn new(
    pos: U16Pos,
    blinking: bool,
    hidden: bool,
    style: CursorStyle,
  ) -> Self {
    Self {
      pos,
      blinking,
      hidden,
      style,
    }
  }

  /// Get position.
  pub fn pos(&self) -> &U16Pos {
    &self.pos
  }

  /// Set position.
  pub fn set_pos(&mut self, pos: U16Pos) {
    self.pos = pos;
  }

  /// Get blinking.
  pub fn blinking(&self) -> bool {
    self.blinking
  }

  /// Set blinking.
  pub fn set_blinking(&mut self, value: bool) {
    self.blinking = value;
  }

  /// Get hidden.
  pub fn hidden(&self) -> bool {
    self.hidden
  }

  /// Set hidden.
  pub fn set_hidden(&mut self, value: bool) {
    self.hidden = value;
  }

  /// Get style.
  pub fn style(&self) -> CursorStyle {
    self.style
  }

  /// Set style.
  pub fn set_style(&mut self, style: CursorStyle) {
    self.style = style;
  }
}

impl Default for Cursor {
  /// Make default cursor.
  fn default() -> Self {
    use crate::ui::widget::cursor::BLINKING;
    use crate::ui::widget::cursor::CURSOR_STYLE;
    use crate::ui::widget::cursor::HIDDEN;

    Cursor {
      pos: point!(0_u16, 0_u16),
      blinking: BLINKING,
      hidden: HIDDEN,
      style: CURSOR_STYLE,
    }
  }
}
