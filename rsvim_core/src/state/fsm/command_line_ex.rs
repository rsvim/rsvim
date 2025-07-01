//! The command-line ex mode.

use crate::js::msg::{EventLoopToJsRuntimeMessage, ExCommandReq};
use crate::js::next_future_id;
use crate::prelude::*;
use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::ops::{Operation, cursor_ops};
use crate::ui::canvas::CursorStyle;
use crate::ui::tree::*;
use crate::ui::widget::command_line::{CommandLine, CommandLineNode};
use crate::ui::widget::window::Window;

use compact_str::{CompactString, ToCompactString};
use crossterm::event::{Event, KeyCode, KeyEventKind};
use tracing::trace;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The command-line ex mode.
pub struct CommandLineExStateful {}

impl CommandLineExStateful {
  fn _get_operation(&self, event: Event) -> Option<Operation> {
    match event {
      Event::FocusGained => None,
      Event::FocusLost => None,
      Event::Key(key_event) => match key_event.kind {
        KeyEventKind::Press => {
          trace!("Event::key:{:?}", key_event);
          match key_event.code {
            // KeyCode::Up | KeyCode::Char('k') => Some(Operation::CursorMoveUpBy(1)),
            // KeyCode::Down | KeyCode::Char('j') => Some(Operation::CursorMoveDownBy(1)),
            KeyCode::Left => Some(Operation::CursorMoveLeftBy(1)),
            KeyCode::Right => Some(Operation::CursorMoveRightBy(1)),
            KeyCode::Home => Some(Operation::CursorMoveLeftBy(usize::MAX)),
            KeyCode::End => Some(Operation::CursorMoveRightBy(usize::MAX)),
            KeyCode::Char(c) => Some(Operation::CursorInsert(c.to_compact_string())),
            KeyCode::Backspace => Some(Operation::CursorDelete(-1)),
            KeyCode::Delete => Some(Operation::CursorDelete(1)),
            KeyCode::Esc => Some(Operation::GotoNormalMode),
            KeyCode::Enter => Some(Operation::ConfirmExCommandAndGotoNormalMode),
            _ => None,
          }
        }
        KeyEventKind::Repeat => None,
        KeyEventKind::Release => None,
      },
      Event::Mouse(_mouse_event) => None,
      Event::Paste(ref _paste_string) => None,
      Event::Resize(_columns, _rows) => None,
    }
  }

  fn _command_line<'a>(&self, tree: &'a mut Tree) -> &'a mut CommandLine {
    debug_assert!(tree.command_line_id().is_some());
    let cmdline_id = tree.command_line_id().unwrap();
    debug_assert!(tree.node_mut(cmdline_id).is_some());
    let cmdline_node = tree.node_mut(cmdline_id).unwrap();
    debug_assert!(matches!(cmdline_node, TreeNode::CommandLine(_)));
    match cmdline_node {
      TreeNode::CommandLine(cmdline) => cmdline,
      _ => unreachable!(),
    }
  }
}

impl Stateful for CommandLineExStateful {
  fn handle(&self, data_access: StatefulDataAccess) -> StatefulValue {
    let event = data_access.event.clone();

    if let Some(op) = self._get_operation(event) {
      return self.handle_op(data_access, op);
    }

    StatefulValue::CommandLineExMode(CommandLineExStateful::default())
  }

  fn handle_op(&self, data_access: StatefulDataAccess, op: Operation) -> StatefulValue {
    match op {
      Operation::CursorMoveBy((_, _))
      | Operation::CursorMoveUpBy(_)
      | Operation::CursorMoveDownBy(_)
      | Operation::CursorMoveLeftBy(_)
      | Operation::CursorMoveRightBy(_)
      | Operation::CursorMoveTo((_, _)) => self.cursor_move(&data_access, op),
      Operation::GotoNormalMode => self.goto_normal_mode(&data_access),
      Operation::ConfirmExCommandAndGotoNormalMode => {
        self.confirm_ex_command_and_goto_normal_mode(&data_access)
      }
      Operation::CursorInsert(text) => self.cursor_insert(&data_access, text),
      Operation::CursorDelete(n) => self.cursor_delete(&data_access, n),
      _ => unreachable!(),
    }
  }
}

