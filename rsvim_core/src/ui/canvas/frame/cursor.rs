//! Cursor of canvas frame.

use crate::prelude::*;

use geo::point;
use std::cmp::{Eq, PartialEq};
use std::fmt;

pub type CursorStyle = crossterm::cursor::SetCursorStyle;

/// Whether two `CursorStyle` equals.
pub fn cursor_style_eq(a: &CursorStyle, b: &CursorStyle) -> bool {
  match a {
    CursorStyle::DefaultUserShape => {
      matches!(b, CursorStyle::DefaultUserShape)
    }
    CursorStyle::BlinkingBlock => {
      matches!(b, CursorStyle::BlinkingBlock)
    }
    CursorStyle::SteadyBlock => {
      matches!(b, CursorStyle::SteadyBlock)
    }
    CursorStyle::BlinkingUnderScore => {
      matches!(b, CursorStyle::BlinkingUnderScore)
    }
    CursorStyle::SteadyUnderScore => {
      matches!(b, CursorStyle::SteadyUnderScore)
    }
    CursorStyle::BlinkingBar => {
      matches!(b, CursorStyle::BlinkingBar)
    }
    CursorStyle::SteadyBar => {
      matches!(b, CursorStyle::SteadyBar)
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
/// NOTE: The [`SetCursorStyle`](crossterm::cursor::SetCursorStyle) doesn't implement the
/// `Debug`/`Display` trait before 0.28.1.
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
    match self.value {
      CursorStyle::DefaultUserShape => write!(f, "DefaultUserShape"),
      CursorStyle::BlinkingBlock => write!(f, "BlinkingBlock"),
      CursorStyle::SteadyBlock => write!(f, "SteadyBlock"),
      CursorStyle::BlinkingUnderScore => write!(f, "BlinkingUnderScore"),
      CursorStyle::SteadyUnderScore => write!(f, "SteadyUnderScore"),
      CursorStyle::BlinkingBar => write!(f, "BlinkingBar"),
      CursorStyle::SteadyBar => write!(f, "SteadyBar"),
    }
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
    assert!(cursor_style_eq(&c.style(), &CursorStyle::SteadyBlock));
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

  #[test]
  fn debug1() {
    let cursors = [
      Cursor::default(),
      Cursor::new(
        point!(x: 0_u16, y: 10_u16),
        false,
        true,
        CursorStyle::SteadyUnderScore,
      ),
      Cursor::new(
        point!(x: 7_u16, y: 3_u16),
        true,
        false,
        CursorStyle::BlinkingBar,
      ),
    ];
    let expects = [
      "Cursor { pos: Point(Coord { x: 0, y: 0 }), blinking: false, hidden: false, style: SteadyBlock }",
      "Cursor { pos: Point(Coord { x: 0, y: 10 }), blinking: false, hidden: true, style: SteadyUnderScore }",
      "Cursor { pos: Point(Coord { x: 7, y: 3 }), blinking: true, hidden: false, style: BlinkingBar }",
    ];
    for (i, c) in cursors.iter().enumerate() {
      let actual = format!("{:?}", c);
      let expect = expects[i];
      assert_eq!(expect, actual);
    }
  }
}
