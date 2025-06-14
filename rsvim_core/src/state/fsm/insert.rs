//! The insert mode.

use crate::buf::{Buffer, BufferWk};
use crate::lock;
use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::ops::Operation;
use crate::state::ops::cursor_ops::{self, CursorMoveDirection};
use crate::ui::canvas::CursorStyle;
use crate::ui::tree::*;
use crate::ui::viewport::{
  CursorViewport, Viewport, ViewportArc, ViewportOptions, ViewportSearchDirection,
};

use compact_str::{CompactString, ToCompactString};
use crossterm::event::{Event, KeyCode, KeyEventKind};
use tracing::trace;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The finite-state-machine for insert mode.
pub struct InsertStateful {}

impl InsertStateful {
  fn _get_operation(&self, event: Event) -> Option<Operation> {
    match event {
      Event::FocusGained => None,
      Event::FocusLost => None,
      Event::Key(key_event) => match key_event.kind {
        KeyEventKind::Press => {
          trace!("Event::key:{:?}", key_event);
          match key_event.code {
            KeyCode::Up => Some(Operation::CursorMoveUpBy(1)),
            KeyCode::Down => Some(Operation::CursorMoveDownBy(1)),
            KeyCode::Left => Some(Operation::CursorMoveLeftBy(1)),
            KeyCode::Right => Some(Operation::CursorMoveRightBy(1)),
            KeyCode::Home => Some(Operation::CursorMoveLeftBy(usize::MAX)),
            KeyCode::End => Some(Operation::CursorMoveRightBy(usize::MAX)),
            KeyCode::Char(c) => Some(Operation::InsertAtCursor(c.to_compact_string())),
            KeyCode::Enter => Some(Operation::InsertAtCursor('\n'.to_compact_string())),
            KeyCode::Backspace => Some(Operation::DeleteAtCursor(-1)),
            KeyCode::Delete => Some(Operation::DeleteAtCursor(1)),
            KeyCode::Esc => Some(Operation::GotoNormalMode),
            _ => None,
          }
        }
        KeyEventKind::Repeat => None,
        KeyEventKind::Release => None,
      },
      Event::Mouse(_mouse_event) => None,
      Event::Paste(ref _paste_string) => None,
      Event::Resize(_columns, _rows) => None,
    }
  }
}

impl Stateful for InsertStateful {
  fn handle(&self, data_access: StatefulDataAccess) -> StatefulValue {
    let event = data_access.event.clone();

    if let Some(op) = self._get_operation(event) {
      return self.handle_op(data_access, op);
    }

    StatefulValue::InsertMode(InsertStateful::default())
  }

  fn handle_op(&self, data_access: StatefulDataAccess, op: Operation) -> StatefulValue {
    match op {
      Operation::GotoNormalMode => self.goto_normal_mode(&data_access),
      Operation::CursorMoveBy((_, _))
      | Operation::CursorMoveUpBy(_)
      | Operation::CursorMoveDownBy(_)
      | Operation::CursorMoveLeftBy(_)
      | Operation::CursorMoveRightBy(_)
      | Operation::CursorMoveTo((_, _)) => self.cursor_move(&data_access, op),
      Operation::InsertAtCursor(text) => self.insert_at_cursor(&data_access, text),
      Operation::DeleteAtCursor(n) => self.delete_at_cursor(&data_access, n),
      _ => unreachable!(),
    }
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
  fn delete_at_cursor(&self, data_access: &StatefulDataAccess, n: isize) -> StatefulValue {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);
    let buffer = self._current_buffer(&mut tree);
    let buffer = buffer.upgrade().unwrap();
    let mut buffer = lock!(buffer);

    // Delete N-chars.
    let (cursor_line_idx_after_deleted, cursor_char_idx_after_deleted) = {
      if let Some(current_window_id) = tree.current_window_id() {
        if let Some(TreeNode::Window(current_window)) = tree.node_mut(current_window_id) {
          let cursor_viewport = current_window.cursor_viewport();

          let maybe_new_cursor_position =
            cursor_ops::delete_at_cursor(&cursor_viewport, buffer.text_mut(), n);

          if maybe_new_cursor_position.is_none() {
            return StatefulValue::InsertMode(InsertStateful::default());
          }

          maybe_new_cursor_position.unwrap()
        } else {
          unreachable!()
        }
      } else {
        unreachable!()
      }
    };

    // Update viewport since the buffer has changed, and viewport doesn't match it any more.
    self._update_viewport_after_buffer_changed(&mut tree, &buffer);

    trace!(
      "Move to inserted pos, line:{cursor_line_idx_after_deleted}, char:{cursor_char_idx_after_deleted}"
    );
    self._cursor_move_impl(
      CursorMoveImplOptions::include_empty_eol(),
      &mut tree,
      &buffer,
      Operation::CursorMoveTo((cursor_char_idx_after_deleted, cursor_line_idx_after_deleted)),
    );

    StatefulValue::InsertMode(InsertStateful::default())
  }
}

