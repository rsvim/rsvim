//! The insert mode.

use crate::buf::undo;
use crate::prelude::*;
use crate::state::State;
use crate::state::StateDataAccess;
use crate::state::Stateful;
use crate::state::ops::CursorInsertPayload;
use crate::state::ops::Operation;
use crate::state::ops::cursor_ops;
use crate::ui::canvas::CursorStyle;
use crate::ui::tree::*;
use compact_str::CompactString;
use compact_str::ToCompactString;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEventKind;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The finite-state-machine for insert mode.
pub struct Insert {}

impl Insert {
  fn get_operation(&self, event: &Event) -> Option<Operation> {
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

impl Stateful for Insert {
  fn handle(&self, data_access: StateDataAccess, event: Event) -> State {
    if let Some(op) = self.get_operation(&event) {
      return self.handle_op(data_access, op);
    }

    State::Insert(Insert::default())
  }

  fn handle_op(&self, data_access: StateDataAccess, op: Operation) -> State {
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

impl Insert {
  pub fn cursor_delete(
    &self,
    data_access: &StateDataAccess,
    n: isize,
  ) -> State {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);
    let current_window = tree.current_window_mut().unwrap();
    let current_window_id = current_window.id();
    let buffer = current_window.buffer().upgrade().unwrap();
    let mut buffer = lock!(buffer);

    // Save editing change
    let absolute_delete_range = cursor_ops::cursor_absolute_delete_chars_range(
      &tree,
      current_window_id,
      buffer.text(),
      n,
    );
    if let Some(absolute_delete_range) = absolute_delete_range
      && !absolute_delete_range.is_empty()
    {
      let payload = buffer
        .text()
        .rope()
        .chars_at(absolute_delete_range.start)
        .take(absolute_delete_range.len())
        .collect::<CompactString>();

      debug_assert_ne!(n, 0);
      if n < 0 {
        if cfg!(debug_assertions) {
          let cursor_viewport =
            tree.editable_cursor_viewport(current_window_id);
          let cursor_line_idx = cursor_viewport.line_idx();
          let cursor_char_idx = cursor_viewport.char_idx();
          debug_assert_eq!(
            absolute_delete_range.end,
            buffer
              .text()
              .absolute_char_idx(cursor_line_idx, cursor_char_idx)
          );
        }
        buffer.undo_manager_mut().delete(undo::Delete {
          payload: payload.clone(),
          char_idx_before: absolute_delete_range.end - 1,
          char_idx_after: absolute_delete_range.start,
        });
      } else {
        if cfg!(debug_assertions) {
          let cursor_viewport =
            tree.editable_cursor_viewport(current_window_id);
          let cursor_line_idx = cursor_viewport.line_idx();
          let cursor_char_idx = cursor_viewport.char_idx();
          debug_assert_eq!(
            absolute_delete_range.start,
            buffer
              .text()
              .absolute_char_idx(cursor_line_idx, cursor_char_idx)
          );
        }
        buffer.undo_manager_mut().delete(undo::Delete {
          payload: payload.clone(),
          char_idx_before: absolute_delete_range.start,
          char_idx_after: absolute_delete_range.start,
        });
      };
      let _absolute_cursor_char_idx_after = absolute_delete_range.start;
      let _cursor_position_after = cursor_ops::cursor_delete(
        &mut tree,
        current_window_id,
        buffer.text_mut(),
        n,
      );
      debug_assert!(_cursor_position_after.is_some());
      debug_assert_eq!(
        buffer.text().absolute_char_idx(
          _cursor_position_after.unwrap().0,
          _cursor_position_after.unwrap().1
        ),
        _absolute_cursor_char_idx_after
      );
    }

    State::Insert(Insert::default())
  }
}

impl Insert {
  pub fn cursor_insert(
    &self,
    data_access: &StateDataAccess,
    payload: CursorInsertPayload,
  ) -> State {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);
    let current_window = tree.current_window_mut().unwrap();
    let current_window_id = current_window.id();
    let buffer = current_window.buffer().upgrade().unwrap();
    let mut buffer = lock!(buffer);

    let payload = match payload {
      CursorInsertPayload::Text(c) => c,
      CursorInsertPayload::Tab => {
        if !buffer.options().expand_tab() {
          '\t'.to_compact_string()
        } else {
          ' '
            .to_compact_string()
            .repeat(buffer.options().shift_width() as usize)
        }
      }
      CursorInsertPayload::Eol => {
        let eol = buffer.options().end_of_line();
        let eol = format!("{eol}");
        trace!("Insert eol:{eol:?}");
        eol.to_compact_string()
      }
    };

    // Save editing change
    let cursor_absolute_char_idx = cursor_ops::cursor_absolute_char_idx(
      &tree,
      current_window_id,
      buffer.text(),
    );
    buffer.undo_manager_mut().insert(undo::Insert {
      payload: payload.clone(),
      char_idx_before: cursor_absolute_char_idx,
      char_idx_after: cursor_absolute_char_idx + payload.chars().count(),
    });
    let (_cursor_line_idx_after, _cursor_char_idx_after) =
      cursor_ops::cursor_insert(
        &mut tree,
        current_window_id,
        buffer.text_mut(),
        payload.clone(),
      );
    debug_assert_eq!(
      buffer
        .text()
        .absolute_char_idx(_cursor_line_idx_after, _cursor_char_idx_after),
      cursor_ops::cursor_absolute_char_idx(
        &tree,
        current_window_id,
        buffer.text(),
      )
    );
    debug_assert_eq!(
      cursor_absolute_char_idx + payload.chars().count(),
      cursor_ops::cursor_absolute_char_idx(
        &tree,
        current_window_id,
        buffer.text(),
      )
    );

    State::Insert(Insert::default())
  }
}

impl Insert {
  pub fn goto_normal_mode(&self, data_access: &StateDataAccess) -> State {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);
    let current_window = tree.current_window_mut().unwrap();
    let current_window_id = current_window.id();
    let buffer = current_window.buffer().upgrade().unwrap();
    let mut buffer = lock!(buffer);

    // Commit editing changes
    buffer.undo_manager_mut().commit();

    let op = Operation::CursorMoveBy((-1, 0));
    cursor_ops::cursor_move(
      &mut tree,
      current_window_id,
      buffer.text(),
      op,
      false,
    );

    if cfg!(debug_assertions) {
      debug_assert!(tree.cursor_id().is_some());
      let cursor_id = tree.cursor_id().unwrap();
      debug_assert!(tree.parent_id(cursor_id).is_some());
      let parent_id = tree.parent_id(cursor_id).unwrap();
      debug_assert!(matches!(
        tree.node(parent_id).unwrap(),
        Node::WindowContent(_)
      ));
      debug_assert!(
        tree.parent_id(tree.parent_id(cursor_id).unwrap()).is_some()
      );
      let parent_parent_id = tree.parent_id(parent_id).unwrap();
      debug_assert!(tree.current_window_id().is_some());
      debug_assert_eq!(parent_parent_id, tree.current_window_id().unwrap());
    }

    tree
      .cursor_mut()
      .unwrap()
      .set_cursor_style(CursorStyle::SteadyBlock);

    State::Normal(super::Normal::default())
  }
}

impl Insert {
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
      true,
    );

    State::Insert(Insert::default())
  }
}
