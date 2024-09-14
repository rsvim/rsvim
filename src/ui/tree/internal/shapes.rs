//! Internal tree node shape utils.

#![allow(clippy::let_and_return)]

use geo::point;
use std::cmp::{max, min};
use tracing::debug;

use crate::cart::{IPos, IRect, ISize, U16Pos, U16Rect};
use crate::geo_point_as;

/// Convert (relative/logical) shape to actual shape, based on its parent's actual shape.
///
/// NOTE:
/// 1. If the widget doesn't have a parent, use the terminal shape as its parent's shape.
/// 2. If the relative/logical shape is outside of it's parent or the terminal, it will be
///    automatically bounded inside of it's parent or the terminal's shape.
pub fn make_actual_shape(shape: IRect, parent_actual_shape: U16Rect) -> U16Rect {
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

/// Bound (truncate) child size by its parent actual size.
pub fn bound_size(shape: IRect, parent_actual_shape: U16Rect) -> IRect {
  use std::cmp::{max, min};

  let top_left_pos: IPos = shape.min().into();

  // Truncate shape if size is larger than parent.
  let height = max(
    min(shape.height(), parent_actual_shape.height() as isize),
    0,
  );
  let width = max(min(shape.width(), parent_actual_shape.width() as isize), 0);
  IRect::new(
    top_left_pos,
    point!(x: top_left_pos.x() + width, y: top_left_pos.y() + height),
  )
}

/// Bound child position by its parent actual shape.
/// When it's out of its parent, simply put it at the boundary.
pub fn bound_position(shape: IRect, parent_actual_shape: U16Rect) -> IRect {
  let top_left_pos: IPos = shape.min().into();
  let bottom_right_pos: IPos = shape.max().into();

  // X-axis
  let top_left_x = if top_left_pos.x() < 0 {
    debug!("x-1, top_left_pos:{:?}", top_left_pos);
    0
  } else if bottom_right_pos.x() > parent_actual_shape.width() as isize {
    debug!(
      "x-2, bottom_right_pos:{:?}, parent_actual_shape.width:{:?}",
      bottom_right_pos,
      parent_actual_shape.width()
    );
    let x_diff =
      num_traits::sign::abs_sub(bottom_right_pos.x(), parent_actual_shape.width() as isize);
    let result = top_left_pos.x() - x_diff;
    debug!("x-2, x_diff:{:?}, result:{:?}", x_diff, result);
    result
  } else {
    debug!("x-3, top_left_pos:{:?}", top_left_pos);
    top_left_pos.x()
  };

  // Y-axis
  let top_left_y = if top_left_pos.y() < 0 {
    debug!("y-1, top_left_pos:{:?}", top_left_pos);
    0
  } else if bottom_right_pos.y() > parent_actual_shape.height() as isize {
    debug!(
      "y-2, bottom_right_pos:{:?}, parent_actual_shape.height:{:?}",
      bottom_right_pos,
      parent_actual_shape.height()
    );
    let y_diff =
      num_traits::sign::abs_sub(bottom_right_pos.y(), parent_actual_shape.height() as isize);
    let result = top_left_pos.y() - y_diff;
    debug!("y-2, y_diff:{:?}, result:{:?}", y_diff, result);
    result
  } else {
    debug!("y-3, top_left_pos:{:?}", top_left_pos);
    top_left_pos.y()
  };

  IRect::new(
    (top_left_x, top_left_y),
    (top_left_x + shape.width(), top_left_y + shape.height()),
  )
}

/// Bound (truncate) child shape (both position and size) by its parent actual shape.
///
/// NOTE: This is a wrapper on both [`bound_size`] and [`bound_position`].
pub fn bound_shape(shape: IRect, parent_actual_shape: U16Rect) -> IRect {
  let bounded = bound_size(shape, parent_actual_shape);
  bound_position(bounded, parent_actual_shape)
}

#[cfg(test)]
mod tests {
  use std::cmp::min;
  use std::sync::Once;
  use tracing::info;

  use crate::cart::{IRect, U16Rect};
  // use crate::test::log::init as test_log_init;

  // static INIT: Once = Once::new();

  use super::*;

  #[test]
  fn make_actual_shapes1() {
    // INIT.call_once(test_log_init);

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
          let actual = make_actual_shape(*t, input_actual_parent_shape);
          info!("expect:{:?}, actual:{:?}", expect, actual);
          assert_eq!(actual, expect);
        }
      }
    }
  }

  #[test]
  fn make_actual_shapes2() {
    // INIT.call_once(test_log_init);

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
      let actual = make_actual_shape(p.0, p.1);
      let expect = expects[i];
      info!(
        "i:{:?}, input:{:?}, actual:{:?}, expect:{:?}",
        i, p, actual, expect
      );
      assert_eq!(actual, expect);
    }
  }

  #[test]
  fn bound_size1() {
    // INIT.call_once(test_log_init);

    let inputs: Vec<(IRect, U16Rect)> = vec![
      (IRect::new((0, 0), (7, 8)), U16Rect::new((0, 0), (10, 10))),
      (IRect::new((3, 2), (10, 10)), U16Rect::new((0, 0), (10, 10))),
      (IRect::new((3, -2), (12, 9)), U16Rect::new((0, 0), (10, 10))),
      (IRect::new((3, 1), (12, 9)), U16Rect::new((0, 0), (0, 0))),
      (IRect::new((-1, -1), (1, 1)), U16Rect::new((0, 0), (0, 0))),
    ];
    let expects: Vec<IRect> = vec![
      IRect::new((0, 0), (7, 8)),
      IRect::new((3, 2), (10, 10)),
      IRect::new((3, -2), (12, 8)),
      IRect::new((3, 1), (3, 1)),
      IRect::new((-1, -1), (-1, -1)),
    ];
    for (i, p) in inputs.iter().enumerate() {
      let actual = bound_size(p.0, p.1);
      let expect = expects[i];
      info!(
        "i:{:?}, input:{:?}, actual:{:?}, expect:{:?}",
        i, p, actual, expect
      );
      assert!(actual == expect);
    }
  }

  #[test]
  fn bound_position1() {
    // INIT.call_once(test_log_init);

    let inputs: Vec<(IRect, U16Rect)> = vec![
      (IRect::new((0, 0), (7, 8)), U16Rect::new((0, 0), (10, 10))),
      (IRect::new((3, 2), (12, 11)), U16Rect::new((0, 0), (10, 10))),
      (IRect::new((7, -2), (13, 8)), U16Rect::new((0, 0), (10, 10))),
      (IRect::new((-8, 8), (-3, 16)), U16Rect::new((3, 7), (8, 15))),
      (
        IRect::new((-5, 19), (-3, 21)),
        U16Rect::new((10, 15), (15, 20)),
      ),
    ];
    let expects: Vec<IRect> = vec![
      IRect::new((0, 0), (7, 8)),
      IRect::new((1, 1), (10, 10)),
      IRect::new((4, 0), (10, 10)),
      IRect::new((0, 0), (5, 8)),
      IRect::new((0, 3), (2, 5)),
    ];
    for (i, p) in inputs.iter().enumerate() {
      let actual = bound_position(p.0, p.1);
      let expect = expects[i];
      info!(
        "i:{:?}, input:{:?}, actual:{:?}, expect:{:?}",
        i, p, actual, expect
      );
      assert!(actual == expect);
    }
  }

  #[test]
  fn bound_shape1() {
    // INIT.call_once(test_log_init);

    let inputs: Vec<(IRect, U16Rect)> = vec![
      (IRect::new((0, 0), (7, 8)), U16Rect::new((0, 0), (10, 10))),
      (IRect::new((3, 2), (17, 11)), U16Rect::new((0, 0), (10, 10))),
      (IRect::new((7, -2), (13, 8)), U16Rect::new((0, 0), (5, 5))),
      (IRect::new((-5, 8), (3, 16)), U16Rect::new((3, 7), (13, 17))),
      (
        IRect::new((-5, 17), (1, 21)),
        U16Rect::new((10, 15), (18, 23)),
      ),
    ];
    let expects: Vec<IRect> = vec![
      IRect::new((0, 0), (7, 8)),
      IRect::new((0, 1), (10, 10)),
      IRect::new((0, 0), (5, 5)),
      IRect::new((0, 2), (8, 10)),
      IRect::new((0, 4), (6, 8)),
    ];
    for (i, p) in inputs.iter().enumerate() {
      let actual = bound_shape(p.0, p.1);
      let expect = expects[i];
      info!(
        "i:{:?}, input:{:?}, actual:{:?}, expect:{:?}",
        i, p, actual, expect
      );
      assert!(actual == expect);
    }
  }
}
