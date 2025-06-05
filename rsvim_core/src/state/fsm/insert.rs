//! The insert mode.

use crate::buf::{Buffer, BufferWk, EndOfLineOption};
use crate::lock;
use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::ops::Operation;
use crate::state::ops::cursor_ops::{self, CursorMoveDirection};
use crate::ui::canvas::CursorStyle;
use crate::ui::tree::*;
use crate::ui::widget::window::{
  CursorViewport, Viewport, ViewportArc, ViewportSearchAnchorDirection,
};

use compact_str::ToCompactString;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use tracing::trace;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The finite-state-machine for insert mode.
pub struct InsertStateful {}

impl Stateful for InsertStateful {
  fn handle(&self, data_access: StatefulDataAccess) -> StatefulValue {
    let event = data_access.event.clone();

    match event {
      Event::FocusGained => {}
      Event::FocusLost => {}
      Event::Key(key_event) => match key_event.kind {
        KeyEventKind::Press => {
          trace!("Event::key:{:?}", key_event);
          match key_event.code {
            KeyCode::Up => {
              return self.cursor_move(&data_access, Operation::CursorMoveUpBy(1));
            }
            KeyCode::Down => {
              return self.cursor_move(&data_access, Operation::CursorMoveDownBy(1));
            }
            KeyCode::Left => {
              return self.cursor_move(&data_access, Operation::CursorMoveLeftBy(1));
            }
            KeyCode::Right => {
              return self.cursor_move(&data_access, Operation::CursorMoveRightBy(1));
            }
            KeyCode::Home => {
              return self.cursor_move(&data_access, Operation::CursorMoveLeftBy(usize::MAX));
            }
            KeyCode::End => {
              return self.cursor_move(&data_access, Operation::CursorMoveRightBy(usize::MAX));
            }
            KeyCode::Char(c) => {
              return self.insert_text(
                &data_access,
                Operation::InsertLineWiseTextAtCursor(c.to_compact_string()),
              );
            }
            KeyCode::Esc => {
              return self.goto_normal_mode(&data_access, Operation::GotoNormalMode);
            }
            _ => { /* Skip */ }
          }
        }
        KeyEventKind::Repeat => {}
        KeyEventKind::Release => {}
      },
      Event::Mouse(_mouse_event) => {}
      Event::Paste(ref _paste_string) => {}
      Event::Resize(_columns, _rows) => {}
    }

    StatefulValue::InsertMode(InsertStateful::default())
  }
}

#[derive(Debug, Copy, Clone)]
struct CursorMoveImplOptions {
  pub include_empty_eol: bool,
}

impl CursorMoveImplOptions {
  pub fn include_empty_eol() -> Self {
    Self {
      include_empty_eol: true,
    }
  }

  pub fn exclude_empty_eol() -> Self {
    Self {
      include_empty_eol: false,
    }
  }
}
impl InsertStateful {
  fn insert_text(&self, data_access: &StatefulDataAccess, op: Operation) -> StatefulValue {
    debug_assert!(matches!(op, Operation::InsertLineWiseTextAtCursor(_)));
    let text = match op {
      Operation::InsertLineWiseTextAtCursor(t) => t,
      _ => unreachable!(),
    };

    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);
    let buffer = self._current_buffer(&mut tree);
    let buffer = buffer.upgrade().unwrap();
    let mut buffer = lock!(buffer);

