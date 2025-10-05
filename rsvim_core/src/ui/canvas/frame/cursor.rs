//! Cursor of canvas frame.

use crate::flags_impl;
use crate::prelude::*;
use geo::point;

pub type CursorStyle = crossterm::cursor::SetCursorStyle;

flags_impl!(Flags, u8, BLINKING, 0b0000_0001, HIDDEN, 0b0000_0010);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Terminal cursor.
pub struct Cursor {
  pos: U16Pos,
  // blinking=false
  // hidden=false
  flags: Flags,
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
    let mut flags = Flags::empty();
    flags.set(Flags::BLINKING, blinking);
    flags.set(Flags::HIDDEN, hidden);
    Cursor { pos, flags, style }
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
    self.flags.contains(Flags::BLINKING)
  }

  /// Set blinking.
  pub fn set_blinking(&mut self, value: bool) {
    self.flags.set(Flags::BLINKING, value);
  }

  /// Get hidden.
  pub fn hidden(&self) -> bool {
    self.flags.contains(Flags::HIDDEN)
  }

  /// Set hidden.
  pub fn set_hidden(&mut self, value: bool) {
    self.flags.set(Flags::HIDDEN, value);
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
    Cursor {
      pos: point! {x:0_u16, y:0_u16},
      blinking: false,
      hidden: false,
      style: CursorStyle::SteadyBlock,
    }
  }
}
