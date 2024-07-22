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

use crate::cart::{IPos, IRect, ISize, UPos, URect};
use crate::geo_point_as;

/// Convert relative/logical shape to actual shape.
/// Note: If the widget doesn't have a parent, use the terminal shape as its parent's shape.
pub fn to_actual_shape(rect: IRect, parent_actual_shape: URect) -> URect {
  let parent_actual_pos: UPos = parent_actual_shape.min().into();
  let parent_actual_ipos: IPos = geo_point_as!(parent_actual_pos, isize);
  let pos: IPos = <IPos>::from(rect.min()) + parent_actual_ipos;
  let bounded_x = min(max(pos.x(), 0), parent_actual_shape.width() as isize);
  let bounded_y = min(max(pos.y(), 0), parent_actual_shape.height() as isize);
  let actual_pos: UPos = point!(x: bounded_x as usize, y: bounded_y as usize);

  let bottom_right_pos: IPos = rect.max().into();
  let bottom_right_bounded_x = min(bottom_right_pos.x(), parent_actual_shape.width() as isize);
  let bottom_right_bounded_y = min(bottom_right_pos.y(), parent_actual_shape.height() as isize);
  let bottom_right_actual_pos: IPos = point!(x: bottom_right_bounded_x, y: bottom_right_bounded_y);
  let actual_isize = ISize::new(
    bottom_right_actual_pos.y() - actual_pos.y() as isize,
    bottom_right_actual_pos.x() - actual_pos.x() as isize,
  );
  URect::new(
    actual_pos,
    point!(x: actual_pos.x() + actual_isize.width() as usize, y: actual_pos.y() + actual_isize.height() as usize),
  )
}

#[cfg(test)]
mod tests {
  use super::*;
  use geo::point;

  #[test]
  fn should_make_actual_shapes() {
    let inputs: Vec<(IRect, Option<UPos>, USize, U16Size)> = vec![(
      IRect::new((0, 0), (3, 5)),
      Some(point!(x: 0_usize, y: 0_usize)),
      USize::new(7, 8),
      U16Size::new(10, 10),
    )];
    let expects: Vec<URect> = vec![URect::new((0, 0), (3, 4))];
    for (i, t) in inputs.iter().enumerate() {
      let actual = to_actual_shape(t.0, t.1, t.2, t.3);
      let expect = expects[i];
      assert_eq!(actual, expect);
    }
  }
}