    // Insert text.
    let (cursor_line_idx_after_inserted, cursor_char_idx_after_inserted) = {
      if let Some(current_window_id) = tree.current_window_id() {
        if let Some(TreeNode::Window(current_window)) = tree.node_mut(current_window_id) {
          let cursor_viewport = current_window.cursor_viewport();
          let cursor_line_idx = cursor_viewport.line_idx();
          let cursor_char_idx = cursor_viewport.char_idx();
          debug_assert!(buffer.get_rope().get_line(cursor_line_idx).is_some());
          let start_char_pos_of_line = buffer.get_rope().line_to_char(cursor_line_idx);
          let before_insert_char_idx = start_char_pos_of_line + cursor_char_idx;
          if cfg!(debug_assertions) {
            use crate::test::buf::bufline_to_string;

            let b = before_insert_char_idx;
            let b1 = before_insert_char_idx.saturating_sub(1);
            let b2 = before_insert_char_idx.saturating_add(1);
            trace!(
              "Before buffer inserted, cursor_line_idx:{cursor_line_idx},before_insert_char_idx({b}):{:?}, before({b1}):{:?}, after({b2}):{:?}",
              buffer
                .get_rope()
                .get_char(b)
                .map(|c| c.to_string())
                .unwrap_or("null?".to_string()),
              buffer
                .get_rope()
                .get_char(b1)
                .map(|c| c.to_string())
                .unwrap_or("null?".to_string()),
              buffer
                .get_rope()
                .get_char(b2)
                .map(|c| c.to_string())
                .unwrap_or("null?".to_string()),
            );
            for i in 0..buffer.get_rope().len_lines() {
              trace!(
                "line:{i}:{:?}",
                bufline_to_string(&buffer.get_rope().line(i))
              );
            }
          }

          buffer
            .get_rope_mut()
            .insert(before_insert_char_idx, text.as_str());
          buffer
            .truncate_cached_line_since_char(cursor_line_idx, cursor_char_idx.saturating_sub(1));
          let after_inserted_char_idx = cursor_char_idx + text.len();

          // For text mode (different from the 'binary' mode, i.e. bin/hex mode), the editor have
          // to always keep an eol (end-of-line) at the end of text file. It helps the cursor
          // motion.
          if cursor_line_idx == buffer.get_rope().len_lines().saturating_sub(1) {
            use crate::defaults::ascii::end_of_line as eol;

            let buf_eol = buffer.options().end_of_line();
            let bufline = buffer.get_rope().line(cursor_line_idx);
            let bufline_len_chars = bufline.len_chars();

            if bufline_len_chars == 0 {
              buffer
                .get_rope_mut()
                .insert(0_usize, buf_eol.to_compact_string().as_str());
              buffer.remove_cached_line(cursor_line_idx);

              if cfg!(debug_assertions) {
                use crate::test::buf::bufline_to_string;

                let b = before_insert_char_idx;
                let b1 = before_insert_char_idx.saturating_sub(1);
                let b2 = before_insert_char_idx.saturating_add(1);
                trace!(
                  "Eol inserted (line=0), cursor_line_idx:{cursor_line_idx},before_insert_char_idx({b}):{:?}, before({b1}):{:?}, after({b2}):{:?}",
                  buffer
                    .get_rope()
                    .get_char(b)
                    .map(|c| c.to_string())
                    .unwrap_or("null?".to_string()),
                  buffer
                    .get_rope()
                    .get_char(b1)
                    .map(|c| c.to_string())
                    .unwrap_or("null?".to_string()),
                  buffer
                    .get_rope()
                    .get_char(b2)
                    .map(|c| c.to_string())
                    .unwrap_or("null?".to_string()),
                );
                for i in 0..buffer.get_rope().len_lines() {
                  trace!(
                    "line:{i}:{:?}",
                    bufline_to_string(&buffer.get_rope().line(i))
                  );
                }
              }
            } else {
              let bufline_start_char_pos = buffer.get_rope().line_to_char(cursor_line_idx);
              let bufline_insert_char_pos = bufline_start_char_pos + bufline_len_chars;

              let last1 = bufline.char(bufline_len_chars - 1);
              if last1.to_compact_string() != eol::CR || last1.to_compact_string() != eol::LF {
                buffer.get_rope_mut().insert(
                  bufline_insert_char_pos,
                  buf_eol.to_compact_string().as_str(),
                );
                buffer.truncate_cached_line_since_char(
                  cursor_line_idx,
                  bufline_len_chars.saturating_sub(1),
                );
                if cfg!(debug_assertions) {
                  use crate::test::buf::bufline_to_string;

                  let b = before_insert_char_idx;
                  let b1 = before_insert_char_idx.saturating_sub(1);
                  let b2 = before_insert_char_idx.saturating_add(1);
                  trace!(
                    "Eol inserted (last=1), cursor_line_idx:{cursor_line_idx},before_insert_char_idx({b}):{:?}, before({b1}):{:?}, after({b2}):{:?}",
                    buffer
                      .get_rope()
                      .get_char(b)
                      .map(|c| c.to_string())
                      .unwrap_or("null?".to_string()),
                    buffer
                      .get_rope()
                      .get_char(b1)
                      .map(|c| c.to_string())
                      .unwrap_or("null?".to_string()),
                    buffer
                      .get_rope()
                      .get_char(b2)
                      .map(|c| c.to_string())
                      .unwrap_or("null?".to_string()),
                  );
                  for i in 0..buffer.get_rope().len_lines() {
                    trace!(
                      "line:{i}:{:?}",
                      bufline_to_string(&buffer.get_rope().line(i))
                    );
                  }
                }
              } else if bufline_len_chars >= 2 {
                let last2 = format!("{}{}", bufline.char(bufline_len_chars - 2), last1);
                if last2 != eol::CRLF {
                  buffer.get_rope_mut().insert(
                    bufline_insert_char_pos,
                    buf_eol.to_compact_string().as_str(),
                  );
                  buffer.truncate_cached_line_since_char(
                    cursor_line_idx,
                    bufline_len_chars.saturating_sub(1),
                  );
                  if cfg!(debug_assertions) {
                    use crate::test::buf::bufline_to_string;

                    let b = before_insert_char_idx;
                    let b1 = before_insert_char_idx.saturating_sub(1);
                    let b2 = before_insert_char_idx.saturating_add(1);
                    trace!(
                      "Eol inserted (last=2), cursor_line_idx:{cursor_line_idx},before_insert_char_idx({b}):{:?}, before({b1}):{:?}, after({b2}):{:?}",
                      buffer
                        .get_rope()
                        .get_char(b)
                        .map(|c| c.to_string())
                        .unwrap_or("null?".to_string()),
                      buffer
                        .get_rope()
                        .get_char(b1)
                        .map(|c| c.to_string())
                        .unwrap_or("null?".to_string()),
                      buffer
                        .get_rope()
                        .get_char(b2)
                        .map(|c| c.to_string())
                        .unwrap_or("null?".to_string()),
                    );
                    for i in 0..buffer.get_rope().len_lines() {
                      trace!(
                        "line:{i}:{:?}",
                        bufline_to_string(&buffer.get_rope().line(i))
                      );
                    }
                  }
                }
              }
            }
          }

          if cfg!(debug_assertions) {
            use crate::test::buf::bufline_to_string;

            let a = after_inserted_char_idx;
            let a1 = a.saturating_sub(1);
            let a2 = a.saturating_add(1);
            match buffer.get_rope().get_line(cursor_line_idx) {
              Some(line) => trace!(
                "After buffer inserted, cursor_line_idx:{cursor_line_idx},after_inserted_char_idx({a}):{:?}, before({a1}):{:?}, after({a2}):{:?}",
                line
                  .get_char(a)
                  .map(|c| c.to_string())
                  .unwrap_or("null-c?".to_string()),
                line
                  .get_char(a1)
                  .map(|c| c.to_string())
                  .unwrap_or("null-c?".to_string()),
                line
                  .get_char(a2)
                  .map(|c| c.to_string())
                  .unwrap_or("null-c?".to_string()),
              ),
              None => trace!(
                "After buffer inserted, cursor_line_idx:{cursor_line_idx}:null-l,after_inserted_char_idx({a}), before({a1}), after({a2})"
              ),
            }
            for i in 0..buffer.get_rope().len_lines() {
              trace!(
                "line:{i}:{:?}",
                bufline_to_string(&buffer.get_rope().line(i))
              );
            }
          }
          (cursor_line_idx, after_inserted_char_idx)
        } else {
          unreachable!()
        }
      } else {
        unreachable!()
      }
    };

    // Update viewport since the buffer doesn't match the viewport.
    if let Some(current_window_id) = tree.current_window_id() {
      if let Some(TreeNode::Window(current_window)) = tree.node_mut(current_window_id) {
        let viewport = current_window.viewport();
        let cursor_viewport = current_window.cursor_viewport();
        trace!("before viewport:{:?}", viewport);
        trace!("before cursor_viewport:{:?}", cursor_viewport);

        let start_line = std::cmp::min(
          viewport.start_line_idx(),
          buffer.get_rope().len_lines().saturating_sub(1),
        );
        debug_assert!(buffer.get_rope().get_line(start_line).is_some());
        let bufline_len_chars = buffer.get_rope().line(start_line).len_chars();
        let start_column = std::cmp::min(
          viewport.start_column_idx(),
          buffer.width_before(start_line, bufline_len_chars),
        );

        let updated_viewport = Viewport::to_arc(Viewport::view(
          &buffer,
          current_window.actual_shape(),
          current_window.options(),
          start_line,
          start_column,
        ));
        trace!("after updated_viewport:{:?}", updated_viewport);

        current_window.set_viewport(updated_viewport.clone());
        if let Some(updated_cursor_viewport) = cursor_ops::cursor_move_to(
          &updated_viewport,
          &cursor_viewport,
          &buffer,
          Operation::CursorMoveTo((cursor_viewport.char_idx(), cursor_viewport.line_idx())),
        ) {
          trace!(
            "after updated_cursor_viewport:{:?}",
            updated_cursor_viewport
          );
          current_window.set_cursor_viewport(updated_cursor_viewport);
        }
      } else {
        unreachable!();
      }
    } else {
      unreachable!();
    }

