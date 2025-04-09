//! The normal mode.

use crate::buf::Buffer;
use crate::state::command::Command;
use crate::state::fsm::quit::QuitStateful;
use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::ui::tree::*;
use crate::ui::widget::window::Viewport;
use crate::wlock;

use crossterm::event::{Event, KeyCode, KeyEventKind};
use std::ptr::NonNull;
use tracing::trace;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The normal editing mode.
pub struct NormalStateful {}

unsafe fn last_char_on_line(raw_buffer: NonNull<Buffer>, line_idx: usize) -> usize {
  unsafe { raw_buffer.as_ref().get_rope().line(line_idx).len_chars() - 1 }
}

unsafe fn last_visible_char_on_line_since(
  raw_buffer: NonNull<Buffer>,
  line_idx: usize,
  char_idx: usize,
) -> usize {
  unsafe {
    let bline = raw_buffer.as_ref().get_rope().get_line(line_idx).unwrap();
    let mut c = char_idx;
    while raw_buffer.as_ref().char_width(bline.get_char(c).unwrap()) == 0 {
      c = c.saturating_sub(1);
      if c == 0 {
        break;
      }
    }
    c
  }
}

unsafe fn last_visible_char_on_line(raw_buffer: NonNull<Buffer>, line_idx: usize) -> usize {
  unsafe {
    let c = last_char_on_line(raw_buffer, line_idx);
    last_visible_char_on_line_since(raw_buffer, line_idx, c)
  }
}

unsafe fn adjust_cursor_char_idx_on_vertical_motion(
  mut raw_buffer: NonNull<Buffer>,
  cursor_line_idx: usize,
  cursor_char_idx: usize,
  line_idx: usize,
) -> usize {
  unsafe {
    let cursor_col_idx = raw_buffer
      .as_mut()
      .width_before(cursor_line_idx, cursor_char_idx);
    let char_idx = match raw_buffer.as_mut().char_after(line_idx, cursor_col_idx) {
      Some(char_idx) => char_idx,
      None => last_visible_char_on_line(raw_buffer, line_idx),
    };
    trace!(
      "cursor_line_idx:{},cursor_col_idx:{},line_idx:{},char_idx:{}",
      cursor_line_idx, cursor_col_idx, line_idx, char_idx
    );
    char_idx
  }
}

