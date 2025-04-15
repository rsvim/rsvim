//! Cursor of canvas frame.

use crate::prelude::*;

use geo::point;

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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default1() {
    let c = Cursor::default();
    assert!(!c.blinking);
    assert!(!c.hidden);
    assert_eq!(c.style(), CursorStyle::SteadyBlock);
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
