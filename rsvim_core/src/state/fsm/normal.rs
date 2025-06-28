//! The normal mode.

use crate::lock;
use crate::state::fsm::quit::QuitStateful;
use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValueDispatcher};
use crate::state::ops::Operation;
use crate::state::ops::cursor_ops;
use crate::ui::canvas::CursorStyle;
use crate::ui::tree::*;
use crate::ui::widget::window::Window;

use compact_str::ToCompactString;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use tracing::trace;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The finite-state-machine for normal mode.
pub struct NormalStateful {}

impl NormalStateful {
  fn _get_operation(&self, event: Event) -> Option<Operation> {
    match event {
      Event::FocusGained => None,
      Event::FocusLost => None,
      Event::Key(key_event) => match key_event.kind {
        KeyEventKind::Press => {
          trace!("Event::key:{:?}", key_event);
          match key_event.code {
            KeyCode::Up | KeyCode::Char('k') => Some(Operation::CursorMoveUpBy(1)),
            KeyCode::Down | KeyCode::Char('j') => Some(Operation::CursorMoveDownBy(1)),
            KeyCode::Left | KeyCode::Char('h') => Some(Operation::CursorMoveLeftBy(1)),
            KeyCode::Right | KeyCode::Char('l') => Some(Operation::CursorMoveRightBy(1)),
            KeyCode::Home => Some(Operation::CursorMoveLeftBy(usize::MAX)),
            KeyCode::End => Some(Operation::CursorMoveRightBy(usize::MAX)),
            KeyCode::Char('i') => Some(Operation::GotoInsertMode),
            KeyCode::Char(':') => Some(Operation::GotoCommandLineExMode),
            // KeyCode::Char('/') => Some(Operation::GotoCommandLineSearchForwardMode),
            // KeyCode::Char('?') => Some(Operation::GotoCommandLineSearchBackwardMode),
            KeyCode::Esc => Some(Operation::EditorQuit),
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

  fn _current_window<'a>(&self, tree: &'a mut Tree) -> &'a mut Window {
    debug_assert!(tree.current_window_id().is_some());
    let current_window_id = tree.current_window_id().unwrap();
    debug_assert!(tree.cursor_id().is_some());
    debug_assert!(tree.parent_id(tree.cursor_id().unwrap()).is_some());
    debug_assert_eq!(
      current_window_id,
      tree.parent_id(tree.cursor_id().unwrap()).unwrap()
    );
    debug_assert!(tree.node_mut(current_window_id).is_some());
    let current_window_node = tree.node_mut(current_window_id).unwrap();
    debug_assert!(matches!(current_window_node, TreeNodeDispatcher::Window(_)));
    match current_window_node {
      TreeNodeDispatcher::Window(current_window) => current_window,
      _ => unreachable!(),
    }
  }
}

impl Stateful for NormalStateful {
  fn handle(&self, data_access: StatefulDataAccess) -> StatefulValueDispatcher {
    let event = data_access.event.clone();

    if let Some(op) = self._get_operation(event) {
      return self.handle_op(data_access, op);
    }

    StatefulValueDispatcher::NormalMode(NormalStateful::default())
  }

  fn handle_op(&self, data_access: StatefulDataAccess, op: Operation) -> StatefulValueDispatcher {
    match op {
      Operation::GotoInsertMode => self.goto_insert_mode(&data_access),
      Operation::GotoCommandLineExMode => self.goto_command_line_ex_mode(&data_access),
      // Operation::GotoCommandLineSearchForwardMode => {
      //   self.goto_command_line_search_forward_mode(&data_access)
      // }
      // Operation::GotoCommandLineSearchBackwardMode => {
      //   self.goto_command_line_search_backward_mode(&data_access)
      // }
      Operation::EditorQuit => self.editor_quit(&data_access),
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

impl NormalStateful {
  pub fn goto_command_line_ex_mode(
    &self,
    data_access: &StatefulDataAccess,
  ) -> StatefulValueDispatcher {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);

    debug_assert!(tree.cursor_id().is_some());
    let cursor_id = tree.cursor_id().unwrap();

    // Remove from current parent
    debug_assert!(tree.parent_id(cursor_id).is_some());
    let cursor_parent_id = tree.parent_id(cursor_id).unwrap();
    debug_assert!(tree.current_window_id().is_some());
    debug_assert_eq!(tree.current_window_id().unwrap(), cursor_parent_id);
    debug_assert!(tree.node(cursor_parent_id).is_some());
    debug_assert!(matches!(
      tree.node(cursor_parent_id).unwrap(),
      TreeNodeDispatcher::Window(_)
    ));
    let cursor_node = tree.remove(cursor_id);
    debug_assert!(cursor_node.is_some());
    let cursor_node = cursor_node.unwrap();
    debug_assert!(matches!(cursor_node, TreeNodeDispatcher::Cursor(_)));
    debug_assert!(!tree.children_ids(cursor_parent_id).contains(&cursor_id));
    let cursor_node = match cursor_node {
      TreeNodeDispatcher::Cursor(mut cursor) => {
        cursor.set_style(&CursorStyle::SteadyBar);
        TreeNodeDispatcher::Cursor(cursor)
      }
      _ => unreachable!(),
    };

    // Insert to new parent
    debug_assert!(tree.command_line_id().is_some());
    let cmdline_id = tree.command_line_id().unwrap();
    debug_assert!(tree.node(cmdline_id).is_some());
    debug_assert!(matches!(
      tree.node(cmdline_id).unwrap(),
      TreeNodeDispatcher::CommandLine(_)
    ));
    let _inserted = tree.bounded_insert(cmdline_id, cursor_node);
    debug_assert!(_inserted.is_none());

    // Initialize command-line contents.
    let contents = data_access.contents.clone();
    let mut contents = lock!(contents);

    cursor_ops::cursor_insert(
      &mut tree,
      cmdline_id,
      contents.command_line_content_mut(),
      ":".to_compact_string(),
    );

    StatefulValueDispatcher::CommandLineExMode(super::CommandLineExStateful::default())
  }
}

impl NormalStateful {
  fn _goto_command_line_search_forward_mode(
    &self,
    _data_access: &StatefulDataAccess,
  ) -> StatefulValueDispatcher {
    // let tree = data_access.tree.clone();
    // let mut tree = lock!(tree);
    //
    // debug_assert!(tree.current_window_id().is_some());
    // let current_window_id = tree.current_window_id().unwrap();
    // debug_assert!(tree.node_mut(current_window_id).is_some());
    // let current_window_node = tree.node_mut(current_window_id).unwrap();
    // debug_assert!(matches!(current_window_node, TreeNode::Window(_)));
    // match current_window_node {
    //   TreeNode::Window(_current_window) => {}
    //   _ => unreachable!(),
    // }
    //
    // let cursor_id = tree.cursor_id().unwrap();
    // debug_assert!(tree.node_mut(cursor_id).is_some());
    // let cursor_node = tree.node_mut(cursor_id).unwrap();
    // debug_assert!(matches!(cursor_node, TreeNode::Cursor(_)));
    // match cursor_node {
    //   TreeNode::Cursor(cursor) => cursor.set_style(&CursorStyle::SteadyBar),
    //   _ => unreachable!(),
    // }

    StatefulValueDispatcher::CommandLineSearchForwardMode(
      super::CommandLineSearchForwardStateful::default(),
    )
  }
}

impl NormalStateful {
  fn _goto_command_line_search_backward_mode(
    &self,
    _data_access: &StatefulDataAccess,
  ) -> StatefulValueDispatcher {
    // let tree = data_access.tree.clone();
    // let mut tree = lock!(tree);
    //
    // debug_assert!(tree.current_window_id().is_some());
    // let current_window_id = tree.current_window_id().unwrap();
    // debug_assert!(tree.node_mut(current_window_id).is_some());
    // let current_window_node = tree.node_mut(current_window_id).unwrap();
    // debug_assert!(matches!(current_window_node, TreeNode::Window(_)));
    // match current_window_node {
    //   TreeNode::Window(_current_window) => {}
    //   _ => unreachable!(),
    // }
    //
    // let cursor_id = tree.cursor_id().unwrap();
    // debug_assert!(tree.node_mut(cursor_id).is_some());
    // let cursor_node = tree.node_mut(cursor_id).unwrap();
    // debug_assert!(matches!(cursor_node, TreeNode::Cursor(_)));
    // match cursor_node {
    //   TreeNode::Cursor(cursor) => cursor.set_style(&CursorStyle::SteadyBar),
    //   _ => unreachable!(),
    // }

    StatefulValueDispatcher::CommandLineSearchBackwardMode(
      super::CommandLineSearchBackwardStateful::default(),
    )
  }
}

impl NormalStateful {
  fn goto_insert_mode(&self, data_access: &StatefulDataAccess) -> StatefulValueDispatcher {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);
    let cursor_id = tree.cursor_id().unwrap();
    debug_assert!(tree.node_mut(cursor_id).is_some());
    let cursor_node = tree.node_mut(cursor_id).unwrap();
    debug_assert!(matches!(cursor_node, TreeNodeDispatcher::Cursor(_)));
    match cursor_node {
      TreeNodeDispatcher::Cursor(cursor) => cursor.set_style(&CursorStyle::SteadyBar),
      _ => unreachable!(),
    }

    StatefulValueDispatcher::InsertMode(super::InsertStateful::default())
  }
}

impl NormalStateful {
  /// Cursor move in current window, with buffer scroll.
  fn cursor_move(
    &self,
    data_access: &StatefulDataAccess,
    op: Operation,
  ) -> StatefulValueDispatcher {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);
    let current_window = self._current_window(&mut tree);
    let current_window_id = current_window.id();
    let buffer = current_window.buffer().upgrade().unwrap();
    let buffer = lock!(buffer);

    cursor_ops::cursor_move(&mut tree, current_window_id, buffer.text(), op, false);

    StatefulValueDispatcher::NormalMode(NormalStateful::default())
  }
}

#[cfg(test)]
use crate::buf::text::Text;
#[cfg(test)]
use crate::ui::viewport::{CursorViewport, ViewportSearchDirection, Viewportable};

impl NormalStateful {
  #[cfg(test)]
  // Returns `(target_cursor_char, target_cursor_line, viewport_search_direction)`.
  fn __target_cursor_exclude_empty_eol(
    &self,
    cursor_viewport: &CursorViewport,
    text: &Text,
    op: Operation,
  ) -> (usize, usize, ViewportSearchDirection) {
    use crate::state::ops::cursor_ops::CursorMoveDirection;

    let (target_cursor_char, target_cursor_line, move_direction) =
      cursor_ops::normalize_to_cursor_move_to_exclude_empty_eol(
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
  fn __test_raw_cursor_move(&self, data_access: &StatefulDataAccess, op: Operation) {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);
    let current_window = self._current_window(&mut tree);
    let buffer = current_window.buffer().upgrade().unwrap();
    let buffer = lock!(buffer);
    let viewport = current_window.viewport();
    let cursor_viewport = current_window.cursor_viewport();

    let (target_cursor_char, target_cursor_line, _search_direction) =
      self.__target_cursor_exclude_empty_eol(&cursor_viewport, buffer.text(), op);

    let maybe_new_cursor_viewport = cursor_ops::raw_cursor_move_to(
      &viewport,
      &cursor_viewport,
      buffer.text(),
      Operation::CursorMoveTo((target_cursor_char, target_cursor_line)),
    );

    if let Some(new_cursor_viewport) = maybe_new_cursor_viewport {
      current_window.set_cursor_viewport(new_cursor_viewport.clone());
      let cursor_id = tree.cursor_id().unwrap();
      tree.bounded_move_to(
        cursor_id,
        new_cursor_viewport.column_idx() as isize,
        new_cursor_viewport.row_idx() as isize,
      );
    }
  }

  #[cfg(test)]
  fn __test_raw_window_scroll(&self, data_access: &StatefulDataAccess, op: Operation) {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);
    let current_window = self._current_window(&mut tree);
    let buffer = current_window.buffer().upgrade().unwrap();
    let buffer = lock!(buffer);
    let viewport = current_window.viewport();

    let (start_column, start_line) = cursor_ops::normalize_to_window_scroll_to(
      op,
      viewport.start_column_idx(),
      viewport.start_line_idx(),
    );
    let maybe_new_viewport_arc = cursor_ops::raw_widget_scroll_to(
      &viewport,
      current_window.actual_shape(),
      current_window.options(),
      buffer.text(),
      Operation::WindowScrollTo((start_column, start_line)),
    );
    if let Some(new_viewport_arc) = maybe_new_viewport_arc.clone() {
      current_window.set_viewport(new_viewport_arc.clone());
    }
  }

  fn editor_quit(&self, _data_access: &StatefulDataAccess) -> StatefulValueDispatcher {
    StatefulValueDispatcher::QuitState(QuitStateful::default())
  }
}

// spellchecker:off
#[cfg(test)]
#[allow(unused_imports)]
mod tests_util {
  use super::*;

  use crate::buf::opt::BufferLocalOptionsBuilder;
  use crate::buf::{BufferArc, BuffersManagerArc};
  use crate::content::{TextContents, TextContentsArc};
  use crate::lock;
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
    let state = State::to_arc(State::default());
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
    let state = State::to_arc(State::default());
    (tree, state, bufs, buf, contents)
  }

  pub fn get_viewport(tree: TreeArc) -> ViewportArc {
    let tree = lock!(tree);
    let current_window_id = tree.current_window_id().unwrap();
    let current_window_node = tree.node(current_window_id).unwrap();
    assert!(matches!(current_window_node, TreeNodeDispatcher::Window(_)));
    match current_window_node {
      TreeNodeDispatcher::Window(current_window) => current_window.viewport(),
      _ => unreachable!(),
    }
  }

  pub fn get_cursor_viewport(tree: TreeArc) -> CursorViewportArc {
    let tree = lock!(tree);
    let cursor_id = tree.cursor_id().unwrap();
    let cursor_parent_id = tree.parent_id(cursor_id).unwrap();
    let cursor_parent_node = tree.node(cursor_parent_id).unwrap();
    assert!(matches!(
      cursor_parent_node,
      TreeNodeDispatcher::Window(_) | TreeNodeDispatcher::CommandLine(_)
    ));
    let vnode: &dyn Viewportable = match cursor_parent_node {
      TreeNodeDispatcher::Window(window) => window,
      TreeNodeDispatcher::CommandLine(cmdline) => cmdline,
      _ => unreachable!(),
    };
    vnode.cursor_viewport()
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
  use crate::state::{State, StateArc};
  use crate::test::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::test::log::init as test_log_init;
  use crate::test::tree::make_tree_with_buffers;
  use crate::ui::tree::TreeArc;
  use crate::ui::viewport::{
    CursorViewport, CursorViewportArc, Viewport, ViewportArc, ViewportSearchDirection,
  };
  use crate::ui::widget::window::{WindowLocalOptions, WindowLocalOptionsBuilder};
  use crate::{lock, state};

  use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
  use std::collections::BTreeMap;
  use tracing::info;

  #[test]
  fn get1() {
    test_log_init();

    let stateful = NormalStateful::default();
    assert!(matches!(
      stateful._get_operation(Event::Key(KeyEvent::new(
        KeyCode::Char('i'),
        KeyModifiers::empty()
      ))),
      Some(Operation::GotoInsertMode)
    ));
    assert!(matches!(
      stateful._get_operation(Event::Key(KeyEvent::new(
        KeyCode::Char('j'),
        KeyModifiers::empty()
      ))),
      Some(Operation::CursorMoveDownBy(_))
    ));
    assert!(matches!(
      stateful._get_operation(Event::Key(KeyEvent::new(
        KeyCode::Esc,
        KeyModifiers::empty()
      ))),
      Some(Operation::EditorQuit)
    ));
  }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests_raw_cursor_move_y_by {
  use super::tests_util::*;
  use super::*;

  use crate::buf::opt::BufferLocalOptionsBuilder;
  use crate::buf::{BufferArc, BuffersManagerArc};
  use crate::content::TextContents;
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

    let data_access = StatefulDataAccess::new(state, tree, bufs, contents, Event::Key(key_event));
    let stateful_machine = NormalStateful::default();
    stateful_machine.__test_raw_cursor_move(&data_access, Operation::CursorMoveUpBy(1));

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

    let data_access = StatefulDataAccess::new(state, tree, bufs, contents, Event::Key(key_event));
    let stateful_machine = NormalStateful::default();
    stateful_machine.__test_raw_cursor_move(&data_access, Operation::CursorMoveUpBy(1));

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

    let data_access = StatefulDataAccess::new(state, tree, bufs, contents, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful.__test_raw_cursor_move(&data_access, Operation::CursorMoveDownBy(3));

    let tree = data_access.tree.clone();
    let actual1 = get_cursor_viewport(tree);
    assert_eq!(actual1.line_idx(), 3);
    assert_eq!(actual1.char_idx(), 0);

    stateful.__test_raw_cursor_move(&data_access, Operation::CursorMoveUpBy(1));

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

    let data_access = StatefulDataAccess::new(state, tree, bufs, contents, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful.__test_raw_cursor_move(&data_access, Operation::CursorMoveDownBy(2));

    let tree = data_access.tree.clone();
    let actual1 = get_cursor_viewport(tree);
    assert_eq!(actual1.line_idx(), 2);
    assert_eq!(actual1.char_idx(), 0);

    stateful.__test_raw_cursor_move(&data_access, Operation::CursorMoveUpBy(1));

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
    let state = State::to_arc(State::default());
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree, bufs, contents, Event::Key(key_event));
    let stateful_machine = NormalStateful::default();
    stateful_machine.__test_raw_cursor_move(&data_access, Operation::CursorMoveDownBy(1));

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

    let data_access = StatefulDataAccess::new(state, tree, bufs, contents, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful.__test_raw_cursor_move(&data_access, Operation::CursorMoveDownBy(2));

    let tree = data_access.tree.clone();
    let actual1 = get_cursor_viewport(tree);
    assert_eq!(actual1.line_idx(), 2);
    assert_eq!(actual1.char_idx(), 0);

    stateful.__test_raw_cursor_move(&data_access, Operation::CursorMoveUpBy(1));

    let tree = data_access.tree.clone();
    let actual2 = get_cursor_viewport(tree);
    assert_eq!(actual2.line_idx(), 1);
    assert_eq!(actual2.char_idx(), 0);
  }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests_raw_cursor_move_x_by {
  use super::tests_util::*;
  use super::*;

  use crate::buf::opt::BufferLocalOptionsBuilder;
  use crate::buf::{BufferArc, BuffersManagerArc};
  use crate::content::TextContents;
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
    let state = State::to_arc(State::default());
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree, bufs, contents, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful.__test_raw_cursor_move(&data_access, Operation::CursorMoveRightBy(1));

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
    let state = State::to_arc(State::default());
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree, bufs, contents, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful.__test_raw_cursor_move(&data_access, Operation::CursorMoveRightBy(1));

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
    let state = State::to_arc(State::default());
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree, bufs, contents, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful.__test_raw_cursor_move(&data_access, Operation::CursorMoveRightBy(20));

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
    let state = State::to_arc(State::default());
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree, bufs, contents, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful.__test_raw_cursor_move(&data_access, Operation::CursorMoveRightBy(5));

    let tree = data_access.tree.clone();
    let actual1 = get_cursor_viewport(tree);
    assert_eq!(actual1.line_idx(), 0);
    assert_eq!(actual1.char_idx(), 5);

    stateful.__test_raw_cursor_move(&data_access, Operation::CursorMoveLeftBy(3));

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
    let state = State::to_arc(State::default());
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('j'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );

    let prev_cursor_viewport = get_cursor_viewport(tree.clone());
    assert_eq!(prev_cursor_viewport.line_idx(), 0);
    assert_eq!(prev_cursor_viewport.char_idx(), 0);

    let data_access = StatefulDataAccess::new(state, tree, bufs, contents, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful.__test_raw_cursor_move(&data_access, Operation::CursorMoveRightBy(5));

    let tree = data_access.tree.clone();
    let actual = get_cursor_viewport(tree);
    assert_eq!(actual.line_idx(), 0);
    assert_eq!(actual.char_idx(), 5);

    for i in (0..=4).rev() {
      stateful.__test_raw_cursor_move(&data_access, Operation::CursorMoveLeftBy(1));

      let tree = data_access.tree.clone();
      let actual = get_cursor_viewport(tree);
      assert_eq!(actual.line_idx(), 0);
      assert_eq!(actual.char_idx(), i);
    }
  }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests_raw_cursor_move_by {
  use super::tests_util::*;
  use super::*;

  use crate::buf::opt::BufferLocalOptionsBuilder;
  use crate::buf::{BufferArc, BuffersManagerArc};
  use crate::content::TextContents;
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
    let data_access = StatefulDataAccess::new(state, tree, bufs, contents, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful.__test_raw_cursor_move(&data_access, Operation::CursorMoveBy((5, 0)));

    let tree = data_access.tree.clone();
    let actual1 = get_cursor_viewport(tree);
    assert_eq!(actual1.line_idx(), 0);
    assert_eq!(actual1.char_idx(), 5);

    // Step-2
    stateful.__test_raw_cursor_move(&data_access, Operation::CursorMoveBy((0, 1)));
    let tree = data_access.tree.clone();
    let actual2 = get_cursor_viewport(tree);
    assert_eq!(actual2.line_idx(), 1);
    assert_eq!(actual2.char_idx(), 5);

    // Step-3
    stateful.__test_raw_cursor_move(&data_access, Operation::CursorMoveBy((-3, 0)));
    let tree = data_access.tree.clone();
    let actual3 = get_cursor_viewport(tree);
    assert_eq!(actual3.line_idx(), 1);
    assert_eq!(actual3.char_idx(), 2);

    // Step-4
    stateful.__test_raw_cursor_move(&data_access, Operation::CursorMoveBy((0, -1)));
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
        stateful.__test_raw_cursor_move(&data_access, c.clone());
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
        stateful.__test_raw_cursor_move(&data_access, c.clone());
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
      contents.clone(),
      Event::Key(key_event),
    );
    let command = Operation::CursorMoveBy((lines[0].len() as isize, 0));
    stateful.__test_raw_cursor_move(&data_access, command);

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
    stateful.__test_raw_cursor_move(&data_access, command);

    let tree = data_access.tree.clone();
    let actual2 = get_cursor_viewport(tree);
    assert_eq!(actual2.line_idx(), 1);
    assert_eq!(actual2.char_idx(), 18);
  }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests_raw_cursor_move_to {
  use super::tests_util::*;
  use super::*;

  use crate::buf::opt::BufferLocalOptionsBuilder;
  use crate::buf::{BufferArc, BuffersManagerArc};
  use crate::content::TextContents;
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
    let data_access = StatefulDataAccess::new(state, tree, bufs, contents, Event::Key(key_event));
    stateful.__test_raw_cursor_move(&data_access, Operation::CursorMoveTo((5, 0)));

    let tree = data_access.tree.clone();
    let actual1 = get_cursor_viewport(tree);
    assert_eq!(actual1.line_idx(), 0);
    assert_eq!(actual1.char_idx(), 5);

    // Step-2
    stateful.__test_raw_cursor_move(&data_access, Operation::CursorMoveTo((5, 1)));
    let tree = data_access.tree.clone();
    let actual2 = get_cursor_viewport(tree);
    assert_eq!(actual2.line_idx(), 1);
    assert_eq!(actual2.char_idx(), 5);

    // Step-3
    stateful.__test_raw_cursor_move(&data_access, Operation::CursorMoveTo((2, 1)));
    let tree = data_access.tree.clone();
    let actual3 = get_cursor_viewport(tree);
    assert_eq!(actual3.line_idx(), 1);
    assert_eq!(actual3.char_idx(), 2);

    // Step-4
    stateful.__test_raw_cursor_move(&data_access, Operation::CursorMoveTo((2, 0)));
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
        stateful.__test_raw_cursor_move(&data_access, c.clone());
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
        stateful.__test_raw_cursor_move(&data_access, c.clone());
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
    stateful.__test_raw_cursor_move(&data_access, command);

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
    stateful.__test_raw_cursor_move(&data_access, command);

    let tree = data_access.tree.clone();
    let actual2 = get_cursor_viewport(tree);
    assert_eq!(actual2.line_idx(), 1);
    assert_eq!(actual2.char_idx(), 18);
  }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests_raw_window_scroll_y_by {
  use super::tests_util::*;
  use super::*;

  use crate::buf::opt::BufferLocalOptionsBuilder;
  use crate::buf::{BufferArc, BuffersManagerArc};
  use crate::lock;
  use crate::prelude::*;
  use crate::state::{State, StateArc};
  use crate::test::buf::{make_buffer_from_lines, make_buffers_manager};
  use crate::test::log::init as test_log_init;
  use crate::test::tree::make_tree_with_buffers;
  use crate::ui::tree::TreeArc;
  use crate::ui::viewport::{
    CursorViewport, CursorViewportArc, Viewport, ViewportArc, ViewportSearchDirection,
  };
  use crate::ui::widget::window::{WindowLocalOptions, WindowLocalOptionsBuilder, content};

  use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
  use std::collections::BTreeMap;
  use tracing::info;

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

    let data_access = StatefulDataAccess::new(state, tree, bufs, contents, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollUpBy(1));

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

    let data_access = StatefulDataAccess::new(state, tree, bufs, contents, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollDownBy(1));

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

    let data_access = StatefulDataAccess::new(state, tree, bufs, contents, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollDownBy(1));

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

    let data_access = StatefulDataAccess::new(state, tree, bufs, contents, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollDownBy(4));

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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollDownBy(4));

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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollDownBy(4));

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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollUpBy(1));

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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollUpBy(4));

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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollUpBy(1));

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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollUpBy(3));

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

    let data_access = StatefulDataAccess::new(state, tree, bufs, contents, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollDownBy(4));

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

    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollDownBy(8));

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

    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollDownBy(1));

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

    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollDownBy(3));

    let tree = data_access.tree.clone();

    // Scroll-3
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec!["", "", ""];
      let expect_fills: BTreeMap<usize, usize> = vec![(10, 0)].into_iter().collect();
      assert_viewport_scroll(
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
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollUpBy(2));

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
  }
}
#[cfg(test)]
#[allow(unused_imports)]
mod tests_raw_window_scroll_x_by {
  use super::tests_util::*;
  use super::*;

