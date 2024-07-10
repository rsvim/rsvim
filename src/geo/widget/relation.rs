//! The relationship calculations between children widgets and their parent widget.
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

use crate::geo::{IPos, IRect, ISize, Size, U16Size, UPos, URect, USize};
use crate::{as_geo_point, as_geo_size};
use geo::point;
use std::cmp::{max, min};

/// Calculate absolute position from relative position and parent's absolute position.
///
/// Note:
/// 1. If the widget doesn't have a parent, i.e. it's the root widget, then the relative position
///    is absolute position itself.
/// 2. If the absolute position is outside of the terminal, it will be automatically bounded inside
///    of the terminal's shape.
///
/// For example:
/// 1. If current widget's relative position is (-1, -1), parent's absolute position is (0, 0).
///    Then the current widget's absolute position is (0, 0).
/// 2. If current widget's relative position is (10, 10), parent's absolute position is (5, 5),
///    terminal's actual size is (5, 5). Then the current widget's absolution position is (5, 5).
pub fn make_absolute_pos(
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

/// Calculate relative position from absolute position and parent's absolute position.
pub fn make_pos(absolute_pos: UPos, parent_absolute_pos: UPos) -> IPos {
  as_geo_point!(absolute_pos, isize) - as_geo_point!(parent_absolute_pos, isize)
}

/// Calculate actual size from relative logic rect and parent's actual size.
///
/// Note: If the actual size is outside of the parent, it will be automatically truncated inside
/// of the parent's shape.
///
/// For example:
/// 1. If current widget's logic size is (10, 10), relative position is (0, 0), parent's actual
///    size is (8, 8). Then the current widget's actual size is (8, 8).
/// 2. If current widget's logic size is (10, 10), relative position is (4, 4), parent's actual
///    size is (8, 8). Then the current widget's actual size is (4, 4).
pub fn make_actual_size(rect: IRect, parent_actual_size: USize) -> USize {
  let top_left: IPos = point! (x: max(rect.min().x, 0), y: max(rect.min().y, 0));
  let bot_right: IPos = point! (x: min(rect.max().x, parent_actual_size.width as isize), y: max(rect.max().y, parent_actual_size.height as isize));
  let s = ISize::new(bot_right.y() - top_left.y(), bot_right.x() - top_left.y());
  as_geo_size!(s, usize)
}

/// Same with [make_actual_size](make_actual_size()), but a rect version.
/// The only difference is it returns `IRect` instead of `USize`.
pub fn make_actual_rect(rect: IRect, parent_actual_size: USize) -> IRect {
  let top_left = rect.min();
  let s = make_actual_size(rect, parent_actual_size);
  let bottom_right = point!(x: top_left.x + s.width as isize, y: top_left.y + s.height as isize);
  IRect::new(top_left.into(), bottom_right)
}

/// Calculate absolute actual rect, from relative logic rect and parent's absolute actual rect.
///
/// Note: This method works like a combined version of both [make_absolute_pos](make_absolute_pos())
/// and [make_actual_size](make_actual_size()).
pub fn make_actual_absolute_rect(
  rect: IRect,
  parent_absolute_actual_rect: URect,
  terminal_size: U16Size,
) -> URect {
  let pos = point!(x: rect.min().x, y: rect.min().y);
  let parent_absolute_pos =
    point!(x: parent_absolute_actual_rect.min().x, y: parent_absolute_actual_rect.min().y);
  let p = make_absolute_pos(pos, Some(parent_absolute_pos), terminal_size);
  let parent_actual_size = USize::from(parent_absolute_actual_rect);
  let s = make_actual_size(rect, parent_actual_size);
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
      let actual = make_absolute_pos(t.0, t.1, t.2);
      let expect = expects[i];
      assert_eq!(actual, expect);
    }
  }

  #[test]
  fn should_make_relative_positions() {
    let inputs: Vec<(UPos, UPos)> = vec![
      (
        point!(x: 1_usize, y:1_usize),
        point!(x: 0_usize, y: 0_usize),
      ),
      (
        point!(x: 1_usize, y:1_usize),
        point!(x: 5_usize, y: 6_usize),
      ),
    ];
    let expects: Vec<IPos> = vec![point!(x: 1_isize, y: 1_isize), point!(x: -4, y: -5)];
    for (i, t) in inputs.iter().enumerate() {
      let actual = make_pos(t.0, t.1);
      let expect = expects[i];
      assert_eq!(actual, expect);
    }
  }

  #[test]
  fn should_make_actual_sizes() {
    let inputs: Vec<(IRect, USize)> = vec![(IRect::new((0, 0), (3, 5)), USize::new(4, 4))];
    let expects: Vec<USize> = vec![USize::new(3, 4)];
    for (i, t) in inputs.iter().enumerate() {
      let actual = make_actual_size(t.0, t.1);
      let expect = expects[i];
      assert_eq!(actual, expect);
    }
  }

  #[test]
  fn should_make_actual_rects() {
    let inputs: Vec<(IRect, USize)> = vec![(IRect::new((0, 0), (3, 5)), USize::new(4, 4))];
    let expects: Vec<IRect> = vec![IRect::new((0, 0), (3, 4))];
    for (i, t) in inputs.iter().enumerate() {
      let actual = make_actual_rect(t.0, t.1);
      let expect = expects[i];
      assert_eq!(actual, expect);
    }
  }

  #[test]
  fn should_make_actual_absolute_rects() {
    let inputs: Vec<(IRect, URect, U16Size)> = vec![(
      IRect::new((0, 0), (3, 5)),
      URect::new((0, 0), (4, 4)),
      U16Size::new(10, 10),
    )];
    let expects: Vec<URect> = vec![URect::new((0, 0), (3, 4))];
    for (i, t) in inputs.iter().enumerate() {
      let actual = make_actual_absolute_rect(t.0, t.1, t.2);
      let expect = expects[i];
      assert_eq!(actual, expect);
    }
  }
}
