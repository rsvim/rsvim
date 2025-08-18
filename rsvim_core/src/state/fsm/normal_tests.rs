#![allow(unused_imports)]

use super::normal::*;

use crate::buf::opt::BufferLocalOptionsBuilder;
use crate::buf::opt::{BufferLocalOptions, BufferLocalOptionsBuilder};
use crate::buf::{BufferArc, BuffersManagerArc};
use crate::buf::{BufferArc, BuffersManagerArc};
use crate::command::{ExCommandsManager, ExCommandsManagerArc};
use crate::content::TextContents;
use crate::content::{TextContents, TextContentsArc};
use crate::prelude::*;
use crate::prelude::*;
use crate::state::State;
use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::ops::Operation;
use crate::state::ops::cursor_ops;
use crate::state::{State, StateArc};
use crate::tests::buf::{make_buffer_from_lines, make_buffers_manager};
use crate::tests::buf::{make_buffer_from_lines, make_buffers_manager};
use crate::tests::log::init as test_log_init;
use crate::tests::log::init as test_log_init;
use crate::tests::tree::make_tree_with_buffers;
use crate::tests::tree::{
  make_tree_with_buffers, make_tree_with_buffers_cmdline,
};
use crate::ui::canvas::{Canvas, CanvasArc};
use crate::ui::tree::TreeArc;
use crate::ui::viewport::{
  CursorViewport, CursorViewportArc, Viewport, ViewportArc,
  ViewportSearchDirection,
};
use crate::ui::widget::command_line::CommandLine;
use crate::ui::widget::window::WindowLocalOptionsBuilder;
use crate::ui::widget::window::{
  WindowLocalOptions, WindowLocalOptionsBuilder,
};

use compact_str::CompactString;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::collections::BTreeMap;
use tokio::sync::mpsc::{Receiver, Sender, channel};

pub fn make_tree_with_buffer_opts(
  terminal_size: U16Size,
  buffer_local_opts: BufferLocalOptions,
  window_local_opts: WindowLocalOptions,
  lines: Vec<&str>,
) -> (
  TreeArc,
  StateArc,
  BuffersManagerArc,
  BufferArc,
  TextContentsArc,
) {
  let buf = make_buffer_from_lines(terminal_size, buffer_local_opts, lines);
  let bufs = make_buffers_manager(buffer_local_opts, vec![buf.clone()]);
  let tree =
    make_tree_with_buffers(terminal_size, window_local_opts, bufs.clone());
  let state = State::to_arc(State::new());
  let contents = TextContents::to_arc(TextContents::new(terminal_size));
  (tree, state, bufs, buf, contents)
}

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
  make_tree_with_buffer_opts(terminal_size, buf_opts, window_local_opts, lines)
}

pub fn make_tree_with_cmdline(
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
  let contents = TextContents::to_arc(TextContents::new(terminal_size));
  let tree = make_tree_with_buffers_cmdline(
    terminal_size,
    window_local_opts,
    bufs.clone(),
    contents.clone(),
  );
  let state = State::to_arc(State::new());
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

pub fn make_canvas(tree: TreeArc, terminal_size: U16Size) -> CanvasArc {
  let canvas = Canvas::new(terminal_size);
  let canvas = Canvas::to_arc(canvas);
  let tree = lock!(tree);
  tree.draw(canvas.clone());
  canvas
}

#[allow(clippy::too_many_arguments)]
pub fn assert_viewport(
  buffer: BufferArc,
  actual: &Viewport,
  expect_rows: &Vec<&str>,
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
  for (i, e) in expect_rows.iter().enumerate() {
    info!("expect rows[{}]:{:?}", i, e);
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
    let (first_line_idx, _first_line_viewport) =
      actual.lines().first().unwrap();
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
  let buflines = buffer.text().rope().lines_at(actual.start_line_idx());
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
        r, payload, expect_rows[*r as usize]
      );
      assert_eq!(payload, expect_rows[*r as usize]);
    }
  }
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
mod tests_raw_cursor_move_y_by {
  use super::*;

