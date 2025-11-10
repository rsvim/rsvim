//! Internal tree node shape utils.

#![allow(clippy::let_and_return)]

use crate::point;
use crate::point_as;
use crate::prelude::*;
use crate::rect;
use crate::size;
use num_traits::clamp;

/// Convert (relative/logical) shape to actual shape, based on its parent's actual shape.
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
  let parent_actual_top_left_pos: U16Pos = parent_actual_shape.min();
  let parent_actual_top_left_ipos: IPos =
    point_as!(parent_actual_top_left_pos, isize);
  let parent_actual_bottom_right_pos: U16Pos = parent_actual_shape.max();
  let parent_actual_bottom_right_ipos: IPos =
    point_as!(parent_actual_bottom_right_pos, isize);

  let top_left_pos: IPos = shape.min();
  let bottom_right_pos: IPos = shape.max();

  let actual_top_left_ipos: IPos = top_left_pos + parent_actual_top_left_ipos;

  let actual_top_left_x = clamp(
    actual_top_left_ipos.x,
    parent_actual_top_left_ipos.x,
    parent_actual_bottom_right_ipos.x,
  );
  let actual_top_left_y = clamp(
    actual_top_left_ipos.y,
    parent_actual_top_left_ipos.y,
    parent_actual_bottom_right_ipos.y,
  );

  let actual_top_left_pos: U16Pos =
    point!(actual_top_left_x as u16, actual_top_left_y as u16);
  // trace!(
  //   "actual_top_left_ipos:{:?}, actual_top_left_pos:{:?}",
  //   actual_top_left_ipos, actual_top_left_pos
  // );

  let actual_bottom_right_ipos: IPos =
    bottom_right_pos + parent_actual_top_left_ipos;
  let actual_bottom_right_x = clamp(
    actual_bottom_right_ipos.x,
    parent_actual_top_left_ipos.x,
    parent_actual_bottom_right_ipos.x,
  );
  let actual_bottom_right_y = clamp(
    actual_bottom_right_ipos.y,
    parent_actual_top_left_ipos.y,
    parent_actual_bottom_right_ipos.y,
  );
  let actual_bottom_right_pos: U16Pos =
    point!(actual_bottom_right_x as u16, actual_bottom_right_y as u16);

  let actual_isize = size!(
    (actual_bottom_right_pos.x as isize) - (actual_top_left_pos.x as isize),
    (actual_bottom_right_pos.y as isize) - (actual_top_left_pos.y as isize)
  );
  // trace!(
  //   "actual_isize:{:?}, actual_top_left_pos:{:?}",
  //   actual_isize, actual_top_left_pos
  // );

  let actual_shape = rect!(
    actual_top_left_pos.x,
    actual_top_left_pos.y,
    actual_top_left_pos.x + actual_isize.width as u16,
    actual_top_left_pos.y + actual_isize.height as u16
  );
  // trace!(
  //   "actual_isize:{:?}, actual_shape:{:?}",
  //   actual_isize, actual_shape
  // );

  actual_shape
}

/// Bound (truncate) child size by its parent actual size.
pub fn bound_size(shape: &IRect, parent_actual_shape: &U16Rect) -> IRect {
  let top_left_pos: IPos = shape.min();

  // Truncate shape if size is larger than parent.
  let height = clamp(shape.height(), 0, parent_actual_shape.height() as isize);
  let width = clamp(shape.width(), 0, parent_actual_shape.width() as isize);
  rect!(
    top_left_pos.x,
    top_left_pos.y,
    top_left_pos.x + width,
    top_left_pos.y + height
  )
}

/// Bound child position by its parent actual shape.
/// When it's out of its parent, simply put it at the boundary.
pub fn bound_position(shape: &IRect, parent_actual_shape: &U16Rect) -> IRect {
  let top_left_pos: IPos = shape.min();
  let bottom_right_pos: IPos = shape.max();

  // X-axis
  let top_left_x = if top_left_pos.x < 0 {
    // trace!("x-1, top_left_pos:{:?}", top_left_pos);
    0
  } else if bottom_right_pos.x > parent_actual_shape.width() as isize {
    // trace!(
    //   "x-2, bottom_right_pos:{:?}, parent_actual_shape.width:{:?}",
    //   bottom_right_pos,
    //   parent_actual_shape.width()
    // );
    let x_diff = num_traits::sign::abs_sub(
      bottom_right_pos.x,
      parent_actual_shape.width() as isize,
    );
    let result = top_left_pos.x - x_diff;
    // trace!("x-2, x_diff:{:?}, result:{:?}", x_diff, result);
    result
  } else {
    // trace!("x-3, top_left_pos:{:?}", top_left_pos);
    top_left_pos.x
  };

  // Y-axis
  let top_left_y = if top_left_pos.y < 0 {
    // trace!("y-1, top_left_pos:{:?}", top_left_pos);
    0
  } else if bottom_right_pos.y > parent_actual_shape.height() as isize {
    // trace!(
    //   "y-2, bottom_right_pos:{:?}, parent_actual_shape.height:{:?}",
    //   bottom_right_pos,
    //   parent_actual_shape.height()
    // );
    let y_diff = num_traits::sign::abs_sub(
      bottom_right_pos.y,
      parent_actual_shape.height() as isize,
    );
    let result = top_left_pos.y - y_diff;
    // trace!("y-2, y_diff:{:?}, result:{:?}", y_diff, result);
    result
  } else {
    // trace!("y-3, top_left_pos:{:?}", top_left_pos);
    top_left_pos.y
  };

  rect!(
    top_left_x,
    top_left_y,
    top_left_x + shape.width(),
    top_left_y + shape.height()
  )
}

/// Bound (truncate) child shape (both position and size) by its parent actual shape.
///
/// NOTE: This is a wrapper on both [`bound_size`] and [`bound_position`].
pub fn bound_shape(shape: &IRect, parent_actual_shape: &U16Rect) -> IRect {
  let bounded = bound_size(shape, parent_actual_shape);
  bound_position(&bounded, parent_actual_shape)
}
