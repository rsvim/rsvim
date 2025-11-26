//! Tree utils for testing.

use crate::buf::BuffersManagerArc;
use crate::content::TextContentsArc;
use crate::prelude::*;
use crate::ui::tree::*;
use crate::ui::widget::command_line::indicator::IndicatorSymbol;
use crate::ui::widget::window::opt::WindowOptions;
use std::sync::Arc;
use taffy::Style;
use taffy::prelude::FromLength;

/// Create tree with 1 window and 1 buffer, the buffer is in buffers manager.
pub fn make_tree_with_buffers(
  canvas_size: U16Size,
  window_local_opts: WindowOptions,
  buffers_manager: BuffersManagerArc,
) -> TreeArc {
  // UI Tree
  let tree_arc = Tree::to_arc(Tree::new(canvas_size).unwrap());
  let buffers = lock!(buffers_manager);

  let mut tree = lock!(tree_arc);
  tree.set_global_local_options(window_local_opts);
  let tree_root_id = tree.root_id();

  // Window
  let window_style = Style {
    size: taffy::Size {
      width: taffy::Dimension::auto(),
      height: taffy::Dimension::auto(),
    },
    ..Default::default()
  };

  let (_, buf) = buffers.first_key_value().unwrap();
  let window_opts = *tree.global_local_options();
  let window_id = tree
    .insert_new_window(
      tree_root_id,
      window_style,
      window_opts,
      Arc::downgrade(buf),
    )
    .unwrap();
  let window_content_id = tree.window(window_id).unwrap().content_id();
  let _cursor_id = tree.insert_new_default_cursor(window_content_id).unwrap();

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
  let tree_arc = Tree::to_arc(Tree::new(canvas_size).unwrap());
  let buffers = lock!(buffers_manager);

  let mut tree = lock!(tree_arc);
  tree.set_global_local_options(window_local_opts);
  let tree_root_id = tree.root_id();

  let window_style = Style {
    size: taffy::Size {
      width: taffy::Dimension::auto(),
      height: taffy::Dimension::auto(),
    },
    ..Default::default()
  };
  let cmdline_style = Style {
    size: taffy::Size {
      width: taffy::Dimension::auto(),
      height: taffy::Dimension::from_length(1_u16),
    },
    ..Default::default()
  };

  let (_, buf) = buffers.first_key_value().unwrap();
  let window_opts = *tree.global_local_options();
  let window_id = tree
    .insert_new_window(
      tree_root_id,
      window_style,
      window_opts,
      Arc::downgrade(buf),
    )
    .unwrap();
  let window_content_id = tree.window(window_id).unwrap().content_id();
  let _cursor_id = tree.insert_new_default_cursor(window_content_id).unwrap();
  tree.set_current_window_id(Some(window_id));

  let _cmdline_id = tree.insert_new_cmdline(
    tree_root_id,
    cmdline_style,
    IndicatorSymbol::Empty,
    Arc::downgrade(&text_contents),
  );

  tree_arc.clone()
}
