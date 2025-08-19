//! Tree utils for testing.

#![allow(unused_imports)]

use crate::buf::BuffersManagerArc;
use crate::content::TextContentsArc;
use crate::prelude::*;
use crate::ui::tree::*;
use crate::ui::widget::command_line::CommandLine;
use crate::ui::widget::command_line::indicator::IndicatorSymbol;
use crate::ui::widget::cursor::Cursor;
use crate::ui::widget::window::Window;
use crate::ui::widget::window::opt::WindowOptions;

use std::sync::Arc;

/// Create tree with 1 window and 1 buffer, the buffer is in buffers manager.
pub fn make_tree_with_buffers(
  canvas_size: U16Size,
  window_local_opts: WindowOptions,
  buffers_manager: BuffersManagerArc,
) -> TreeArc {
  // UI Tree
  let tree_arc = Tree::to_arc(Tree::new(canvas_size));
  let buffers = lock!(buffers_manager);

  let mut tree = lock!(tree_arc);
  tree.set_global_local_options(&window_local_opts);
  let tree_root_id = tree.root_id();

  // Window
  let window_shape = IRect::new(
    (0, 0),
    (canvas_size.width() as isize, canvas_size.height() as isize),
  );
  let mut window = {
    let (_, buf) = buffers.first_key_value().unwrap();
    Window::new(
      tree.global_local_options(),
      window_shape,
      Arc::downgrade(buf),
    )
  };
  let window_id = window.id();

  // Cursor.
  let cursor_shape = IRect::new((0, 0), (1, 1));
  let cursor = Cursor::default(cursor_shape);
  window.insert_cursor(cursor);

  tree.bounded_insert(tree_root_id, TreeNode::Window(window));
  tree.set_current_window_id(Some(window_id));

  tree_arc.clone()
}

/// Create tree with 1 window, 1 buffer, and 1 command-line, the buffer is in buffers manager, the
/// command-line is in the text contents.
pub fn make_tree_with_buffers_cmdline(
  canvas_size: U16Size,
  window_local_opts: WindowOptions,
  buffers_manager: BuffersManagerArc,
  text_contents: TextContentsArc,
) -> TreeArc {
  // UI Tree
  let tree_arc = Tree::to_arc(Tree::new(canvas_size));
  let buffers = lock!(buffers_manager);

  let mut tree = lock!(tree_arc);
  tree.set_global_local_options(&window_local_opts);
  let tree_root_id = tree.root_id();

  // window
  let window_shape = IRect::new(
    (0, 0),
    (
      canvas_size.width() as isize,
      canvas_size.height().saturating_sub(1) as isize,
    ),
  );
  let mut window = {
    let (_, buf) = buffers.first_key_value().unwrap();
    Window::new(
      tree.global_local_options(),
      window_shape,
      Arc::downgrade(buf),
    )
  };
  let window_id = window.id();

  // cursor
  let cursor_shape = IRect::new((0, 0), (1, 1));
  let cursor = Cursor::default(cursor_shape);
  window.insert_cursor(cursor);

  tree.bounded_insert(tree_root_id, TreeNode::Window(window));
  tree.set_current_window_id(Some(window_id));

  // command-line
  let cmdline_shape = IRect::new(
    (0, canvas_size.height().saturating_sub(1) as isize),
    (canvas_size.width() as isize, canvas_size.height() as isize),
  );
  let cmdline = CommandLine::new(cmdline_shape, Arc::downgrade(&text_contents));
  let _cmdline_id = cmdline.id();

  tree.bounded_insert(tree_root_id, TreeNode::CommandLine(cmdline));

  tree_arc.clone()
}
