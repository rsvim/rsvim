//! Coordinates on the 2-dimensional Cartesian plain.
//! Relative coordinates use signed integers, absolute coordinates use unsigned integers.

use geo::geometry::Coord;
use geo::geometry::Rect;

// Positions {

pub type IPos = Coord<isize>;

pub type UPos = Coord<usize>;
pub type U16Pos = Coord<u16>;

// Positions }

// Sizes {

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Size<T> {
  pub height: T,
  pub width: T,
}

impl<T> Size<T> {
  pub fn new(height: T, width: T) -> Self {
    Size { height, width }
  }
}

pub type USize = Size<usize>;
pub type U16Size = Size<u16>;

// Sizes }

// Rectangles {

pub type IRect = Rect<isize>;

pub type URect = Rect<usize>;
pub type U16Rect = Rect<u16>;

// Rectangles }
