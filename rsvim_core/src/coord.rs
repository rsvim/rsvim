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
//! The top-left of the terminal is the `(0,0)` point, e.g. `(left,top)`. The
//! bottom-right of the terminal is the `(width,height)` point, e.g.
//! `(right,bottom)`. The `width` and `height` is the size of the terminal.
//!
//! This is also compatible with the coordinates used in the
//! [crossterm](https://docs.rs/crossterm/latest/crossterm/index.html) library.

// Point
pub type Point<T> = taffy::geometry::Point<T>;
pub type IPos = Point<isize>;
pub type UPos = Point<usize>;
pub type U16Pos = Point<u16>;

// Rectangle
pub type Rect<T> = taffy::geometry::Rect<T>;
pub type IRect = Rect<isize>;
pub type URect = Rect<usize>;
pub type U16Rect = Rect<u16>;

// Size
pub type Size<T> = taffy::geometry::Size<T>;
pub type ISize = Size<isize>;
pub type USize = Size<usize>;
pub type U16Size = Size<u16>;

/// Convert the generic type `T` inside `geo::Point<T>` to another type `U`.
#[macro_export]
macro_rules! geo_point_as {
  ($p:ident,$ty:ty) => {
    $crate::coord::Point {
      x: $p.x() as $ty,
      y: $p.y() as $ty,
    }
  };
}

/// Convert the generic type `T` inside `geo::Rect<T>` to another type `U`.
#[macro_export]
macro_rules! geo_rect_as {
  ($r:ident,$ty:ty) => {
    $crate::coord::Rect {
      left: $r.left as $ty,
      top: $r.top as $ty,
      right: $r.right as $ty,
      bottom: $r.bottom as $ty,
    } as $crate::coord::Rect<$ty>
  };
}

/// Convert the generic type `T` inside `Size<T>` to another type `U`.
#[macro_export]
macro_rules! geo_size_as {
  ($s:ident,$ty:ty) => {
    $crate::coord::Size::new($s.width() as $ty, $s.height() as $ty) as Size<$ty>
  };
}

/// Convert the `Size<T>` to `Rect<U>` with another type `U`. The min point is `(0, 0)`, max point
/// is `(width, height)` where width/height is from `Size<T>`.
#[macro_export]
macro_rules! geo_size_into_rect {
  ($s:ident,$ty:ty) => {
    geo::Rect::new(
      (0 as $ty, 0 as $ty),
      ($s.width() as $ty, $s.height() as $ty),
    ) as geo::Rect<$ty>
  };
}
