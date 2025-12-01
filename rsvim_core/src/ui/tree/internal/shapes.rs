//! Internal tree relative shape and actual shape converters.

use crate::prelude::*;

/// Bound (truncate) relative shape based on its parent's size.
pub fn bound_shape(shape: &IRect, parent_actual_shape: &U16Rect) -> IRect {
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

/// Convert relative shape to actual shape, based on its parent's actual shape.
///
/// NOTE:
/// 1. If the widget doesn't have a parent, use the terminal shape as its
///    parent's shape.
/// 2. If the relative shape is outside of it's parent or the terminal, it will
///    be automatically bounded inside of it's parent or the terminal's shape.
pub fn convert_to_actual_shape(
  shape: &IRect,
  parent_actual_shape: &U16Rect,
) -> U16Rect {
  let parent_min_pos: U16Pos = parent_actual_shape.min().into();
  let parent_min_pos: IPos = point_as!(parent_min_pos, isize);
  let parent_max_pos: U16Pos = parent_actual_shape.max().into();
  let parent_max_pos: IPos = point_as!(parent_max_pos, isize);

  let min_pos: IPos = shape.min().into();
  let max_pos: IPos = shape.max().into();

  let actual_min_pos: IPos = min_pos + parent_min_pos;
  let actual_min_x = num_traits::clamp(
    actual_min_pos.x(),
    parent_min_pos.x(),
    parent_max_pos.x(),
  );
  let actual_min_y = num_traits::clamp(
    actual_min_pos.y(),
    parent_min_pos.y(),
    parent_max_pos.y(),
  );

  let actual_max_pos: IPos = max_pos + parent_min_pos;
  let actual_max_x =
    num_traits::clamp(actual_max_pos.x(), actual_min_x, parent_max_pos.x());
  let actual_max_y =
    num_traits::clamp(actual_max_pos.y(), actual_min_y, parent_max_pos.y());

  let actual_shape =
    rect!(actual_min_x, actual_min_y, actual_max_x, actual_max_y);
  let actual_shape = rect_as!(actual_shape, u16);

  actual_shape
}
