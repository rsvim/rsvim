#![allow(unused_imports)]

use super::insert::*;

use crate::buf::opt::BufferLocalOptionsBuilder;
use crate::buf::{BufferArc, BuffersManagerArc};
use crate::content::{TextContents, TextContentsArc};
use crate::prelude::*;
use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::ops::Operation;
use crate::state::ops::cursor_ops;
use crate::state::{State, StateArc};
use crate::test::buf::{make_buffer_from_lines, make_buffers_manager};
use crate::test::log::init as test_log_init;
use crate::test::tree::make_tree_with_buffers;
use crate::ui::canvas::Canvas;
use crate::ui::tree::TreeArc;
use crate::ui::viewport::{
  CursorViewport, CursorViewportArc, Viewport, ViewportArc, ViewportSearchDirection,
};
use crate::ui::widget::Widgetable;
use crate::ui::widget::window::content::{self, WindowContent};
use crate::ui::widget::window::{WindowLocalOptions, WindowLocalOptionsBuilder};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tracing::info;

pub fn make_tree(
  terminal_size: U16Size,
  window_local_opts: WindowLocalOptions,
  lines: Vec<&str>,
) -> (
  TreeArc,
  StateArc,
  BuffersManagerArc,
  BufferArc,
  TextContentsArc,
) {
  let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
  let buf = make_buffer_from_lines(terminal_size, buf_opts, lines);
  let bufs = make_buffers_manager(buf_opts, vec![buf.clone()]);
  let tree = make_tree_with_buffers(terminal_size, window_local_opts, bufs.clone());
  let (jsrt_tick_dispatcher, _jsrt_tick_queue) = channel(1);
  let state = State::to_arc(State::new(jsrt_tick_dispatcher));
  let contents = TextContents::to_arc(TextContents::new(terminal_size));
  (tree, state, bufs, buf, contents)
}

pub fn get_viewport(tree: TreeArc) -> ViewportArc {
  let tree = lock!(tree);
  tree.current_window().unwrap().viewport()
}

