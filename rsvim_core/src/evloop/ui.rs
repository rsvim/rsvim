//! UI utility.

use crate::buf::BufferArc;
use crate::content::TextContentsArc;
use crate::prelude::*;
use crate::ui::canvas::CursorStyle;
use crate::ui::tree::Inodeable;
use crate::ui::tree::Tree;
use crate::ui::tree::TreeNode;
use crate::ui::widget::command_line::CommandLine;
use crate::ui::widget::cursor::Cursor;
use crate::ui::widget::window::Window;
use std::sync::Arc;

pub fn init_window(
  canvas_size: &U16Size,
  tree: &mut Tree,
  buf: BufferArc,
  text_contents: TextContentsArc,
  cursor_blinking: bool,
  cursor_hidden: bool,
  cursor_style: CursorStyle,
) {
  let tree_root_id = tree.root_id();

  // Initialize default window.
  let window_shape = rect!(
    0,
    0,
    canvas_size.width(),
    canvas_size.height().saturating_sub(1)
  );
  let window_shape = rect_as!(window_shape, isize);
  let mut window = {
    trace!("Bind first buffer to default window {:?}", lock!(buf).id());
    Window::new(
      tree.global_local_options(),
      window_shape,
      Arc::downgrade(&buf),
    )
  };
  let window_id = window.id();

  // Initialize cursor inside the default window.
  let cursor_shape = rect!(0, 0, 1, 1);
  let cursor =
    Cursor::new(cursor_shape, cursor_blinking, cursor_hidden, cursor_style);
  let _previous_inserted_cursor = window.insert_cursor(cursor);
  debug_assert!(_previous_inserted_cursor.is_none());

  tree.bounded_insert(tree_root_id, TreeNode::Window(window));
  tree.set_current_window_id(Some(window_id));

  // Initialize default command-line.
  let cmdline_shape = rect!(
    0,
    canvas_size.height().saturating_sub(1) as isize,
    canvas_size.width() as isize,
    canvas_size.height() as isize
  );
  let cmdline = CommandLine::new(cmdline_shape, Arc::downgrade(&text_contents));

  tree.bounded_insert(tree_root_id, TreeNode::CommandLine(cmdline));
}
