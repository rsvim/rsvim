//! Cartesian coordinate system.
//!
//! For terminal based coordinate system, it's not working like the 2-dimensional coordinate system
//! in mathematics. In mathematics, 2-dimensional coordinate system usually look like:
//!
//! ```text
//!                  Y
//!                  |
//!                (0,1)
//!                  |
//!  X-----(-1,0)--(0,0)--(1,0)-----
//!                  |
//!                (0,-1)
//!                  |
//! ```
//!
//! But in a terminal based coordinate system, it's not working like that.
//!
//! We usually say the line in the top is the first line, the line in the bottom is the last line,
//! the column in the left side is the first column, the column in the right side is the last
//! column. Thus we need to flip the coordinate system upside down:
//!
//! ```text
//!
//!   (0,0)------------------(width,0)--------Y
//!     |                         |
//!     |  Terminal               |
//!     |                         |
//!     |                         |
//!   (0,height)-------------(width,height)
//!     |
//!     X
//! ```
//!
//! NOTE: The X-axis remains the same, the Y-axis is upside down.
//!
//! The top-left of the terminal is the `(0,0)` position, the bottom-right of the terminal is the
//! `(width,height)` position, where the `width` and `height` is the size of the terminal.
//!
//! This is also compatible with the coordinates used in the
//! [crossterm](https://docs.rs/crossterm/latest/crossterm/index.html) library.

use geo::{Point, Rect};

// Positions {

/// Position that uses [`isize`] as internal type.
pub type IPos = Point<isize>;

/// Position that uses [`usize`] as internal type.
pub type UPos = Point<usize>;

/// Position that uses [`u16`] as internal type. NOTE: This is especially for terminal devices.
pub type U16Pos = Point<u16>;

// Positions }

// Rectangles {

/// Rectangle that uses [`isize`] as internal type.
pub type IRect = Rect<isize>;

/// Rectangle that uses [`usize`] as internal type.
pub type URect = Rect<usize>;

/// Rectangle that uses [`u16`] as internal type. NOTE: This is especially for terminal devices.
pub type U16Rect = Rect<u16>;

// Rectangles }

// Size {

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
/// Generic rectangle size.
pub struct Size<
  T: Copy
    + PartialOrd
    + Ord
    + PartialEq
    + Eq
    + std::fmt::Debug
    + num_traits::Num
    + num_traits::NumCast,
> {
  width: T,
  height: T,
}

impl<T> Size<T>
where
  T: Copy
    + PartialOrd
    + Ord
    + PartialEq
    + Eq
    + std::fmt::Debug
    + num_traits::Num
    + num_traits::NumCast,
{
  /// Make size from width(columns) and height(rows).
  ///
  /// NOTE: Width/columns is Y-axis, height/rows is X-axis.
  pub fn new(width: T, height: T) -> Self {
    Size { width, height }
  }

  /// Get width(columns).
  pub fn width(&self) -> T {
    self.width
  }

  /// Get height(rows).
  pub fn height(&self) -> T {
    self.height
  }
}

impl<T> From<Rect<T>> for Size<T>
where
  T: Copy
    + PartialOrd
    + Ord
    + PartialEq
    + Eq
    + std::fmt::Debug
    + num_traits::Num
    + num_traits::NumCast,
{
  /// Make size from [`Rect`].
  fn from(rect: Rect<T>) -> Size<T> {
    Size::new(rect.width() as T, rect.height() as T)
  }
}

/// Size that uses [`isize`] as internal type.
pub type ISize = Size<isize>;

/// Size that uses [`usize`] as internal type.
pub type USize = Size<usize>;

/// Size that uses [`u16`] as internal type. NOTE: This is especially for terminal devices.
pub type U16Size = Size<u16>;

// Size }