pub fn get_cursor_viewport(tree: TreeArc) -> CursorViewportArc {
  let tree = lock!(tree);
  tree.current_window().unwrap().cursor_viewport()
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
    let (first_line_idx, _first_line_viewport) = actual.lines().first().unwrap();
    let (last_line_idx, _last_line_viewport) = actual.lines().last().unwrap();
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
    .text()
    .rope()
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

      if r > rows.first().unwrap().0 {
        let prev_r = r - 1;
        let prev_row = rows.get(&prev_r).unwrap();
        info!(
          "row-{:?}, current[{}]:{:?}, previous[{}]:{:?}",
          r, r, row, prev_r, prev_row
        );
      }
      if r < rows.last().unwrap().0 {
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

pub fn make_canvas(
  terminal_size: U16Size,
  window_options: WindowLocalOptions,
  buffer: BufferArc,
  viewport: ViewportArc,
) -> Canvas {
  let mut tree = Tree::new(terminal_size);
  tree.set_global_local_options(&window_options);
  let shape = IRect::new(
    (0, 0),
    (
      terminal_size.width() as isize,
      terminal_size.height() as isize,
    ),
  );
  let window_content =
    WindowContent::new(shape, Arc::downgrade(&buffer), Arc::downgrade(&viewport));
  let mut canvas = Canvas::new(terminal_size);
  window_content.draw(&mut canvas);
  canvas
}

pub fn assert_canvas(actual: &Canvas, expect: &[&str]) {
  let actual = actual
    .frame()
    .raw_symbols()
    .iter()
    .map(|cs| cs.join(""))
    .collect::<Vec<_>>();
  info!("actual:{}", actual.len());
  for a in actual.iter() {
    info!("{:?}", a);
  }
  info!("expect:{}", expect.len());
  for e in expect.iter() {
    info!("{:?}", e);
  }

  assert_eq!(actual.len(), expect.len());
  for i in 0..actual.len() {
    let e = &expect[i];
    let a = &actual[i];
    info!("i-{}, actual[{}]:{:?}, expect[{}]:{:?}", i, i, a, i, e);
    assert_eq!(e.len(), a.len());
    assert_eq!(e, a);
  }
}

#[cfg(test)]
mod tests_cursor_move {
  use super::*;

  use crate::prelude::*;
  use crate::state::{State, StateArc};
  use crate::test::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::test::log::init as test_log_init;
  use crate::test::tree::make_tree_with_buffers;
  use crate::ui::tree::TreeArc;
  use crate::ui::viewport::{
    CursorViewport, CursorViewportArc, Viewport, ViewportArc, ViewportSearchDirection,
  };
  use crate::ui::widget::window::{WindowLocalOptions, WindowLocalOptionsBuilder};

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
    let (tree, state, bufs, buf, contents) = make_tree(
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
    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents.clone(),
      Event::Key(key_event),
    );
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
      assert_eq!(actual2.row_idx(), 3);
      assert_eq!(actual2.column_idx(), 9);

      let viewport = get_viewport(tree.clone());
      let expect = vec!["", "", "", "endering.\n", "", "", "ut in the ", ""];
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
      "AAAAAAAAAA\n",
      "1st.\n",
      "2nd.\n",
      "3rd.\n",
      "4th.\n",
      "5th.\n",
      "6th.\n",
      "BBBBBBBBBBCCCCCCCCCC\n",
      "8th.\n",
      "9th.\n",
      "10th.\n",
      "11th.\n",
      "12th.\n",
    ];
    let (tree, state, bufs, buf, contents) = make_tree(
      U16Size::new(10, 6),
      WindowLocalOptionsBuilder::default()
        .wrap(true)
        .line_break(false)
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
    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = InsertStateful::default();

    // Move-1
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveBy((3, 2)));
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 2);
      assert_eq!(actual1.char_idx(), 3);
      assert_eq!(actual1.row_idx(), 2);
      assert_eq!(actual1.column_idx(), 3);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "AAAAAAAAAA",
        "1st.\n",
        "2nd.\n",
        "3rd.\n",
        "4th.\n",
        "5th.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0)]
          .into_iter()
          .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        6,
        &expect_fills,
        &expect_fills,
      );
    }

    // Move-2
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(2));
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 2);
      assert_eq!(actual2.char_idx(), 4);
      assert_eq!(actual2.row_idx(), 2);
      assert_eq!(actual2.column_idx(), 4);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "AAAAAAAAAA",
        "1st.\n",
        "2nd.\n",
        "3rd.\n",
        "4th.\n",
        "5th.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0)]
          .into_iter()
          .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        6,
        &expect_fills,
        &expect_fills,
      );
    }

    // Move-3
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveTo((10, 0)));
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 0);
      assert_eq!(actual2.char_idx(), 10);
      assert_eq!(actual2.row_idx(), 1);
      assert_eq!(actual2.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "AAAAAAAAAA",
        "1st.\n",
        "2nd.\n",
        "3rd.\n",
        "4th.\n",
        "5th.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0)]
          .into_iter()
          .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        6,
        &expect_fills,
        &expect_fills,
      );
    }

    // Move-4
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveTo((0, 7)));
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 7);
      assert_eq!(actual2.char_idx(), 0);
      assert_eq!(actual2.row_idx(), 4);
      assert_eq!(actual2.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "3rd.\n",
        "4th.\n",
        "5th.\n",
        "6th.\n",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
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

    // Move-5
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(13));
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 7);
      assert_eq!(actual2.char_idx(), 13);
      assert_eq!(actual2.row_idx(), 5);
      assert_eq!(actual2.column_idx(), 3);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "3rd.\n",
        "4th.\n",
        "5th.\n",
        "6th.\n",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
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

    // Move-6
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(7));
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 7);
      assert_eq!(actual2.char_idx(), 20);
      assert_eq!(actual2.row_idx(), 5);
      assert_eq!(actual2.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "4th.\n",
        "5th.\n",
        "6th.\n",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
        "8th.\n",
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
  fn wrap_linebreak1() {
    test_log_init();

    let lines = vec![
      "AAAAAAAAAA\n",
      "1st.\n",
      "2nd.\n",
      "3rd.\n",
      "4th.\n",
      "5th.\n",
      "6th.\n",
      "BBBBBBBBBBCCCCCCCCCC\n",
      "8th.\n",
      "9th.\n",
      "10th.\n",
      "11th.\n",
      "12th.\n",
    ];
    let (tree, state, bufs, buf, contents) = make_tree(
      U16Size::new(10, 6),
      WindowLocalOptionsBuilder::default()
        .wrap(true)
        .line_break(true)
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
    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = InsertStateful::default();

    // Move-1
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveBy((3, 2)));
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 2);
      assert_eq!(actual1.char_idx(), 3);
      assert_eq!(actual1.row_idx(), 2);
      assert_eq!(actual1.column_idx(), 3);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "AAAAAAAAAA",
        "1st.\n",
        "2nd.\n",
        "3rd.\n",
        "4th.\n",
        "5th.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0)]
          .into_iter()
          .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        6,
        &expect_fills,
        &expect_fills,
      );
    }

    // Move-2
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(2));
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 2);
      assert_eq!(actual2.char_idx(), 4);
      assert_eq!(actual2.row_idx(), 2);
      assert_eq!(actual2.column_idx(), 4);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "AAAAAAAAAA",
        "1st.\n",
        "2nd.\n",
        "3rd.\n",
        "4th.\n",
        "5th.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0)]
          .into_iter()
          .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        6,
        &expect_fills,
        &expect_fills,
      );
    }

    // Move-3
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveTo((10, 0)));
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 0);
      assert_eq!(actual2.char_idx(), 10);
      assert_eq!(actual2.row_idx(), 1);
      assert_eq!(actual2.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "AAAAAAAAAA",
        "1st.\n",
        "2nd.\n",
        "3rd.\n",
        "4th.\n",
        "5th.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0)]
          .into_iter()
          .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        6,
        &expect_fills,
        &expect_fills,
      );
    }

    // Move-4
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveTo((0, 7)));
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 7);
      assert_eq!(actual2.char_idx(), 0);
      assert_eq!(actual2.row_idx(), 4);
      assert_eq!(actual2.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "3rd.\n",
        "4th.\n",
        "5th.\n",
        "6th.\n",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
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

    // Move-5
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(13));
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 7);
      assert_eq!(actual2.char_idx(), 13);
      assert_eq!(actual2.row_idx(), 5);
      assert_eq!(actual2.column_idx(), 3);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "3rd.\n",
        "4th.\n",
        "5th.\n",
        "6th.\n",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
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

    // Move-6
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(7));
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 7);
      assert_eq!(actual2.char_idx(), 20);
      assert_eq!(actual2.row_idx(), 5);
      assert_eq!(actual2.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "4th.\n",
        "5th.\n",
        "6th.\n",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
        "8th.\n",
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
}