  use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
  };
  use tokio::sync::mpsc::{Receiver, Sender, channel};

  #[test]
  fn nowrap1() {
    test_log_init();

    let (jsrt_tick_dispatcher, _jsrt_tick_queue) = channel(1);
    let commands = ExCommandsManager::to_arc(ExCommandsManager::new());
    let (tree, state, bufs, _buf, contents) = make_tree(
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

    let data_access = StatefulDataAccess::new(
      state,
      tree,
      bufs,
      contents,
      commands,
      jsrt_tick_dispatcher,
      Event::Key(key_event),
    );
    let stateful_machine = NormalStateful::default();
    stateful_machine
      ._test_raw_cursor_move(&data_access, Operation::CursorMoveUpBy(1));

    let tree = data_access.tree.clone();
    let actual = get_cursor_viewport(tree);
    assert_eq!(actual.line_idx(), 0);
    assert_eq!(actual.char_idx(), 0);
  }

  #[test]
  fn nowrap2() {
    test_log_init();

    let (jsrt_tick_dispatcher, _jsrt_tick_queue) = channel(1);
    let commands = ExCommandsManager::to_arc(ExCommandsManager::new());

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let (tree, state, bufs, _buf, contents) = make_tree(
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

    let data_access = StatefulDataAccess::new(
      state,
      tree,
      bufs,
      contents,
      commands,
      jsrt_tick_dispatcher,
      Event::Key(key_event),
    );
    let stateful_machine = NormalStateful::default();
    stateful_machine
      ._test_raw_cursor_move(&data_access, Operation::CursorMoveUpBy(1));

    let tree = data_access.tree.clone();
    let actual = get_cursor_viewport(tree);
    assert_eq!(actual.line_idx(), 0);
    assert_eq!(actual.char_idx(), 0);
  }

  #[test]
  fn nowrap3() {
    test_log_init();
    let (jsrt_tick_dispatcher, _jsrt_tick_queue) = channel(1);
    let commands = ExCommandsManager::to_arc(ExCommandsManager::new());

    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let (tree, state, bufs, _buf, contents) = make_tree(
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

    let data_access = StatefulDataAccess::new(
      state,
      tree,
      bufs,
      contents,
      commands,
      jsrt_tick_dispatcher,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_cursor_move(&data_access, Operation::CursorMoveDownBy(3));

    let tree = data_access.tree.clone();
    let actual1 = get_cursor_viewport(tree);
    assert_eq!(actual1.line_idx(), 3);
    assert_eq!(actual1.char_idx(), 0);

    stateful._test_raw_cursor_move(&data_access, Operation::CursorMoveUpBy(1));

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
    let (tree, state, bufs, _buf, contents) = make_tree(
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

    let (jsrt_tick_dispatcher, _jsrt_tick_queue) = channel(1);
    let commands = ExCommandsManager::to_arc(ExCommandsManager::new());
    let data_access = StatefulDataAccess::new(
      state,
      tree,
      bufs,
      contents,
      commands,
      jsrt_tick_dispatcher,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_cursor_move(&data_access, Operation::CursorMoveDownBy(2));

    let tree = data_access.tree.clone();
    let actual1 = get_cursor_viewport(tree);
    assert_eq!(actual1.line_idx(), 2);
    assert_eq!(actual1.char_idx(), 0);

    stateful._test_raw_cursor_move(&data_access, Operation::CursorMoveUpBy(1));

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
    let buf = make_buffer_from_lines(terminal_size, buf_opts, lines);
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let contents = TextContents::to_arc(TextContents::new(terminal_size));
    let tree = make_tree_with_buffers(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      bufs.clone(),
    );
    let (jsrt_tick_dispatcher, _jsrt_tick_queue) = channel(1);
    let state = State::to_arc(State::new(jsrt_tick_dispatcher));
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let data_access = StatefulDataAccess::new(
      state,
      tree,
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful_machine = NormalStateful::default();
    stateful_machine
      ._test_raw_cursor_move(&data_access, Operation::CursorMoveDownBy(1));

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
    let (tree, state, bufs, _buf, contents) = make_tree(
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

    let data_access = StatefulDataAccess::new(
      state,
      tree,
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_cursor_move(&data_access, Operation::CursorMoveDownBy(2));

    let tree = data_access.tree.clone();
    let actual1 = get_cursor_viewport(tree);
    assert_eq!(actual1.line_idx(), 2);
    assert_eq!(actual1.char_idx(), 0);

    stateful._test_raw_cursor_move(&data_access, Operation::CursorMoveUpBy(1));

    let tree = data_access.tree.clone();
    let actual2 = get_cursor_viewport(tree);
    assert_eq!(actual2.line_idx(), 1);
    assert_eq!(actual2.char_idx(), 0);
  }
}

#[cfg(test)]
mod tests_raw_cursor_move_x_by {
  use super::*;

  use crate::buf::opt::BufferLocalOptionsBuilder;
  use crate::buf::{BufferArc, BuffersManagerArc};
  use crate::content::TextContents;
  use crate::prelude::*;
  use crate::state::State;
  use crate::tests::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::tests::log::init as test_log_init;
  use crate::tests::tree::make_tree_with_buffers;
  use crate::ui::widget::window::WindowLocalOptionsBuilder;

  use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
  };
  use tokio::sync::mpsc::{Receiver, Sender, channel};

  #[test]
  fn nowrap1() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let lines = vec![];
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let buf = make_buffer_from_lines(terminal_size, buf_opts, lines);
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let contents = TextContents::to_arc(TextContents::new(terminal_size));
    let tree = make_tree_with_buffers(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      bufs.clone(),
    );
    let (jsrt_tick_dispatcher, _jsrt_tick_queue) = channel(1);
    let state = State::to_arc(State::new(jsrt_tick_dispatcher));
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let data_access = StatefulDataAccess::new(
      state,
      tree,
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_cursor_move(&data_access, Operation::CursorMoveRightBy(1));

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
    let buf = make_buffer_from_lines(terminal_size, buf_opts, lines);
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let contents = TextContents::to_arc(TextContents::new(terminal_size));
    let tree = make_tree_with_buffers(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      bufs.clone(),
    );
    let (jsrt_tick_dispatcher, _jsrt_tick_queue) = channel(1);
    let state = State::to_arc(State::new(jsrt_tick_dispatcher));
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let data_access = StatefulDataAccess::new(
      state,
      tree,
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_cursor_move(&data_access, Operation::CursorMoveRightBy(1));

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
    let buf = make_buffer_from_lines(terminal_size, buf_opts, lines);
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let contents = TextContents::to_arc(TextContents::new(terminal_size));
    let tree = make_tree_with_buffers(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      bufs.clone(),
    );
    let (jsrt_tick_dispatcher, _jsrt_tick_queue) = channel(1);
    let state = State::to_arc(State::new(jsrt_tick_dispatcher));
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let data_access = StatefulDataAccess::new(
      state,
      tree,
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_cursor_move(&data_access, Operation::CursorMoveRightBy(20));

    let tree = data_access.tree.clone();
    let actual = get_cursor_viewport(tree);
    assert_eq!(actual.line_idx(), 0);
    assert_eq!(actual.char_idx(), 12);
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
    let buf = make_buffer_from_lines(terminal_size, buf_opts, lines);
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let contents = TextContents::to_arc(TextContents::new(terminal_size));
    let tree = make_tree_with_buffers(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      bufs.clone(),
    );
    let (jsrt_tick_dispatcher, _jsrt_tick_queue) = channel(1);
    let state = State::to_arc(State::new(jsrt_tick_dispatcher));
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let data_access = StatefulDataAccess::new(
      state,
      tree,
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_cursor_move(&data_access, Operation::CursorMoveRightBy(5));

    let tree = data_access.tree.clone();
    let actual1 = get_cursor_viewport(tree);
    assert_eq!(actual1.line_idx(), 0);
    assert_eq!(actual1.char_idx(), 5);

    stateful
      ._test_raw_cursor_move(&data_access, Operation::CursorMoveLeftBy(3));

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
    let buf = make_buffer_from_lines(terminal_size, buf_opts, lines);
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let contents = TextContents::to_arc(TextContents::new(terminal_size));
    let tree = make_tree_with_buffers(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      bufs.clone(),
    );
    let (jsrt_tick_dispatcher, _jsrt_tick_queue) = channel(1);
    let state = State::to_arc(State::new(jsrt_tick_dispatcher));
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let data_access = StatefulDataAccess::new(
      state,
      tree,
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_cursor_move(&data_access, Operation::CursorMoveRightBy(5));

    let tree = data_access.tree.clone();
    let actual = get_cursor_viewport(tree);
    assert_eq!(actual.line_idx(), 0);
    assert_eq!(actual.char_idx(), 5);

    for i in (0..=4).rev() {
      stateful
        ._test_raw_cursor_move(&data_access, Operation::CursorMoveLeftBy(1));

      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree);
      assert_eq!(actual.line_idx(), 0);
      assert_eq!(actual.char_idx(), i);
    }
  }
}

#[cfg(test)]
mod tests_raw_cursor_move_by {
  use super::*;

  use crate::buf::opt::BufferLocalOptionsBuilder;
  use crate::buf::{BufferArc, BuffersManagerArc};
  use crate::content::TextContents;
  use crate::prelude::*;
  use crate::state::State;
  use crate::tests::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::tests::log::init as test_log_init;
  use crate::tests::tree::make_tree_with_buffers;
  use crate::ui::widget::window::WindowLocalOptionsBuilder;

  use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
  };
  use tokio::sync::mpsc::{Receiver, Sender, channel};

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
    let buf = make_buffer_from_lines(terminal_size, buf_opts, lines);
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let contents = TextContents::to_arc(TextContents::new(terminal_size));
    let tree = make_tree_with_buffers(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      bufs.clone(),
    );
    let (jsrt_tick_dispatcher, _jsrt_tick_queue) = channel(1);
    let state = State::to_arc(State::new(jsrt_tick_dispatcher));
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    // Step-1
    let data_access = StatefulDataAccess::new(
      state,
      tree,
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_cursor_move(&data_access, Operation::CursorMoveBy((5, 0)));

    let tree = data_access.tree.clone();
    let actual1 = get_cursor_viewport(tree);
    assert_eq!(actual1.line_idx(), 0);
    assert_eq!(actual1.char_idx(), 5);

    // Step-2
    stateful
      ._test_raw_cursor_move(&data_access, Operation::CursorMoveBy((0, 1)));
    let tree = data_access.tree.clone();
    let actual2 = get_cursor_viewport(tree);
    assert_eq!(actual2.line_idx(), 1);
    assert_eq!(actual2.char_idx(), 5);

    // Step-3
    stateful
      ._test_raw_cursor_move(&data_access, Operation::CursorMoveBy((-3, 0)));
    let tree = data_access.tree.clone();
    let actual3 = get_cursor_viewport(tree);
    assert_eq!(actual3.line_idx(), 1);
    assert_eq!(actual3.char_idx(), 2);

    // Step-4
    stateful
      ._test_raw_cursor_move(&data_access, Operation::CursorMoveBy((0, -1)));
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
    let buf = make_buffer_from_lines(terminal_size, buf_opts, lines);
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let contents = TextContents::to_arc(TextContents::new(terminal_size));
    let tree = make_tree_with_buffers(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      bufs.clone(),
    );
    let (jsrt_tick_dispatcher, _jsrt_tick_queue) = channel(1);
    let state = State::to_arc(State::new(jsrt_tick_dispatcher));
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
        Operation::CursorMoveBy((0, 2)),
        Operation::CursorMoveBy((3, 0)),
        Operation::CursorMoveBy((0, -2)),
        Operation::CursorMoveBy((-3, 0)),
      ];
      let data_access = StatefulDataAccess::new(
        state.clone(),
        tree.clone(),
        bufs.clone(),
        contents.clone(),
        Event::Key(key_event),
      );
      for c in commands.iter() {
        stateful._test_raw_cursor_move(&data_access, c.clone());
      }
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree);
      assert_eq!(actual.line_idx(), 0);
      assert_eq!(actual.char_idx(), 0);
    }

    for _ in 0..10 {
      let commands = [
        Operation::CursorMoveBy((5, 0)),
        Operation::CursorMoveBy((0, 1)),
        Operation::CursorMoveBy((-5, 0)),
        Operation::CursorMoveBy((0, -1)),
      ];
      let data_access = StatefulDataAccess::new(
        state.clone(),
        tree.clone(),
        bufs.clone(),
        contents.clone(),
        Event::Key(key_event),
      );
      for c in commands.iter() {
        stateful._test_raw_cursor_move(&data_access, c.clone());
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
    let buf = make_buffer_from_lines(terminal_size, buf_opts, lines.clone());
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let contents = TextContents::to_arc(TextContents::new(terminal_size));
    let tree = make_tree_with_buffers(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(true)
        .build()
        .unwrap(),
      bufs.clone(),
    );
    let (jsrt_tick_dispatcher, _jsrt_tick_queue) = channel(1);
    let state = State::to_arc(State::new(jsrt_tick_dispatcher));
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
      contents.clone(),
      Event::Key(key_event),
    );
    let command = Operation::CursorMoveBy((lines[0].len() as isize, 0));
    stateful._test_raw_cursor_move(&data_access, command);

    let tree = data_access.tree.clone();
    let actual1 = get_cursor_viewport(tree.clone());
    assert_eq!(actual1.line_idx(), 0);
    assert_eq!(actual1.char_idx(), 27);

    // step-2: Move down to line-2.
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree.clone(),
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let command = Operation::CursorMoveBy((0, 1));
    stateful._test_raw_cursor_move(&data_access, command);

    let tree = data_access.tree.clone();
    let actual2 = get_cursor_viewport(tree);
    assert_eq!(actual2.line_idx(), 1);
    assert_eq!(actual2.char_idx(), 18);
  }
}

#[cfg(test)]
mod tests_raw_cursor_move_to {
  use super::*;

  use crate::buf::opt::BufferLocalOptionsBuilder;
  use crate::buf::{BufferArc, BuffersManagerArc};
  use crate::content::TextContents;
  use crate::prelude::*;
  use crate::state::State;
  use crate::tests::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::tests::log::init as test_log_init;
  use crate::tests::tree::make_tree_with_buffers;
  use crate::ui::widget::window::WindowLocalOptionsBuilder;

  use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
  };
  use tokio::sync::mpsc::{Receiver, Sender, channel};

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
    let buf = make_buffer_from_lines(terminal_size, buf_opts, lines);
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let contents = TextContents::to_arc(TextContents::new(terminal_size));
    let tree = make_tree_with_buffers(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      bufs.clone(),
    );
    let (jsrt_tick_dispatcher, _jsrt_tick_queue) = channel(1);
    let state = State::to_arc(State::new(jsrt_tick_dispatcher));
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
    let data_access = StatefulDataAccess::new(
      state,
      tree,
      bufs,
      contents,
      Event::Key(key_event),
    );
    stateful
      ._test_raw_cursor_move(&data_access, Operation::CursorMoveTo((5, 0)));

    let tree = data_access.tree.clone();
    let actual1 = get_cursor_viewport(tree);
    assert_eq!(actual1.line_idx(), 0);
    assert_eq!(actual1.char_idx(), 5);

    // Step-2
    stateful
      ._test_raw_cursor_move(&data_access, Operation::CursorMoveTo((5, 1)));
    let tree = data_access.tree.clone();
    let actual2 = get_cursor_viewport(tree);
    assert_eq!(actual2.line_idx(), 1);
    assert_eq!(actual2.char_idx(), 5);

    // Step-3
    stateful
      ._test_raw_cursor_move(&data_access, Operation::CursorMoveTo((2, 1)));
    let tree = data_access.tree.clone();
    let actual3 = get_cursor_viewport(tree);
    assert_eq!(actual3.line_idx(), 1);
    assert_eq!(actual3.char_idx(), 2);

    // Step-4
    stateful
      ._test_raw_cursor_move(&data_access, Operation::CursorMoveTo((2, 0)));
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
    let buf = make_buffer_from_lines(terminal_size, buf_opts, lines);
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let contents = TextContents::to_arc(TextContents::new(terminal_size));
    let tree = make_tree_with_buffers(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      bufs.clone(),
    );
    let (jsrt_tick_dispatcher, _jsrt_tick_queue) = channel(1);
    let state = State::to_arc(State::new(jsrt_tick_dispatcher));
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
        Operation::CursorMoveTo((0, 2)),
        Operation::CursorMoveTo((3, 2)),
        Operation::CursorMoveTo((3, 0)),
        Operation::CursorMoveTo((0, 0)),
      ];
      let data_access = StatefulDataAccess::new(
        state.clone(),
        tree.clone(),
        bufs.clone(),
        contents.clone(),
        Event::Key(key_event),
      );
      for c in commands.iter() {
        stateful._test_raw_cursor_move(&data_access, c.clone());
      }
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree);
      assert_eq!(actual.line_idx(), 0);
      assert_eq!(actual.char_idx(), 0);
    }

    for _ in 0..10 {
      let commands = [
        Operation::CursorMoveTo((5, 0)),
        Operation::CursorMoveTo((5, 1)),
        Operation::CursorMoveTo((5, 1)),
        Operation::CursorMoveTo((0, 0)),
      ];
      let data_access = StatefulDataAccess::new(
        state.clone(),
        tree.clone(),
        bufs.clone(),
        contents.clone(),
        Event::Key(key_event),
      );
      for c in commands.iter() {
        stateful._test_raw_cursor_move(&data_access, c.clone());
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
    let buf = make_buffer_from_lines(terminal_size, buf_opts, lines.clone());
    let bufs = make_buffers_manager(buf_opts, vec![buf]);
    let contents = TextContents::to_arc(TextContents::new(terminal_size));
    let tree = make_tree_with_buffers(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(true)
        .build()
        .unwrap(),
      bufs.clone(),
    );
    let (jsrt_tick_dispatcher, _jsrt_tick_queue) = channel(1);
    let state = State::to_arc(State::new(jsrt_tick_dispatcher));
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
    assert_eq!(first_line_len, 29);

    // step-1: Move to the end of line-1.
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree.clone(),
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let command = Operation::CursorMoveTo((first_line_len, 0));
    stateful._test_raw_cursor_move(&data_access, command);

    let tree = data_access.tree.clone();
    let actual1 = get_cursor_viewport(tree.clone());
    assert_eq!(actual1.line_idx(), 0);
    assert_eq!(actual1.char_idx(), 27);

    // step-2: Move down to line-2.
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree.clone(),
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let command = Operation::CursorMoveTo((first_line_len, 1));
    stateful._test_raw_cursor_move(&data_access, command);

    let tree = data_access.tree.clone();
    let actual2 = get_cursor_viewport(tree);
    assert_eq!(actual2.line_idx(), 1);
    assert_eq!(actual2.char_idx(), 18);
  }
}

#[cfg(test)]
mod tests_raw_window_scroll_y_by {
  use super::*;

  use crate::buf::opt::BufferLocalOptionsBuilder;
  use crate::buf::{BufferArc, BuffersManagerArc};
  use crate::prelude::*;
  use crate::state::{State, StateArc};
  use crate::tests::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::tests::log::init as test_log_init;
  use crate::tests::tree::make_tree_with_buffers;
  use crate::ui::tree::TreeArc;
  use crate::ui::viewport::{
    CursorViewport, CursorViewportArc, Viewport, ViewportArc,
    ViewportSearchDirection,
  };
  use crate::ui::widget::window::{
    WindowLocalOptions, WindowLocalOptionsBuilder, content,
  };

  use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
  };
  use std::collections::BTreeMap;
  use tokio::sync::mpsc::{Receiver, Sender, channel};

  #[test]
  fn nowrap1() {
    test_log_init();

    let (tree, state, bufs, buf, contents) = make_tree(
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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0)].into_iter().collect();

      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        1,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(
      state,
      tree,
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollUpBy(1));

    let tree = data_access.tree.clone();

    // After cursor scroll
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![""];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0)].into_iter().collect();

      assert_viewport(
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
    let (tree, state, bufs, buf, contents) = make_tree(
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
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        8,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(
      state,
      tree,
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollDownBy(1));

    let tree = data_access.tree.clone();

    // After cursor scroll
    {
      info!("after cursor scroll");
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "     * The",
        "     * The",
        "",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]
          .into_iter()
          .collect();
      assert_viewport(
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
    let (tree, state, bufs, buf, contents) = make_tree(
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

      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        7,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(
      state,
      tree,
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollDownBy(1));

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

      assert_viewport(
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
    let (tree, state, bufs, buf, contents) = make_tree(
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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();

      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(
      state,
      tree,
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollDownBy(4));

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(4, 0), (5, 0), (6, 0), (7, 0), (8, 0)]
          .into_iter()
          .collect();

      assert_viewport(
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
    let (tree, state, bufs, buf, contents) = make_tree(
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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();

      assert_viewport(
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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollDownBy(4));

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(4, 0), (5, 0), (6, 0), (7, 0), (8, 0)]
          .into_iter()
          .collect();

      assert_viewport(
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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollDownBy(4));

    let tree = data_access.tree.clone();
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec!["     * The", "     * The", "", "", ""];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(8, 0), (9, 0), (10, 0)].into_iter().collect();

      assert_viewport(
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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollUpBy(1));

    let tree = data_access.tree.clone();
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec!["  3. If a ", "     * The", "     * The", "", ""];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(7, 0), (8, 0), (9, 0), (10, 0)].into_iter().collect();

      assert_viewport(
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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollUpBy(4));

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]
          .into_iter()
          .collect();

      assert_viewport(
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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollUpBy(1));

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(2, 0), (3, 0), (4, 0), (5, 0), (6, 0)]
          .into_iter()
          .collect();

      assert_viewport(
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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollUpBy(3));

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();

      assert_viewport(
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
    let (tree, state, bufs, buf, contents) = make_tree(
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
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(
      state,
      tree,
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollDownBy(4));

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(4, 0), (5, 0)].into_iter().collect();
      assert_viewport(
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
    let (tree, state, bufs, buf, contents) = make_tree(
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
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollDownBy(8));

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(8, 0), (9, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        8,
        10,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollDownBy(1));

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(9, 0), (10, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        9,
        11,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollDownBy(3));

    let tree = data_access.tree.clone();

    // Scroll-3
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec!["", "", ""];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(10, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        10,
        11,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollUpBy(2));

    let tree = data_access.tree.clone();

    // Scroll-4
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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(8, 0), (9, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        8,
        10,
        &expect_fills,
        &expect_fills,
      );
    }
  }
}
#[cfg(test)]
mod tests_raw_window_scroll_x_by {
  use super::*;

  use crate::buf::opt::BufferLocalOptionsBuilder;
  use crate::buf::{BufferArc, BuffersManagerArc};
  use crate::prelude::*;
  use crate::state::{State, StateArc};
  use crate::tests::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::tests::log::init as test_log_init;
  use crate::tests::tree::make_tree_with_buffers;
  use crate::ui::tree::TreeArc;
  use crate::ui::viewport::{
    CursorViewport, CursorViewportArc, Viewport, ViewportArc,
    ViewportSearchDirection,
  };
  use crate::ui::widget::window::{
    WindowLocalOptions, WindowLocalOptionsBuilder,
  };

  use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
  };
  use std::collections::BTreeMap;
  use tokio::sync::mpsc::{Receiver, Sender, channel};

  #[test]
  fn nowrap1() {
    test_log_init();

    let (tree, state, bufs, buf, contents) = make_tree(
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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0)].into_iter().collect();

      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        1,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(
      state,
      tree,
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollRightBy(1));

    let tree = data_access.tree.clone();

    // After cursor scroll
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![""];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0)].into_iter().collect();

      assert_viewport(
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
    let (tree, state, bufs, buf, contents) = make_tree(
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
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        8,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(
      state,
      tree,
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollRightBy(1));

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
      assert_viewport(
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
    let (tree, state, bufs, buf, contents) = make_tree(
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

      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        7,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(
      state,
      tree,
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollLeftBy(1));

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

      assert_viewport(
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
    let (tree, state, bufs, buf, contents) = make_tree(
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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();

      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(
      state,
      tree,
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful._test_raw_window_scroll(
      &data_access,
      Operation::WindowScrollRightBy(12),
    );

    let tree = data_access.tree.clone();

    // After cursor scroll
    {
      let viewport = get_viewport(tree);
      let expect = vec![
        "!\n",
        "ite simple",
        " contains ",
        "e line is ",
        "e line is ",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();

      assert_viewport(
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
    let (tree, state, bufs, buf, contents) = make_tree(
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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();

      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful._test_raw_window_scroll(
      &data_access,
      Operation::WindowScrollRightBy(12),
    );

    let tree = data_access.tree.clone();

    // Scroll-1
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "!\n",
        "ite simple",
        " contains ",
        "e line is ",
        "e line is ",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();

      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful._test_raw_window_scroll(
      &data_access,
      Operation::WindowScrollRightBy(10),
    );

    let tree = data_access.tree.clone();

    // Scroll-2
    {
      let viewport = get_viewport(tree.clone());
      let expect =
        vec!["", " and small", "several th", "small enou", "too long t"];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();

      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful._test_raw_window_scroll(
      &data_access,
      Operation::WindowScrollRightBy(160),
    );

    let tree = data_access.tree.clone();

    // Scroll-3
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec!["", "", "", "\n", ""];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();

      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful._test_raw_window_scroll(
      &data_access,
      Operation::WindowScrollLeftBy(156),
    );

    let tree = data_access.tree.clone();

    // Scroll-4
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "llo, RSVIM",
        "is is a qu",
        "t still it",
        "1. When th",
        "2. When th",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();

      assert_viewport(
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
    let (tree, state, bufs, buf, contents) = make_tree(
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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();

      assert_viewport(
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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollRightBy(4));

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();

      assert_viewport(
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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollRightBy(4));

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();

      assert_viewport(
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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollLeftBy(1));

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();

      assert_viewport(
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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollLeftBy(4));

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();

      assert_viewport(
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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollLeftBy(1));

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();

      assert_viewport(
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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollLeftBy(3));

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();

      assert_viewport(
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
    let (tree, state, bufs, buf, contents) = make_tree(
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
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(
      state,
      tree,
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollRightBy(4));

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
      assert_viewport(
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
    let (tree, state, bufs, buf, contents) = make_tree(
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
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollRightBy(8));

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
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollRightBy(1));

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
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollRightBy(3));

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
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollLeftBy(1));

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
      assert_viewport(
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
mod tests_raw_window_scroll_to {
  use super::*;

  use crate::buf::opt::BufferLocalOptionsBuilder;
  use crate::buf::{BufferArc, BuffersManagerArc};
  use crate::prelude::*;
  use crate::state::{State, StateArc};
  use crate::tests::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::tests::log::init as test_log_init;
  use crate::tests::tree::make_tree_with_buffers;
  use crate::ui::tree::TreeArc;
  use crate::ui::viewport::{
    CursorViewport, CursorViewportArc, Viewport, ViewportArc,
    ViewportSearchDirection,
  };
  use crate::ui::widget::window::{
    WindowLocalOptions, WindowLocalOptionsBuilder,
  };

  use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
  };
  use std::collections::BTreeMap;
  use tokio::sync::mpsc::{Receiver, Sender, channel};

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
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        8,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(
      state,
      tree,
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollTo((0, 1)));

    let tree = data_access.tree.clone();

    // After cursor scroll
    {
      info!("after cursor scroll");
      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
        "     * The",
        "     * The",
        "",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]
          .into_iter()
          .collect();
      assert_viewport(
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
    let (tree, state, bufs, buf, contents) = make_tree(
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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();

      assert_viewport(
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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollTo((0, 4)));

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(4, 0), (5, 0), (6, 0), (7, 0), (8, 0)]
          .into_iter()
          .collect();

      assert_viewport(
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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollTo((0, 8)));

    let tree = data_access.tree.clone();
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec!["     * The", "     * The", "", "", ""];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(8, 0), (9, 0), (10, 0)].into_iter().collect();

      assert_viewport(
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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollTo((0, 7)));

    let tree = data_access.tree.clone();
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec!["  3. If a ", "     * The", "     * The", "", ""];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(7, 0), (8, 0), (9, 0), (10, 0)].into_iter().collect();

      assert_viewport(
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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollTo((0, 3)));

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]
          .into_iter()
          .collect();

      assert_viewport(
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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollTo((0, 2)));

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(2, 0), (3, 0), (4, 0), (5, 0), (6, 0)]
          .into_iter()
          .collect();

      assert_viewport(
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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollTo((0, 0)));

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();

      assert_viewport(
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
    let (tree, state, bufs, buf, contents) = make_tree(
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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();

      assert_viewport(
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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollTo((4, 0)));

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();

      assert_viewport(
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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollTo((8, 0)));

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();

      assert_viewport(
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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollTo((7, 0)));

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();

      assert_viewport(
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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollTo((3, 0)));

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();

      assert_viewport(
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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollTo((2, 0)));

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();

      assert_viewport(
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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful
      ._test_raw_window_scroll(&data_access, Operation::WindowScrollTo((0, 0)));

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();

      assert_viewport(
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
mod tests_cursor_move {
  use super::*;

  use crate::buf::opt::{BufferLocalOptionsBuilder, FileFormatOption};
  use crate::buf::{BufferArc, BuffersManagerArc};
  use crate::prelude::*;
  use crate::state::{State, StateArc};
  use crate::tests::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::tests::log::init as test_log_init;
  use crate::tests::tree::make_tree_with_buffers;
  use crate::ui::tree::TreeArc;
  use crate::ui::viewport::{
    CursorViewport, CursorViewportArc, Viewport, ViewportArc,
    ViewportSearchDirection,
  };
  use crate::ui::widget::window::{
    WindowLocalOptions, WindowLocalOptionsBuilder,
  };

  use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
  };
  use std::collections::BTreeMap;
  use tokio::sync::mpsc::{Receiver, Sender, channel};

  #[test]
  fn nowrap1() {
    test_log_init();

    let (tree, state, bufs, _buf, contents) = make_tree(
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

    let data_access = StatefulDataAccess::new(
      state,
      tree,
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful_machine = NormalStateful::default();
    stateful_machine.cursor_move(&data_access, Operation::CursorMoveUpBy(1));

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
    let (tree, state, bufs, _buf, contents) = make_tree(
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

    let data_access = StatefulDataAccess::new(
      state,
      tree,
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful_machine = NormalStateful::default();
    stateful_machine.cursor_move(&data_access, Operation::CursorMoveUpBy(1));

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
    let (tree, state, bufs, buf, contents) = make_tree(
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

    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(3));

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
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        8,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(5));

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
      assert_viewport(
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
  fn nowrap3_crlf_win() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\r\n",
      "This is a quite simple and small test lines.\r\n",
      "But still it contains several things we want to test:\r\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\r\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\r\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\r\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\r\n",
    ];
    let buf_opts = BufferLocalOptionsBuilder::default()
      .file_format(FileFormatOption::Dos)
      .build()
      .unwrap();
    let (tree, state, bufs, buf, contents) = make_tree_with_buffer_opts(
      U16Size::new(10, 10),
      buf_opts,
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

    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(3));

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
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        8,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(5));

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
      assert_viewport(
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
  fn nowrap3_cr_mac() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\r",
      "This is a quite simple and small test lines.\r",
      "But still it contains several things we want to test:\r",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\r",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\r",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\r",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\r",
    ];
    let buf_opts = BufferLocalOptionsBuilder::default()
      .file_format(FileFormatOption::Mac)
      .build()
      .unwrap();
    let (tree, state, bufs, buf, contents) = make_tree_with_buffer_opts(
      U16Size::new(10, 10),
      buf_opts,
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

    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(3));

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
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        8,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(5));

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
      assert_viewport(
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
    let (tree, state, bufs, buf, contents) = make_tree(
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

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(3));

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
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(5));

    // Move-2
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 3);
      assert_eq!(actual2.char_idx(), 5);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(5));

    // Move-3
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 3);
      assert_eq!(actual2.char_idx(), 10);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "ello, RSVI",
        "his is a q",
        "ut still i",
        " 1. When t",
        " 2. When t",
        "    * The ",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(5));

    // Move-4
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 3);
      assert_eq!(actual2.char_idx(), 15);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        " RSVIM!\n",
        "s a quite ",
        "ill it con",
        "hen the li",
        "hen the li",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(5));

    // Move-5
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 3);
      assert_eq!(actual2.char_idx(), 20);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "M!\n",
        "uite simpl",
        "t contains",
        "he line is",
        "he line is",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(11));

    // Move-6
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 3);
      assert_eq!(actual2.char_idx(), 31);

      let viewport = get_viewport(tree.clone());
      let expect =
        vec!["", " and small", "several th", "small enou", "too long t"];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(120));

    // Move-7
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 3);
      assert_eq!(actual2.char_idx(), 151);

      let viewport = get_viewport(tree.clone());
      let expect = vec!["", "", "", "t the rend", ""];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(2));

    // Move-8
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 5);
      assert_eq!(actual2.char_idx(), 93);

      let viewport = get_viewport(tree.clone());
      let expect = vec!["", "", "t, then th", "ere're mul", ".\n"];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(1, 0), (2, 0), (3, 0), (4, 0), (5, 0)]
          .into_iter()
          .collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        1,
        6,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(1));

    // Move-9
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 6);
      assert_eq!(actual2.char_idx(), 93);

      let viewport = get_viewport(tree.clone());
      let expect = vec!["", "t, then th", "ere're mul", ".\n", "are been s"];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(2, 0), (3, 0), (4, 0), (5, 0), (6, 0)]
          .into_iter()
          .collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        2,
        7,
        &expect_fills,
        &expect_fills,
      );
    }
  }

  #[test]
  fn nowrap4_crlf_win() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\r\n",
      "This is a quite simple and small test lines.\r\n",
      "But still it contains several things we want to test:\r\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\r\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\r\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\r\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\r\n",
    ];
    let buf_opts = BufferLocalOptionsBuilder::default()
      .file_format(FileFormatOption::Dos)
      .build()
      .unwrap();
    let (tree, state, bufs, buf, contents) = make_tree_with_buffer_opts(
      U16Size::new(10, 5),
      buf_opts,
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

    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(3));

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
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(5));

    // Move-2
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 3);
      assert_eq!(actual2.char_idx(), 5);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(5));

    // Move-3
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 3);
      assert_eq!(actual2.char_idx(), 10);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "ello, RSVI",
        "his is a q",
        "ut still i",
        " 1. When t",
        " 2. When t",
        "    * The ",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(5));

    // Move-4
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 3);
      assert_eq!(actual2.char_idx(), 15);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        " RSVIM!\r\n",
        "s a quite ",
        "ill it con",
        "hen the li",
        "hen the li",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(5));

    // Move-5
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 3);
      assert_eq!(actual2.char_idx(), 20);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "M!\r\n",
        "uite simpl",
        "t contains",
        "he line is",
        "he line is",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(11));

    // Move-6
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 3);
      assert_eq!(actual2.char_idx(), 31);

      let viewport = get_viewport(tree.clone());
      let expect =
        vec!["", " and small", "several th", "small enou", "too long t"];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(150));

    // Move-7
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 3);
      assert_eq!(actual2.char_idx(), 157);

      let viewport = get_viewport(tree.clone());
      let expect = vec!["", "", "", "rendering.", ""];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(2));

    // Move-8
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 5);
      assert_eq!(actual2.char_idx(), 93);

      let viewport = get_viewport(tree.clone());
      let expect = vec!["", "", "t, then th", "ere're mul", ".\r\n"];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(1, 0), (2, 0), (3, 0), (4, 0), (5, 0)]
          .into_iter()
          .collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        1,
        6,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(1));

    // Move-9
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 6);
      assert_eq!(actual2.char_idx(), 93);

      let viewport = get_viewport(tree.clone());
      let expect = vec!["", "t, then th", "ere're mul", ".\r\n", "are been s"];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(2, 0), (3, 0), (4, 0), (5, 0), (6, 0)]
          .into_iter()
          .collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        2,
        7,
        &expect_fills,
        &expect_fills,
      );
    }
  }

  #[test]
  fn nowrap4_cr_mac() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\r",
      "This is a quite simple and small test lines.\r",
      "But still it contains several things we want to test:\r",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\r",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\r",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\r",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\r",
    ];
    let buf_opts = BufferLocalOptionsBuilder::default()
      .file_format(FileFormatOption::Mac)
      .build()
      .unwrap();
    let (tree, state, bufs, buf, contents) = make_tree_with_buffer_opts(
      U16Size::new(10, 5),
      buf_opts,
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

    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(3));

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
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(5));

    // Move-2
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 3);
      assert_eq!(actual2.char_idx(), 5);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSV",
        "This is a ",
        "But still ",
        "  1. When ",
        "  2. When ",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(5));

    // Move-3
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 3);
      assert_eq!(actual2.char_idx(), 10);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "ello, RSVI",
        "his is a q",
        "ut still i",
        " 1. When t",
        " 2. When t",
        "    * The ",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(5));

    // Move-4
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 3);
      assert_eq!(actual2.char_idx(), 15);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        " RSVIM!\r",
        "s a quite ",
        "ill it con",
        "hen the li",
        "hen the li",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(5));

    // Move-5
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 3);
      assert_eq!(actual2.char_idx(), 20);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "M!\r",
        "uite simpl",
        "t contains",
        "he line is",
        "he line is",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(11));

    // Move-6
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 3);
      assert_eq!(actual2.char_idx(), 31);

      let viewport = get_viewport(tree.clone());
      let expect =
        vec!["", " and small", "several th", "small enou", "too long t"];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(150));

    // Move-7
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 3);
      assert_eq!(actual2.char_idx(), 157);

      let viewport = get_viewport(tree.clone());
      let expect = vec!["", "", "", "rendering.", ""];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(2));

    // Move-8
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 5);
      assert_eq!(actual2.char_idx(), 93);

      let viewport = get_viewport(tree.clone());
      let expect = vec!["", "", "t, then th", "ere're mul", ".\r"];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(1, 0), (2, 0), (3, 0), (4, 0), (5, 0)]
          .into_iter()
          .collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        1,
        6,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(1));

    // Move-9
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 6);
      assert_eq!(actual2.char_idx(), 93);

      let viewport = get_viewport(tree.clone());
      let expect = vec!["", "t, then th", "ere're mul", ".\r", "are been s"];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(2, 0), (3, 0), (4, 0), (5, 0), (6, 0)]
          .into_iter()
          .collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        2,
        7,
        &expect_fills,
        &expect_fills,
      );
    }
  }

  #[test]
  fn nowrap5() {
    test_log_init();

    let terminal_size = U16Size::new(29, 5);
    let lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "\t1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "\t2. When the line is too long to be completely put in a row of the window content widget, there're still multiple cases.\n",
    ];
    let (tree, state, bufs, buf, contents) = make_tree(
      terminal_size,
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

    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.cursor_move(&data_access, Operation::CursorMoveTo((21, 3)));

    // Move-1
    {
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 3);
      assert_eq!(actual1.char_idx(), 21);
      assert_eq!(actual1.row_idx(), 3);
      assert_eq!(actual1.column_idx(), 28);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and sm",
        "But still it contains several",
        "\t1. When the line is s",
        "\t2. When the line is t",
      ];
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );

      let expect_canvas = vec![
        "Hello, RSVIM!                ",
        "This is a quite simple and sm",
        "But still it contains several",
        "        1. When the line is s",
        "        2. When the line is t",
      ];
      let actual_canvas = make_canvas(tree.clone(), terminal_size);
      let actual_canvas = lock!(actual_canvas);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(1));

    // Move-2
    {
      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 3);
      assert_eq!(actual1.char_idx(), 22);
      assert_eq!(actual1.row_idx(), 3);
      assert_eq!(actual1.column_idx(), 28);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "ello, RSVIM!\n",
        "his is a quite simple and sma",
        "ut still it contains several ",
        "1. When the line is sm",
        "2. When the line is to",
      ];
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 7), (4, 7)]
          .into_iter()
          .collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
          .into_iter()
          .collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );

      let expect_canvas = vec![
        "ello, RSVIM!                 ",
        "his is a quite simple and sma",
        "ut still it contains several ",
        ">>>>>>>1. When the line is sm",
        ">>>>>>>2. When the line is to",
      ];
      let actual_canvas = make_canvas(tree.clone(), terminal_size);
      let actual_canvas = lock!(actual_canvas);
      assert_canvas(&actual_canvas, &expect_canvas);
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
    let (tree, state, bufs, buf, contents) = make_tree(
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

    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(3, 0)].into_iter().collect();
      assert_viewport(
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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(5, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        5,
        6,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(1));

    // Move-3
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 6);
      assert_eq!(actual2.char_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "     * The",
        " extra par",
        "ts are spl",
        "it into th",
        "e next row",
        ", if eithe",
        "r line-wra",
        "p or word-",
        "wrap optio",
        "ns are bee",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        6,
        7,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveUpBy(3));

    // Move-4
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 3);
      assert_eq!(actual2.char_idx(), 0);

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(3, 0)].into_iter().collect();
      assert_viewport(
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
  fn wrap_nolinebreak2() {
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

    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(3));

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        3,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(2));

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        3,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(80));

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        3,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(40));

    // Move-4
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 122);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "mall enoug",
        "h to compl",
        "etely put ",
        "inside a r",
        "ow of the ",
        "window con",
        "tent widge",
        "t, then th",
        "e line-wra",
        "p and word",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        3,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(40));

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        3,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(5));

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        3,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveLeftBy(8));

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        3,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveLeftBy(100));

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        3,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveLeftBy(100));

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(3, 0)].into_iter().collect();
      assert_viewport(
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
  fn wrap_nolinebreak3() {
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
    let (tree, state, bufs, buf, contents) = make_tree(
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
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(3));

    // Move-1
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "But still it contains sev",
        "eral things we want to te",
        "st:\n",
        "  1. When the line is sma",
        "ll enough to completely p",
        "ut inside a row.\n",
        "  2. When the line is too",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(2, 0), (3, 0), (4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        2,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(24));

    // Move-2
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 24);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "But still it contains sev",
        "eral things we want to te",
        "st:\n",
        "  1. When the line is sma",
        "ll enough to completely p",
        "ut inside a row.\n",
        "  2. When the line is too",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(2, 0), (3, 0), (4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        2,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(45));

    // Move-3
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 65);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "But still it contains sev",
        "eral things we want to te",
        "st:\n",
        "  1. When the line is sma",
        "ll enough to completely p",
        "ut inside a row.\n",
        "  2. When the line is too",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(2, 0), (3, 0), (4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        2,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(1));

    // Move-4
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 4);
      assert_eq!(actual.char_idx(), 65);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "  2. When the line is too",
        " long to be completely pu",
        "t in a row of the window ",
        "content widget, there're ",
        "multiple cases:\n",
        "    * The extra parts are",
        " been truncated if both l",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(4, 0), (5, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        4,
        6,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveUpBy(1));

    // Move-5
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 65);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "  1. When the line is sma",
        "ll enough to completely p",
        "ut inside a row.\n",
        "  2. When the line is too",
        " long to be completely pu",
        "t in a row of the window ",
        "content widget, there're ",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(3, 0), (4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        3,
        5,
        &expect_fills,
        &expect_fills,
      );
    }
  }

  #[test]
  fn wrap_nolinebreak3_crlf_win() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\r\n",
      "This is a quite simple and small test lines.\r\n",
      "But still it contains several things we want to test:\r\n",
      "  1. When the line is small enough to completely put inside a row.\r\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\r\n",
      "    * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\r\n",
      "    * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\r\n",
    ];
    let buf_opts = BufferLocalOptionsBuilder::default()
      .file_format(FileFormatOption::Dos)
      .build()
      .unwrap();
    let (tree, state, bufs, buf, contents) = make_tree_with_buffer_opts(
      U16Size::new(25, 7),
      buf_opts,
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
        "Hello, RSVIM!\r\n",
        "This is a quite simple an",
        "d small test lines.\r\n",
        "But still it contains sev",
        "eral things we want to te",
        "st:\r\n",
        "  1. When the line is sma",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(3));

    // Move-1
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "But still it contains sev",
        "eral things we want to te",
        "st:\r\n",
        "  1. When the line is sma",
        "ll enough to completely p",
        "ut inside a row.\r\n",
        "  2. When the line is too",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(2, 0), (3, 0), (4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        2,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(24));

    // Move-2
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 24);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "But still it contains sev",
        "eral things we want to te",
        "st:\r\n",
        "  1. When the line is sma",
        "ll enough to completely p",
        "ut inside a row.\r\n",
        "  2. When the line is too",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(2, 0), (3, 0), (4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        2,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(45));

    // Move-3
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 65);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "But still it contains sev",
        "eral things we want to te",
        "st:\r\n",
        "  1. When the line is sma",
        "ll enough to completely p",
        "ut inside a row.\r\n",
        "  2. When the line is too",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(2, 0), (3, 0), (4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        2,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(1));

    // Move-4
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 4);
      assert_eq!(actual.char_idx(), 65);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "  2. When the line is too",
        " long to be completely pu",
        "t in a row of the window ",
        "content widget, there're ",
        "multiple cases:\r\n",
        "    * The extra parts are",
        " been truncated if both l",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(4, 0), (5, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        4,
        6,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveUpBy(1));

    // Move-5
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 65);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "  1. When the line is sma",
        "ll enough to completely p",
        "ut inside a row.\r\n",
        "  2. When the line is too",
        " long to be completely pu",
        "t in a row of the window ",
        "content widget, there're ",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(3, 0), (4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        3,
        5,
        &expect_fills,
        &expect_fills,
      );
    }
  }

  #[test]
  fn wrap_nolinebreak3_cr_mac() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\r",
      "This is a quite simple and small test lines.\r",
      "But still it contains several things we want to test:\r",
      "  1. When the line is small enough to completely put inside a row.\r",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\r",
      "    * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\r",
      "    * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\r",
    ];
    let buf_opts = BufferLocalOptionsBuilder::default()
      .file_format(FileFormatOption::Mac)
      .build()
      .unwrap();
    let (tree, state, bufs, buf, contents) = make_tree_with_buffer_opts(
      U16Size::new(25, 7),
      buf_opts,
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
        "Hello, RSVIM!\r",
        "This is a quite simple an",
        "d small test lines.\r",
        "But still it contains sev",
        "eral things we want to te",
        "st:\r",
        "  1. When the line is sma",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(3));

    // Move-1
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "But still it contains sev",
        "eral things we want to te",
        "st:\r",
        "  1. When the line is sma",
        "ll enough to completely p",
        "ut inside a row.\r",
        "  2. When the line is too",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(2, 0), (3, 0), (4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        2,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(24));

    // Move-2
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 24);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "But still it contains sev",
        "eral things we want to te",
        "st:\r",
        "  1. When the line is sma",
        "ll enough to completely p",
        "ut inside a row.\r",
        "  2. When the line is too",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(2, 0), (3, 0), (4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        2,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(45));

    // Move-3
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 65);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "But still it contains sev",
        "eral things we want to te",
        "st:\r",
        "  1. When the line is sma",
        "ll enough to completely p",
        "ut inside a row.\r",
        "  2. When the line is too",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(2, 0), (3, 0), (4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        2,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(1));

    // Move-4
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 4);
      assert_eq!(actual.char_idx(), 65);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "  2. When the line is too",
        " long to be completely pu",
        "t in a row of the window ",
        "content widget, there're ",
        "multiple cases:\r",
        "    * The extra parts are",
        " been truncated if both l",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(4, 0), (5, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        4,
        6,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveUpBy(1));

    // Move-5
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 65);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "  1. When the line is sma",
        "ll enough to completely p",
        "ut inside a row.\r",
        "  2. When the line is too",
        " long to be completely pu",
        "t in a row of the window ",
        "content widget, there're ",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(3, 0), (4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        3,
        5,
        &expect_fills,
        &expect_fills,
      );
    }
  }

  #[test]
  fn wrap_nolinebreak4() {
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
    let (tree, state, bufs, buf, contents) = make_tree(
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
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.cursor_move(&data_access, Operation::CursorMoveBy((50, 3)));

    // Move-1
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 50);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "But still it contains sev",
        "eral things we want to te",
        "st:\n",
        "  1. When the line is sma",
        "ll enough to completely p",
        "ut inside a row.\n",
        "  2. When the line is too",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(2, 0), (3, 0), (4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        2,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveBy((24, 1)));

    // Move-2
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 4);
      assert_eq!(actual.char_idx(), 74);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "  2. When the line is too",
        " long to be completely pu",
        "t in a row of the window ",
        "content widget, there're ",
        "multiple cases:\n",
        "    * The extra parts are",
        " been truncated if both l",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(4, 0), (5, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        4,
        6,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveBy((-4, -4)));

    // Move-3
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 0);
      assert_eq!(actual.char_idx(), 12);

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
      assert_viewport(
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
  fn wrap_nolinebreak4_crlf_win() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\r\n",
      "This is a quite simple lines.\r\n",
      "But still it contains several:\r\n",
      "1. The line is very small.\r\n",
      "2. The line is very long\r\n",
      "  * The extra parts are been truncated.\r\n",
      "  * The extra parts are split into next row.\r\n",
    ];
    let buf_opts = BufferLocalOptionsBuilder::default()
      .file_format(FileFormatOption::Dos)
      .build()
      .unwrap();
    let (tree, state, bufs, buf, contents) = make_tree_with_buffer_opts(
      U16Size::new(15, 7),
      buf_opts,
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
        "Hello, RSVIM!\r\n",
        "This is a quite",
        " simple lines.\r\n",
        "But still it co",
        "ntains several:",
        "1. The line is ",
        "very small.\r\n",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.cursor_move(&data_access, Operation::CursorMoveBy((50, 3)));

    // Move-1
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 25);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSVIM!\r\n",
        "This is a quite",
        " simple lines.\r\n",
        "But still it co",
        "ntains several:",
        "1. The line is ",
        "very small.\r\n",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveBy((24, 1)));

    // Move-2
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 4);
      assert_eq!(actual.char_idx(), 23);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "But still it co",
        "ntains several:",
        "1. The line is ",
        "very small.\r\n",
        "2. The line is ",
        "very long\r\n",
        "  * The extra p",
        "arts are been t",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(2, 0), (3, 0), (4, 0), (5, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        2,
        6,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveBy((-4, -4)));

    // Move-3
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 0);
      assert_eq!(actual.char_idx(), 12);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSVIM!\r\n",
        "This is a quite",
        " simple lines.\r\n",
        "But still it co",
        "ntains several:",
        "1. The line is ",
        "very small.\r\n",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
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
  fn wrap_nolinebreak4_cr_mac() {
    test_log_init();

    let lines = vec![
      "Hello, RSVIM!\r",
      "This is a quite simple lines.\r",
      "But still it contains several:\r",
      "1. The line is very small.\r",
      "2. The line is very long\r",
      "  * The extra parts are been truncated.\r",
      "  * The extra parts are split into next row.\r",
    ];
    let buf_opts = BufferLocalOptionsBuilder::default()
      .file_format(FileFormatOption::Mac)
      .build()
      .unwrap();
    let (tree, state, bufs, buf, contents) = make_tree_with_buffer_opts(
      U16Size::new(15, 7),
      buf_opts,
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
        "Hello, RSVIM!\r",
        "This is a quite",
        " simple lines.\r",
        "But still it co",
        "ntains several:",
        "1. The line is ",
        "very small.\r",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.cursor_move(&data_access, Operation::CursorMoveBy((50, 3)));

    // Move-1
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 25);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSVIM!\r",
        "This is a quite",
        " simple lines.\r",
        "But still it co",
        "ntains several:",
        "1. The line is ",
        "very small.\r",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveBy((24, 1)));

    // Move-2
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 4);
      assert_eq!(actual.char_idx(), 23);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "But still it co",
        "ntains several:",
        "1. The line is ",
        "very small.\r",
        "2. The line is ",
        "very long\r",
        "  * The extra p",
        "arts are been t",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(2, 0), (3, 0), (4, 0), (5, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        2,
        6,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveBy((-4, -4)));

    // Move-3
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 0);
      assert_eq!(actual.char_idx(), 12);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "Hello, RSVIM!\r",
        "This is a quite",
        " simple lines.\r",
        "But still it co",
        "ntains several:",
        "1. The line is ",
        "very small.\r",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
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
    let (tree, state, bufs, buf, contents) = make_tree(
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

    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(3, 0)].into_iter().collect();
      assert_viewport(
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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(5, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        5,
        6,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(1));

    // Move-3
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 6);
      assert_eq!(actual2.char_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "     * The",
        " extra ",
        "parts are ",
        "split into",
        " the next ",
        "row, if ",
        "either ",
        "line-wrap ",
        "or word-",
        "wrap ",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        6,
        7,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveUpBy(3));

    // Move-4
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 3);
      assert_eq!(actual2.char_idx(), 0);

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(3, 0)].into_iter().collect();
      assert_viewport(
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
  fn wrap_linebreak2() {
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

    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(3));

    // Move-1
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 0);

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        3,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(2));

    // Move-2
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 2);

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        3,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(80));

    // Move-3
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 82);

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        3,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(40));

    // Move-4
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 122);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "completely",
        " put ",
        "inside a ",
        "row of the",
        " window ",
        "content ",
        "widget, ",
        "then the ",
        "line-wrap ",
        "and word-",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        3,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(40));

    // Move-5
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 157);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "ow content",
        " widget, ",
        "then the ",
        "line-wrap ",
        "and word-",
        "wrap ",
        "doesn't ",
        "affect the",
        " rendering",
        ".\n",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        3,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(5));

    // Move-6
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 157);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "ow content",
        " widget, ",
        "then the ",
        "line-wrap ",
        "and word-",
        "wrap ",
        "doesn't ",
        "affect the",
        " rendering",
        ".\n",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        3,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveLeftBy(8));

    // Move-7
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 149);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "ow content",
        " widget, ",
        "then the ",
        "line-wrap ",
        "and word-",
        "wrap ",
        "doesn't ",
        "affect the",
        " rendering",
        ".\n",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        3,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveLeftBy(100));

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
        " content ",
        "widget, ",
        "then the ",
        "line-wrap ",
        "and word-",
        "wrap ",
        "doesn't ",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        3,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveLeftBy(100));

    // Move-9
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 0);

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
      let expect_fills: BTreeMap<usize, usize> =
        vec![(3, 0)].into_iter().collect();
      assert_viewport(
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
  fn wrap_linebreak3() {
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
    let (tree, state, bufs, buf, contents) = make_tree(
      U16Size::new(25, 7),
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

    // Before
    {
      let viewport = get_viewport(tree.clone());

      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite simple ",
        "and small test lines.\n",
        "But still it contains ",
        "several things we want to",
        " test:\n",
        "  1. When the line is ",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(3));

    // Move-1
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 0);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "But still it contains ",
        "several things we want to",
        " test:\n",
        "  1. When the line is ",
        "small enough to ",
        "completely put inside a ",
        "row.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        2,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(24));

    // Move-2
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 24);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "But still it contains ",
        "several things we want to",
        " test:\n",
        "  1. When the line is ",
        "small enough to ",
        "completely put inside a ",
        "row.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        2,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(45));

    // Move-3
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 65);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "But still it contains ",
        "several things we want to",
        " test:\n",
        "  1. When the line is ",
        "small enough to ",
        "completely put inside a ",
        "row.\n",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        2,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(1));

    // Move-4
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 4);
      assert_eq!(actual.char_idx(), 65);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "  2. When the line is too",
        " long to be completely ",
        "put in a row of the ",
        "window content widget, ",
        "there're multiple cases:\n",
        "    * The extra parts are",
        " been truncated if both ",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(4, 0), (5, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        4,
        6,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveUpBy(1));

    // Move-5
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 65);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "  1. When the line is ",
        "small enough to ",
        "completely put inside a ",
        "row.\n",
        "  2. When the line is too",
        " long to be completely ",
        "put in a row of the ",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(3, 0), (4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        3,
        5,
        &expect_fills,
        &expect_fills,
      );
    }
  }

  #[test]
  fn wrap_linebreak4() {
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
    let (tree, state, bufs, buf, contents) = make_tree(
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
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        4,
        &expect_fills,
        &expect_fills,
      );
    }

    let data_access = StatefulDataAccess::new(
      state,
      tree.clone(),
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.cursor_move(&data_access, Operation::CursorMoveBy((50, 3)));

    // Move-1
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 3);
      assert_eq!(actual.char_idx(), 50);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "But still it contains sev",
        "eral things we want to te",
        "st:\n",
        "  1. When the line is sma",
        "ll enough to completely p",
        "ut inside a row.\n",
        "  2. When the line is too",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(2, 0), (3, 0), (4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        2,
        5,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveBy((24, 1)));

    // Move-2
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 4);
      assert_eq!(actual.char_idx(), 74);

      let viewport = get_viewport(tree.clone());
      let expect = vec![
        "  2. When the line is too",
        " long to be completely pu",
        "t in a row of the window ",
        "content widget, there're ",
        "multiple cases:\n",
        "    * The extra parts are",
        " been truncated if both l",
      ];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(4, 0), (5, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        4,
        6,
        &expect_fills,
        &expect_fills,
      );
    }

    stateful.cursor_move(&data_access, Operation::CursorMoveBy((-4, -4)));

    // Move-3
    {
      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree.clone());
      assert_eq!(actual.line_idx(), 0);
      assert_eq!(actual.char_idx(), 12);

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
      assert_viewport(
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
mod tests_goto_command_line_ex_mode {
  use super::*;

  use crate::buf::opt::BufferLocalOptionsBuilder;
  use crate::buf::{BufferArc, BuffersManagerArc};
  use crate::prelude::*;
  use crate::state::{State, StateArc};
  use crate::tests::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::tests::log::init as test_log_init;
  use crate::tests::tree::make_tree_with_buffers;
  use crate::ui::tree::TreeArc;
  use crate::ui::viewport::{
    CursorViewport, CursorViewportArc, Viewport, ViewportArc,
    ViewportSearchDirection,
  };
  use crate::ui::widget::window::{
    WindowLocalOptions, WindowLocalOptionsBuilder,
  };

  use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
  };
  use std::collections::BTreeMap;
  use tokio::sync::mpsc::{Receiver, Sender, channel};

  #[test]
  fn nowrap1() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let (tree, state, bufs, _buf, contents) = make_tree_with_cmdline(
      terminal_size,
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

    let data_access = StatefulDataAccess::new(
      state,
      tree,
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.goto_command_line_ex_mode(&data_access);

    let tree = data_access.tree.clone();
    let actual_cursor = lock!(tree.clone())
      .command_line()
      .unwrap()
      .cursor_viewport();
    assert_eq!(actual_cursor.line_idx(), 0);
    assert_eq!(actual_cursor.char_idx(), 0);
    assert_eq!(actual_cursor.row_idx(), 0);
    assert_eq!(actual_cursor.column_idx(), 0);

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
      ":         ",
    ];
    let actual_canvas = make_canvas(tree.clone(), terminal_size);
    let actual_canvas = lock!(actual_canvas);
    assert_canvas(&actual_canvas, &expect_canvas);
  }
}

