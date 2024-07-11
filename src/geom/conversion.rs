//! The coordinate system conversions between children widgets and their parent widget.
//!
//! A widget's shape is always a rectangle (`rect`), and it's position is the top-left corner of
//! the rectangle.
//!
//! There're two kinds of positions in the widget tree: relative and absolute.
//! A relative position is always based on it's parent's position, an absolute position is always
//! based on the terminal device.
//!
//! There're two kinds of sizes in the widget tree: logic and actual.
//! A logic size can be infinite on it's parent's infinite canvas, an actual size is bounded inside
//! of it's parent's shape.
//!
//! By default we use the relative positions and logic sizes, omit the adjective "relative" and
//! "logic". When it comes to the absolute positions and actual sizes, we always add the adjective
//! "absolute" and "actual".
//!
//! For rectangle, when it presents an absolute position and an actual size, it's called an
//! `absolute_actual_rect`, when it presents a relative position and a logic size, it's simply
//! called a `rect`.
//!
//! Note:
//! 1. If a widget doesn't have a parent (i.e. the root widget), the relative position itself is
//!    already absolute.
//! 2. If the absolute position is outside of the terminal, it will be automatically bounded inside
//!    of the terminal's shape.
//! 3. A widget's actual size will be automatically truncated inside of it's parent's shape, or by
//!    the terminal's shape if it's the root widget already.

use crate::geom::{IPos, IRect, ISize, Size, U16Size, UPos, URect, USize};
use crate::{as_geo_point, as_geo_size};
use geo::point;
use std::cmp::{max, min};

/// Convert relative position to absolute position.
///
/// For example:
/// 1. If current widget's relative position is (-1, -1), parent's absolute position is (0, 0).
///    Then the current widget's absolute position is (0, 0).
/// 2. If current widget's relative position is (10, 10), parent's absolute position is (5, 5),
///    terminal's actual size is (5, 5). Then the current widget's absolution position is (5, 5).
pub fn to_absolute_pos(
  pos: IPos,
  parent_absolute_pos: Option<UPos>,
  terminal_size: U16Size,
) -> UPos {
  let p = match parent_absolute_pos {
    Some(pap) => pos + as_geo_point!(pap, isize),
    None => pos,
  };
  let x = min(max(p.x(), 0), terminal_size.width as isize);
  let y = min(max(p.y(), 0), terminal_size.height as isize);
  point!(x: x as usize, y: y as usize)
}

/// Convert logical size to actual size.
///
/// Note: If the widget doesn't have a parent, use the terminal size as its parent actual size.
///
/// For example:
/// 1. If current widget's logic size is (10, 10), relative position is (0, 0), parent's actual
///    size is (8, 8). Then the current widget's actual size is (8, 8).
/// 2. If current widget's logic size is (10, 10), relative position is (4, 4), parent's actual
///    size is (8, 8). Then the current widget's actual size is (4, 4).
pub fn to_actual_size(rect: IRect, parent_actual_size: USize) -> USize {
  let bottom_left: IPos = point! (x: max(rect.min().x, 0), y: max(rect.min().y, 0));
  let top_right: IPos = point! (x: min(rect.max().x, parent_actual_size.width as isize), y: max(rect.max().y, parent_actual_size.height as isize));
  let s = ISize::new(
    top_right.y() - bottom_left.y(),
    top_right.x() - bottom_left.y(),
  );
  as_geo_size!(s, usize)
}

/// Same with [to_actual_size](to_actual_size()), but a rect version.
/// The only difference is it returns `IRect` instead of `USize`.
pub fn to_actual_rect(rect: IRect, parent_actual_size: USize) -> IRect {
  let bottom_left = rect.min();
  let s = to_actual_size(rect, parent_actual_size);
  let top_right = point!(x: bottom_left.x + s.width as isize, y: bottom_left.y + s.height as isize);
  IRect::new(bottom_left.into(), top_right)
}

