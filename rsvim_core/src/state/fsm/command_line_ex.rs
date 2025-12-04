//! The command-line ex mode.

use crate::msg;
use crate::msg::ExCommandReq;
use crate::msg::JsMessage;
use crate::prelude::*;
use crate::state::StateDataAccess;
use crate::state::StateMachine;
use crate::state::Stateful;
use crate::state::ops::CursorInsertPayload;
use crate::state::ops::Operation;
use crate::state::ops::cmdline_ops;
use crate::state::ops::cursor_ops;
use crate::ui::canvas::CursorStyle;
use crate::ui::tree::*;
use crate::ui::widget::command_line::indicator::IndicatorSymbol;
use compact_str::CompactString;
use compact_str::ToCompactString;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEventKind;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The command-line ex mode.
pub struct CommandLineExStateful {}

impl CommandLineExStateful {
  fn get_operation(&self, event: &Event) -> Option<Operation> {
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
      Event::Paste(_paste_string) => None,
      Event::Resize(_columns, _rows) => None,
    }
  }
}

impl Stateful for CommandLineExStateful {
  fn handle(&self, data_access: StateDataAccess, event: Event) -> StateMachine {
    if let Some(op) = self.get_operation(&event) {
      return self.handle_op(data_access, op);
    }

    StateMachine::CommandLineExMode(CommandLineExStateful::default())
  }

  fn handle_op(
    &self,
    data_access: StateDataAccess,
    op: Operation,
  ) -> StateMachine {
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
    data_access: &StateDataAccess,
  ) -> StateMachine {
    let cmdline_input_content = self._goto_normal_mode_impl(data_access);

    let tree = data_access.tree.clone();
    let tree = lock!(tree);
    let current_window = tree.current_window().unwrap();
    let current_win_id = current_window.id();
    let buffer = current_window.buffer().upgrade().unwrap();
    let buffer = lock!(buffer);
    let current_buf_id = buffer.id();

    msg::send_to_jsrt(
      data_access.jsrt_forwarder_tx.clone(),
      JsMessage::ExCommandReq(ExCommandReq {
        payload: cmdline_input_content,
        current_buf_id,
        current_win_id,
      }),
    );

    StateMachine::NormalMode(super::NormalStateful::default())
  }
}

impl CommandLineExStateful {
  pub fn _goto_normal_mode_impl(
    &self,
    data_access: &StateDataAccess,
  ) -> CompactString {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);

    // Jump cursor from command-line to "current window".
    debug_assert!(tree.command_line_id().is_some());
    let current_window_id = tree.current_window_id().unwrap();
    let cmdline_id = tree.command_line_id().unwrap();

    tree
      .cursor_mut()
      .unwrap()
      .set_cursor_style(CursorStyle::SteadyBlock);
    let _old_widget_id = tree.jump_cursor_to(current_window_id);
    debug_assert!(_old_widget_id.is_some());
    debug_assert_eq!(_old_widget_id, Some(cmdline_id));

    // Command-line show message, hide input/indicator.
    tree.cmdline_show_message().unwrap();

    // Move cursor to previous position.
    let cursor_viewport = tree.current_window().unwrap().cursor_viewport();
    tree.move_cursor_to(
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
      .cmdline_indicator_mut()
      .unwrap()
      .set_symbol(IndicatorSymbol::Empty);

    cmdline_input_content.to_compact_string()
  }

  pub fn goto_normal_mode(
    &self,
    data_access: &StateDataAccess,
  ) -> StateMachine {
    self._goto_normal_mode_impl(data_access);

    StateMachine::NormalMode(super::NormalStateful::default())
  }
}

impl CommandLineExStateful {
  pub fn cursor_move(
    &self,
    data_access: &StateDataAccess,
    op: Operation,
  ) -> StateMachine {
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

    StateMachine::CommandLineExMode(CommandLineExStateful::default())
  }
}

impl CommandLineExStateful {
  pub fn cursor_insert(
    &self,
    data_access: &StateDataAccess,
    payload: CursorInsertPayload,
  ) -> StateMachine {
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

    StateMachine::CommandLineExMode(CommandLineExStateful::default())
  }
}

impl CommandLineExStateful {
  pub fn cursor_delete(
    &self,
    data_access: &StateDataAccess,
    n: isize,
  ) -> StateMachine {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);
    let contents = data_access.contents.clone();
    let mut contents = lock!(contents);
    let text = contents.command_line_input_mut();

    let cmdline_id = tree.command_line_id().unwrap();
    cursor_ops::cursor_delete(&mut tree, cmdline_id, text, n);

    StateMachine::CommandLineExMode(CommandLineExStateful::default())
  }
}
