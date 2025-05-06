//! The normal mode.

use crate::buf::Buffer;
use crate::lock;
use crate::state::command::Command;
use crate::state::fsm::quit::QuitStateful;
use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::ui::tree::*;
use crate::ui::widget::window::{CursorViewport, Viewport};

use crossterm::event::{Event, KeyCode, KeyEventKind};
use tracing::trace;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The normal editing mode.
pub struct NormalStateful {}

fn adjust_cursor_char_idx_on_vertical_motion(
  buffer: &Buffer,
  cursor_line_idx: usize,
  cursor_char_idx: usize,
  line_idx: usize,
) -> usize {
  let cursor_col_idx = buffer.width_before(cursor_line_idx, cursor_char_idx);
  let char_idx = match buffer.char_after(line_idx, cursor_col_idx) {
    Some(char_idx) => char_idx,
    None => {
      debug_assert!(buffer.get_rope().get_line(line_idx).is_some());
      debug_assert!(buffer.get_rope().line(line_idx).len_chars() > 0);
      buffer.last_visible_char_on_line(line_idx).unwrap()
    }
  };
  trace!(
    "cursor_line_idx:{},cursor_col_idx:{},line_idx:{},char_idx:{}",
    cursor_line_idx, cursor_col_idx, line_idx, char_idx
  );
  char_idx
}

#[allow(dead_code)]
// Normalize CursorMove* commands to CursorMoveBy((x,y))
fn normalize_as_cursor_move_by(
  command: Command,
  cursor_char_idx: usize,
  cursor_line_idx: usize,
) -> (isize, isize) {
  match command {
    Command::CursorMoveLeftBy(n) => (-(n as isize), 0),
    Command::CursorMoveRightBy(n) => (n as isize, 0),
    Command::CursorMoveUpBy(n) => (0, -(n as isize)),
    Command::CursorMoveDownBy(n) => (0, n as isize),
    Command::CursorMoveTo((x, y)) => {
      let x = (x as isize) - (cursor_char_idx as isize);
      let y = (y as isize) - (cursor_line_idx as isize);
      (x, y)
    }
    Command::CursorMoveBy((x, y)) => (x, y),
    _ => unreachable!(),
  }
}

#[allow(dead_code)]
// Normalize CursorMove* commands to CursorMoveTo((x,y))
fn normalize_as_cursor_move_to(
  command: Command,
  cursor_char_idx: usize,
  cursor_line_idx: usize,
) -> (usize, usize) {
  match command {
    Command::CursorMoveLeftBy(n) => {
      let x = cursor_char_idx.saturating_sub(n);
      (x, cursor_line_idx)
    }
    Command::CursorMoveRightBy(n) => {
      let x = cursor_char_idx.saturating_add(n);
      (x, cursor_line_idx)
    } // (n as isize, 0),
    Command::CursorMoveUpBy(n) => {
      let y = cursor_line_idx.saturating_sub(n);
      (cursor_char_idx, y)
    } // (0, -(n as isize)),
    Command::CursorMoveDownBy(n) => {
      let y = cursor_line_idx.saturating_add(n);
      (cursor_char_idx, y)
    } // (0, n as isize),
    Command::CursorMoveBy((x, y)) => {
      let x = cursor_char_idx.saturating_add_signed(x);
      let y = cursor_line_idx.saturating_add_signed(y);
      (x, y)
    }
    Command::CursorMoveTo((x, y)) => (x, y),
    _ => unreachable!(),
  }
}

// Normalize WindowScroll* commands to WindowScrollBy((x,y))
fn normalize_as_window_scroll_by(
  command: Command,
  viewport_start_column_idx: usize,
  viewport_start_line_idx: usize,
) -> (isize, isize) {
  match command {
    Command::WindowScrollLeftBy(n) => (-(n as isize), 0),
    Command::WindowScrollRightBy(n) => (n as isize, 0),
    Command::WindowScrollUpBy(n) => (0, -(n as isize)),
    Command::WindowScrollDownBy(n) => (0, n as isize),
    Command::WindowScrollTo((x, y)) => {
      let x = (x as isize) - (viewport_start_column_idx as isize);
      let y = (y as isize) - (viewport_start_line_idx as isize);
      (x, y)
    }
    Command::WindowScrollBy((x, y)) => (x, y),
    _ => unreachable!(),
  }
}

#[allow(dead_code)]
// Normalize WindowScroll* commands to WindowScrollTo((x,y))
fn normalize_as_window_scroll_to(
  command: Command,
  viewport_start_column_idx: usize,
  viewport_start_line_idx: usize,
) -> (usize, usize) {
  match command {
    Command::WindowScrollLeftBy(n) => {
      let x = viewport_start_column_idx.saturating_sub(n);
      (x, 0)
    }
    Command::WindowScrollRightBy(n) => {
      let x = viewport_start_column_idx.saturating_add(n);
      (x, 0)
    } // (n as isize, 0),
    Command::WindowScrollUpBy(n) => {
      let y = viewport_start_line_idx.saturating_sub(n);
      (0, y)
    } // (0, -(n as isize)),
    Command::WindowScrollDownBy(n) => {
      let y = viewport_start_line_idx.saturating_add(n);
      (0, y)
    } // (0, n as isize),
    Command::WindowScrollBy((x, y)) => {
      let x = viewport_start_column_idx.saturating_add_signed(x);
      let y = viewport_start_line_idx.saturating_add_signed(y);
      (x, y)
    }
    Command::WindowScrollTo((x, y)) => (x, y),
    _ => unreachable!(),
  }
}

