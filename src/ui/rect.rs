#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
/// Axis system on terminal screen: x/y.
///
/// * `x`: Column number.
/// * `y`: Row number.
pub struct Position<T> {
  pub x: T, // row
  pub y: T, // column
}

impl<T> Position<T> {
  pub fn new(x: T, y: T) -> Self {
    Position { x, y }
  }
}

// relative position.
pub type IPos = Position<isize>;

// absolute position.
pub type UPos = Position<usize>;

/// Rectangle size: height/width.
///
/// * `height`: Rows count.
/// * `width`: Columns count.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Size {
  // height
  pub height: usize,
  // width
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
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn should_all_zero_on_relpos_default() {
    let p1: IPos = Default::default();
    let p2 = IPos::new(0, 0);
    assert_eq!(p1, p2);
  }

  #[test]
  fn should_equal_row_column_on_relpos_x_y() {
    let p1 = IPos::new(5, 10);
    assert_eq!(p1.x, 5);
    assert_eq!(p1.y, 10);
  }

  #[test]
  fn should_all_zero_on_abspos_default() {
    let p1: UPos = Default::default();
    let p2 = UPos::new(0, 0);
    assert_eq!(p1, p2);
  }

  #[test]
  fn should_equal_row_column_on_abspos_x_y() {
    let p1 = UPos::new(5, 10);
    assert_eq!(p1.x, 5);
    assert_eq!(p1.y, 10);
  }

  #[test]
  fn should_all_zero_on_size_default() {
    let p1: Size = Default::default();
    let p2 = Size::new(0, 0);
    assert_eq!(p1, p2);
  }
}