#[cfg(test)]
mod tests_insert_text {
  use super::*;

  use crate::prelude::*;
  use crate::state::{State, StateArc};
  use crate::test::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::test::log::init as test_log_init;
  use crate::test::tree::make_tree_with_buffers;
  use crate::ui::tree::TreeArc;
  use crate::ui::viewport::{
    CursorViewport, CursorViewportArc, Viewport, ViewportArc, ViewportSearchDirection,
  };
  use crate::ui::widget::window::{WindowLocalOptions, WindowLocalOptionsBuilder};

  use compact_str::CompactString;
  use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
  use jiff::fmt::friendly::Designator::Compact;
  use std::collections::BTreeMap;
  use tracing::info;

  #[test]
  fn nowrap1() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let window_options = WindowLocalOptionsBuilder::default()
      .wrap(false)
      .build()
      .unwrap();
    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let (tree, state, bufs, buf, contents) = make_tree(terminal_size, window_options, lines);

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );
    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = InsertStateful::default();

    // Insert-1
    {
      stateful.cursor_insert(&data_access, CompactString::new("Bye, "));

      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 5);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 5);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Bye, Hello",
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

      let expect_canvas = vec![
        "Bye, Hello",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "     * The",
        "     * The",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Move-2
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(20));

      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 18);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 9);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "o, RSVIM!\n",
        " quite sim",
        " it contai",
        " the line ",
        " the line ",
        "e extra pa",
        "e extra pa",
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

      let expect_canvas = vec![
        "o, RSVIM! ",
        " quite sim",
        " it contai",
        " the line ",
        " the line ",
        "e extra pa",
        "e extra pa",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Insert-3
    {
      stateful.cursor_insert(&data_access, CompactString::new(" Go!"));

      let tree = data_access.tree.clone();
      let actual3 = get_cursor_viewport(tree.clone());
      assert_eq!(actual3.line_idx(), 0);
      assert_eq!(actual3.char_idx(), 22);
      assert_eq!(actual3.row_idx(), 0);
      assert_eq!(actual3.column_idx(), 9);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "SVIM! Go!\n",
        "te simple ",
        "contains s",
        " line is s",
        " line is t",
        "tra parts ",
        "tra parts ",
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

      let expect_canvas = vec![
        "SVIM! Go! ",
        "te simple ",
        "contains s",
        " line is s",
        " line is t",
        "tra parts ",
        "tra parts ",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }
  }

  #[test]
  fn nowrap2() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let window_option = WindowLocalOptionsBuilder::default()
      .wrap(false)
      .build()
      .unwrap();
    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside.\n",
      "  2. When the line is too long to be completely put in.\n",
      "  3. Is there any other cases?\n",
    ];
    let (tree, state, bufs, buf, contents) = make_tree(terminal_size, window_option, lines);

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );
    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = InsertStateful::default();

    // Move-1
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveBy((100, 6)));

      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 5);
      assert_eq!(actual1.char_idx(), 30);
      assert_eq!(actual1.row_idx(), 5);
      assert_eq!(actual1.column_idx(), 9);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "",
        "e and smal",
        " several t",
        " small eno",
        " too long ",
        "er cases?\n",
        "",
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

      let expect_canvas = vec![
        "          ",
        "e and smal",
        " several t",
        " small eno",
        " too long ",
        "er cases? ",
        "          ",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_option, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Insert-2
    {
      stateful.cursor_insert(&data_access, CompactString::new("a"));

      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 5);
      assert_eq!(actual2.char_idx(), 31);
      assert_eq!(actual2.row_idx(), 5);
      assert_eq!(actual2.column_idx(), 9);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "",
        " and small",
        "several th",
        "small enou",
        "too long t",
        "r cases?a\n",
        "",
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

      let expect_canvas = vec![
        "          ",
        " and small",
        "several th",
        "small enou",
        "too long t",
        "r cases?a ",
        "          ",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_option, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }
  }

  #[test]
  fn nowrap3() {
    test_log_init();

    let terminal_size = U16Size::new(10, 5);
    let window_option = WindowLocalOptionsBuilder::default()
      .wrap(false)
      .build()
      .unwrap();
    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside.\n",
      "  2. When the line is too long to be completely put in.\n",
      "  3. Is there any other cases?\n",
    ];
    let (tree, state, bufs, buf, contents) = make_tree(terminal_size, window_option, lines);

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );
    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = InsertStateful::default();

    // Move-1
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(3));

      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 3);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 3);

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

      let expect_canvas = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_option, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Insert-2
    {
      let buf_eol = lock!(buf).options().end_of_line();
      let text2 = CompactString::new(format!(
        "Let's{buf_eol}insert{buf_eol}multiple lines!{buf_eol}"
      ));
      stateful.cursor_insert(&data_access, text2);

      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 3);
      assert_eq!(actual2.char_idx(), 0);
      assert_eq!(actual2.row_idx(), 3);
      assert_eq!(actual2.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let l0 = format!("HelLet's{buf_eol}");
      let l1 = format!("insert{buf_eol}");
      let expect = vec![
        l0.as_str(),
        l1.as_str(),
        "multiple l",
        "lo, RSVIM!",
        "This is a ",
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

      let expect_canvas = vec![
        "HelLet's  ",
        "insert    ",
        "multiple l",
        "lo, RSVIM!",
        "This is a ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_option, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Insert-3
    {
      let buf_eol = lock!(buf).options().end_of_line();
      let text2 = CompactString::new(format!(
        "Insert two lines again!{buf_eol}There's no line-break"
      ));
      stateful.cursor_insert(&data_access, text2);

      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 4);
      assert_eq!(actual2.char_idx(), 21);
      assert_eq!(actual2.row_idx(), 4);
      assert_eq!(actual2.column_idx(), 9);

      let viewport = get_viewport(tree.clone());
      let l2 = format!("es!{buf_eol}");
      let expect = vec!["", "", l2.as_str(), "ines again", "ine-breakl"];
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

      let expect_canvas = vec![
        "          ",
        "          ",
        "es!       ",
        "ines again",
        "ine-breakl",
      ];
      let actual_canvas = make_canvas(terminal_size, window_option, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Move-4
    {
      // let buf_eol = lock!(buf).options().end_of_line();
      stateful.cursor_move(&data_access, Operation::CursorMoveBy((100, 6)));

      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 9);
      assert_eq!(actual1.char_idx(), 30);
      assert_eq!(actual1.row_idx(), 4);
      assert_eq!(actual1.column_idx(), 9);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "e and smal",
        " several t",
        " small eno",
        " too long ",
        "er cases?\n",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(5, 0), (6, 0), (7, 0), (8, 0), (9, 0)]
        .into_iter()
        .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        5,
        10,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "e and smal",
        " several t",
        " small eno",
        " too long ",
        "er cases? ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_option, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Insert-5
    {
      let buf_eol = lock!(buf).options().end_of_line();
      let text5 = CompactString::new(format!(
        "Final 3 lines.{buf_eol}The inserted 2nd{buf_eol}The inserted 3rd{buf_eol}"
      ));
      stateful.cursor_insert(&data_access, text5);

      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 12);
      assert_eq!(actual1.char_idx(), 0);
      assert_eq!(actual1.row_idx(), 4);
      assert_eq!(actual1.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec!["  2. When ", "  3. Is th", "The insert", "The insert", "\n"];
      let expect_fills: BTreeMap<usize, usize> = vec![(8, 0), (9, 0), (10, 0), (11, 0), (12, 0)]
        .into_iter()
        .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        8,
        13,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "  2. When ",
        "  3. Is th",
        "The insert",
        "The insert",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_option, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Insert-6
    {
      stateful.cursor_insert(&data_access, "Insert 4th".to_compact_string());

      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 12);
      assert_eq!(actual1.char_idx(), 10);
      assert_eq!(actual1.row_idx(), 4);
      assert_eq!(actual1.column_idx(), 9);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        " 2. When t",
        " 3. Is the",
        "he inserte",
        "he inserte",
        "nsert 4th\n",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(8, 0), (9, 0), (10, 0), (11, 0), (12, 0)]
        .into_iter()
        .collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        8,
        13,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        " 2. When t",
        " 3. Is the",
        "he inserte",
        "he inserte",
        "nsert 4th ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_option, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }
  }

  #[test]
  fn nowrap4() {
    test_log_init();

    let terminal_size = U16Size::new(10, 5);
    let window_option = WindowLocalOptionsBuilder::default()
      .wrap(false)
      .build()
      .unwrap();
    let lines = vec![];
    let (tree, state, bufs, buf, contents) = make_tree(terminal_size, window_option, lines);

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );
    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = InsertStateful::default();

    // Insert-1
    {
      stateful.cursor_insert(&data_access, "Hi".to_compact_string());

      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 0);
      assert_eq!(actual2.char_idx(), 2);
      assert_eq!(actual2.row_idx(), 0);
      assert_eq!(actual2.column_idx(), 2);

      let viewport = get_viewport(tree.clone());
      let buf_eol = lock!(buf).options().end_of_line();
      let l0 = format!("Hi{buf_eol}");
      let expect = vec![l0.as_str(), ""];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        2,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "Hi        ",
        "          ",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_option, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }
  }

  #[test]
  fn wrap_nolinebreak1() {
    test_log_init();

    let terminal_size = U16Size::new(10, 6);
    let window_options = WindowLocalOptionsBuilder::default()
      .wrap(true)
      .line_break(false)
      .build()
      .unwrap();
    let lines = vec![
      "AAAAAAAAAA\n",
      "1st.\n",
      "2nd.\n",
      "3rd.\n",
      "4th.\n",
      "5th.\n",
      "6th.\n",
      "BBBBBBBBBBCCCCCCCCCC\n",
      "8th.\n",
      "9th.\n",
      "10th.\n",
      "11th.\n",
      "12th.\n",
    ];
    let (tree, state, bufs, buf, contents) = make_tree(terminal_size, window_options, lines);

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );
    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = InsertStateful::default();

    // Insert-1
    {
      stateful.cursor_insert(&data_access, CompactString::new("Hello, "));
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 7);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 7);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, AAA",
        "AAAAAAA\n",
        "1st.\n",
        "2nd.\n",
        "3rd.\n",
        "4th.\n",
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

      let expect_canvas = vec![
        "Hello, AAA",
        "AAAAAAA   ",
        "1st.      ",
        "2nd.      ",
        "3rd.      ",
        "4th.      ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Move-2
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveBy((3, 2)));
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 2);
      assert_eq!(actual1.char_idx(), 4);
      assert_eq!(actual1.row_idx(), 3);
      assert_eq!(actual1.column_idx(), 4);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, AAA",
        "AAAAAAA\n",
        "1st.\n",
        "2nd.\n",
        "3rd.\n",
        "4th.\n",
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

      let expect_canvas = vec![
        "Hello, AAA",
        "AAAAAAA   ",
        "1st.      ",
        "2nd.      ",
        "3rd.      ",
        "4th.      ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Insert-3
    {
      stateful.cursor_insert(&data_access, CompactString::new("World!"));
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 2);
      assert_eq!(actual1.char_idx(), 10);
      assert_eq!(actual1.row_idx(), 4);
      assert_eq!(actual1.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, AAA",
        "AAAAAAA\n",
        "1st.\n",
        "2nd.World!",
        "3rd.\n",
        "4th.\n",
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

      let expect_canvas = vec![
        "Hello, AAA",
        "AAAAAAA   ",
        "1st.      ",
        "2nd.World!",
        "3rd.      ",
        "4th.      ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Insert-4
    {
      stateful.cursor_insert(&data_access, CompactString::new("Go!"));
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 2);
      assert_eq!(actual1.char_idx(), 13);
      assert_eq!(actual1.row_idx(), 4);
      assert_eq!(actual1.column_idx(), 3);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, AAA",
        "AAAAAAA\n",
        "1st.\n",
        "2nd.World!",
        "Go!\n",
        "3rd.\n",
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

      let expect_canvas = vec![
        "Hello, AAA",
        "AAAAAAA   ",
        "1st.      ",
        "2nd.World!",
        "Go!       ",
        "3rd.      ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Move-5
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveTo((20, 7)));
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 7);
      assert_eq!(actual2.char_idx(), 20);
      assert_eq!(actual2.row_idx(), 5);
      assert_eq!(actual2.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "4th.\n",
        "5th.\n",
        "6th.\n",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
        "8th.\n",
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

      let expect_canvas = vec![
        "4th.      ",
        "5th.      ",
        "6th.      ",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
        "8th.      ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Insert-6
    {
      stateful.cursor_insert(&data_access, CompactString::new("DDDDDDDDDD"));
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 7);
      assert_eq!(actual1.char_idx(), 30);
      assert_eq!(actual1.row_idx(), 5);
      assert_eq!(actual1.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "5th.\n",
        "6th.\n",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
        "DDDDDDDDDD",
        "8th.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(5, 0), (6, 0), (7, 0), (8, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        5,
        9,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "5th.      ",
        "6th.      ",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
        "DDDDDDDDDD",
        "8th.      ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Move-7
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveLeftBy(17));
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 7);
      assert_eq!(actual2.char_idx(), 13);
      assert_eq!(actual2.row_idx(), 3);
      assert_eq!(actual2.column_idx(), 3);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "5th.\n",
        "6th.\n",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
        "DDDDDDDDDD",
        "8th.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(5, 0), (6, 0), (7, 0), (8, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        5,
        9,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "5th.      ",
        "6th.      ",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
        "DDDDDDDDDD",
        "8th.      ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Insert-8
    {
      stateful.cursor_insert(&data_access, CompactString::new("abc"));
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 7);
      assert_eq!(actual1.char_idx(), 16);
      assert_eq!(actual1.row_idx(), 3);
      assert_eq!(actual1.column_idx(), 6);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "5th.\n",
        "6th.\n",
        "BBBBBBBBBB",
        "CCCabcCCCC",
        "CCCDDDDDDD",
        "DDD\n",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(5, 0), (6, 0), (7, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        5,
        8,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "5th.      ",
        "6th.      ",
        "BBBBBBBBBB",
        "CCCabcCCCC",
        "CCCDDDDDDD",
        "DDD       ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }
  }

  #[test]
  fn wrap_nolinebreak2() {
    test_log_init();

    let terminal_size = U16Size::new(10, 6);
    let window_options = WindowLocalOptionsBuilder::default()
      .wrap(true)
      .line_break(false)
      .build()
      .unwrap();
    let lines = vec![];
    let (tree, state, bufs, buf, contents) = make_tree(terminal_size, window_options, lines);

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );
    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = InsertStateful::default();

    // Insert-1
    {
      stateful.cursor_insert(&data_access, CompactString::new("a"));
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 1);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 1);

      let viewport = get_viewport(tree.clone());
      let buf_eol = lock!(buf.clone()).options().end_of_line();
      let a = format!("a{buf_eol}");
      let expect = vec![a.as_str(), ""];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        2,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "a         ",
        "          ",
        "          ",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }
  }

  #[test]
  fn wrap_nolinebreak3() {
    test_log_init();

    let terminal_size = U16Size::new(10, 6);
    let window_options = WindowLocalOptionsBuilder::default()
      .wrap(true)
      .line_break(false)
      .build()
      .unwrap();
    let lines = vec![""];
    let (tree, state, bufs, buf, contents) = make_tree(terminal_size, window_options, lines);

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );
    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = InsertStateful::default();

    // Insert-1
    {
      stateful.cursor_insert(&data_access, CompactString::new("b"));
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 1);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 1);

      let viewport = get_viewport(tree.clone());
      let buf_eol = lock!(buf.clone()).options().end_of_line();
      let b = format!("b{buf_eol}");
      let expect = vec![b.as_str(), ""];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        2,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "b         ",
        "          ",
        "          ",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }
  }

  #[test]
  fn wrap_nolinebreak4() {
    test_log_init();

    let terminal_size = U16Size::new(10, 6);
    let window_options = WindowLocalOptionsBuilder::default()
      .wrap(true)
      .line_break(false)
      .build()
      .unwrap();
    let lines = vec![""];
    let (tree, state, bufs, buf, contents) = make_tree(terminal_size, window_options, lines);

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );
    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = InsertStateful::default();

    // Insert-1
    {
      stateful.cursor_insert(&data_access, CompactString::new("这个"));
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 2);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 4);

      let viewport = get_viewport(tree.clone());
      let buf_eol = lock!(buf.clone()).options().end_of_line();
      let b = format!("这个{buf_eol}");
      let expect = vec![b.as_str(), ""];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        2,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "这个      ",
        "          ",
        "          ",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }
  }

  #[test]
  fn wrap_linebreak1() {
    test_log_init();

    let terminal_size = U16Size::new(10, 6);
    let window_options = WindowLocalOptionsBuilder::default()
      .wrap(true)
      .line_break(true)
      .build()
      .unwrap();
    let lines = vec![
      "AAAAAAAAAA\n",
      "1st line that we could make it longer.\n",
      "2nd line that we must make it shorter!\n",
      "3rd.\n",
      "4th.\n",
      "5th.\n",
      "6th.\n",
      "BBBBBBBBBBCCCCCCCCCC\n",
      "8th.\n",
      "9th.\n",
      "10th.\n",
    ];
    let (tree, state, bufs, buf, contents) = make_tree(terminal_size, window_options, lines);

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );
    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = InsertStateful::default();

    // Insert-1
    {
      stateful.cursor_insert(&data_access, CompactString::new("Hello, "));
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 7);
      assert_eq!(actual1.row_idx(), 1);
      assert_eq!(actual1.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, ",
        "AAAAAAAAAA",
        "1st line ",
        "that we ",
        "could make",
        " it longer",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        0,
        2,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "Hello,    ",
        "AAAAAAAAAA",
        "1st line  ",
        "that we   ",
        "could make",
        " it longer",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Move-2
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveBy((3, 2)));
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 2);
      assert_eq!(actual1.char_idx(), 10);
      assert_eq!(actual1.row_idx(), 1);
      assert_eq!(actual1.column_idx(), 1);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "2nd line ",
        "that we ",
        "must make ",
        "it shorter",
        "!\n",
        "3rd.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        2,
        4,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "2nd line  ",
        "that we   ",
        "must make ",
        "it shorter",
        "!         ",
        "3rd.      ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Insert-3
    {
      stateful.cursor_insert(&data_access, CompactString::new("World!"));
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 2);
      assert_eq!(actual1.char_idx(), 16);
      assert_eq!(actual1.row_idx(), 1);
      assert_eq!(actual1.column_idx(), 7);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "2nd line ",
        "tWorld!hat",
        " we must ",
        "make it ",
        "shorter!\n",
        "3rd.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        2,
        4,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "2nd line  ",
        "tWorld!hat",
        " we must  ",
        "make it   ",
        "shorter!  ",
        "3rd.      ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Insert-4
    {
      stateful.cursor_insert(&data_access, CompactString::new("Let's go further!"));
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 2);
      assert_eq!(actual1.char_idx(), 33);
      assert_eq!(actual1.row_idx(), 4);
      assert_eq!(actual1.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "2nd line ",
        "tWorld!",
        "Let's go ",
        "further!",
        "hat we ",
        "must make ",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(2, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        2,
        3,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "2nd line  ",
        "tWorld!   ",
        "Let's go  ",
        "further!  ",
        "hat we    ",
        "must make ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Move-5
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveTo((20, 7)));
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 7);
      assert_eq!(actual2.char_idx(), 20);
      assert_eq!(actual2.row_idx(), 5);
      assert_eq!(actual2.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "4th.\n",
        "5th.\n",
        "6th.\n",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
        "8th.\n",
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

      let expect_canvas = vec![
        "4th.      ",
        "5th.      ",
        "6th.      ",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
        "8th.      ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Insert-6
    {
      stateful.cursor_insert(&data_access, CompactString::new("DDDDDDDDDD"));
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 7);
      assert_eq!(actual1.char_idx(), 30);
      assert_eq!(actual1.row_idx(), 5);
      assert_eq!(actual1.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "5th.\n",
        "6th.\n",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
        "DDDDDDDDDD",
        "8th.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(5, 0), (6, 0), (7, 0), (8, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        5,
        9,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "5th.      ",
        "6th.      ",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
        "DDDDDDDDDD",
        "8th.      ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Move-7
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveLeftBy(17));
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 7);
      assert_eq!(actual2.char_idx(), 13);
      assert_eq!(actual2.row_idx(), 3);
      assert_eq!(actual2.column_idx(), 3);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "5th.\n",
        "6th.\n",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
        "DDDDDDDDDD",
        "8th.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(5, 0), (6, 0), (7, 0), (8, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        5,
        9,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "5th.      ",
        "6th.      ",
        "BBBBBBBBBB",
        "CCCCCCCCCC",
        "DDDDDDDDDD",
        "8th.      ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Insert-8
    {
      stateful.cursor_insert(&data_access, CompactString::new("abc"));
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 7);
      assert_eq!(actual1.char_idx(), 16);
      assert_eq!(actual1.row_idx(), 3);
      assert_eq!(actual1.column_idx(), 6);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "5th.\n",
        "6th.\n",
        "BBBBBBBBBB",
        "CCCabcCCCC",
        "CCCDDDDDDD",
        "DDD\n",
      ];
      let expect_fills: BTreeMap<usize, usize> = vec![(5, 0), (6, 0), (7, 0)].into_iter().collect();
      assert_viewport_scroll(
        buf.clone(),
        &viewport,
        &expect,
        5,
        8,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "5th.      ",
        "6th.      ",
        "BBBBBBBBBB",
        "CCCabcCCCC",
        "CCCDDDDDDD",
        "DDD       ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }
  }
}

