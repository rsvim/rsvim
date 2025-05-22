//! Cursor operations.

use crate::buf::Buffer;
use crate::lock;
use crate::state::ops::Operation;
use crate::ui::tree::*;
use crate::ui::widget::window::{
  CursorViewport, CursorViewportArc, Viewport, ViewportArc, ViewportSearchAnchorDirection, Window,
};

use tracing::trace;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// Cursor move direction.
pub enum CursorMoveDirection {
  Up,
  Down,
  Left,
  Right,
}

fn _cursor_direction(by_x: isize, by_y: isize) -> CursorMoveDirection {
  if by_y > 0 {
    CursorMoveDirection::Down
  } else if by_y < 0 {
    CursorMoveDirection::Up
  } else if by_x > 0 {
    CursorMoveDirection::Right
  } else {
    CursorMoveDirection::Left
  }
}

/// Normalize `Operation::CursorMove*` to `Operation::CursorMoveBy((x,y))`.
pub fn normalize_as_cursor_move_by(
  op: Operation,
  cursor_char_idx: usize,
  cursor_line_idx: usize,
) -> (isize, isize, CursorMoveDirection) {
  match op {
    Operation::CursorMoveLeftBy(n) => (-(n as isize), 0, CursorMoveDirection::Left),
    Operation::CursorMoveRightBy(n) => (n as isize, 0, CursorMoveDirection::Right),
    Operation::CursorMoveUpBy(n) => (0, -(n as isize), CursorMoveDirection::Up),
    Operation::CursorMoveDownBy(n) => (0, n as isize, CursorMoveDirection::Down),
    Operation::CursorMoveTo((x, y)) => {
      let x = (x as isize) - (cursor_char_idx as isize);
      let y = (y as isize) - (cursor_line_idx as isize);
      (x, y, _cursor_direction(x, y))
    }
    Operation::CursorMoveBy((x, y)) => (x, y, _cursor_direction(x, y)),
    _ => unreachable!(),
  }
}

/// Normalize `Operation::CursorMove*` to `Operation::CursorMoveTo((x,y))`.
pub fn normalize_as_cursor_move_to(
  op: Operation,
  cursor_char_idx: usize,
  cursor_line_idx: usize,
) -> (usize, usize, CursorMoveDirection) {
  match op {
    Operation::CursorMoveLeftBy(n) => {
      let x = cursor_char_idx.saturating_sub(n);
      let y = cursor_line_idx;
      (x, y, CursorMoveDirection::Left)
    }
    Operation::CursorMoveRightBy(n) => {
      let x = cursor_char_idx.saturating_add(n);
      let y = cursor_line_idx;
      (x, y, CursorMoveDirection::Right)
    }
    Operation::CursorMoveUpBy(n) => {
      let x = cursor_char_idx;
      let y = cursor_line_idx.saturating_sub(n);
      (x, y, CursorMoveDirection::Up)
    }
    Operation::CursorMoveDownBy(n) => {
      let x = cursor_char_idx;
      let y = cursor_line_idx.saturating_add(n);
      (x, y, CursorMoveDirection::Down)
    }
    Operation::CursorMoveBy((x, y)) => {
      let to_x = std::cmp::max(0, (cursor_char_idx as isize) + x) as usize;
      let to_y = std::cmp::max(0, (cursor_line_idx as isize) + y) as usize;
      (to_x, to_y, _cursor_direction(x, y))
    }
    Operation::CursorMoveTo((x, y)) => {
      let by_x = (x as isize) - (cursor_char_idx as isize);
      let by_y = (y as isize) - (cursor_line_idx as isize);
      (x, y, _cursor_direction(by_x, by_y))
    }
    _ => unreachable!(),
  }
}

/// Normalize `Operation::WindowScroll*` to `Operation::WindowScrollBy((x,y))`.
pub fn normalize_as_window_scroll_by(
  op: Operation,
  viewport_start_column_idx: usize,
  viewport_start_line_idx: usize,
) -> (isize, isize) {
  match op {
    Operation::WindowScrollLeftBy(n) => (-(n as isize), 0),
    Operation::WindowScrollRightBy(n) => (n as isize, 0),
    Operation::WindowScrollUpBy(n) => (0, -(n as isize)),
    Operation::WindowScrollDownBy(n) => (0, n as isize),
    Operation::WindowScrollTo((x, y)) => {
      let x = (x as isize) - (viewport_start_column_idx as isize);
      let y = (y as isize) - (viewport_start_line_idx as isize);
      (x, y)
    }
    Operation::WindowScrollBy((x, y)) => (x, y),
    _ => unreachable!(),
  }
}

