//! Cursor on a terminal frame.

use crate::cart::U16Pos;
use geo::point;
use std::{cmp, fmt, hash};

pub type CursorStyle = crossterm::cursor::SetCursorStyle;

/// Whether two `CursorStyle` equals.
pub fn cursor_style_eq(a: CursorStyle, b: CursorStyle) -> bool {
  match a {
    crossterm::cursor::SetCursorStyle::DefaultUserShape => {
      matches!(b, crossterm::cursor::SetCursorStyle::DefaultUserShape)
    }
    crossterm::cursor::SetCursorStyle::BlinkingBlock => {
      matches!(b, crossterm::cursor::SetCursorStyle::BlinkingBlock)
    }
    crossterm::cursor::SetCursorStyle::SteadyBlock => {
      matches!(b, crossterm::cursor::SetCursorStyle::SteadyBlock)
    }
    crossterm::cursor::SetCursorStyle::BlinkingUnderScore => {
      matches!(b, crossterm::cursor::SetCursorStyle::BlinkingUnderScore)
    }
    crossterm::cursor::SetCursorStyle::SteadyUnderScore => {
      matches!(b, crossterm::cursor::SetCursorStyle::SteadyUnderScore)
    }
    crossterm::cursor::SetCursorStyle::BlinkingBar => {
      matches!(b, crossterm::cursor::SetCursorStyle::BlinkingBar)
    }
    crossterm::cursor::SetCursorStyle::SteadyBar => {
      matches!(b, crossterm::cursor::SetCursorStyle::SteadyBar)
    }
  }
}

#[derive(Copy, Clone)]
/// Terminal cursor.
/// Note: This is the real terminal cursor of the device, not a virtual one in multiple cursors.
pub struct Cursor {
  pub pos: U16Pos,
  pub blinking: bool,
  pub hidden: bool,
  pub style: CursorStyle,
}

pub struct CursorStyleFormatter {
  value: CursorStyle,
}

impl From<CursorStyle> for CursorStyleFormatter {
  fn from(style: CursorStyle) -> Self {
    CursorStyleFormatter { value: style }
  }
}

impl fmt::Debug for CursorStyleFormatter {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
    write!(f, "{}", self.value)
  }
}

impl Cursor {
  pub fn new(pos: U16Pos, blinking: bool, hidden: bool, style: CursorStyle) -> Self {
    Cursor {
      pos,
      blinking,
      hidden,
      style,
    }
  }
}

impl Default for Cursor {
  fn default() -> Self {
    Cursor {
      pos: point! {x:0_u16, y:0_u16},
      blinking: false,
      hidden: false,
      style: CursorStyle::DefaultUserShape,
    }
  }
}

impl fmt::Debug for Cursor {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
    let style_formatter = CursorStyleFormatter::from(self.style);
    f.debug_struct("Cursor")
      .field("pos", &self.pos)
      .field("blinking", &self.blinking)
      .field("hidden", &self.hidden)
      .field("style", &style_formatter)
      .finish()
  }
}

impl cmp::PartialEq for Cursor {
  /// Whether equals to other.
  fn eq(&self, other: &Self) -> bool {
    self.pos == other.pos
  }
}

impl cmp::Eq for Cursor {}

impl hash::Hash for Cursor {
  fn hash<H: hash::Hasher>(&self, state: &mut H) {
    self.pos.hash(state);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn cursor_style_equals1() {
    assert!(cursor_style_eq(
      CursorStyle::DefaultUserShape,
      CursorStyle::DefaultUserShape
    ));
    let cs1 = CursorStyle::DefaultUserShape;
    let cs2 = CursorStyle::BlinkingBlock;
    let cs3 = CursorStyle::SteadyBlock;
    let cs4 = CursorStyle::BlinkingUnderScore;
    let cs5 = CursorStyle::SteadyUnderScore;
    let cs6 = CursorStyle::BlinkingBar;
    let cs7 = CursorStyle::SteadyBar;
    let cs8 = CursorStyle::DefaultUserShape;
    assert!(!cursor_style_eq(cs1, cs2));
    assert!(!cursor_style_eq(cs1, cs3));
    assert!(!cursor_style_eq(cs1, cs4));
    assert!(!cursor_style_eq(cs1, cs5));
    assert!(!cursor_style_eq(cs1, cs6));
    assert!(!cursor_style_eq(cs1, cs7));
    assert!(cursor_style_eq(cs1, cs8));
  }
}
