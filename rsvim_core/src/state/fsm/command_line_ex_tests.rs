#![allow(unused_imports, dead_code, unused_variables)]

use super::command_line_ex::*;
use crate::buf::BufferArc;
use crate::buf::BuffersManagerArc;
use crate::buf::opt::BufferOptions;
use crate::buf::opt::BufferOptionsBuilder;
use crate::buf::opt::FileFormatOption;
use crate::buf::text::Text;
use crate::content::TextContents;
use crate::content::TextContentsArc;
use crate::prelude::*;
use crate::state::StateDataAccess;
use crate::state::StateMachine;
use crate::state::Stateful;
use crate::state::fsm::NormalStateful;
use crate::state::ops::CursorInsertPayload;
use crate::state::ops::Operation;
use crate::state::ops::cursor_ops;
use crate::tests::buf::make_buffer_from_lines;
use crate::tests::buf::make_buffers_manager;
use crate::tests::fsm::make_fsm;
use crate::tests::fsm::make_fsm_default_bufopts;
use crate::tests::fsm::make_fsm_with_cmdline;
use crate::tests::fsm::make_fsm_with_cmdline_default_bufopts;
use crate::tests::log::init as test_log_init;
use crate::tests::tree::make_tree_with_buffers;
use crate::tests::tree::make_tree_with_buffers_cmdline;
use crate::tests::viewport::assert_canvas;
use crate::tests::viewport::assert_viewport;
use crate::tests::viewport::make_tree_canvas;
use crate::ui::canvas::Canvas;
use crate::ui::canvas::CanvasArc;
use crate::ui::tree::*;
use crate::ui::viewport::CursorViewport;
use crate::ui::viewport::CursorViewportArc;
use crate::ui::viewport::Viewport;
use crate::ui::viewport::ViewportArc;
use crate::ui::viewport::ViewportSearchDirection;
use crate::ui::widget::command_line::Cmdline;
use crate::ui::widget::window::opt::WindowOptions;
use crate::ui::widget::window::opt::WindowOptionsBuilder;
use compact_str::CompactString;
use compact_str::ToCompactString;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::unbounded_channel;

#[cfg(test)]
mod tests_goto_normal_mode {
  use super::*;

  #[test]
  fn nowrap1() {
    test_log_init();

    let terminal_size = size!(11, 5);
    let window_options =
      WindowOptionsBuilder::default().wrap(false).build().unwrap();
    let lines = vec![];
    let (event, tree, bufs, _buf, contents, data_access) =
      make_fsm_with_cmdline_default_bufopts(
        terminal_size,
        window_options,
        lines,
      );

    let prev_cursor_viewport = lock!(tree.clone())
      .current_window()
      .unwrap()
      .cursor_viewport();
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let stateful = NormalStateful::default();

    // Prepare
    {
      stateful.goto_command_line_ex_mode(&data_access);

      let tree = data_access.tree.clone();
      let actual1 = lock!(tree.clone())
        .command_line()
        .unwrap()
        .input_cursor_viewport();
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
        lock!(contents).command_line_input(),
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
      let actual_canvas = make_tree_canvas(tree.clone(), terminal_size);
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
        .input_cursor_viewport();
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 3);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 3);

