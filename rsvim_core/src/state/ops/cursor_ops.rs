//! Cursor operations.

use crate::buf::Buffer;
use crate::state::ops::Operation;
use crate::ui::tree::*;
use crate::ui::widget::window::{CursorViewport, CursorViewportArc, Viewport, ViewportArc, Window};

// use tracing::trace;

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
pub fn normalize_to_cursor_move_by(
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
pub fn normalize_to_cursor_move_to(
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

/// Same with [`normalize_to_cursor_move_to`], except it exclude the empty eol.
pub fn normalize_to_cursor_move_to_exclude_empty_eol(
  buffer: &Buffer,
  op: Operation,
  cursor_char_idx: usize,
  cursor_line_idx: usize,
) -> (usize, usize, CursorMoveDirection) {
  let (x, y, move_direction) = normalize_to_cursor_move_to(op, cursor_char_idx, cursor_line_idx);
  let y = std::cmp::min(y, buffer.get_rope().len_lines().saturating_sub(1));
  let x = match buffer.last_char_on_line_no_empty_eol(y) {
    Some(last_char) => std::cmp::min(x, last_char),
    None => {
      debug_assert!(buffer.get_rope().get_line(y).is_some());
      std::cmp::min(x, buffer.get_rope().line(y).len_chars().saturating_sub(1))
    }
  };
  (x, y, move_direction)
}

/// Same with [`normalize_to_cursor_move_to`], except it include the empty eol.
pub fn normalize_to_cursor_move_to_include_empty_eol(
  buffer: &Buffer,
  op: Operation,
  cursor_char_idx: usize,
  cursor_line_idx: usize,
) -> (usize, usize, CursorMoveDirection) {
  let (x, y, move_direction) = normalize_to_cursor_move_to(op, cursor_char_idx, cursor_line_idx);
  let y = std::cmp::min(y, buffer.get_rope().len_lines().saturating_sub(1));
  let x = match buffer.last_char_on_line(y) {
    Some(last_char) => std::cmp::min(x, last_char),
    None => {
      debug_assert!(buffer.get_rope().get_line(y).is_some());
      std::cmp::min(x, buffer.get_rope().line(y).len_chars().saturating_sub(1))
    }
  };
  (x, y, move_direction)
}

/// Normalize `Operation::WindowScroll*` to `Operation::WindowScrollBy((x,y))`.
pub fn normalize_to_window_scroll_by(
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
pub fn normalize_to_window_scroll_to(
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

  let result = _raw_cursor_move_to(viewport, cursor_viewport, buffer, to_char, to_line);

  if let Some((line_idx, char_idx)) = result {
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
  _cursor_viewport: &CursorViewport,
  buffer: &Buffer,
  char_idx: usize,
  line_idx: usize,
) -> Option<(usize, usize)> {
  let line_idx = std::cmp::min(line_idx, viewport.end_line_idx().saturating_sub(1));
  debug_assert!(line_idx < viewport.end_line_idx());
  debug_assert!(buffer.get_rope().get_line(line_idx).is_some());

  let bufline = buffer.get_rope().line(line_idx);
  debug_assert!(bufline.len_chars() >= char_idx);

  let char_idx = if bufline.len_chars() == 0 {
    0_usize
  } else {
    std::cmp::min(char_idx, bufline.len_chars().saturating_sub(1))
  };

  if bufline.len_chars() == 0 {
    debug_assert_eq!(char_idx, 0_usize);
  } else {
    debug_assert!(bufline.len_chars() > char_idx);
  }

  Some((line_idx, char_idx))
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

  let result = _raw_window_scroll_to(viewport, current_window, buffer, to_column, to_line);

  if let Some((start_line_idx, start_column_idx)) = result {
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
  current_window: &Window,
  buffer: &Buffer,
  column_idx: usize,
  line_idx: usize,
) -> Option<(usize, usize)> {
  let start_line_idx = viewport.start_line_idx();
  let end_line_idx = viewport.end_line_idx();
  let start_column_idx = viewport.start_column_idx();
  let buffer_len_lines = buffer.get_rope().len_lines();
  debug_assert!(end_line_idx <= buffer_len_lines);

  let line_idx = if buffer_len_lines == 0 {
    0_usize
  } else {
    std::cmp::min(line_idx, buffer_len_lines.saturating_sub(1))
  };

  if buffer_len_lines == 0 {
    debug_assert_eq!(line_idx, 0_usize);
  } else {
    debug_assert!(line_idx < buffer_len_lines);
  }
  debug_assert!(buffer.get_rope().get_line(line_idx).is_some());

  let window_actual_shape = current_window.actual_shape();
  let max_len_chars = _max_len_chars_since_line(buffer, line_idx, window_actual_shape.height());
  let column_idx = std::cmp::min(column_idx, max_len_chars.saturating_sub(1));

  // If the newly `start_line_idx`/`start_column_idx` is the same with current viewport, then
  // there's no need to scroll anymore.
  if line_idx == start_line_idx && column_idx == start_column_idx {
    return None;
  }

  Some((line_idx, column_idx))
}

fn _max_len_chars_since_line(
  buffer: &Buffer,
  mut start_line_idx: usize,
  window_height: u16,
) -> usize {
  let buffer_len_lines = buffer.get_rope().len_lines();

  let mut max_len_chars = 0_usize;
  let mut i = 0_u16;
  while i < window_height && start_line_idx < buffer_len_lines {
    debug_assert!(buffer.get_rope().get_line(start_line_idx).is_some());
    let bufline = buffer.get_rope().line(start_line_idx);
    max_len_chars = std::cmp::max(max_len_chars, bufline.len_chars());
    i += 1;
    start_line_idx += 1;
  }
  max_len_chars
}
