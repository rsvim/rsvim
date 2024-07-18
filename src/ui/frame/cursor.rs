//! Cursor on a terminal frame.

use crate::cart::U16Pos;
use geo::point;
use std::{cmp, fmt, hash};

pub type CursorStyle = crossterm::cursor::SetCursorStyle;

#[derive(Copy, Clone)]
/// Terminal cursor.
/// Note: This is the real terminal cursor of the device, not a virtual one in multiple cursors.
pub struct Cursor {
  pub pos: U16Pos,
  pub blinking: bool,
  pub hidden: bool,
  pub style: CursorStyle,
  pub dirty: bool,
}

struct CursorStyleFormatter {
  style: CursorStyle,
}

impl From<CursorStyle> for CursorStyleFormatter {
  fn from(style: CursorStyle) -> Self {
    CursorStyleFormatter { style }
  }
}

impl fmt::Debug for CursorStyleFormatter {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
    write!(f, "{}", self.style)
  }
}

impl Cursor {
  pub fn new(pos: U16Pos, blinking: bool, hidden: bool, style: CursorStyle) -> Self {
    Cursor {
      pos,
      blinking,
      hidden,
      style,
      dirty: true,
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
      dirty: true,
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
