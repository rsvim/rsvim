//! Geometric space and coordinate system on the 2-dimensional Cartesian plain.

use geo::{Point, Rect};
use std::marker::Copy;

pub mod conversion;

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
pub struct Size<T: Copy> {
  pub height: T,
  pub width: T,
}

impl<T> Size<T> {
  pub fn new(height: T, width: T) -> Self {
    Size { height, width }
  }
  pub fn height(&self) -> T {
    self.height
  }
  pub fn width(&self) -> T {
    self.width
  }
}

impl<T> From<Rect<T>> for Size<T>
where
  T: Copy + PartialOrd + std::fmt::Debug + num_traits::Num + num_traits::NumCast,
{
  fn from(rect: Rect<T>) -> Size<T> {
    Size::new(rect.height() as T, rect.width() as T)
  }
}

pub type ISize = Size<isize>;
pub type USize = Size<usize>;
pub type U16Size = Size<u16>;

// Size }

/// Convert `geom::Point<T1>` to another type `geom::Point<T2>`.
#[macro_export]
macro_rules! as_geo_point {
  ($point_var:ident,$type_name:ty) => {
    point!(x: $point_var.x() as $type_name, y: $point_var.y() as $type_name)
  };
}

/// Convert `geo::Rect<T1>` to another type `geo::Rect<T2>`.
#[macro_export]
macro_rules! as_geo_rect {
  ($rect_var:ident,$type_name:ty) => {
    geo::Rect::new(point!(x: $rect_var.min().x as $type_name, y: $rect_var.min().y as $type_name), point!(x: $rect_var.max().x as $type_name, y: $rect_var.max().y as $type_name)) as geo::Rect<$type_name>
  };
}

/// Convert `geom::Size<T1>` to another type `geom::Size<T2>`.
#[macro_export]
macro_rules! as_geo_size {
  ($size_var:ident,$type_name:ty) => {
    crate::geom::Size::new(
      $size_var.height as $type_name,
      $size_var.width as $type_name,
    ) as crate::geom::Size<$type_name>
  };
}

#[cfg(test)]
mod tests {
  use super::*;
  use geo::{point, Rect};
  use std::mem;

  #[test]
  fn point_should_cast_types() {
    let p1: IPos = point!(x: 1, y: 2);
    let actual1 = as_geo_point!(p1, usize);
    let actual1_x = actual1.x();
    let actual1_y = actual1.y();
    assert_eq!(mem::size_of_val(&actual1_x), mem::size_of_val(&1_usize));
    assert_eq!(mem::size_of_val(&actual1_y), mem::size_of_val(&2_usize));

    let p2: U16Pos = point!(x: 15_u16, y: 25_u16);
    let actual2 = as_geo_point!(p2, i32);
    let actual2_x = actual2.x();
    let actual2_y = actual2.y();
    assert_eq!(mem::size_of_val(&actual2_x), mem::size_of_val(&15_i32));
    assert_eq!(mem::size_of_val(&actual2_y), mem::size_of_val(&25_i32));

    let p3: Point<u32> = point!(x: 78_u32, y: 88_u32);
    let actual3 = as_geo_point!(p3, i16);
    let actual3_x = actual3.x();
    let actual3_y = actual3.y();
    assert_eq!(mem::size_of_val(&actual3_x), mem::size_of_val(&78_i16));
    assert_eq!(mem::size_of_val(&actual3_y), mem::size_of_val(&88_i16));
  }

  #[test]
  fn rect_should_cast_types() {
    let r1: IRect = IRect::new((1, 2), (3, 4));
    let actual1 = as_geo_rect!(r1, u8);
    let actual1_min = actual1.min();
    let actual1_max = actual1.max();
    assert_eq!(mem::size_of_val(&actual1_min.x), mem::size_of_val(&1_u8));
    assert_eq!(mem::size_of_val(&actual1_min.y), mem::size_of_val(&2_u8));
    assert_eq!(mem::size_of_val(&actual1_max.x), mem::size_of_val(&3_u8));
    assert_eq!(mem::size_of_val(&actual1_max.y), mem::size_of_val(&4_u8));

    let r2: Rect<u16> = Rect::new((15_u16, 25_u16), (35_u16, 45_u16));
    let actual2 = as_geo_rect!(r2, i32);
    let actual2_min = actual2.min();
    let actual2_max = actual2.max();
    assert_eq!(mem::size_of_val(&actual2_min.x), mem::size_of_val(&15_i32));
    assert_eq!(mem::size_of_val(&actual2_min.y), mem::size_of_val(&25_i32));
    assert_eq!(mem::size_of_val(&actual2_max.x), mem::size_of_val(&35_i32));
    assert_eq!(mem::size_of_val(&actual2_max.y), mem::size_of_val(&45_i32));

    let r3: Rect<u32> = Rect::new((78_u32, 88_u32), (99_u32, 100_u32));
    let actual3 = as_geo_rect!(r3, i16);
    let actual3_min = actual3.min();
    let actual3_max = actual3.max();
    assert_eq!(mem::size_of_val(&actual3_min.x), mem::size_of_val(&78_i16));
    assert_eq!(mem::size_of_val(&actual3_min.y), mem::size_of_val(&88_i16));
    assert_eq!(mem::size_of_val(&actual3_max.x), mem::size_of_val(&99_i16));
    assert_eq!(mem::size_of_val(&actual3_max.y), mem::size_of_val(&100_i16));
  }
}
