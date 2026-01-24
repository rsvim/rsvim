//! The normal mode.

use crate::buf::undo;
use crate::prelude::*;
use crate::state::State;
use crate::state::StateDataAccess;
use crate::state::Stateful;
use crate::state::ops::GotoInsertModeVariant;
use crate::state::ops::Operation;
use crate::state::ops::cursor_ops;
use crate::ui::canvas::CursorStyle;
use crate::ui::tree::*;
use crate::ui::widget::cmdline::indicator::CmdlineIndicatorSymbol;
use compact_str::CompactString;
use compact_str::ToCompactString;
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
pub struct Normal {}

impl Normal {
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
            KeyCode::Char(':') => Some(Operation::GotoCmdlineExMode),
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

impl Stateful for Normal {
  fn handle(&self, data_access: StateDataAccess, event: Event) -> State {
    if let Some(op) = self.get_operation(&event) {
      return self.handle_op(data_access, op);
    }

    State::Normal(Normal::default())
  }

  fn handle_op(&self, data_access: StateDataAccess, op: Operation) -> State {
    match op {
      Operation::GotoInsertMode(insert_motion) => {
        self.goto_insert_mode(&data_access, insert_motion)
      }
      Operation::GotoCmdlineExMode => self.goto_cmdline_ex_mode(&data_access),
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

impl Normal {
  pub fn goto_cmdline_ex_mode(&self, data_access: &StateDataAccess) -> State {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);

    if cfg!(debug_assertions) {
      debug_assert!(tree.cmdline_id().is_some());
      debug_assert!(tree.current_window_id().is_some());
      let cursor_id = tree.cursor_id();
      debug_assert!(cursor_id.is_some());
      let cursor_id = cursor_id.unwrap();
      let cursor_parent_id = tree.parent_id(cursor_id);
      debug_assert!(cursor_parent_id.is_some());
      let cursor_parent_id = cursor_parent_id.unwrap();
      let cursor_parent = tree.node(cursor_parent_id);
      debug_assert!(cursor_parent.is_some());
      let cursor_parent = cursor_parent.unwrap();
      debug_assert!(matches!(cursor_parent, Node::WindowContent(_)));
    }

    // Show input/hide message, and update layouts/shapes.
    tree.cmdline_show_input().unwrap();

    let cmdline_id = tree.cmdline_id().unwrap();
    let _old_widget_id = cursor_ops::cursor_jump(&mut tree, cmdline_id);

    if cfg!(debug_assertions) {
      debug_assert!(tree.current_window_id().is_some());
      debug_assert_eq!(_old_widget_id, tree.current_window_id());
      let cursor_id = tree.cursor_id();
      debug_assert!(cursor_id.is_some());
      let cursor_id = cursor_id.unwrap();
      let cursor_parent_id = tree.parent_id(cursor_id);
      debug_assert!(cursor_parent_id.is_some());
      let cursor_parent_id = cursor_parent_id.unwrap();
      let cursor_parent = tree.node(cursor_parent_id);
      debug_assert!(cursor_parent.is_some());
      let cursor_parent = cursor_parent.unwrap();
      debug_assert!(matches!(cursor_parent, Node::CmdlineInput(_)));
    }

    let cursor_viewport = tree.editable_cursor_viewport(cmdline_id);
    tree
      .cursor_move_position_to(
        cursor_viewport.column_idx() as isize,
        cursor_viewport.row_idx() as isize,
      )
      .unwrap();
    tree
      .cursor_mut()
      .unwrap()
      .set_cursor_style(CursorStyle::SteadyBar);
    tree.set_cmdline_indicator_symbol(CmdlineIndicatorSymbol::Colon);

    if cfg!(debug_assertions) {
      let contents = data_access.contents.clone();
      let contents = lock!(contents);
      let cmdline_input_content =
        contents.cmdline_input().rope().to_compact_string();
      debug_assert!(cmdline_input_content.is_empty());
    }

    State::CmdlineEx(super::CmdlineEx::default())
  }
}

impl Normal {
  fn _goto_command_line_search_forward_mode(
    &self,
    _data_access: &StateDataAccess,
  ) -> State {
    State::CmdlineSearchForward(super::CmdlineSearchForward::default())
  }
}

impl Normal {
  fn _goto_command_line_search_backward_mode(
    &self,
    _data_access: &StateDataAccess,
  ) -> State {
    State::CmdlineSearchBackward(super::CmdlineSearchBackward::default())
  }
}

impl Normal {
  pub fn goto_insert_mode(
    &self,
    data_access: &StateDataAccess,
    insert_motion: GotoInsertModeVariant,
  ) -> State {
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

        // Save editing change
        let cursor_absolute_char_idx = cursor_ops::cursor_absolute_char_idx(
          &tree,
          current_window_id,
          buffer.text(),
        );
        buffer.undo_manager_mut().insert(undo::Insert {
          payload: eol.clone(),
          char_idx_before: cursor_absolute_char_idx,
          char_idx_after: cursor_absolute_char_idx + eol.chars().count(),
        });
        cursor_ops::cursor_insert(
          &mut tree,
          current_window_id,
          buffer.text_mut(),
          eol,
        );
      }
    };

    tree
      .cursor_mut()
      .unwrap()
      .set_cursor_style(CursorStyle::SteadyBar);

    State::Insert(super::Insert::default())
  }
}

impl Normal {
  /// Cursor move in current window, with buffer scroll.
  pub fn cursor_move(
    &self,
    data_access: &StateDataAccess,
    op: Operation,
  ) -> State {
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
    State::Normal(Normal::default())
  }
}

impl Normal {
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

    let new_cursor_viewport = cursor_ops::raw_cursor_viewport_move_to(
      &mut tree,
      current_window_id,
      &viewport,
      buffer.text(),
      Operation::CursorMoveTo((target_cursor_char, target_cursor_line)),
    );

    tree
      .cursor_move_position_to(
        new_cursor_viewport.column_idx() as isize,
        new_cursor_viewport.row_idx() as isize,
      )
      .unwrap();
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

    cursor_ops::raw_viewport_scroll_to(
      &mut tree,
      current_window_id,
      &viewport,
      buffer.text(),
      Operation::WindowScrollTo((start_column, start_line)),
    );
  }
}
