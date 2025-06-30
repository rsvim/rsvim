//! Tree utils for testing.

#![allow(unused_imports)]

use crate::buf::BuffersManagerArc;
use crate::content::TextContentsArc;
use crate::lock;
use crate::prelude::*;
use crate::ui::tree::*;
use crate::ui::widget::command_line::CommandLine;
use crate::ui::widget::cursor::Cursor;
use crate::ui::widget::window::{Window, WindowLocalOptions};

use std::sync::Arc;
use tracing::{self};

#[cfg(test)]
/// Create tree with 1 window and 1 buffer, the buffer is in buffers manager.
pub fn make_tree_with_buffers(
  canvas_size: U16Size,
  window_local_opts: WindowLocalOptions,
  buffers_manager: BuffersManagerArc,
) -> TreeArc {
  // UI Tree
  let tree = Tree::to_arc(Tree::new(canvas_size));
  let buffers = lock!(buffers_manager);

  let mut tree_mut = lock!(tree);
  tree_mut.set_global_local_options(&window_local_opts);
  let tree_root_id = tree_mut.root_id();
  let window_shape = IRect::new(
    (0, 0),
    (canvas_size.width() as isize, canvas_size.height() as isize),
  );
  let window = {
    let (_, buf) = buffers.first_key_value().unwrap();
    Window::new(
      tree_mut.global_local_options(),
      window_shape,
      Arc::downgrade(buf),
    )
  };
  let window_id = window.id();
  let window_node = TreeNode::Window(window);
  tree_mut.bounded_insert(tree_root_id, window_node);

  // Initialize cursor.
  let cursor_shape = IRect::new((0, 0), (1, 1));
  let cursor = Cursor::default(cursor_shape);
  let cursor_node = TreeNode::Cursor(cursor);
  tree_mut.bounded_insert(window_id, cursor_node);

  tree.clone()
}

#[cfg(test)]
/// Create tree with 1 window, 1 buffer, and 1 command-line, the buffer is in buffers manager, the
/// command-line is in the text contents.
pub fn make_tree_with_buffers_cmdline(
  canvas_size: U16Size,
  window_local_opts: WindowLocalOptions,
  buffers_manager: BuffersManagerArc,
  text_contents: TextContentsArc,
) -> TreeArc {
  // UI Tree
  let tree = Tree::to_arc(Tree::new(canvas_size));
  let buffers = lock!(buffers_manager);

  let mut tree_mut = lock!(tree);
  tree_mut.set_global_local_options(&window_local_opts);
  let tree_root_id = tree_mut.root_id();

  // window
  let window_shape = IRect::new(
    (0, 0),
    (
      canvas_size.width() as isize,
      canvas_size.height().saturating_sub(1) as isize,
    ),
  );
  let window = {
    let (_, buf) = buffers.first_key_value().unwrap();
    Window::new(
      tree_mut.global_local_options(),
      window_shape,
      Arc::downgrade(buf),
    )
  };
  let window_id = window.id();
  let window_node = TreeNode::Window(window);
  tree_mut.bounded_insert(tree_root_id, window_node);

  // command-line
  let cmdline_shape = IRect::new(
    (0, canvas_size.height().saturating_sub(1) as isize),
    (canvas_size.width() as isize, canvas_size.height() as isize),
  );
  let cmdline = CommandLine::new(cmdline_shape, Arc::downgrade(&text_contents));
  let _cmdline_id = cmdline.id();
  let cmdline = TreeNode::CommandLine(cmdline);
  tree_mut.bounded_insert(tree_root_id, cmdline);

  // cursor
  let cursor_shape = IRect::new((0, 0), (1, 1));
  let cursor = Cursor::default(cursor_shape);
  let cursor_node = TreeNode::Cursor(cursor);
  tree_mut.bounded_insert(window_id, cursor_node);

  tree.clone()
}
