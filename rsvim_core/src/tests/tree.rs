//! Tree utils for testing.

use crate::buf::BuffersManagerArc;
use crate::content::TextContentsArc;
use crate::evloop::ui as evloop_ui;
use crate::prelude::*;
use crate::ui::tree::*;
use crate::ui::widget::cursor::CURSOR_BLINKING;
use crate::ui::widget::cursor::CURSOR_HIDDEN;
use crate::ui::widget::cursor::CURSOR_STYLE;
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
  let window_shape = rect_from_size!(canvas_size);
  let window_shape = rect_as!(window_shape, isize);
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
  let cursor_shape = rect!(0, 0, 1, 1);
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
  let (_, buf) = buffers.first_key_value().unwrap();
  let buf = Arc::downgrade(buf);
  let text_contents = Arc::downgrade(&text_contents);

  let mut tree = lock!(tree_arc);
  tree.set_global_local_options(&window_local_opts);

  evloop_ui::init_default_window(
    &canvas_size,
    &mut tree,
    buf,
    text_contents,
    CURSOR_BLINKING,
    CURSOR_HIDDEN,
    CURSOR_STYLE,
  );

  tree_arc.clone()
}