impl Stateful for NormalStateful {
  fn handle(&self, data_access: StatefulDataAccess) -> StatefulValue {
    let event = data_access.event.clone();

    match event {
      Event::FocusGained => {}
      Event::FocusLost => {}
      Event::Key(key_event) => match key_event.kind {
        KeyEventKind::Press => {
          match key_event.code {
            KeyCode::Up | KeyCode::Char('k') => {
              return self.cursor_move(&data_access, Command::CursorMoveUpBy(1));
            }
            KeyCode::Down | KeyCode::Char('j') => {
              return self.cursor_move(&data_access, Command::CursorMoveDownBy(1));
            }
            KeyCode::Left | KeyCode::Char('h') => {
              return self.cursor_move(&data_access, Command::CursorMoveLeftBy(1));
            }
            KeyCode::Right | KeyCode::Char('l') => {
              return self.cursor_move(&data_access, Command::CursorMoveRightBy(1));
            }
            KeyCode::Esc => {
              // quit loop
              return self.quit(&data_access, Command::EditorQuit);
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

    // if event == Event::Key(KeyCode::Char('c').into()) {
    //   println!("Cursor position: {:?}\r", crossterm::cursor::position());
    // }

    // // quit loop
    // if event == Event::Key(KeyCode::Esc.into()) {
    //   // println!("ESC: {:?}\r", crossterm::cursor::position());
    //   return StateMachine::QuitState(QuitStateful::default());
    // }

    StatefulValue::NormalMode(NormalStateful::default())
  }
}

impl NormalStateful {
  /// Cursor move in current window, with buffer scroll.
  fn cursor_move(&self, data_access: &StatefulDataAccess, command: Command) -> StatefulValue {
    // Get (window_scroll_to, cursor_move_to).
    let scrolls_and_moves = self._expected_move_and_scroll(data_access, command);

    // First try window scroll.
    let scrolls = scrolls_and_moves.0;
    if let Some((scroll_to_x, scroll_to_y)) = scrolls {
      self._window_scroll(
        data_access,
        Command::WindowScrollTo((scroll_to_x, scroll_to_y)),
      );
    }

    // Then try cursor move.
    let moves = scrolls_and_moves.1;
    if let Some((move_to_x, move_to_y)) = moves {
      self._cursor_move(data_access, Command::CursorMoveTo((move_to_x, move_to_y)));
    }

    StatefulValue::NormalMode(NormalStateful::default())
  }

  #[allow(clippy::type_complexity)]
  // Returns `(Command::WindowScrollTo((x,y)), Command::CursorMoveTo((x,y)))`.
  fn _expected_move_and_scroll(
    &self,
    data_access: &StatefulDataAccess,
    command: Command,
  ) -> (Option<(usize, usize)>, Option<(usize, usize)>) {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);
    if let Some(current_window_id) = tree.current_window_id() {
      if let Some(TreeNode::Window(current_window)) = tree.node_mut(current_window_id) {
        let buffer = current_window.buffer().upgrade().unwrap();
        let buffer = lock!(buffer);
        let viewport = current_window.viewport();
        let viewport = lock!(viewport);
        let cursor_viewport = current_window.cursor_viewport();
        let cursor_viewport = lock!(cursor_viewport);
        let (to_x, to_y) = normalize_as_cursor_move_to(
          command,
          cursor_viewport.char_idx(),
          cursor_viewport.line_idx(),
        );

        // If goes out of window bottom:
        //
        // Condition-1
        // - Cursor is moving down.
        // - The target cursor line > window's bottom line.
        // - Window's bottom line < buffer last line.
        //
        // Condition-2
        // - Cursor is moving down.
        // - The target cursor line = window's bottom line.
        // - Window's bottom line is not fully rendered, i.e. window's first char > 0 or window's
        //   last char.
        let goes_out_of_bottom = {
          if let Some((&last_line_idx, _last_line_viewport)) = viewport.lines().last_key_value() {
            let cond1 = to_y > cursor_viewport.line_idx()
              && to_y > last_line_idx
              && last_line_idx < buffer.get_rope().len_lines().saturating_sub(1);

            let cond2 = to_y > cursor_viewport.line_idx() && to_y == last_line_idx && {
              let head_not_show = self._line_head_not_show(&viewport, last_line_idx);
              let tail_not_show = self._line_tail_not_show(&viewport, &buffer, last_line_idx);
              head_not_show || tail_not_show
            };

            cond1 || cond2
          } else {
            false
          }
        };

        // If goes out of window top:
        //
        // Condition-1
        // - Cursor is moving up.
        // - The target cursor line < window's top line.
        // - Window's top line > buffer's first line, i.e. 0.
        //
        // Condition-2
        // - Cursor is moving up.
        // - The target cursor line = window's top line.
        // - Window's top line is not fully rendered, i.e. window's first char > 0 or window's last
        //   char.
        let goes_out_of_top = {
          if let Some((&first_line_idx, _first_line_viewport)) = viewport.lines().first_key_value()
          {
            let cond1 =
              to_y < cursor_viewport.line_idx() && to_y < first_line_idx && first_line_idx > 0;

            let cond2 = to_y < cursor_viewport.line_idx() && to_y == first_line_idx && {
              let head_not_show = self._line_head_not_show(&viewport, first_line_idx);
              let tail_not_show = self._line_tail_not_show(&viewport, &buffer, first_line_idx);
              head_not_show || tail_not_show
            };

            cond1 || cond2
          } else {
            false
          }
        };

        // If goes out of window right border:
        //
        // Group-1
        // - Cursor is moving right.
        // - The target cursor char > window's last char of the line.
        // - Window's last char of the line < buffer's last visible char of the line.
        let goes_out_of_right = {
          if let Some(line_viewport) = viewport.lines().get(&cursor_viewport.line_idx()) {
            debug_assert!(
              buffer
                .get_rope()
                .get_line(cursor_viewport.line_idx())
                .is_some()
            );
            let bufline_len_chars = buffer
              .get_rope()
              .line(cursor_viewport.line_idx())
              .len_chars();
            let rows = line_viewport.rows();
            if let Some((_last_row_idx, last_row_viewport)) = rows.last_key_value() {
              to_x > cursor_viewport.char_idx()
                && to_x > last_row_viewport.end_char_idx().saturating_sub(1)
                && last_row_viewport.end_char_idx() < bufline_len_chars
            } else {
              false
            }
          } else {
            false
          }
        };

        // If goes out of window left border:
        //
        // Group-1
        // - Cursor is moving left.
        // - The target cursor char < window's first char of the line.
        // - Windows's first char of the line > buffer's first char of the line, i.e. 0.
        let goes_out_of_left = {
          if let Some(line_viewport) = viewport.lines().get(&cursor_viewport.line_idx()) {
            debug_assert!(
              buffer
                .get_rope()
                .get_line(cursor_viewport.line_idx())
                .is_some()
            );
            let rows = line_viewport.rows();
            if let Some((_first_row_idx, first_row_viewport)) = rows.first_key_value() {
              to_x < cursor_viewport.char_idx()
                && to_x < first_row_viewport.start_char_idx()
                && first_row_viewport.start_char_idx() > 0
            } else {
              false
            }
          } else {
            false
          }
        };

        if goes_out_of_bottom || goes_out_of_top {
          (
            Some((viewport.start_column_idx(), to_y)),
            Some((to_x, to_y)),
          )
        } else if goes_out_of_right || goes_out_of_left {
          (Some((to_x, viewport.start_line_idx())), Some((to_x, to_y)))
        } else {
          (None, Some((to_x, to_y)))
        }
      } else {
        (None, None)
      }
    } else {
      (None, None)
    }
  }

  fn _line_head_not_show(&self, viewport: &Viewport, line_idx: usize) -> bool {
    debug_assert!(viewport.lines().contains_key(&line_idx));
    let line_viewport = viewport.lines().get(&line_idx).unwrap();
    let rows = line_viewport.rows();
    debug_assert!(rows.first_key_value().is_some());
    let (_first_row_idx, first_row_viewport) = rows.first_key_value().unwrap();
    first_row_viewport.start_char_idx() > 0
  }

  fn _line_tail_not_show(&self, viewport: &Viewport, buffer: &Buffer, line_idx: usize) -> bool {
    debug_assert!(buffer.get_rope().get_line(line_idx).is_some());
    let bufline_last_visible_char = buffer
      .last_visible_char_on_line(line_idx)
      .unwrap_or(0_usize);

    debug_assert!(viewport.lines().contains_key(&line_idx));
    let line_viewport = viewport.lines().get(&line_idx).unwrap();
    let rows = line_viewport.rows();
    debug_assert!(rows.last_key_value().is_some());
    let (_last_row_idx, last_row_viewport) = rows.last_key_value().unwrap();
    last_row_viewport.end_char_idx().saturating_sub(1) < bufline_last_visible_char
  }

  /// Cursor move in current window.
  /// NOTE: This will not scroll the buffer if cursor reaches the window border.
  fn _cursor_move(&self, data_access: &StatefulDataAccess, command: Command) {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);
    if let Some(current_window_id) = tree.current_window_id() {
      if let Some(TreeNode::Window(current_window)) = tree.node_mut(current_window_id) {
        let buffer = current_window.buffer().upgrade().unwrap();
        let buffer = lock!(buffer);
        let viewport = current_window.viewport();
        let viewport = lock!(viewport);
        let cursor_viewport = current_window.cursor_viewport();
        let cursor_viewport = lock!(cursor_viewport);

        let (x, y) = normalize_as_cursor_move_by(
          command,
          cursor_viewport.char_idx(),
          cursor_viewport.line_idx(),
        );

        let cursor_move_result = self._cursor_move_by(&viewport, &cursor_viewport, &buffer, x, y);

        trace!("cursor_move_result:{:?}", cursor_move_result);
        if let Some((line_idx, char_idx)) = cursor_move_result {
          let moved_cursor_viewport =
            CursorViewport::from_position(&viewport, &buffer, line_idx, char_idx);
          current_window.set_cursor_viewport(CursorViewport::to_arc(moved_cursor_viewport));

          let cursor_id = tree.cursor_id().unwrap();
          tree.bounded_move_to(
            cursor_id,
            moved_cursor_viewport.column_idx() as isize,
            moved_cursor_viewport.row_idx() as isize,
          );
          trace!("(after) cursor node position:{:?}", moved_cursor_viewport);
        }
        // Or, just do nothing, stay at where you are
      }
    }
  }

  // Returns the `line_idx`/`char_idx` for new cursor position.
  // NOTE: `x` is chars count, `y` is lines count.
  fn _cursor_move_by(
    &self,
    viewport: &Viewport,
    cursor_viewport: &CursorViewport,
    buffer: &Buffer,
    x: isize,
    y: isize,
  ) -> Option<(usize, usize)> {
    let cursor_line_idx = cursor_viewport.line_idx();
    let cursor_char_idx = cursor_viewport.char_idx();
    let line_idx =
      self._bounded_cursor_move_y_by(viewport, cursor_line_idx, cursor_char_idx, buffer, y);

    // If `line_idx` doesn't exist, or line is empty.
    match buffer.get_rope().get_line(line_idx) {
      Some(line) => {
        if line.len_chars() == 0 {
          return Some((line_idx, 0_usize));
        }
      }
      None => return None,
    }

    let char_idx =
      adjust_cursor_char_idx_on_vertical_motion(buffer, cursor_line_idx, cursor_char_idx, line_idx);
    let char_idx = self._bounded_cursor_move_x_by(viewport, line_idx, char_idx, buffer, x);

    Some((line_idx, char_idx))
  }

  // // Returns the `line_idx`/`char_idx` for new cursor position.
  // // NOTE: `y` is lines count.
  // fn _cursor_move_y_by(
  //   &self,
  //   viewport: &Viewport,
  //   cursor_viewport: &CursorViewport,
  //   buffer: &Buffer,
  //   y: isize,
  // ) -> Option<(usize, usize)> {
  //   let cursor_line_idx = cursor_viewport.line_idx();
  //   let cursor_char_idx = cursor_viewport.char_idx();
  //
  //   let line_idx = if y < 0 {
  //     let n = -y as usize;
  //     cursor_line_idx.saturating_sub(n)
  //   } else {
  //     let n = y as usize;
  //     let expected = cursor_line_idx.saturating_add(n);
  //     let last_line_idx = viewport.end_line_idx().saturating_sub(1);
  //     trace!(
  //       "cursor_line_idx:{:?},expected:{:?},last_line_idx:{:?}",
  //       cursor_line_idx, expected, last_line_idx
  //     );
  //     std::cmp::min(expected, last_line_idx)
  //   };
  //   trace!(
  //     "cursor:{}/{},line_idx:{}",
  //     cursor_line_idx, cursor_char_idx, line_idx
  //   );
  //
  //   // If line index doesn't change, early return.
  //   if line_idx == cursor_line_idx {
  //     return None;
  //   }
  //
  //   match buffer.get_rope().get_line(line_idx) {
  //     Some(line) => {
  //       trace!("line.len_chars:{}", line.len_chars());
  //       if line.len_chars() == 0 {
  //         return None;
  //       }
  //     }
  //     None => {
  //       trace!("get_line not found:{}", line_idx);
  //       return None;
  //     }
  //   }
  //   let char_idx =
  //     adjust_cursor_char_idx_on_vertical_motion(buffer, cursor_line_idx, cursor_char_idx, line_idx);
  //   Some((line_idx, char_idx))
  // }

  // The `y` is lines count.
  fn _bounded_cursor_move_y_by(
    &self,
    viewport: &Viewport,
    cursor_line_idx: usize,
    _cursor_char_idx: usize,
    _buffer: &Buffer,
    y: isize,
  ) -> usize {
    if y < 0 {
      let n = -y as usize;
      cursor_line_idx.saturating_sub(n)
    } else {
      let n = y as usize;
      let expected_line_idx = cursor_line_idx.saturating_add(n);
      let last_line_idx = viewport.end_line_idx().saturating_sub(1);
      trace!(
        "base_line_idx:{:?},expected:{:?},last_line_idx:{:?}",
        cursor_line_idx, expected_line_idx, last_line_idx
      );
      std::cmp::min(expected_line_idx, last_line_idx)
    }
  }

  // // Returns the `line_idx`/`char_idx` for new cursor position.
  // // NOTE: `x` is chars count.
  // fn _cursor_move_x_by(
  //   &self,
  //   viewport: &Viewport,
  //   cursor_viewport: &CursorViewport,
  //   buffer: &Buffer,
  //   x: isize,
  // ) -> Option<(usize, usize)> {
  //   let cursor_line_idx = cursor_viewport.line_idx();
  //   let cursor_char_idx = cursor_viewport.char_idx();
  //
  //   match buffer.get_rope().get_line(cursor_line_idx) {
  //     Some(line) => {
  //       if line.len_chars() == 0 {
  //         return None;
  //       }
  //     }
  //     None => return None,
  //   }
  //
  //   let char_idx = if x < 0 {
  //     let n = -x as usize;
  //     cursor_char_idx.saturating_sub(n)
  //   } else {
  //     let n = x as usize;
  //     let expected = cursor_char_idx.saturating_add(n);
  //     let upper_bounded = {
  //       debug_assert!(viewport.lines().contains_key(&cursor_line_idx));
  //       let line_viewport = viewport.lines().get(&cursor_line_idx).unwrap();
  //       let (_last_row_idx, last_row_viewport) = line_viewport.rows().last_key_value().unwrap();
  //       let last_char_on_row = last_row_viewport.end_char_idx() - 1;
  //       trace!(
  //         "cursor_char_idx:{}, expected:{}, last_row_viewport:{:?}, last_char_on_row:{}",
  //         cursor_char_idx, expected, last_row_viewport, last_char_on_row
  //       );
  //       buffer
  //         .last_visible_char_on_line_since(cursor_line_idx, last_char_on_row)
  //         .unwrap()
  //     };
  //     std::cmp::min(expected, upper_bounded)
  //   };
  //
  //   Some((cursor_line_idx, char_idx))
  // }

  // The `x` is chars count.
  fn _bounded_cursor_move_x_by(
    &self,
    viewport: &Viewport,
    cursor_line_idx: usize,
    cursor_char_idx: usize,
    buffer: &Buffer,
    x: isize,
  ) -> usize {
    if x < 0 {
      let n = -x as usize;
      cursor_char_idx.saturating_sub(n)
    } else {
      let n = x as usize;
      let expected = cursor_char_idx.saturating_add(n);
      let upper_bounded = {
        debug_assert!(viewport.lines().contains_key(&cursor_line_idx));
        let line_viewport = viewport.lines().get(&cursor_line_idx).unwrap();
        let (_last_row_idx, last_row_viewport) = line_viewport.rows().last_key_value().unwrap();
        let last_char_on_row = last_row_viewport.end_char_idx() - 1;
        trace!(
          "cursor_char_idx:{}, expected:{}, last_row_viewport:{:?}, last_char_on_row:{}",
          cursor_char_idx, expected, last_row_viewport, last_char_on_row
        );
        buffer
          .last_visible_char_on_line_since(cursor_line_idx, last_char_on_row)
          .unwrap()
      };
      std::cmp::min(expected, upper_bounded)
    }
  }

  /// Window scrolls buffer content.
  fn _window_scroll(&self, data_access: &StatefulDataAccess, command: Command) {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);

    if let Some(current_window_id) = tree.current_window_id() {
      if let Some(TreeNode::Window(current_window)) = tree.node_mut(current_window_id) {
        let viewport = current_window.viewport();
        let viewport = lock!(viewport);
        let buffer = current_window.buffer().upgrade().unwrap();
        let buffer = lock!(buffer);

        // TODO: The `(x,y)` in command is lines/chars, while the returned `(x,y)` is
        // lines/columns. Since the lines/chars are user facing command, user moves their cursor by
        // lines/chars instead of terminal rows/columns/cells.
        let (x, y) = normalize_as_window_scroll_by(
          command,
          viewport.start_column_idx(),
          viewport.start_line_idx(),
        );

        let window_scroll_result = self._window_scroll_by(&viewport, &buffer, x, y);

        if let Some((start_line_idx, start_column_idx)) = window_scroll_result {
          // Sync the viewport
          let window_actual_shape = current_window.window_content().actual_shape();
          let window_local_options = current_window.options();
          current_window.set_viewport(Viewport::to_arc(Viewport::view(
            &buffer,
            window_actual_shape,
            window_local_options,
            start_line_idx,
            start_column_idx,
          )));
        }
        // Or, just do nothing, keep the old viewport.
      }
    }
  }

  /// Returns the `start_line_idx`/`start_column_idx` for new window viewport.
  /// NOTE: `x` is the columns count, `y` is the lines count.
  fn _window_scroll_by(
    &self,
    viewport: &Viewport,
    buffer: &Buffer,
    x: isize,
    y: isize,
  ) -> Option<(usize, usize)> {
    let start_line_idx = viewport.start_line_idx();
    let end_line_idx = viewport.end_line_idx();
    let start_column_idx = viewport.start_column_idx();
    let buffer_len_lines = buffer.get_rope().len_lines();
    debug_assert!(end_line_idx <= buffer_len_lines);

    let mut line_idx = self._bounded_window_scroll_y_by(start_line_idx, buffer, y);

    // If viewport wants to scroll down (i.e. y > 0), and viewport already shows that last line in
    // the buffer, then cannot scroll down anymore, just still keep the old `line_idx`.
    if y > 0 && end_line_idx == buffer_len_lines {
      line_idx = start_line_idx;
    }

    let column_idx = self._bounded_window_scroll_x_by(start_column_idx, viewport, buffer, x);

    // If the newly `start_line_idx`/`start_column_idx` is the same with current viewport, then
    // there's no need to scroll anymore.
    if line_idx == start_line_idx && column_idx == start_column_idx {
      return None;
    }

    Some((line_idx, column_idx))
  }

  // /// Returns the `start_line_idx`/`start_column_idx` for new window viewport.
  // /// NOTE: `y` is the lines count.
  // fn _window_scroll_y_by(
  //   &self,
  //   viewport: &Viewport,
  //   buffer: &Buffer,
  //   y: isize,
  // ) -> Option<(usize, usize)> {
  //   let start_line_idx = viewport.start_line_idx();
  //   let end_line_idx = viewport.end_line_idx();
  //   let start_column_idx = viewport.start_column_idx();
  //   let buffer_len_lines = buffer.get_rope().len_lines();
  //
  //   let line_idx = if y < 0 {
  //     let n = -y as usize;
  //     start_line_idx.saturating_sub(n)
  //   } else {
  //     let n = y as usize;
  //     // Viewport already shows the last line of buffer, cannot scroll down anymore.
  //     debug_assert!(end_line_idx <= buffer_len_lines);
  //     if end_line_idx == buffer_len_lines {
  //       return None;
  //     }
  //
  //     // Expected start line cannot go out of buffer, i.e. it cannot be greater than the last
  //     // line.
  //     let expected_start_line = std::cmp::min(
  //       start_line_idx.saturating_add(n),
  //       buffer_len_lines.saturating_sub(1),
  //     );
  //
  //     // If the expected (after scrolled) start line index is current start line index, then don't
  //     // scroll.
  //     if expected_start_line == start_line_idx {
  //       return None;
  //     }
  //
  //     trace!(
  //       "start_line_idx:{:?},end_line_idx:{:?},expected_start_line:{:?}",
  //       start_line_idx, end_line_idx, expected_start_line
  //     );
  //     expected_start_line
  //   };
  //
  //   Some((line_idx, start_column_idx))
  // }

  /// The `y` is the lines count.
  fn _bounded_window_scroll_y_by(&self, start_line_idx: usize, buffer: &Buffer, y: isize) -> usize {
    let buffer_len_lines = buffer.get_rope().len_lines();

    if y < 0 {
      let n = -y as usize;
      start_line_idx.saturating_sub(n)
    } else {
      let n = y as usize;

      // Expected start line cannot go out of buffer, i.e. it cannot be greater than the last
      // line.
      let expected_start_line = std::cmp::min(
        start_line_idx.saturating_add(n),
        buffer_len_lines.saturating_sub(1),
      );

      trace!(
        "start_line_idx:{:?},expected_start_line:{:?}",
        start_line_idx, expected_start_line
      );
      expected_start_line
    }
  }

  // Calculate how many columns that each line (in current viewport) need to scroll until
  // their own line's end. This is the upper bound of the actual columns that could
  // scroll.
  fn _bounded_window_scroll_x_max_scrolls(&self, viewport: &Viewport, buffer: &Buffer) -> usize {
    let mut max_scrolls = 0_usize;
    for (line_idx, line_viewport) in viewport.lines().iter() {
      trace!("line_idx:{},line_viewport:{:?}", line_idx, line_viewport);
      debug_assert!(!line_viewport.rows().is_empty());
      let (_last_row_idx, last_row_viewport) = line_viewport.rows().last_key_value().unwrap();
      trace!(
        "_last_row_idx:{},last_row_viewport:{:?}",
        _last_row_idx, last_row_viewport
      );
      debug_assert!(buffer.get_rope().get_line(*line_idx).is_some());
      // If `last_row_viewport` is empty, i.e. the `end_char_idx == start_char_idx`, the scrolls is 0.
      if last_row_viewport.end_char_idx() > last_row_viewport.start_char_idx() {
        let max_scrolls_on_line = match buffer.last_visible_char_on_line(*line_idx) {
          Some(last_visible_c) => {
            let last_visible_col = buffer.width_at(*line_idx, last_visible_c);
            let last_col_on_row = buffer.width_at(
              *line_idx,
              last_row_viewport.end_char_idx().saturating_sub(1),
            );
            let column_difference = last_visible_col.saturating_sub(last_col_on_row);
            trace!(
              "last_visible_c:{},last_row_viewport.end_char_idx:{},last_visible_col:{},last_col_on_row:{},column_difference:{}",
              last_visible_c,
              last_row_viewport.end_char_idx(),
              last_visible_col,
              last_col_on_row,
              column_difference
            );
            column_difference
          }
          None => 0_usize,
        };
        trace!("result:{}", max_scrolls_on_line);
        max_scrolls = std::cmp::max(max_scrolls, max_scrolls_on_line);
      }
    }
    max_scrolls
  }

  // /// Returns the `start_line_idx`/`start_column_idx` for new window viewport.
  // /// NOTE: `x` is the columns count (not chars).
  // fn _window_scroll_x_by(
  //   &self,
  //   viewport: &Viewport,
  //   buffer: &Buffer,
  //   x: isize,
  // ) -> Option<(usize, usize)> {
  //   let start_line_idx = viewport.start_line_idx();
  //   let end_line_idx = viewport.end_line_idx();
  //   let start_column_idx = viewport.start_column_idx();
  //
  //   if end_line_idx == start_line_idx {
  //     return None;
  //   }
  //
  //   debug_assert!(end_line_idx > start_line_idx);
  //   debug_assert!(viewport.lines().contains_key(&start_line_idx));
  //
  //   let start_col = if x < 0 {
  //     let n = -x as usize;
  //     start_column_idx.saturating_sub(n)
  //   } else {
  //     let n = x as usize;
  //     let expected = start_column_idx.saturating_add(n);
  //     let max_scrolls = self._window_scroll_x_max_scrolls(viewport, buffer);
  //     let upper_bounded = start_column_idx.saturating_add(max_scrolls);
  //     trace!(
  //       "max_scrolls:{},upper_bounded:{},expected:{}",
  //       max_scrolls, upper_bounded, expected
  //     );
  //     std::cmp::min(expected, upper_bounded)
  //   };
  //
  //   if start_col == start_column_idx {
  //     return None;
  //   }
  //
  //   Some((start_line_idx, start_col))
  // }

  /// The `x` is the columns count (not chars).
  fn _bounded_window_scroll_x_by(
    &self,
    start_column_idx: usize,
    viewport: &Viewport,
    buffer: &Buffer,
    x: isize,
  ) -> usize {
    if x < 0 {
      let n = -x as usize;
      start_column_idx.saturating_sub(n)
    } else {
      let n = x as usize;
      let expected = start_column_idx.saturating_add(n);
      let max_scrolls = self._bounded_window_scroll_x_max_scrolls(viewport, buffer);
      let upper_bounded = start_column_idx.saturating_add(max_scrolls);
      trace!(
        "max_scrolls:{},upper_bounded:{},expected:{}",
        max_scrolls, upper_bounded, expected
      );
      std::cmp::min(expected, upper_bounded)
    }
  }

  fn quit(&self, _data_access: &StatefulDataAccess, _command: Command) -> StatefulValue {
    StatefulValue::QuitState(QuitStateful::default())
  }
}

// spellchecker:off
#[cfg(test)]
#[allow(unused_imports)]
mod tests_util {
  use super::*;

  use crate::buf::{BufferArc, BufferLocalOptionsBuilder, BuffersManagerArc};
  use crate::lock;
  use crate::prelude::*;
  use crate::state::{State, StateArc};
  use crate::test::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::test::log::init as test_log_init;
  use crate::test::tree::make_tree_with_buffers;
  use crate::ui::tree::TreeArc;
  use crate::ui::widget::window::{Viewport, WindowLocalOptions, WindowLocalOptionsBuilder};

  use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
  use std::collections::BTreeMap;
  use tracing::info;

  pub fn make_tree(
    terminal_size: U16Size,
    window_local_opts: WindowLocalOptions,
    lines: Vec<&str>,
  ) -> (TreeArc, StateArc, BuffersManagerArc, BufferArc) {
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let buf = make_buffer_from_lines(terminal_size.height(), buf_opts, lines);
    let bufs = make_buffers_manager(buf_opts, vec![buf.clone()]);
    let tree = make_tree_with_buffers(terminal_size, window_local_opts, bufs.clone());
    let state = State::to_arc(State::default());
    (tree, state, bufs, buf)
  }

  pub fn get_viewport(tree: TreeArc) -> Viewport {
    let tree = lock!(tree);
    let current_window_id = tree.current_window_id().unwrap();
    let current_window_node = tree.node(current_window_id).unwrap();
    assert!(matches!(current_window_node, TreeNode::Window(_)));
    match current_window_node {
      TreeNode::Window(current_window) => {
        let viewport = current_window.viewport();
        let viewport = lock!(viewport);
        viewport.clone()
      }
      _ => unreachable!(),
    }
  }

  pub fn get_cursor_viewport(tree: TreeArc) -> CursorViewport {
    let tree = lock!(tree);
    let current_window_id = tree.current_window_id().unwrap();
    let current_window_node = tree.node(current_window_id).unwrap();
    assert!(matches!(current_window_node, TreeNode::Window(_)));
    match current_window_node {
      TreeNode::Window(current_window) => {
        let cursor_viewport = current_window.cursor_viewport();
        let cursor_viewport = lock!(cursor_viewport);
        *cursor_viewport
      }
      _ => unreachable!(),
    }
  }

  #[allow(clippy::too_many_arguments)]
  pub fn assert_viewport_scroll(
    buffer: BufferArc,
    actual: &Viewport,
    expect: &Vec<&str>,
    expect_start_line: usize,
    expect_end_line: usize,
    expect_start_fills: &BTreeMap<usize, usize>,
    expect_end_fills: &BTreeMap<usize, usize>,
  ) {
    info!(
      "actual start_line/end_line:{:?}/{:?}",
      actual.start_line_idx(),
      actual.end_line_idx()
    );
    info!(
      "expect start_line/end_line:{:?}/{:?}",
      expect_start_line, expect_end_line
    );
    for (k, v) in actual.lines().iter() {
      info!("actual line[{:?}]: {:?}", k, v);
    }
    for (i, e) in expect.iter().enumerate() {
      info!("expect line[{}]:{:?}", i, e);
    }
    assert_eq!(expect_start_fills.len(), expect_end_fills.len());
    for (k, start_v) in expect_start_fills.iter() {
      let end_v = expect_end_fills.get(k).unwrap();
      info!(
        "expect start_fills/end_fills line[{}]:{:?}/{:?}",
        k, start_v, end_v
      );
    }

    assert_eq!(actual.start_line_idx(), expect_start_line);
    assert_eq!(actual.end_line_idx(), expect_end_line);
    if actual.lines().is_empty() {
      assert!(actual.end_line_idx() <= actual.start_line_idx());
    } else {
      let (first_line_idx, _first_line_viewport) = actual.lines().first_key_value().unwrap();
      let (last_line_idx, _last_line_viewport) = actual.lines().last_key_value().unwrap();
      assert_eq!(*first_line_idx, actual.start_line_idx());
      assert_eq!(*last_line_idx, actual.end_line_idx() - 1);
    }
    assert_eq!(
      actual.end_line_idx() - actual.start_line_idx(),
      actual.lines().len()
    );
    assert_eq!(
      actual.end_line_idx() - actual.start_line_idx(),
      expect_start_fills.len()
    );
    assert_eq!(
      actual.end_line_idx() - actual.start_line_idx(),
      expect_end_fills.len()
    );

    let buffer = lock!(buffer);
    let buflines = buffer
      .get_rope()
      .get_lines_at(actual.start_line_idx())
      .unwrap();
    let total_lines = expect_end_line - expect_start_line;

    for (l, line) in buflines.enumerate() {
      if l >= total_lines {
        break;
      }
      let actual_line_idx = l + expect_start_line;
      let line_viewport = actual.lines().get(&actual_line_idx).unwrap();

      info!(
        "l-{:?}, actual_line_idx:{}, line_viewport:{:?}",
        l, actual_line_idx, line_viewport
      );
      info!(
        "start_filled_cols expect:{:?}, actual:{}",
        expect_start_fills.get(&actual_line_idx),
        line_viewport.start_filled_cols()
      );
      assert_eq!(
        line_viewport.start_filled_cols(),
        *expect_start_fills.get(&actual_line_idx).unwrap()
      );
      info!(
        "end_filled_cols expect:{:?}, actual:{}",
        expect_end_fills.get(&actual_line_idx),
        line_viewport.end_filled_cols()
      );
      assert_eq!(
        line_viewport.end_filled_cols(),
        *expect_end_fills.get(&actual_line_idx).unwrap()
      );

      let rows = &line_viewport.rows();
      for (r, row) in rows.iter() {
        info!("row-index-{:?}, row:{:?}", r, row);

        if r > rows.first_key_value().unwrap().0 {
          let prev_r = r - 1;
          let prev_row = rows.get(&prev_r).unwrap();
          info!(
            "row-{:?}, current[{}]:{:?}, previous[{}]:{:?}",
            r, r, row, prev_r, prev_row
          );
        }
        if r < rows.last_key_value().unwrap().0 {
          let next_r = r + 1;
          let next_row = rows.get(&next_r).unwrap();
          info!(
            "row-{:?}, current[{}]:{:?}, next[{}]:{:?}",
            r, r, row, next_r, next_row
          );
        }

        let mut payload = String::new();
        for c_idx in row.start_char_idx()..row.end_char_idx() {
          let c = line.get_char(c_idx).unwrap();
          payload.push(c);
        }
        info!(
          "row-{:?}, payload actual:{:?}, expect:{:?}",
          r, payload, expect[*r as usize]
        );
        assert_eq!(payload, expect[*r as usize]);
      }
    }
  }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests_cursor_move_y_by {
  use super::tests_util::*;
  use super::*;

  use crate::buf::BufferLocalOptionsBuilder;
  use crate::prelude::*;
  use crate::state::State;
  use crate::test::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::test::log::init as test_log_init;
  use crate::test::tree::make_tree_with_buffers;
  use crate::ui::widget::window::WindowLocalOptionsBuilder;

  use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

  #[test]
  fn nowrap1() {
    test_log_init();

    let (tree, state, bufs, _buf) = make_tree(
      U16Size::new(10, 10),
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      vec![],
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful_machine = NormalStateful::default();
    stateful_machine._cursor_move(&data_access, Command::CursorMoveUpBy(1));

    let tree = data_access.tree.clone();
    let actual = get_cursor_viewport(tree);
    assert_eq!(actual.line_idx(), 0);
    assert_eq!(actual.char_idx(), 0);
  }

  #[test]
  fn nowrap2() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let (tree, state, bufs, _buf) = make_tree(
      U16Size::new(10, 10),
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      lines,
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful_machine = NormalStateful::default();
    stateful_machine._cursor_move(&data_access, Command::CursorMoveUpBy(1));

    let tree = data_access.tree.clone();
    let actual = get_cursor_viewport(tree);
    assert_eq!(actual.line_idx(), 0);
    assert_eq!(actual.char_idx(), 0);
  }

  #[test]
  fn nowrap3() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let (tree, state, bufs, _buf) = make_tree(
      U16Size::new(10, 10),
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      lines,
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._cursor_move(&data_access, Command::CursorMoveDownBy(3));

    let tree = data_access.tree.clone();
    let actual1 = get_cursor_viewport(tree);
    assert_eq!(actual1.line_idx(), 3);
    assert_eq!(actual1.char_idx(), 0);

    stateful._cursor_move(&data_access, Command::CursorMoveUpBy(1));

    let tree = data_access.tree.clone();
    let actual2 = get_cursor_viewport(tree);
    assert_eq!(actual2.line_idx(), 2);
    assert_eq!(actual2.char_idx(), 0);
  }

  #[test]
  fn nowrap4() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let (tree, state, bufs, _buf) = make_tree(
      U16Size::new(10, 10),
      WindowLocalOptionsBuilder::default()
        .wrap(true)
        .build()
        .unwrap(),
      lines,
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._cursor_move(&data_access, Command::CursorMoveDownBy(2));

    let tree = data_access.tree.clone();
    let actual1 = get_cursor_viewport(tree);
    assert_eq!(actual1.line_idx(), 2);
    assert_eq!(actual1.char_idx(), 0);

    stateful._cursor_move(&data_access, Command::CursorMoveUpBy(1));

    let tree = data_access.tree.clone();
    let actual2 = get_cursor_viewport(tree);
    assert_eq!(actual2.line_idx(), 1);
    assert_eq!(actual2.char_idx(), 0);
  }

  #[test]
  fn nowrap5() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let lines = vec![];
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let buf = make_buffer_from_lines(terminal_size.height(), buf_opts, lines);
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let tree = make_tree_with_buffers(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      bufs.clone(),
    );
    let state = State::to_arc(State::default());
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful_machine = NormalStateful::default();
    stateful_machine._cursor_move(&data_access, Command::CursorMoveDownBy(1));

    let tree = data_access.tree.clone();
    let actual = get_cursor_viewport(tree);
    assert_eq!(actual.line_idx(), 0);
    assert_eq!(actual.char_idx(), 0);
  }

  #[test]
  fn wrap1() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let (tree, state, bufs, _buf) = make_tree(
      U16Size::new(10, 10),
      WindowLocalOptionsBuilder::default()
        .wrap(true)
        .build()
        .unwrap(),
      lines,
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._cursor_move(&data_access, Command::CursorMoveDownBy(10));

    let tree = data_access.tree.clone();
    let actual1 = get_cursor_viewport(tree);
    assert_eq!(actual1.line_idx(), 2);
    assert_eq!(actual1.char_idx(), 0);

    stateful._cursor_move(&data_access, Command::CursorMoveUpBy(1));

    let tree = data_access.tree.clone();
    let actual2 = get_cursor_viewport(tree);
    assert_eq!(actual2.line_idx(), 1);
    assert_eq!(actual2.char_idx(), 0);
  }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests_cursor_move_x_by {
  use super::tests_util::*;
  use super::*;

  use crate::buf::BufferLocalOptionsBuilder;
  use crate::prelude::*;
  use crate::state::State;
  use crate::test::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::test::log::init as test_log_init;
  use crate::test::tree::make_tree_with_buffers;
  use crate::ui::widget::window::WindowLocalOptionsBuilder;

  use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

  #[test]
  fn nowrap1() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let lines = vec![];
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let buf = make_buffer_from_lines(terminal_size.height(), buf_opts, lines);
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let tree = make_tree_with_buffers(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      bufs.clone(),
    );
    let state = State::to_arc(State::default());
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._cursor_move(&data_access, Command::CursorMoveRightBy(1));

    let tree = data_access.tree.clone();
    let actual = get_cursor_viewport(tree);
    assert_eq!(actual.line_idx(), 0);
    assert_eq!(actual.char_idx(), 0);
  }

  #[test]
  fn nowrap2() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let buf = make_buffer_from_lines(terminal_size.height(), buf_opts, lines);
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let tree = make_tree_with_buffers(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      bufs.clone(),
    );
    let state = State::to_arc(State::default());
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._cursor_move(&data_access, Command::CursorMoveRightBy(1));

    let tree = data_access.tree.clone();
    let actual = get_cursor_viewport(tree);
    assert_eq!(actual.line_idx(), 0);
    assert_eq!(actual.char_idx(), 1);
  }

  #[test]
  fn nowrap3() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let buf = make_buffer_from_lines(terminal_size.height(), buf_opts, lines);
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let tree = make_tree_with_buffers(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      bufs.clone(),
    );
    let state = State::to_arc(State::default());
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._cursor_move(&data_access, Command::CursorMoveRightBy(20));

    let tree = data_access.tree.clone();
    let actual = get_cursor_viewport(tree);
    assert_eq!(actual.line_idx(), 0);
    assert_eq!(actual.char_idx(), 9);
  }

  #[test]
  fn nowrap4() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let buf = make_buffer_from_lines(terminal_size.height(), buf_opts, lines);
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let tree = make_tree_with_buffers(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      bufs.clone(),
    );
    let state = State::to_arc(State::default());
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._cursor_move(&data_access, Command::CursorMoveRightBy(5));

    let tree = data_access.tree.clone();
    let actual1 = get_cursor_viewport(tree);
    assert_eq!(actual1.line_idx(), 0);
    assert_eq!(actual1.char_idx(), 5);

    stateful._cursor_move(&data_access, Command::CursorMoveLeftBy(3));

    let tree = data_access.tree.clone();
    let actual2 = get_cursor_viewport(tree);
    assert_eq!(actual2.line_idx(), 0);
    assert_eq!(actual2.char_idx(), 2);
  }

