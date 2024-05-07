#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position<T> {
  pub x: T, // row
  pub y: T, // column
}

impl<T> Position<T> {
  pub fn new(x: T, y: T) -> Self {
    Position { x, y }
  }

  pub fn swap(self) -> Self {
    Position::new(self.y, self.x)
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

// relative position.
pub type Pos = Position<isize>;

// absolute position.
pub type AbsPos = Position<usize>;

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

  pub fn swap(self) -> Self {
    Size::new(self.width, self.height)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn should_all_zero_on_relpos_default() {
    let p1: Pos = Default::default();
    let p2 = Pos::new(0, 0);
    assert_eq!(p1, p2);
  }

  #[test]
  fn should_reverse_after_relpos_swap() {
    let p1 = Pos::new(1, 2);
    assert_eq!(p1.swap(), Pos::new(2, 1));
  }

  #[test]
  fn should_equal_row_column_on_relpos_x_y() {
    let p1 = Pos::new(5, 10);
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
