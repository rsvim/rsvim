//! The normal mode.

use crate::buf::Buffer;
use crate::cart::IRect;
use crate::state::command::Command;
use crate::state::fsm::quit::QuitStateful;
use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::ui::tree::internal::Inodeable;
use crate::ui::tree::TreeNode;
use crate::ui::widget::window::CursorViewport;
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
  fn cursor_move(&self, data_access: &StatefulDataAccess, command: Command) -> StatefulValue {
    let tree = data_access.tree.clone();
    let mut tree = wlock!(tree);

    if let Some(current_window_id) = tree.current_window_id() {
      if let Some(TreeNode::Window(current_window)) = tree.node_mut(&current_window_id) {
        let viewport = current_window.viewport();
        let mut viewport = wlock!(viewport);
        let cursor_viewport = viewport.cursor();
        let buffer = viewport.buffer();
        let buffer = buffer.upgrade().unwrap();
        let mut buffer = wlock!(buffer);
        unsafe {
          // Fix multiple mutable references on `buffer`.
          let mut raw_buffer: NonNull<Buffer> = NonNull::new(&mut *buffer as *mut Buffer).unwrap();

          let cursor_move_result = match command {
            Command::CursorMoveUp(_) | Command::CursorMoveDown(_) => {
              self.cursor_move_vertically(cursor_viewport, raw_buffer, command)
            }
            Command::CursorMoveLeft(_) | Command::CursorMoveRight(_) => {
              self.cursor_move_horizontally(cursor_viewport, raw_buffer, command)
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
                row_viewport.start_char_idx() >= char_idx && row_viewport.end_char_idx() < char_idx
              })
              .collect::<Vec<_>>();
            assert_eq!(cursor_row.len(), 1);

            let (row_idx, row_viewport) = cursor_row[0];
            let cursor_id = tree.cursor_id().unwrap();

            if let Some(&mut TreeNode::Cursor(ref mut cursor_node)) = tree.node_mut(&cursor_id) {
              let row_start_width = raw_buffer
                .as_mut()
                .width_before(line_idx, row_viewport.start_char_idx());
              let char_start_width = raw_buffer.as_mut().width_before(line_idx, char_idx);
              let col_idx = (char_start_width - row_start_width) as isize;
              let shape = IRect::new((*row_idx as isize, col_idx), (*row_idx as isize, col_idx));
              cursor_node.set_shape(&shape);
            } else {
              unreachable!();
            }
          } else {
            // Do nothing, stay at where you are
          }
        }
      }
    }
    StatefulValue::NormalMode(NormalStateful::default())
  }

  unsafe fn cursor_move_vertically(
    &self,
    cursor_viewport: &CursorViewport,
    mut raw_buffer: NonNull<Buffer>,
    command: Command,
  ) -> Option<CursorMoveResult> {
    trace!("command:{:?}", command);
    let cursor_line_idx = cursor_viewport.line_idx();
    let cursor_char_idx = cursor_viewport.char_idx();

    let line_idx = match command {
      Command::CursorMoveUp(n) => cursor_line_idx.saturating_sub(n as usize),
      Command::CursorMoveDown(n) => std::cmp::min(
        cursor_line_idx.saturating_add(n as usize),
        raw_buffer.as_ref().get_rope().len_lines(),
      ),
      _ => unreachable!(),
    };
    trace!(
      "cursor:{}/{},line_idx:{}",
      cursor_line_idx,
      cursor_char_idx,
      line_idx
    );

    // If line index doesn't change, early return.
    if line_idx == cursor_line_idx {
      return None;
    }

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

  unsafe fn cursor_move_horizontally(
    &self,
    cursor_viewport: &CursorViewport,
    raw_buffer: NonNull<Buffer>,
    command: Command,
  ) -> Option<CursorMoveResult> {
    let cursor_line_idx = cursor_viewport.line_idx();
    let cursor_char_idx = cursor_viewport.char_idx();

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
      Command::CursorMoveRight(n) => std::cmp::max(
        cursor_char_idx.saturating_add(n as usize),
        raw_buffer
          .as_ref()
          .get_rope()
          .get_line(cursor_line_idx)
          .unwrap()
          .len_chars()
          - 1,
      ),
      _ => unreachable!(),
    };

    Some(CursorMoveResult(cursor_line_idx, char_idx))
  }

  fn quit(&self, _data_access: &StatefulDataAccess, _command: Command) -> StatefulValue {
    StatefulValue::QuitState(QuitStateful::default())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use crate::buf::{BufferLocalOptions, BuffersManagerArc};
  use crate::cart::U16Size;
  use crate::rlock;
  use crate::state::{State, StateArc};
  use crate::test::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::test::log::init as test_log_init;
  use crate::test::tree::make_tree_with_buffers;
  use crate::ui::tree::TreeArc;
  use crate::ui::widget::window::{Viewport, WindowLocalOptions};

  use crossterm::event::Event;
  use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

  fn make_tree(
    window_local_opts: WindowLocalOptions,
    canvas_size: U16Size,
    lines: Vec<&str>,
  ) -> (TreeArc, StateArc, BuffersManagerArc) {
    let buf_opts = BufferLocalOptions::default();
    let buf = make_buffer_from_lines(buf_opts.clone(), lines);
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let tree = make_tree_with_buffers(window_local_opts, canvas_size, bufs.clone());
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
  fn cursor_move_up1() {
    test_log_init();

    let (tree, state, bufs) = make_tree(
      WindowLocalOptions::builder().wrap(false).build(),
      U16Size::new(10, 10),
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
  fn cursor_move_up2() {
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
      WindowLocalOptions::builder().wrap(false).build(),
      U16Size::new(10, 10),
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
  fn cursor_move_up3() {
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
      WindowLocalOptions::builder().wrap(false).build(),
      U16Size::new(10, 10),
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
  fn cursor_move_down1() {
    test_log_init();

    let lines = vec![];
    let buf_opts = BufferLocalOptions::default();
    let buf = make_buffer_from_lines(buf_opts.clone(), lines);
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let tree = make_tree_with_buffers(
      WindowLocalOptions::builder().wrap(false).build(),
      U16Size::new(10, 10),
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
}
