//! The normal mode.

use crate::buf::Buffer;
use crate::cart::U16Rect;
use crate::state::command::Command;
use crate::state::fsm::quit::QuitStateful;
use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::ui::tree::internal::{InodeBase, Inodeable};
use crate::ui::tree::TreeNode;
use crate::ui::widget::window::{CursorViewport, Viewport};
use crate::wlock;

use crossterm::event::{Event, KeyCode, KeyEventKind};
use std::ptr::NonNull;

#[derive(Debug, Copy, Clone, Default)]
/// The normal editing mode.
pub struct NormalStateful {}

impl Stateful for NormalStateful {
  fn handle(&self, data_access: StatefulDataAccess) -> StatefulValue {
    let _state = data_access.state;
    let tree = data_access.tree;
    let event = data_access.event;

    match event {
      Event::FocusGained => {}
      Event::FocusLost => {}
      Event::Key(key_event) => match key_event.kind {
        KeyEventKind::Press => {
          match key_event.code {
            KeyCode::Up | KeyCode::Char('k') => {
              // Up
              let mut tree = wlock!(tree);
              match tree.cursor_id() {
                Some(cursor_id) => {
                  tree.bounded_move_vertically_by(cursor_id, -1);
                }
                None => { /* Skip */ }
              }
            }
            KeyCode::Down | KeyCode::Char('j') => {
              // Down
              let mut tree = wlock!(tree);
              match tree.cursor_id() {
                Some(cursor_id) => {
                  tree.bounded_move_vertically_by(cursor_id, 1);
                }
                None => { /* Skip */ }
              }
            }
            KeyCode::Left | KeyCode::Char('h') => {
              // Left
              let mut tree = wlock!(tree);
              match tree.cursor_id() {
                Some(cursor_id) => {
                  tree.bounded_move_horizontally_by(cursor_id, -1);
                }
                None => { /* Skip */ }
              }
            }
            KeyCode::Right | KeyCode::Char('l') => {
              // Right
              let mut tree = wlock!(tree);
              match tree.cursor_id() {
                Some(cursor_id) => {
                  tree.bounded_move_horizontally_by(cursor_id, 1);
                }
                None => { /* Skip */ }
              }
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

    // quit loop
    if event == Event::Key(KeyCode::Esc.into()) {
      // println!("ESC: {:?}\r", crossterm::cursor::position());
      return StatefulValue::QuitState(QuitStateful::default());
    }

    StatefulValue::NormalMode(NormalStateful::default())
  }
}

impl NormalStateful {
  fn cursor_move(&self, data_access: StatefulDataAccess, command: Command) {
    let _state = data_access.state;
    let tree = data_access.tree;
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

          match command {
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
              viewport.set_cursor(line_idx, char_idx);

              let cursor_id = tree.cursor_id().unwrap();
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

              viewport.set_cursor(cursor_line_idx, char_idx);

              let cursor_id = tree.cursor_id().unwrap();
            }
            _ => unreachable!(),
          }
        }
      }
    }
  }

  fn quit(&self, data_access: StatefulDataAccess, command: Command) {}
}