impl CommandLineExStateful {
  fn confirm_ex_command_and_goto_normal_mode(
    &self,
    data_access: &StatefulDataAccess,
  ) -> StatefulValue {
    let cmdline_content = self._goto_normal_mode_impl(data_access);
    let state = data_access.state.clone();
    let jsrt_tick_dispatcher = lock!(state).jsrt_tick_dispatcher().clone();

    let current_handle = tokio::runtime::Handle::current();
    current_handle.spawn_blocking(move || {
      jsrt_tick_dispatcher
        .blocking_send(EventLoopToJsRuntimeMessage::ExCommandReq(
          ExCommandReq::new(next_future_id(), cmdline_content),
        ))
        .unwrap();
    });

    StatefulValue::NormalMode(super::NormalStateful::default())
  }
}

impl CommandLineExStateful {
  fn _goto_normal_mode_impl(&self, data_access: &StatefulDataAccess) -> CompactString {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);

    debug_assert!(tree.command_line_id().is_some());
    let cmdline_id = tree.command_line_id().unwrap();
    let cmdline = tree.command_line_mut().unwrap();
    debug_assert!(cmdline.cursor_id().is_some());

    // Remove from current parent
    let cursor = match cmdline.remove_cursor().unwrap() {
      CommandLineNode::Cursor(mut cursor) => {
        cursor.set_style(&CursorStyle::SteadyBlock);
        cursor
      }
      _ => unreachable!(),
    };
    debug_assert!(cmdline.cursor_id().is_none());

    // Insert to new parent
    let current_window = tree.current_window_mut().unwrap();
    let cursor_viewport = current_window.cursor_viewport();
    trace!("before viewport:{:?}", current_window.viewport());
    trace!("before cursor_viewport:{:?}", cursor_viewport);
    let _current_window_id = current_window.id();
    let _previous_cursor = current_window.insert_cursor(cursor);
    debug_assert!(_previous_cursor.is_none());
    current_window.move_cursor_to(
      cursor_viewport.column_idx() as isize,
      cursor_viewport.row_idx() as isize,
    );

    // Clear command-line contents.
    let contents = data_access.contents.clone();
    let mut contents = lock!(contents);
    let cmdline_content = contents.command_line_content().rope().to_compact_string();

    cursor_ops::cursor_clear(&mut tree, cmdline_id, contents.command_line_content_mut());

    let cmdline_content = cmdline_content.trim();
    debug_assert!(cmdline_content.starts_with(":"));
    let cmdline_content = &cmdline_content[1..];
    CompactString::new(cmdline_content)
  }

  fn goto_normal_mode(&self, data_access: &StatefulDataAccess) -> StatefulValue {
    self._goto_normal_mode_impl(data_access);

    StatefulValue::NormalMode(super::NormalStateful::default())
  }
}

impl CommandLineExStateful {
  fn cursor_move(&self, data_access: &StatefulDataAccess, op: Operation) -> StatefulValue {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);
    debug_assert!(tree.command_line_id().is_some());
    let cmdline_id = tree.command_line_id().unwrap();
    let contents = data_access.contents.clone();
    let contents = lock!(contents);

    cursor_ops::cursor_move(
      &mut tree,
      cmdline_id,
      contents.command_line_content(),
      op,
      true,
    );

    StatefulValue::CommandLineExMode(CommandLineExStateful::default())
  }
}

impl CommandLineExStateful {
  fn cursor_insert(
    &self,
    data_access: &StatefulDataAccess,
    payload: CompactString,
  ) -> StatefulValue {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);
    debug_assert!(tree.command_line_id().is_some());
    let cmdline_id = tree.command_line_id().unwrap();
    let contents = data_access.contents.clone();
    let mut contents = lock!(contents);

    cursor_ops::cursor_insert(
      &mut tree,
      cmdline_id,
      contents.command_line_content_mut(),
      payload,
    );

    StatefulValue::CommandLineExMode(CommandLineExStateful::default())
  }
}

impl CommandLineExStateful {
  fn cursor_delete(&self, data_access: &StatefulDataAccess, n: isize) -> StatefulValue {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);
    let contents = data_access.contents.clone();
    let mut contents = lock!(contents);
    let text = contents.command_line_content_mut();

    let cmdline = tree.command_line_mut().unwrap();
    let cmdline_id = cmdline.id();
    let cursor_viewport = cmdline.cursor_viewport();
    let cursor_line_idx = cursor_viewport.line_idx();
    debug_assert_eq!(cursor_line_idx, 0);
    let cursor_char_idx = cursor_viewport.char_idx();
    debug_assert!(text.rope().get_line(cursor_line_idx).is_some());