#[cfg(test)]
mod tests_goto_insert_mode {
  use super::*;

  use crate::buf::opt::BufferLocalOptionsBuilder;
  use crate::buf::{BufferArc, BuffersManagerArc};
  use crate::prelude::*;
  use crate::state::fsm::InsertStateful;
  use crate::state::ops::CursorInsertPayload;
  use crate::state::{State, StateArc};
  use crate::tests::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::tests::log::init as test_log_init;
  use crate::tests::tree::make_tree_with_buffers;
  use crate::ui::tree::TreeArc;
  use crate::ui::viewport::{
    CursorViewport, CursorViewportArc, Viewport, ViewportArc,
    ViewportSearchDirection,
  };
  use crate::ui::widget::window::{
    WindowLocalOptions, WindowLocalOptionsBuilder,
  };

  use compact_str::ToCompactString;
  use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
  };
  use std::collections::BTreeMap;
  use tokio::sync::mpsc::{Receiver, Sender, channel};

  #[test]
  fn nowrap_goto_insert_keep1() {
    test_log_init();

    let terminal_size = U16Size::new(30, 3);
    let (tree, state, bufs, buf, contents) = make_tree(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      vec!["Should go to insert mode\n"],
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let data_access = StatefulDataAccess::new(
      state,
      tree,
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();

    // Goto Insert-1 (Keep)
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(2));
      let insert_result = stateful.goto_insert_mode(
        &data_access,
        crate::state::ops::GotoInsertModeVariant::Keep,
      );
      assert_eq!(
        insert_result,
        StatefulValue::InsertMode(InsertStateful::default())
      );

      let tree = data_access.tree.clone();
      let actual_cursor = get_cursor_viewport(tree.clone());
      assert_eq!(actual_cursor.line_idx(), 0);
      assert_eq!(actual_cursor.char_idx(), 2);
      assert_eq!(actual_cursor.row_idx(), 0);
      assert_eq!(actual_cursor.column_idx(), 2);

      let expect_canvas = vec![
        "Should go to insert mode      ",
        "                              ",
        "                              ",
      ];
      let actual_canvas = make_canvas(tree.clone(), terminal_size);
      let actual_canvas = lock!(actual_canvas);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    let stateful = InsertStateful::default();
    // Insert-2
    {
      stateful.cursor_insert(
        &data_access,
        CursorInsertPayload::Text("Bye, ".to_compact_string()),
      );

      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 7);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 7);

      let viewport = get_viewport(tree.clone());
      let expect = vec!["ShBye, ould go to insert mode\n", ""];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        2,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "ShBye, ould go to insert mode ",
        "                              ",
        "                              ",
      ];
      let actual_canvas = make_canvas(tree.clone(), terminal_size);
      let actual_canvas = lock!(actual_canvas);
      assert_canvas(&actual_canvas, &expect_canvas);
    }
  }

  #[test]
  fn nowrap_goto_insert_append2() {
    test_log_init();

    let terminal_size = U16Size::new(30, 3);
    let (tree, state, bufs, buf, contents) = make_tree(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      vec!["Should go to insert mode\n"],
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let data_access = StatefulDataAccess::new(
      state,
      tree,
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();

    // Goto Insert-1 (Append)
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(2));
      let insert_result = stateful.goto_insert_mode(
        &data_access,
        crate::state::ops::GotoInsertModeVariant::Append,
      );
      assert_eq!(
        insert_result,
        StatefulValue::InsertMode(InsertStateful::default())
      );

      let tree = data_access.tree.clone();
      let actual_cursor = get_cursor_viewport(tree.clone());
      assert_eq!(actual_cursor.line_idx(), 0);
      assert_eq!(actual_cursor.char_idx(), 3);
      assert_eq!(actual_cursor.row_idx(), 0);
      assert_eq!(actual_cursor.column_idx(), 3);

      let expect_canvas = vec![
        "Should go to insert mode      ",
        "                              ",
        "                              ",
      ];
      let actual_canvas = make_canvas(tree.clone(), terminal_size);
      let actual_canvas = lock!(actual_canvas);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    let stateful = InsertStateful::default();
    // Insert-2
    {
      stateful.cursor_insert(
        &data_access,
        CursorInsertPayload::Text("Bye, ".to_compact_string()),
      );

      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 8);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 8);

      let viewport = get_viewport(tree.clone());
      let expect = vec!["ShoBye, uld go to insert mode\n", ""];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        2,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "ShoBye, uld go to insert mode ",
        "                              ",
        "                              ",
      ];
      let actual_canvas = make_canvas(tree.clone(), terminal_size);
      let actual_canvas = lock!(actual_canvas);
      assert_canvas(&actual_canvas, &expect_canvas);
    }
  }

  #[test]
  fn nowrap_goto_insert_append3() {
    test_log_init();

    let terminal_size = U16Size::new(30, 3);
    let (tree, state, bufs, buf, contents) = make_tree(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      vec!["Should go to insert mode\n"],
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let data_access = StatefulDataAccess::new(
      state,
      tree,
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();

    // Goto Insert-1 (Append)
    {
      stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(30));
      let insert_result = stateful.goto_insert_mode(
        &data_access,
        crate::state::ops::GotoInsertModeVariant::Append,
      );
      assert_eq!(
        insert_result,
        StatefulValue::InsertMode(InsertStateful::default())
      );

      let tree = data_access.tree.clone();
      let actual_cursor = get_cursor_viewport(tree.clone());
      assert_eq!(actual_cursor.line_idx(), 0);
      assert_eq!(actual_cursor.char_idx(), 24);
      assert_eq!(actual_cursor.row_idx(), 0);
      assert_eq!(actual_cursor.column_idx(), 24);

      let expect_canvas = vec![
        "Should go to insert mode      ",
        "                              ",
        "                              ",
      ];
      let actual_canvas = make_canvas(tree.clone(), terminal_size);
      let actual_canvas = lock!(actual_canvas);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    let stateful = InsertStateful::default();
    // Insert-2
    {
      stateful.cursor_insert(
        &data_access,
        CursorInsertPayload::Text("Bye, ".to_compact_string()),
      );

      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 29);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 29);

      let viewport = get_viewport(tree.clone());
      let expect = vec!["Should go to insert modeBye, \n", ""];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        2,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "Should go to insert modeBye,  ",
        "                              ",
        "                              ",
      ];
      let actual_canvas = make_canvas(tree.clone(), terminal_size);
      let actual_canvas = lock!(actual_canvas);
      assert_canvas(&actual_canvas, &expect_canvas);
    }
  }

  #[test]
  fn nowrap_goto_insert_newline4() {
    test_log_init();

    let terminal_size = U16Size::new(30, 3);
    let (tree, state, bufs, buf, contents) = make_tree(
      terminal_size,
      WindowLocalOptionsBuilder::default()
        .wrap(false)
        .build()
        .unwrap(),
      vec!["Should go to insert mode\n"],
    );

    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let data_access = StatefulDataAccess::new(
      state,
      tree,
      bufs,
      contents,
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();

    // Goto Insert-1 (NewLine)
    {
      let insert_result = stateful.goto_insert_mode(
        &data_access,
        crate::state::ops::GotoInsertModeVariant::NewLine,
      );
      assert_eq!(
        insert_result,
        StatefulValue::InsertMode(InsertStateful::default())
      );

      let tree = data_access.tree.clone();
      let actual_cursor = get_cursor_viewport(tree.clone());
      assert_eq!(actual_cursor.line_idx(), 1);
      assert_eq!(actual_cursor.char_idx(), 0);
      assert_eq!(actual_cursor.row_idx(), 1);
      assert_eq!(actual_cursor.column_idx(), 0);

      let expect_canvas = vec![
        "Should go to insert mode      ",
        "                              ",
        "                              ",
      ];
      let actual_canvas = make_canvas(tree.clone(), terminal_size);
      let actual_canvas = lock!(actual_canvas);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    let stateful = InsertStateful::default();
    // Insert-2
    {
      stateful.cursor_insert(
        &data_access,
        CursorInsertPayload::Text("Bye, ".to_compact_string()),
      );

      let tree = data_access.tree.clone();
      let actual1 = get_cursor_viewport(tree.clone());
      assert_eq!(actual1.line_idx(), 1);
      assert_eq!(actual1.char_idx(), 5);
      assert_eq!(actual1.row_idx(), 1);
      assert_eq!(actual1.column_idx(), 5);

      let viewport = get_viewport(tree.clone());
      let buf_eol = lock!(buf).options().end_of_line();
      let line1 = format!("Should go to insert mode{buf_eol}");
      let expect = vec![line1.as_str(), "Bye, \n", ""];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &viewport,
        &expect,
        0,
        3,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "Should go to insert mode      ",
        "Bye,                          ",
        "                              ",
      ];
      let actual_canvas = make_canvas(tree.clone(), terminal_size);
      let actual_canvas = lock!(actual_canvas);
      assert_canvas(&actual_canvas, &expect_canvas);
    }
  }
}