/// Same with [to_absolute_pos](to_absolute_pos()), but a rect version.
/// The only difference is it returns `URect` instead of `UPos`.
pub fn to_absolute_rect(
  rect: IRect,
  parent_absolute_pos: Option<UPos>,
  terminal_size: U16Size,
) -> URect {
  let bottom_left = rect.min();
  let p = to_absolute_pos(bottom_left.into(), parent_absolute_pos, terminal_size);
  let top_right = point!(x: p.x() + rect.width() as usize, y: p.y() + rect.height() as usize);
  URect::new(p, top_right)
}

/// Convert relative/logical rect to absolute/actual rect.
///
/// Note:
/// 1. This is a combined version of [to_absolute_pos](to_absolute_pos()) and
///    [to_actual_size](to_actual_size()).
/// 2. If the widget doesn't have a parent, use the terminal size as its parent actual size.
pub fn to_actual_absolute_rect(
  rect: IRect,
  parent_absolute_pos: Option<UPos>,
  parent_actual_size: USize,
  terminal_size: U16Size,
) -> URect {
  let pos = point!(x: rect.min().x, y: rect.min().y);
  let p = to_absolute_pos(pos, parent_absolute_pos, terminal_size);
  let s = to_actual_size(rect, parent_actual_size);
  URect::new(p, point!(x: p.x() + s.width, y: p.y() + s.height))
}

#[cfg(test)]
mod tests {
  use super::*;
  use geo::point;

  #[test]
  fn should_make_absolute_positions() {
    let inputs: Vec<(IPos, Option<UPos>, U16Size)> = vec![
      (
        point!(x: -1, y:-1),
        Some(point!(x: 0_usize, y: 0_usize)),
        U16Size::new(5_u16, 4_u16),
      ),
      (
        point!(x: 3, y:3),
        Some(point!(x: 4_usize, y: 4_usize)),
        U16Size::new(5_u16, 5_u16),
      ),
      (
        point!(x: 7, y:1),
        Some(point!(x: 3_usize, y: 6_usize)),
        U16Size::new(8_u16, 8_u16),
      ),
    ];
    let expects: Vec<UPos> = vec![
      point!(x: 0_usize, y: 0_usize),
      point!(x: 5_usize, y: 5_usize),
      point!(x: 8_usize, y: 7_usize),
    ];
    for (i, t) in inputs.iter().enumerate() {
      let actual = to_absolute_pos(t.0, t.1, t.2);
      let expect = expects[i];
      assert_eq!(actual, expect);
    }
  }

  #[test]
  fn should_make_actual_sizes() {
    let inputs: Vec<(IRect, USize)> = vec![(IRect::new((0, 0), (3, 5)), USize::new(4, 4))];
    let expects: Vec<USize> = vec![USize::new(3, 4)];
    for (i, t) in inputs.iter().enumerate() {
      let actual = to_actual_size(t.0, t.1);
      let expect = expects[i];
      assert_eq!(actual, expect);
    }
  }

  #[test]
  fn should_make_actual_rects() {
    let inputs: Vec<(IRect, USize)> = vec![(IRect::new((0, 0), (3, 5)), USize::new(4, 4))];
    let expects: Vec<IRect> = vec![IRect::new((0, 0), (3, 4))];
    for (i, t) in inputs.iter().enumerate() {
      let actual = to_actual_rect(t.0, t.1);
      let expect = expects[i];
      assert_eq!(actual, expect);
    }
  }

  #[test]
  fn should_make_actual_absolute_rects() {
    let inputs: Vec<(IRect, Option<UPos>, USize, U16Size)> = vec![(
      IRect::new((0, 0), (3, 5)),
      Some(point!(x: 0_usize, y: 0_usize)),
      USize::new(7, 8),
      U16Size::new(10, 10),
    )];
    let expects: Vec<URect> = vec![URect::new((0, 0), (3, 4))];
    for (i, t) in inputs.iter().enumerate() {
      let actual = to_actual_absolute_rect(t.0, t.1, t.2, t.3);
      let expect = expects[i];
      assert_eq!(actual, expect);
    }
  }
}
