//! Cursor on a terminal frame.

use crate::cart::U16Pos;
use geo::point;
use std::cmp::{Eq, PartialEq};
use std::fmt;

pub type CursorStyle = crossterm::cursor::SetCursorStyle;

/// Whether two `CursorStyle` equals.
pub fn cursor_style_eq(a: &CursorStyle, b: &CursorStyle) -> bool {
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
pub struct Cursor {
  pos: U16Pos,
  blinking: bool,
  hidden: bool,
  style: CursorStyle,
}

/// The [`CursorStyle`] formatter that helps implement the `Debug`/`Display` trait.
///
/// Note: The [`SetCursorStyle`](crossterm::cursor::SetCursorStyle) doesn't implement the
/// `Debug`/`Display` traitn before 0.28.1.
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
  /// Make new terminal cursor.
  pub fn new(pos: U16Pos, blinking: bool, hidden: bool, style: CursorStyle) -> Self {
    Cursor {
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
  pub fn set_blinking(&mut self, blinking: bool) {
    self.blinking = blinking;
  }

  /// Get hidden.
  pub fn hidden(&self) -> bool {
    self.hidden
  }

  /// Set hidden.
  pub fn set_hidden(&mut self, hidden: bool) {
    self.hidden = hidden;
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
  /// Make default terminal cursor.
  fn default() -> Self {
    Cursor {
      pos: point! {x:0_u16, y:0_u16},
      blinking: true,
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

impl PartialEq for Cursor {
  /// Whether two cursors equals to each other.
  fn eq(&self, other: &Self) -> bool {
    self.pos == other.pos
      && self.blinking == other.blinking
      && self.hidden == other.hidden
      && cursor_style_eq(&self.style, &other.style)
  }
}

impl Eq for Cursor {}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default1() {
    let c = Cursor::default();
    assert!(c.blinking);
    assert!(!c.hidden);
    assert!(cursor_style_eq(&c.style(), &CursorStyle::DefaultUserShape));
  }

  #[test]
  fn cursor_style_equals1() {
    assert!(cursor_style_eq(
      &CursorStyle::DefaultUserShape,
      &CursorStyle::DefaultUserShape
    ));
    let cs1 = CursorStyle::DefaultUserShape;
    let cs2 = CursorStyle::BlinkingBlock;
    let cs3 = CursorStyle::DefaultUserShape;
    assert!(!cursor_style_eq(&cs1, &cs2));
    assert!(cursor_style_eq(&cs1, &cs3));
  }
}
