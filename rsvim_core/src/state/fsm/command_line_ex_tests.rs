#![allow(unused_imports, dead_code)]

use super::command_line_ex::*;

use crate::buf::opt::FileFormatOption;
use crate::buf::opt::{BufferOptions, BufferOptionsBuilder};
use crate::buf::text::Text;
use crate::buf::{BufferArc, BuffersManagerArc};
use crate::content::{TextContents, TextContentsArc};
use crate::prelude::*;
use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::ops::Operation;
use crate::state::ops::cursor_ops;
use crate::state::{State, StateArc};
use crate::tests::buf::{make_buffer_from_lines, make_buffers_manager};
use crate::tests::log::init as test_log_init;
use crate::tests::tree::{
  make_tree_with_buffers, make_tree_with_buffers_cmdline,
};
use crate::ui::canvas::{Canvas, CanvasArc};
use crate::ui::tree::*;
use crate::ui::viewport::{
  CursorViewport, CursorViewportArc, Viewport, ViewportArc,
  ViewportSearchDirection,
};
use crate::ui::widget::command_line::CommandLine;
use crate::ui::widget::window::{WindowOptions, WindowOptionsBuilder};

use compact_str::{CompactString, ToCompactString};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::collections::BTreeMap;
use tokio::sync::mpsc::{Receiver, Sender, channel};

pub fn make_tree(
  terminal_size: U16Size,
  window_local_opts: WindowOptions,
  lines: Vec<&str>,
) -> (
  TreeArc,
  StateArc,
  BuffersManagerArc,
  BufferArc,
  TextContentsArc,
) {
  let buf_opts = BufferOptionsBuilder::default().build().unwrap();
  let buf = make_buffer_from_lines(terminal_size, buf_opts, lines);
  let bufs = make_buffers_manager(buf_opts, vec![buf.clone()]);
  let tree =
    make_tree_with_buffers(terminal_size, window_local_opts, bufs.clone());
  let (jsrt_tick_dispatcher, _jsrt_tick_queue) = channel(1);
  let state = State::to_arc(State::new(jsrt_tick_dispatcher));
  let contents = TextContents::to_arc(TextContents::new(terminal_size));
  (tree, state, bufs, buf, contents)
}

pub fn make_tree_with_cmdline_and_buffer_options(
  terminal_size: U16Size,
  buffer_local_opts: BufferOptions,
  window_local_opts: WindowOptions,
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
  let contents = TextContents::to_arc(TextContents::new(terminal_size));
  let tree = make_tree_with_buffers_cmdline(
    terminal_size,
    window_local_opts,
    bufs.clone(),
    contents.clone(),
  );
  let (jsrt_tick_dispatcher, _jsrt_tick_queue) = channel(1);
  let state = State::to_arc(State::new(jsrt_tick_dispatcher));
  (tree, state, bufs, buf, contents)
}