  use crate::buf::opt::BufferLocalOptionsBuilder;
  use crate::buf::{BufferArc, BuffersManagerArc};
  use crate::lock;
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

    let data_access = StatefulDataAccess::new(state, tree, bufs, contents, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollRightBy(1));

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

    let data_access = StatefulDataAccess::new(state, tree, bufs, contents, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollRightBy(1));

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

    let data_access = StatefulDataAccess::new(state, tree, bufs, contents, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollLeftBy(1));

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

    let data_access = StatefulDataAccess::new(state, tree, bufs, contents, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollRightBy(12));

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

    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollRightBy(12));

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

    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollRightBy(10));

    let tree = data_access.tree.clone();

    // Scroll-2
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec!["", " and small", "several th", "small enou", "too long t"];
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

    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollRightBy(160));

    let tree = data_access.tree.clone();

    // Scroll-3
    {
      let viewport = get_viewport(tree.clone());
      let expect = vec!["", "", "", "\n", ""];
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

    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollLeftBy(156));

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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollRightBy(4));

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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollRightBy(4));

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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollLeftBy(1));

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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollLeftBy(4));

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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollLeftBy(1));

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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollLeftBy(3));

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

    let data_access = StatefulDataAccess::new(state, tree, bufs, contents, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollRightBy(4));

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

    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollRightBy(8));

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

    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollRightBy(1));

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

    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollRightBy(3));

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

    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollLeftBy(1));

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
mod tests_raw_window_scroll_to {
  use super::tests_util::*;
  use super::*;