    trace!(
      "Move to inserted pos, line:{cursor_line_idx_after_inserted}, char:{cursor_char_idx_after_inserted}"
    );
    self._cursor_move_impl(
      CursorMoveImplOptions::include_empty_eol(),
      &mut tree,
      &buffer,
      Operation::CursorMoveTo((
        cursor_char_idx_after_inserted,
        cursor_line_idx_after_inserted,
      )),
    );

    StatefulValue::InsertMode(InsertStateful::default())
  }

  fn _current_buffer(&self, tree: &mut Tree) -> BufferWk {
    if let Some(current_window_id) = tree.current_window_id() {
      if let Some(TreeNode::Window(current_window)) = tree.node_mut(current_window_id) {
        current_window.buffer()
      } else {
        unreachable!()
      }
    } else {
      unreachable!()
    }
  }
}

impl InsertStateful {
  fn goto_normal_mode(&self, data_access: &StatefulDataAccess, _op: Operation) -> StatefulValue {
    debug_assert!(matches!(_op, Operation::GotoNormalMode));

    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);
    let buffer = self._current_buffer(&mut tree);
    let buffer = buffer.upgrade().unwrap();
    let buffer = lock!(buffer);

    self._cursor_move_impl(
      CursorMoveImplOptions::exclude_empty_eol(),
      &mut tree,
      &buffer,
      Operation::CursorMoveBy((0, 0)),
    );

    let cursor_id = tree.cursor_id().unwrap();
    if let Some(TreeNode::Cursor(cursor)) = tree.node_mut(cursor_id) {
      cursor.set_style(&CursorStyle::SteadyBlock);
    } else {
      unreachable!()
    }

    StatefulValue::NormalMode(super::NormalStateful::default())
  }
}

impl InsertStateful {
  fn cursor_move(&self, data_access: &StatefulDataAccess, op: Operation) -> StatefulValue {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);
    let buffer = self._current_buffer(&mut tree);
    let buffer = buffer.upgrade().unwrap();
    let buffer = lock!(buffer);

    self._cursor_move_impl(
      CursorMoveImplOptions::include_empty_eol(),
      &mut tree,
      &buffer,
      op,
    );

    StatefulValue::InsertMode(InsertStateful::default())
  }

  fn _cursor_move_impl(
    &self,
    opts: CursorMoveImplOptions,
    tree: &mut Tree,
    buffer: &Buffer,
    op: Operation,
  ) {
    if let Some(current_window_id) = tree.current_window_id() {
      if let Some(TreeNode::Window(current_window)) = tree.node_mut(current_window_id) {
        let viewport = current_window.viewport();
        let cursor_viewport = current_window.cursor_viewport();

        // Only move cursor when it is different from current cursor.
        let (target_cursor_char, target_cursor_line, search_direction) =
          self._target_cursor_considering_empty_eol(opts, &cursor_viewport, buffer, op);

        let new_viewport: Option<ViewportArc> = {
          let (start_line, start_column) = viewport.search_anchor(
            search_direction,
            buffer,
            current_window.actual_shape(),
            current_window.options(),
            target_cursor_line,
            target_cursor_char,
          );

          // First try window scroll.
          if start_line != viewport.start_line_idx() || start_column != viewport.start_column_idx()
          {
            let new_viewport = cursor_ops::window_scroll_to(
              &viewport,
              current_window,
              buffer,
              Operation::WindowScrollTo((start_column, start_line)),
            );
            if let Some(new_viewport_arc) = new_viewport.clone() {
              current_window.set_viewport(new_viewport_arc.clone());
            }
            new_viewport
          } else {
            None
          }
        };

        // Then try cursor move.
        {
          let current_viewport = new_viewport.unwrap_or(viewport);

          let new_cursor_viewport = cursor_ops::cursor_move_to(
            &current_viewport,
            &cursor_viewport,
            buffer,
            Operation::CursorMoveTo((target_cursor_char, target_cursor_line)),
          );

          if let Some(new_cursor_viewport) = new_cursor_viewport {
            current_window.set_cursor_viewport(new_cursor_viewport.clone());
            let cursor_id = tree.cursor_id().unwrap();
            tree.bounded_move_to(
              cursor_id,
              new_cursor_viewport.column_idx() as isize,
              new_cursor_viewport.row_idx() as isize,
            );
          }
        }
      } else {
        unreachable!()
      }
    } else {
      unreachable!()
    }
  }

  // Returns `(target_cursor_char, target_cursor_line, viewport_search_direction)`.
  fn _target_cursor_considering_empty_eol(
    &self,
    opts: CursorMoveImplOptions,
    cursor_viewport: &CursorViewport,
    buffer: &Buffer,
    op: Operation,
  ) -> (usize, usize, ViewportSearchAnchorDirection) {
    let (target_cursor_char, target_cursor_line, move_direction) = if opts.include_empty_eol {
      cursor_ops::normalize_to_cursor_move_to_include_empty_eol(
        buffer,
        op,
        cursor_viewport.char_idx(),
        cursor_viewport.line_idx(),
      )
    } else {
      cursor_ops::normalize_to_cursor_move_to_exclude_empty_eol(
        buffer,
        op,
        cursor_viewport.char_idx(),
        cursor_viewport.line_idx(),
      )
    };

    let search_direction = match move_direction {
      CursorMoveDirection::Up => ViewportSearchAnchorDirection::Up,
      CursorMoveDirection::Down => ViewportSearchAnchorDirection::Down,
      CursorMoveDirection::Left => ViewportSearchAnchorDirection::Left,
      CursorMoveDirection::Right => ViewportSearchAnchorDirection::Right,
    };
    (target_cursor_char, target_cursor_line, search_direction)
  }
}

// spellchecker:off
#[cfg(test)]
#[allow(unused_imports)]
mod tests_util {
  use super::*;

  use crate::buf::{BufferArc, BufferLocalOptionsBuilder, BuffersManagerArc};
  use crate::lock;
  use crate::prelude::*;
  use crate::state::{State, StateArc};
  use crate::test::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::test::log::init as test_log_init;
  use crate::test::tree::make_tree_with_buffers;
  use crate::ui::canvas::Canvas;
  use crate::ui::tree::TreeArc;
  use crate::ui::widget::Widgetable;
  use crate::ui::widget::window::content::WindowContent;
  use crate::ui::widget::window::{
    CursorViewport, CursorViewportArc, Viewport, ViewportArc, WindowLocalOptions,
    WindowLocalOptionsBuilder,
  };

  use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
  use std::collections::BTreeMap;
  use std::sync::Arc;
  use tracing::info;

  pub fn make_tree(
    terminal_size: U16Size,
    window_local_opts: WindowLocalOptions,
    lines: Vec<&str>,
  ) -> (TreeArc, StateArc, BuffersManagerArc, BufferArc) {
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let buf = make_buffer_from_lines(terminal_size.height(), buf_opts, lines);
    let bufs = make_buffers_manager(buf_opts, vec![buf.clone()]);
    let tree = make_tree_with_buffers(terminal_size, window_local_opts, bufs.clone());
    let state = State::to_arc(State::default());
    (tree, state, bufs, buf)
  }

