//! Tree utils for testing.

use crate::buf::BuffersManagerArc;
use crate::content::TextContentsArc;
use crate::prelude::*;
use crate::ui::tree::*;
use crate::ui::widget::command_line::CommandLine;
use crate::ui::widget::cursor::Cursor;
use crate::ui::widget::window::Window;
use crate::ui::widget::window::opt::WindowOptions;
use std::sync::Arc;
use taffy::Style;
use taffy::prelude::FromLength;
use taffy::prelude::TaffyMaxContent;

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
  tree.set_global_local_options(&window_local_opts);
  let tree_root_id = tree.root_id();
  let tree_root_loid = tree.root_loid();

  // Window
  let window_style = Style {
    size: taffy::Size {
      width: taffy::Dimension::auto(),
      height: taffy::Dimension::auto(),
    },
    ..Default::default()
  };
  let cursor_style = Style {
    size: taffy::Size {
      width: taffy::Dimension::from_length(1_u16),
      height: taffy::Dimension::from_length(1_u16),
    },
    padding: taffy::Rect {
      left: taffy::LengthPercentage::from_length(0_u16),
      top: taffy::LengthPercentage::from_length(0_u16),
      right: taffy::LengthPercentage::calc(
        taffy::style::CompactLength::auto().calc_value(),
      ),
      bottom: taffy::LengthPercentage::calc(
        taffy::style::CompactLength::auto().calc_value(),
      ),
    },
    ..Default::default()
  };

  let (window_loid, window_shape, cursor_loid, cursor_shape) = {
    let lotree = tree.lotree();
    let mut lo = lotree.borrow_mut();
    let window_loid = lo.new_leaf(window_style).unwrap();
    let cursor_loid = lo.new_leaf(cursor_style).unwrap();
    lo.add_child(tree_root_loid, window_loid).unwrap();
    lo.add_child(window_loid, cursor_loid).unwrap();
    lo.compute_layout(tree_root_loid, taffy::Size::MAX_CONTENT)
      .unwrap();
    let window_layout = lo.layout(window_loid).unwrap();
    let cursor_layout = lo.layout(cursor_loid).unwrap();
    let window_shape = rect_from_layout!(window_layout, u16);
    let cursor_shape = rect!(0, 0, 1, 1);
    (window_loid, window_shape, cursor_loid, cursor_shape)
  };

  let mut window = {
    let (_, buf) = buffers.first_key_value().unwrap();
    Window::new(
      tree.lotree(),
      window_loid,
      window_shape,
      tree.global_local_options(),
      Arc::downgrade(buf),
    )
    .unwrap()
  };
  let window_id = window.id();

  // Cursor.
  let cursor = Cursor::default(cursor_loid, cursor_shape);
  window.insert_cursor(cursor);

  tree.insert(tree_root_id, TreeNode::Window(window));
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
  tree.set_global_local_options(&window_local_opts);
  let tree_root_id = tree.root_id();
  let tree_root_loid = tree.root_loid();

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
  let cursor_style = Style {
    size: taffy::Size {
      width: taffy::Dimension::from_length(1_u16),
      height: taffy::Dimension::from_length(1_u16),
    },
    padding: taffy::Rect {
      left: taffy::LengthPercentage::from_length(0_u16),
      top: taffy::LengthPercentage::from_length(0_u16),
      right: taffy::LengthPercentage::calc(
        taffy::style::CompactLength::auto().calc_value(),
      ),
      bottom: taffy::LengthPercentage::calc(
        taffy::style::CompactLength::auto().calc_value(),
      ),
    },
    ..Default::default()
  };

  let (
    window_loid,
    window_shape,
    cmdline_loid,
    cmdline_shape,
    cursor_loid,
    cursor_shape,
  ) = {
    let lotree = tree.lotree();
    let mut lo = lotree.borrow_mut();
    let window_loid = lo.new_leaf(window_style).unwrap();
    let cmdline_loid = lo.new_leaf(cmdline_style).unwrap();
    let cursor_loid = lo.new_leaf(cursor_style).unwrap();
    lo.add_child(tree_root_loid, window_loid).unwrap();
    lo.add_child(tree_root_loid, cmdline_loid).unwrap();
    lo.compute_layout(tree_root_loid, taffy::Size::MAX_CONTENT)
      .unwrap();
    let window_layout = lo.layout(window_loid).unwrap();
    let cmdline_layout = lo.layout(cmdline_loid).unwrap();
    let window_shape = rect_from_layout!(window_layout, u16);
    let cmdline_shape = rect_from_layout!(cmdline_layout, u16);
    let cursor_shape = rect!(0, 0, 1, 1); // dummy shape
    (
      window_loid,
      window_shape,
      cmdline_loid,
      cmdline_shape,
      cursor_loid,
      cursor_shape,
    )
  };

  // window
  let mut window = {
    let (_, buf) = buffers.first_key_value().unwrap();
    Window::new(
      tree.lotree(),
      window_loid,
      window_shape,
      tree.global_local_options(),
      Arc::downgrade(buf),
    )
    .unwrap()
  };
  let window_id = window.id();

  // cursor
  let cursor = Cursor::default(cursor_loid, cursor_shape);
  window.insert_cursor(cursor);

  tree.insert(tree_root_id, TreeNode::Window(window));
  tree.set_current_window_id(Some(window_id));

  // command-line
  let cmdline_shape = rect!(
    0,
    canvas_size.height().saturating_sub(1) as isize,
    canvas_size.width() as isize,
    canvas_size.height() as isize
  );
  let cmdline = CommandLine::new(cmdline_shape, Arc::downgrade(&text_contents));
  let _cmdline_id = cmdline.id();

  tree.bounded_insert(tree_root_id, TreeNode::CommandLine(cmdline));

  tree_arc.clone()
}
