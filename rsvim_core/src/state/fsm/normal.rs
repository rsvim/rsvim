//! The normal mode.

#![allow(unused_imports)]

use crate::envar;
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
    match tree.current_window_id() {
      Some(current_window_id) => {
        let cursor_id = tree.cursor_id().unwrap();

        match tree.node_mut(&current_window_id) {
          Some(current_window) => match current_window {
            TreeNode::Window(cur_win) => {
              let viewport = cur_win.viewport();
              let viewport = wlock!(viewport);
              let cursor_viewport = viewport.cursor();

              let next_cursor_viewport = match command {
                Command::CursorMoveLeft(n) => {
                  let line_idx = cursor_viewport.line_idx();
                  let row_idx = cursor_viewport.row_idx();
                  let line_viewport = viewport.lines().get(&line_idx).unwrap();
                  let line_viewport_row = line_viewport.rows().get(&row_idx).unwrap();

                  let next_char_idx = if cursor_viewport.char_idx() > 0 {
                    cursor_viewport.char_idx() - 1
                  } else {
                    0
                  };

                  let (next_start_dcolumn, next_end_dcolumn) = line_viewport_row
                    .char2dcolumns()
                    .get(&next_char_idx)
                    .unwrap();
                  let next_cursor_viewport = CursorViewport::new(
                    *next_start_dcolumn..*next_end_dcolumn,
                    next_char_idx,
                    row_idx,
                    line_idx,
                  );

                  // If cursor is already 
                  if cursor_viewport.char_idx() == 0 {
                    assert!(*cursor_viewport == next_cursor_viewport);
                  }

                  next_cursor_viewport
                }
                Command::CursorMoveRight(n) => {
                  let line_idx = cursor_viewport.line_idx();
                  let row_idx = cursor_viewport.row_idx();
                  let line_viewport = viewport.lines().get(&line_idx).unwrap();
                  let line_viewport_row = line_viewport.rows().get(&row_idx).unwrap();

                  let next_char_idx = if cursor_viewport.char_idx() > 0 {
                    cursor_viewport.char_idx() - 1
                  } else {
                    0
                  };

                  if line_viewport_row.end_char_idx() > 0
                    && cursor_viewport.char_idx() < line_viewport_row.end_char_idx() - 1
                  {
                    let next_char_idx = cursor_viewport.char_idx() + 1;
                    let (next_start_dcolumn, next_end_dcolumn) = line_viewport_row
                      .char2dcolumns()
                      .get(&next_char_idx)
                      .unwrap();
                    CursorViewport::new(
                      *next_start_dcolumn..*next_end_dcolumn,
                      next_char_idx,
                      row_idx,
                      line_idx,
                    )
                  } else {
                    cursor_viewport.clone()
                  };
                }
              };
            }
            _ => unreachable!("Cursor widget parent must be window widget."),
          },
          None => { /* Skip */ }
        }
      }
      None => { /* Skip */ }
    }

    match command {
      Command::CursorMoveUp(n) => {}
      Command::CursorMoveDown(n) => {}
      Command::CursorMoveLeft(n) => {}
      Command::CursorMoveRight(n) => {}
    }
  }

  fn quit(&self, data_access: StatefulDataAccess) {}
}
