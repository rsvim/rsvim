//! Cursor edit operations.

use crate::buf::text::Text;
use crate::coord::*;
use crate::state::ops::{Operation, cursor_move_ops};
use crate::ui::tree::*;
use crate::ui::viewport::{CursorViewport, CursorViewportArc, Viewport, ViewportArc, Viewportable};
use crate::ui::widget::command_line::CommandLine;
use crate::ui::widget::window::{Window, WindowLocalOptions};

use compact_str::{CompactString, ToCompactString};
use tracing::trace;

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
  if let Some(updated_cursor_viewport) = cursor_move_ops::cursor_move_to(
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
    insert_at_cursor(&cursor_viewport, text, payload);
  // Update viewport since the buffer doesn't match the viewport.
  update_viewport_after_text_changed(tree, cursor_parent_id, text);

  trace!(
    "Move to inserted pos, line:{cursor_line_idx_after_inserted}, char:{cursor_char_idx_after_inserted}"
  );
  let op = Operation::CursorMoveTo((
    cursor_char_idx_after_inserted,
    cursor_line_idx_after_inserted,
  ));
  cursor_move_ops::cursor_move(tree, text, op, true);
}
