//! The normal mode.

use crate::buf::Buffer;
use crate::cart::IRect;
use crate::state::command::Command;
use crate::state::fsm::quit::QuitStateful;
use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::ui::tree::internal::Inodeable;
use crate::ui::tree::TreeNode;
use crate::wlock;

use crossterm::event::{Event, KeyCode, KeyEventKind};
use std::ptr::NonNull;

#[derive(Debug, Copy, Clone, Default)]
/// The normal editing mode.
pub struct NormalStateful {}

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
    //   println!("Curosr position: {:?}\r", crossterm::cursor::position());
    // }

    // // quit loop
    // if event == Event::Key(KeyCode::Esc.into()) {
    //   // println!("ESC: {:?}\r", crossterm::cursor::position());
    //   return StatefulValue::QuitState(QuitStateful::default());
    // }

    StatefulValue::NormalMode(NormalStateful::default())
  }
}

impl NormalStateful {
  fn cursor_move(&self, data_access: &StatefulDataAccess, command: Command) -> StatefulValue {
    let tree = data_access.tree.clone();
    let mut tree = wlock!(tree);

    if let Some(current_window_id) = tree.current_window_id() {
      if let Some(TreeNode::Window(current_window_node)) = tree.node_mut(&current_window_id) {
        let viewport = current_window_node.viewport();
        let mut viewport = wlock!(viewport);
        let cursor_viewport = viewport.cursor();
        let cursor_line_idx = cursor_viewport.line_idx();
        let cursor_char_idx = cursor_viewport.char_idx();

        let buffer = viewport.buffer();
        let buffer = buffer.upgrade().unwrap();
        let mut buffer = wlock!(buffer);
        unsafe {
          // Fix multiple mutable references on `buffer`.
          let mut raw_buffer = NonNull::new(&mut *buffer as *mut Buffer).unwrap();

          let (line_idx, char_idx) = match command {
            Command::CursorMoveUp(_) | Command::CursorMoveDown(_) => {
              let line_idx = match command {
                Command::CursorMoveUp(n) => cursor_line_idx.saturating_sub(n as usize),
                Command::CursorMoveDown(n) => std::cmp::max(
                  cursor_line_idx.saturating_add(n as usize),
                  buffer.get_rope().len_lines(),
                ),
                _ => unreachable!(),
              };
              debug_assert!(buffer.get_rope().get_line(line_idx).is_some());
              debug_assert!(buffer.get_rope().get_line(line_idx).unwrap().len_chars() > 0);
              let cursor_col_idx = raw_buffer
                .as_mut()
                .width_before(cursor_line_idx, cursor_char_idx);
              let char_idx = match raw_buffer.as_mut().char_at(line_idx, cursor_col_idx) {
                Some(char_idx) => char_idx,
                None => buffer.get_rope().line(line_idx).len_chars() - 1,
              };
              // let col_start = raw_buffer.as_mut().width_before(line_idx, char_idx);
              // let col_end = raw_buffer.as_mut().width_at(line_idx, char_idx);
              (line_idx, char_idx)
            }
            Command::CursorMoveLeft(_) | Command::CursorMoveRight(_) => {
              debug_assert!(buffer.get_rope().get_line(cursor_line_idx).is_some());
              debug_assert!(
                buffer
                  .get_rope()
                  .get_line(cursor_line_idx)
                  .unwrap()
                  .len_chars()
                  > 0
              );
              let char_idx = match command {
                Command::CursorMoveLeft(n) => cursor_char_idx.saturating_sub(n as usize),
                Command::CursorMoveRight(n) => std::cmp::max(
                  cursor_char_idx.saturating_add(n as usize),
                  buffer
                    .get_rope()
                    .get_line(cursor_line_idx)
                    .unwrap()
                    .len_chars()
                    - 1,
                ),
                _ => unreachable!(),
              };

              (cursor_line_idx, char_idx)
            }
            _ => unreachable!(),
          };

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
          assert!(cursor_row.len() == 1);

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
        }
      }
    }
    StatefulValue::NormalMode(NormalStateful::default())
  }

  fn quit(&self, _data_access: &StatefulDataAccess, _command: Command) -> StatefulValue {
    StatefulValue::QuitState(QuitStateful::default())
  }
}
