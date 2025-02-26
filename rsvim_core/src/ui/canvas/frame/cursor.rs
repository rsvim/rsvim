//! Cursor of canvas frame.

use crate::cart::U16Pos;
use geo::point;
use std::cmp::{Eq, PartialEq};
use std::fmt;

pub type CCursorStyle = crossterm::cursor::SetCursorStyle;

/// Whether two `CursorStyle` equals.
pub fn ccursor_style_eq(a: &CCursorStyle, b: &CCursorStyle) -> bool {
  match a {
    CCursorStyle::DefaultUserShape => {
      matches!(b, CCursorStyle::DefaultUserShape)
    }
    CCursorStyle::BlinkingBlock => {
      matches!(b, CCursorStyle::BlinkingBlock)
    }
    CCursorStyle::SteadyBlock => {
      matches!(b, CCursorStyle::SteadyBlock)
    }
    CCursorStyle::BlinkingUnderScore => {
      matches!(b, CCursorStyle::BlinkingUnderScore)
    }
    CCursorStyle::SteadyUnderScore => {
      matches!(b, CCursorStyle::SteadyUnderScore)
    }
    CCursorStyle::BlinkingBar => {
      matches!(b, CCursorStyle::BlinkingBar)
    }
    CCursorStyle::SteadyBar => {
      matches!(b, CCursorStyle::SteadyBar)
    }
  }
}

#[derive(Copy, Clone)]
/// Terminal cursor.
pub struct CCursor {
  pos: U16Pos,
  blinking: bool,
  hidden: bool,
  style: CCursorStyle,
}

/// The [`CCursorStyle`] formatter that helps implement the `Debug`/`Display` trait.
pub struct CCursorStyleFormatter {
  value: CCursorStyle,
}

impl From<CCursorStyle> for CCursorStyleFormatter {
  fn from(style: CCursorStyle) -> Self {
    CCursorStyleFormatter { value: style }
  }
}

impl fmt::Debug for CCursorStyleFormatter {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
    match self.value {
      CCursorStyle::DefaultUserShape => write!(f, "DefaultUserShape"),
      CCursorStyle::BlinkingBlock => write!(f, "BlinkingBlock"),
      CCursorStyle::SteadyBlock => write!(f, "SteadyBlock"),
      CCursorStyle::BlinkingUnderScore => write!(f, "BlinkingUnderScore"),
      CCursorStyle::SteadyUnderScore => write!(f, "SteadyUnderScore"),
      CCursorStyle::BlinkingBar => write!(f, "BlinkingBar"),
      CCursorStyle::SteadyBar => write!(f, "SteadyBar"),
    }
  }
}

impl CCursor {
  /// Make new terminal cursor.
  pub fn new(pos: U16Pos, blinking: bool, hidden: bool, style: CCursorStyle) -> Self {
    CCursor {
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
  pub fn style(&self) -> CCursorStyle {
    self.style
  }

  /// Set style.
  pub fn set_style(&mut self, style: CCursorStyle) {
    self.style = style;
  }
}

impl Default for CCursor {
  /// Make default cursor.
  fn default() -> Self {
    CCursor {
      pos: point! {x:0_u16, y:0_u16},
      blinking: true,
      hidden: false,
      style: CCursorStyle::DefaultUserShape,
    }
  }
}

impl fmt::Debug for CCursor {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
    let style_formatter = CCursorStyleFormatter::from(self.style);
    f.debug_struct("Cursor")
      .field("pos", &self.pos)
      .field("blinking", &self.blinking)
      .field("hidden", &self.hidden)
      .field("style", &style_formatter)
      .finish()
  }
}

impl PartialEq for CCursor {
  /// Whether two cursors equals to each other.
  fn eq(&self, other: &Self) -> bool {
    self.pos == other.pos
      && self.blinking == other.blinking
      && self.hidden == other.hidden
      && ccursor_style_eq(&self.style, &other.style)
  }
}

impl Eq for CCursor {}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default1() {
    let c = CCursor::default();
    assert!(c.blinking);
    assert!(!c.hidden);
    assert!(ccursor_style_eq(
      &c.style(),
      &CCursorStyle::DefaultUserShape
    ));
  }

  #[test]
  fn cursor_style_equals1() {
    assert!(ccursor_style_eq(
      &CCursorStyle::DefaultUserShape,
      &CCursorStyle::DefaultUserShape
    ));
    let cs1 = CCursorStyle::DefaultUserShape;
    let cs2 = CCursorStyle::BlinkingBlock;
    let cs3 = CCursorStyle::DefaultUserShape;
    assert!(!ccursor_style_eq(&cs1, &cs2));
    assert!(ccursor_style_eq(&cs1, &cs3));
  }

  #[test]
  fn debug1() {
    let cursors = [
      CCursor::default(),
      CCursor::new(
        point!(x: 0_u16, y: 10_u16),
        false,
        true,
        CCursorStyle::SteadyUnderScore,
      ),
      CCursor::new(
        point!(x: 7_u16, y: 3_u16),
        true,
        false,
        CCursorStyle::BlinkingBar,
      ),
    ];
    let expects = [
        "Cursor { pos: Point(Coord { x: 0, y: 0 }), blinking: true, hidden: false, style: DefaultUserShape }",
        "Cursor { pos: Point(Coord { x: 0, y: 10 }), blinking: false, hidden: true, style: SteadyUnderScore }",
        "Cursor { pos: Point(Coord { x: 7, y: 3 }), blinking: true, hidden: false, style: BlinkingBar }"
    ];
    for (i, c) in cursors.iter().enumerate() {
      let actual = format!("{:?}", c);
      let expect = expects[i];
      assert_eq!(expect, actual);
    }
  }
}
