//! Tree utils for testing.

use crate::buf::BuffersManagerArc;
use crate::content::TextContentsArc;
use crate::evloop::ui as evloop_ui;
use crate::prelude::*;
use crate::ui::canvas::CursorStyle;
use crate::ui::tree::*;
use crate::ui::widget::cursor::CURSOR_BLINKING;
use crate::ui::widget::cursor::CURSOR_HIDDEN;
use crate::ui::widget::cursor::CURSOR_STYLE;
use crate::ui::widget::window::opt::WindowOptions;
use std::sync::Arc;
use taffy::Style;

/// Create tree with 1 window and 1 buffer, the buffer is in buffers manager.
pub fn make_tree_with_buffers(
  canvas_size: U16Size,
  window_local_opts: WindowOptions,
  buffers_manager: BuffersManagerArc,
) -> TreeArc {
  let tree_style = Style {
    size: taffy::Size {
      width: taffy::prelude::length(canvas_size.width()),
      height: taffy::prelude::length(canvas_size.height()),
    },
    ..Default::default()
  };
  let window_style = Style {
    size: taffy::Size {
      width: taffy::prelude::percent(1.0),
      height: taffy::prelude::percent(1.0),
    },
    ..Default::default()
  };

  let tree_arc = Tree::to_arc(Tree::new(tree_style).unwrap());
  let buffers = lock!(buffers_manager);

  let mut tree = lock!(tree_arc);
  tree.set_global_local_options(window_local_opts);
  let tree_root_id = tree.root_id();

  // Window
  let (_, buf) = buffers.first_key_value().unwrap();
  let window_id = tree
    .new_window_with_parent(
      tree_root_id,
      window_style,
      window_local_opts,
      Arc::downgrade(&buf),
    )
    .unwrap();
  let window_content_id = tree.window(window_id).content_id();

  // Cursor.
  let cursor_id = tree
    .new_cursor_with_parent(
      window_content_id,
      false,
      false,
      CursorStyle::SteadyBlock,
    )
    .unwrap();

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
    &mut tree,
    buf,
    text_contents,
    CURSOR_BLINKING,
    CURSOR_HIDDEN,
    CURSOR_STYLE,
  );

  tree_arc.clone()
}
