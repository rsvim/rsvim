//! Internal tree node shape utils.

#![allow(clippy::let_and_return)]

use crate::prelude::*;

/// Convert relative shape to absolute, based on its parent's actual shape.
///
/// NOTE:
/// 1. If the widget doesn't have a parent, use the terminal shape as its
///    parent's shape.
/// 2. If the relative shape is outside of it's parent or the terminal, it will
///    be automatically bounded by it's parent.
pub fn convert_relative_to_absolute(
  shape: &IRect,
  parent_actual_shape: &U16Rect,
) -> U16Rect {
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
  let actual_max_x =
    num_traits::clamp(actual_max.x(), actual_min_x, parent_actual_max.x());
  let actual_max_y =
    num_traits::clamp(actual_max.y(), actual_min_y, parent_actual_max.y());

  let actual_shape =
    rect!(actual_min_x, actual_min_y, actual_max_x, actual_max_y);
  rect_as!(actual_shape, u16)
}

/// Truncate relative shape by its parent size.
pub fn truncate_shape(shape: &IRect, parent_actual_shape: &U16Rect) -> IRect {
  let parent_size = parent_actual_shape.size();
  let min_x = num_traits::clamp(shape.min().x, 0, parent_size.width() as isize);
  let min_y =
    num_traits::clamp(shape.min().y, 0, parent_size.height() as isize);
  let max_x =
    num_traits::clamp(shape.max().x, min_x, parent_size.width() as isize);
  let max_y =
    num_traits::clamp(shape.max().y, min_y, parent_size.height() as isize);
  rect!(min_x, min_y, max_x, max_y)
}

/// Bound child size by its parent actual size.
pub fn _bound_size(shape: &IRect, parent_actual_size: &U16Size) -> IRect {
  let top_left_pos: IPos = shape.min().into();

  // Truncate shape if size is larger than parent.
  let height =
    num_traits::clamp(shape.height(), 0, parent_actual_size.height() as isize);
  let width =
    num_traits::clamp(shape.width(), 0, parent_actual_size.width() as isize);
  rect!(
    top_left_pos.x(),
    top_left_pos.y(),
    top_left_pos.x() + width,
    top_left_pos.y() + height
  )
}

/// Bound child position by its parent actual shape.
/// When it's out of its parent, simply put it at the boundary.
pub fn _bound_pos(shape: &IRect, parent_actual_size: &U16Size) -> IRect {
  let top_left_pos: IPos = shape.min().into();
  let bottom_right_pos: IPos = shape.max().into();

  // X-axis
  let top_left_x = if top_left_pos.x() < 0 {
    // trace!("x-1, top_left_pos:{:?}", top_left_pos);
    0
  } else if bottom_right_pos.x() > parent_actual_size.width() as isize {
    // trace!(
    //   "x-2, bottom_right_pos:{:?}, parent_actual_shape.width:{:?}",
    //   bottom_right_pos,
    //   parent_actual_shape.width()
    // );
    let x_diff = num_traits::sign::abs_sub(
      bottom_right_pos.x(),
      parent_actual_size.width() as isize,
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
  } else if bottom_right_pos.y() > parent_actual_size.height() as isize {
    // trace!(
    //   "y-2, bottom_right_pos:{:?}, parent_actual_shape.height:{:?}",
    //   bottom_right_pos,
    //   parent_actual_shape.height()
    // );
    let y_diff = num_traits::sign::abs_sub(
      bottom_right_pos.y(),
      parent_actual_size.height() as isize,
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

/// Bound child shape (both position and size) by its parent actual shape.
pub fn bound_shape(shape: &IRect, parent_actual_size: &U16Size) -> IRect {
  let bounded = _bound_size(shape, parent_actual_size);
  _bound_pos(&bounded, parent_actual_size)
}
