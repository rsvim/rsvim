//! Cursor operations.

use crate::buf::text::Text;
use crate::coord::U16Rect;
use crate::state::ops::Operation;
use crate::ui::tree::*;
use crate::ui::viewport::{
  CursorViewport, CursorViewportArc, Viewport, ViewportArc, ViewportSearchDirection, Viewportable,
};
use crate::ui::widget::window::WindowLocalOptions;

use compact_str::CompactString;
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

/// Normalize `Operation::CursorMove*` to `Operation::CursorMoveTo((x,y))`, it excludes the empty
/// eol.
pub fn normalize_to_cursor_move_to_exclude_empty_eol(
  text: &Text,
  op: Operation,
  cursor_char_idx: usize,
  cursor_line_idx: usize,
) -> (usize, usize, CursorMoveDirection) {
  let (x, y, move_direction) = normalize_to_cursor_move_to(op, cursor_char_idx, cursor_line_idx);
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
  let (x, y, move_direction) = normalize_to_cursor_move_to(op, cursor_char_idx, cursor_line_idx);
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
pub fn raw_cursor_move_to(
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

pub fn raw_widget_scroll_to(
  viewport: &Viewport,
  actual_shape: &U16Rect,
  window_options: &WindowLocalOptions,
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

  let max_len_chars = _max_len_chars_since_line(text, line_idx, actual_shape.height());
  let column_idx = std::cmp::min(column_idx, max_len_chars.saturating_sub(1));

  // If the newly `start_line_idx`/`start_column_idx` is the same with current viewport, then
  // there's no need to scroll anymore.
  if line_idx == viewport.start_line_idx() && column_idx == viewport.start_column_idx() {
    return None;
  }

  // Sync the viewport
  let new_viewport = Viewport::to_arc(Viewport::view(
    window_options,
    text,
    actual_shape,
    line_idx,
    column_idx,
  ));
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

/// Returns `(cursor_line_idx, cursor_char_idx)` if delete successful, or returns `None` if failed.
pub fn raw_delete_at_cursor(
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
  text.append_empty_eol_at_end_if_not_exist();

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
pub fn raw_insert_at_cursor(
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
  text.append_empty_eol_at_end_if_not_exist();

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

pub fn update_viewport_after_text_changed(tree: &mut Tree, id: TreeNodeId, text: &Text) {
  debug_assert!(tree.node_mut(id).is_some());
  let node = tree.node_mut(id).unwrap();
  debug_assert!(matches!(
    node,
    TreeNode::Window(_) | TreeNode::CommandLine(_)
  ));

  let actual_shape = match node {
    TreeNode::Window(window) => *window.actual_shape(),
    TreeNode::CommandLine(cmdline) => *cmdline.actual_shape(),
    _ => unreachable!(),
  };
  let vnode: &mut dyn Viewportable = match node {
    TreeNode::Window(window) => window,
    TreeNode::CommandLine(cmdline) => cmdline,
    _ => unreachable!(),
  };

  let viewport = vnode.viewport();
  let cursor_viewport = vnode.cursor_viewport();
  trace!("before viewport:{:?}", viewport);
  trace!("before cursor_viewport:{:?}", cursor_viewport);

  let start_line = std::cmp::min(
    viewport.start_line_idx(),
    text.rope().len_lines().saturating_sub(1),
  );
  debug_assert!(text.rope().get_line(start_line).is_some());
  let bufline_len_chars = text.rope().line(start_line).len_chars();
  let start_column = std::cmp::min(
    viewport.start_column_idx(),
    text.width_before(start_line, bufline_len_chars),
  );

  let updated_viewport = Viewport::to_arc(Viewport::view(
    vnode.options(),
    text,
    &actual_shape,
    start_line,
    start_column,
  ));
  trace!("after updated_viewport:{:?}", updated_viewport);

  vnode.set_viewport(updated_viewport.clone());
  if let Some(updated_cursor_viewport) = raw_cursor_move_to(
    &updated_viewport,
    &cursor_viewport,
    text,
    Operation::CursorMoveTo((cursor_viewport.char_idx(), cursor_viewport.line_idx())),
  ) {
    trace!(
      "after updated_cursor_viewport:{:?}",
      updated_cursor_viewport
    );
    vnode.set_cursor_viewport(updated_cursor_viewport);
  }
}

/// The operation must be `Operation::CursorMove*`.
pub fn cursor_move(tree: &mut Tree, text: &Text, op: Operation, include_empty_eol: bool) {
  debug_assert!(tree.cursor_id().is_some());
  let cursor_id = tree.cursor_id().unwrap();
  debug_assert!(tree.parent_id(cursor_id).is_some());
  let cursor_parent_id = tree.parent_id(cursor_id).unwrap();
  debug_assert!(tree.node_mut(cursor_parent_id).is_some());
  let cursor_parent_node = tree.node_mut(cursor_parent_id).unwrap();
  debug_assert!(matches!(
    cursor_parent_node,
    TreeNode::Window(_) | TreeNode::CommandLine(_)
  ));
  let vnode_actual_shape = match cursor_parent_node {
    TreeNode::Window(window) => *window.actual_shape(),
    TreeNode::CommandLine(cmdline) => *cmdline.actual_shape(),
    _ => unreachable!(),
  };
  let vnode: &mut dyn Viewportable = match cursor_parent_node {
    TreeNode::Window(window) => window,
    TreeNode::CommandLine(cmdline) => cmdline,
    _ => unreachable!(),
  };

  let viewport = vnode.viewport();
  let cursor_viewport = vnode.cursor_viewport();

  // Only move cursor when it is different from current cursor.
  let (target_cursor_char, target_cursor_line, move_direction) = if include_empty_eol {
    normalize_to_cursor_move_to_include_empty_eol(
      text,
      op,
      cursor_viewport.char_idx(),
      cursor_viewport.line_idx(),
    )
  } else {
    normalize_to_cursor_move_to_exclude_empty_eol(
      text,
      op,
      cursor_viewport.char_idx(),
      cursor_viewport.line_idx(),
    )
  };

  let search_direction = match move_direction {
    CursorMoveDirection::Up => ViewportSearchDirection::Up,
    CursorMoveDirection::Down => ViewportSearchDirection::Down,
    CursorMoveDirection::Left => ViewportSearchDirection::Left,
    CursorMoveDirection::Right => ViewportSearchDirection::Right,
  };

  let new_viewport: Option<ViewportArc> = {
    let (start_line, start_column) = viewport.search_anchor(
      search_direction,
      vnode.options(),
      text,
      &vnode_actual_shape,
      target_cursor_line,
      target_cursor_char,
    );

    // First try window scroll.
    if start_line != viewport.start_line_idx() || start_column != viewport.start_column_idx() {
      let new_viewport = raw_widget_scroll_to(
        &viewport,
        &vnode_actual_shape,
        vnode.options(),
        text,
        Operation::WindowScrollTo((start_column, start_line)),
      );
      if let Some(new_viewport_arc) = new_viewport.clone() {
        vnode.set_viewport(new_viewport_arc.clone());
      }
      new_viewport
    } else {
      None
    }
  };

  // Then try cursor move.
  {
    let current_viewport = new_viewport.unwrap_or(viewport);

    let new_cursor_viewport = raw_cursor_move_to(
      &current_viewport,
      &cursor_viewport,
      text,
      Operation::CursorMoveTo((target_cursor_char, target_cursor_line)),
    );

    if let Some(new_cursor_viewport) = new_cursor_viewport {
      vnode.set_cursor_viewport(new_cursor_viewport.clone());
      let cursor_id = tree.cursor_id().unwrap();
      tree.bounded_move_to(
        cursor_id,
        new_cursor_viewport.column_idx() as isize,
        new_cursor_viewport.row_idx() as isize,
      );
    }
  }
}

pub fn cursor_insert(tree: &mut Tree, text: &mut Text, payload: CompactString) {
  debug_assert!(tree.cursor_id().is_some());
  let cursor_id = tree.cursor_id().unwrap();
  debug_assert!(tree.parent_id(cursor_id).is_some());
  let cursor_parent_id = tree.parent_id(cursor_id).unwrap();
  debug_assert!(tree.node_mut(cursor_parent_id).is_some());
  let cursor_parent_node = tree.node_mut(cursor_parent_id).unwrap();
  debug_assert!(matches!(
    cursor_parent_node,
    TreeNode::Window(_) | TreeNode::CommandLine(_)
  ));
  let vnode: &mut dyn Viewportable = match cursor_parent_node {
    TreeNode::Window(window) => window,
    TreeNode::CommandLine(cmdline) => cmdline,
    _ => unreachable!(),
  };

  // Insert text.
  let cursor_viewport = vnode.cursor_viewport();
  let (cursor_line_idx_after_inserted, cursor_char_idx_after_inserted) =
    raw_insert_at_cursor(&cursor_viewport, text, payload);
  // Update viewport since the buffer doesn't match the viewport.
  update_viewport_after_text_changed(tree, cursor_parent_id, text);

  trace!(
    "Move to inserted pos, line:{cursor_line_idx_after_inserted}, char:{cursor_char_idx_after_inserted}"
  );
  let op = Operation::CursorMoveTo((
    cursor_char_idx_after_inserted,
    cursor_line_idx_after_inserted,
  ));
  cursor_move(tree, text, op, true);
}

pub fn cursor_delete(tree: &mut Tree, text: &mut Text, n: isize) -> bool {
  debug_assert!(tree.cursor_id().is_some());
  let cursor_id = tree.cursor_id().unwrap();
  debug_assert!(tree.parent_id(cursor_id).is_some());
  let cursor_parent_id = tree.parent_id(cursor_id).unwrap();
  debug_assert!(tree.node_mut(cursor_parent_id).is_some());
  let cursor_parent_node = tree.node_mut(cursor_parent_id).unwrap();
  debug_assert!(matches!(
    cursor_parent_node,
    TreeNode::Window(_) | TreeNode::CommandLine(_)
  ));
  let vnode: &mut dyn Viewportable = match cursor_parent_node {
    TreeNode::Window(window) => window,
    TreeNode::CommandLine(cmdline) => cmdline,
    _ => unreachable!(),
  };

  // Delete N-chars.
  let cursor_viewport = vnode.cursor_viewport();
  let maybe_new_cursor_position = raw_delete_at_cursor(&cursor_viewport, text, n);
  if maybe_new_cursor_position.is_none() {
    return false;
  }

  // Update viewport since the buffer doesn't match the viewport.
  update_viewport_after_text_changed(tree, cursor_parent_id, text);
  let (cursor_line_idx_after_deleted, cursor_char_idx_after_deleted) =
    maybe_new_cursor_position.unwrap();

  trace!(
    "Move to deleted pos, line:{cursor_line_idx_after_deleted}, char:{cursor_char_idx_after_deleted}"
  );
  let op = Operation::CursorMoveTo((cursor_char_idx_after_deleted, cursor_line_idx_after_deleted));
  cursor_move(tree, text, op, true);

  true
}