#[cfg(test)]
mod tests_delete_text {
  use super::*;

  use crate::prelude::*;
  use crate::state::{State, StateArc};
  use crate::test::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::test::log::init as test_log_init;
  use crate::test::tree::make_tree_with_buffers;
  use crate::ui::tree::TreeArc;
  use crate::ui::viewport::{
    CursorViewport, CursorViewportArc, Viewport, ViewportArc, ViewportSearchDirection,
  };
  use crate::ui::widget::window::{WindowLocalOptions, WindowLocalOptionsBuilder};

  use compact_str::CompactString;
  use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
  use jiff::fmt::friendly::Designator::Compact;
  use std::collections::BTreeMap;
  use tracing::info;

  #[test]
  fn nowrap1() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let window_options = WindowLocalOptionsBuilder::default()
      .wrap(false)
      .build()
      .unwrap();
    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "* The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "* The extra.\n",
    ];
    let (tree, state, bufs, buf, contents) = make_tree(terminal_size, window_options, lines);

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );
    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = InsertStateful::default();

    // Delete-1
    {
      stateful.cursor_delete(&data_access, -1);

      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 0);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "* The extr",
        "* The extr",
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

      let expect_canvas = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "* The extr",
        "* The extr",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Move-2
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(7));

      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 7);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 7);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "* The extr",
        "* The extr",
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

      let expect_canvas = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "* The extr",
        "* The extr",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Delete-3
    {
      stateful.cursor_delete(&data_access, -5);

      let tree = data_access.tree.clone();
      let actual3 = get_cursor_viewport(tree.clone());
      assert_eq!(actual3.line_idx(), 0);
      assert_eq!(actual3.char_idx(), 2);
      assert_eq!(actual3.row_idx(), 0);
      assert_eq!(actual3.column_idx(), 2);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "HeRSVIM!\n",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "* The extr",
        "* The extr",
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

      let expect_canvas = vec![
        "HeRSVIM!  ",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "* The extr",
        "* The extr",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Move-4
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(3));

      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 3);
      assert_eq!(actual1.char_idx(), 2);
      assert_eq!(actual1.row_idx(), 3);
      assert_eq!(actual1.column_idx(), 2);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "HeRSVIM!\n",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "* The extr",
        "* The extr",
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

      let expect_canvas = vec![
        "HeRSVIM!  ",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "* The extr",
        "* The extr",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Delete-5
    {
      stateful.cursor_delete(&data_access, -50);

      let tree = data_access.tree.clone();
      let actual3 = get_cursor_viewport(tree.clone());
      assert_eq!(actual3.line_idx(), 2);
      assert_eq!(actual3.char_idx(), 6);
      assert_eq!(actual3.row_idx(), 2);
      assert_eq!(actual3.column_idx(), 6);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "HeRSVIM!\n",
        "This is a ",
        "But st1. W",
        "  2. When ",
        "* The extr",
        "* The extr",
        "",
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

      let expect_canvas = vec![
        "HeRSVIM!  ",
        "This is a ",
        "But st1. W",
        "  2. When ",
        "* The extr",
        "* The extr",
        "          ",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Delete-6
    {
      stateful.cursor_delete(&data_access, 60);

      let tree = data_access.tree.clone();
      let actual3 = get_cursor_viewport(tree.clone());
      assert_eq!(actual3.line_idx(), 2);
      assert_eq!(actual3.char_idx(), 6);
      assert_eq!(actual3.row_idx(), 2);
      assert_eq!(actual3.column_idx(), 6);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "HeRSVIM!\n",
        "This is a ",
        "But strow ",
        "  2. When ",
        "* The extr",
        "* The extr",
        "",
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

      let expect_canvas = vec![
        "HeRSVIM!  ",
        "This is a ",
        "But strow ",
        "  2. When ",
        "* The extr",
        "* The extr",
        "          ",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Delete-7
    {
      stateful.cursor_delete(&data_access, -1);

      let tree = data_access.tree.clone();
      let actual3 = get_cursor_viewport(tree.clone());
      assert_eq!(actual3.line_idx(), 2);
      assert_eq!(actual3.char_idx(), 5);
      assert_eq!(actual3.row_idx(), 2);
      assert_eq!(actual3.column_idx(), 5);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "HeRSVIM!\n",
        "This is a ",
        "But srow o",
        "  2. When ",
        "* The extr",
        "* The extr",
        "",
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

      let expect_canvas = vec![
        "HeRSVIM!  ",
        "This is a ",
        "But srow o",
        "  2. When ",
        "* The extr",
        "* The extr",
        "          ",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Delete-8
    {
      stateful.cursor_delete(&data_access, 1);

      let tree = data_access.tree.clone();
      let actual3 = get_cursor_viewport(tree.clone());
      assert_eq!(actual3.line_idx(), 2);
      assert_eq!(actual3.char_idx(), 5);
      assert_eq!(actual3.row_idx(), 2);
      assert_eq!(actual3.column_idx(), 5);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "HeRSVIM!\n",
        "This is a ",
        "But sow of",
        "  2. When ",
        "* The extr",
        "* The extr",
        "",
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

      let expect_canvas = vec![
        "HeRSVIM!  ",
        "This is a ",
        "But sow of",
        "  2. When ",
        "* The extr",
        "* The extr",
        "          ",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Move-9
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveBy((500, 10)));

      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 5);
      assert_eq!(actual1.char_idx(), 12);
      assert_eq!(actual1.row_idx(), 5);
      assert_eq!(actual1.column_idx(), 9);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "SVIM!\n",
        "s is a qui",
        " sow of th",
        ". When the",
        "he extra p",
        "he extra.\n",
        "",
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

      let expect_canvas = vec![
        "SVIM!     ",
        "s is a qui",
        " sow of th",
        ". When the",
        "he extra p",
        "he extra. ",
        "          ",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Delete-10
    {
      stateful.cursor_delete(&data_access, 1);

      let tree = data_access.tree.clone();
      let actual3 = get_cursor_viewport(tree.clone());
      assert_eq!(actual3.line_idx(), 5);
      assert_eq!(actual3.char_idx(), 12);
      assert_eq!(actual3.row_idx(), 5);
      assert_eq!(actual3.column_idx(), 9);

      let viewport = get_viewport(tree.clone());
      let buf_eol = lock!(buf).options().end_of_line();
      let text5 = CompactString::new(format!("he extra.{buf_eol}"));
      let expect = vec![
        "SVIM!\n",
        "s is a qui",
        " sow of th",
        ". When the",
        "he extra p",
        text5.as_str(),
        "",
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

      let expect_canvas = vec![
        "SVIM!     ",
        "s is a qui",
        " sow of th",
        ". When the",
        "he extra p",
        "he extra. ",
        "          ",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Delete-11
    {
      stateful.cursor_delete(&data_access, -1);

      let tree = data_access.tree.clone();
      let actual3 = get_cursor_viewport(tree.clone());
      assert_eq!(actual3.line_idx(), 5);
      assert_eq!(actual3.char_idx(), 11);
      assert_eq!(actual3.row_idx(), 5);
      assert_eq!(actual3.column_idx(), 8);

      let viewport = get_viewport(tree.clone());
      let buf_eol = lock!(buf).options().end_of_line();
      let text5 = CompactString::new(format!("he extra{buf_eol}"));
      let expect = vec![
        "SVIM!\n",
        "s is a qui",
        " sow of th",
        ". When the",
        "he extra p",
        text5.as_str(),
        "",
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

      let expect_canvas = vec![
        "SVIM!     ",
        "s is a qui",
        " sow of th",
        ". When the",
        "he extra p",
        "he extra  ",
        "          ",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }
  }

  #[test]
  fn nowrap2() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let window_options = WindowLocalOptionsBuilder::default()
      .wrap(false)
      .build()
      .unwrap();
    let lines = vec![];
    let (tree, state, bufs, buf, contents) = make_tree(terminal_size, window_options, lines);

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );
    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = InsertStateful::default();

    // Delete-1
    {
      stateful.cursor_delete(&data_access, -1);

      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 0);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 0);

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

      let expect_canvas = vec![
        "          ",
        "          ",
        "          ",
        "          ",
        "          ",
        "          ",
        "          ",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }
  }

  #[test]
  fn nowrap3() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let window_options = WindowLocalOptionsBuilder::default()
      .wrap(false)
      .build()
      .unwrap();
    let lines = vec![];
    let (tree, state, bufs, buf, contents) = make_tree(terminal_size, window_options, lines);

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );
    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = InsertStateful::default();

    // Delete-1
    {
      stateful.cursor_delete(&data_access, 1);

      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 0);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 0);

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

      let expect_canvas = vec![
        "          ",
        "          ",
        "          ",
        "          ",
        "          ",
        "          ",
        "          ",
        "          ",
        "          ",
        "          ",
      ];
      let actual_canvas = make_canvas(terminal_size, window_options, buf.clone(), viewport);
      assert_canvas(&actual_canvas, &expect_canvas);
    }
  }
}
