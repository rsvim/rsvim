//! Coordinate system helper methods for shape/position/size.

#![allow(clippy::let_and_return)]

use geo::point;
use std::cmp::{max, min};
// use tracing::debug;

use crate::cart::{IPos, IRect, ISize, U16Pos, U16Rect};
use crate::geo_point_as;

/// Convert (relative/logical) shape to actual shape.
///
/// Note:
/// 1. If the widget doesn't have a parent, use the terminal shape as its parent's shape.
/// 2. If the relative/logical shape is outside of it's parent or the terminal, it will be
///    automatically bounded inside of it's parent or the terminal's shape.
pub fn convert_to_actual_shape(shape: IRect, parent_actual_shape: U16Rect) -> U16Rect {
  // debug!(
  //   "shape:{:?}, parent_actual_shape:{:?}",
  //   shape, parent_actual_shape
  // );
  let parent_actual_top_left_pos: U16Pos = parent_actual_shape.min().into();
  let parent_actual_top_left_ipos: IPos = geo_point_as!(parent_actual_top_left_pos, isize);
  let parent_actual_bottom_right_pos: U16Pos = parent_actual_shape.max().into();
  let parent_actual_bottom_right_ipos: IPos = geo_point_as!(parent_actual_bottom_right_pos, isize);

  let top_left_pos: IPos = shape.min().into();
  let bottom_right_pos: IPos = shape.max().into();

  let actual_top_left_ipos: IPos = top_left_pos + parent_actual_top_left_ipos;
  let actual_top_left_x = min(
    max(actual_top_left_ipos.x(), parent_actual_top_left_ipos.x()),
    parent_actual_bottom_right_ipos.x(),
  );
  let actual_top_left_y = min(
    max(actual_top_left_ipos.y(), parent_actual_top_left_ipos.y()),
    parent_actual_bottom_right_ipos.y(),
  );
  let actual_top_left_pos: U16Pos =
    point!(x: actual_top_left_x as u16, y: actual_top_left_y as u16);
  // debug!(
  //   "actual_top_left_ipos:{:?}, actual_top_left_pos:{:?}",
  //   actual_top_left_ipos, actual_top_left_pos
  // );

  let actual_bottom_right_ipos: IPos = bottom_right_pos + parent_actual_top_left_ipos;
  let actual_bottom_right_x = min(
    max(
      actual_bottom_right_ipos.x(),
      parent_actual_top_left_ipos.x(),
    ),
    parent_actual_bottom_right_ipos.x(),
  );
  let actual_bottom_right_y = min(
    max(
      actual_bottom_right_ipos.y(),
      parent_actual_top_left_ipos.y(),
    ),
    parent_actual_bottom_right_ipos.y(),
  );
  let actual_bottom_right_pos: U16Pos =
    point!(x: actual_bottom_right_x as u16, y: actual_bottom_right_y as u16);

  let actual_isize = ISize::new(
    (actual_bottom_right_pos.x() as isize) - (actual_top_left_pos.x() as isize),
    (actual_bottom_right_pos.y() as isize) - (actual_top_left_pos.y() as isize),
  );
  // debug!(
  //   "actual_isize:{:?}, actual_top_left_pos:{:?}",
  //   actual_isize, actual_top_left_pos
  // );
  let actual_shape = U16Rect::new(
    actual_top_left_pos,
    point!(x: actual_top_left_pos.x() + actual_isize.width() as u16, y: actual_top_left_pos.y() + actual_isize.height() as u16),
  );
  // debug!(
  //   "actual_isize:{:?}, actual_shape:{:?}",
  //   actual_isize, actual_shape
  // );

  actual_shape
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::cart::{IRect, U16Rect};
  use crate::test::log::init as test_log_init;
  use std::cmp::min;
  use std::sync::Once;
  use tracing::info;

  static INIT: Once = Once::new();

  #[test]
  fn convert_to_actual_shapes1() {
    INIT.call_once(|| test_log_init());

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
          let expect = U16Rect::new((0, 0), (min(t.max().x, p) as u16, min(t.max().y, q) as u16));
          let actual = convert_to_actual_shape(*t, input_actual_parent_shape);
          info!("expect:{:?}, actual:{:?}", expect, actual);
          assert_eq!(actual, expect);
        }
      }
    }
  }

  #[test]
  fn convert_to_actual_shapes2() {
    INIT.call_once(|| test_log_init());

    let inputs: Vec<(IRect, U16Rect)> = vec![
      (IRect::new((0, 0), (3, 5)), U16Rect::new((0, 0), (7, 8))),
      (IRect::new((-3, 1), (1, 5)), U16Rect::new((3, 2), (9, 8))),
      (IRect::new((3, 9), (6, 10)), U16Rect::new((1, 1), (2, 2))),
      (IRect::new((0, 0), (0, 0)), U16Rect::new((0, 0), (0, 0))),
      (IRect::new((5, 3), (6, 4)), U16Rect::new((0, 0), (5, 3))),
    ];
    let expects: Vec<U16Rect> = vec![
      U16Rect::new((0, 0), (3, 5)),
      U16Rect::new((3, 3), (4, 7)),
      U16Rect::new((2, 2), (2, 2)),
      U16Rect::new((0, 0), (0, 0)),
      U16Rect::new((5, 3), (5, 3)),
    ];
    for (i, p) in inputs.iter().enumerate() {
      let actual = convert_to_actual_shape(p.0, p.1);
      let expect = expects[i];
      info!(
        "i:{:?}, input:{:?}, actual:{:?}, expect:{:?}",
        i, p, actual, expect
      );
      assert_eq!(actual, expect);
    }
  }
}