  use crate::buf::opt::BufferLocalOptionsBuilder;
  use crate::buf::{BufferArc, BuffersManagerArc};
  use crate::lock;
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

    let data_access = StatefulDataAccess::new(state, tree, bufs, contents, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollTo((0, 1)));

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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollTo((0, 4)));

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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollTo((0, 8)));

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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollTo((0, 7)));

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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollTo((0, 3)));

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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollTo((0, 2)));

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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollTo((0, 0)));

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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollTo((4, 0)));

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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollTo((8, 0)));

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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollTo((7, 0)));

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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollTo((3, 0)));

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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollTo((2, 0)));

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
    let data_access = StatefulDataAccess::new(
      state.clone(),
      tree,
      bufs.clone(),
      contents.clone(),
      Event::Key(key_event),
    );
    let stateful = NormalStateful::default();
    stateful.__test_raw_window_scroll(&data_access, Operation::WindowScrollTo((0, 0)));

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
mod tests_cursor_move {
  use super::tests_util::*;
  use super::*;

  use crate::buf::opt::BufferLocalOptionsBuilder;
  use crate::buf::{BufferArc, BuffersManagerArc};
  use crate::lock;
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

    let data_access = StatefulDataAccess::new(state, tree, bufs, contents, Event::Key(key_event));
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

