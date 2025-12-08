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