    let to_be_deleted_n = if n > 0 {
      // Delete to right side
      n
    } else {
      // Delete to left side, do NOT delete the first ':' char.
      let left_bound = std::cmp::max(1_usize, cursor_char_idx.saturating_add_signed(n));
      -(cursor_char_idx.saturating_sub(left_bound) as isize)
    };

    cursor_ops::cursor_delete(&mut tree, cmdline_id, text, to_be_deleted_n);

    StatefulValue::CommandLineExMode(CommandLineExStateful::default())
  }
}

// spellchecker:off
#[cfg(test)]
#[allow(unused_imports, dead_code)]
mod tests_util {
  use super::*;

  use crate::buf::opt::BufferLocalOptionsBuilder;
  use crate::buf::text::Text;
  use crate::buf::{BufferArc, BuffersManagerArc};
  use crate::content::{TextContents, TextContentsArc};
  use crate::prelude::*;
  use crate::state::{State, StateArc};
  use crate::test::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::test::log::init as test_log_init;
  use crate::test::tree::{make_tree_with_buffers, make_tree_with_buffers_cmdline};
  use crate::ui::canvas::{Canvas, CanvasArc};
  use crate::ui::tree::TreeArc;
  use crate::ui::viewport::{
    CursorViewport, CursorViewportArc, Viewport, ViewportArc, ViewportSearchDirection,
  };
  use crate::ui::widget::command_line::CommandLine;
  use crate::ui::widget::window::{WindowLocalOptions, WindowLocalOptionsBuilder};

  use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
  use std::collections::BTreeMap;
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
    let (jsrt_tick_dispatcher, _jsrt_tick_queue) = channel(1);
    let state = State::to_arc(State::new(jsrt_tick_dispatcher));
    (tree, state, bufs, buf, contents)
  }

  pub fn make_canvas(tree: TreeArc, terminal_size: U16Size) -> CanvasArc {
    let canvas = Canvas::new(terminal_size);
    let canvas = Canvas::to_arc(canvas);
    let tree = lock!(tree);
    tree.draw(canvas.clone());
    canvas
  }

  #[allow(clippy::too_many_arguments)]
  pub fn assert_viewport_scroll(
    text: &Text,
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
          r, payload, expect[*r as usize]
        );
        assert_eq!(payload, expect[*r as usize]);
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
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests_get_operation {
  use super::tests_util::*;
  use super::*;

  use crate::buf::opt::BufferLocalOptionsBuilder;
  use crate::buf::{BufferArc, BuffersManagerArc};
  use crate::prelude::*;
  use crate::state::{self, State, StateArc};
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
  fn get1() {
    test_log_init();

    let stateful = CommandLineExStateful::default();
    assert!(matches!(
      stateful._get_operation(Event::Key(KeyEvent::new(
        KeyCode::Char('i'),
        KeyModifiers::empty()
      ))),
      Some(Operation::CursorInsert(_))
    ));
    assert!(matches!(
      stateful._get_operation(Event::Key(KeyEvent::new(
        KeyCode::Char('j'),
        KeyModifiers::empty()
      ))),
      Some(Operation::CursorInsert(_))
    ));
    assert!(matches!(
      stateful._get_operation(Event::Key(KeyEvent::new(
        KeyCode::Esc,
        KeyModifiers::empty()
      ))),
      Some(Operation::GotoNormalMode)
    ));
    assert!(matches!(
      stateful._get_operation(Event::Key(KeyEvent::new(
        KeyCode::Backspace,
        KeyModifiers::empty()
      ))),
      Some(Operation::CursorDelete(_))
    ));
    assert!(matches!(
      stateful._get_operation(Event::Key(KeyEvent::new(
        KeyCode::Delete,
        KeyModifiers::empty()
      ))),
      Some(Operation::CursorDelete(_))
    ));
  }
}
#[cfg(test)]
#[allow(unused_imports)]
mod tests_goto_normal_mode {
  use super::tests_util::*;
  use super::*;

