//! Cursor operations.

use crate::buf::Buffer;
use crate::state::ops::Operation;
use crate::ui::tree::*;
use crate::ui::widget::window::{CursorViewport, CursorViewportArc, Viewport, ViewportArc, Window};

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

/// Same with [`normalize_as_cursor_move_to`], except it exclude the empty eol.
pub fn normalize_as_cursor_move_to_exclude_empty_eol(
  buffer: &Buffer,
  op: Operation,
  cursor_char_idx: usize,
  cursor_line_idx: usize,
) -> (usize, usize, CursorMoveDirection) {
  let (x, y, move_direction) = normalize_as_cursor_move_to(op, cursor_char_idx, cursor_line_idx);
  let y = std::cmp::min(y, buffer.get_rope().len_lines().saturating_sub(1));
  let x = match buffer.last_char_on_line_no_empty_eol(y) {
    Some(last_char) => std::cmp::min(x, last_char),
    None => x,
  };
  (x, y, move_direction)
}

/// Same with [`normalize_as_cursor_move_to`], except it include the empty eol.
pub fn normalize_as_cursor_move_to_include_empty_eol(
  buffer: &Buffer,
  op: Operation,
  cursor_char_idx: usize,
  cursor_line_idx: usize,
) -> (usize, usize, CursorMoveDirection) {
  let (x, y, move_direction) = normalize_as_cursor_move_to(op, cursor_char_idx, cursor_line_idx);
  let y = std::cmp::min(y, buffer.get_rope().len_lines().saturating_sub(1));
  let x = match buffer.last_char_on_line(y) {
    Some(last_char) => std::cmp::min(x, last_char),
    None => x,
  };
  (x, y, move_direction)
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

/// Normalize `Operation::WindowScroll*` to `Operation::WindowScrollTo((x,y))`.
pub fn normalize_as_window_scroll_to(
  op: Operation,
  viewport_start_column_idx: usize,
  viewport_start_line_idx: usize,
) -> (usize, usize) {
  match op {
    Operation::WindowScrollLeftBy(n) => {
      let x = viewport_start_column_idx.saturating_add_signed(-(n as isize));
      let y = viewport_start_line_idx;
      (x, y)
    }
    Operation::WindowScrollRightBy(n) => {
      let x = viewport_start_column_idx.saturating_add_signed(n as isize);
      let y = viewport_start_line_idx;
      (x, y)
    }
    Operation::WindowScrollUpBy(n) => {
      let x = viewport_start_column_idx;
      let y = viewport_start_line_idx.saturating_add_signed(-(n as isize));
      (x, y)
    }
    Operation::WindowScrollDownBy(n) => {
      let x = viewport_start_column_idx;
      let y = viewport_start_line_idx.saturating_add_signed(n as isize);
      (x, y)
    }
    Operation::WindowScrollTo((x, y)) => (x, y),
    Operation::WindowScrollBy((by_x, by_y)) => {
      let x = viewport_start_column_idx.saturating_add_signed(by_x);
      let y = viewport_start_line_idx.saturating_add_signed(by_y);
      (x, y)
    }
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
pub fn cursor_move_to(
  viewport: &Viewport,
  cursor_viewport: &CursorViewport,
  buffer: &Buffer,
  cursor_move_to_op: Operation,
) -> Option<CursorViewportArc> {
  debug_assert!(matches!(cursor_move_to_op, Operation::CursorMoveTo((_, _))));
  let (to_char, to_line) = match cursor_move_to_op {
    Operation::CursorMoveTo((c, l)) => (c, l),
    _ => unreachable!(),
  };

  let cursor_move_to_result =
    _raw_cursor_move_to(viewport, cursor_viewport, buffer, to_char, to_line);

  if let Some((line_idx, char_idx)) = cursor_move_to_result {
    let new_cursor_viewport = CursorViewport::from_position(viewport, buffer, line_idx, char_idx);
    let new_cursor_viewport = CursorViewport::to_arc(new_cursor_viewport);
    // New cursor position
    Some(new_cursor_viewport)
  } else {
    // Or, just do nothing, stay at where you are
    None
  }
}

// Returns the `line_idx`/`char_idx` for new cursor position.
fn _raw_cursor_move_to(
  viewport: &Viewport,
  cursor_viewport: &CursorViewport,
  buffer: &Buffer,
  char_idx: usize,
  line_idx: usize,
) -> Option<(usize, usize)> {
  let cursor_line_idx = cursor_viewport.line_idx();
  let cursor_char_idx = cursor_viewport.char_idx();

  let line_idx =
    _bounded_raw_cursor_move_y_to(viewport, cursor_line_idx, cursor_char_idx, buffer, line_idx);

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
    _bounded_raw_cursor_move_x_to(viewport, line_idx, cursor_char_idx, buffer, char_idx);

  Some((line_idx, char_idx))
}

fn _bounded_raw_cursor_move_y_to(
  viewport: &Viewport,
  cursor_line_idx: usize,
  _cursor_char_idx: usize,
  _buffer: &Buffer,
  line_idx: usize,
) -> usize {
  let last_line_idx = viewport.end_line_idx().saturating_sub(1);
  trace!(
    "cursor_line_idx:{:?},last_line_idx:{:?}",
    cursor_line_idx, last_line_idx
  );
  std::cmp::min(line_idx, last_line_idx)
}

fn _bounded_raw_cursor_move_x_to(
  _viewport: &Viewport,
  cursor_line_idx: usize,
  _cursor_char_idx: usize,
  buffer: &Buffer,
  char_idx: usize,
) -> usize {
  match buffer.last_char_on_line(cursor_line_idx) {
    Some(last_char) => std::cmp::min(last_char, char_idx),
    None => char_idx,
  }
}

pub fn window_scroll_to(
  viewport: &Viewport,
  current_window: &Window,
  buffer: &Buffer,
  window_scroll_to_op: Operation,
) -> Option<ViewportArc> {
  debug_assert!(matches!(
    window_scroll_to_op,
    Operation::WindowScrollTo((_, _))
  ));
  let (to_column, to_line) = match window_scroll_to_op {
    Operation::WindowScrollTo((c, l)) => (c, l),
    _ => unreachable!(),
  };

  let window_scroll_to_result = _raw_window_scroll_to(viewport, buffer, to_column, to_line);

  if let Some((start_line_idx, start_column_idx)) = window_scroll_to_result {
    // Sync the viewport
    let window_actual_shape = current_window.window_content().actual_shape();
    let window_local_options = current_window.options();
    let new_viewport = Viewport::to_arc(Viewport::view(
      buffer,
      window_actual_shape,
      window_local_options,
      start_line_idx,
      start_column_idx,
    ));
    Some(new_viewport)
  } else {
    // Or just do nothing and keep current viewport
    None
  }
}

/// Returns the `start_line_idx`/`start_column_idx` for new window viewport.
fn _raw_window_scroll_to(
  viewport: &Viewport,
  buffer: &Buffer,
  column_idx: usize,
  line_idx: usize,
) -> Option<(usize, usize)> {
  let start_line_idx = viewport.start_line_idx();
  let end_line_idx = viewport.end_line_idx();
  let start_column_idx = viewport.start_column_idx();
  let buffer_len_lines = buffer.get_rope().len_lines();
  debug_assert!(end_line_idx <= buffer_len_lines);

  let mut line_idx = _bounded_raw_window_scroll_y_to(buffer, line_idx);

  // If viewport wants to scroll down (i.e. lines_idx > start_line_idx), and viewport already shows
  // that last line in the buffer, then cannot scroll down anymore, just still keep the old
  // `line_idx`.
  if line_idx > start_line_idx && end_line_idx == buffer_len_lines {
    line_idx = start_line_idx;
  }

  let column_idx = _bounded_raw_window_scroll_x_to(start_column_idx, viewport, buffer, column_idx);

  // If the newly `start_line_idx`/`start_column_idx` is the same with current viewport, then
  // there's no need to scroll anymore.
  if line_idx == start_line_idx && column_idx == start_column_idx {
    return None;
  }

  Some((line_idx, column_idx))
}

fn _bounded_raw_window_scroll_y_to(buffer: &Buffer, line_idx: usize) -> usize {
  let buffer_len_lines = buffer.get_rope().len_lines();
  std::cmp::min(line_idx, buffer_len_lines.saturating_sub(1))
}

// Calculate how many columns that each line (in current viewport) need to scroll until
// their own line's end. This is the upper bound of the actual columns that could
// scroll.
fn _bounded_raw_window_scroll_x_max_scrolls(viewport: &Viewport, buffer: &Buffer) -> usize {
  let mut max_scrolls = 0_usize;
  for (line_idx, line_viewport) in viewport.lines().iter() {
    trace!("line_idx:{},line_viewport:{:?}", line_idx, line_viewport);
    debug_assert!(!line_viewport.rows().is_empty());
    let (_last_row_idx, last_row_viewport) = line_viewport.rows().last_key_value().unwrap();
    trace!(
      "_last_row_idx:{},last_row_viewport:{:?}",
      _last_row_idx, last_row_viewport
    );
    debug_assert!(buffer.get_rope().get_line(*line_idx).is_some());
    // If `last_row_viewport` is empty, i.e. the `end_char_idx == start_char_idx`, the scrolls is 0.
    if last_row_viewport.end_char_idx() > last_row_viewport.start_char_idx() {
      let max_scrolls_on_line = match buffer.last_char_on_line_no_empty_eol(*line_idx) {
        Some(last_visible_c) => {
          let last_visible_col = buffer.width_until(*line_idx, last_visible_c);
          let last_col_on_row = buffer.width_until(
            *line_idx,
            last_row_viewport.end_char_idx().saturating_sub(1),
          );
          let column_difference = last_visible_col.saturating_sub(last_col_on_row);
          trace!(
            "last_visible_c:{},last_row_viewport.end_char_idx:{},last_visible_col:{},last_col_on_row:{},column_difference:{}",
            last_visible_c,
            last_row_viewport.end_char_idx(),
            last_visible_col,
            last_col_on_row,
            column_difference
          );
          column_difference
        }
        None => 0_usize,
      };
      trace!("result:{}", max_scrolls_on_line);
      max_scrolls = std::cmp::max(max_scrolls, max_scrolls_on_line);
    }
  }
  max_scrolls
}

fn _bounded_raw_window_scroll_x_to(
  start_column_idx: usize,
  viewport: &Viewport,
  buffer: &Buffer,
  column_idx: usize,
) -> usize {
  let max_scrolls = _bounded_raw_window_scroll_x_max_scrolls(viewport, buffer);
  let upper_bounded = start_column_idx.saturating_add(max_scrolls);
  trace!(
    "max_scrolls:{},upper_bounded:{},column_idx:{}",
    max_scrolls, upper_bounded, column_idx
  );
  std::cmp::min(column_idx, upper_bounded)
}
