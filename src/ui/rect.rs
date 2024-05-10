#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
/// Axis system on terminal screen: x/y.
///
/// * `x`: Also known as column number.
/// * `y`: Also known as row number.
pub struct Position<T> {
  /// Column number.
  pub x: T,
  /// Row number.
  pub y: T,
}

impl<T> Position<T> {
  pub fn new(x: T, y: T) -> Self {
    Position { x, y }
  }
}

// Relative position.
pub type IPos = Position<isize>;

// Absolute position.
pub type UPos = Position<usize>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
/// Rectangle size.
///
/// * `height`: Also known as rows count.
/// * `width`: Also known as columns count.
pub struct Size {
  pub height: usize,
  pub width: usize,
}

impl Size {
  pub fn new(height: usize, width: usize) -> Self {
    Size { height, width }
  }

  pub fn area(&self) -> usize {
    self.height * self.width
  }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
/// Rectangle.
///
/// * `pos`: Rectangle position.
/// * `size`: Rectangle size.
pub struct Rect {
  pub pos: UPos,
  pub size: Size,
}

impl Rect {
  pub fn new(pos: UPos, size: Size) -> Self {
    Rect { pos, size }
  }

  pub fn x(&self) -> usize {
    self.pos.x
  }

  pub fn y(&self) -> usize {
    self.pos.y
  }

  pub fn height(&self) -> usize {
    self.size.height
  }

  pub fn width(&self) -> usize {
    self.size.width
  }

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
