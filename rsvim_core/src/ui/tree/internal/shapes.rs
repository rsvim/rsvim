//! Conversion between relative shape and actual shape.

#![allow(clippy::let_and_return)]

use crate::point_as;
use crate::prelude::*;

/// Convert relative shape to actual shape, based on its parent's actual shape.
///
/// NOTE:
/// 1. If the widget doesn't have a parent, use the terminal shape as its parent's shape.
/// 2. If the relative/logical shape is outside of it's parent or the terminal, it will be
///    automatically bounded inside of it's parent or the terminal's shape.
pub fn make_actual_shape(
  shape: &IRect,
  parent_actual_shape: &U16Rect,
) -> U16Rect {
  // trace!(
  //   "shape:{:?}, parent_actual_shape:{:?}",
  //   shape, parent_actual_shape
  // );
  let parent_actual_top_left_pos: U16Pos = parent_actual_shape.min().into();
  let parent_actual_top_left_ipos: IPos =
    point_as!(parent_actual_top_left_pos, isize);
  let parent_actual_bottom_right_pos: U16Pos = parent_actual_shape.max().into();
  let parent_actual_bottom_right_ipos: IPos =
    point_as!(parent_actual_bottom_right_pos, isize);

  let top_left_pos: IPos = shape.min().into();
  let bottom_right_pos: IPos = shape.max().into();

  let actual_top_left_ipos: IPos = top_left_pos + parent_actual_top_left_ipos;
  let actual_top_left_x = num_traits::clamp(
    actual_top_left_ipos.x(),
    parent_actual_top_left_ipos.x(),
    parent_actual_bottom_right_ipos.x(),
  );
  let actual_top_left_y = num_traits::clamp(
    actual_top_left_ipos.y(),
    parent_actual_top_left_ipos.y(),
    parent_actual_bottom_right_ipos.y(),
  );
  let actual_top_left_pos: U16Pos =
    point!(actual_top_left_x as u16, actual_top_left_y as u16);
  // trace!(
  //   "actual_top_left_ipos:{:?}, actual_top_left_pos:{:?}",
  //   actual_top_left_ipos, actual_top_left_pos
  // );

  let actual_bottom_right_ipos: IPos =
    bottom_right_pos + parent_actual_top_left_ipos;
  let actual_bottom_right_x = num_traits::clamp(
    actual_bottom_right_ipos.x(),
    parent_actual_top_left_ipos.x(),
    parent_actual_bottom_right_ipos.x(),
  );
  let actual_bottom_right_y = num_traits::clamp(
    actual_bottom_right_ipos.y(),
    parent_actual_top_left_ipos.y(),
    parent_actual_bottom_right_ipos.y(),
  );
  let actual_bottom_right_pos: U16Pos =
    point!(actual_bottom_right_x as u16, actual_bottom_right_y as u16);

  let actual_isize = size!(
    (actual_bottom_right_pos.x() as isize) - (actual_top_left_pos.x() as isize),
    (actual_bottom_right_pos.y() as isize) - (actual_top_left_pos.y() as isize)
  );
  // trace!(
  //   "actual_isize:{:?}, actual_top_left_pos:{:?}",
  //   actual_isize, actual_top_left_pos
  // );

  let actual_shape = rect!(
    actual_top_left_pos.x(),
    actual_top_left_pos.y(),
    actual_top_left_pos.x() + actual_isize.width() as u16,
    actual_top_left_pos.y() + actual_isize.height() as u16
  );
  // trace!(
  //   "actual_isize:{:?}, actual_shape:{:?}",
  //   actual_isize, actual_shape
  // );

  actual_shape
}
