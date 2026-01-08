//! Cursor operations.

use crate::buf::text::Text;
use crate::prelude::*;
use crate::state::ops::Operation;
use crate::ui::tree::*;
use crate::ui::viewport::CursorViewport;
use crate::ui::viewport::CursorViewportArc;
use crate::ui::viewport::Viewport;
use crate::ui::viewport::ViewportArc;
use crate::ui::viewport::ViewportSearchDirection;
use compact_str::CompactString;

#[derive(Debug, Copy, Clone)]
/// Cursor move direction.
pub enum CursorMoveDirection {
  Up,
  Down,
  Left,
  Right,
}

fn _to_viewport_search_direction(
  move_direction: CursorMoveDirection,
) -> ViewportSearchDirection {
  match move_direction {
    CursorMoveDirection::Up => ViewportSearchDirection::Up,
    CursorMoveDirection::Down => ViewportSearchDirection::Down,
    CursorMoveDirection::Left => ViewportSearchDirection::Left,
    CursorMoveDirection::Right => ViewportSearchDirection::Right,
  }
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
    Operation::CursorMoveLeftBy(n) => {
      (-(n as isize), 0, CursorMoveDirection::Left)
    }
    Operation::CursorMoveRightBy(n) => {
      (n as isize, 0, CursorMoveDirection::Right)
    }
    Operation::CursorMoveUpBy(n) => (0, -(n as isize), CursorMoveDirection::Up),
    Operation::CursorMoveDownBy(n) => {
      (0, n as isize, CursorMoveDirection::Down)
    }
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
///
/// In `ropey` library, it gives 1 extra line after the last line break. For example we have such a
/// text content:
///
/// ```text
/// Hello\nWorld\n
/// ```
///
/// The rope treats it as 3 lines:
/// 1. `Hello\n`
/// 2. `World\n`
/// 3. ``
///
/// The last line is an empty line after the last line break `\n`.
///
/// Or for an empty text content:
///
/// ```text
/// ```
///
/// The rope still treats it as 1 line:
/// 1. ``
///
/// The only line is an empty line.
pub fn normalize_to_cursor_move_to_exclude_eol(
  text: &Text,
  op: Operation,
  cursor_char_idx: usize,
  cursor_line_idx: usize,
) -> (usize, usize, CursorMoveDirection) {
  let (x, y, move_direction) =
    normalize_to_cursor_move_to(op, cursor_char_idx, cursor_line_idx);
  let y = std::cmp::min(y, text.rope().len_lines().saturating_sub(1));

  let x = match text.last_char_on_line_no_eol(y) {
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
pub fn normalize_to_cursor_move_to_include_eol(
  text: &Text,
  op: Operation,
  cursor_char_idx: usize,
  cursor_line_idx: usize,
) -> (usize, usize, CursorMoveDirection) {
  let (x, y, move_direction) =
    normalize_to_cursor_move_to(op, cursor_char_idx, cursor_line_idx);
  let y = std::cmp::min(y, text.rope().len_lines().saturating_sub(1));

  let x = match text.last_char_on_line_no_eol(y) {
    Some(last_char) => std::cmp::min(x, last_char + 1), // For include eol, allow extra 1 eol char.
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

// NOTE: This API can be used on "window" and "cmdline-input" widgets, but not
// on "cmdline-message", since the formers have cursor inside and can be
// editing, while the ladder doesn't.
fn _update_viewport(
  tree: &mut Tree,
  id: TreeNodeId,
  text: &Text,
  start_line: usize,
  start_column: usize,
) -> ViewportArc {
  let new_viewport = Viewport::to_arc(Viewport::view(
    tree.editable_options(id),
    text,
    &tree.editable_actual_shape(id).size(),
    start_line,
    start_column,
  ));

  tree.set_editable_viewport(id, new_viewport.clone());

  new_viewport
}

// NOTE: This API can be used on "window" and "cmdline-input" widgets, but not
// on "cmdline-message", since the formers have cursor inside and can be
// editing, while the ladder doesn't.
fn _update_cursor_viewport(
  tree: &mut Tree,
  id: TreeNodeId,
  viewport: &Viewport,
  text: &Text,
  cursor_line: usize,
  cursor_char: usize,
) -> CursorViewportArc {
  // New cursor position
  let new_cursor_viewport = CursorViewport::to_arc(
    CursorViewport::from_position(viewport, text, cursor_line, cursor_char),
  );

  tree.set_editable_cursor_viewport(id, new_cursor_viewport.clone());

  new_cursor_viewport
}

/// Calculate new cursor viewport by `Operation::CursorMove*` operations, as
/// if the cursor wants to move to a specific position, or by a specific
/// distance. Then update/move the cursor to the new position.
///
/// This API is bounded the cursor motion by the parent widget which the cursor
/// belongs to, the parent window/widget will not be scroll.
///
/// # Returns
///
/// It returns new cursor viewport.
///
/// # Panics
///
/// It panics if the operation is not a `Operation::CursorMove*` operation.
pub fn raw_cursor_viewport_move_to(
  tree: &mut Tree,
  id: TreeNodeId,
  viewport: &Viewport,
  text: &Text,
  cursor_move_to_op: Operation,
) -> CursorViewportArc {
  debug_assert!(matches!(cursor_move_to_op, Operation::CursorMoveTo((_, _))));
  let (char_idx, line_idx) = match cursor_move_to_op {
    Operation::CursorMoveTo((c, l)) => (c, l),
    _ => unreachable!(),
  };

  let line_idx =
    std::cmp::min(line_idx, viewport.end_line_idx().saturating_sub(1));
  debug_assert!(line_idx < viewport.end_line_idx());
  debug_assert!(text.rope().get_line(line_idx).is_some());

  let bufline = text.rope().line(line_idx);

  let char_idx = std::cmp::min(char_idx, bufline.len_chars().saturating_sub(1));
  debug_assert!(bufline.len_chars() >= char_idx);

  if bufline.len_chars() == 0 {
    debug_assert_eq!(char_idx, 0_usize);
  } else {
    debug_assert!(bufline.len_chars() > char_idx);
  }

  let new_cursor_viewport =
    _update_cursor_viewport(tree, id, viewport, text, line_idx, char_idx);
  trace!(
    "after updated, new_cursor_viewport:{:?}",
    new_cursor_viewport
  );

  new_cursor_viewport
}

/// Calculate the new viewport by `Operation::WindowScroll*` operations, as if
/// the cursor wants to move to a specific position, or by a specific distance.
/// Then update/move the new viewport for the widget.
///
/// This API only scrolls the parent window/widget where the cursor belongs to,
/// it will not moves the cursor position.
///
/// # Returns
///
/// It returns new viewport if the operation is valid, returns `None` if the
/// widget doesn't move.
///
/// # Panics
///
/// It panics if the operation is not a `Operation::WindowScroll*` operation.
pub fn raw_viewport_scroll_to(
  tree: &mut Tree,
  id: TreeNodeId,
  viewport: &Viewport,
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

  let max_len_chars = _max_len_chars_since_line(
    text,
    line_idx,
    tree.editable_actual_shape(id).height(),
  );
  let column_idx = std::cmp::min(column_idx, max_len_chars.saturating_sub(1));

  // If the newly `start_line_idx`/`start_column_idx` is the same with current viewport, then
  // there's no need to scroll anymore.
  if line_idx == viewport.start_line_idx()
    && column_idx == viewport.start_column_idx()
  {
    return None;
  }

  // Sync the viewport
  let new_viewport = Viewport::to_arc(Viewport::view(
    tree.editable_options(id),
    text,
    &tree.editable_actual_shape(id).size(),
    line_idx,
    column_idx,
  ));
  tree.set_editable_viewport(id, new_viewport.clone());

  Some(new_viewport)
}

fn _max_len_chars_since_line(
  text: &Text,
  mut start_line_idx: usize,
  window_height: u16,
) -> usize {
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

fn _update_viewport_after_text_changed(
  tree: &mut Tree,
  id: TreeNodeId,
  text: &Text,
) {
  let viewport = tree.editable_viewport(id);
  let cursor_viewport = tree.editable_cursor_viewport(id);
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

  let updated_viewport =
    _update_viewport(tree, id, text, start_line, start_column);
  trace!("after updated_viewport:{:?}", updated_viewport);

  raw_cursor_viewport_move_to(
    tree,
    id,
    &updated_viewport,
    text,
    Operation::CursorMoveTo((
      cursor_viewport.char_idx(),
      cursor_viewport.line_idx(),
    )),
  );
}

/// High-level cursor move operation.
///
/// This API will move the cursor (and possibly scroll the window/cmdline it
/// belongs to), as if the user is operating the editor (for example, using
/// `jk` to go up/down to view a very large file).
pub fn cursor_move(
  tree: &mut Tree,
  id: TreeNodeId,
  text: &Text,
  op: Operation,
  include_eol: bool,
) {
  let viewport = tree.editable_viewport(id);
  let cursor_viewport = tree.editable_cursor_viewport(id);

  // Only move cursor when it is different from current cursor.
  let (target_cursor_char, target_cursor_line, move_direction) = if include_eol
  {
    normalize_to_cursor_move_to_include_eol(
      text,
      op,
      cursor_viewport.char_idx(),
      cursor_viewport.line_idx(),
    )
  } else {
    normalize_to_cursor_move_to_exclude_eol(
      text,
      op,
      cursor_viewport.char_idx(),
      cursor_viewport.line_idx(),
    )
  };

  let search_direction = _to_viewport_search_direction(move_direction);

  let new_viewport: Option<ViewportArc> = {
    let (start_line, start_column) = viewport.search_anchor(
      search_direction,
      tree.editable_options(id),
      text,
      &tree.editable_actual_shape(id).size(),
      target_cursor_line,
      target_cursor_char,
    );

    // First try window scroll.
    if start_line != viewport.start_line_idx()
      || start_column != viewport.start_column_idx()
    {
      raw_viewport_scroll_to(
        tree,
        id,
        &viewport,
        text,
        Operation::WindowScrollTo((start_column, start_line)),
      )
    } else {
      None
    }
  };

  // Then move cursor.
  let new_viewport = new_viewport.unwrap_or(viewport);

  let new_cursor_viewport = raw_cursor_viewport_move_to(
    tree,
    id,
    &new_viewport,
    text,
    Operation::CursorMoveTo((target_cursor_char, target_cursor_line)),
  );

  debug_assert!(tree.cursor_id().is_some());
  tree
    .cursor_move_position_to(
      new_cursor_viewport.column_idx() as isize,
      new_cursor_viewport.row_idx() as isize,
    )
    .unwrap();
}

/// High-level cursor insert operation.
///
/// This API will insert text at the cursor (and possibly scroll the
/// window/cmdline), as if user is typing in insert mode.
///
/// # Returns
///
/// - It returns new cursor position `(cursor_line_idx,cursor_char_idx)` if
///   inserts successfully.
/// - It returns `None` if failed.
pub fn cursor_insert(
  tree: &mut Tree,
  id: TreeNodeId,
  text: &mut Text,
  payload: CompactString,
) -> (usize, usize) {
  debug_assert!(tree.node(id).is_some());
  let cursor_viewport = tree.editable_cursor_viewport(id);

  // Insert text.
  let cursor_line_idx = cursor_viewport.line_idx();
  let cursor_char_idx = cursor_viewport.char_idx();
  debug_assert!(text.rope().get_line(cursor_line_idx).is_some());
  debug_assert!(
    cursor_char_idx <= text.rope().line(cursor_line_idx).len_chars()
  );
  let (cursor_line_idx_after_inserted, cursor_char_idx_after_inserted) =
    text.insert_at(cursor_line_idx, cursor_char_idx, payload);

  // Update viewport since the buffer doesn't match the viewport.
  _update_viewport_after_text_changed(tree, id, text);

  trace!(
    "Move to inserted pos, line:{cursor_line_idx_after_inserted}, char:{cursor_char_idx_after_inserted}"
  );
  let op = Operation::CursorMoveTo((
    cursor_char_idx_after_inserted,
    cursor_line_idx_after_inserted,
  ));
  cursor_move(tree, id, text, op, true);

  (
    cursor_line_idx_after_inserted,
    cursor_char_idx_after_inserted,
  )
}

/// High-level cursor delete operation.
///
/// This API will delete text at the cursor to either left/right direction (and
/// possibly scroll the window/cmdline), as if user presses
/// `backspace`/`delete` in insert mode.
///
/// # Returns
///
/// - It returns new cursor position `(cursor_line_idx,cursor_char_idx)` if
///   deletes successfully.
/// - It returns `None` if delete nothing.
pub fn cursor_delete(
  tree: &mut Tree,
  id: TreeNodeId,
  text: &mut Text,
  n: isize,
) -> Option<(usize, usize)> {
  let cursor_viewport = tree.editable_cursor_viewport(id);

  // Delete N-chars.
  let cursor_line_idx = cursor_viewport.line_idx();
  let cursor_char_idx = cursor_viewport.char_idx();
  debug_assert!(text.rope().get_line(cursor_line_idx).is_some());

  // If line is empty, cursor cannot delete any text content.
  if cursor_char_idx >= text.rope().line(cursor_line_idx).len_chars() {
    return None;
  }

  debug_assert!(
    cursor_char_idx < text.rope().line(cursor_line_idx).len_chars()
  );
  let maybe_new_cursor_position =
    text.delete_at(cursor_line_idx, cursor_char_idx, n);

  maybe_new_cursor_position?;

  // Update viewport since the buffer doesn't match the viewport.
  _update_viewport_after_text_changed(tree, id, text);
  let (cursor_line_idx_after_deleted, cursor_char_idx_after_deleted) =
    maybe_new_cursor_position.unwrap();

  trace!(
    "Move to deleted pos, line:{cursor_line_idx_after_deleted}, char:{cursor_char_idx_after_deleted}"
  );
  let op = Operation::CursorMoveTo((
    cursor_char_idx_after_deleted,
    cursor_line_idx_after_deleted,
  ));
  cursor_move(tree, id, text, op, true);

  Some((cursor_line_idx_after_deleted, cursor_char_idx_after_deleted))
}

/// Cursor widget jumps from current parent widget to another parent widget.
/// This API only changes its parent widget, without modifying cursor viewport.
/// When cursor jumps between different window/command-line widgets, this API
/// changes its parent widget only.
///
/// The `parent_id` must be either valid window ID or command-line ID.
pub fn cursor_jump(
  tree: &mut Tree,
  parent_id: TreeNodeId,
) -> Option<TreeNodeId> {
  let cursor_id = tree.cursor_id().unwrap();
  let old_parent_id = tree.parent_id(cursor_id).unwrap();

  let removed_id = match tree.node(old_parent_id).unwrap() {
    TreeNode::WindowContent(_content) => {
      debug_assert!(tree.parent_id(old_parent_id).is_some());
      debug_assert!(tree.current_window_id().is_some());
      let old_window_id = tree.parent_id(old_parent_id).unwrap();
      debug_assert_eq!(old_window_id, tree.current_window_id().unwrap());
      if old_window_id == parent_id {
        return None;
      }
      old_window_id
    }
    TreeNode::CmdlineInput(_input) => {
      debug_assert!(tree.parent_id(old_parent_id).is_some());
      let old_input_panel_id = tree.parent_id(old_parent_id).unwrap();
      debug_assert!(tree.parent_id(old_input_panel_id).is_some());
      let old_cmdline_id = tree.parent_id(old_input_panel_id).unwrap();
      if old_cmdline_id == parent_id {
        return None;
      }
      old_cmdline_id
    }
    _ => unreachable!(),
  };

  tree
    .context()
    .borrow_mut()
    .remove_child(old_parent_id, cursor_id)
    .unwrap();

  match tree.node_mut(parent_id).unwrap() {
    TreeNode::Window(window) => {
      let content_id = window.content_id();
      tree
        .context()
        .borrow_mut()
        .add_child(content_id, cursor_id)
        .unwrap();
    }
    TreeNode::Cmdline(cmdline) => {
      let input_id = cmdline.input_id();
      tree
        .context()
        .borrow_mut()
        .add_child(input_id, cursor_id)
        .unwrap();
    }
    _ => unreachable!(),
  }

  Some(removed_id)
}