  #[test]
  fn nowrap5() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let buf = make_buffer_from_lines(terminal_size.height(), buf_opts, lines);
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let tree = make_tree_with_buffers(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      bufs.clone(),
    );
    let state = State::to_arc(State::default());
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._cursor_move(&data_access, Command::CursorMoveRightBy(5));

    let tree = data_access.tree.clone();
    let actual = get_cursor_viewport(tree);
    assert_eq!(actual.line_idx(), 0);
    assert_eq!(actual.char_idx(), 5);

    for i in (0..=4).rev() {
      stateful._cursor_move(&data_access, Command::CursorMoveLeftBy(1));

      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree);
      assert_eq!(actual.line_idx(), 0);
      assert_eq!(actual.char_idx(), i);
    }
  }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests_cursor_move_by {
  use super::tests_util::*;
  use super::*;

  use crate::buf::BufferLocalOptionsBuilder;
  use crate::prelude::*;
  use crate::state::State;
  use crate::test::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::test::log::init as test_log_init;
  use crate::test::tree::make_tree_with_buffers;
  use crate::ui::widget::window::WindowLocalOptionsBuilder;

  use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

  #[test]
  fn nowrap1() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let buf = make_buffer_from_lines(terminal_size.height(), buf_opts, lines);
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let tree = make_tree_with_buffers(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      bufs.clone(),
    );
    let state = State::to_arc(State::default());
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    // Step-1
    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._cursor_move(&data_access, Command::CursorMoveBy((5, 0)));

    let tree = data_access.tree.clone();
    let actual1 = get_cursor_viewport(tree);
    assert_eq!(actual1.line_idx(), 0);
    assert_eq!(actual1.char_idx(), 5);

    // Step-2
    stateful._cursor_move(&data_access, Command::CursorMoveBy((0, 1)));
    let tree = data_access.tree.clone();
    let actual2 = get_cursor_viewport(tree);
    assert_eq!(actual2.line_idx(), 1);
    assert_eq!(actual2.char_idx(), 5);

    // Step-3
    stateful._cursor_move(&data_access, Command::CursorMoveBy((-3, 0)));
    let tree = data_access.tree.clone();
    let actual3 = get_cursor_viewport(tree);
    assert_eq!(actual3.line_idx(), 1);
    assert_eq!(actual3.char_idx(), 2);

    // Step-4
    stateful._cursor_move(&data_access, Command::CursorMoveBy((0, -1)));
    let tree = data_access.tree.clone();
    let actual4 = get_cursor_viewport(tree);
    assert_eq!(actual4.line_idx(), 0);
    assert_eq!(actual4.char_idx(), 2);
  }

  #[test]
  fn nowrap2() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let buf = make_buffer_from_lines(terminal_size.height(), buf_opts, lines);
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let tree = make_tree_with_buffers(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      bufs.clone(),
    );
    let state = State::to_arc(State::default());
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let stateful = NormalStateful::default();
    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    for _ in 0..10 {
      let commands = [
        Command::CursorMoveBy((0, 2)),
        Command::CursorMoveBy((3, 0)),
        Command::CursorMoveBy((0, -2)),
        Command::CursorMoveBy((-3, 0)),
      ];
      let data_access = StatefulDataAccess::new(
        state.clone(),
        tree.clone(),
        bufs.clone(),
        Event::Key(key_event),
      );
      for c in commands.iter() {
        stateful._cursor_move(&data_access, *c);
      }
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree);
      assert_eq!(actual.line_idx(), 0);
      assert_eq!(actual.char_idx(), 0);
    }

    for _ in 0..10 {
      let commands = [
        Command::CursorMoveBy((5, 0)),
        Command::CursorMoveBy((0, 1)),
        Command::CursorMoveBy((-5, 0)),
        Command::CursorMoveBy((0, -1)),
      ];
      let data_access = StatefulDataAccess::new(
        state.clone(),
        tree.clone(),
        bufs.clone(),
        Event::Key(key_event),
      );
      for c in commands.iter() {
        stateful._cursor_move(&data_access, *c);
      }
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree);
      assert_eq!(actual.line_idx(), 0);
      assert_eq!(actual.char_idx(), 0);
    }
  }

  #[test]
  fn wrap1() {
    test_log_init();

    let lines = vec![
      "This is a quite simple test.\n",
      "It has these parts:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let terminal_size = U16Size::new(50, 50);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let buf = make_buffer_from_lines(terminal_size.height(), buf_opts, lines.clone());
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let tree = make_tree_with_buffers(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(true)
        .build()
        .unwrap(),
      bufs.clone(),
    );
    let state = State::to_arc(State::default());
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let stateful = NormalStateful::default();
    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    // step-1: Move to the end of line-1.
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree.clone(),
      bufs.clone(),
      Event::Key(key_event),
    );
    let command = Command::CursorMoveBy((lines[0].len() as isize, 0));
    stateful._cursor_move(&data_access, command);

    let tree = data_access.tree.clone();
    let actual1 = get_cursor_viewport(tree.clone());
    assert_eq!(actual1.line_idx(), 0);
    assert_eq!(actual1.char_idx(), 27);

    // step-2: Move down to line-2.
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree.clone(),
      bufs.clone(),
      Event::Key(key_event),
    );
    let command = Command::CursorMoveBy((0, 1));
    stateful._cursor_move(&data_access, command);

    let tree = data_access.tree.clone();
    let actual2 = get_cursor_viewport(tree);
    assert_eq!(actual2.line_idx(), 1);
    assert_eq!(actual2.char_idx(), 18);
  }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests_cursor_move_to {
  use super::tests_util::*;
  use super::*;

  use crate::buf::BufferLocalOptionsBuilder;
  use crate::prelude::*;
  use crate::state::State;
  use crate::test::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::test::log::init as test_log_init;
  use crate::test::tree::make_tree_with_buffers;
  use crate::ui::widget::window::WindowLocalOptionsBuilder;

  use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

  #[test]
  fn nowrap1() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let buf = make_buffer_from_lines(terminal_size.height(), buf_opts, lines);
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let tree = make_tree_with_buffers(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      bufs.clone(),
    );
    let state = State::to_arc(State::default());
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let stateful = NormalStateful::default();
    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    // Step-1
    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    stateful._cursor_move(&data_access, Command::CursorMoveTo((5, 0)));

    let tree = data_access.tree.clone();
    let actual1 = get_cursor_viewport(tree);
    assert_eq!(actual1.line_idx(), 0);
    assert_eq!(actual1.char_idx(), 5);

    // Step-2
    stateful._cursor_move(&data_access, Command::CursorMoveTo((5, 1)));
    let tree = data_access.tree.clone();
    let actual2 = get_cursor_viewport(tree);
    assert_eq!(actual2.line_idx(), 1);
    assert_eq!(actual2.char_idx(), 5);

    // Step-3
    stateful._cursor_move(&data_access, Command::CursorMoveTo((2, 1)));
    let tree = data_access.tree.clone();
    let actual3 = get_cursor_viewport(tree);
    assert_eq!(actual3.line_idx(), 1);
    assert_eq!(actual3.char_idx(), 2);

    // Step-4
    stateful._cursor_move(&data_access, Command::CursorMoveTo((2, 0)));
    let tree = data_access.tree.clone();
    let actual4 = get_cursor_viewport(tree);
    assert_eq!(actual4.line_idx(), 0);
    assert_eq!(actual4.char_idx(), 2);
  }

  #[test]
  fn nowrap2() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let buf = make_buffer_from_lines(terminal_size.height(), buf_opts, lines);
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let tree = make_tree_with_buffers(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      bufs.clone(),
    );
    let state = State::to_arc(State::default());
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let stateful = NormalStateful::default();
    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    for _ in 0..10 {
      let commands = [
        Command::CursorMoveTo((0, 2)),
        Command::CursorMoveTo((3, 2)),
        Command::CursorMoveTo((3, 0)),
        Command::CursorMoveTo((0, 0)),
      ];
      let data_access = StatefulDataAccess::new(
        state.clone(),
        tree.clone(),
        bufs.clone(),
        Event::Key(key_event),
      );
      for c in commands.iter() {
        stateful._cursor_move(&data_access, *c);
      }
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree);
      assert_eq!(actual.line_idx(), 0);
      assert_eq!(actual.char_idx(), 0);
    }

    for _ in 0..10 {
      let commands = [
        Command::CursorMoveTo((5, 0)),
        Command::CursorMoveTo((5, 1)),
        Command::CursorMoveTo((5, 1)),
        Command::CursorMoveTo((0, 0)),
      ];
      let data_access = StatefulDataAccess::new(
        state.clone(),
        tree.clone(),
        bufs.clone(),
        Event::Key(key_event),
      );
      for c in commands.iter() {
        stateful._cursor_move(&data_access, *c);
      }
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree);
      assert_eq!(actual.line_idx(), 0);
      assert_eq!(actual.char_idx(), 0);
    }
  }

  #[test]
  fn wrap1() {
    test_log_init();

    let lines = vec![
      "This is a quite simple test.\n",
      "It has these parts:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];

    let terminal_size = U16Size::new(50, 50);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let buf = make_buffer_from_lines(terminal_size.height(), buf_opts, lines.clone());
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let tree = make_tree_with_buffers(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(true)
        .build()
        .unwrap(),
      bufs.clone(),
    );
    let state = State::to_arc(State::default());
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let stateful = NormalStateful::default();
    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let first_line_len = lines[0].len();

    // step-1: Move to the end of line-1.
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree.clone(),
      bufs.clone(),
      Event::Key(key_event),
    );
    let command = Command::CursorMoveTo((first_line_len, 0));
    stateful._cursor_move(&data_access, command);

    let tree = data_access.tree.clone();
    let actual1 = get_cursor_viewport(tree.clone());
    assert_eq!(actual1.line_idx(), 0);
    assert_eq!(actual1.char_idx(), 27);

    // step-2: Move down to line-2.
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree.clone(),
      bufs.clone(),
      Event::Key(key_event),
    );
    let command = Command::CursorMoveTo((first_line_len, 1));
    stateful._cursor_move(&data_access, command);

    let tree = data_access.tree.clone();
    let actual2 = get_cursor_viewport(tree);
    assert_eq!(actual2.line_idx(), 1);
    assert_eq!(actual2.char_idx(), 18);
  }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests_window_scroll_y_by {
  use super::tests_util::*;
  use super::*;

  use crate::buf::{BufferArc, BufferLocalOptionsBuilder, BuffersManagerArc};
  use crate::lock;
  use crate::prelude::*;
  use crate::state::{State, StateArc};
  use crate::test::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::test::log::init as test_log_init;
  use crate::test::tree::make_tree_with_buffers;
  use crate::ui::tree::TreeArc;
  use crate::ui::widget::window::{Viewport, WindowLocalOptions, WindowLocalOptionsBuilder};

  use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
  use std::collections::BTreeMap;
  use tracing::info;

  #[test]
  fn nowrap1() {
    test_log_init();

    let (tree, state, bufs, buf) = make_tree(
      U16Size::new(10, 10),
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      vec![],
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    // Before cursor scroll
    {
      let viewport = get_viewport(tree.clone());
      let cursor_viewport = get_cursor_viewport(tree.clone());
      assert_eq!(cursor_viewport.line_idx(), 0);
      assert_eq!(cursor_viewport.char_idx(), 0);

      let expect = vec![""];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        1,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollUpBy(1));

    let tree = data_access.tree.clone();

    // After cursor scroll
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![""];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        1,
        &expect_fills,
        &expect_fills,
      );
    }
  }

  #[test]
  fn nowrap2() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let (tree, state, bufs, buf) = make_tree(
      U16Size::new(10, 10),
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      lines,
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    // Before cursor scroll
    {
      info!("before cursor scroll");
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "     * The",
        "     * The",
        "",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![
        (0, 0),
        (1, 0),
        (2, 0),
        (3, 0),
        (4, 0),
        (5, 0),
        (6, 0),
        (7, 0),
      ]
      .into_iter()
      .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        8,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollDownBy(1));

    let tree = data_access.tree.clone();

    // After cursor scroll
    {
      info!("after cursor scroll");
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "     * The",
        "     * The",
        "",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![
        (0, 0),
        (1, 0),
        (2, 0),
        (3, 0),
        (4, 0),
        (5, 0),
        (6, 0),
        (7, 0),
      ]
      .into_iter()
      .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        8,
        &expect_fills,
        &expect_fills,
      );
    }
  }

  #[test]
  fn nowrap3() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      "  3. If a single char needs multiple cells to display on the window, and it happens the char is at the end of the row, there can be multiple cases:\n",
      "     * The char exactly ends at the end of the row, i.e. the last display column of the char is exactly the last column on the row. In this case, we are happy because the char can be put at the end of the row.\n",
      "     * The char is too long to put at the end of the row, thus we will have to put the char to the beginning of the next row (because we don't cut a single char into pieces)\n",
    ];
    let (tree, state, bufs, buf) = make_tree(
      U16Size::new(10, 7),
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      lines,
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    // Before cursor scroll
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "     * The",
        "     * The",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0)]
          .into_iter()
          .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        7,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollDownBy(1));

    let tree = data_access.tree.clone();

    // After cursor scroll
    {
      let viewport = get_viewport(tree);
      let expect = vec![
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "     * The",
        "     * The",
        "  3. If a ",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]
          .into_iter()
          .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        1,
        8,
        &expect_fills,
        &expect_fills,
      );
    }
  }

  #[test]
  fn nowrap4() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      "  3. If a single char needs multiple cells to display on the window, and it happens the char is at the end of the row, there can be multiple cases:\n",
      "     * The char exactly ends at the end of the row, i.e. the last display column of the char is exactly the last column on the row. In this case, we are happy because the char can be put at the end of the row.\n",
      "     * The char is too long to put at the end of the row, thus we will have to put the char to the beginning of the next row (because we don't cut a single char into pieces)\n",
    ];
    let (tree, state, bufs, buf) = make_tree(
      U16Size::new(10, 5),
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      lines,
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    // Before cursor scroll
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollDownBy(4));

    let tree = data_access.tree.clone();

    // After cursor scroll
    {
      let viewport = get_viewport(tree);
      let expect = vec![
        "  2. When ",
        "     * The",
        "     * The",
        "  3. If a ",
        "     * The",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(4, 0), (5, 0), (6, 0), (7, 0), (8, 0)]
        .into_iter()
        .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        4,
        9,
        &expect_fills,
        &expect_fills,
      );
    }
  }

  #[test]
  fn nowrap5() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      "  3. If a single char needs multiple cells to display on the window, and it happens the char is at the end of the row, there can be multiple cases:\n",
      "     * The char exactly ends at the end of the row, i.e. the last display column of the char is exactly the last column on the row. In this case, we are happy because the char can be put at the end of the row.\n",
      "     * The char is too long to put at the end of the row, thus we will have to put the char to the beginning of the next row (because we don't cut a single char into pieces)\n",
    ];
    let (tree, state, bufs, buf) = make_tree(
      U16Size::new(10, 5),
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      lines,
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    // Before cursor scroll
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    // Scroll-1
    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollDownBy(4));

    let tree = data_access.tree.clone();
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "  2. When ",
        "     * The",
        "     * The",
        "  3. If a ",
        "     * The",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(4, 0), (5, 0), (6, 0), (7, 0), (8, 0)]
        .into_iter()
        .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        4,
        9,
        &expect_fills,
        &expect_fills,
      );
    }

    // Scroll-2
    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollDownBy(4));

    let tree = data_access.tree.clone();
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec!["     * The", "     * The", "", "", ""];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(8, 0), (9, 0), (10, 0)].into_iter().collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        8,
        11,
        &expect_fills,
        &expect_fills,
      );
    }

    // Scroll-3
    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollUpBy(1));

    let tree = data_access.tree.clone();
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec!["  3. If a ", "     * The", "     * The", "", ""];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(7, 0), (8, 0), (9, 0), (10, 0)].into_iter().collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        7,
        11,
        &expect_fills,
        &expect_fills,
      );
    }

    // Scroll-4
    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollUpBy(4));

    let tree = data_access.tree.clone();
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "  1. When ",
        "  2. When ",
        "     * The",
        "     * The",
        "  3. If a ",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        3,
        8,
        &expect_fills,
        &expect_fills,
      );
    }

    // Scroll-5
    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollUpBy(1));

    let tree = data_access.tree.clone();
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "But still ",
        "  1. When ",
        "  2. When ",
        "     * The",
        "     * The",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 0), (5, 0), (6, 0)]
        .into_iter()
        .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        2,
        7,
        &expect_fills,
        &expect_fills,
      );
    }

    // Scroll-6
    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollUpBy(3));

    let tree = data_access.tree.clone();
    {
      let viewport = get_viewport(tree);
      let expect = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }
  }

  #[test]
  fn wrap1() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      "  3. If a single char needs multiple cells to display on the window, and it happens the char is at the end of the row, there can be multiple cases:\n",
      "     * The char exactly ends at the end of the row, i.e. the last display column of the char is exactly the last column on the row. In this case, we are happy because the char can be put at the end of the row.\n",
      "     * The char is too long to put at the end of the row, thus we will have to put the char to the beginning of the next row (because we don't cut a single char into pieces)\n",
    ];
    let (tree, state, bufs, buf) = make_tree(
      U16Size::new(15, 15),
      WindowLocalOptionsBuilder::default()
        .wrap(true)
        .build()
        .unwrap(),
      lines,
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    // Before cursor scroll
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite",
        " simple and sma",
        "ll test lines.\n",
        "But still it co",
        "ntains several ",
        "things we want ",
        "to test:\n",
        "  1. When the l",
        "ine is small en",
        "ough to complet",
        "ely put inside ",
        "a row of the wi",
        "ndow content wi",
        "dget, then the ",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollDownBy(4));

    let tree = data_access.tree.clone();

    // After cursor scroll
    {
      let viewport = get_viewport(tree);
      let expect = vec![
        "  2. When the l",
        "ine is too long",
        " to be complete",
        "ly put in a row",
        " of the window ",
        "content widget,",
        " there're multi",
        "ple cases:\n",
        "     * The extr",
        "a parts are bee",
        "n truncated if ",
        "both line-wrap ",
        "and word-wrap o",
        "ptions are not ",
        "set.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(4, 0), (5, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        4,
        6,
        &expect_fills,
        &expect_fills,
      );
    }
  }

  #[test]
  fn wrap2() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      "  3. If a single char needs multiple cells to display on the window, and it happens the char is at the end of the row, there can be multiple cases:\n",
      "     * The char exactly ends at the end of the row, i.e. the last display column of the char is exactly the last column on the row. In this case, we are happy because the char can be put at the end of the row.\n",
      "     * The char is too long to put at the end of the row, thus we will have to put the char to the beginning of the next row (because we don't cut a single char into pieces)\n",
    ];
    let (tree, state, bufs, buf) = make_tree(
      U16Size::new(15, 15),
      WindowLocalOptionsBuilder::default()
        .wrap(true)
        .build()
        .unwrap(),
      lines,
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    // Before cursor scroll
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite",
        " simple and sma",
        "ll test lines.\n",
        "But still it co",
        "ntains several ",
        "things we want ",
        "to test:\n",
        "  1. When the l",
        "ine is small en",
        "ough to complet",
        "ely put inside ",
        "a row of the wi",
        "ndow content wi",
        "dget, then the ",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollDownBy(8));

    let tree = data_access.tree.clone();

    // Scroll-1
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "     * The char",
        " exactly ends a",
        "t the end of th",
        "e row, i.e. the",
        " last display c",
        "olumn of the ch",
        "ar is exactly t",
        "he last column ",
        "on the row. In ",
        "this case, we a",
        "re happy becaus",
        "e the char can ",
        "be put at the e",
        "nd of the row.\n",
        "     * The char",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(8, 0), (9, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        8,
        10,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollDownBy(1));

    let tree = data_access.tree.clone();

    // Scroll-2
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "     * The char",
        " is too long to",
        " put at the end",
        " of the row, th",
        "us we will have",
        " to put the cha",
        "r to the beginn",
        "ing of the next",
        " row (because w",
        "e don't cut a s",
        "ingle char into",
        " pieces)\n",
        "",
        "",
        "",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(9, 0), (10, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        9,
        11,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollDownBy(3));

    let tree = data_access.tree.clone();

    // Scroll-3
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "     * The char",
        " is too long to",
        " put at the end",
        " of the row, th",
        "us we will have",
        " to put the cha",
        "r to the beginn",
        "ing of the next",
        " row (because w",
        "e don't cut a s",
        "ingle char into",
        " pieces)\n",
        "",
        "",
        "",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(9, 0), (10, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        9,
        11,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollUpBy(2));

    let tree = data_access.tree.clone();

    // Scroll-4
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "  3. If a singl",
        "e char needs mu",
        "ltiple cells to",
        " display on the",
        " window, and it",
        " happens the ch",
        "ar is at the en",
        "d of the row, t",
        "here can be mul",
        "tiple cases:\n",
        "     * The char",
        " exactly ends a",
        "t the end of th",
        "e row, i.e. the",
        " last display c",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(7, 0), (8, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        7,
        9,
        &expect_fills,
        &expect_fills,
      );
    }
  }
}
#[cfg(test)]
#[allow(unused_imports)]
mod tests_window_scroll_x_by {
  use super::tests_util::*;
  use super::*;

