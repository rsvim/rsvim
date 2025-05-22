//! The insert mode.

use crate::lock;
use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::ops::Operation;
use crate::ui::canvas::CursorStyle;
use crate::ui::tree::*;
// use crate::ui::widget::window::{
//   CursorViewport, CursorViewportArc, Viewport, ViewportArc, ViewportSearchAnchorDirection, Window,
// };

use crossterm::event::{Event, KeyCode, KeyEventKind};
use tracing::trace;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The insert editing mode.
pub struct InsertStateful {}

impl Stateful for InsertStateful {
  fn handle(&self, data_access: StatefulDataAccess) -> StatefulValue {
    let event = data_access.event.clone();

    match event {
      Event::FocusGained => {}
      Event::FocusLost => {}
      Event::Key(key_event) => match key_event.kind {
        KeyEventKind::Press => {
          trace!("Event::key:{:?}", key_event);
          match key_event.code {
            // KeyCode::Up | KeyCode::Char('k') => {
            //   return self.cursor_move(&data_access, Command::CursorMoveUpBy(1));
            // }
            // KeyCode::Down | KeyCode::Char('j') => {
            //   return self.cursor_move(&data_access, Command::CursorMoveDownBy(1));
            // }
            // KeyCode::Left | KeyCode::Char('h') => {
            //   return self.cursor_move(&data_access, Command::CursorMoveLeftBy(1));
            // }
            // KeyCode::Right | KeyCode::Char('l') => {
            //   return self.cursor_move(&data_access, Command::CursorMoveRightBy(1));
            // }
            // KeyCode::Home => {
            //   return self.cursor_move(&data_access, Command::CursorMoveLeftBy(usize::MAX));
            // }
            // KeyCode::End => {
            //   return self.cursor_move(&data_access, Command::CursorMoveRightBy(usize::MAX));
            // }
            // KeyCode::Char('i') => {
            //   return self.goto_insert_mode(&data_access, Command::GotoInsertMode);
            // }
            KeyCode::Esc => {
              return self.goto_normal_mode(&data_access, Operation::GotoNormalMode);
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

    StatefulValue::InsertMode(InsertStateful::default())
  }
}

impl InsertStateful {
  fn goto_normal_mode(
    &self,
    data_access: &StatefulDataAccess,
    _command: Operation,
  ) -> StatefulValue {
    debug_assert!(matches!(_command, Operation::GotoNormalMode));

    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);
    let cursor_id = tree.cursor_id().unwrap();
    if let Some(TreeNode::Cursor(cursor)) = tree.node_mut(cursor_id) {
      cursor.set_style(&CursorStyle::SteadyBlock);
    } else {
      unreachable!()
    }

    StatefulValue::NormalMode(super::NormalStateful::default())
  }
}
