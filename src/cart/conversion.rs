//! Cartesian coordinate system conversions between children widgets and their parent widget.
//!
//! A widget's shape is always a rectangle, and it's position is the top-left corner of the
//! rectangle.
//!
//! There're two kinds of positions:
//! * Relative: Based on it's parent's position.
//! * Absolute: Based on the terminal device.
//!
//! There're two kinds of sizes:
//! * Logical: An infinite size on the imaginary canvas.
//! * Actual: An actual size bounded by it's parent's actual shape, if it doesn't have a parent,
//!   bounded by the terminal device's actual shape.
//!
//! Note:
//! 1. If a widget doesn't have a parent (i.e. it's the root widget), the relative position is
//!    already absolute.
//! 2. If the relative/logical shape is outside of it's parent or the terminal, it will be
//!    automatically bounded inside of it's parent or the terminal's shape.

use geo::point;
use std::cmp::{max, min};

use crate::cart::{IPos, IRect, ISize, U16Pos, U16Rect};
use crate::geo_point_as;

/// Convert relative/logical shape to actual shape.
/// Note: If the widget doesn't have a parent, use the terminal shape as its parent's shape.
pub fn to_actual_shape(shape: IRect, parent_actual_shape: U16Rect) -> U16Rect {
  let parent_actual_pos: U16Pos = parent_actual_shape.min().into();
  let parent_actual_ipos: IPos = geo_point_as!(parent_actual_pos, isize);
  let pos: IPos = <IPos>::from(shape.min()) + parent_actual_ipos;
  let bounded_x = min(max(pos.x(), 0), parent_actual_shape.width() as isize);
  let bounded_y = min(max(pos.y(), 0), parent_actual_shape.height() as isize);
  let actual_pos: U16Pos = point!(x: bounded_x as u16, y: bounded_y as u16);

  let bottom_right_pos: IPos = shape.max().into();
  let bottom_right_bounded_x = min(bottom_right_pos.x(), parent_actual_shape.width() as isize);
  let bottom_right_bounded_y = min(bottom_right_pos.y(), parent_actual_shape.height() as isize);
  let bottom_right_actual_pos: IPos = point!(x: bottom_right_bounded_x, y: bottom_right_bounded_y);
  let actual_isize = ISize::new(
    bottom_right_actual_pos.y() - actual_pos.y() as isize,
    bottom_right_actual_pos.x() - actual_pos.x() as isize,
  );
  U16Rect::new(
    actual_pos,
    point!(x: actual_pos.x() + actual_isize.width() as u16, y: actual_pos.y() + actual_isize.height() as u16),
  )
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::cart::{IRect, U16Rect, URect};
  use std::cmp::max;

  #[test]
  fn convert_to_actual_shapes() {
    let inputs: Vec<IRect> = vec![
      IRect::new((0, 0), (3, 5)),
      IRect::new((0, 0), (1, 5)),
      IRect::new((0, 0), (3, 7)),
      IRect::new((0, 0), (0, 0)),
      IRect::new((0, 0), (5, 4)),
    ];
    for t in inputs.iter() {
      for p in 0..10 {
        for q in 0..10 {
          let input_actual_parent_shape = U16Rect::new((0, 0), (p as u16, q as u16));
          let expect = U16Rect::new((0, 0), (max(t.max().x, p) as u16, max(t.max().y, q) as u16));
          let actual = to_actual_shape(*t, input_actual_parent_shape);
          assert_eq!(actual, expect);
        }
      }
    }
  }
}