  use crate::buf::{BufferArc, BufferLocalOptionsBuilder, BuffersManagerArc};
  use crate::lock;
  use crate::prelude::*;
  use crate::state::{State, StateArc};
  use crate::test::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::test::log::init as test_log_init;
  use crate::test::tree::make_tree_with_buffers;
  use crate::ui::tree::TreeArc;
  use crate::ui::widget::window::{Viewport, WindowLocalOptions, WindowLocalOptionsBuilder};

  use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
  use std::collections::BTreeMap;
  use tracing::info;

  #[test]
  fn nowrap1() {
    test_log_init();

    let (tree, state, bufs, buf) = make_tree(
      U16Size::new(10, 10),
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      vec![],
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    // Before cursor scroll
    {
      let viewport = get_viewport(tree.clone());
      let cursor_viewport = get_cursor_viewport(tree.clone());
      assert_eq!(cursor_viewport.line_idx(), 0);
      assert_eq!(cursor_viewport.char_idx(), 0);

      let expect = vec![""];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        1,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollRightBy(1));

    let tree = data_access.tree.clone();

    // After cursor scroll
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![""];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        1,
        &expect_fills,
        &expect_fills,
      );
    }
  }

  #[test]
  fn nowrap2() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let (tree, state, bufs, buf) = make_tree(
      U16Size::new(10, 10),
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      lines,
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    // Before cursor scroll
    {
      info!("before cursor scroll");
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "     * The",
        "     * The",
        "",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![
        (0, 0),
        (1, 0),
        (2, 0),
        (3, 0),
        (4, 0),
        (5, 0),
        (6, 0),
        (7, 0),
      ]
      .into_iter()
      .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        8,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollRightBy(1));

    let tree = data_access.tree.clone();

    // After cursor scroll
    {
      info!("after cursor scroll");
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "ello, RSVI",
        "his is a q",
        "ut still i",
        " 1. When t",
        " 2. When t",
        "    * The ",
        "    * The ",
        "",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![
        (0, 0),
        (1, 0),
        (2, 0),
        (3, 0),
        (4, 0),
        (5, 0),
        (6, 0),
        (7, 0),
      ]
      .into_iter()
      .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        8,
        &expect_fills,
        &expect_fills,
      );
    }
  }

  #[test]
  fn nowrap3() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      "  3. If a single char needs multiple cells to display on the window, and it happens the char is at the end of the row, there can be multiple cases:\n",
      "     * The char exactly ends at the end of the row, i.e. the last display column of the char is exactly the last column on the row. In this case, we are happy because the char can be put at the end of the row.\n",
      "     * The char is too long to put at the end of the row, thus we will have to put the char to the beginning of the next row (because we don't cut a single char into pieces)\n",
    ];
    let (tree, state, bufs, buf) = make_tree(
      U16Size::new(10, 7),
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      lines,
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    // Before cursor scroll
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "     * The",
        "     * The",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0)]
          .into_iter()
          .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        7,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollLeftBy(1));

    let tree = data_access.tree.clone();

    // After cursor scroll
    {
      let viewport = get_viewport(tree);
      let expect = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "     * The",
        "     * The",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0)]
          .into_iter()
          .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        7,
        &expect_fills,
        &expect_fills,
      );
    }
  }

  #[test]
  fn nowrap4() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      "  3. If a single char needs multiple cells to display on the window, and it happens the char is at the end of the row, there can be multiple cases:\n",
      "     * The char exactly ends at the end of the row, i.e. the last display column of the char is exactly the last column on the row. In this case, we are happy because the char can be put at the end of the row.\n",
      "     * The char is too long to put at the end of the row, thus we will have to put the char to the beginning of the next row (because we don't cut a single char into pieces)\n",
    ];
    let (tree, state, bufs, buf) = make_tree(
      U16Size::new(10, 5),
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      lines,
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    // Before cursor scroll
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollRightBy(149));

    let tree = data_access.tree.clone();

    // After cursor scroll
    {
      let viewport = get_viewport(tree);
      let expect = vec!["", "", "", "rendering.", ""];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }
  }

  #[test]
  fn nowrap5() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      "  3. If a single char needs multiple cells to display on the window, and it happens the char is at the end of the row, there can be multiple cases:\n",
      "     * The char exactly ends at the end of the row, i.e. the last display column of the char is exactly the last column on the row. In this case, we are happy because the char can be put at the end of the row.\n",
      "     * The char is too long to put at the end of the row, thus we will have to put the char to the beginning of the next row (because we don't cut a single char into pieces)\n",
    ];
    let (tree, state, bufs, buf) = make_tree(
      U16Size::new(10, 5),
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      lines,
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    // Before cursor scroll
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollRightBy(100));

    let tree = data_access.tree.clone();

    // Scroll-1
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec!["", "", "", " the line-", "multiple c"];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollRightBy(10));

    let tree = data_access.tree.clone();

    // Scroll-2
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec!["", "", "", "wrap and w", "ases:\n"];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollRightBy(50));

    let tree = data_access.tree.clone();

    // Scroll-3
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec!["", "", "", "rendering.", ""];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollLeftBy(10));

    let tree = data_access.tree.clone();

    // Scroll-4
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec!["", "", "", "ffect the ", ""];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }
  }

  #[test]
  fn nowrap6() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      "  3. If a single char needs multiple cells to display on the window, and it happens the char is at the end of the row, there can be multiple cases:\n",
      "     * The char exactly ends at the end of the row, i.e. the last display column of the char is exactly the last column on the row. In this case, we are happy because the char can be put at the end of the row.\n",
      "     * The char is too long to put at the end of the row, thus we will have to put the char to the beginning of the next row (because we don't cut a single char into pieces)\n",
    ];
    let (tree, state, bufs, buf) = make_tree(
      U16Size::new(10, 5),
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      lines,
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    // Before cursor scroll
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    // Scroll-1
    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollRightBy(4));

    let tree = data_access.tree.clone();
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "o, RSVIM!\n",
        " is a quit",
        "still it c",
        " When the ",
        " When the ",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    // Scroll-2
    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollRightBy(4));

    let tree = data_access.tree.clone();
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "SVIM!\n",
        "a quite si",
        "l it conta",
        "n the line",
        "n the line",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    // Scroll-3
    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollLeftBy(1));

    let tree = data_access.tree.clone();
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "RSVIM!\n",
        " a quite s",
        "ll it cont",
        "en the lin",
        "en the lin",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    // Scroll-4
    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollLeftBy(4));

    let tree = data_access.tree.clone();
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "lo, RSVIM!",
        "s is a qui",
        " still it ",
        ". When the",
        ". When the",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    // Scroll-5
    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollLeftBy(1));

    let tree = data_access.tree.clone();
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "llo, RSVIM",
        "is is a qu",
        "t still it",
        "1. When th",
        "2. When th",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    // Scroll-6
    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollLeftBy(3));

    let tree = data_access.tree.clone();
    {
      let viewport = get_viewport(tree);
      let expect = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }
  }

  #[test]
  fn wrap1() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      "  3. If a single char needs multiple cells to display on the window, and it happens the char is at the end of the row, there can be multiple cases:\n",
      "     * The char exactly ends at the end of the row, i.e. the last display column of the char is exactly the last column on the row. In this case, we are happy because the char can be put at the end of the row.\n",
      "     * The char is too long to put at the end of the row, thus we will have to put the char to the beginning of the next row (because we don't cut a single char into pieces)\n",
    ];
    let (tree, state, bufs, buf) = make_tree(
      U16Size::new(15, 15),
      WindowLocalOptionsBuilder::default()
        .wrap(true)
        .build()
        .unwrap(),
      lines,
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    // Before cursor scroll
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite",
        " simple and sma",
        "ll test lines.\n",
        "But still it co",
        "ntains several ",
        "things we want ",
        "to test:\n",
        "  1. When the l",
        "ine is small en",
        "ough to complet",
        "ely put inside ",
        "a row of the wi",
        "ndow content wi",
        "dget, then the ",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollRightBy(4));

    let tree = data_access.tree.clone();

    // After cursor scroll
    {
      let viewport = get_viewport(tree);
      let expect = vec![
        "o, RSVIM!\n",
        " is a quite sim",
        "ple and small t",
        "est lines.\n",
        "still it contai",
        "ns several thin",
        "gs we want to t",
        "est:\n",
        " When the line ",
        "is small enough",
        " to completely ",
        "put inside a ro",
        "w of the window",
        " content widget",
        ", then the line",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        4,
        &expect_fills,
        &expect_fills,
      );
    }
  }

  #[test]
  fn wrap2() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      "  3. If a single char needs multiple cells to display on the window, and it happens the char is at the end of the row, there can be multiple cases:\n",
      "     * The char exactly ends at the end of the row, i.e. the last display column of the char is exactly the last column on the row. In this case, we are happy because the char can be put at the end of the row.\n",
      "     * The char is too long to put at the end of the row, thus we will have to put the char to the beginning of the next row (because we don't cut a single char into pieces)\n",
    ];
    let (tree, state, bufs, buf) = make_tree(
      U16Size::new(15, 15),
      WindowLocalOptionsBuilder::default()
        .wrap(true)
        .build()
        .unwrap(),
      lines,
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    // Before cursor scroll
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite",
        " simple and sma",
        "ll test lines.\n",
        "But still it co",
        "ntains several ",
        "things we want ",
        "to test:\n",
        "  1. When the l",
        "ine is small en",
        "ough to complet",
        "ely put inside ",
        "a row of the wi",
        "ndow content wi",
        "dget, then the ",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollRightBy(8));

    let tree = data_access.tree.clone();

    // Scroll-1
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "SVIM!\n",
        "a quite simple ",
        "and small test ",
        "lines.\n",
        "l it contains s",
        "everal things w",
        "e want to test:",
        "n the line is s",
        "mall enough to ",
        "completely put ",
        "inside a row of",
        " the window con",
        "tent widget, th",
        "en the line-wra",
        "p and word-wrap",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollRightBy(1));

    let tree = data_access.tree.clone();

    // Scroll-2
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "VIM!\n",
        " quite simple a",
        "nd small test l",
        "ines.\n",
        " it contains se",
        "veral things we",
        " want to test:\n",
        " the line is sm",
        "all enough to c",
        "ompletely put i",
        "nside a row of ",
        "the window cont",
        "ent widget, the",
        "n the line-wrap",
        " and word-wrap ",
        "doesn't affect ",
        "the rendering.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollRightBy(3));

    let tree = data_access.tree.clone();

    // Scroll-3
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "!\n",
        "ite simple and ",
        "small test line",
        "s.\n",
        " contains sever",
        "al things we wa",
        "nt to test:\n",
        "e line is small",
        " enough to comp",
        "letely put insi",
        "de a row of the",
        " window content",
        " widget, then t",
        "he line-wrap an",
        "d word-wrap doe",
        "sn't affect the",
        " rendering.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollLeftBy(1));

    let tree = data_access.tree.clone();

    // Scroll-4
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "M!\n",
        "uite simple and",
        " small test lin",
        "es.\n",
        "t contains seve",
        "ral things we w",
        "ant to test:\n",
        "he line is smal",
        "l enough to com",
        "pletely put ins",
        "ide a row of th",
        "e window conten",
        "t widget, then ",
        "the line-wrap a",
        "nd word-wrap do",
        "esn't affect th",
        "e rendering.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        4,
        &expect_fills,
        &expect_fills,
      );
    }
  }
}
#[cfg(test)]
#[allow(unused_imports)]
mod tests_window_scroll_to {
  use super::tests_util::*;
  use super::*;