    let data_access = StatefulDataAccess::new(state, tree, bufs, contents, Event::Key(key_event));
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

    let data_access =
      StatefulDataAccess::new(state, tree.clone(), bufs, contents, Event::Key(key_event));
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

    stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(5));

    // Move-2
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 6);
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

    let data_access =
      StatefulDataAccess::new(state, tree.clone(), bufs, contents, Event::Key(key_event));
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

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(11));

    // Move-6
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 3);
      assert_eq!(actual2.char_idx(), 31);

      let viewport = get_viewport(tree.clone());
      let expect = vec!["", " and small", "several th", "small enou", "too long t"];
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

    stateful.cursor_move(&data_access, Operation::CursorMoveRightBy(120));

    // Move-7
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 3);
      assert_eq!(actual2.char_idx(), 151);

      let viewport = get_viewport(tree.clone());
      let expect = vec!["", "", "", "t the rend", ""];
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

    stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(2));

    // Move-8
    {
      let tree = data_access.tree.clone();
      let actual2 = get_cursor_viewport(tree.clone());
      assert_eq!(actual2.line_idx(), 5);
      assert_eq!(actual2.char_idx(), 93);

      let viewport = get_viewport(tree.clone());
      let expect = vec!["", "", "t, then th", "ere're mul", ".\n"];
      let expect_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0), (3, 0), (4, 0), (5, 0)]
        .into_iter()
        .collect();
      assert_viewport_scroll(
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

    let data_access =
      StatefulDataAccess::new(state, tree.clone(), bufs, contents, Event::Key(key_event));
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

    stateful.cursor_move(&data_access, Operation::CursorMoveDownBy(3));

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
      let expect_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport_scroll(
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

    let data_access =
      StatefulDataAccess::new(state, tree.clone(), bufs, contents, Event::Key(key_event));
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
      StatefulDataAccess::new(state, tree.clone(), bufs, contents, Event::Key(key_event));
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
      let expect_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 0)].into_iter().collect();
      assert_viewport_scroll(
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
      let expect_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 0)].into_iter().collect();
      assert_viewport_scroll(
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
      let expect_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 0)].into_iter().collect();
      assert_viewport_scroll(
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
      let expect_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0)].into_iter().collect();
      assert_viewport_scroll(
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
      StatefulDataAccess::new(state, tree.clone(), bufs, contents, Event::Key(key_event));
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
      let expect_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 0)].into_iter().collect();
      assert_viewport_scroll(
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

    let data_access =
      StatefulDataAccess::new(state, tree.clone(), bufs, contents, Event::Key(key_event));
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
      let expect_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport_scroll(
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

    let data_access =
      StatefulDataAccess::new(state, tree.clone(), bufs, contents, Event::Key(key_event));
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
      StatefulDataAccess::new(state, tree.clone(), bufs, contents, Event::Key(key_event));
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
      let expect_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0)].into_iter().collect();
      assert_viewport_scroll(
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
      StatefulDataAccess::new(state, tree.clone(), bufs, contents, Event::Key(key_event));
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
      let expect_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 0)].into_iter().collect();
      assert_viewport_scroll(
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
mod tests_goto_command_line_ex_mode {
  use super::tests_util::*;
  use super::*;

  use crate::buf::opt::BufferLocalOptionsBuilder;
  use crate::buf::{BufferArc, BuffersManagerArc};
  use crate::lock;
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

    let data_access = StatefulDataAccess::new(state, tree, bufs, contents, Event::Key(key_event));
    let stateful = NormalStateful::default();
    stateful.goto_command_line_ex_mode(&data_access);

    let tree = data_access.tree.clone();
    let actual_cursor = get_cursor_viewport(tree.clone());
    assert_eq!(actual_cursor.line_idx(), 0);
    assert_eq!(actual_cursor.char_idx(), 1);
    assert_eq!(actual_cursor.row_idx(), 0);
    assert_eq!(actual_cursor.column_idx(), 1);

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
// spellchecker:on
