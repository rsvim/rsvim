//! The normal mode.

use crate::buf::text::Text;
use crate::prelude::*;
use crate::state::fsm::quit::QuitStateful;
use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::ops::cursor_ops;
use crate::state::ops::{GotoInsertModeVariant, Operation};
use crate::ui::canvas::CursorStyle;
use crate::ui::tree::*;
use crate::ui::widget::command_line::CommandLineIndicatorSymbol;
use crate::ui::widget::window::WindowNode;

use compact_str::CompactString;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use tracing::trace;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The finite-state-machine for normal mode.
pub struct NormalStateful {}

impl NormalStateful {
  fn get_operation(&self, event: Event) -> Option<Operation> {
    match event {
      Event::FocusGained => None,
      Event::FocusLost => None,
      Event::Key(key_event) => match key_event.kind {
        KeyEventKind::Press => {
          trace!("Event::key:{:?}", key_event);
          match key_event.code {
            KeyCode::Up | KeyCode::Char('k') => Some(Operation::CursorMoveUpBy(1)),
            KeyCode::Down | KeyCode::Char('j') => Some(Operation::CursorMoveDownBy(1)),
            KeyCode::Left | KeyCode::Char('h') => Some(Operation::CursorMoveLeftBy(1)),
            KeyCode::Right | KeyCode::Char('l') => Some(Operation::CursorMoveRightBy(1)),
            KeyCode::Home => Some(Operation::CursorMoveLeftBy(usize::MAX)),
            KeyCode::End => Some(Operation::CursorMoveRightBy(usize::MAX)),
            KeyCode::Char('i') => Some(Operation::GotoInsertMode(GotoInsertModeVariant::Keep)),
            KeyCode::Char('a') => Some(Operation::GotoInsertMode(GotoInsertModeVariant::Append)),
            KeyCode::Char('o') => Some(Operation::GotoInsertMode(GotoInsertModeVariant::NewLine)),
            KeyCode::Char(':') => Some(Operation::GotoCommandLineExMode),
            // KeyCode::Char('/') => Some(Operation::GotoCommandLineSearchForwardMode),
            // KeyCode::Char('?') => Some(Operation::GotoCommandLineSearchBackwardMode),
            KeyCode::Esc => Some(Operation::EditorQuit),
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

impl Stateful for NormalStateful {
  fn handle(&self, data_access: StatefulDataAccess) -> StatefulValue {
    let event = data_access.event.clone();

    if let Some(op) = self.get_operation(event) {
      return self.handle_op(data_access, op);
    }

    StatefulValue::NormalMode(NormalStateful::default())
  }

  fn handle_op(&self, data_access: StatefulDataAccess, op: Operation) -> StatefulValue {
    match op {
      Operation::GotoInsertMode(insert_motion) => {
        self.goto_insert_mode(&data_access, insert_motion)
      }
      Operation::GotoCommandLineExMode => self.goto_command_line_ex_mode(&data_access),
      // Operation::GotoCommandLineSearchForwardMode => {
      //   self.goto_command_line_search_forward_mode(&data_access)
      // }
      // Operation::GotoCommandLineSearchBackwardMode => {
      //   self.goto_command_line_search_backward_mode(&data_access)
      // }
      Operation::EditorQuit => self.editor_quit(&data_access),
      Operation::CursorMoveBy((_, _))
      | Operation::CursorMoveUpBy(_)
      | Operation::CursorMoveDownBy(_)
      | Operation::CursorMoveLeftBy(_)
      | Operation::CursorMoveRightBy(_)
      | Operation::CursorMoveTo((_, _)) => self.cursor_move(&data_access, op),
      _ => unreachable!(),
    }
  }
}

impl NormalStateful {
  pub fn goto_command_line_ex_mode(&self, data_access: &StatefulDataAccess) -> StatefulValue {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);

    // Remove cursor from current window
    let current_window = tree.current_window_mut().unwrap();
    debug_assert!(current_window.cursor_id().is_some());
    let cursor = match current_window.remove_cursor().unwrap() {
      WindowNode::Cursor(mut cursor) => {
        cursor.set_style(&CursorStyle::SteadyBar);
        cursor
      }
      _ => unreachable!(),
    };
    debug_assert!(current_window.cursor_id().is_none());

    // Insert to command-line
    debug_assert!(tree.command_line_mut().is_some());
    let cmdline = tree.command_line_mut().unwrap();
    let _previous_cursor = cmdline.insert_cursor(cursor);
    debug_assert!(_previous_cursor.is_none());
    cmdline.move_cursor_to(0, 0);
    cmdline
      .indicator_mut()
      .set_symbol(CommandLineIndicatorSymbol::Ex);

    StatefulValue::CommandLineExMode(super::CommandLineExStateful::default())
  }
}

impl NormalStateful {
  fn _goto_command_line_search_forward_mode(
    &self,
    _data_access: &StatefulDataAccess,
  ) -> StatefulValue {
    StatefulValue::CommandLineSearchForwardMode(super::CommandLineSearchForwardStateful::default())
  }
}

impl NormalStateful {
  fn _goto_command_line_search_backward_mode(
    &self,
    _data_access: &StatefulDataAccess,
  ) -> StatefulValue {
    StatefulValue::CommandLineSearchBackwardMode(super::CommandLineSearchBackwardStateful::default())
  }
}

impl NormalStateful {
  pub fn goto_insert_mode(
    &self,
    data_access: &StatefulDataAccess,
    insert_motion: GotoInsertModeVariant,
  ) -> StatefulValue {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);

    match insert_motion {
      GotoInsertModeVariant::Keep => {}
      GotoInsertModeVariant::Append => {
        let current_window = tree.current_window_mut().unwrap();
        let current_window_id = current_window.id();
        let buffer = current_window.buffer().upgrade().unwrap();
        let buffer = lock!(buffer);
        self._cursor_move_impl(
          &mut tree,
          current_window_id,
          buffer.text(),
          Operation::CursorMoveRightBy(1),
          true,
        );
      }
      GotoInsertModeVariant::NewLine => {
        let current_window = tree.current_window_mut().unwrap();
        let current_window_id = current_window.id();
        let buffer = current_window.buffer().upgrade().unwrap();
        let mut buffer = lock!(buffer);
        self._cursor_move_impl(
          &mut tree,
          current_window_id,
          buffer.text(),
          Operation::CursorMoveRightBy(usize::MAX),
          true,
        );
        let eol = CompactString::new(format!("{}", buffer.options().end_of_line()));
        cursor_ops::cursor_insert(&mut tree, current_window_id, buffer.text_mut(), eol);
      }
    };

    let current_window = tree.current_window_mut().unwrap();
    let cursor = current_window.cursor_mut().unwrap();
    cursor.set_style(&CursorStyle::SteadyBar);

    StatefulValue::InsertMode(super::InsertStateful::default())
  }
}

impl NormalStateful {
  /// Cursor move in current window, with buffer scroll.
  pub fn cursor_move(&self, data_access: &StatefulDataAccess, op: Operation) -> StatefulValue {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);
    let current_window = tree.current_window_mut().unwrap();
    let current_window_id = current_window.id();
    let buffer = current_window.buffer().upgrade().unwrap();
    let buffer = lock!(buffer);

    self._cursor_move_impl(&mut tree, current_window_id, buffer.text(), op, false)
  }

  fn _cursor_move_impl(
    &self,
    tree: &mut Tree,
    current_window_id: TreeNodeId,
    text: &Text,
    op: Operation,
    include_eol: bool,
  ) -> StatefulValue {
    cursor_ops::cursor_move(tree, current_window_id, text, op, include_eol);

    StatefulValue::NormalMode(NormalStateful::default())
  }
}

#[cfg(test)]
use crate::ui::viewport::{CursorViewport, ViewportSearchDirection};

impl NormalStateful {
  #[cfg(test)]
  // Returns `(target_cursor_char, target_cursor_line, viewport_search_direction)`.
  pub fn _target_cursor_exclude_eol(
    &self,
    cursor_viewport: &CursorViewport,
    text: &Text,
    op: Operation,
  ) -> (usize, usize, ViewportSearchDirection) {
    use crate::state::ops::cursor_ops::CursorMoveDirection;

    let (target_cursor_char, target_cursor_line, move_direction) =
      cursor_ops::normalize_to_cursor_move_to_exclude_eol(
        text,
        op,
        cursor_viewport.char_idx(),
        cursor_viewport.line_idx(),
      );

    let search_direction = match move_direction {
      CursorMoveDirection::Up => ViewportSearchDirection::Up,
      CursorMoveDirection::Down => ViewportSearchDirection::Down,
      CursorMoveDirection::Left => ViewportSearchDirection::Left,
      CursorMoveDirection::Right => ViewportSearchDirection::Right,
    };
    (target_cursor_char, target_cursor_line, search_direction)
  }

  #[cfg(test)]
  pub fn _test_raw_cursor_move(&self, data_access: &StatefulDataAccess, op: Operation) {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);
    let current_window = tree.current_window_mut().unwrap();
    let buffer = current_window.buffer().upgrade().unwrap();
    let buffer = lock!(buffer);
    let viewport = current_window.viewport();
    let cursor_viewport = current_window.cursor_viewport();

    let (target_cursor_char, target_cursor_line, _search_direction) =
      self._target_cursor_exclude_eol(&cursor_viewport, buffer.text(), op);

    let maybe_new_cursor_viewport = cursor_ops::raw_cursor_viewport_move_to(
      &viewport,
      &cursor_viewport,
      buffer.text(),
      Operation::CursorMoveTo((target_cursor_char, target_cursor_line)),
    );

    if let Some(new_cursor_viewport) = maybe_new_cursor_viewport {
      current_window.set_cursor_viewport(new_cursor_viewport.clone());
      current_window.move_cursor_to(
        new_cursor_viewport.column_idx() as isize,
        new_cursor_viewport.row_idx() as isize,
      );
    }
  }

  #[cfg(test)]
  pub fn _test_raw_window_scroll(&self, data_access: &StatefulDataAccess, op: Operation) {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);
    let current_window = tree.current_window_mut().unwrap();
    let buffer = current_window.buffer().upgrade().unwrap();
    let buffer = lock!(buffer);
    let viewport = current_window.viewport();

    let (start_column, start_line) = cursor_ops::normalize_to_window_scroll_to(
      op,
      viewport.start_column_idx(),
      viewport.start_line_idx(),
    );
    let maybe_new_viewport_arc = cursor_ops::raw_viewport_scroll_to(
      &viewport,
      current_window.actual_shape(),
      current_window.options(),
      buffer.text(),
      Operation::WindowScrollTo((start_column, start_line)),
    );
    if let Some(new_viewport_arc) = maybe_new_viewport_arc.clone() {
      current_window.set_viewport(new_viewport_arc.clone());
    }
  }

  pub fn editor_quit(&self, _data_access: &StatefulDataAccess) -> StatefulValue {
    StatefulValue::QuitState(QuitStateful::default())
  }
}