impl Stateful for NormalStateful {
  fn handle(&self, data_access: StatefulDataAccess) -> StatefulValue {
    let event = data_access.event.clone();

    match event {
      Event::FocusGained => {}
      Event::FocusLost => {}
      Event::Key(key_event) => match key_event.kind {
        KeyEventKind::Press => {
          match key_event.code {
            KeyCode::Up | KeyCode::Char('k') => {
              return self.cursor_move(&data_access, Command::CursorMoveUp(1));
            }
            KeyCode::Down | KeyCode::Char('j') => {
              return self.cursor_move(&data_access, Command::CursorMoveDown(1));
            }
            KeyCode::Left | KeyCode::Char('h') => {
              return self.cursor_move(&data_access, Command::CursorMoveLeft(1));
            }
            KeyCode::Right | KeyCode::Char('l') => {
              return self.cursor_move(&data_access, Command::CursorMoveRight(1));
            }
            KeyCode::Esc => {
              // quit loop
              return self.quit(&data_access, Command::QuitEditor);
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

    // if event == Event::Key(KeyCode::Char('c').into()) {
    //   println!("Cursor position: {:?}\r", crossterm::cursor::position());
    // }

    // // quit loop
    // if event == Event::Key(KeyCode::Esc.into()) {
    //   // println!("ESC: {:?}\r", crossterm::cursor::position());
    //   return StateMachine::QuitState(QuitStateful::default());
    // }

    StatefulValue::NormalMode(NormalStateful::default())
  }
}

impl NormalStateful {
  /// Cursor move up/down/left/right in current window, or scroll buffer up/down if it reaches the
  /// top/bottom of the window and the buffer has more contents.
  fn _cursor_move_or_scroll(
    &self,
    _data_access: &StatefulDataAccess,
    _command: Command,
  ) -> StatefulValue {
    StatefulValue::NormalMode(NormalStateful::default())
  }

  /// Cursor move up/down/left/right in current window.
  /// NOTE: This will not scroll the buffer if cursor reaches the top/bottom of the window.
  ///
  /// Also see [`NormalStateful::cursor_move_with_scroll`].
  fn cursor_move(&self, data_access: &StatefulDataAccess, command: Command) -> StatefulValue {
    let tree = data_access.tree.clone();
    let mut tree = wlock!(tree);

    if let Some(current_window_id) = tree.current_window_id() {
      if let Some(TreeNode::Window(current_window)) = tree.node_mut(&current_window_id) {
        let viewport = current_window.viewport();
        let mut viewport = wlock!(viewport);
        let buffer = viewport.buffer();
        let buffer = buffer.upgrade().unwrap();
        let mut buffer = wlock!(buffer);
        unsafe {
          // Fix multiple mutable references on `buffer`.
          let mut raw_buffer: NonNull<Buffer> = NonNull::new(&mut *buffer as *mut Buffer).unwrap();

          let cursor_move_result = match command {
            Command::CursorMoveUp(_) | Command::CursorMoveDown(_) => {
              self._cursor_move_vertically(&viewport, raw_buffer, command)
            }
            Command::CursorMoveLeft(_) | Command::CursorMoveRight(_) => {
              self._cursor_move_horizontally(&viewport, raw_buffer, command)
            }
            _ => unreachable!(),
          };

          trace!("cursor_move_result:{:?}", cursor_move_result);
          if let Some((line_idx, char_idx)) = cursor_move_result {
            viewport.set_cursor(line_idx, char_idx);
            let cursor_row = viewport
              .lines()
              .get(&line_idx)
              .unwrap()
              .rows()
              .iter()
              .filter(|(_row_idx, row_viewport)| {
                trace!(
                  "row_viewport:{:?}, start_char_idx:{:?},end_char_idx:{:?},char_idx:{:?}",
                  row_viewport,
                  row_viewport.start_char_idx(),
                  row_viewport.end_char_idx(),
                  char_idx
                );
                row_viewport.start_char_idx() <= char_idx && row_viewport.end_char_idx() > char_idx
              })
              .collect::<Vec<_>>();
            trace!(
              "char_idx:{:?}, cursor_row({:?}):{:?}",
              char_idx,
              cursor_row.len(),
              cursor_row
            );
            assert_eq!(cursor_row.len(), 1);

            let (row_idx, row_viewport) = cursor_row[0];
            let cursor_id = tree.cursor_id().unwrap();

            let row_start_width = raw_buffer
              .as_mut()
              .width_before(line_idx, row_viewport.start_char_idx());
            let char_start_width = raw_buffer.as_mut().width_before(line_idx, char_idx);
            let col_idx = (char_start_width - row_start_width) as isize;
            let row_idx = *row_idx as isize;
            tree.bounded_move_to(cursor_id, col_idx, row_idx);
            trace!(
              "(after) cursor node position x/y:{:?}/{:?}",
              col_idx, row_idx
            );
          }
          // Or, just do nothing, stay at where you are
        }
      }
    }
    StatefulValue::NormalMode(NormalStateful::default())
  }

  /// Returns the `line_idx` and `char_idx` for the cursor.
  unsafe fn _cursor_move_vertically(
    &self,
    viewport: &Viewport,
    raw_buffer: NonNull<Buffer>,
    command: Command,
  ) -> Option<(usize, usize)> {
    trace!("command:{:?}", command);
    let cursor_viewport = viewport.cursor();
    let cursor_line_idx = cursor_viewport.line_idx();
    let cursor_char_idx = cursor_viewport.char_idx();

    let line_idx = match command {
      Command::CursorMoveUp(n) => cursor_line_idx.saturating_sub(n),
      Command::CursorMoveDown(n) => {
        let expected = cursor_line_idx.saturating_add(n);
        let end_line_idx = viewport.end_line_idx();
        let last_line_idx = end_line_idx.saturating_sub(1);
        trace!(
          "cursor_line_idx:{:?},expected:{:?},end_line_idx:{:?},last_line_idx:{:?}",
          cursor_line_idx, expected, end_line_idx, last_line_idx
        );
        std::cmp::min(expected, last_line_idx)
      }
      _ => unreachable!(),
    };
    trace!(
      "cursor:{}/{},line_idx:{}",
      cursor_line_idx, cursor_char_idx, line_idx
    );

    // If line index doesn't change, early return.
    if line_idx == cursor_line_idx {
      return None;
    }

    unsafe {
      match raw_buffer.as_ref().get_rope().get_line(line_idx) {
        Some(line) => {
          trace!("line.len_chars:{}", line.len_chars());
          if line.len_chars() == 0 {
            return None;
          }
        }
        None => {
          trace!("get_line not found:{}", line_idx);
          return None;
        }
      }
      let char_idx = adjust_cursor_char_idx_on_vertical_motion(
        raw_buffer,
        cursor_line_idx,
        cursor_char_idx,
        line_idx,
      );
      Some((line_idx, char_idx))
    }
  }

  /// Returns the `line_idx` and `char_idx` for the cursor.
  unsafe fn _cursor_move_horizontally(
    &self,
    viewport: &Viewport,
    raw_buffer: NonNull<Buffer>,
    command: Command,
  ) -> Option<(usize, usize)> {
    let cursor_viewport = viewport.cursor();
    let cursor_line_idx = cursor_viewport.line_idx();
    let cursor_char_idx = cursor_viewport.char_idx();

    unsafe {
      match raw_buffer.as_ref().get_rope().get_line(cursor_line_idx) {
        Some(line) => {
          if line.len_chars() == 0 {
            return None;
          }
        }
        None => return None,
      }

      let char_idx = match command {
        Command::CursorMoveLeft(n) => cursor_char_idx.saturating_sub(n),
        Command::CursorMoveRight(n) => {
          let expected = cursor_char_idx.saturating_add(n);
          let last_char_idx = {
            debug_assert!(viewport.lines().contains_key(&cursor_line_idx));
            let line_viewport = viewport.lines().get(&cursor_line_idx).unwrap();
            let (_last_row_idx, last_row_viewport) = line_viewport.rows().last_key_value().unwrap();
            let last_char_on_row = last_row_viewport.end_char_idx() - 1;
            trace!(
              "cursor_char_idx:{}, expected:{}, last_row_viewport:{:?}, last_char_on_row:{}",
              cursor_char_idx, expected, last_row_viewport, last_char_on_row
            );
            last_visible_char_on_line_since(raw_buffer, cursor_line_idx, last_char_on_row)
          };
          std::cmp::min(expected, last_char_idx)
        }
        _ => unreachable!(),
      };

      Some((cursor_line_idx, char_idx))
    }
  }

  /// Cursor scroll buffer up/down in current window.
  ///
  /// NOTE: The cursor actually stays still in the window, its "position" is not changed. The
  /// buffer contents changed, i.e. moved up/down.
  fn cursor_scroll(&self, data_access: &StatefulDataAccess, command: Command) -> StatefulValue {
    let tree = data_access.tree.clone();
    let mut tree = wlock!(tree);

    if let Some(current_window_id) = tree.current_window_id() {
      if let Some(TreeNode::Window(current_window)) = tree.node_mut(&current_window_id) {
        let viewport = current_window.viewport();
        let mut viewport = wlock!(viewport);
        let buffer = viewport.buffer();
        let buffer = buffer.upgrade().unwrap();
        let mut buffer = wlock!(buffer);

        unsafe {
          // Fix multiple mutable references on `buffer`.
          let raw_buffer: NonNull<Buffer> = NonNull::new(&mut *buffer as *mut Buffer).unwrap();

          let cursor_scroll_result = match command {
            Command::CursorMoveUp(_n) | Command::CursorMoveDown(_n) => {
              self._cursor_scroll_vertically(&viewport, raw_buffer, command)
            }
            Command::CursorMoveLeft(_n) | Command::CursorMoveRight(_n) => {
              self._cursor_scroll_horizontally(&viewport, raw_buffer, command)
            }
            _ => unreachable!(),
          };

          if let Some((start_line_idx, start_column_idx)) = cursor_scroll_result {
            // Sync the viewport
            viewport.sync_from_top_left(start_line_idx, start_column_idx);
          }
          // Or, just do nothing, keep the old viewport.
        }
      }
    }

    StatefulValue::NormalMode(NormalStateful::default())
  }

  /// Returns the `start_line_idx`/`start_column_idx` for viewport.
  unsafe fn _cursor_scroll_vertically(
    &self,
    viewport: &Viewport,
    mut _raw_buffer: NonNull<Buffer>,
    command: Command,
  ) -> Option<(usize, usize)> {
    let start_line_idx = viewport.start_line_idx();
    let start_column_idx = viewport.start_column_idx();

    let line_idx = match command {
      Command::CursorMoveUp(n) => start_line_idx.saturating_sub(n),
      Command::CursorMoveDown(n) => {
        let expected = start_line_idx.saturating_add(n);
        let end_line_idx = viewport.end_line_idx();
        let last_line_idx = end_line_idx.saturating_sub(1);
        trace!(
          "start_line_idx:{:?},expected:{:?},end_line_idx:{:?},last_line_idx:{:?}",
          start_line_idx, expected, end_line_idx, last_line_idx
        );
        std::cmp::min(expected, last_line_idx)
      }
      _ => unreachable!(),
    };

    if line_idx == start_line_idx {
      return None;
    }

    Some((line_idx, start_column_idx))
  }

  /// Returns the same as [`Self::_cursor_scroll_vertically`].
  unsafe fn _cursor_scroll_horizontally(
    &self,
    viewport: &Viewport,
    mut raw_buffer: NonNull<Buffer>,
    command: Command,
  ) -> Option<(usize, usize)> {
    let start_line_idx = viewport.start_line_idx();
    let start_column_idx = viewport.start_column_idx();

    unsafe {
      let (start_col) = match raw_buffer
        .as_mut()
        .char_at(start_line_idx, start_column_idx)
      {
        Some(start_char_idx) => match command {
          Command::CursorMoveLeft(n) => {
            let c = start_char_idx.saturating_sub(n);
            raw_buffer.as_mut().width_before(start_line_idx, c)
          }
          Command::CursorMoveRight(n) => {
            debug_assert!(viewport.lines().contains_key(&start_line_idx));
            let line_viewport = viewport.lines().get(&start_line_idx).unwrap();
            let (_last_row_idx, last_row_viewport) = line_viewport.rows().last_key_value().unwrap();
            let end_char_idx = last_row_viewport.end_char_idx();
            let expected = end_char_idx.saturating_add(n);
            let bline = raw_buffer
              .as_ref()
              .get_rope()
              .get_line(start_line_idx)
              .unwrap();
            let end_c = std::cmp::min(bline.len_chars(), expected);
            let scrolled_right_columns = raw_buffer
              .as_mut()
              .width_before(start_line_idx, end_c)
              .saturating_sub(
                raw_buffer
                  .as_mut()
                  .width_before(start_line_idx, end_char_idx),
              );
            start_column_idx.saturating_add(scrolled_right_columns)
          }
          _ => unreachable!(),
        },
        None => 0_usize,
      };

      if start_col == start_column_idx {
        return None;
      }

      Some((start_line_idx, start_col))
    }
  }

  fn quit(&self, _data_access: &StatefulDataAccess, _command: Command) -> StatefulValue {
    StatefulValue::QuitState(QuitStateful::default())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use crate::buf::{BufferLocalOptionsBuilder, BuffersManagerArc};
  use crate::prelude::*;
  use crate::rlock;
  use crate::state::{State, StateArc};
  use crate::test::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::test::log::init as test_log_init;
  use crate::test::tree::make_tree_with_buffers;
  use crate::ui::tree::TreeArc;
  use crate::ui::widget::window::{Viewport, WindowLocalOptions, WindowLocalOptionsBuilder};

  use crossterm::event::Event;
  use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

  fn make_tree(
    terminal_size: U16Size,
    window_local_opts: WindowLocalOptions,
    lines: Vec<&str>,
  ) -> (TreeArc, StateArc, BuffersManagerArc) {
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let buf = make_buffer_from_lines(terminal_size.height(), buf_opts, lines);
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let tree = make_tree_with_buffers(terminal_size, window_local_opts, bufs.clone());
    let state = State::to_arc(State::default());
    (tree, state, bufs)
  }

  fn get_viewport(tree: TreeArc) -> Viewport {
    let tree = rlock!(tree);
    let current_window_id = tree.current_window_id().unwrap();
    let current_window_node = tree.node(&current_window_id).unwrap();
    assert!(matches!(current_window_node, TreeNode::Window(_)));
    match current_window_node {
      TreeNode::Window(current_window) => {
        let viewport = current_window.viewport();
        let viewport = rlock!(viewport);
        viewport.clone()
      }
      _ => unreachable!(),
    }
  }

  #[test]
  fn cursor_move_vertically_nowrap1() {
    test_log_init();

    let (tree, state, bufs) = make_tree(
      U16Size::new(10, 10),
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      vec![],
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_viewport = get_viewport(tree.clone());
    assert_eq!(prev_viewport.cursor().line_idx(), 0);
    assert_eq!(prev_viewport.cursor().char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful_machine = NormalStateful::default();
    let next_stateful = stateful_machine.cursor_move(&data_access, Command::CursorMoveUp(1));
    assert!(matches!(next_stateful, StatefulValue::NormalMode(_)));

    let tree = data_access.tree.clone();
    let actual_viewport = get_viewport(tree);
    assert_eq!(actual_viewport.cursor().line_idx(), 0);
    assert_eq!(actual_viewport.cursor().char_idx(), 0);
  }

  #[test]
  fn cursor_move_vertically_nowrap2() {
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
    let (tree, state, bufs) = make_tree(
      U16Size::new(10, 10),
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      lines,
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_viewport = get_viewport(tree.clone());
    assert_eq!(prev_viewport.cursor().line_idx(), 0);
    assert_eq!(prev_viewport.cursor().char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful_machine = NormalStateful::default();
    let next_stateful = stateful_machine.cursor_move(&data_access, Command::CursorMoveUp(1));
    assert!(matches!(next_stateful, StatefulValue::NormalMode(_)));

    let tree = data_access.tree.clone();
    let actual_viewport = get_viewport(tree);
    assert_eq!(actual_viewport.cursor().line_idx(), 0);
    assert_eq!(actual_viewport.cursor().char_idx(), 0);
  }

  #[test]
  fn cursor_move_vertically_nowrap3() {
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
    let (tree, state, bufs) = make_tree(
      U16Size::new(10, 10),
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      lines,
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_viewport = get_viewport(tree.clone());
    assert_eq!(prev_viewport.cursor().line_idx(), 0);
    assert_eq!(prev_viewport.cursor().char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful = NormalStateful::default();
    let next_stateful = stateful.cursor_move(&data_access, Command::CursorMoveDown(3));
    assert!(matches!(next_stateful, StatefulValue::NormalMode(_)));

    let stateful = match next_stateful {
      StatefulValue::NormalMode(s) => s,
      _ => unreachable!(),
    };

    let tree = data_access.tree.clone();
    let viewport1 = get_viewport(tree);
    assert_eq!(viewport1.cursor().line_idx(), 3);
    assert_eq!(viewport1.cursor().char_idx(), 0);

    let next_stateful = stateful.cursor_move(&data_access, Command::CursorMoveUp(1));
    assert!(matches!(next_stateful, StatefulValue::NormalMode(_)));

    let tree = data_access.tree.clone();
    let viewport2 = get_viewport(tree);
    assert_eq!(viewport2.cursor().line_idx(), 2);
    assert_eq!(viewport2.cursor().char_idx(), 0);
  }

  #[test]
  fn cursor_move_vertically_nowrap4() {
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
    let (tree, state, bufs) = make_tree(
      U16Size::new(10, 10),
      WindowLocalOptionsBuilder::default()
        .wrap(true)
        .build()
        .unwrap(),
      lines,
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_viewport = get_viewport(tree.clone());
    assert_eq!(prev_viewport.cursor().line_idx(), 0);
    assert_eq!(prev_viewport.cursor().char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful = NormalStateful::default();
    let next_stateful = stateful.cursor_move(&data_access, Command::CursorMoveDown(2));
    assert!(matches!(next_stateful, StatefulValue::NormalMode(_)));

    let stateful = match next_stateful {
      StatefulValue::NormalMode(s) => s,
      _ => unreachable!(),
    };

    let tree = data_access.tree.clone();
    let viewport1 = get_viewport(tree);
    assert_eq!(viewport1.cursor().line_idx(), 2);
    assert_eq!(viewport1.cursor().char_idx(), 0);

    let next_stateful = stateful.cursor_move(&data_access, Command::CursorMoveUp(1));
    assert!(matches!(next_stateful, StatefulValue::NormalMode(_)));

    let tree = data_access.tree.clone();
    let viewport2 = get_viewport(tree);
    assert_eq!(viewport2.cursor().line_idx(), 1);
    assert_eq!(viewport2.cursor().char_idx(), 0);
  }

  #[test]
  fn cursor_move_vertically_nowrap5() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let lines = vec![];
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let buf = make_buffer_from_lines(terminal_size.height(), buf_opts, lines);
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let tree = make_tree_with_buffers(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      bufs.clone(),
    );
    let state = State::to_arc(State::default());
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_viewport = get_viewport(tree.clone());
    assert_eq!(prev_viewport.cursor().line_idx(), 0);
    assert_eq!(prev_viewport.cursor().char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful_machine = NormalStateful::default();
    let next_stateful = stateful_machine.cursor_move(&data_access, Command::CursorMoveDown(1));
    assert!(matches!(next_stateful, StatefulValue::NormalMode(_)));

    let tree = data_access.tree.clone();
    let actual_viewport = get_viewport(tree);
    assert_eq!(actual_viewport.cursor().line_idx(), 0);
    assert_eq!(actual_viewport.cursor().char_idx(), 0);
  }

  #[test]
  fn cursor_move_vertically_wrap1() {
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
    let (tree, state, bufs) = make_tree(
      U16Size::new(10, 10),
      WindowLocalOptionsBuilder::default()
        .wrap(true)
        .build()
        .unwrap(),
      lines,
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_viewport = get_viewport(tree.clone());
    assert_eq!(prev_viewport.cursor().line_idx(), 0);
    assert_eq!(prev_viewport.cursor().char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful = NormalStateful::default();
    let next_stateful = stateful.cursor_move(&data_access, Command::CursorMoveDown(10));
    assert!(matches!(next_stateful, StatefulValue::NormalMode(_)));

    let stateful = match next_stateful {
      StatefulValue::NormalMode(s) => s,
      _ => unreachable!(),
    };

    let tree = data_access.tree.clone();
    let viewport1 = get_viewport(tree);
    assert_eq!(viewport1.cursor().line_idx(), 2);
    assert_eq!(viewport1.cursor().char_idx(), 0);

    let next_stateful = stateful.cursor_move(&data_access, Command::CursorMoveUp(1));
    assert!(matches!(next_stateful, StatefulValue::NormalMode(_)));

    let tree = data_access.tree.clone();
    let viewport2 = get_viewport(tree);
    assert_eq!(viewport2.cursor().line_idx(), 1);
    assert_eq!(viewport2.cursor().char_idx(), 0);
  }

  #[test]
  fn cursor_move_horizontally_nowrap1() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let lines = vec![];
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let buf = make_buffer_from_lines(terminal_size.height(), buf_opts, lines);
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let tree = make_tree_with_buffers(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      bufs.clone(),
    );
    let state = State::to_arc(State::default());
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_viewport = get_viewport(tree.clone());
    assert_eq!(prev_viewport.cursor().line_idx(), 0);
    assert_eq!(prev_viewport.cursor().char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful = NormalStateful::default();
    let next_stateful = stateful.cursor_move(&data_access, Command::CursorMoveRight(1));
    assert!(matches!(next_stateful, StatefulValue::NormalMode(_)));

    let tree = data_access.tree.clone();
    let actual_viewport = get_viewport(tree);
    assert_eq!(actual_viewport.cursor().line_idx(), 0);
    assert_eq!(actual_viewport.cursor().char_idx(), 0);
  }

  #[test]
  fn cursor_move_horizontally_nowrap2() {
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
    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let buf = make_buffer_from_lines(terminal_size.height(), buf_opts, lines);
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let tree = make_tree_with_buffers(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      bufs.clone(),
    );
    let state = State::to_arc(State::default());
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_viewport = get_viewport(tree.clone());
    assert_eq!(prev_viewport.cursor().line_idx(), 0);
    assert_eq!(prev_viewport.cursor().char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful = NormalStateful::default();
    let next_stateful = stateful.cursor_move(&data_access, Command::CursorMoveRight(1));
    assert!(matches!(next_stateful, StatefulValue::NormalMode(_)));

    let tree = data_access.tree.clone();
    let actual_viewport = get_viewport(tree);
    assert_eq!(actual_viewport.cursor().line_idx(), 0);
    assert_eq!(actual_viewport.cursor().char_idx(), 1);
  }

  #[test]
  fn cursor_move_horizontally_nowrap3() {
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

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let buf = make_buffer_from_lines(terminal_size.height(), buf_opts, lines);
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let tree = make_tree_with_buffers(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      bufs.clone(),
    );
    let state = State::to_arc(State::default());
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_viewport = get_viewport(tree.clone());
    assert_eq!(prev_viewport.cursor().line_idx(), 0);
    assert_eq!(prev_viewport.cursor().char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful = NormalStateful::default();
    let next_stateful = stateful.cursor_move(&data_access, Command::CursorMoveRight(20));
    assert!(matches!(next_stateful, StatefulValue::NormalMode(_)));

    let tree = data_access.tree.clone();
    let actual_viewport = get_viewport(tree);
    assert_eq!(actual_viewport.cursor().line_idx(), 0);
    assert_eq!(actual_viewport.cursor().char_idx(), 9);
  }

  #[test]
  fn cursor_move_horizontally_nowrap4() {
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
    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let buf = make_buffer_from_lines(terminal_size.height(), buf_opts, lines);
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let tree = make_tree_with_buffers(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      bufs.clone(),
    );
    let state = State::to_arc(State::default());
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_viewport = get_viewport(tree.clone());
    assert_eq!(prev_viewport.cursor().line_idx(), 0);
    assert_eq!(prev_viewport.cursor().char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful = NormalStateful::default();
    let next_stateful = stateful.cursor_move(&data_access, Command::CursorMoveRight(5));
    assert!(matches!(next_stateful, StatefulValue::NormalMode(_)));

    let tree = data_access.tree.clone();
    let actual_viewport = get_viewport(tree);
    assert_eq!(actual_viewport.cursor().line_idx(), 0);
    assert_eq!(actual_viewport.cursor().char_idx(), 5);

    let stateful = match next_stateful {
      StatefulValue::NormalMode(s) => s,
      _ => unreachable!(),
    };
    let next_stateful = stateful.cursor_move(&data_access, Command::CursorMoveLeft(3));
    assert!(matches!(next_stateful, StatefulValue::NormalMode(_)));

    let tree = data_access.tree.clone();
    let actual_viewport = get_viewport(tree);
    assert_eq!(actual_viewport.cursor().line_idx(), 0);
    assert_eq!(actual_viewport.cursor().char_idx(), 2);
  }

  #[test]
  fn cursor_move_horizontally_nowrap5() {
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
    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let buf = make_buffer_from_lines(terminal_size.height(), buf_opts, lines);
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let tree = make_tree_with_buffers(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      bufs.clone(),
    );
    let state = State::to_arc(State::default());
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_viewport = get_viewport(tree.clone());
    assert_eq!(prev_viewport.cursor().line_idx(), 0);
    assert_eq!(prev_viewport.cursor().char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful = NormalStateful::default();
    let next_stateful = stateful.cursor_move(&data_access, Command::CursorMoveRight(5));
    assert!(matches!(next_stateful, StatefulValue::NormalMode(_)));

    let tree = data_access.tree.clone();
    let actual_viewport = get_viewport(tree);
    assert_eq!(actual_viewport.cursor().line_idx(), 0);
    assert_eq!(actual_viewport.cursor().char_idx(), 5);

    for i in (0..=4).rev() {
      let stateful = match next_stateful {
        StatefulValue::NormalMode(s) => s,
        _ => unreachable!(),
      };
      let next_stateful = stateful.cursor_move(&data_access, Command::CursorMoveLeft(1));
      assert!(matches!(next_stateful, StatefulValue::NormalMode(_)));

      let tree = data_access.tree.clone();
      let actual_viewport = get_viewport(tree);
      assert_eq!(actual_viewport.cursor().line_idx(), 0);
      assert_eq!(actual_viewport.cursor().char_idx(), i);
    }
  }

  #[test]
  fn cursor_move_nowrap1() {
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
    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let buf = make_buffer_from_lines(terminal_size.height(), buf_opts, lines);
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let tree = make_tree_with_buffers(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      bufs.clone(),
    );
    let state = State::to_arc(State::default());
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_viewport = get_viewport(tree.clone());
    assert_eq!(prev_viewport.cursor().line_idx(), 0);
    assert_eq!(prev_viewport.cursor().char_idx(), 0);

    // Step-1
    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful = NormalStateful::default();
    let next_stateful = stateful.cursor_move(&data_access, Command::CursorMoveRight(5));
    assert!(matches!(next_stateful, StatefulValue::NormalMode(_)));

    let tree = data_access.tree.clone();
    let actual_viewport = get_viewport(tree);
    assert_eq!(actual_viewport.cursor().line_idx(), 0);
    assert_eq!(actual_viewport.cursor().char_idx(), 5);

    // Step-2
    let stateful = match next_stateful {
      StatefulValue::NormalMode(s) => s,
      _ => unreachable!(),
    };
    let next_stateful = stateful.cursor_move(&data_access, Command::CursorMoveDown(1));
    assert!(matches!(next_stateful, StatefulValue::NormalMode(_)));
    let tree = data_access.tree.clone();
    let actual_viewport = get_viewport(tree);
    assert_eq!(actual_viewport.cursor().line_idx(), 1);
    assert_eq!(actual_viewport.cursor().char_idx(), 5);

    // Step-3
    let stateful = match next_stateful {
      StatefulValue::NormalMode(s) => s,
      _ => unreachable!(),
    };
    let next_stateful = stateful.cursor_move(&data_access, Command::CursorMoveLeft(3));
    assert!(matches!(next_stateful, StatefulValue::NormalMode(_)));
    let tree = data_access.tree.clone();
    let actual_viewport = get_viewport(tree);
    assert_eq!(actual_viewport.cursor().line_idx(), 1);
    assert_eq!(actual_viewport.cursor().char_idx(), 2);

    // Step-4
    let stateful = match next_stateful {
      StatefulValue::NormalMode(s) => s,
      _ => unreachable!(),
    };
    let next_stateful = stateful.cursor_move(&data_access, Command::CursorMoveUp(1));
    assert!(matches!(next_stateful, StatefulValue::NormalMode(_)));
    let tree = data_access.tree.clone();
    let actual_viewport = get_viewport(tree);
    assert_eq!(actual_viewport.cursor().line_idx(), 0);
    assert_eq!(actual_viewport.cursor().char_idx(), 2);
  }

  #[test]
  fn cursor_move_nowrap2() {
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
    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let buf = make_buffer_from_lines(terminal_size.height(), buf_opts, lines);
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let tree = make_tree_with_buffers(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      bufs.clone(),
    );
    let state = State::to_arc(State::default());
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_viewport = get_viewport(tree.clone());
    assert_eq!(prev_viewport.cursor().line_idx(), 0);
    assert_eq!(prev_viewport.cursor().char_idx(), 0);

    for _ in 0..10 {
      let commands = [
        Command::CursorMoveDown(2),
        Command::CursorMoveRight(3),
        Command::CursorMoveUp(2),
        Command::CursorMoveLeft(3),
      ];
      let data_access = StatefulDataAccess::new(
        state.clone(),
        tree.clone(),
        bufs.clone(),
        Event::Key(key_event),
      );
      for c in commands.iter() {
        let stateful = NormalStateful::default();
        let next_stateful = stateful.cursor_move(&data_access, *c);
        assert!(matches!(next_stateful, StatefulValue::NormalMode(_)));
      }
      let tree = data_access.tree.clone();
      let actual_viewport = get_viewport(tree);
      assert_eq!(actual_viewport.cursor().line_idx(), 0);
      assert_eq!(actual_viewport.cursor().char_idx(), 0);
    }

    for _ in 0..10 {
      let commands = [
        Command::CursorMoveRight(5),
        Command::CursorMoveDown(1),
        Command::CursorMoveLeft(5),
        Command::CursorMoveUp(1),
      ];
      let data_access = StatefulDataAccess::new(
        state.clone(),
        tree.clone(),
        bufs.clone(),
        Event::Key(key_event),
      );
      for c in commands.iter() {
        let stateful = NormalStateful::default();
        let next_stateful = stateful.cursor_move(&data_access, *c);
        assert!(matches!(next_stateful, StatefulValue::NormalMode(_)));
      }
      let tree = data_access.tree.clone();
      let actual_viewport = get_viewport(tree);
      assert_eq!(actual_viewport.cursor().line_idx(), 0);
      assert_eq!(actual_viewport.cursor().char_idx(), 0);
    }
  }

  #[test]
  fn cursor_move_wrap1() {
    test_log_init();

    let lines = vec![
      "This is a quite simple test.\n",
      "It has these parts:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let terminal_size = U16Size::new(50, 50);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let buf = make_buffer_from_lines(terminal_size.height(), buf_opts, lines.clone());
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let tree = make_tree_with_buffers(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(true)
        .build()
        .unwrap(),
      bufs.clone(),
    );
    let state = State::to_arc(State::default());
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_viewport = get_viewport(tree.clone());
    assert_eq!(prev_viewport.cursor().line_idx(), 0);
    assert_eq!(prev_viewport.cursor().char_idx(), 0);

    // step-1: Move to the end of line-1.
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree.clone(),
      bufs.clone(),
      Event::Key(key_event),
    );
    let command = Command::CursorMoveRight(lines[0].len());
    let stateful = NormalStateful::default();
    let next_stateful = stateful.cursor_move(&data_access, command);

    assert!(matches!(next_stateful, StatefulValue::NormalMode(_)));
    let actual_tree = data_access.tree.clone();
    let actual_viewport = get_viewport(actual_tree.clone());
    assert_eq!(actual_viewport.cursor().line_idx(), 0);
    assert_eq!(actual_viewport.cursor().char_idx(), 27);

    // step-2: Move down to line-2.
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree.clone(),
      bufs.clone(),
      Event::Key(key_event),
    );
    let command = Command::CursorMoveDown(1);
    let stateful = NormalStateful::default();
    let next_stateful = stateful.cursor_move(&data_access, command);

    assert!(matches!(next_stateful, StatefulValue::NormalMode(_)));
    let actual_tree = data_access.tree.clone();
    let actual_viewport = get_viewport(actual_tree);
    assert_eq!(actual_viewport.cursor().line_idx(), 1);
    assert_eq!(actual_viewport.cursor().char_idx(), 18);
  }

  #[test]
  fn cursor_scroll_vertically_nowrap1() {
    test_log_init();

    let (tree, state, bufs) = make_tree(
      U16Size::new(10, 10),
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      vec![],
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_viewport = get_viewport(tree.clone());
    assert_eq!(prev_viewport.cursor().line_idx(), 0);
    assert_eq!(prev_viewport.cursor().char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful_machine = NormalStateful::default();
    let next_stateful = stateful_machine.cursor_scroll(&data_access, Command::CursorMoveUp(1));
    assert!(matches!(next_stateful, StatefulValue::NormalMode(_)));

    let tree = data_access.tree.clone();
    let actual_viewport = get_viewport(tree);
    assert_eq!(actual_viewport.cursor().line_idx(), 0);
    assert_eq!(actual_viewport.cursor().char_idx(), 0);
  }

  #[test]
  fn cursor_scroll_vertically_nowrap2() {
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
    let (tree, state, bufs) = make_tree(
      U16Size::new(10, 10),
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      lines,
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_viewport = get_viewport(tree.clone());
    assert_eq!(prev_viewport.cursor().line_idx(), 0);
    assert_eq!(prev_viewport.cursor().char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful_machine = NormalStateful::default();
    let next_stateful = stateful_machine.cursor_scroll(&data_access, Command::CursorMoveDown(1));
    assert!(matches!(next_stateful, StatefulValue::NormalMode(_)));

    let tree = data_access.tree.clone();
    let actual_viewport = get_viewport(tree);
    assert_eq!(actual_viewport.cursor().line_idx(), 0);
    assert_eq!(actual_viewport.cursor().char_idx(), 0);
  }

  #[test]
  fn cursor_scroll_vertically_nowrap3() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      "  3. If a single char needs multiple cells to display on the window, and it happens the char is at the end of the row, there can be multiple cases:\n",
      "     * The char exactly ends at the end of the row, i.e. the last display column of the char is exactly the last column on the row. In this case, we are happy because the char can be put at the end of the row.\n",
      "     * The char is too long to put at the end of the row, thus we will have to put the char to the beginning of the next row (because we don't cut a single char into pieces)\n",
    ];
    let (tree, state, bufs) = make_tree(
      U16Size::new(10, 7),
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      lines,
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_viewport = get_viewport(tree.clone());
    assert_eq!(prev_viewport.cursor().line_idx(), 0);
    assert_eq!(prev_viewport.cursor().char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful_machine = NormalStateful::default();
    let next_stateful = stateful_machine.cursor_scroll(&data_access, Command::CursorMoveDown(1));
    assert!(matches!(next_stateful, StatefulValue::NormalMode(_)));

    let tree = data_access.tree.clone();
    let actual_viewport = get_viewport(tree);
    assert_eq!(actual_viewport.cursor().line_idx(), 0);
    assert_eq!(actual_viewport.cursor().char_idx(), 0);
  }
}
