//! Coordinates system: x/y, also known as row/column.

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
/// Coordinates system on [terminal](crate::ui::term::Terminal).
pub struct Position<T> {
  /// Also known as the column number.
  pub x: T,
  /// Also known as the row number.
  pub y: T,
}

impl<T> Position<T> {
  /// Create new [position](Position) from [x](Position::x) and [y](Position::y).
  pub fn new(x: T, y: T) -> Self {
    Position { x, y }
  }
}

/// Relative position, the coordinates ([x](IPosition::x)/[y](IPosition::y)) could be negative.
pub type IPosition = Position<isize>;

/// Absolute position, the coordinates ([x](UPosition::x)/[y](UPosition::y)) are always
/// non-negative.
pub type UPosition = Position<usize>;