  use crate::buf::{BufferArc, BufferLocalOptionsBuilder, BuffersManagerArc};
  use crate::lock;
  use crate::prelude::*;
  use crate::state::{State, StateArc};
  use crate::test::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::test::log::init as test_log_init;
  use crate::test::tree::make_tree_with_buffers;
  use crate::ui::tree::TreeArc;
  use crate::ui::widget::window::{Viewport, WindowLocalOptions, WindowLocalOptionsBuilder};

  use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
  use std::collections::BTreeMap;
  use tracing::info;

  #[test]
  fn nowrap1() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let (tree, state, bufs, buf) = make_tree(
      U16Size::new(10, 10),
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      lines,
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    // Before cursor scroll
    {
      info!("before cursor scroll");
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "     * The",
        "     * The",
        "",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![
        (0, 0),
        (1, 0),
        (2, 0),
        (3, 0),
        (4, 0),
        (5, 0),
        (6, 0),
        (7, 0),
      ]
      .into_iter()
      .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        8,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollTo((0, 1)));

    let tree = data_access.tree.clone();

    // After cursor scroll
    {
      info!("after cursor scroll");
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "     * The",
        "     * The",
        "",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![
        (0, 0),
        (1, 0),
        (2, 0),
        (3, 0),
        (4, 0),
        (5, 0),
        (6, 0),
        (7, 0),
      ]
      .into_iter()
      .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        8,
        &expect_fills,
        &expect_fills,
      );
    }
  }

  #[test]
  fn nowrap2() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      "  3. If a single char needs multiple cells to display on the window, and it happens the char is at the end of the row, there can be multiple cases:\n",
      "     * The char exactly ends at the end of the row, i.e. the last display column of the char is exactly the last column on the row. In this case, we are happy because the char can be put at the end of the row.\n",
      "     * The char is too long to put at the end of the row, thus we will have to put the char to the beginning of the next row (because we don't cut a single char into pieces)\n",
    ];
    let (tree, state, bufs, buf) = make_tree(
      U16Size::new(10, 5),
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      lines,
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    // Before cursor scroll
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    // Scroll-1
    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollTo((0, 4)));

    let tree = data_access.tree.clone();
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "  2. When ",
        "     * The",
        "     * The",
        "  3. If a ",
        "     * The",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(4, 0), (5, 0), (6, 0), (7, 0), (8, 0)]
        .into_iter()
        .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        4,
        9,
        &expect_fills,
        &expect_fills,
      );
    }

    // Scroll-2
    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollTo((0, 8)));

    let tree = data_access.tree.clone();
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec!["     * The", "     * The", "", "", ""];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(8, 0), (9, 0), (10, 0)].into_iter().collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        8,
        11,
        &expect_fills,
        &expect_fills,
      );
    }

    // Scroll-3
    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollTo((0, 7)));

    let tree = data_access.tree.clone();
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec!["  3. If a ", "     * The", "     * The", "", ""];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(7, 0), (8, 0), (9, 0), (10, 0)].into_iter().collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        7,
        11,
        &expect_fills,
        &expect_fills,
      );
    }

    // Scroll-4
    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollTo((0, 3)));

    let tree = data_access.tree.clone();
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "  1. When ",
        "  2. When ",
        "     * The",
        "     * The",
        "  3. If a ",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        3,
        8,
        &expect_fills,
        &expect_fills,
      );
    }

    // Scroll-5
    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollTo((0, 2)));

    let tree = data_access.tree.clone();
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "But still ",
        "  1. When ",
        "  2. When ",
        "     * The",
        "     * The",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 0), (5, 0), (6, 0)]
        .into_iter()
        .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        2,
        7,
        &expect_fills,
        &expect_fills,
      );
    }

    // Scroll-6
    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollTo((0, 0)));

    let tree = data_access.tree.clone();
    {
      let viewport = get_viewport(tree);
      let expect = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }
  }

  #[test]
  fn nowrap3() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      "  3. If a single char needs multiple cells to display on the window, and it happens the char is at the end of the row, there can be multiple cases:\n",
      "     * The char exactly ends at the end of the row, i.e. the last display column of the char is exactly the last column on the row. In this case, we are happy because the char can be put at the end of the row.\n",
      "     * The char is too long to put at the end of the row, thus we will have to put the char to the beginning of the next row (because we don't cut a single char into pieces)\n",
    ];
    let (tree, state, bufs, buf) = make_tree(
      U16Size::new(10, 5),
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      lines,
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    // Before cursor scroll
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    // Scroll-1
    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollTo((4, 0)));

    let tree = data_access.tree.clone();
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "o, RSVIM!\n",
        " is a quit",
        "still it c",
        " When the ",
        " When the ",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    // Scroll-2
    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollTo((8, 0)));

    let tree = data_access.tree.clone();
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "SVIM!\n",
        "a quite si",
        "l it conta",
        "n the line",
        "n the line",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    // Scroll-3
    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollTo((7, 0)));

    let tree = data_access.tree.clone();
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "RSVIM!\n",
        " a quite s",
        "ll it cont",
        "en the lin",
        "en the lin",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    // Scroll-4
    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollTo((3, 0)));

    let tree = data_access.tree.clone();
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "lo, RSVIM!",
        "s is a qui",
        " still it ",
        ". When the",
        ". When the",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    // Scroll-5
    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollTo((2, 0)));

    let tree = data_access.tree.clone();
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "llo, RSVIM",
        "is is a qu",
        "t still it",
        "1. When th",
        "2. When th",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    // Scroll-6
    let data_access =
      StatefulDataAccess::new(state.clone(), tree, bufs.clone(), Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful._window_scroll(&data_access, Command::WindowScrollTo((0, 0)));

    let tree = data_access.tree.clone();
    {
      let viewport = get_viewport(tree);
      let expect = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();

      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }
  }
}
#[cfg(test)]
#[allow(unused_imports)]
mod tests_cursor_move_and_scroll {
  use super::tests_util::*;
  use super::*;