/// Calculate new cursor viewport by `Operation::CursorMove*` operations.
///
/// It returns new cursor viewport if the operation is valid, returns `None` if the cursor cannot
/// move to the position.
///
/// # Panics
///
/// It panics if the operation is not a `Operation::CursorMove*` operation.
pub fn cursor_move(
  viewport: &Viewport,
  cursor_viewport: &CursorViewport,
  buffer: &Buffer,
  op: Operation,
) -> Option<CursorViewportArc> {
  let (by_chars, by_lines, _) =
    normalize_as_cursor_move_by(op, cursor_viewport.char_idx(), cursor_viewport.line_idx());

  let cursor_move_result =
    _raw_cursor_move_by(viewport, cursor_viewport, buffer, by_chars, by_lines);

  if let Some((line_idx, char_idx)) = cursor_move_result {
    let new_cursor_viewport = CursorViewport::from_position(viewport, buffer, line_idx, char_idx);
    let new_cursor_viewport = CursorViewport::to_arc(new_cursor_viewport);
    // New cursor position
    Some(new_cursor_viewport)
  } else {
    // Or, just do nothing, stay at where you are
    None
  }
}

fn _adjust_cursor_char_idx_on_vertical_motion(
  buffer: &Buffer,
  cursor_line_idx: usize,
  cursor_char_idx: usize,
  line_idx: usize,
) -> usize {
  let cursor_col_idx = buffer.width_before(cursor_line_idx, cursor_char_idx);
  let char_idx = match buffer.char_after(line_idx, cursor_col_idx) {
    Some(char_idx) => char_idx,
    None => {
      debug_assert!(buffer.get_rope().get_line(line_idx).is_some());
      debug_assert!(buffer.get_rope().line(line_idx).len_chars() > 0);
      buffer.last_visible_char_on_line(line_idx).unwrap()
    }
  };
  trace!(
    "cursor_line_idx:{},cursor_col_idx:{},line_idx:{},char_idx:{}",
    cursor_line_idx, cursor_col_idx, line_idx, char_idx
  );
  char_idx
}

// Returns the `line_idx`/`char_idx` for new cursor position.
pub fn _raw_cursor_move_by(
  viewport: &Viewport,
  cursor_viewport: &CursorViewport,
  buffer: &Buffer,
  chars: isize,
  lines: isize,
) -> Option<(usize, usize)> {
  let cursor_line_idx = cursor_viewport.line_idx();
  let cursor_char_idx = cursor_viewport.char_idx();
  let line_idx =
    _bounded_raw_cursor_move_y_by(viewport, cursor_line_idx, cursor_char_idx, buffer, lines);

  // If `line_idx` doesn't exist, or line is empty.
  match buffer.get_rope().get_line(line_idx) {
    Some(line) => {
      if line.len_chars() == 0 {
        return Some((line_idx, 0_usize));
      }
    }
    None => return None,
  }

  let char_idx =
    _adjust_cursor_char_idx_on_vertical_motion(buffer, cursor_line_idx, cursor_char_idx, line_idx);
  let char_idx = _bounded_raw_cursor_move_x_by(viewport, line_idx, char_idx, buffer, chars);

  Some((line_idx, char_idx))
}

fn _bounded_raw_cursor_move_y_by(
  viewport: &Viewport,
  cursor_line_idx: usize,
  _cursor_char_idx: usize,
  _buffer: &Buffer,
  lines: isize,
) -> usize {
  if lines < 0 {
    let n = -lines as usize;
    cursor_line_idx.saturating_sub(n)
  } else {
    let n = lines as usize;
    let expected_line_idx = cursor_line_idx.saturating_add(n);
    let last_line_idx = viewport.end_line_idx().saturating_sub(1);
    trace!(
      "base_line_idx:{:?},expected:{:?},last_line_idx:{:?}",
      cursor_line_idx, expected_line_idx, last_line_idx
    );
    std::cmp::min(expected_line_idx, last_line_idx)
  }
}

fn _bounded_raw_cursor_move_x_by(
  viewport: &Viewport,
  cursor_line_idx: usize,
  cursor_char_idx: usize,
  buffer: &Buffer,
  chars: isize,
) -> usize {
  if chars < 0 {
    let n = -chars as usize;
    cursor_char_idx.saturating_sub(n)
  } else {
    let n = chars as usize;
    let expected = cursor_char_idx.saturating_add(n);
    let upper_bounded = {
      debug_assert!(viewport.lines().contains_key(&cursor_line_idx));
      let line_viewport = viewport.lines().get(&cursor_line_idx).unwrap();
      let (_last_row_idx, last_row_viewport) = line_viewport.rows().last_key_value().unwrap();
      let last_char_on_row = last_row_viewport.end_char_idx().saturating_sub(1);
      trace!(
        "cursor_char_idx:{}, expected:{}, last_row_viewport:{:?}, last_char_on_row:{}",
        cursor_char_idx, expected, last_row_viewport, last_char_on_row
      );
      buffer
        .last_visible_char_on_line_since(cursor_line_idx, last_char_on_row)
        .unwrap()
    };
    std::cmp::min(expected, upper_bounded)
  }
}
