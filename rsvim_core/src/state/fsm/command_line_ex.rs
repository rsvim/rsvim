//! The command-line mode, ex-command variant.

use crate::lock;
use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::ops::Operation;
use crate::ui::canvas::CursorStyle;
use crate::ui::tree::*;

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
  fn goto_normal_mode(&self, data_access: &StatefulDataAccess) -> StatefulValue {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);

    debug_assert!(tree.current_window_id().is_some());
    let current_window_id = tree.current_window_id().unwrap();
    debug_assert!(tree.node(current_window_id).is_some());
    debug_assert!(matches!(
      tree.node(current_window_id).unwrap(),
      TreeNode::Window(_)
    ));

    debug_assert!(tree.cursor_id().is_some());
    let cursor_id = tree.cursor_id().unwrap();

    // Remove from current parent
    debug_assert!(tree.command_line_id().is_some());
    let cmdline_id = tree.command_line_id().unwrap();
    debug_assert!(tree.parent_id(cursor_id).is_some());
    debug_assert_eq!(tree.parent_id(cursor_id).unwrap(), cmdline_id);
    debug_assert!(tree.node(cmdline_id).is_some());
    debug_assert!(matches!(
      tree.node(cmdline_id).unwrap(),
      TreeNode::CommandLine(_)
    ));
    let cursor_node = tree.remove(cursor_id);
    debug_assert!(cursor_node.is_some());
    let cursor_node = cursor_node.unwrap();
    debug_assert!(matches!(cursor_node, TreeNode::Cursor(_)));
    debug_assert!(!tree.children_ids(cmdline_id).contains(&cursor_id));
    match cursor_node {
      TreeNode::Cursor(mut cursor) => cursor.set_style(&CursorStyle::SteadyBlock),
      _ => unreachable!(),
    }

    // Insert to new parent
    let _inserted = tree.bounded_insert(current_window_id, cursor_node);
    debug_assert!(_inserted.is_none());

    // Clear command-line contents.
    let contents = data_access.contents.clone();
    let mut contents = lock!(contents);
    contents.command_line_content_mut().rope_mut().remove(0..);
    contents.command_line_content_mut().clear_cached_lines();

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
