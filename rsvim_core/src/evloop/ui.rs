//! UI utility.

use crate::buf::BufferWk;
use crate::content::TextContentsWk;
use crate::ui::canvas::CursorStyle;
use crate::ui::tree::Tree;
use crate::ui::widget::cmdline::indicator::CmdlineIndicatorSymbol;
use taffy::Style;

pub fn init_default_window_without_cmdline(
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
      width: taffy::prelude::percent(1.0),
      height: taffy::prelude::percent(1.0),
    },
    ..Default::default()
  };
  let cmdline_style = Style {
    min_size: taffy::Size {
      width: taffy::prelude::percent(1.0),
      height: taffy::prelude::percent(1.0),
    },
    ..Default::default()
  };

  let window_opts = *tree.global_local_options();

  // Initialize default window.
  let window_id = tree
    .new_window_with_parent(tree_root_id, window_style, window_opts, buf)
    .unwrap();

  // Initialize cursor inside the default window.
  let window_content_id = tree.window(window_id).content_id();
  let _cursor_id = tree
    .new_cursor_with_parent(
      window_content_id,
      cursor_blinking,
      cursor_hidden,
      cursor_style,
    )
    .unwrap();

  // Initialize command-line.
  let _cmdline_id = tree
    .new_cmdline_with_parent(
      tree_root_id,
      cmdline_style,
      CmdlineIndicatorSymbol::Empty,
      text_contents,
    )
    .unwrap();
}

pub fn init_default_window(
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
      width: taffy::prelude::percent(1.0),
      height: taffy::prelude::percent(1.0),
    },
    ..Default::default()
  };
  let cmdline_style = Style {
    min_size: taffy::Size {
      width: taffy::prelude::percent(1.0),
      height: taffy::prelude::percent(1.0),
    },
    ..Default::default()
  };

  let window_opts = *tree.global_local_options();

  // Initialize default window.
  let window_id = tree
    .new_window_with_parent(tree_root_id, window_style, window_opts, buf)
    .unwrap();

  // Initialize cursor inside the default window.
  let window_content_id = tree.window(window_id).content_id();
  let _cursor_id = tree
    .new_cursor_with_parent(
      window_content_id,
      cursor_blinking,
      cursor_hidden,
      cursor_style,
    )
    .unwrap();

  // Initialize command-line.
  let _cmdline_id = tree
    .new_cmdline_with_parent(
      tree_root_id,
      cmdline_style,
      CmdlineIndicatorSymbol::Empty,
      text_contents,
    )
    .unwrap();
}
