//! Cursor on a terminal frame.

use crate::cart::U16Pos;
use geo::point;
use std::{cmp, fmt, hash};

pub type CursorStyle = crossterm::cursor::SetCursorStyle;

/// Whether two `CursorStyle` equals.
pub fn cursor_style_eq(a: CursorStyle, b: CursorStyle) -> bool {
  match a {
    crossterm::cursor::SetCursorStyle::DefaultUserShape => match b {
      crossterm::cursor::SetCursorStyle::DefaultUserShape => true,
      _ => false,
    },
    crossterm::cursor::SetCursorStyle::BlinkingBlock => match b {
      crossterm::cursor::SetCursorStyle::BlinkingBlock => true,
      _ => false,
    },
    crossterm::cursor::SetCursorStyle::SteadyBlock => match b {
      crossterm::cursor::SetCursorStyle::SteadyBlock => true,
      _ => false,
    },
    crossterm::cursor::SetCursorStyle::BlinkingUnderScore => match b {
      crossterm::cursor::SetCursorStyle::BlinkingUnderScore => true,
      _ => false,
    },
    crossterm::cursor::SetCursorStyle::SteadyUnderScore => match b {
      crossterm::cursor::SetCursorStyle::SteadyUnderScore => true,
      _ => false,
    },
    crossterm::cursor::SetCursorStyle::BlinkingBar => match b {
      crossterm::cursor::SetCursorStyle::BlinkingBar => true,
      _ => false,
    },
    crossterm::cursor::SetCursorStyle::SteadyBar => match b {
      crossterm::cursor::SetCursorStyle::SteadyBar => true,
      _ => false,
    },
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
