//! The insert mode.

use crate::buf::Buffer;
use crate::lock;
use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::ops::Operation;
use crate::state::ops::cursor_ops::{self, CursorMoveDirection};
use crate::ui::canvas::CursorStyle;
use crate::ui::tree::*;
use crate::ui::widget::window::{CursorViewport, ViewportArc, ViewportSearchAnchorDirection};

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
            KeyCode::Up => {
              return self.cursor_move(&data_access, Operation::CursorMoveUpBy(1));
            }
            KeyCode::Down => {
              return self.cursor_move(&data_access, Operation::CursorMoveDownBy(1));
            }
            KeyCode::Left => {
              return self.cursor_move(&data_access, Operation::CursorMoveLeftBy(1));
            }
            KeyCode::Right => {
              return self.cursor_move(&data_access, Operation::CursorMoveRightBy(1));
            }
            KeyCode::Home => {
              return self.cursor_move(&data_access, Operation::CursorMoveLeftBy(usize::MAX));
            }
            KeyCode::End => {
              return self.cursor_move(&data_access, Operation::CursorMoveRightBy(usize::MAX));
            }
            // KeyCode::Char('i') => {
            //   return self.goto_insert_mode(&data_access, Operation::GotoInsertMode);
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

impl InsertStateful {
  fn cursor_move(&self, data_access: &StatefulDataAccess, op: Operation) -> StatefulValue {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);

    if let Some(current_window_id) = tree.current_window_id() {
      if let Some(TreeNode::Window(current_window)) = tree.node_mut(current_window_id) {
        let buffer = current_window.buffer().upgrade().unwrap();
        let buffer = lock!(buffer);
        let viewport = current_window.viewport();
        let cursor_viewport = current_window.cursor_viewport();
        let cursor_viewport = lock!(cursor_viewport);

        // Only move cursor when it is different from current cursor.
        if let Some((target_cursor_char, target_cursor_line, search_direction)) =
          self._target_cursor_include_empty_eol(&cursor_viewport, &buffer, op)
        {
          let new_viewport: Option<ViewportArc> = {
            let viewport = lock!(viewport);
            let (start_line, start_column) = viewport.search_anchor(
              search_direction,
              &buffer,
              current_window.actual_shape(),
              current_window.options(),
              target_cursor_line,
              target_cursor_char,
            );

            // First try window scroll.
            if start_line != viewport.start_line_idx()
              || start_column != viewport.start_column_idx()
            {
              let new_viewport = cursor_ops::window_scroll(
                &viewport,
                current_window,
                &buffer,
                Operation::WindowScrollTo((start_column, start_line)),
              );
              if let Some(new_viewport_arc) = new_viewport.clone() {
                current_window.set_viewport(new_viewport_arc.clone());
              }
              new_viewport
            } else {
              None
            }
          };

          // Then try cursor move.
          {
            let current_viewport = new_viewport.unwrap_or(viewport);
            let current_viewport = lock!(current_viewport);

            let new_cursor_viewport = cursor_ops::cursor_move(
              &current_viewport,
              &cursor_viewport,
              &buffer,
              Operation::CursorMoveTo((target_cursor_char, target_cursor_line)),
            );

            if let Some(new_cursor_viewport) = new_cursor_viewport {
              current_window.set_cursor_viewport(new_cursor_viewport.clone());
              let cursor_id = tree.cursor_id().unwrap();
              let new_cursor_viewport = lock!(new_cursor_viewport);
              tree.bounded_move_to(
                cursor_id,
                new_cursor_viewport.column_idx() as isize,
                new_cursor_viewport.row_idx() as isize,
              );
            }
          }
        }
      } else {
        unreachable!()
      }
    } else {
      unreachable!()
    }

    StatefulValue::InsertMode(InsertStateful::default())
  }

  // Returns `(target_cursor_char, target_cursor_line, viewport_search_direction)`.
  fn _target_cursor_include_empty_eol(
    &self,
    cursor_viewport: &CursorViewport,
    buffer: &Buffer,
    op: Operation,
  ) -> Option<(usize, usize, ViewportSearchAnchorDirection)> {
    let (target_cursor_char, target_cursor_line, move_direction) =
      cursor_ops::normalize_as_cursor_move_to(
        op,
        cursor_viewport.char_idx(),
        cursor_viewport.line_idx(),
      );
    let target_cursor_line = std::cmp::min(
      target_cursor_line,
      buffer.get_rope().len_lines().saturating_sub(1),
    );
    let target_cursor_char = match buffer.last_char_on_line(target_cursor_line) {
      Some(last_visible_char) => std::cmp::min(target_cursor_char, last_visible_char),
      None => target_cursor_char,
    };
    if target_cursor_char != cursor_viewport.char_idx()
      || target_cursor_line != cursor_viewport.line_idx()
    {
      let search_direction = match move_direction {
        CursorMoveDirection::Up => ViewportSearchAnchorDirection::Up,
        CursorMoveDirection::Down => ViewportSearchAnchorDirection::Down,
        CursorMoveDirection::Left => ViewportSearchAnchorDirection::Left,
        CursorMoveDirection::Right => ViewportSearchAnchorDirection::Right,
      };
      Some((target_cursor_char, target_cursor_line, search_direction))
    } else {
      None
    }
  }
}
