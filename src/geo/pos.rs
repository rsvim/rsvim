//! Coordinates system: x/y, also known as row/column.

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
/// Coordinates system on [terminal](crate::ui::term::Terminal).
pub struct Pos<T> {
  /// Also known as the column number.
  pub x: T,
  /// Also known as the row number.
  pub y: T,
}

impl<T> Pos<T> {
  /// Create new [position](Pos) from [x](Pos::x) and [y](Pos::y).
  pub fn new(x: T, y: T) -> Self {
    Pos { x, y }
  }
}

/// Relative position, the coordinates ([x](IPos::x)/[y](IPos::y)) could be negative.
pub type IPos = Pos<isize>;

/// Absolute position, the coordinates ([x](UPos::x)/[y](UPos::y)) are always non-negative.
pub type UPos = Pos<usize>;

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
}
