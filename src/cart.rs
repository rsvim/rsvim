//! Cartesian coordinate system on the 2-dimensional plane.
//!
//! For terminal based coordinate system, it's not working like the 2-dimensional coordinate system
//! in mathematics. In mathematics, the 2-dimensional coordinates look like:
//!
//! The 2-dimensional coordinate system in mathematics usually look like:
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
//! column.
//!
//! Thus we need to flip the coordinate system upside down:
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
//! This is also compatible with the coordinates used by the
//! [crossterm](https://docs.rs/crossterm/latest/crossterm/index.html) library.

use geo::{Point, Rect};
use std::marker::Copy;

// Positions {

pub type IPos = Point<isize>;

pub type UPos = Point<usize>;
pub type U16Pos = Point<u16>;

// Positions }

// Rectangles {

pub type IRect = Rect<isize>;

pub type URect = Rect<usize>;
pub type U16Rect = Rect<u16>;

// Rectangles }

// Size {

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
/// Rectangle size.
pub struct Size<T: Copy> {
  width: T,
  height: T,
}

impl<T> Size<T>
where
  T: Copy + PartialOrd + std::fmt::Debug + num_traits::Num + num_traits::NumCast,
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
  T: Copy + PartialOrd + std::fmt::Debug + num_traits::Num + num_traits::NumCast,
{
  fn from(rect: Rect<T>) -> Size<T> {
    Size::new(rect.width() as T, rect.height() as T)
  }
}

pub type ISize = Size<isize>;
pub type USize = Size<usize>;
pub type U16Size = Size<u16>;

// Size }

/// Convert the generic type `T` inside `geo::Point<T>` to another type `U`.
#[macro_export]
macro_rules! geo_point_as {
  ($point_var:ident,$type_name:ty) => {
    geo::point!(x: $point_var.x() as $type_name, y: $point_var.y() as $type_name)
  };
}

/// Convert the generic type `T` inside `geo::Rect<T>` to another type `U`.
///
/// It requires:
///
/// ```rust
/// use geo::{self, point};
/// ```
#[macro_export]
macro_rules! geo_rect_as {
  ($rect_var:ident,$type_name:ty) => {
    geo::Rect::new(geo::point!(x: $rect_var.min().x as $type_name, y: $rect_var.min().y as $type_name), geo::point!(x: $rect_var.max().x as $type_name, y: $rect_var.max().y as $type_name)) as geo::Rect<$type_name>
  };
}

/// Convert the generic type `T` inside `Size<T>` to another type `U`.
#[macro_export]
macro_rules! geo_size_as {
  ($size_var:ident,$type_name:ty) => {
    Size::new(
      $size_var.height as $type_name,
      $size_var.width as $type_name,
    ) as Size<$type_name>
  };
}

#[cfg(test)]
mod tests {
  use super::*;
  use geo::{point, Rect};
  use std::mem;

  #[test]
  fn cast_geo_points() {
    let p1: IPos = point!(x: 1, y: 2);
    let actual1 = geo_point_as!(p1, usize);
    let actual1_x = actual1.x();
    let actual1_y = actual1.y();
    assert_eq!(mem::size_of_val(&actual1_x), mem::size_of_val(&1_usize));
    assert_eq!(mem::size_of_val(&actual1_y), mem::size_of_val(&2_usize));

    let p2: U16Pos = point!(x: 15_u16, y: 25_u16);
    let actual2 = geo_point_as!(p2, i32);
    let actual2_x = actual2.x();
    let actual2_y = actual2.y();
    assert_eq!(mem::size_of_val(&actual2_x), mem::size_of_val(&15_i32));
    assert_eq!(mem::size_of_val(&actual2_y), mem::size_of_val(&25_i32));

    let p3: Point<u32> = point!(x: 78_u32, y: 88_u32);
    let actual3 = geo_point_as!(p3, i16);
    let actual3_x = actual3.x();
    let actual3_y = actual3.y();
    assert_eq!(mem::size_of_val(&actual3_x), mem::size_of_val(&78_i16));
    assert_eq!(mem::size_of_val(&actual3_y), mem::size_of_val(&88_i16));
  }

  #[test]
  fn cast_geo_rects() {
    let r1: IRect = IRect::new((1, 2), (3, 4));
    let actual1 = geo_rect_as!(r1, u8);
    let actual1_min = actual1.min();
    let actual1_max = actual1.max();
    assert_eq!(mem::size_of_val(&actual1_min.x), mem::size_of_val(&1_u8));
    assert_eq!(mem::size_of_val(&actual1_min.y), mem::size_of_val(&2_u8));
    assert_eq!(mem::size_of_val(&actual1_max.x), mem::size_of_val(&3_u8));
    assert_eq!(mem::size_of_val(&actual1_max.y), mem::size_of_val(&4_u8));

    let r2: Rect<u16> = Rect::new((15_u16, 25_u16), (35_u16, 45_u16));
    let actual2 = geo_rect_as!(r2, i32);
    let actual2_min = actual2.min();
    let actual2_max = actual2.max();
    assert_eq!(mem::size_of_val(&actual2_min.x), mem::size_of_val(&15_i32));
    assert_eq!(mem::size_of_val(&actual2_min.y), mem::size_of_val(&25_i32));
    assert_eq!(mem::size_of_val(&actual2_max.x), mem::size_of_val(&35_i32));
    assert_eq!(mem::size_of_val(&actual2_max.y), mem::size_of_val(&45_i32));

    let r3: Rect<u32> = Rect::new((78_u32, 88_u32), (99_u32, 100_u32));
    let actual3 = geo_rect_as!(r3, i16);
    let actual3_min = actual3.min();
    let actual3_max = actual3.max();
    assert_eq!(mem::size_of_val(&actual3_min.x), mem::size_of_val(&78_i16));
    assert_eq!(mem::size_of_val(&actual3_min.y), mem::size_of_val(&88_i16));
    assert_eq!(mem::size_of_val(&actual3_max.x), mem::size_of_val(&99_i16));
    assert_eq!(mem::size_of_val(&actual3_max.y), mem::size_of_val(&100_i16));
  }

  #[test]
  fn cast_geo_sizes() {
    let s1: ISize = ISize::new(1, 2);
    let actual1 = geo_size_as!(s1, u8);
    let actual1_w = actual1.width();
    let actual1_h = actual1.height();
    assert_eq!(mem::size_of_val(&actual1_w), mem::size_of_val(&1_u8));
    assert_eq!(mem::size_of_val(&actual1_h), mem::size_of_val(&2_u8));

    let s2: U16Size = U16Size::new(15_u16, 25_u16);
    let actual2 = geo_size_as!(s2, i32);
    let actual2_w = actual2.width();
    let actual2_h = actual2.height();
    assert_eq!(mem::size_of_val(&actual2_w), mem::size_of_val(&15_i32));
    assert_eq!(mem::size_of_val(&actual2_h), mem::size_of_val(&25_i32));

    let s3: Size<u32> = Size::new(78_u32, 88_u32);
    let actual3 = geo_size_as!(s3, i16);
    let actual3_h = actual3.height();
    let actual3_w = actual3.width();
    assert_eq!(mem::size_of_val(&actual3_w), mem::size_of_val(&78_i16));
    assert_eq!(mem::size_of_val(&actual3_h), mem::size_of_val(&88_i16));
  }
}
