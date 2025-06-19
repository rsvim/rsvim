//! The command-line mode, ex-command variant.

use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::ops::Operation;

use compact_str::{CompactString, ToCompactString};
use crossterm::event::{Event, KeyCode, KeyEventKind};
use tracing::trace;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The command-line mode, ex-command variant.
pub struct CommandLineExStateful {}

impl CommandLineExStateful {
  fn _get_operation(&self, event: Event) -> Option<Operation> {
    match event {
      Event::FocusGained => None,
      Event::FocusLost => None,
      Event::Key(key_event) => match key_event.kind {
        KeyEventKind::Press => {
          trace!("Event::key:{:?}", key_event);
          match key_event.code {
            // KeyCode::Up | KeyCode::Char('k') => Some(Operation::CursorMoveUpBy(1)),
            // KeyCode::Down | KeyCode::Char('j') => Some(Operation::CursorMoveDownBy(1)),
            KeyCode::Left | KeyCode::Char('h') => Some(Operation::CursorMoveLeftByCommandLineEx(1)),
            KeyCode::Right | KeyCode::Char('l') => {
              Some(Operation::CursorMoveRightByCommandLineEx(1))
            }
            KeyCode::Home => Some(Operation::CursorMoveLeftByCommandLineEx(usize::MAX)),
            KeyCode::End => Some(Operation::CursorMoveRightByCommandLineEx(usize::MAX)),
            KeyCode::Char(c) => Some(Operation::InsertAtCursorCommandLineEx(
              c.to_compact_string(),
            )),
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

impl Stateful for CommandLineExStateful {
  fn handle(&self, data_access: StatefulDataAccess) -> StatefulValue {
    let event = data_access.event.clone();

    if let Some(op) = self._get_operation(event) {
      return self.handle_op(data_access, op);
    }

    StatefulValue::CommandLineExMode(CommandLineExStateful::default())
  }

  fn handle_op(&self, data_access: StatefulDataAccess, op: Operation) -> StatefulValue {
    match op {
      Operation::CursorMoveByCommandLineEx((_, _))
      | Operation::CursorMoveUpByCommandLineEx(_)
      | Operation::CursorMoveDownByCommandLineEx(_)
      | Operation::CursorMoveLeftByCommandLineEx(_)
      | Operation::CursorMoveRightByCommandLineEx(_)
      | Operation::CursorMoveToCommandLineEx((_, _)) => self.cursor_move(&data_access, op),
      Operation::GotoNormalMode => self.goto_normal_mode(&data_access),
      Operation::InsertAtCursorCommandLineEx(text) => self.insert_at_cursor(&data_access, text),
      Operation::DeleteAtCursorCommandLineEx(n) => self.delete_at_cursor(&data_access, n),
      _ => unreachable!(),
    }
  }
}

impl CommandLineExStateful {
  fn goto_normal_mode(&self, _data_access: &StatefulDataAccess) -> StatefulValue {
    StatefulValue::NormalMode(super::NormalStateful::default())
  }
}

impl CommandLineExStateful {
  fn cursor_move(&self, _data_access: &StatefulDataAccess, _op: Operation) -> StatefulValue {
    StatefulValue::CommandLineExMode(CommandLineExStateful::default())
  }
}

impl CommandLineExStateful {
  fn insert_at_cursor(
    &self,
    _data_access: &StatefulDataAccess,
    _text: CompactString,
  ) -> StatefulValue {
    StatefulValue::CommandLineExMode(CommandLineExStateful::default())
  }
}

impl CommandLineExStateful {
  fn delete_at_cursor(&self, _data_access: &StatefulDataAccess, _n: isize) -> StatefulValue {
    StatefulValue::CommandLineExMode(CommandLineExStateful::default())
  }
}