  use crate::buf::{BufferArc, BufferLocalOptionsBuilder, BuffersManagerArc};
  use crate::lock;
  use crate::prelude::*;
  use crate::state::{State, StateArc};
  use crate::test::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::test::log::init as test_log_init;
  use crate::test::tree::make_tree_with_buffers;
  use crate::ui::tree::TreeArc;
  use crate::ui::widget::window::{Viewport, WindowLocalOptions, WindowLocalOptionsBuilder};

  use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
  use std::collections::BTreeMap;
  use tracing::info;

  #[test]
  fn nowrap1() {
    test_log_init();

    let (tree, state, bufs, _buf) = make_tree(
      U16Size::new(10, 10),
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      vec![],
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful_machine = NormalStateful::default();
    stateful_machine.cursor_move(&data_access, Command::CursorMoveUpBy(1));

    let tree = data_access.tree.clone();
    let actual = get_cursor_viewport(tree);
    assert_eq!(actual.line_idx(), 0);
    assert_eq!(actual.char_idx(), 0);
  }

  #[test]
  fn nowrap2() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let (tree, state, bufs, _buf) = make_tree(
      U16Size::new(10, 10),
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      lines,
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree, bufs, Event::Key(key_event));
    let stateful_machine = NormalStateful::default();
    stateful_machine.cursor_move(&data_access, Command::CursorMoveUpBy(1));

    let tree = data_access.tree.clone();
    let actual = get_cursor_viewport(tree);
    assert_eq!(actual.line_idx(), 0);
    assert_eq!(actual.char_idx(), 0);
  }