  pub fn get_viewport(tree: TreeArc) -> ViewportArc {
    let tree = lock!(tree);
    let current_window_id = tree.current_window_id().unwrap();
    let current_window_node = tree.node(current_window_id).unwrap();
    assert!(matches!(current_window_node, TreeNode::Window(_)));
    match current_window_node {
      TreeNode::Window(current_window) => current_window.viewport(),
      _ => unreachable!(),
    }
  }

  pub fn get_cursor_viewport(tree: TreeArc) -> CursorViewportArc {
    let tree = lock!(tree);
    let current_window_id = tree.current_window_id().unwrap();
    let current_window_node = tree.node(current_window_id).unwrap();
    assert!(matches!(current_window_node, TreeNode::Window(_)));
    match current_window_node {
      TreeNode::Window(current_window) => current_window.cursor_viewport(),
      _ => unreachable!(),
    }
  }

  #[allow(clippy::too_many_arguments)]
  pub fn assert_viewport_scroll(
    buffer: BufferArc,
    actual: &Viewport,
    expect: &Vec<&str>,
    expect_start_line: usize,
    expect_end_line: usize,
    expect_start_fills: &BTreeMap<usize, usize>,
    expect_end_fills: &BTreeMap<usize, usize>,
  ) {
    info!(
      "actual start_line/end_line:{:?}/{:?}",
      actual.start_line_idx(),
      actual.end_line_idx()
    );
    info!(
      "expect start_line/end_line:{:?}/{:?}",
      expect_start_line, expect_end_line
    );
    for (k, v) in actual.lines().iter() {
      info!("actual line[{:?}]: {:?}", k, v);
    }
    for (i, e) in expect.iter().enumerate() {
      info!("expect line[{}]:{:?}", i, e);
    }
    assert_eq!(expect_start_fills.len(), expect_end_fills.len());
    for (k, start_v) in expect_start_fills.iter() {
      let end_v = expect_end_fills.get(k).unwrap();
      info!(
        "expect start_fills/end_fills line[{}]:{:?}/{:?}",
        k, start_v, end_v
      );
    }

    assert_eq!(actual.start_line_idx(), expect_start_line);
    assert_eq!(actual.end_line_idx(), expect_end_line);
    if actual.lines().is_empty() {
      assert!(actual.end_line_idx() <= actual.start_line_idx());
    } else {
      let (first_line_idx, _first_line_viewport) = actual.lines().first().unwrap();
      let (last_line_idx, _last_line_viewport) = actual.lines().last().unwrap();
      assert_eq!(*first_line_idx, actual.start_line_idx());
      assert_eq!(*last_line_idx, actual.end_line_idx() - 1);
    }
    assert_eq!(
      actual.end_line_idx() - actual.start_line_idx(),
      actual.lines().len()
    );
    assert_eq!(
      actual.end_line_idx() - actual.start_line_idx(),
      expect_start_fills.len()
    );
    assert_eq!(
      actual.end_line_idx() - actual.start_line_idx(),
      expect_end_fills.len()
    );

    let buffer = lock!(buffer);
    let buflines = buffer
      .get_rope()
      .get_lines_at(actual.start_line_idx())
      .unwrap();
    let total_lines = expect_end_line - expect_start_line;

    for (l, line) in buflines.enumerate() {
      if l >= total_lines {
        break;
      }
      let actual_line_idx = l + expect_start_line;
      let line_viewport = actual.lines().get(&actual_line_idx).unwrap();

      info!(
        "l-{:?}, actual_line_idx:{}, line_viewport:{:?}",
        l, actual_line_idx, line_viewport
      );
      info!(
        "start_filled_cols expect:{:?}, actual:{}",
        expect_start_fills.get(&actual_line_idx),
        line_viewport.start_filled_cols()
      );
      assert_eq!(
        line_viewport.start_filled_cols(),
        *expect_start_fills.get(&actual_line_idx).unwrap()
      );
      info!(
        "end_filled_cols expect:{:?}, actual:{}",
        expect_end_fills.get(&actual_line_idx),
        line_viewport.end_filled_cols()
      );
      assert_eq!(
        line_viewport.end_filled_cols(),
        *expect_end_fills.get(&actual_line_idx).unwrap()
      );

      let rows = &line_viewport.rows();
      for (r, row) in rows.iter() {
        info!("row-index-{:?}, row:{:?}", r, row);

        if r > rows.first().unwrap().0 {
          let prev_r = r - 1;
          let prev_row = rows.get(&prev_r).unwrap();
          info!(
            "row-{:?}, current[{}]:{:?}, previous[{}]:{:?}",
            r, r, row, prev_r, prev_row
          );
        }
        if r < rows.last().unwrap().0 {
          let next_r = r + 1;
          let next_row = rows.get(&next_r).unwrap();
          info!(
            "row-{:?}, current[{}]:{:?}, next[{}]:{:?}",
            r, r, row, next_r, next_row
          );
        }

        let mut payload = String::new();
        for c_idx in row.start_char_idx()..row.end_char_idx() {
          let c = line.get_char(c_idx).unwrap();
          payload.push(c);
        }
        info!(
          "row-{:?}, payload actual:{:?}, expect:{:?}",
          r, payload, expect[*r as usize]
        );
        assert_eq!(payload, expect[*r as usize]);
      }
    }
  }

  pub fn make_canvas(
    terminal_size: U16Size,
    window_options: WindowLocalOptions,
    buffer: BufferArc,
    viewport: ViewportArc,
  ) -> Canvas {
    let mut tree = Tree::new(terminal_size);
    tree.set_global_local_options(&window_options);
    let shape = IRect::new(
      (0, 0),
      (
        terminal_size.width() as isize,
        terminal_size.height() as isize,
      ),
    );
    let window_content =
      WindowContent::new(shape, Arc::downgrade(&buffer), Arc::downgrade(&viewport));
    let mut canvas = Canvas::new(terminal_size);
    window_content.draw(&mut canvas);
    canvas
  }

  pub fn assert_canvas(actual: &Canvas, expect: &[&str]) {
    let actual = actual
      .frame()
      .raw_symbols()
      .iter()
      .map(|cs| cs.join(""))
      .collect::<Vec<_>>();
    info!("actual:{}", actual.len());
    for a in actual.iter() {
      info!("{:?}", a);
    }
    info!("expect:{}", expect.len());
    for e in expect.iter() {
      info!("{:?}", e);
    }

    assert_eq!(actual.len(), expect.len());
    for i in 0..actual.len() {
      let e = &expect[i];
      let a = &actual[i];
      info!("i-{}, actual[{}]:{:?}, expect[{}]:{:?}", i, i, a, i, e);
      assert_eq!(e.len(), a.len());
      assert_eq!(e, a);
    }
  }
}
#[cfg(test)]
#[allow(unused_imports)]
mod tests_cursor_move {
  use super::tests_util::*;
  use super::*;