impl InsertStateful {
  fn insert_at_cursor(
    &self,
    data_access: &StatefulDataAccess,
    text: CompactString,
  ) -> StatefulValue {
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
          cursor_ops::insert_at_cursor(&cursor_viewport, buffer.text_mut(), text)
        } else {
          unreachable!()
        }
      } else {
        unreachable!()
      }
    };

    // Update viewport since the buffer doesn't match the viewport.
    self._update_viewport_after_buffer_changed(&mut tree, &buffer);

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

  // Update viewport since the buffer has changed, and the viewport doesn't match the buffer.
  fn _update_viewport_after_buffer_changed(&self, tree: &mut Tree, buffer: &Buffer) {
    if let Some(current_window_id) = tree.current_window_id() {
      if let Some(TreeNode::Window(current_window)) = tree.node_mut(current_window_id) {
        let text = buffer.text();
        let viewport = current_window.viewport();
        let cursor_viewport = current_window.cursor_viewport();
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

        let viewport_opts = ViewportOptions::from(current_window.options());
        let updated_viewport = Viewport::to_arc(Viewport::view(
          &viewport_opts,
          text,
          current_window.actual_shape(),
          start_line,
          start_column,
        ));
        trace!("after updated_viewport:{:?}", updated_viewport);

        current_window.set_viewport(updated_viewport.clone());
        if let Some(updated_cursor_viewport) = cursor_ops::cursor_move_to(
          &updated_viewport,
          &cursor_viewport,
          text,
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
  fn goto_normal_mode(&self, data_access: &StatefulDataAccess) -> StatefulValue {
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
          let viewport_opts = ViewportOptions::from(current_window.options());
          let (start_line, start_column) = viewport.search_anchor(
            search_direction,
            &viewport_opts,
            buffer.text(),
            current_window.actual_shape(),
            target_cursor_line,
            target_cursor_char,
          );

          // First try window scroll.
          if start_line != viewport.start_line_idx() || start_column != viewport.start_column_idx()
          {
            let new_viewport = cursor_ops::window_scroll_to(
              &viewport,
              current_window,
              buffer.text(),
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
            buffer.text(),
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
  ) -> (usize, usize, ViewportSearchDirection) {
    let (target_cursor_char, target_cursor_line, move_direction) = if opts.include_empty_eol {
      cursor_ops::normalize_to_cursor_move_to_include_empty_eol(
        buffer.text(),
        op,
        cursor_viewport.char_idx(),
        cursor_viewport.line_idx(),
      )
    } else {
      cursor_ops::normalize_to_cursor_move_to_exclude_empty_eol(
        buffer.text(),
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
  use crate::ui::viewport::{
    CursorViewport, CursorViewportArc, Viewport, ViewportArc, ViewportSearchDirection,
  };
  use crate::ui::widget::Widgetable;
  use crate::ui::widget::window::content::WindowContent;
  use crate::ui::widget::window::{WindowLocalOptions, WindowLocalOptionsBuilder};

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
      .text()
      .rope()
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
mod tests_get_operation {
  use super::tests_util::*;
  use super::*;

  use crate::buf::{BufferArc, BufferLocalOptionsBuilder, BuffersManagerArc};
  use crate::prelude::*;
  use crate::state::{State, StateArc};
  use crate::test::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::test::log::init as test_log_init;
  use crate::test::tree::make_tree_with_buffers;
  use crate::ui::tree::TreeArc;
  use crate::ui::viewport::{
    CursorViewport, CursorViewportArc, Viewport, ViewportArc, ViewportSearchDirection,
  };
  use crate::ui::widget::window::{WindowLocalOptions, WindowLocalOptionsBuilder};
  use crate::{lock, state};

  use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
  use std::collections::BTreeMap;
  use tracing::info;

  #[test]
  fn get1() {
    test_log_init();

    let stateful = InsertStateful::default();
    assert!(matches!(
      stateful._get_operation(Event::Key(KeyEvent::new(
        KeyCode::Char('c'),
        KeyModifiers::empty()
      ))),
      Some(Operation::InsertAtCursor(_))
    ));
    assert!(matches!(
      stateful._get_operation(Event::Key(KeyEvent::new(
        KeyCode::Up,
        KeyModifiers::empty()
      ))),
      Some(Operation::CursorMoveUpBy(_))
    ));
    assert!(matches!(
      stateful._get_operation(Event::Key(KeyEvent::new(
        KeyCode::Esc,
        KeyModifiers::empty()
      ))),
      Some(Operation::GotoNormalMode)
    ));
    assert!(matches!(
      stateful._get_operation(Event::Key(KeyEvent::new(
        KeyCode::Backspace,
        KeyModifiers::empty()
      ))),
      Some(Operation::DeleteAtCursor(_))
    ));
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
  use crate::ui::viewport::{
    CursorViewport, CursorViewportArc, Viewport, ViewportArc, ViewportSearchDirection,
  };
  use crate::ui::widget::window::{WindowLocalOptions, WindowLocalOptionsBuilder};

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
  use crate::ui::viewport::{
    CursorViewport, CursorViewportArc, Viewport, ViewportArc, ViewportSearchDirection,
  };
  use crate::ui::widget::window::{WindowLocalOptions, WindowLocalOptionsBuilder};

  use compact_str::CompactString;
  use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
  use jiff::fmt::friendly::Designator::Compact;
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
      stateful.insert_at_cursor(&data_access, CompactString::new("Bye, "));

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
      stateful.insert_at_cursor(&data_access, CompactString::new(" Go!"));

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
      stateful.cursor_move(&data_access, Operation::CursorMoveBy((100, 6)));

      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 5);
      assert_eq!(actual1.char_idx(), 30);
      assert_eq!(actual1.row_idx(), 5);
      assert_eq!(actual1.column_idx(), 9);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "",
        "e and smal",
        " several t",
        " small eno",
        " too long ",
        "er cases?\n",
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
        "          ",
        "e and smal",
        " several t",
        " small eno",
        " too long ",
        "er cases? ",
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
      stateful.insert_at_cursor(&data_access, CompactString::new("a"));

      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 5);
      assert_eq!(actual2.char_idx(), 31);
      assert_eq!(actual2.row_idx(), 5);
      assert_eq!(actual2.column_idx(), 9);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "",
        " and small",
        "several th",
        "small enou",
        "too long t",
        "r cases?a\n",
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
        "          ",
        " and small",
        "several th",
        "small enou",
        "too long t",
        "r cases?a ",
        "          ",
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
      stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(3));

      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 3);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 3);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
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
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_option, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Insert-2
    {
      let buf_eol = lock!(buf).options().end_of_line();
      let text2 = CompactString::new(format!(
        "Let's{}insert{}multiple lines!{}",
        buf_eol, buf_eol, buf_eol
      ));
      stateful.insert_at_cursor(&data_access, text2);

      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 3);
      assert_eq!(actual2.char_idx(), 0);
      assert_eq!(actual2.row_idx(), 3);
      assert_eq!(actual2.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let l0 = format!("HelLet's{}", buf_eol);
      let l1 = format!("insert{}", buf_eol);
      let expect = vec![
        l0.as_str(),
        l1.as_str(),
        "multiple l",
        "lo, RSVIM!",
        "This is a ",
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
        "HelLet's  ",
        "insert    ",
        "multiple l",
        "lo, RSVIM!",
        "This is a ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_option, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Insert-3
    {
      let buf_eol = lock!(buf).options().end_of_line();
      let text2 = CompactString::new(format!(
        "Insert two lines again!{}There's no line-break",
        buf_eol
      ));
      stateful.insert_at_cursor(&data_access, text2);

      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 4);
      assert_eq!(actual2.char_idx(), 21);
      assert_eq!(actual2.row_idx(), 4);
      assert_eq!(actual2.column_idx(), 9);

      let viewport = get_viewport(tree.clone());
      let l2 = format!("es!{}", buf_eol);
      let expect = vec!["", "", l2.as_str(), "ines again", "ine-breakl"];
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
        "          ",
        "          ",
        "es!       ",
        "ines again",
        "ine-breakl",
      ];
      let actual_canvas = make_canvas(terminal_size, window_option, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Move-4
    {
      // let buf_eol = lock!(buf).options().end_of_line();
      stateful.cursor_move(&data_access, Operation::CursorMoveBy((100, 6)));

      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 9);
      assert_eq!(actual1.char_idx(), 30);
      assert_eq!(actual1.row_idx(), 4);
      assert_eq!(actual1.column_idx(), 9);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "e and smal",
        " several t",
        " small eno",
        " too long ",
        "er cases?\n",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(5, 0), (6, 0), (7, 0), (8, 0), (9, 0)]
        .into_iter()
        .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        5,
        10,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "e and smal",
        " several t",
        " small eno",
        " too long ",
        "er cases? ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_option, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Insert-5
    {
      let buf_eol = lock!(buf).options().end_of_line();
      let text5 = CompactString::new(format!(
        "Final 3 lines.{}The inserted 2nd{}The inserted 3rd{}",
        buf_eol, buf_eol, buf_eol
      ));
      stateful.insert_at_cursor(&data_access, text5);

      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 12);
      assert_eq!(actual1.char_idx(), 0);
      assert_eq!(actual1.row_idx(), 4);
      assert_eq!(actual1.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec!["  2. When ", "  3. Is th", "The insert", "The insert", "\n"];
      let expect_fills: BTreeMap<usize, usize> = vec![(8, 0), (9, 0), (10, 0), (11, 0), (12, 0)]
        .into_iter()
        .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        8,
        13,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "  2. When ",
        "  3. Is th",
        "The insert",
        "The insert",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_option, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Insert-6
    {
      stateful.insert_at_cursor(&data_access, "Insert 4th".to_compact_string());

      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 12);
      assert_eq!(actual1.char_idx(), 10);
      assert_eq!(actual1.row_idx(), 4);
      assert_eq!(actual1.column_idx(), 9);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        " 2. When t",
        " 3. Is the",
        "he inserte",
        "he inserte",
        "nsert 4th\n",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(8, 0), (9, 0), (10, 0), (11, 0), (12, 0)]
        .into_iter()
        .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        8,
        13,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        " 2. When t",
        " 3. Is the",
        "he inserte",
        "he inserte",
        "nsert 4th ",
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
      stateful.insert_at_cursor(&data_access, CompactString::new("Hello, "));
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
      stateful.insert_at_cursor(&data_access, CompactString::new("World!"));
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
      stateful.insert_at_cursor(&data_access, CompactString::new("Go!"));
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
      stateful.insert_at_cursor(&data_access, CompactString::new("DDDDDDDDDD"));
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
      stateful.insert_at_cursor(&data_access, CompactString::new("abc"));
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
      stateful.insert_at_cursor(&data_access, CompactString::new("a"));
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 1);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 1);

      let viewport = get_viewport(tree.clone());
      let a = format!("a{}", lock!(buf.clone()).options().end_of_line());
      let expect = vec![a.as_str(), ""];
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
      stateful.insert_at_cursor(&data_access, CompactString::new("b"));
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 1);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 1);

      let viewport = get_viewport(tree.clone());
      let b = format!("b{}", lock!(buf.clone()).options().end_of_line());
      let expect = vec![b.as_str(), ""];
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
  fn wrap_nolinebreak4() {
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
      stateful.insert_at_cursor(&data_access, CompactString::new("这个"));
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 2);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 4);

      let viewport = get_viewport(tree.clone());
      let b = format!("这个{}", lock!(buf.clone()).options().end_of_line());
      let expect = vec![b.as_str(), ""];
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
        "这个      ",
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
      stateful.insert_at_cursor(&data_access, CompactString::new("Hello, "));
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
      stateful.insert_at_cursor(&data_access, CompactString::new("World!"));
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
      stateful.insert_at_cursor(&data_access, CompactString::new("Let's go further!"));
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
      stateful.insert_at_cursor(&data_access, CompactString::new("DDDDDDDDDD"));
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
      stateful.insert_at_cursor(&data_access, CompactString::new("abc"));
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

#[cfg(test)]
#[allow(unused_imports)]
mod tests_delete_text {
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
  use crate::ui::viewport::{
    CursorViewport, CursorViewportArc, Viewport, ViewportArc, ViewportSearchDirection,
  };
  use crate::ui::widget::window::{WindowLocalOptions, WindowLocalOptionsBuilder};

  use compact_str::CompactString;
  use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
  use jiff::fmt::friendly::Designator::Compact;
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
      "* The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "* The extra.\n",
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

    // Delete-1
    {
      stateful.delete_at_cursor(&data_access, -1);

      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 0);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "* The extr",
        "* The extr",
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
        "* The extr",
        "* The extr",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Move-2
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(7));

      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 7);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 7);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "* The extr",
        "* The extr",
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
        "* The extr",
        "* The extr",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Delete-3
    {
      stateful.delete_at_cursor(&data_access, -5);

      let tree = data_access.tree.clone();
      let actual3 = get_cursor_viewport(tree.clone());
      assert_eq!(actual3.line_idx(), 0);
      assert_eq!(actual3.char_idx(), 2);
      assert_eq!(actual3.row_idx(), 0);
      assert_eq!(actual3.column_idx(), 2);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "HeRSVIM!\n",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "* The extr",
        "* The extr",
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
        "HeRSVIM!  ",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "* The extr",
        "* The extr",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Move-4
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(3));

      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 3);
      assert_eq!(actual1.char_idx(), 2);
      assert_eq!(actual1.row_idx(), 3);
      assert_eq!(actual1.column_idx(), 2);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "HeRSVIM!\n",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "* The extr",
        "* The extr",
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
        "HeRSVIM!  ",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "* The extr",
        "* The extr",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Delete-5
    {
      stateful.delete_at_cursor(&data_access, -50);

      let tree = data_access.tree.clone();
      let actual3 = get_cursor_viewport(tree.clone());
      assert_eq!(actual3.line_idx(), 2);
      assert_eq!(actual3.char_idx(), 6);
      assert_eq!(actual3.row_idx(), 2);
      assert_eq!(actual3.column_idx(), 6);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "HeRSVIM!\n",
        "This is a ",
        "But st1. W",
        "  2. When ",
        "* The extr",
        "* The extr",
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
        "HeRSVIM!  ",
        "This is a ",
        "But st1. W",
        "  2. When ",
        "* The extr",
        "* The extr",
        "          ",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Delete-6
    {
      stateful.delete_at_cursor(&data_access, 60);

      let tree = data_access.tree.clone();
      let actual3 = get_cursor_viewport(tree.clone());
      assert_eq!(actual3.line_idx(), 2);
      assert_eq!(actual3.char_idx(), 6);
      assert_eq!(actual3.row_idx(), 2);
      assert_eq!(actual3.column_idx(), 6);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "HeRSVIM!\n",
        "This is a ",
        "But strow ",
        "  2. When ",
        "* The extr",
        "* The extr",
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
        "HeRSVIM!  ",
        "This is a ",
        "But strow ",
        "  2. When ",
        "* The extr",
        "* The extr",
        "          ",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Delete-7
    {
      stateful.delete_at_cursor(&data_access, -1);

      let tree = data_access.tree.clone();
      let actual3 = get_cursor_viewport(tree.clone());
      assert_eq!(actual3.line_idx(), 2);
      assert_eq!(actual3.char_idx(), 5);
      assert_eq!(actual3.row_idx(), 2);
      assert_eq!(actual3.column_idx(), 5);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "HeRSVIM!\n",
        "This is a ",
        "But srow o",
        "  2. When ",
        "* The extr",
        "* The extr",
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
        "HeRSVIM!  ",
        "This is a ",
        "But srow o",
        "  2. When ",
        "* The extr",
        "* The extr",
        "          ",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Delete-8
    {
      stateful.delete_at_cursor(&data_access, 1);

      let tree = data_access.tree.clone();
      let actual3 = get_cursor_viewport(tree.clone());
      assert_eq!(actual3.line_idx(), 2);
      assert_eq!(actual3.char_idx(), 5);
      assert_eq!(actual3.row_idx(), 2);
      assert_eq!(actual3.column_idx(), 5);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "HeRSVIM!\n",
        "This is a ",
        "But sow of",
        "  2. When ",
        "* The extr",
        "* The extr",
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
        "HeRSVIM!  ",
        "This is a ",
        "But sow of",
        "  2. When ",
        "* The extr",
        "* The extr",
        "          ",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Move-9
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveBy((500, 10)));

      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 5);
      assert_eq!(actual1.char_idx(), 12);
      assert_eq!(actual1.row_idx(), 5);
      assert_eq!(actual1.column_idx(), 9);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "SVIM!\n",
        "s is a qui",
        " sow of th",
        ". When the",
        "he extra p",
        "he extra.\n",
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
        "SVIM!     ",
        "s is a qui",
        " sow of th",
        ". When the",
        "he extra p",
        "he extra. ",
        "          ",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Delete-10
    {
      stateful.delete_at_cursor(&data_access, 1);

      let tree = data_access.tree.clone();
      let actual3 = get_cursor_viewport(tree.clone());
      assert_eq!(actual3.line_idx(), 5);
      assert_eq!(actual3.char_idx(), 12);
      assert_eq!(actual3.row_idx(), 5);
      assert_eq!(actual3.column_idx(), 9);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "SVIM!\n",
        "s is a qui",
        " sow of th",
        ". When the",
        "he extra p",
        "he extra.\n",
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
        "SVIM!     ",
        "s is a qui",
        " sow of th",
        ". When the",
        "he extra p",
        "he extra. ",
        "          ",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Delete-11
    {
      stateful.delete_at_cursor(&data_access, -1);

      let tree = data_access.tree.clone();
      let actual3 = get_cursor_viewport(tree.clone());
      assert_eq!(actual3.line_idx(), 5);
      assert_eq!(actual3.char_idx(), 11);
      assert_eq!(actual3.row_idx(), 5);
      assert_eq!(actual3.column_idx(), 8);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "SVIM!\n",
        "s is a qui",
        " sow of th",
        ". When the",
        "he extra p",
        "he extra\n",
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
        "SVIM!     ",
        "s is a qui",
        " sow of th",
        ". When the",
        "he extra p",
        "he extra  ",
        "          ",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }
  }
}
// spellchecker:on
