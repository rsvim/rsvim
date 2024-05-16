//! Rectangle position and size.

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
/// Axis system on [terminal](crate::ui::term::Terminal).
pub struct Position<T> {
  /// Also known as column number.
  pub x: T,
  /// Also known as row number.
  pub y: T,
}

impl<T> Position<T> {
  /// Create new [position](crate::ui::rect::Position).
  pub fn new(x: T, y: T) -> Self {
    Position { x, y }
  }
}

/// Relative [position](crate::ui::rect::Position), the coordinates
/// ([x](crate::ui::rect::IPos::x)/[y](crate::ui::rect::IPos::y)) could be negative.
pub type IPos = Position<isize>;

/// Absolute [position](crate::ui::rect::Position), the coordinates
/// ([x](crate::ui::rect::UPos::x)/[y](crate::ui::rect::UPos::y)) are always non-negative.
pub type UPos = Position<usize>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
/// Rectangle size.
pub struct Size {
  /// Also known as rows count.
  pub height: usize,
  /// Also known as columns count.
  pub width: usize,
}

impl Size {
  /// Make new [size](crate::ui::rect::Size) from [height](crate::ui::rect::Size::height) and [width](crate::ui::rect::Size::width).
  pub fn new(height: usize, width: usize) -> Self {
    Size { height, width }
  }

  /// The area of the [size](crate::ui::rect::Size), i.e. [height](crate::ui::rect::Size::height) * [width](crate::ui::rect::Size::width).
  pub fn area(&self) -> usize {
    self.height * self.width
  }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
/// Rectangle.
pub struct Rect {
  /// Rectangle [position](crate::ui::rect::UPos).
  pub pos: UPos,
  /// Rectangle [size](crate::ui::rect::Size).
  pub size: Size,
}

impl Rect {
  /// Make new rect from position and size.
  pub fn new(pos: UPos, size: Size) -> Self {
    Rect { pos, size }
  }

  /// Same as `self.pos.x`.
  pub fn x(&self) -> usize {
    self.pos.x
  }

  /// Same as `self.pos.y`.
  pub fn y(&self) -> usize {
    self.pos.y
  }

  /// Same as `self.size.height`.
  pub fn height(&self) -> usize {
    self.size.height
  }

  /// Same as `self.size.width`.
  pub fn width(&self) -> usize {
    self.size.width
  }

  /// Same as `self.size.area()`.
  pub fn area(&self) -> usize {
    self.size.area()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn should_all_zero_on_pos_default() {
    let p1: IPos = IPos::default();
    assert_eq!(p1.x, 0);
    assert_eq!(p1.y, 0);
    let p2: UPos = UPos::default();
    assert_eq!(p2.x, 0);
    assert_eq!(p2.y, 0);
  }

  #[test]
  fn should_equal_on_size_area() {
    let sz = Size::new(5, 10);
    assert_eq!(sz.height, 5);
    assert_eq!(sz.width, 10);
    assert_eq!(sz.area(), 5 * 10);
  }

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
