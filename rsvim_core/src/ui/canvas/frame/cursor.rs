//! Cursor of canvas frame.

use crate::prelude::*;
use bitflags::bitflags;
use geo::point;
use std::fmt::Debug;

pub type CursorStyle = crossterm::cursor::SetCursorStyle;

bitflags! {
  #[derive(Copy, Clone, PartialEq, Eq)]
  struct Flags: u8 {
    const BLINKING = 1;
    const HIDDEN = 1 << 1;
  }
}

impl Debug for Flags {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Flags")
      .field("bits", &format!("{:b}", self.bits()))
      .finish()
  }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Terminal cursor.
pub struct Cursor {
  pos: U16Pos,
  style: CursorStyle,
  // blinking
  // hidden
  flags: Flags,
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
    if blinking {
      flags.insert(Flags::BLINKING);
    }
    if hidden {
      flags.insert(Flags::HIDDEN);
    }
    Cursor { pos, style, flags }
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
    if value {
      self.flags.insert(Flags::BLINKING);
    } else {
      self.flags.remove(Flags::BLINKING);
    }
  }

  /// Get hidden.
  pub fn hidden(&self) -> bool {
    self.flags.contains(Flags::HIDDEN)
  }

  /// Set hidden.
  pub fn set_hidden(&mut self, value: bool) {
    if value {
      self.flags.insert(Flags::HIDDEN);
    } else {
      self.flags.remove(Flags::HIDDEN);
    }
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
    // blinking=false
    // hidden=false
    let flags = Flags::empty();
    Cursor {
      pos: point! {x:0_u16, y:0_u16},
      style: CursorStyle::SteadyBlock,
      flags,
    }
  }
}
