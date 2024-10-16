//! The normal mode.

#![allow(unused_imports)]

use crossterm::event::{Event, KeyCode, KeyEventKind, KeyEventState, KeyModifiers};
use std::time::Duration;

use crate::glovar;
use crate::state::fsm::quit::QuitStateful;
use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::mode::Mode;

#[derive(Debug, Copy, Clone, Default)]
/// The normal editing mode.
pub struct NormalStateful {}

impl Stateful for NormalStateful {
  fn handle(&self, data_access: StatefulDataAccess) -> StatefulValue {
    let state = data_access.state;
    let tree = data_access.tree;
    let event = data_access.event;

    state.set_mode(Mode::Normal);

    match event {
      Event::FocusGained => {}
      Event::FocusLost => {}
      Event::Key(key_event) => match key_event.kind {
        KeyEventKind::Press => {
          match key_event.code {
            KeyCode::Up | KeyCode::Char('k') => {
              // Up
              let mut tree = tree.try_write_for(glovar::MUTEX_TIMEOUT()).unwrap();
              match tree.cursor_id() {
                Some(cursor_id) => {
                  tree.bounded_move_up_by(cursor_id, 1);
                }
                None => { /* Skip */ }
              }
            }
            KeyCode::Down | KeyCode::Char('j') => {
              // Down
              let mut tree = tree.try_write_for(glovar::MUTEX_TIMEOUT()).unwrap();
              match tree.cursor_id() {
                Some(cursor_id) => {
                  tree.bounded_move_down_by(cursor_id, 1);
                }
                None => { /* Skip */ }
              }
            }
            KeyCode::Left | KeyCode::Char('h') => {
              // Left
              let mut tree = tree.try_write_for(glovar::MUTEX_TIMEOUT()).unwrap();
              match tree.cursor_id() {
                Some(cursor_id) => {
                  tree.bounded_move_left_by(cursor_id, 1);
                }
                None => { /* Skip */ }
              }
            }
            KeyCode::Right | KeyCode::Char('l') => {
              // Right
              let mut tree = tree.try_write_for(glovar::MUTEX_TIMEOUT()).unwrap();
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
