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

#[derive(Debug, Copy, Clone)]
/// The first is line index, second is char index.
pub struct CursorMoveResult(usize, usize);

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
          if let Some(CursorMoveResult(line_idx, char_idx)) = cursor_move_result {
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

  unsafe fn _cursor_move_vertically(
    &self,
    viewport: &Viewport,
    mut raw_buffer: NonNull<Buffer>,
    command: Command,
  ) -> Option<CursorMoveResult> {
    trace!("command:{:?}", command);
    let cursor_viewport = viewport.cursor();
    let cursor_line_idx = cursor_viewport.line_idx();
    let cursor_char_idx = cursor_viewport.char_idx();

    let line_idx = match command {
      Command::CursorMoveUp(n) => cursor_line_idx.saturating_sub(n as usize),
      Command::CursorMoveDown(n) => {
        let expected = cursor_line_idx.saturating_add(n as usize);
        let end_line_idx = viewport.end_line_idx();
        let upper_bound = end_line_idx.saturating_sub(1);
        trace!(
          "cursor_line_idx:{:?},expected:{:?},end_line_idx:{:?},upper_bound:{:?}",
          cursor_line_idx, expected, end_line_idx, upper_bound
        );
        std::cmp::min(expected, upper_bound)
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
      let cursor_col_idx = raw_buffer
        .as_mut()
        .width_before(cursor_line_idx, cursor_char_idx);
      let char_idx = match raw_buffer.as_mut().char_after(line_idx, cursor_col_idx) {
        Some(char_idx) => char_idx,
        None => raw_buffer.as_ref().get_rope().line(line_idx).len_chars() - 1,
      };
      trace!("cursor_col_idx:{},char_idx:{}", cursor_col_idx, char_idx);
      Some(CursorMoveResult(line_idx, char_idx))
    }
  }

  unsafe fn _cursor_move_horizontally(
    &self,
    viewport: &Viewport,
    raw_buffer: NonNull<Buffer>,
    command: Command,
  ) -> Option<CursorMoveResult> {
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
        Command::CursorMoveLeft(n) => cursor_char_idx.saturating_sub(n as usize),
        Command::CursorMoveRight(n) => {
          let expected = cursor_char_idx.saturating_add(n as usize);
          let last_char_idx = {
            let cursor_line = raw_buffer
              .as_ref()
              .get_rope()
              .get_line(cursor_line_idx)
              .unwrap();
            assert!(viewport.lines().contains_key(&cursor_line_idx));
            let line_viewport = viewport.lines().get(&cursor_line_idx).unwrap();
            let (_last_row_idx, last_row_viewport) = line_viewport.rows().last_key_value().unwrap();
            let mut c = last_row_viewport.end_char_idx() - 1;
            trace!(
              "cursor_char_idx:{}, expected:{}, last_row_viewport:{:?}, c:{}",
              cursor_char_idx, expected, last_row_viewport, c
            );
            while raw_buffer
              .as_ref()
              .char_width(cursor_line.get_char(c).unwrap())
              == 0
            {
              c = c.saturating_sub(1);
              if c == 0 {
                break;
              }
            }
            c
          };
          std::cmp::min(expected, last_char_idx)
        }
        _ => unreachable!(),
      };

      Some(CursorMoveResult(cursor_line_idx, char_idx))
    }
  }

  /// Cursor move up/down/left/right in current window, or scroll buffer up/down if it reaches the
  /// top/bottom of the window and the buffer has more contents.
  fn _cursor_move_or_scroll(
    &self,
    _data_access: &StatefulDataAccess,
    _command: Command,
  ) -> StatefulValue {
    StatefulValue::NormalMode(NormalStateful::default())
  }

  /// Cursor scroll buffer up/down in current window.
  /// NOTE: The cursor actually stays still in the window, its "position" is not changed. The
  /// buffer contents changed, i.e. moved up/down.
  fn _cursor_scroll(&self, data_access: &StatefulDataAccess, command: Command) -> StatefulValue {
    let tree = data_access.tree.clone();
    let mut tree = wlock!(tree);

    if let Some(current_window_id) = tree.current_window_id() {
      if let Some(TreeNode::Window(current_window)) = tree.node_mut(&current_window_id) {
        let viewport = current_window.viewport();
        let viewport = wlock!(viewport);
        let buffer = viewport.buffer();
        let buffer = buffer.upgrade().unwrap();
        let mut _buffer = wlock!(buffer);

        match command {
          Command::CursorScrollUp(_n) | Command::CursorScrollDown(_n) => {}
          Command::CursorScrollLeft(_n) | Command::CursorScrollRight(_n) => {}
          _ => unreachable!(),
        }
      }
    }

    StatefulValue::NormalMode(NormalStateful::default())
  }

  fn _cursor_scroll_vertically(
    &self,
    data_access: &StatefulDataAccess,
    command: Command,
  ) -> StatefulValue {
    let tree = data_access.tree.clone();
    let mut tree = wlock!(tree);

    if let Some(current_window_id) = tree.current_window_id() {
      if let Some(TreeNode::Window(current_window)) = tree.node_mut(&current_window_id) {
        let viewport = current_window.viewport();
        let viewport = wlock!(viewport);
        let buffer = viewport.buffer();
        let buffer = buffer.upgrade().unwrap();
        let mut _buffer = wlock!(buffer);

        match command {
          Command::CursorScrollUp(_n) => {}
          Command::CursorScrollDown(_n) => {}
          _ => unreachable!(),
        }
      }
    }

    StatefulValue::NormalMode(NormalStateful::default())
  }

  fn _cursor_scroll_horizontally(
    &self,
    data_access: &StatefulDataAccess,
    command: Command,
  ) -> StatefulValue {
    let tree = data_access.tree.clone();
    let mut tree = wlock!(tree);

    if let Some(current_window_id) = tree.current_window_id() {
      if let Some(TreeNode::Window(current_window)) = tree.node_mut(&current_window_id) {
        let viewport = current_window.viewport();
        let viewport = wlock!(viewport);
        let buffer = viewport.buffer();
        let buffer = buffer.upgrade().unwrap();
        let mut _buffer = wlock!(buffer);

        match command {
          Command::CursorScrollLeft(_n) => {}
          Command::CursorScrollRight(_n) => {}
          _ => unreachable!(),
        }
      }
    }

    StatefulValue::NormalMode(NormalStateful::default())
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
  fn cursor_move_vertically1() {
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
  fn cursor_move_vertically2() {
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
  fn cursor_move_vertically3() {
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
  fn cursor_move_vertically4() {
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
  fn cursor_move_vertically5() {
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
  fn cursor_move_vertically6() {
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
  fn cursor_move_horizontally1() {
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
  fn cursor_move_horizontally2() {
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
  fn cursor_move_horizontally3() {
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
  fn cursor_move_horizontally4() {
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
  fn cursor_move_horizontally5() {
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
  fn cursor_move1() {
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
  fn cursor_move2() {
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
}
