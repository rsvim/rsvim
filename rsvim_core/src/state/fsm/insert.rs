//! The insert mode.

use crate::prelude::*;
use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::ops::cursor_ops;
use crate::state::ops::{CursorInsertPayload, Operation};
use crate::ui::canvas::CursorStyle;
use crate::ui::tree::*;

use compact_str::ToCompactString;
use crossterm::event::{Event, KeyCode, KeyEventKind};

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The finite-state-machine for insert mode.
pub struct InsertStateful {}

impl InsertStateful {
  fn get_operation(
    &self,
    data_access: &StatefulDataAccess,
  ) -> Option<Operation> {
    let event = &data_access.event;

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
            KeyCode::Char(c) => Some(Operation::CursorInsert(
              CursorInsertPayload::Text(c.to_compact_string()),
            )),
            KeyCode::Tab => {
              Some(Operation::CursorInsert(CursorInsertPayload::Tab))
            }
            KeyCode::Enter => {
              Some(Operation::CursorInsert(CursorInsertPayload::Eol))
            }
            KeyCode::Backspace => Some(Operation::CursorDelete(-1)),
            KeyCode::Delete => Some(Operation::CursorDelete(1)),
            KeyCode::Esc => Some(Operation::GotoNormalMode),
            _ => None,
          }
        }
        KeyEventKind::Repeat => None,
        KeyEventKind::Release => None,
      },
      Event::Mouse(_mouse_event) => None,
      Event::Paste(_paste_string) => None,
      Event::Resize(_columns, _rows) => None,
    }
  }
}

impl Stateful for InsertStateful {
  fn handle(&self, data_access: StatefulDataAccess) -> StatefulValue {
    if let Some(op) = self.get_operation(&data_access) {
      return self.handle_op(data_access, op);
    }

    StatefulValue::InsertMode(InsertStateful::default())
  }

  fn handle_op(
    &self,
    data_access: StatefulDataAccess,
    op: Operation,
  ) -> StatefulValue {
    match op {
      Operation::GotoNormalMode => self.goto_normal_mode(&data_access),
      Operation::CursorMoveBy((_, _))
      | Operation::CursorMoveUpBy(_)
      | Operation::CursorMoveDownBy(_)
      | Operation::CursorMoveLeftBy(_)
      | Operation::CursorMoveRightBy(_)
      | Operation::CursorMoveTo((_, _)) => self.cursor_move(&data_access, op),
      Operation::CursorInsert(payload) => {
        self.cursor_insert(&data_access, payload)
      }
      Operation::CursorDelete(n) => self.cursor_delete(&data_access, n),
      _ => unreachable!(),
    }
  }
}

impl InsertStateful {
  pub fn cursor_delete(
    &self,
    data_access: &StatefulDataAccess,
    n: isize,
  ) -> StatefulValue {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);
    let current_window = tree.current_window_mut().unwrap();
    let current_window_id = current_window.id();
    let buffer = current_window.buffer().upgrade().unwrap();
    let mut buffer = lock!(buffer);

    cursor_ops::cursor_delete(
      &mut tree,
      current_window_id,
      buffer.text_mut(),
      n,
    );

    StatefulValue::InsertMode(InsertStateful::default())
  }
}

impl InsertStateful {
  pub fn cursor_insert(
    &self,
    data_access: &StatefulDataAccess,
    payload: CursorInsertPayload,
  ) -> StatefulValue {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);
    let current_window = tree.current_window_mut().unwrap();
    let current_window_id = current_window.id();
    let buffer = current_window.buffer().upgrade().unwrap();
    let mut buffer = lock!(buffer);

    let payload = match payload {
      CursorInsertPayload::Text(c) => c,
      CursorInsertPayload::Tab => '\t'.to_compact_string(),
      CursorInsertPayload::Eol => {
        let eol = buffer.options().end_of_line();
        let eol = format!("{eol}");
        trace!("Insert eol:{eol:?}");
        eol.to_compact_string()
      }
    };

    cursor_ops::cursor_insert(
      &mut tree,
      current_window_id,
      buffer.text_mut(),
      payload,
    );

    StatefulValue::InsertMode(InsertStateful::default())
  }
}

impl InsertStateful {
  pub fn goto_normal_mode(
    &self,
    data_access: &StatefulDataAccess,
  ) -> StatefulValue {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);
    let current_window = tree.current_window_mut().unwrap();
    let current_window_id = current_window.id();
    let buffer = current_window.buffer().upgrade().unwrap();
    let buffer = lock!(buffer);

    let op = Operation::CursorMoveBy((0, 0));
    cursor_ops::cursor_move(
      &mut tree,
      current_window_id,
      buffer.text(),
      op,
      false,
    );

    let current_window = tree.current_window_mut().unwrap();
    debug_assert!(current_window.cursor_id().is_some());
    let _cursor_id = current_window.cursor_id().unwrap();
    debug_assert!(current_window.cursor_mut().is_some());
    let cursor = current_window.cursor_mut().unwrap();
    debug_assert_eq!(_cursor_id, cursor.id());
    cursor.set_style(&CursorStyle::SteadyBlock);

    StatefulValue::NormalMode(super::NormalStateful::default())
  }
}

impl InsertStateful {
  pub fn cursor_move(
    &self,
    data_access: &StatefulDataAccess,
    op: Operation,
  ) -> StatefulValue {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);
    let current_window = tree.current_window_mut().unwrap();
    let current_window_id = current_window.id();
    let buffer = current_window.buffer().upgrade().unwrap();
    let buffer = lock!(buffer);

    cursor_ops::cursor_move(
      &mut tree,
      current_window_id,
      buffer.text(),
      op,
      true,
    );

    StatefulValue::InsertMode(InsertStateful::default())
  }
}
