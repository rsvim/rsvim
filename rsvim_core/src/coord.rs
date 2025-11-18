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
pub type Point<T> = taffy::geometry::Point<T>;
pub type IPos = Point<isize>;
pub type UPos = Point<usize>;
pub type U16Pos = Point<u16>;

// Rectangle
pub type Rect<T> = taffy::geometry::Rect<T>;
pub type IRect = Rect<isize>;
pub type URect = Rect<usize>;
pub type U16Rect = Rect<u16>;

pub trait RectExt<T> {
  /// `min` and `top_left` are the same.
  fn min(&self) -> Point<T>;
  fn top_left(&self) -> Point<T>;

  /// `max` and `bottom_right` are the same.
  fn max(&self) -> Point<T>;
  fn bottom_right(&self) -> Point<T>;

  fn height(&self) -> T;
  fn width(&self) -> T;
}

impl<T> RectExt<T> for Rect<T>
where
  T: Copy + PartialOrd + num_traits::Num,
{
  fn min(&self) -> Point<T> {
    self.top_left()
  }

  fn top_left(&self) -> Point<T> {
    Point {
      x: self.left,
      y: self.top,
    }
  }

  fn max(&self) -> Point<T> {
    self.bottom_right()
  }

  fn bottom_right(&self) -> Point<T> {
    Point {
      x: self.right,
      y: self.bottom,
    }
  }

  fn height(&self) -> T {
    debug_assert!(self.bottom >= self.top);
    self.bottom - self.top
  }

  fn width(&self) -> T {
    debug_assert!(self.right >= self.left);
    self.right - self.left
  }
}

// Size
pub type Size<T> = taffy::geometry::Size<T>;
pub type ISize = Size<isize>;
pub type USize = Size<usize>;
pub type U16Size = Size<u16>;

#[macro_export]
macro_rules! point {
  ($x:expr,$y:expr) => {
    $crate::coord::Point { x: $x, y: $y }
  };
}

/// Convert the generic type `T` inside `geo::Point<T>` to another type `U`.
#[macro_export]
macro_rules! point_as {
  ($p:ident,$ty:ty) => {
    $crate::coord::Point {
      x: $p.x as $ty,
      y: $p.y as $ty,
    }
  };
}

#[macro_export]
macro_rules! rect {
  ($left:expr,$top:expr,$right:expr,$bottom:expr) => {
    $crate::coord::Rect {
      left: $left,
      top: $top,
      right: $right,
      bottom: $bottom,
    }
  };
}

/// Convert the generic type `T` inside `geo::Rect<T>` to another type `U`.
#[macro_export]
macro_rules! rect_as {
  ($r:ident,$ty:ty) => {
    $crate::coord::Rect {
      left: $r.left as $ty,
      top: $r.top as $ty,
      right: $r.right as $ty,
      bottom: $r.bottom as $ty,
    } as $crate::coord::Rect<$ty>
  };
}

#[macro_export]
macro_rules! size {
  ($width:expr,$height:expr) => {
    $crate::coord::Size {
      width: $width,
      height: $height,
    }
  };
}

/// Convert the generic type `T` inside `Size<T>` to another type `U`.
#[macro_export]
macro_rules! size_as {
  ($s:ident,$ty:ty) => {
    $crate::coord::Size {
      width: $s.width as $ty,
      height: $s.height as $ty,
    } as $crate::coord::Size<$ty>
  };
}

/// Convert the `Size<T>` to `Rect<U>` with another type `U`. The min point is `(0, 0)`, max point
/// is `(width, height)` where width/height is from `Size<T>`.
#[macro_export]
macro_rules! size_into_rect {
  ($s:ident,$ty:ty) => {
    $crate::coord::Rect {
      left: 0 as $ty,
      top: 0 as $ty,
      right: $s.width as $ty,
      bottom: $s.height as $ty,
    } as $crate::coord::Rect<$ty>
  };
}