pub fn make_tree_with_cmdline(
  terminal_size: U16Size,
  window_local_opts: WindowOptions,
  lines: Vec<&str>,
) -> (
  TreeArc,
  StateArc,
  BuffersManagerArc,
  BufferArc,
  TextContentsArc,
) {
  let buf_opts = BufferOptionsBuilder::default().build().unwrap();
  make_tree_with_cmdline_and_buffer_options(
    terminal_size,
    buf_opts,
    window_local_opts,
    lines,
  )
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
  text: &Text,
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
    info!("expect row[{}]:{:?}", i, e);
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

  let buflines = text.rope().lines_at(actual.start_line_idx());
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
mod tests_goto_normal_mode {
  use super::*;

  use crate::buf::opt::BufferOptionsBuilder;
  use crate::buf::{BufferArc, BuffersManagerArc};
  use crate::prelude::*;
  use crate::state::ops::CursorInsertPayload;
  use crate::state::{self, State, StateArc};
  use crate::tests::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::tests::log::init as test_log_init;
  use crate::tests::tree::make_tree_with_buffers;
  use crate::ui::tree::TreeArc;
  use crate::ui::viewport::{
    CursorViewport, CursorViewportArc, Viewport, ViewportArc,
    ViewportSearchDirection,
  };
  use crate::ui::widget::window::{WindowOptions, WindowOptionsBuilder};

  use crate::state::fsm::NormalStateful;
  use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
  };
  use std::collections::BTreeMap;

  #[test]
  fn nowrap1() {
    test_log_init();

    let terminal_size = U16Size::new(11, 5);
    let window_options =
      WindowOptionsBuilder::default().wrap(false).build().unwrap();
    let lines = vec![];
    let (tree, state, bufs, _buf, contents) =
      make_tree_with_cmdline(terminal_size, window_options, lines);

    let prev_cursor_viewport = lock!(tree.clone())
      .current_window()
      .unwrap()
      .cursor_viewport();
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
    let stateful = NormalStateful::default();

    // Prepare
    {
      stateful.goto_command_line_ex_mode(&data_access);

      let tree = data_access.tree.clone();
      let actual1 = lock!(tree.clone())
        .command_line()
        .unwrap()
        .cursor_viewport();
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 0);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 0);

      let viewport =
        lock!(tree.clone()).command_line().unwrap().input_viewport();
      let expect = vec![""];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0)].into_iter().collect();
      assert_viewport(
        lock!(contents).command_line_content(),
        &viewport,
        &expect,
        0,
        1,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "           ",
        "           ",
        "           ",
        "           ",
        ":          ",
      ];
      let actual_canvas = make_canvas(tree.clone(), terminal_size);
      let actual_canvas = lock!(actual_canvas);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    let stateful = CommandLineExStateful::default();

    // Insert-1
    {
      stateful.cursor_insert(
        &data_access,
        CursorInsertPayload::Text("Bye".to_compact_string()),
      );

      let tree = data_access.tree.clone();
      let actual1 = lock!(tree.clone())
        .command_line()
        .unwrap()
        .cursor_viewport();
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 3);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 3);

      let viewport =
        lock!(tree.clone()).command_line().unwrap().input_viewport();
      let cmdline_eol = lock!(contents)
        .command_line_content()
        .options()
        .end_of_line();
      let line0 = format!("Bye{cmdline_eol}");
      let expect = vec![line0.as_str()];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0)].into_iter().collect();
      assert_viewport(
        lock!(contents).command_line_content(),
        &viewport,
        &expect,
        0,
        1,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "           ",
        "           ",
        "           ",
        "           ",
        ":Bye       ",
      ];
      let actual_canvas = make_canvas(tree.clone(), terminal_size);
      let actual_canvas = lock!(actual_canvas);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Goto Normal-2
    {
      stateful.goto_normal_mode(&data_access);

      let tree = data_access.tree.clone();
      let actual1 = lock!(tree.clone())
        .command_line()
        .unwrap()
        .cursor_viewport();
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 0);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 0);

      let viewport =
        lock!(tree.clone()).command_line().unwrap().input_viewport();
      let expect = vec![""];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0)].into_iter().collect();
      assert_viewport(
        lock!(contents).command_line_content(),
        &viewport,
        &expect,
        0,
        1,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "           ",
        "           ",
        "           ",
        "           ",
        "           ",
      ];
      let actual_canvas = make_canvas(tree.clone(), terminal_size);
      let actual_canvas = lock!(actual_canvas);
      assert_canvas(&actual_canvas, &expect_canvas);
    }
  }

  #[test]
  fn nowrap1_crlf_win() {
    test_log_init();

    let terminal_size = U16Size::new(11, 5);
    let buf_opts = BufferOptionsBuilder::default()
      .file_format(FileFormatOption::Dos)
      .build()
      .unwrap();
    let window_options =
      WindowOptionsBuilder::default().wrap(false).build().unwrap();
    let lines = vec![];
    let (tree, state, bufs, _buf, contents) =
      make_tree_with_cmdline_and_buffer_options(
        terminal_size,
        buf_opts,
        window_options,
        lines,
      );

    let prev_cursor_viewport = lock!(tree.clone())
      .current_window()
      .unwrap()
      .cursor_viewport();
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
    let stateful = NormalStateful::default();

    // Prepare
    {
      stateful.goto_command_line_ex_mode(&data_access);

      let tree = data_access.tree.clone();
      let actual1 = lock!(tree.clone())
        .command_line()
        .unwrap()
        .cursor_viewport();
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 0);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 0);

      let viewport =
        lock!(tree.clone()).command_line().unwrap().input_viewport();
      let expect = vec![""];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0)].into_iter().collect();
      assert_viewport(
        lock!(contents).command_line_content(),
        &viewport,
        &expect,
        0,
        1,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "           ",
        "           ",
        "           ",
        "           ",
        ":          ",
      ];
      let actual_canvas = make_canvas(tree.clone(), terminal_size);
      let actual_canvas = lock!(actual_canvas);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    let stateful = CommandLineExStateful::default();

    // Insert-1
    {
      stateful.cursor_insert(
        &data_access,
        CursorInsertPayload::Text("Bye".to_compact_string()),
      );

      let tree = data_access.tree.clone();
      let actual1 = lock!(tree.clone())
        .command_line()
        .unwrap()
        .cursor_viewport();
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 3);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 3);

      let viewport =
        lock!(tree.clone()).command_line().unwrap().input_viewport();
      let cmdline_eol = lock!(contents)
        .command_line_content()
        .options()
        .end_of_line();
      let line0 = format!("Bye{cmdline_eol}");
      let expect = vec![line0.as_str()];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0)].into_iter().collect();
      assert_viewport(
        lock!(contents).command_line_content(),
        &viewport,
        &expect,
        0,
        1,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "           ",
        "           ",
        "           ",
        "           ",
        ":Bye       ",
      ];
      let actual_canvas = make_canvas(tree.clone(), terminal_size);
      let actual_canvas = lock!(actual_canvas);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Goto Normal-2
    {
      stateful.goto_normal_mode(&data_access);

      let tree = data_access.tree.clone();
      let actual1 = lock!(tree.clone())
        .command_line()
        .unwrap()
        .cursor_viewport();
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 0);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 0);

      let viewport =
        lock!(tree.clone()).command_line().unwrap().input_viewport();
      let expect = vec![""];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0)].into_iter().collect();
      assert_viewport(
        lock!(contents).command_line_content(),
        &viewport,
        &expect,
        0,
        1,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "           ",
        "           ",
        "           ",
        "           ",
        "           ",
      ];
      let actual_canvas = make_canvas(tree.clone(), terminal_size);
      let actual_canvas = lock!(actual_canvas);
      assert_canvas(&actual_canvas, &expect_canvas);
    }
  }
}

