//! Geometry calculations for [Widget](crate::ui::widget::Widget): positions, sizes and rects.

use crate::geo::{IPos, IRect, U16Size, UPos, URect, USize};
use geo::{coord, point, Coord};
use std::cmp::{max, min};

/// Calculate absolute position from relative position and parent's absolute position.
///
/// Note: If the absolute position is outside of the terminal, it will be automatically bounded
/// inside of the terminal's shape.
///
/// For example:
/// 1. If current widget's relative position is (-1, -1), parent's absolute position is (0, 0).
///    Then the current widget's absolute position is (0, 0).
/// 2. If current widget's relative position is (10, 10), parent's actual size is (5, 5),
///    terminal's actual size is (5, 5). Then the current widget's absolution position is (5, 5).
fn make_absolute_pos(rect: IRect, parent_absolute_rect: URect, terminal_size: U16Size) -> URect {
  let p3: IPos =
    pos + point!(x: parent_absolute_pos.x() as isize, y: parent_absolute_pos.y() as isize);
  let x = min(max(p3.x(), 0), terminal_size.width as isize) as usize;
  let y = min(max(p3.y(), 0), terminal_size.height as isize) as usize;
  point!(x: x, y: y) as UPos
}

/// Calculate relative position from absolute position
/// ([`Widget::absolute_pos()`](crate::ui::widget::Widget::absolute_pos())) and parent's absolute
/// position ([`Widget::absolute_pos()`](crate::ui::widget::Widget::absolute_pos())).
fn make_pos(absolute_pos: UPos, parent_absolute_pos: UPos) -> IPos {
  point!(x: absolute_pos.x() as isize, y: absolute_pos.y() as isize)
    - point!(x: parent_absolute_pos.x() as isize, y: parent_absolute_pos.y() as isize)
}

/// Calculate actual size from logic size ([`Widget::size()`](crate::ui::widget::Widget::size()))
/// and parent's actual size ([`Widget::actual_size()`](crate::ui::widget::Widget::actual_size())).
///
/// Note: If the actual size is outside of the parent, it will be automatically truncated inside
/// of the parent's shape.
///
/// For example:
/// 1. If current widget's logic size is (10, 10), relative position is (0, 0), parent's actual
///    size is (8, 8). Then the current widget's actual size is (8, 8).
/// 2. If current widget's logic size is (10, 10), relative position is (4, 4), parent's actual
///    size is (8, 8). Then the current widget's actual size is (4, 4).
fn make_actual_size(rect: IRect, parent_actual_size: USize) -> USize {
  let top_left: Coord<isize> = coord! {x: max(rect.min().x, 0), y: max(rect.min().y, 0)};
  let bot_right: Coord<isize> = coord! {x: min(rect.max().x, parent_actual_size.width as isize), y: max(rect.max().y, parent_actual_size.height as isize)};
  USize::new(
    (bot_right.y - top_left.y) as usize,
    (bot_right.x - top_left.y) as usize,
  )
}

/// Calculate relative rect with actual size, from logic rect
/// ([`Widget::rect()`](crate::ui::widget::Widget::rect())) and parent's actual size
/// ([`Widget::actual_size()`](crate::ui::widget::Widget::actual_size())).
///
/// Note: This method works exactly same with [make_actual_size](make_actual_size()), except
/// it returns a `IRect` instead of a `USize`.
fn make_actual_rect(rect: IRect, parent_actual_size: USize) -> IRect {
  let top_left = rect.min();
  IRect::new(
    top_left.into(),
    point!(x: top_left.x + parent_actual_size.width as isize, y: top_left.y + parent_actual_size.height as isize),
  )
}

/// Calculate absolute rect with actual size, from logic rect
/// ([`Widget::rect()`](crate::ui::widget::Widget::rect())) and parent's absolute pos 
/// ([`Widget::absolute_pos()`](crate::ui::widget::Widget::absolute_pos())) and actual size 
/// ([`Widget::actual_size()`](crate::ui::widget::Widget::actual_size())).
///
/// Note: This method works exactly same with [make_absolute_pos](make_absolute_pos()) and
/// [make_actual_size](make_actual_size()), except it returns a `URect` instead of a `UPos`
/// or a `USize`.
fn make_actual_absolute_rect(rect: IRect, parent_absolute_pos: UPos, parent_actual_size: USize, terminal_size: U16Size) -> URect {
    let pos = point!(x: rect.min().x(), y: rect.min().y());
  let p1 = make_absolute_pos(pos, parent_absolute_pos, terminal_size);
  let s1 = make_actual_size(rect, parent_actual_size);
  URect::new(p1, point!(x: p1.x() + s1.width, y: p1.y() + s1.height))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn should_be_bounded_inside_of_terminal() {
      let inputs : Vec<(IPos, UPos, U16Size)> = vec![
          (point!(x: -1, y:-1), point!(x: 0_usize, y: 0_usize), ),
      ];
  }

  #[test]
  fn should_equal_on_buffer_new() {
    let sz = U16Size::new(1, 2);
    let b = Frame::new(sz, Cursor::default());
    assert_eq!(b.size.height, 1);
    assert_eq!(b.size.width, 2);
    assert_eq!(
      b.cells.len(),
      b.size.height as usize * b.size.width as usize
    );
    for c in b.cells.iter() {
      assert_eq!(c.symbol(), Cell::default().symbol());
    }
  }
}
