//! Internal tree node shape utils.

#![allow(clippy::let_and_return)]

use crate::prelude::*;

/// Convert relative shape to absolute shape, based on its parent's actual shape.
///
/// NOTE:
/// 1. If the node doesn't have a parent, it uses the terminal's actual shape
///    as its parent's shape.
/// 2. If the relative shape is outside of it's parent or the terminal, it will
///    be automatically bounded inside it's parent shape.
pub fn make_actual_shape(
  shape: &IRect,
  parent_actual_shape: &U16Rect,
) -> U16Rect {
  // trace!(
  //   "shape:{:?}, parent_actual_shape:{:?}",
  //   shape, parent_actual_shape
  // );
  let parent_actual_min: U16Pos = parent_actual_shape.min().into();
  let parent_actual_min: IPos = point_as!(parent_actual_min, isize);
  let parent_actual_max: U16Pos = parent_actual_shape.max().into();
  let parent_actual_max: IPos = point_as!(parent_actual_max, isize);

  let min_pos: IPos = shape.min().into();
  let max_pos: IPos = shape.max().into();

  let actual_min: IPos = min_pos + parent_actual_min;
  let actual_min_x = num_traits::clamp(
    actual_min.x(),
    parent_actual_min.x(),
    parent_actual_max.x(),
  );
  let actual_min_y = num_traits::clamp(
    actual_min.y(),
    parent_actual_min.y(),
    parent_actual_max.y(),
  );

  let actual_max: IPos = max_pos + parent_actual_min;
  let actual_max_x = num_traits::clamp(
    actual_max.x(),
    parent_actual_min.x(),
    parent_actual_max.x(),
  );
  let actual_max_y = num_traits::clamp(
    actual_max.y(),
    parent_actual_min.y(),
    parent_actual_max.y(),
  );

  let actual_isize =
    size!(actual_max_x - actual_min_x, actual_max_y - actual_min_y);

  let actual_shape = rect!(
    actual_min_x,
    actual_min_y,
    actual_min_x + actual_isize.width(),
    actual_min_y + actual_isize.height()
  );
  // trace!(
  //   "actual_isize:{:?}, actual_shape:{:?}",
  //   actual_isize, actual_shape
  // );

  rect_as!(actual_shape, u16)
}

/// Bound (truncate) child size by its parent actual size.
pub fn bound_size(shape: &IRect, parent_actual_shape: &U16Rect) -> IRect {
  let top_left_pos: IPos = shape.min().into();

  // Truncate shape if size is larger than parent.
  let height =
    num_traits::clamp(shape.height(), 0, parent_actual_shape.height() as isize);
  let width =
    num_traits::clamp(shape.width(), 0, parent_actual_shape.width() as isize);
  rect!(
    top_left_pos.x(),
    top_left_pos.y(),
    top_left_pos.x() + width,
    top_left_pos.y() + height
  )
}

/// Bound child position by its parent actual shape.
/// When it's out of its parent, simply put it at the boundary.
pub fn bound_position(shape: &IRect, parent_actual_shape: &U16Rect) -> IRect {
  let top_left_pos: IPos = shape.min().into();
  let bottom_right_pos: IPos = shape.max().into();

  // X-axis
  let top_left_x = if top_left_pos.x() < 0 {
    // trace!("x-1, top_left_pos:{:?}", top_left_pos);
    0
  } else if bottom_right_pos.x() > parent_actual_shape.width() as isize {
    // trace!(
    //   "x-2, bottom_right_pos:{:?}, parent_actual_shape.width:{:?}",
    //   bottom_right_pos,
    //   parent_actual_shape.width()
    // );
    let x_diff = num_traits::sign::abs_sub(
      bottom_right_pos.x(),
      parent_actual_shape.width() as isize,
    );
    let result = top_left_pos.x() - x_diff;
    // trace!("x-2, x_diff:{:?}, result:{:?}", x_diff, result);
    result
  } else {
    // trace!("x-3, top_left_pos:{:?}", top_left_pos);
    top_left_pos.x()
  };

  // Y-axis
  let top_left_y = if top_left_pos.y() < 0 {
    // trace!("y-1, top_left_pos:{:?}", top_left_pos);
    0
  } else if bottom_right_pos.y() > parent_actual_shape.height() as isize {
    // trace!(
    //   "y-2, bottom_right_pos:{:?}, parent_actual_shape.height:{:?}",
    //   bottom_right_pos,
    //   parent_actual_shape.height()
    // );
    let y_diff = num_traits::sign::abs_sub(
      bottom_right_pos.y(),
      parent_actual_shape.height() as isize,
    );
    let result = top_left_pos.y() - y_diff;
    // trace!("y-2, y_diff:{:?}, result:{:?}", y_diff, result);
    result
  } else {
    // trace!("y-3, top_left_pos:{:?}", top_left_pos);
    top_left_pos.y()
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