  use crate::buf::{BufferArc, BufferLocalOptionsBuilder, BuffersManagerArc};
  use crate::lock;
  use crate::prelude::*;
  use crate::state::{State, StateArc};
  use crate::test::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::test::log::init as test_log_init;
  use crate::test::tree::make_tree_with_buffers;
  use crate::ui::tree::TreeArc;
  use crate::ui::widget::window::{Viewport, WindowLocalOptions, WindowLocalOptionsBuilder};

  use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
  use std::collections::BTreeMap;
  use tracing::info;

  #[test]
  fn nowrap1() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let (tree, state, bufs, buf) = make_tree(
      U16Size::new(10, 10),
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      lines,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );
    let data_access = StatefulDataAccess::new(state, tree.clone(), bufs, Event::Key(key_event));
    let stateful = InsertStateful::default();

    // Move-1
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveBy((5, 3)));

      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 3);
      assert_eq!(actual1.char_idx(), 5);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "     * The",
        "     * The",
        "",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![
        (0, 0),
        (1, 0),
        (2, 0),
        (3, 0),
        (4, 0),
        (5, 0),
        (6, 0),
        (7, 0),
      ]
      .into_iter()
      .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        8,
        &expect_fills,
        &expect_fills,
      );
    }

    // Move-2
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(158));

      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 3);
      assert_eq!(actual2.char_idx(), 158);
      assert_eq!(actual2.row_idx(), 3);
      assert_eq!(actual2.column_idx(), 9);

      let viewport = get_viewport(tree.clone());
      let expect = vec!["", "", "", "endering.\n", "", "", "ut in the ", ""];
      let expect_fills: BTreeMap<usize, usize> = vec![
        (0, 0),
        (1, 0),
        (2, 0),
        (3, 0),
        (4, 0),
        (5, 0),
        (6, 0),
        (7, 0),
      ]
      .into_iter()
      .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        8,
        &expect_fills,
        &expect_fills,
      );
    }
  }

  #[test]
  fn wrap_nolinebreak1() {
    test_log_init();

    let lines = vec![
      "AAAAAAAAAA\n",
      "1st.\n",
      "2nd.\n",
      "3rd.\n",
      "4th.\n",
      "5th.\n",
      "6th.\n",
      "BBBBBBBBBBCCCCCCCCCC\n",
      "8th.\n",
      "9th.\n",
      "10th.\n",
      "11th.\n",
      "12th.\n",
    ];
    let (tree, state, bufs, buf) = make_tree(
      U16Size::new(10, 6),
      WindowLocalOptionsBuilder::default()
        .wrap(true)
        .line_break(false)
        .build()
        .unwrap(),
      lines,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );
    let data_access = StatefulDataAccess::new(state, tree.clone(), bufs, Event::Key(key_event));
    let stateful = InsertStateful::default();

    // Move-1
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveBy((3, 2)));
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 2);
      assert_eq!(actual1.char_idx(), 3);
      assert_eq!(actual1.row_idx(), 2);
      assert_eq!(actual1.column_idx(), 3);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "AAAAAAAAAA",
        "1st.\n",
        "2nd.\n",
        "3rd.\n",
        "4th.\n",
        "5th.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0)]
          .into_iter()
          .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        6,
        &expect_fills,
        &expect_fills,
      );
    }

    // Move-2
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(2));
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 2);
      assert_eq!(actual2.char_idx(), 4);
      assert_eq!(actual2.row_idx(), 2);
      assert_eq!(actual2.column_idx(), 4);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "AAAAAAAAAA",
        "1st.\n",
        "2nd.\n",
        "3rd.\n",
        "4th.\n",
        "5th.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0)]
          .into_iter()
          .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        6,
        &expect_fills,
        &expect_fills,
      );
    }

    // Move-3
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveTo((10, 0)));
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 0);
      assert_eq!(actual2.char_idx(), 10);
      assert_eq!(actual2.row_idx(), 1);
      assert_eq!(actual2.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "AAAAAAAAAA",
        "1st.\n",
        "2nd.\n",
        "3rd.\n",
        "4th.\n",
        "5th.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0)]
          .into_iter()
          .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        6,
        &expect_fills,
        &expect_fills,
      );
    }

    // Move-4
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveTo((0, 7)));
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 7);
      assert_eq!(actual2.char_idx(), 0);
      assert_eq!(actual2.row_idx(), 4);
      assert_eq!(actual2.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "3rd.\n",
        "4th.\n",
        "5th.\n",
        "6th.\n",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        3,
        8,
        &expect_fills,
        &expect_fills,
      );
    }

    // Move-5
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(13));
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 7);
      assert_eq!(actual2.char_idx(), 13);
      assert_eq!(actual2.row_idx(), 5);
      assert_eq!(actual2.column_idx(), 3);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "3rd.\n",
        "4th.\n",
        "5th.\n",
        "6th.\n",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        3,
        8,
        &expect_fills,
        &expect_fills,
      );
    }

    // Move-6
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(7));
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 7);
      assert_eq!(actual2.char_idx(), 20);
      assert_eq!(actual2.row_idx(), 5);
      assert_eq!(actual2.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "4th.\n",
        "5th.\n",
        "6th.\n",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
        "8th.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(4, 0), (5, 0), (6, 0), (7, 0), (8, 0)]
        .into_iter()
        .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        4,
        9,
        &expect_fills,
        &expect_fills,
      );
    }
  }

  #[test]
  fn wrap_linebreak1() {
    test_log_init();

    let lines = vec![
      "AAAAAAAAAA\n",
      "1st.\n",
      "2nd.\n",
      "3rd.\n",
      "4th.\n",
      "5th.\n",
      "6th.\n",
      "BBBBBBBBBBCCCCCCCCCC\n",
      "8th.\n",
      "9th.\n",
      "10th.\n",
      "11th.\n",
      "12th.\n",
    ];
    let (tree, state, bufs, buf) = make_tree(
      U16Size::new(10, 6),
      WindowLocalOptionsBuilder::default()
        .wrap(true)
        .line_break(true)
        .build()
        .unwrap(),
      lines,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );
    let data_access = StatefulDataAccess::new(state, tree.clone(), bufs, Event::Key(key_event));
    let stateful = InsertStateful::default();

    // Move-1
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveBy((3, 2)));
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 2);
      assert_eq!(actual1.char_idx(), 3);
      assert_eq!(actual1.row_idx(), 2);
      assert_eq!(actual1.column_idx(), 3);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "AAAAAAAAAA",
        "1st.\n",
        "2nd.\n",
        "3rd.\n",
        "4th.\n",
        "5th.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0)]
          .into_iter()
          .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        6,
        &expect_fills,
        &expect_fills,
      );
    }

    // Move-2
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(2));
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 2);
      assert_eq!(actual2.char_idx(), 4);
      assert_eq!(actual2.row_idx(), 2);
      assert_eq!(actual2.column_idx(), 4);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "AAAAAAAAAA",
        "1st.\n",
        "2nd.\n",
        "3rd.\n",
        "4th.\n",
        "5th.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0)]
          .into_iter()
          .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        6,
        &expect_fills,
        &expect_fills,
      );
    }

    // Move-3
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveTo((10, 0)));
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 0);
      assert_eq!(actual2.char_idx(), 10);
      assert_eq!(actual2.row_idx(), 1);
      assert_eq!(actual2.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "AAAAAAAAAA",
        "1st.\n",
        "2nd.\n",
        "3rd.\n",
        "4th.\n",
        "5th.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0)]
          .into_iter()
          .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        6,
        &expect_fills,
        &expect_fills,
      );
    }

    // Move-4
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveTo((0, 7)));
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 7);
      assert_eq!(actual2.char_idx(), 0);
      assert_eq!(actual2.row_idx(), 4);
      assert_eq!(actual2.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "3rd.\n",
        "4th.\n",
        "5th.\n",
        "6th.\n",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        3,
        8,
        &expect_fills,
        &expect_fills,
      );
    }

    // Move-5
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(13));
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 7);
      assert_eq!(actual2.char_idx(), 13);
      assert_eq!(actual2.row_idx(), 5);
      assert_eq!(actual2.column_idx(), 3);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "3rd.\n",
        "4th.\n",
        "5th.\n",
        "6th.\n",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        3,
        8,
        &expect_fills,
        &expect_fills,
      );
    }

    // Move-6
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(7));
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 7);
      assert_eq!(actual2.char_idx(), 20);
      assert_eq!(actual2.row_idx(), 5);
      assert_eq!(actual2.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "4th.\n",
        "5th.\n",
        "6th.\n",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
        "8th.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(4, 0), (5, 0), (6, 0), (7, 0), (8, 0)]
        .into_iter()
        .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        4,
        9,
        &expect_fills,
        &expect_fills,
      );
    }
  }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests_insert_text {
  use super::tests_util::*;
  use super::*;

  use crate::buf::{BufferArc, BufferLocalOptionsBuilder, BuffersManagerArc};
  use crate::lock;
  use crate::prelude::*;
  use crate::state::{State, StateArc};
  use crate::test::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::test::log::init as test_log_init;
  use crate::test::tree::make_tree_with_buffers;
  use crate::ui::tree::TreeArc;
  use crate::ui::widget::window::{Viewport, WindowLocalOptions, WindowLocalOptionsBuilder};

  use compact_str::CompactString;
  use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
  use std::collections::BTreeMap;
  use tracing::info;

  #[test]
  fn nowrap1() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let window_options = WindowLocalOptionsBuilder::default()
      .wrap(false)
      .build()
      .unwrap();
    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let (tree, state, bufs, buf) = make_tree(terminal_size, window_options, lines);

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );
    let data_access = StatefulDataAccess::new(state, tree.clone(), bufs, Event::Key(key_event));
    let stateful = InsertStateful::default();

    // Insert-1
    {
      stateful.insert_text(
        &data_access,
        Operation::InsertLineWiseTextAtCursor(CompactString::new("Bye, ")),
      );

      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 5);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 5);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Bye, Hello",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "     * The",
        "     * The",
        "",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![
        (0, 0),
        (1, 0),
        (2, 0),
        (3, 0),
        (4, 0),
        (5, 0),
        (6, 0),
        (7, 0),
      ]
      .into_iter()
      .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        8,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "Bye, Hello",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "     * The",
        "     * The",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Move-2
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(20));

      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 18);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 9);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "o, RSVIM!\n",
        " quite sim",
        " it contai",
        " the line ",
        " the line ",
        "e extra pa",
        "e extra pa",
        "",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![
        (0, 0),
        (1, 0),
        (2, 0),
        (3, 0),
        (4, 0),
        (5, 0),
        (6, 0),
        (7, 0),
      ]
      .into_iter()
      .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        8,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "o, RSVIM! ",
        " quite sim",
        " it contai",
        " the line ",
        " the line ",
        "e extra pa",
        "e extra pa",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Insert-3
    {
      stateful.insert_text(
        &data_access,
        Operation::InsertLineWiseTextAtCursor(CompactString::new(" Go!")),
      );

      let tree = data_access.tree.clone();
      let actual3 = get_cursor_viewport(tree.clone());
      assert_eq!(actual3.line_idx(), 0);
      assert_eq!(actual3.char_idx(), 22);
      assert_eq!(actual3.row_idx(), 0);
      assert_eq!(actual3.column_idx(), 9);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "SVIM! Go!\n",
        "te simple ",
        "contains s",
        " line is s",
        " line is t",
        "tra parts ",
        "tra parts ",
        "",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![
        (0, 0),
        (1, 0),
        (2, 0),
        (3, 0),
        (4, 0),
        (5, 0),
        (6, 0),
        (7, 0),
      ]
      .into_iter()
      .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        8,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "SVIM! Go! ",
        "te simple ",
        "contains s",
        " line is s",
        " line is t",
        "tra parts ",
        "tra parts ",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }
  }

  #[test]
  fn nowrap2() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let window_option = WindowLocalOptionsBuilder::default()
      .wrap(false)
      .build()
      .unwrap();
    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside.\n",
      "  2. When the line is too long to be completely put in.\n",
      "  3. Is there any other cases?\n",
    ];
    let (tree, state, bufs, buf) = make_tree(terminal_size, window_option, lines);

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );
    let data_access = StatefulDataAccess::new(state, tree.clone(), bufs, Event::Key(key_event));
    let stateful = InsertStateful::default();

    // Move-1
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(6));

      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 6);
      assert_eq!(actual1.char_idx(), 0);
      assert_eq!(actual1.row_idx(), 6);
      assert_eq!(actual1.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "  3. Is th",
        "",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0)]
          .into_iter()
          .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        7,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "  3. Is th",
        "          ",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_option, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Insert-2
    {
      stateful.insert_text(
        &data_access,
        Operation::InsertLineWiseTextAtCursor(CompactString::new("a")),
      );

      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 6);
      assert_eq!(actual2.char_idx(), 1);
      assert_eq!(actual2.row_idx(), 6);
      assert_eq!(actual2.column_idx(), 1);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "  3. Is th",
        "a\n",
        "",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![
        (0, 0),
        (1, 0),
        (2, 0),
        (3, 0),
        (4, 0),
        (5, 0),
        (6, 0),
        (7, 0),
      ]
      .into_iter()
      .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        8,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "  3. Is th",
        "a         ",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_option, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }
  }

  #[test]
  fn nowrap3() {
    test_log_init();

    let terminal_size = U16Size::new(10, 5);
    let window_option = WindowLocalOptionsBuilder::default()
      .wrap(false)
      .build()
      .unwrap();
    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside.\n",
      "  2. When the line is too long to be completely put in.\n",
      "  3. Is there any other cases?\n",
    ];
    let (tree, state, bufs, buf) = make_tree(terminal_size, window_option, lines);

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );
    let data_access = StatefulDataAccess::new(state, tree.clone(), bufs, Event::Key(key_event));
    let stateful = InsertStateful::default();

    // Move-1
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(6));

      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 6);
      assert_eq!(actual1.char_idx(), 0);
      assert_eq!(actual1.row_idx(), 4);
      assert_eq!(actual1.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec!["But still ", "  1. When ", "  2. When ", "  3. Is th", ""];
      let expect_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 0), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        2,
        7,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "But still ",
        "  1. When ",
        "  2. When ",
        "  3. Is th",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_option, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Insert-2
    {
      stateful.insert_text(
        &data_access,
        Operation::InsertLineWiseTextAtCursor(CompactString::new("a")),
      );

      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 6);
      assert_eq!(actual2.char_idx(), 1);
      assert_eq!(actual2.row_idx(), 4);
      assert_eq!(actual2.column_idx(), 1);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "But still ",
        "  1. When ",
        "  2. When ",
        "  3. Is th",
        "a\n",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 0), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        2,
        7,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "But still ",
        "  1. When ",
        "  2. When ",
        "  3. Is th",
        "a         ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_option, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }
  }

  #[test]
  fn wrap_nolinebreak1() {
    test_log_init();

    let terminal_size = U16Size::new(10, 6);
    let window_options = WindowLocalOptionsBuilder::default()
      .wrap(true)
      .line_break(false)
      .build()
      .unwrap();
    let lines = vec![
      "AAAAAAAAAA\n",
      "1st.\n",
      "2nd.\n",
      "3rd.\n",
      "4th.\n",
      "5th.\n",
      "6th.\n",
      "BBBBBBBBBBCCCCCCCCCC\n",
      "8th.\n",
      "9th.\n",
      "10th.\n",
      "11th.\n",
      "12th.\n",
    ];
    let (tree, state, bufs, buf) = make_tree(terminal_size, window_options, lines);

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );
    let data_access = StatefulDataAccess::new(state, tree.clone(), bufs, Event::Key(key_event));
    let stateful = InsertStateful::default();

    // Insert-1
    {
      stateful.insert_text(
        &data_access,
        Operation::InsertLineWiseTextAtCursor(CompactString::new("Hello, ")),
      );
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 7);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 7);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, AAA",
        "AAAAAAA\n",
        "1st.\n",
        "2nd.\n",
        "3rd.\n",
        "4th.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "Hello, AAA",
        "AAAAAAA   ",
        "1st.      ",
        "2nd.      ",
        "3rd.      ",
        "4th.      ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Move-2
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveBy((3, 2)));
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 2);
      assert_eq!(actual1.char_idx(), 4);
      assert_eq!(actual1.row_idx(), 3);
      assert_eq!(actual1.column_idx(), 4);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, AAA",
        "AAAAAAA\n",
        "1st.\n",
        "2nd.\n",
        "3rd.\n",
        "4th.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "Hello, AAA",
        "AAAAAAA   ",
        "1st.      ",
        "2nd.      ",
        "3rd.      ",
        "4th.      ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Insert-3
    {
      stateful.insert_text(
        &data_access,
        Operation::InsertLineWiseTextAtCursor(CompactString::new("World!")),
      );
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 2);
      assert_eq!(actual1.char_idx(), 10);
      assert_eq!(actual1.row_idx(), 4);
      assert_eq!(actual1.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, AAA",
        "AAAAAAA\n",
        "1st.\n",
        "2nd.World!",
        "3rd.\n",
        "4th.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "Hello, AAA",
        "AAAAAAA   ",
        "1st.      ",
        "2nd.World!",
        "3rd.      ",
        "4th.      ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Insert-4
    {
      stateful.insert_text(
        &data_access,
        Operation::InsertLineWiseTextAtCursor(CompactString::new("Go!")),
      );
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 2);
      assert_eq!(actual1.char_idx(), 13);
      assert_eq!(actual1.row_idx(), 4);
      assert_eq!(actual1.column_idx(), 3);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, AAA",
        "AAAAAAA\n",
        "1st.\n",
        "2nd.World!",
        "Go!\n",
        "3rd.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        4,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "Hello, AAA",
        "AAAAAAA   ",
        "1st.      ",
        "2nd.World!",
        "Go!       ",
        "3rd.      ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Move-5
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveTo((20, 7)));
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 7);
      assert_eq!(actual2.char_idx(), 20);
      assert_eq!(actual2.row_idx(), 5);
      assert_eq!(actual2.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "4th.\n",
        "5th.\n",
        "6th.\n",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
        "8th.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(4, 0), (5, 0), (6, 0), (7, 0), (8, 0)]
        .into_iter()
        .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        4,
        9,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "4th.      ",
        "5th.      ",
        "6th.      ",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
        "8th.      ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Insert-6
    {
      stateful.insert_text(
        &data_access,
        Operation::InsertLineWiseTextAtCursor(CompactString::new("DDDDDDDDDD")),
      );
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 7);
      assert_eq!(actual1.char_idx(), 30);
      assert_eq!(actual1.row_idx(), 5);
      assert_eq!(actual1.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "5th.\n",
        "6th.\n",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
        "DDDDDDDDDD",
        "8th.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(5, 0), (6, 0), (7, 0), (8, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        5,
        9,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "5th.      ",
        "6th.      ",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
        "DDDDDDDDDD",
        "8th.      ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Move-7
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveLeftBy(17));
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 7);
      assert_eq!(actual2.char_idx(), 13);
      assert_eq!(actual2.row_idx(), 3);
      assert_eq!(actual2.column_idx(), 3);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "5th.\n",
        "6th.\n",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
        "DDDDDDDDDD",
        "8th.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(5, 0), (6, 0), (7, 0), (8, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        5,
        9,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "5th.      ",
        "6th.      ",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
        "DDDDDDDDDD",
        "8th.      ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Insert-8
    {
      stateful.insert_text(
        &data_access,
        Operation::InsertLineWiseTextAtCursor(CompactString::new("abc")),
      );
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 7);
      assert_eq!(actual1.char_idx(), 16);
      assert_eq!(actual1.row_idx(), 3);
      assert_eq!(actual1.column_idx(), 6);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "5th.\n",
        "6th.\n",
        "BBBBBBBBBB",
        "CCCabcCCCC",
        "CCCDDDDDDD",
        "DDD\n",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(5, 0), (6, 0), (7, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        5,
        8,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "5th.      ",
        "6th.      ",
        "BBBBBBBBBB",
        "CCCabcCCCC",
        "CCCDDDDDDD",
        "DDD       ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }
  }

  #[test]
  fn wrap_nolinebreak2() {
    test_log_init();

    let terminal_size = U16Size::new(10, 6);
    let window_options = WindowLocalOptionsBuilder::default()
      .wrap(true)
      .line_break(false)
      .build()
      .unwrap();
    let lines = vec![];
    let (tree, state, bufs, buf) = make_tree(terminal_size, window_options, lines);

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );
    let data_access = StatefulDataAccess::new(state, tree.clone(), bufs, Event::Key(key_event));
    let stateful = InsertStateful::default();

    // Insert-1
    {
      stateful.insert_text(
        &data_access,
        Operation::InsertLineWiseTextAtCursor(CompactString::new("a")),
      );
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 1);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 1);

      let viewport = get_viewport(tree.clone());
      let expect = vec!["a\n", ""];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        2,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "a         ",
        "          ",
        "          ",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }
  }

  #[test]
  fn wrap_nolinebreak3() {
    test_log_init();

    let terminal_size = U16Size::new(10, 6);
    let window_options = WindowLocalOptionsBuilder::default()
      .wrap(true)
      .line_break(false)
      .build()
      .unwrap();
    let lines = vec![""];
    let (tree, state, bufs, buf) = make_tree(terminal_size, window_options, lines);

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );
    let data_access = StatefulDataAccess::new(state, tree.clone(), bufs, Event::Key(key_event));
    let stateful = InsertStateful::default();

    // Insert-1
    {
      stateful.insert_text(
        &data_access,
        Operation::InsertLineWiseTextAtCursor(CompactString::new("b")),
      );
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 1);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 1);

      let viewport = get_viewport(tree.clone());
      let expect = vec!["b\n", ""];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        2,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "b         ",
        "          ",
        "          ",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }
  }

  #[test]
  fn wrap_linebreak1() {
    test_log_init();

    let terminal_size = U16Size::new(10, 6);
    let window_options = WindowLocalOptionsBuilder::default()
      .wrap(true)
      .line_break(true)
      .build()
      .unwrap();
    let lines = vec![
      "AAAAAAAAAA\n",
      "1st line that we could make it longer.\n",
      "2nd line that we must make it shorter!\n",
      "3rd.\n",
      "4th.\n",
      "5th.\n",
      "6th.\n",
      "BBBBBBBBBBCCCCCCCCCC\n",
      "8th.\n",
      "9th.\n",
      "10th.\n",
    ];
    let (tree, state, bufs, buf) = make_tree(terminal_size, window_options, lines);

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );
    let data_access = StatefulDataAccess::new(state, tree.clone(), bufs, Event::Key(key_event));
    let stateful = InsertStateful::default();

    // Insert-1
    {
      stateful.insert_text(
        &data_access,
        Operation::InsertLineWiseTextAtCursor(CompactString::new("Hello, ")),
      );
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 7);
      assert_eq!(actual1.row_idx(), 1);
      assert_eq!(actual1.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, ",
        "AAAAAAAAAA",
        "1st line ",
        "that we ",
        "could make",
        " it longer",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        2,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "Hello,    ",
        "AAAAAAAAAA",
        "1st line  ",
        "that we   ",
        "could make",
        " it longer",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Move-2
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveBy((3, 2)));
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 2);
      assert_eq!(actual1.char_idx(), 10);
      assert_eq!(actual1.row_idx(), 1);
      assert_eq!(actual1.column_idx(), 1);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "2nd line ",
        "that we ",
        "must make ",
        "it shorter",
        "!\n",
        "3rd.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        2,
        4,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "2nd line  ",
        "that we   ",
        "must make ",
        "it shorter",
        "!         ",
        "3rd.      ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Insert-3
    {
      stateful.insert_text(
        &data_access,
        Operation::InsertLineWiseTextAtCursor(CompactString::new("World!")),
      );
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 2);
      assert_eq!(actual1.char_idx(), 16);
      assert_eq!(actual1.row_idx(), 1);
      assert_eq!(actual1.column_idx(), 7);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "2nd line ",
        "tWorld!hat",
        " we must ",
        "make it ",
        "shorter!\n",
        "3rd.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        2,
        4,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "2nd line  ",
        "tWorld!hat",
        " we must  ",
        "make it   ",
        "shorter!  ",
        "3rd.      ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Insert-4
    {
      stateful.insert_text(
        &data_access,
        Operation::InsertLineWiseTextAtCursor(CompactString::new("Let's go further!")),
      );
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 2);
      assert_eq!(actual1.char_idx(), 33);
      assert_eq!(actual1.row_idx(), 4);
      assert_eq!(actual1.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "2nd line ",
        "tWorld!",
        "Let's go ",
        "further!",
        "hat we ",
        "must make ",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(2, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        2,
        3,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "2nd line  ",
        "tWorld!   ",
        "Let's go  ",
        "further!  ",
        "hat we    ",
        "must make ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Move-5
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveTo((20, 7)));
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 7);
      assert_eq!(actual2.char_idx(), 20);
      assert_eq!(actual2.row_idx(), 5);
      assert_eq!(actual2.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "4th.\n",
        "5th.\n",
        "6th.\n",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
        "8th.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(4, 0), (5, 0), (6, 0), (7, 0), (8, 0)]
        .into_iter()
        .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        4,
        9,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "4th.      ",
        "5th.      ",
        "6th.      ",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
        "8th.      ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Insert-6
    {
      stateful.insert_text(
        &data_access,
        Operation::InsertLineWiseTextAtCursor(CompactString::new("DDDDDDDDDD")),
      );
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 7);
      assert_eq!(actual1.char_idx(), 30);
      assert_eq!(actual1.row_idx(), 5);
      assert_eq!(actual1.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "5th.\n",
        "6th.\n",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
        "DDDDDDDDDD",
        "8th.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(5, 0), (6, 0), (7, 0), (8, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        5,
        9,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "5th.      ",
        "6th.      ",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
        "DDDDDDDDDD",
        "8th.      ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Move-7
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveLeftBy(17));
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 7);
      assert_eq!(actual2.char_idx(), 13);
      assert_eq!(actual2.row_idx(), 3);
      assert_eq!(actual2.column_idx(), 3);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "5th.\n",
        "6th.\n",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
        "DDDDDDDDDD",
        "8th.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(5, 0), (6, 0), (7, 0), (8, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        5,
        9,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "5th.      ",
        "6th.      ",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
        "DDDDDDDDDD",
        "8th.      ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Insert-8
    {
      stateful.insert_text(
        &data_access,
        Operation::InsertLineWiseTextAtCursor(CompactString::new("abc")),
      );
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 7);
      assert_eq!(actual1.char_idx(), 16);
      assert_eq!(actual1.row_idx(), 3);
      assert_eq!(actual1.column_idx(), 6);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "5th.\n",
        "6th.\n",
        "BBBBBBBBBB",
        "CCCabcCCCC",
        "CCCDDDDDDD",
        "DDD\n",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(5, 0), (6, 0), (7, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        5,
        8,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "5th.      ",
        "6th.      ",
        "BBBBBBBBBB",
        "CCCabcCCCC",
        "CCCDDDDDDD",
        "DDD       ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }
  }
}
// spellchecker:on
