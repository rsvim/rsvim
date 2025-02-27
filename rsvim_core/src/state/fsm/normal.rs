//! The normal mode.

use crate::state::command::Command;
use crate::state::fsm::quit::QuitStateful;
use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::mode::Mode;
use crate::ui::tree::TreeNode;
use crate::ui::widget::window::CursorViewport;
use crate::wlock;

use crossterm::event::{Event, KeyCode, KeyEventKind, KeyEventState, KeyModifiers};
use std::time::Duration;

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
                  tree.bounded_move_up_by(cursor_id, 1);
                }
                None => { /* Skip */ }
              }
            }
            KeyCode::Down | KeyCode::Char('j') => {
              // Down
              let mut tree = wlock!(tree);
              match tree.cursor_id() {
                Some(cursor_id) => {
                  tree.bounded_move_down_by(cursor_id, 1);
                }
                None => { /* Skip */ }
              }
            }
            KeyCode::Left | KeyCode::Char('h') => {
              // Left
              let mut tree = wlock!(tree);
              match tree.cursor_id() {
                Some(cursor_id) => {
                  tree.bounded_move_left_by(cursor_id, 1);
                }
                None => { /* Skip */ }
              }
            }
            KeyCode::Right | KeyCode::Char('l') => {
              // Right
              let mut tree = wlock!(tree);
              match tree.cursor_id() {
                Some(cursor_id) => {
                  tree.bounded_move_right_by(cursor_id, 1);
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
  fn handle_cursor_move(&self, data_access: StatefulDataAccess, command: Command) {
    let _state = data_access.state;
    let tree = data_access.tree;

    let mut tree = wlock!(tree);
    let cursor_id = tree.cursor_id().unwrap();

    if let Some(current_window_id) = tree.current_window_id() {
      if let Some(current_window) = tree.node_mut(&current_window_id) {
        if let TreeNode::Window(current_window_node) = current_window {
          let viewport = current_window_node.viewport();
          let viewport = wlock!(viewport);
          let buffer = viewport.buffer();
          let buffer = buffer.upgrade().unwrap();
          let buffer = wlock!(buffer);
          let cursor_viewport = viewport.cursor();
          match command {
            Command::CursorMoveUp(n) => {}
            Command::CursorMoveDown(n) => {}
            Command::CursorMoveLeft(n) => {}
            Command::CursorMoveRight(n) => {}
          }
        }
      }
    }
  }

  fn quit(&self, data_access: StatefulDataAccess) {}
}