      let viewport =
        lock!(tree.clone()).command_line().unwrap().input_viewport();
      let cmdline_eol =
        lock!(contents).command_line_input().options().end_of_line();
      let line0 = format!("Bye{cmdline_eol}");
      let expect = vec![line0.as_str()];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0)].into_iter().collect();
      assert_viewport(
        lock!(contents).command_line_input(),
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
      let actual_canvas = make_tree_canvas(tree.clone(), terminal_size);
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
        .input_cursor_viewport();
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
        lock!(contents).command_line_input(),
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
      let actual_canvas = make_tree_canvas(tree.clone(), terminal_size);
      let actual_canvas = lock!(actual_canvas);
      assert_canvas(&actual_canvas, &expect_canvas);
    }
  }

  #[test]
  fn nowrap1_crlf_win() {
    test_log_init();

    let terminal_size = size!(11, 5);
    let buf_opts = BufferOptionsBuilder::default()
      .file_format(FileFormatOption::Dos)
      .build()
      .unwrap();
    let window_options =
      WindowOptionsBuilder::default().wrap(false).build().unwrap();
    let lines = vec![];
    let (event, tree, bufs, _buf, contents, data_access) =
      make_fsm_with_cmdline(terminal_size, buf_opts, window_options, lines);

    let prev_cursor_viewport = lock!(tree.clone())
      .current_window()
      .unwrap()
      .cursor_viewport();
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let stateful = NormalStateful::default();

    // Prepare
    {
      stateful.goto_command_line_ex_mode(&data_access);

      let tree = data_access.tree.clone();
      let actual1 = lock!(tree.clone())
        .command_line()
        .unwrap()
        .input_cursor_viewport();
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
        lock!(contents).command_line_input(),
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
      let actual_canvas = make_tree_canvas(tree.clone(), terminal_size);
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
        .input_cursor_viewport();
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 3);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 3);

      let viewport =
        lock!(tree.clone()).command_line().unwrap().input_viewport();
      let cmdline_eol =
        lock!(contents).command_line_input().options().end_of_line();
      let line0 = format!("Bye{cmdline_eol}");
      let expect = vec![line0.as_str()];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0)].into_iter().collect();
      assert_viewport(
        lock!(contents).command_line_input(),
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
      let actual_canvas = make_tree_canvas(tree.clone(), terminal_size);
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
        .input_cursor_viewport();
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
        lock!(contents).command_line_input(),
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
      let actual_canvas = make_tree_canvas(tree.clone(), terminal_size);
      let actual_canvas = lock!(actual_canvas);
      assert_canvas(&actual_canvas, &expect_canvas);
    }
  }
}

#[cfg(test)]
mod tests_confirm_ex_command_and_goto_normal_mode {
  use super::*;

  #[test]
  fn nowrap1() {
    test_log_init();

    let terminal_size = size!(11, 5);
    let window_options =
      WindowOptionsBuilder::default().wrap(false).build().unwrap();
    let lines = vec![];
    let (event, tree, bufs, _buf, contents, data_access) =
      make_fsm_with_cmdline_default_bufopts(
        terminal_size,
        window_options,
        lines,
      );

    let prev_cursor_viewport = lock!(tree.clone())
      .current_window()
      .unwrap()
      .cursor_viewport();
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let stateful = NormalStateful::default();

    // Prepare
    {
      stateful.goto_command_line_ex_mode(&data_access);

      let tree = data_access.tree.clone();
      let actual1 = lock!(tree.clone())
        .command_line()
        .unwrap()
        .input_cursor_viewport();
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
        lock!(contents).command_line_input(),
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
      let actual_canvas = make_tree_canvas(tree.clone(), terminal_size);
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
        .input_cursor_viewport();
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 34);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 9);

      let viewport =
        lock!(tree.clone()).command_line().unwrap().input_viewport();
      let cmdline_eol =
        lock!(contents).command_line_input().options().end_of_line();
      let line0 = format!("Bye6 Bye7{cmdline_eol}");
      let expect = vec![line0.as_str()];
      let expect_fills: BTreeMap<usize, usize> =
        vec![(0, 0)].into_iter().collect();
      assert_viewport(
        lock!(contents).command_line_input(),
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
      let actual_canvas = make_tree_canvas(tree.clone(), terminal_size);
      let actual_canvas = lock!(actual_canvas);
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Goto Normal-2
    {
      let cmdline_input_content = stateful._goto_normal_mode_impl(&data_access);
      info!("cmdline content:{cmdline_input_content:?}");
      assert_eq!(
        "Bye1 Bye2 Bye3 Bye4 Bye5 Bye6 Bye7",
        cmdline_input_content.as_str()
      );
    }
  }
}
