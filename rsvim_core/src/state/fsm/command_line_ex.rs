//! The command-line ex mode.

use crate::js::{self, next_future_id};
use crate::msg::{self, ExCommandReq, JsMessage, MasterMessage};
use crate::prelude::*;
use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::ops::{
  CursorInsertPayload, Operation, cmdline_ops, cursor_ops,
};
use crate::ui::canvas::CursorStyle;
use crate::ui::tree::*;
use crate::ui::widget::command_line::CommandLineNode;
use crate::ui::widget::command_line::indicator::IndicatorSymbol;

use compact_str::{CompactString, ToCompactString};
use crossterm::event::{Event, KeyCode, KeyEventKind};

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The command-line ex mode.
pub struct CommandLineExStateful {}

impl CommandLineExStateful {
  fn get_operation(&self, event: Event) -> Option<Operation> {
    match event {
      Event::FocusGained => None,
      Event::FocusLost => None,
      Event::Key(key_event) => match key_event.kind {
        KeyEventKind::Press => {
          trace!("key_event:{key_event:?}");
          match key_event.code {
            // KeyCode::Up | KeyCode::Char('k') => Some(Operation::CursorMoveUpBy(1)),
            // KeyCode::Down | KeyCode::Char('j') => Some(Operation::CursorMoveDownBy(1)),
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
            KeyCode::Backspace => Some(Operation::CursorDelete(-1)),
            KeyCode::Delete => Some(Operation::CursorDelete(1)),
            KeyCode::Esc => Some(Operation::GotoNormalMode),
            KeyCode::Enter => {
              Some(Operation::ConfirmExCommandAndGotoNormalMode)
            }
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

    if let Some(op) = self.get_operation(event) {
      return self.handle_op(data_access, op);
    }

    StatefulValue::CommandLineExMode(CommandLineExStateful::default())
  }

  fn handle_op(
    &self,
    data_access: StatefulDataAccess,
    op: Operation,
  ) -> StatefulValue {
    match op {
      Operation::CursorMoveBy((_, _))
      | Operation::CursorMoveUpBy(_)
      | Operation::CursorMoveDownBy(_)
      | Operation::CursorMoveLeftBy(_)
      | Operation::CursorMoveRightBy(_)
      | Operation::CursorMoveTo((_, _)) => self.cursor_move(&data_access, op),
      Operation::GotoNormalMode => self.goto_normal_mode(&data_access),
      Operation::ConfirmExCommandAndGotoNormalMode => {
        self.confirm_ex_command_and_goto_normal_mode(&data_access)
      }
      Operation::CursorInsert(text) => self.cursor_insert(&data_access, text),
      Operation::CursorDelete(n) => self.cursor_delete(&data_access, n),
      _ => unreachable!(),
    }
  }
}

impl CommandLineExStateful {
  pub fn confirm_ex_command_and_goto_normal_mode(
    &self,
    data_access: &StatefulDataAccess,
  ) -> StatefulValue {
    let cmdline_input_content = self._goto_normal_mode_impl(data_access);

    let commands = data_access.commands.clone();
    let commands = lock!(commands);

    let current_handle = tokio::runtime::Handle::current();
    match commands.parse(&cmdline_input_content) {
      Some(parsed_cmd) => {
        let jsrt_tick_dispatcher = data_access.jsrt_tick_dispatcher.clone();
        current_handle.spawn_blocking(move || {
          jsrt_tick_dispatcher
            .blocking_send(JsMessage::ExCommandReq(ExCommandReq::new(
              next_future_id(),
              parsed_cmd,
            )))
            .unwrap();
        });
      }
      None => {
        // Print error message
        let jsrt_to_master = data_access.jsrt_to_master.clone();
        let message_id = js::next_future_id();
        let e = format!("Error: invalid command {cmdline_input_content:?}");
        let e = e.to_compact_string();
        current_handle.spawn_blocking(move || {
          jsrt_to_master
            .blocking_send(MasterMessage::PrintReq(msg::PrintReq::new(
              message_id, e,
            )))
            .unwrap();
        });
      }
    }

    StatefulValue::NormalMode(super::NormalStateful::default())
  }
}

impl CommandLineExStateful {
  pub fn _goto_normal_mode_impl(
    &self,
    data_access: &StatefulDataAccess,
  ) -> CompactString {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);

    debug_assert!(tree.command_line_id().is_some());
    let cmdline_id = tree.command_line_id().unwrap();
    let cmdline = tree.command_line_mut().unwrap();

    cmdline.show_message();

    debug_assert!(cmdline.cursor_id().is_some());

    // Remove from current parent
    let cursor = match cmdline.remove_cursor().unwrap() {
      CommandLineNode::Cursor(mut cursor) => {
        cursor.set_style(&CursorStyle::SteadyBlock);
        cursor
      }
      _ => unreachable!(),
    };
    debug_assert!(cmdline.cursor_id().is_none());

    // Insert to new parent
    let current_window = tree.current_window_mut().unwrap();
    let cursor_viewport = current_window.cursor_viewport();
    trace!("before viewport:{:?}", current_window.viewport());
    trace!("before cursor_viewport:{:?}", cursor_viewport);
    let _current_window_id = current_window.id();
    let _previous_cursor = current_window.insert_cursor(cursor);
    debug_assert!(_previous_cursor.is_none());
    current_window.move_cursor_to(
      cursor_viewport.column_idx() as isize,
      cursor_viewport.row_idx() as isize,
    );

    // Clear command-line both input content and message.
    let contents = data_access.contents.clone();
    let mut contents = lock!(contents);
    let cmdline_input_content =
      contents.command_line_input().rope().to_compact_string();

    cmdline_ops::cmdline_clear_message(&mut tree, &mut contents);
    cursor_ops::cursor_clear(
      &mut tree,
      cmdline_id,
      contents.command_line_input_mut(),
    );

    let cmdline_input_content = cmdline_input_content.trim();
    tree
      .command_line_mut()
      .unwrap()
      .indicator_mut()
      .set_symbol(IndicatorSymbol::Empty);

    cmdline_input_content.to_compact_string()
  }

  pub fn goto_normal_mode(
    &self,
    data_access: &StatefulDataAccess,
  ) -> StatefulValue {
    self._goto_normal_mode_impl(data_access);

    StatefulValue::NormalMode(super::NormalStateful::default())
  }
}

impl CommandLineExStateful {
  pub fn cursor_move(
    &self,
    data_access: &StatefulDataAccess,
    op: Operation,
  ) -> StatefulValue {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);
    debug_assert!(tree.command_line_id().is_some());
    let cmdline_id = tree.command_line_id().unwrap();
    let contents = data_access.contents.clone();
    let contents = lock!(contents);

    cursor_ops::cursor_move(
      &mut tree,
      cmdline_id,
      contents.command_line_input(),
      op,
      true,
    );

    StatefulValue::CommandLineExMode(CommandLineExStateful::default())
  }
}

impl CommandLineExStateful {
  pub fn cursor_insert(
    &self,
    data_access: &StatefulDataAccess,
    payload: CursorInsertPayload,
  ) -> StatefulValue {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);
    debug_assert!(tree.command_line_id().is_some());
    let cmdline_id = tree.command_line_id().unwrap();
    let contents = data_access.contents.clone();
    let mut contents = lock!(contents);

    let payload = match payload {
      CursorInsertPayload::Text(c) => c,
      CursorInsertPayload::Tab => '\t'.to_compact_string(),
      _ => unreachable!(),
    };

    cursor_ops::cursor_insert(
      &mut tree,
      cmdline_id,
      contents.command_line_input_mut(),
      payload,
    );

    StatefulValue::CommandLineExMode(CommandLineExStateful::default())
  }
}

impl CommandLineExStateful {
  pub fn cursor_delete(
    &self,
    data_access: &StatefulDataAccess,
    n: isize,
  ) -> StatefulValue {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);
    let contents = data_access.contents.clone();
    let mut contents = lock!(contents);
    let text = contents.command_line_input_mut();

    let cmdline = tree.command_line_mut().unwrap();
    let cmdline_id = cmdline.id();
    debug_assert_eq!(cmdline.input_cursor_viewport().line_idx(), 0);
    debug_assert!(
      text
        .rope()
        .get_line(cmdline.input_cursor_viewport().line_idx())
        .is_some()
    );

    cursor_ops::cursor_delete(&mut tree, cmdline_id, text, n);

    StatefulValue::CommandLineExMode(CommandLineExStateful::default())
  }
}