  use crate::buf::opt::BufferLocalOptionsBuilder;
  use crate::buf::{BufferArc, BuffersManagerArc};
  use crate::prelude::*;
  use crate::state::{self, State, StateArc};
  use crate::test::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::test::log::init as test_log_init;
  use crate::test::tree::make_tree_with_buffers;
  use crate::ui::tree::TreeArc;
  use crate::ui::viewport::{
    CursorViewport, CursorViewportArc, Viewport, ViewportArc, ViewportSearchDirection,
  };
  use crate::ui::widget::window::{WindowLocalOptions, WindowLocalOptionsBuilder};

  use crate::state::fsm::NormalStateful;
  use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
  use std::collections::BTreeMap;
  use tracing::info;

  #[test]
  fn nowrap1() {
    test_log_init();

    let terminal_size = U16Size::new(11, 5);
    let window_options = WindowLocalOptionsBuilder::default()
      .wrap(false)
      .build()
      .unwrap();
    let lines = vec![];
    let (tree, state, bufs, buf, contents) =
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
      assert_eq!(actual1.char_idx(), 1);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 1);

      let viewport = lock!(tree.clone()).command_line().unwrap().viewport();
      let buf_eol = lock!(buf).options().end_of_line();
      let text1 = CompactString::new(format!(":{buf_eol}"));
      let expect = vec![text1.as_str()];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport_scroll(
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
      stateful.cursor_insert(&data_access, CompactString::new("Bye"));

      let tree = data_access.tree.clone();
      let actual1 = lock!(tree.clone())
        .command_line()
        .unwrap()
        .cursor_viewport();
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 4);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 4);

      let viewport = lock!(tree.clone()).command_line().unwrap().viewport();
      let buf_eol = lock!(buf).options().end_of_line();
      let text1 = CompactString::new(format!(":Bye{buf_eol}"));
      let expect = vec![text1.as_str()];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport_scroll(
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

      let viewport = lock!(tree.clone()).command_line().unwrap().viewport();
      let expect = vec![""];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport_scroll(
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
#[allow(unused_imports)]
mod tests_confirm_ex_command_and_goto_normal_mode {
  use super::tests_util::*;
  use super::*;

  use crate::buf::opt::BufferLocalOptionsBuilder;
  use crate::buf::{BufferArc, BuffersManagerArc};
  use crate::prelude::*;
  use crate::state::{self, State, StateArc};
  use crate::test::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::test::log::init as test_log_init;
  use crate::test::tree::make_tree_with_buffers;
  use crate::ui::tree::TreeArc;
  use crate::ui::viewport::{
    CursorViewport, CursorViewportArc, Viewport, ViewportArc, ViewportSearchDirection,
  };
  use crate::ui::widget::window::{WindowLocalOptions, WindowLocalOptionsBuilder};

  use crate::state::fsm::NormalStateful;
  use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
  use std::collections::BTreeMap;
  use tracing::info;

  #[test]
  fn nowrap1() {
    test_log_init();

    let terminal_size = U16Size::new(11, 5);
    let window_options = WindowLocalOptionsBuilder::default()
      .wrap(false)
      .build()
      .unwrap();
    let lines = vec![];
    let (tree, state, bufs, buf, contents) =
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
      assert_eq!(actual1.char_idx(), 1);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 1);

      let viewport = lock!(tree.clone()).command_line().unwrap().viewport();
      let buf_eol = lock!(buf).options().end_of_line();
      let text1 = CompactString::new(format!(":{buf_eol}"));
      let expect = vec![text1.as_str()];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport_scroll(
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
      stateful.cursor_insert(&data_access, CompactString::new("Bye"));

      let tree = data_access.tree.clone();
      let actual1 = lock!(tree.clone())
        .command_line()
        .unwrap()
        .cursor_viewport();
      assert_eq!(actual1.line_idx(), 0);
      assert_eq!(actual1.char_idx(), 4);
      assert_eq!(actual1.row_idx(), 0);
      assert_eq!(actual1.column_idx(), 4);

      let viewport = lock!(tree.clone()).command_line().unwrap().viewport();
      let buf_eol = lock!(buf).options().end_of_line();
      let text1 = CompactString::new(format!(":Bye{buf_eol}"));
      let expect = vec![text1.as_str()];
      let expect_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport_scroll(
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
      let cmdline_content = stateful._goto_normal_mode_impl(&data_access);
      info!("cmdline content:{cmdline_content:?}");
      assert_eq!("Bye", cmdline_content.as_str());
    }
  }
}
// spellchecker:on
