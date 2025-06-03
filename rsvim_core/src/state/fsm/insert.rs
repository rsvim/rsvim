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
  use crate::ui::widget::window::{
    CursorViewport, Viewport, WindowLocalOptions, WindowLocalOptionsBuilder,
  };

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
mod tests_cursor_move {
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

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );
    let data_access = StatefulDataAccess::new(state, tree.clone(), bufs, Event::Key(key_event));
    let stateful = InsertStateful::default();

    // Move-1
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveBy((5, 3)));

      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 3);
      assert_eq!(actual1.char_idx(), 5);

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

    // Move-2
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(158));

      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 3);
      assert_eq!(actual2.char_idx(), 158);

      let viewport = get_viewport(tree.clone());
      let expect = vec!["", "", "", "endering.\n", "", "", "     * The", ""];
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
  fn wrap_nolinebreak1() {
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
    let stateful = InsertStateful::default();

    // Move-1
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(3));
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

    // Move-2
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(2));
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

    // Move-3
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(3));
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

    // Move-4
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveUpBy(3));
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
  fn wrap_linebreak1() {
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
        .line_break(true)
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
    let stateful = InsertStateful::default();
    stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(3));

    // Move-1
    {
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 3);
      assert_eq!(actual1.char_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "  1. When ",
        "the line ",
        "is small ",
        "enough to ",
        "completely",
        " put ",
        "inside a ",
        "row of the",
        " window ",
        "content ",
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

    stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(2));

    // Move-2
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 5);
      assert_eq!(actual2.char_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "     * The",
        " extra ",
        "parts are ",
        "been ",
        "truncated ",
        "if both ",
        "line-wrap ",
        "and word-",
        "wrap ",
        "options ",
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

    stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(3));

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

    stateful.cursor_move(&data_access, Operation::CursorMoveUpBy(3));

    // Move-4
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 4);
      assert_eq!(actual2.char_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "  2. When ",
        "the line ",
        "is too ",
        "long to be",
        " ",
        "completely",
        " put in a ",
        "row of the",
        " window ",
        "content ",
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
}
// spellchecker:on
