//! UI utility.

use crate::buf::BufferWk;
use crate::content::TextContentsWk;
use crate::prelude::*;
use crate::ui::canvas::CursorStyle;
use crate::ui::tree::Inodeable;
use crate::ui::tree::Tree;
use crate::ui::tree::TreeNode;
use crate::ui::widget::cmdline::Cmdline;
use crate::ui::widget::cursor::Cursor;
use crate::ui::widget::window::Window;
use taffy::Style;

pub fn init_default_window(
  canvas_size: &U16Size,
  tree: &mut Tree,
  buf: BufferWk,
  text_contents: TextContentsWk,
  cursor_blinking: bool,
  cursor_hidden: bool,
  cursor_style: CursorStyle,
) {
  let tree_root_id = tree.root_id();

  let window_style = Style {
    size: taffy::Size {
      width: taffy::prelude::length(1.0),
      height: taffy::prelude::length(1.0),
    },
    ..Default::default()
  };
  let cmdline_style = Style {
    min_size: taffy::Size {
      width: taffy::prelude::length(1.0),
      height: taffy::prelude::length(1_u16),
    },
    ..Default::default()
  };

  // Initialize default window.
  let window_shape = rect!(
    0,
    0,
    canvas_size.width(),
    canvas_size.height().saturating_sub(1)
  );
  let window_shape = rect_as!(window_shape, isize);
  let mut window =
    { Window::new(tree.global_local_options(), window_shape, buf) };
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
  let cmdline = Cmdline::new(cmdline_shape, text_contents);

  tree.bounded_insert(tree_root_id, TreeNode::Cmdline(cmdline));
}
