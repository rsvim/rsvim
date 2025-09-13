//! The normal mode.

use crate::prelude::*;
use crate::state::StateDataAccess;
use crate::state::StateMachine;
use crate::state::Stateful;
use crate::state::ops::GotoInsertModeVariant;
use crate::state::ops::Operation;
use crate::state::ops::cursor_ops;
use crate::ui::canvas::CursorStyle;
use crate::ui::tree::*;
use crate::ui::widget::command_line::indicator::IndicatorSymbol;
use crate::ui::widget::window::WindowNode;
use compact_str::CompactString;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEventKind;

#[cfg(test)]
use crate::buf::text::Text;
#[cfg(test)]
use crate::ui::viewport::CursorViewport;
#[cfg(test)]
use crate::ui::viewport::ViewportSearchDirection;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The finite-state-machine for normal mode.
pub struct NormalStateful {}

impl NormalStateful {
  fn get_operation(&self, event: &Event) -> Option<Operation> {
    match event {
      Event::FocusGained => None,
      Event::FocusLost => None,
      Event::Key(key_event) => match key_event.kind {
        KeyEventKind::Press => {
          trace!("Event::key:{:?}", key_event);
          match key_event.code {
            KeyCode::Up | KeyCode::Char('k') => {
              Some(Operation::CursorMoveUpBy(1))
            }
            KeyCode::Down | KeyCode::Char('j') => {
              Some(Operation::CursorMoveDownBy(1))
            }
            KeyCode::Left | KeyCode::Char('h') => {
              Some(Operation::CursorMoveLeftBy(1))
            }
            KeyCode::Right | KeyCode::Char('l') => {
              Some(Operation::CursorMoveRightBy(1))
            }
            KeyCode::Home => Some(Operation::CursorMoveLeftBy(usize::MAX)),
            KeyCode::End => Some(Operation::CursorMoveRightBy(usize::MAX)),
            KeyCode::Char('i') => {
              Some(Operation::GotoInsertMode(GotoInsertModeVariant::Keep))
            }
            KeyCode::Char('a') => {
              Some(Operation::GotoInsertMode(GotoInsertModeVariant::Append))
            }
            KeyCode::Char('o') => {
              Some(Operation::GotoInsertMode(GotoInsertModeVariant::NewLine))
            }
            KeyCode::Char(':') => Some(Operation::GotoCommandLineExMode),
            // KeyCode::Char('/') => Some(Operation::GotoCommandLineSearchForwardMode),
            // KeyCode::Char('?') => Some(Operation::GotoCommandLineSearchBackwardMode),
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

impl Stateful for NormalStateful {
  fn handle(&self, data_access: StateDataAccess, event: Event) -> StateMachine {
    if let Some(op) = self.get_operation(&event) {
      return self.handle_op(data_access, op);
    }

    StateMachine::NormalMode(NormalStateful::default())
  }

  fn handle_op(
    &self,
    data_access: StateDataAccess,
    op: Operation,
  ) -> StateMachine {
    match op {
      Operation::GotoInsertMode(insert_motion) => {
        self.goto_insert_mode(&data_access, insert_motion)
      }
      Operation::GotoCommandLineExMode => {
        self.goto_command_line_ex_mode(&data_access)
      }
      // Operation::GotoCommandLineSearchForwardMode => {
      //   self.goto_command_line_search_forward_mode(&data_access)
      // }
      // Operation::GotoCommandLineSearchBackwardMode => {
      //   self.goto_command_line_search_backward_mode(&data_access)
      // }
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
  pub fn goto_command_line_ex_mode(
    &self,
    data_access: &StateDataAccess,
  ) -> StateMachine {
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

    cmdline.show_input();

    let _previous_cursor = cmdline.insert_cursor(cursor);
    debug_assert!(_previous_cursor.is_none());
    cmdline.move_cursor_to(0, 0);
    cmdline.indicator_mut().set_symbol(IndicatorSymbol::Ex);

    StateMachine::CommandLineExMode(super::CommandLineExStateful::default())
  }
}

impl NormalStateful {
  fn _goto_command_line_search_forward_mode(
    &self,
    _data_access: &StateDataAccess,
  ) -> StateMachine {
    StateMachine::CommandLineSearchForwardMode(
      super::CommandLineSearchForwardStateful::default(),
    )
  }
}

impl NormalStateful {
  fn _goto_command_line_search_backward_mode(
    &self,
    _data_access: &StateDataAccess,
  ) -> StateMachine {
    StateMachine::CommandLineSearchBackwardMode(
      super::CommandLineSearchBackwardStateful::default(),
    )
  }
}

impl NormalStateful {
  pub fn goto_insert_mode(
    &self,
    data_access: &StateDataAccess,
    insert_motion: GotoInsertModeVariant,
  ) -> StateMachine {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);

    match insert_motion {
      GotoInsertModeVariant::Keep => {}
      GotoInsertModeVariant::Append => {
        let current_window = tree.current_window_mut().unwrap();
        let current_window_id = current_window.id();
        let buffer = current_window.buffer().upgrade().unwrap();
        let buffer = lock!(buffer);
        let op = Operation::CursorMoveRightBy(1);
        cursor_ops::cursor_move(
          &mut tree,
          current_window_id,
          buffer.text(),
          op,
          true,
        );
      }
      GotoInsertModeVariant::NewLine => {
        let current_window = tree.current_window_mut().unwrap();
        let current_window_id = current_window.id();
        let buffer = current_window.buffer().upgrade().unwrap();
        let mut buffer = lock!(buffer);
        let op = Operation::CursorMoveRightBy(usize::MAX);
        cursor_ops::cursor_move(
          &mut tree,
          current_window_id,
          buffer.text(),
          op,
          true,
        );
        let eol =
          CompactString::new(format!("{}", buffer.options().end_of_line()));
        cursor_ops::cursor_insert(
          &mut tree,
          current_window_id,
          buffer.text_mut(),
          eol,
        );
      }
    };

    let current_window = tree.current_window_mut().unwrap();
    let cursor = current_window.cursor_mut().unwrap();
    cursor.set_style(&CursorStyle::SteadyBar);

    StateMachine::InsertMode(super::InsertStateful::default())
  }
}

impl NormalStateful {
  /// Cursor move in current window, with buffer scroll.
  pub fn cursor_move(
    &self,
    data_access: &StateDataAccess,
    op: Operation,
  ) -> StateMachine {
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
      false,
    );
    StateMachine::NormalMode(NormalStateful::default())
  }
}

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
  pub fn _test_raw_cursor_move(
    &self,
    data_access: &StateDataAccess,
    op: Operation,
  ) {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);

    let (buffer, viewport, cursor_viewport, current_window_id) = {
      let current_window = tree.current_window_mut().unwrap();
      let buffer = current_window.buffer().upgrade().unwrap();
      let viewport = current_window.viewport();
      let cursor_viewport = current_window.cursor_viewport();
      (buffer, viewport, cursor_viewport, current_window.id())
    };
    let buffer = lock!(buffer);

    let (target_cursor_char, target_cursor_line, _search_direction) =
      self._target_cursor_exclude_eol(&cursor_viewport, buffer.text(), op);

    let vnode =
      cursor_ops::editable_tree_node_mut(&mut tree, current_window_id);
    let new_cursor_viewport = cursor_ops::raw_cursor_viewport_move_to(
      vnode,
      &viewport,
      buffer.text(),
      Operation::CursorMoveTo((target_cursor_char, target_cursor_line)),
    );

    tree.current_window_mut().unwrap().move_cursor_to(
      new_cursor_viewport.column_idx() as isize,
      new_cursor_viewport.row_idx() as isize,
    );
  }

  #[cfg(test)]
  pub fn _test_raw_window_scroll(
    &self,
    data_access: &StateDataAccess,
    op: Operation,
  ) {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);
    let (buffer, viewport, current_window_id) = {
      let current_window = tree.current_window_mut().unwrap();
      let buffer = current_window.buffer().upgrade().unwrap();
      let viewport = current_window.viewport();
      (buffer, viewport, current_window.id())
    };
    let buffer = lock!(buffer);

    let (start_column, start_line) = cursor_ops::normalize_to_window_scroll_to(
      op,
      viewport.start_column_idx(),
      viewport.start_line_idx(),
    );

    let vnode =
      cursor_ops::editable_tree_node_mut(&mut tree, current_window_id);
    cursor_ops::raw_viewport_scroll_to(
      vnode,
      &viewport,
      buffer.text(),
      Operation::WindowScrollTo((start_column, start_line)),
    );
  }
}
