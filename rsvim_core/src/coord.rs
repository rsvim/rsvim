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
//!  (left,top)
//!   (0,0)------------------(width,0)--------Y
//!     |                         |
//!     |  Terminal               |
//!     |                         |
//!     |                         |
//!   (0,height)-------------(width,height)
//!     |                    (right,bottom)
//!     |
//!     X
//! ```
//!
//! NOTE: The X-axis remains the same, the Y-axis is upside down.
//!
//! The top-left of the terminal is the `(0,0)` position, e.g. `(left,top)`.
//! The bottom-right of the terminal is the `(width,height)` position, e.g.
//! `(right,bottom)`. The `width` and `height` is the size of the terminal.
//!
//! This is also compatible with the coordinates used in the
//! [crossterm](https://docs.rs/crossterm/latest/crossterm/index.html) library.

// Coord
pub type Coord<T> = geo::Coord<T>;

// Position
pub type Point<T> = geo::Point<T>;
pub type IPos = Point<isize>;
pub type UPos = Point<usize>;
pub type U16Pos = Point<u16>;

// Rectangle
pub type Rect<T> = geo::Rect<T>;
pub type IRect = Rect<isize>;
pub type URect = Rect<usize>;
pub type U16Rect = Rect<u16>;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
// Size
pub struct Size<T>
where
  T: geo::CoordNum,
{
  width: T,
  height: T,
}

pub type ISize = Size<isize>;
pub type USize = Size<usize>;
pub type U16Size = Size<u16>;

impl<T> Size<T>
where
  T: geo::CoordNum,
{
  pub fn new(width: T, height: T) -> Self {
    Self { width, height }
  }

  pub fn width(&self) -> T {
    self.width
  }

  pub fn height(&self) -> T {
    self.height
  }
}

#[macro_export]
macro_rules! point {
  ($x:expr,$y:expr) => {
    geo::point!(x: $x, y: $y)
  };
}

/// Convert the generic type `T` inside `geo::Point<T>` to another type `U`.
#[macro_export]
macro_rules! point_as {
  ($p:ident,$ty:ty) => {
    geo::point!(x: $p.x() as $ty, y: $p.y() as $ty)
  };
}

#[macro_export]
macro_rules! rect {
  ($left:expr,$top:expr,$right:expr,$bottom:expr) => {
    $crate::coord::Rect::new(($left, $top), ($right, $bottom))
  };

  (($left:expr,$top:expr),($right:expr,$bottom:expr)) => {
    $crate::coord::Rect::new(($left, $top), ($right, $bottom))
  };

  ($min:expr,$max:expr) => {
    $crate::coord::Rect::new($min, $max)
  };
}

/// Convert the generic type `T` inside `geo::Rect<T>` to another type `U`.
#[macro_export]
macro_rules! rect_as {
  ($r:ident,$ty:ty) => {
    $crate::coord::Rect::new(
      ($r.min().x as $ty, $r.min().y as $ty),
      ($r.max().x as $ty, $r.max().y as $ty),
    ) as $crate::coord::Rect<$ty>
  };
}

#[macro_export]
macro_rules! rect_from_layout {
  ($l:ident,$tt:ty) => {
    {
      let pos = geo::point!(x: $l.location.x as $tt, y: $l.location.y as $tt);
      let size = Size::new($l.size.width as $tt, $l.size.height as $tt);
      Rect::new(
        (pos.x(), pos.y()),
        (pos.x() + size.width(), pos.y() + size.height()),
      )
    }
  }
}

#[macro_export]
macro_rules! size {
  ($width:expr,$height:expr) => {
    $crate::coord::Size::new($width, $height)
  };
}

/// Convert the generic type `T` inside `Size<T>` to another type `U`.
#[macro_export]
macro_rules! size_as {
  ($s:ident,$ty:ty) => {
    $crate::coord::Size::new($s.width() as $ty, $s.height() as $ty)
      as $crate::coord::Size<$ty>
  };
}

/// Convert the `Size<T>` to `Rect<U>` with another type `U`. The min point is `(0, 0)`, max point
/// is `(width, height)` where width/height is from `Size<T>`.
#[macro_export]
macro_rules! size_into_rect {
  ($s:ident,$ty:ty) => {
    $crate::coord::Rect::new(
      (0 as $ty, 0 as $ty),
      ($s.width() as $ty, $s.height() as $ty),
    ) as $crate::coord::Rect<$ty>
  };
}