  #[test]
  fn nowrap3() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let (tree, state, bufs, buf) = make_tree(
      U16Size::new(10, 10),
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      lines,
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree.clone(), bufs, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful.cursor_move(&data_access, Command::CursorMoveDownBy(3));

    // Move-1
    {
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 3);
      assert_eq!(actual1.char_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "     * The",
        "     * The",
        "",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![
        (0, 0),
        (1, 0),
        (2, 0),
        (3, 0),
        (4, 0),
        (5, 0),
        (6, 0),
        (7, 0),
      ]
      .into_iter()
      .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        8,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Command::CursorMoveDownBy(5));

    // Move-2
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 7);
      assert_eq!(actual2.char_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "     * The",
        "     * The",
        "",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![
        (0, 0),
        (1, 0),
        (2, 0),
        (3, 0),
        (4, 0),
        (5, 0),
        (6, 0),
        (7, 0),
      ]
      .into_iter()
      .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        8,
        &expect_fills,
        &expect_fills,
      );
    }
  }

  #[test]
  fn wrap1() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let (tree, state, bufs, buf) = make_tree(
      U16Size::new(10, 10),
      WindowLocalOptionsBuilder::default()
        .wrap(true)
        .build()
        .unwrap(),
      lines,
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree.clone(), bufs, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful.cursor_move(&data_access, Command::CursorMoveDownBy(3));

    // Move-1
    {
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 3);
      assert_eq!(actual1.char_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "  1. When ",
        "the line i",
        "s small en",
        "ough to co",
        "mpletely p",
        "ut inside ",
        "a row of t",
        "he window ",
        "content wi",
        "dget, then",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        3,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Command::CursorMoveDownBy(2));

    // Move-2
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 5);
      assert_eq!(actual2.char_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "     * The",
        " extra par",
        "ts are bee",
        "n truncate",
        "d if both ",
        "line-wrap ",
        "and word-w",
        "rap option",
        "s are not ",
        "set.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        5,
        6,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Command::CursorMoveDownBy(3));

    // Move-3
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 7);
      assert_eq!(actual2.char_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![""];
      let expect_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        7,
        8,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Command::CursorMoveUpBy(3));

    // Move-4
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 4);
      assert_eq!(actual2.char_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "  2. When ",
        "the line i",
        "s too long",
        " to be com",
        "pletely pu",
        "t in a row",
        " of the wi",
        "ndow conte",
        "nt widget,",
        " there're ",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        4,
        5,
        &expect_fills,
        &expect_fills,
      );
    }
  }

  #[test]
  fn wrap2() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let (tree, state, bufs, buf) = make_tree(
      U16Size::new(10, 10),
      WindowLocalOptionsBuilder::default()
        .wrap(true)
        .build()
        .unwrap(),
      lines,
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree.clone(), bufs, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful.cursor_move(&data_access, Command::CursorMoveDownBy(3));

    // Move-1
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "  1. When ",
        "the line i",
        "s small en",
        "ough to co",
        "mpletely p",
        "ut inside ",
        "a row of t",
        "he window ",
        "content wi",
        "dget, then",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        3,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Command::CursorMoveRightBy(2));

    // Move-2
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 2);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "  1. When ",
        "the line i",
        "s small en",
        "ough to co",
        "mpletely p",
        "ut inside ",
        "a row of t",
        "he window ",
        "content wi",
        "dget, then",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        3,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Command::CursorMoveRightBy(80));

    // Move-3
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 82);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "  1. When ",
        "the line i",
        "s small en",
        "ough to co",
        "mpletely p",
        "ut inside ",
        "a row of t",
        "he window ",
        "content wi",
        "dget, then",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        3,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Command::CursorMoveRightBy(40));

    // Move-4
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 122);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "e a row of",
        " the windo",
        "w content ",
        "widget, th",
        "en the lin",
        "e-wrap and",
        " word-wrap",
        " doesn't a",
        "ffect the ",
        "rendering.",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        3,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Command::CursorMoveRightBy(40));

    // Move-5
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 157);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "e a row of",
        " the windo",
        "w content ",
        "widget, th",
        "en the lin",
        "e-wrap and",
        " word-wrap",
        " doesn't a",
        "ffect the ",
        "rendering.",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        3,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Command::CursorMoveRightBy(5));

    // Move-6
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 157);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "e a row of",
        " the windo",
        "w content ",
        "widget, th",
        "en the lin",
        "e-wrap and",
        " word-wrap",
        " doesn't a",
        "ffect the ",
        "rendering.",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        3,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Command::CursorMoveLeftBy(8));

    // Move-7
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 149);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "e a row of",
        " the windo",
        "w content ",
        "widget, th",
        "en the lin",
        "e-wrap and",
        " word-wrap",
        " doesn't a",
        "ffect the ",
        "rendering.",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        3,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Command::CursorMoveLeftBy(100));

    // Move-8
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 49);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "put inside",
        " a row of ",
        "the window",
        " content w",
        "idget, the",
        "n the line",
        "-wrap and ",
        "word-wrap ",
        "doesn't af",
        "fect the r",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        3,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Command::CursorMoveLeftBy(100));

    // Move-9
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "  1. When ",
        "the line i",
        "s small en",
        "ough to co",
        "mpletely p",
        "ut inside ",
        "a row of t",
        "he window ",
        "content wi",
        "dget, then",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        3,
        4,
        &expect_fills,
        &expect_fills,
      );
    }
  }

  #[test]
  fn wrap3() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "    * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "    * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let (tree, state, bufs, buf) = make_tree(
      U16Size::new(25, 7),
      WindowLocalOptionsBuilder::default()
        .wrap(true)
        .build()
        .unwrap(),
      lines,
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    // Before
    {
      let viewport = get_viewport(tree.clone());

      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite simple an",
        "d small test lines.\n",
        "But still it contains sev",
        "eral things we want to te",
        "st:\n",
        "  1. When the line is sma",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(state, tree.clone(), bufs, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful.cursor_move(&data_access, Command::CursorMoveDownBy(3));

    // Move-1
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite simple an",
        "d small test lines.\n",
        "But still it contains sev",
        "eral things we want to te",
        "st:\n",
        "  1. When the line is sma",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Command::CursorMoveRightBy(24));

    // Move-2
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 24);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite simple an",
        "d small test lines.\n",
        "But still it contains sev",
        "eral things we want to te",
        "st:\n",
        "  1. When the line is sma",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Command::CursorMoveRightBy(1));

    // Move-3
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 25);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite simple an",
        "d small test lines.\n",
        "But still it contains sev",
        "eral things we want to te",
        "st:\n",
        "  1. When the line is sma",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        4,
        &expect_fills,
        &expect_fills,
      );
    }
  }
}
// spellchecker:on
