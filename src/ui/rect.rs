#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pos<T> {
  pub x: T, // row
  pub y: T, // column
}

impl<T> Pos<T> {
  pub fn new(x: T, y: T) -> Self {
    Pos { x, y }
  }

  pub fn swap(self) -> Self {
    Pos::new(self.y, self.x)
  }

  // x-axis on the coordinate is row number on terminal
  pub fn row(self) -> T {
    self.x
  }

  // y-axis on the coordinate is column number on terminal
  pub fn column(self) -> T {
    self.y
  }
}

// Relative position.
pub type RelPos = Pos<i32>;

// Absolute position.
pub type AbsPos = Pos<u32>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Size {
  // height
  pub height: u32,
  // width
  pub width: u32,
  // absolute position of top-left corner
  pub pos: AbsPos,
}

impl Size {
  pub fn new(height: u32, width: u32, pos: AbsPos) -> Self {
    Size { height, width, pos }
  }

  pub fn swap(self) -> Self {
    Size::new(self.width, self.height, self.pos)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn should_all_zero_on_relpos_default() {
    let p1: RelPos = Default::default();
    let p2 = RelPos::new(0, 0);
    assert_eq!(p1, p2);
  }

  #[test]
  fn should_reverse_after_relpos_swap() {
    let p1 = RelPos::new(1, 2);
    assert_eq!(p1.swap(), RelPos::new(2, 1));
  }

  #[test]
  fn should_equal_row_column_on_relpos_x_y() {
    let p1 = RelPos::new(5, 10);
    assert_eq!(p1.column(), 10);
    assert_eq!(p1.row(), 5);
  }

  #[test]
  fn should_all_zero_on_abspos_default() {
    let p1: AbsPos = Default::default();
    let p2 = AbsPos::new(0, 0);
    assert_eq!(p1, p2);
  }

  #[test]
  fn should_reverse_after_abspos_swap() {
    let p1 = AbsPos::new(1, 2);
    assert_eq!(p1.swap(), AbsPos::new(2, 1));
  }

  #[test]
  fn should_equal_row_column_on_abspos_x_y() {
    let p1 = AbsPos::new(5, 10);
    assert_eq!(p1.column(), 10);
    assert_eq!(p1.row(), 5);
  }

  #[test]
  fn should_all_zero_on_size_default() {
    let p1: Size = Default::default();
    let p2 = Size::new(0, 0);
    assert_eq!(p1, p2);
  }

  #[test]
  fn should_reverse_on_size_swap() {
    let p2 = Size::new(100, 50);
    let p1 = p2.swap();
    assert_eq!(p1, Size::new(50, 100));
  }
}
