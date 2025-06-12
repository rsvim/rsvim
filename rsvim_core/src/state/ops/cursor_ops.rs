//! Cursor operations.

use crate::buf::Text;
use crate::state::ops::Operation;
use crate::ui::tree::*;
use crate::ui::viewport::{
  CursorViewport, CursorViewportArc, Viewport, ViewportArc, ViewportOptions,
};
use crate::ui::widget::window::Window;

use compact_str::{CompactString, ToCompactString};
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
fn _normalize_to_cursor_move_by(
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
fn _normalize_to_cursor_move_to(
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

/// Normalize `Operation::CursorMove*` to `Operation::CursorMoveTo((x,y))`, it excludes the empty
/// eol.
pub fn normalize_to_cursor_move_to_exclude_empty_eol(
  text: &Text,
  op: Operation,
  cursor_char_idx: usize,
  cursor_line_idx: usize,
) -> (usize, usize, CursorMoveDirection) {
  let (x, y, move_direction) = _normalize_to_cursor_move_to(op, cursor_char_idx, cursor_line_idx);
  let mut y = std::cmp::min(y, text.rope().len_lines().saturating_sub(1));
  if text.rope().line(y).len_chars() == 0 {
    // If the `y` has no chars (because the `y` is the last line in rope and separate by the last
    // line break '\n'), sub y by extra 1.
    y = y.saturating_sub(1);
  }
  let x = match text.last_char_on_line_no_empty_eol(y) {
    Some(last_char) => std::cmp::min(x, last_char),
    None => {
      debug_assert!(text.rope().get_line(y).is_some());
      std::cmp::min(x, text.rope().line(y).len_chars().saturating_sub(1))
    }
  };
  (x, y, move_direction)
}

/// Normalize `Operation::CursorMove*` to `Operation::CursorMoveTo((x,y))`, it includes the empty
/// eol.
pub fn normalize_to_cursor_move_to_include_empty_eol(
  text: &Text,
  op: Operation,
  cursor_char_idx: usize,
  cursor_line_idx: usize,
) -> (usize, usize, CursorMoveDirection) {
  let (x, y, move_direction) = _normalize_to_cursor_move_to(op, cursor_char_idx, cursor_line_idx);
  let mut y = std::cmp::min(y, text.rope().len_lines().saturating_sub(1));
  if text.rope().line(y).len_chars() == 0 {
    // If the `y` has no chars (because the `y` is the last line in rope and separate by the last
    // line break '\n'), sub y by extra 1.
    y = y.saturating_sub(1);
  }
  let x = match text.last_char_on_line(y) {
    Some(last_char) => std::cmp::min(x, last_char),
    None => {
      debug_assert!(text.rope().get_line(y).is_some());
      std::cmp::min(x, text.rope().line(y).len_chars().saturating_sub(1))
    }
  };
  (x, y, move_direction)
}

/// Normalize `Operation::WindowScroll*` to `Operation::WindowScrollBy((x,y))`.
fn _normalize_to_window_scroll_by(
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
  _cursor_viewport: &CursorViewport,
  text: &Text,
  cursor_move_to_op: Operation,
) -> Option<CursorViewportArc> {
  debug_assert!(matches!(cursor_move_to_op, Operation::CursorMoveTo((_, _))));
  let (char_idx, line_idx) = match cursor_move_to_op {
    Operation::CursorMoveTo((c, l)) => (c, l),
    _ => unreachable!(),
  };

  let line_idx = std::cmp::min(line_idx, viewport.end_line_idx().saturating_sub(1));
  debug_assert!(line_idx < viewport.end_line_idx());
  debug_assert!(text.rope().get_line(line_idx).is_some());

  let bufline = text.rope().line(line_idx);
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

  let new_cursor_viewport = CursorViewport::from_position(viewport, text, line_idx, char_idx);
  let new_cursor_viewport = CursorViewport::to_arc(new_cursor_viewport);
  // New cursor position
  Some(new_cursor_viewport)
}

pub fn window_scroll_to(
  viewport: &Viewport,
  current_window: &Window,
  text: &Text,
  window_scroll_to_op: Operation,
) -> Option<ViewportArc> {
  debug_assert!(matches!(
    window_scroll_to_op,
    Operation::WindowScrollTo((_, _))
  ));
  let (column_idx, line_idx) = match window_scroll_to_op {
    Operation::WindowScrollTo((c, l)) => (c, l),
    _ => unreachable!(),
  };

  let buffer_len_lines = text.rope().len_lines();
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
  debug_assert!(text.rope().get_line(line_idx).is_some());

  let shape = current_window.actual_shape();
  let max_len_chars = _max_len_chars_since_line(text, line_idx, shape.height());
  let column_idx = std::cmp::min(column_idx, max_len_chars.saturating_sub(1));

  // If the newly `start_line_idx`/`start_column_idx` is the same with current viewport, then
  // there's no need to scroll anymore.
  if line_idx == viewport.start_line_idx() && column_idx == viewport.start_column_idx() {
    return None;
  }

  // Sync the viewport
  let opts = ViewportOptions::from(current_window.options());
  let new_viewport = Viewport::to_arc(Viewport::view(&opts, text, shape, line_idx, column_idx));
  Some(new_viewport)
}

fn _max_len_chars_since_line(text: &Text, mut start_line_idx: usize, window_height: u16) -> usize {
  let buffer_len_lines = text.rope().len_lines();

  let mut max_len_chars = 0_usize;
  let mut i = 0_u16;
  while i < window_height && start_line_idx < buffer_len_lines {
    debug_assert!(text.rope().get_line(start_line_idx).is_some());
    let bufline = text.rope().line(start_line_idx);
    max_len_chars = std::cmp::max(max_len_chars, bufline.len_chars());
    i += 1;
    start_line_idx += 1;
  }
  max_len_chars
}

/// Only for testing
pub fn _bufline_to_string(bufline: &ropey::RopeSlice) -> String {
  let mut builder = String::with_capacity(bufline.len_chars());
  for c in bufline.chars() {
    builder.push(c);
  }
  builder
}

/// Only for testing
pub fn _dbg_print_details(buffer: &Text, line_idx: usize, char_idx: usize, msg: &str) {
  if cfg!(debug_assertions) {
    match buffer.rope().get_line(line_idx) {
      Some(bufline) => {
        trace!(
          "{}, line:{}, len_chars:{}, focus char:{}",
          msg,
          line_idx,
          bufline.len_chars(),
          char_idx
        );
        let start_char_on_line = buffer.rope().line_to_char(line_idx);

        let mut builder1 = String::new();
        let mut builder2 = String::new();
        for (i, c) in bufline.chars().enumerate() {
          let w = buffer.char_width(c);
          if w > 0 {
            builder1.push(c);
          }
          let s: String = std::iter::repeat_n(
            if i + start_char_on_line == char_idx {
              '^'
            } else {
              ' '
            },
            w,
          )
          .collect();
          builder2.push_str(s.as_str());
        }
        trace!("-{}-", builder1);
        trace!("-{}-", builder2);
      }
      None => trace!(
        "{} line:{}, focus char:{}, not exist",
        msg, line_idx, char_idx
      ),
    }

    trace!("{}, Whole buffer:", msg);
    for i in 0..buffer.rope().len_lines() {
      trace!("{i}:{:?}", _bufline_to_string(&buffer.rope().line(i)));
    }
  }
}

pub fn _dbg_print_details_on_line(buffer: &Text, line_idx: usize, char_idx: usize, msg: &str) {
  if cfg!(debug_assertions) {
    match buffer.rope().get_line(line_idx) {
      Some(bufline) => {
        trace!(
          "{} line:{}, len_chars:{}, focus char:{}",
          msg,
          line_idx,
          bufline.len_chars(),
          char_idx
        );
        let mut builder1 = String::new();
        let mut builder2 = String::new();
        for (i, c) in bufline.chars().enumerate() {
          let w = buffer.char_width(c);
          if w > 0 {
            builder1.push(c);
          }
          let s: String = std::iter::repeat_n(if i == char_idx { '^' } else { ' ' }, w).collect();
          builder2.push_str(s.as_str());
        }
        trace!("-{}-", builder1);
        trace!("-{}-", builder2);
      }
      None => trace!(
        "{} line:{}, focus char:{}, not exist",
        msg, line_idx, char_idx
      ),
    }

    trace!("{}, Whole buffer:", msg);
    for i in 0..buffer.rope().len_lines() {
      trace!("{i}:{:?}", _bufline_to_string(&buffer.rope().line(i)));
    }
  }
}

// For text mode (different from the 'binary' mode, i.e. bin/hex mode), the editor have
// to always keep an eol (end-of-line) at the end of text file. It helps the cursor
// motion.
fn _append_eol_at_file_end(text: &mut Text) {
  use crate::defaults::ascii::end_of_line as eol;
  let buf_eol = text.options().end_of_line();

  let buffer_len_chars = text.rope().len_chars();
  let last_char_on_buf = buffer_len_chars.saturating_sub(1);
  match text.rope().get_char(last_char_on_buf) {
    Some(c) => {
      if c.to_compact_string() != eol::LF && c.to_compact_string() != eol::CR {
        text
          .rope_mut()
          .insert(buffer_len_chars, buf_eol.to_compact_string().as_str());
        let inserted_line_idx = text.rope().char_to_line(buffer_len_chars);
        text.retain_cached_lines(|line_idx, _column_idx| *line_idx < inserted_line_idx);
        _dbg_print_details(
          text,
          inserted_line_idx,
          buffer_len_chars,
          "Eol appended(non-empty)",
        );
      }
    }
    None => {
      text
        .rope_mut()
        .insert(0_usize, buf_eol.to_compact_string().as_str());
      text.clear_cached_lines();
      _dbg_print_details(text, 0_usize, buffer_len_chars, "Eol appended(empty)");
    }
  }
}

/// Returns `(cursor_line_idx, cursor_char_idx)` if delete successful, or returns `None` if failed.
pub fn delete_at_cursor(
  cursor_viewport: &CursorViewport,
  text: &mut Text,
  n: isize,
) -> Option<(usize, usize)> {
  let cursor_line_idx = cursor_viewport.line_idx();
  let cursor_char_idx = cursor_viewport.char_idx();
  debug_assert!(text.rope().get_line(cursor_line_idx).is_some());

  let cursor_char_absolute_pos_before_delete =
    text.rope().line_to_char(cursor_line_idx) + cursor_char_idx;

  _dbg_print_details(
    text,
    cursor_line_idx,
    cursor_char_absolute_pos_before_delete,
    "Before delete",
  );

  let to_be_deleted_range = if n > 0 {
    // Delete to right side, on range `[cursor..cursor+n)`.
    cursor_char_absolute_pos_before_delete
      ..(std::cmp::min(
        cursor_char_absolute_pos_before_delete + n as usize,
        text.rope().len_chars().saturating_sub(1),
      ))
  } else {
    // Delete to left side, on range `[cursor-n,cursor)`.
    (std::cmp::max(
      0_usize,
      cursor_char_absolute_pos_before_delete.saturating_add_signed(n),
    ))..cursor_char_absolute_pos_before_delete
  };

  if to_be_deleted_range.is_empty() {
    return None;
  }

  text.rope_mut().remove(to_be_deleted_range);

  // Append eol at file end if it doesn't exist.
  _append_eol_at_file_end(text);

  let cursor_char_absolute_pos_after_deleted = if n > 0 {
    cursor_char_absolute_pos_before_delete
  } else {
    cursor_char_absolute_pos_before_delete.saturating_add_signed(n)
  };
  let cursor_char_absolute_pos_after_deleted = std::cmp::min(
    cursor_char_absolute_pos_after_deleted,
    text.rope().len_chars().saturating_sub(1),
  );
  let cursor_line_idx_after_deleted = text
    .rope()
    .char_to_line(cursor_char_absolute_pos_after_deleted);
  let cursor_line_absolute_pos_after_deleted =
    text.rope().line_to_char(cursor_line_idx_after_deleted);
  let cursor_char_idx_after_deleted =
    cursor_char_absolute_pos_after_deleted - cursor_line_absolute_pos_after_deleted;

  if cursor_line_idx == cursor_line_idx_after_deleted {
    // If before/after insert, the cursor line doesn't change, it means the inserted text doesn't contain line break, i.e. it is still the same line.
    // Thus only need to truncate chars after insert position on the same line.
    let min_cursor_char_idx = std::cmp::min(cursor_char_idx_after_deleted, cursor_char_idx);
    text.truncate_cached_line_since_char(cursor_line_idx, min_cursor_char_idx);
  } else {
    // Otherwise the inserted text contains line breaks, and we have to truncate all the cached lines below the cursor line, because we have new lines.
    let min_cursor_line_idx = std::cmp::min(cursor_line_idx_after_deleted, cursor_line_idx);
    text.retain_cached_lines(|line_idx, _column_idx| *line_idx < min_cursor_line_idx);
  }

  _dbg_print_details_on_line(
    text,
    cursor_line_idx,
    cursor_char_idx_after_deleted,
    "After deleted",
  );

  Some((cursor_line_idx_after_deleted, cursor_char_idx_after_deleted))
}

/// Returns `(cursor_line_idx, cursor_char_idx)` after insertion.
pub fn insert_at_cursor(
  cursor_viewport: &CursorViewport,
  text: &mut Text,
  payload: CompactString,
) -> (usize, usize) {
  let cursor_line_idx = cursor_viewport.line_idx();
  let cursor_char_idx = cursor_viewport.char_idx();
  debug_assert!(text.rope().get_line(cursor_line_idx).is_some());

  let cursor_line_absolute_pos = text.rope().line_to_char(cursor_line_idx);
  let cursor_char_absolute_pos_before_insert = cursor_line_absolute_pos + cursor_char_idx;

  _dbg_print_details(
    text,
    cursor_line_idx,
    cursor_char_absolute_pos_before_insert,
    "Before insert",
  );

  text
    .rope_mut()
    .insert(cursor_char_absolute_pos_before_insert, payload.as_str());

  // The `text` may contains line break '\n', which can interrupts the `cursor_line_idx`
  // and we need to re-calculate it.
  let cursor_char_absolute_pos_after_inserted =
    cursor_char_absolute_pos_before_insert + payload.chars().count();
  let cursor_line_idx_after_inserted = text
    .rope()
    .char_to_line(cursor_char_absolute_pos_after_inserted);
  let cursor_line_absolute_pos_after_inserted =
    text.rope().line_to_char(cursor_line_idx_after_inserted);
  let cursor_char_idx_after_inserted =
    cursor_char_absolute_pos_after_inserted - cursor_line_absolute_pos_after_inserted;

  // Append eol at file end if it doesn't exist.
  _append_eol_at_file_end(text);

  if cursor_line_idx == cursor_line_idx_after_inserted {
    // If before/after insert, the cursor line doesn't change, it means the inserted text doesn't contain line break, i.e. it is still the same line.
    // Thus only need to truncate chars after insert position on the same line.
    debug_assert!(cursor_char_idx_after_inserted >= cursor_char_idx);
    let min_cursor_char_idx = std::cmp::min(cursor_char_idx_after_inserted, cursor_char_idx);
    text.truncate_cached_line_since_char(cursor_line_idx, min_cursor_char_idx.saturating_sub(1));
  } else {
    // Otherwise the inserted text contains line breaks, and we have to truncate all the cached lines below the cursor line, because we have new lines.
    let min_cursor_line_idx = std::cmp::min(cursor_line_idx_after_inserted, cursor_line_idx);
    text.retain_cached_lines(|line_idx, _column_idx| *line_idx < min_cursor_line_idx);
  }

  _dbg_print_details_on_line(
    text,
    cursor_line_idx,
    cursor_char_idx_after_inserted,
    "After inserted",
  );

  (
    cursor_line_idx_after_inserted,
    cursor_char_idx_after_inserted,
  )
}