#[cfg(test)]
mod tests_confirm_ex_command_and_goto_normal_mode {
  use super::*;

  use crate::buf::opt::BufferOptionsBuilder;
  use crate::buf::{BufferArc, BuffersManagerArc};
  use crate::prelude::*;
  use crate::state::ops::CursorInsertPayload;
  use crate::state::{self, State, StateArc};
  use crate::tests::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::tests::log::init as test_log_init;
  use crate::tests::tree::make_tree_with_buffers;
  use crate::ui::tree::TreeArc;
  use crate::ui::viewport::{
    CursorViewport, CursorViewportArc, Viewport, ViewportArc,
    ViewportSearchDirection,
  };
  use crate::ui::widget::window::{WindowOptions, WindowOptionsBuilder};

  use crate::state::fsm::NormalStateful;
  use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
  };
  use std::collections::BTreeMap;

  #[test]
  fn nowrap1() {
    test_log_init();

    let terminal_size = U16Size::new(11, 5);
    let window_options =
      WindowOptionsBuilder::default().wrap(false).build().unwrap();
    let lines = vec![];
    let (tree, state, bufs, _buf, contents) =
      make_tree_with_cmdline(terminal_size, window_options, lines);

    let prev_cursor_viewport = lock!(tree.clone())
      .current_window()
      .unwrap()
      .cursor_viewport();
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
    let stateful = NormalStateful::default();

    // Prepare
    {
      stateful.goto_command_line_ex_mode(&data_access);

      let tree = data_access.tree.clone();
      let actual1 = lock!(tree.clone())
        .command_line()
        .unwrap()
        .cursor_viewport();
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 0);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 0);

      let viewport =
        lock!(tree.clone()).command_line().unwrap().input_viewport();
      let expect = vec![""];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0)].into_iter().collect();
      assert_viewport(
        lock!(contents).command_line_content(),
        &viewport,
        &expect,
        0,
        1,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "           ",
        "           ",
        "           ",
        "           ",
        ":          ",
      ];
      let actual_canvas = make_canvas(tree.clone(), terminal_size);
      let actual_canvas = lock!(actual_canvas);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    let stateful = CommandLineExStateful::default();

    // Insert-1
    {
      stateful.cursor_insert(
        &data_access,
        CursorInsertPayload::Text(
          "Bye1 Bye2 Bye3 Bye4 Bye5 Bye6 Bye7".to_compact_string(),
        ),
      );

      let tree = data_access.tree.clone();
      let actual1 = lock!(tree.clone())
        .command_line()
        .unwrap()
        .cursor_viewport();
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 34);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 9);

      let viewport =
        lock!(tree.clone()).command_line().unwrap().input_viewport();
      let cmdline_eol = lock!(contents)
        .command_line_content()
        .options()
        .end_of_line();
      let line0 = format!("Bye6 Bye7{cmdline_eol}");
      let expect = vec![line0.as_str()];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0)].into_iter().collect();
      assert_viewport(
        lock!(contents).command_line_content(),
        &viewport,
        &expect,
        0,
        1,
        &expect_fills,
        &expect_fills,
      );

      let expect_canvas = vec![
        "           ",
        "           ",
        "           ",
        "           ",
        ":Bye6 Bye7 ",
      ];
      let actual_canvas = make_canvas(tree.clone(), terminal_size);
      let actual_canvas = lock!(actual_canvas);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Goto Normal-2
    {
      let cmdline_content = stateful._goto_normal_mode_impl(&data_access);
      info!("cmdline content:{cmdline_content:?}");
      // After go to normal mode, content is cleared
      assert_eq!("", cmdline_content.as_str());
    }
  }
}
