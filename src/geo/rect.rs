//! Rectangle: position + size.

use crate::geo::pos::UPos;
use crate::geo::size::Size;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
/// Rectangle.
pub struct Rect {
  /// Rectangle top-left corner's position.
  pub pos: UPos,
  /// Rectangle's size.
  pub size: Size,
}

impl Rect {
  /// Make new [rect](Rect) from [position](UPos) and [size](Size).
  pub fn new(pos: UPos, size: Size) -> Self {
    Rect { pos, size }
  }

  /// Same as [self.pos.x](UPos::x).
  pub fn x(&self) -> usize {
    self.pos.x
  }

  /// Same as [self.pos.y](UPos::y).
  pub fn y(&self) -> usize {
    self.pos.y
  }

  /// Same as [self.size.height](Size::height).
  pub fn height(&self) -> usize {
    self.size.height
  }

  /// Same as [self.size.width](Size::width).
  pub fn width(&self) -> usize {
    self.size.width
  }

  /// Same as [self.size.area()](Size::area()).
  pub fn area(&self) -> usize {
    self.size.area()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn should_equal_on_rect_area() {
    let r = Rect::new(UPos::new(1, 2), Size::new(3, 4));
    assert_eq!(r.x(), 1);
    assert_eq!(r.y(), 2);
    assert_eq!(r.height(), 3);
    assert_eq!(r.width(), 4);
    assert_eq!(r.area(), 3 * 4);
  }
}
