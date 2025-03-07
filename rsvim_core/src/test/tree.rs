//! Tree utils for testing.

#![allow(unused_imports)]

use crate::buf::{Buffer, BufferArc, BufferLocalOptions, BuffersManager, BuffersManagerArc};
use crate::cart::{IRect, U16Size};
use crate::ui::canvas::{Canvas, CanvasArc, Shader, ShaderCommand};
use crate::ui::tree::{Tree, TreeArc, TreeNode};
use crate::ui::widget::{Cursor, Window};
use crate::{rlock, wlock};

use ropey::{Rope, RopeBuilder, RopeSlice};
use std::collections::BTreeSet;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{self, info};

/// Create tree with 1 window and 1 buffer, the buffer is loaded from the filename.
pub fn make_tree_from_file(canvas_size: U16Size, filename: &str) -> TreeArc {
  // UI Tree
  let tree = Tree::to_arc(Tree::new(canvas_size));
  // Buffers
  let buffers_manager = BuffersManager::to_arc(BuffersManager::new());

  let mut buffers = wlock!(buffers_manager);
  let buf_id = buffers.new_file_buffer(Path::new(filename)).unwrap();

  let mut tree_mut = wlock!(tree);
  let tree_root_id = tree_mut.root_id();
  let window_shape = IRect::new(
    (0, 0),
    (canvas_size.width() as isize, canvas_size.height() as isize),
  );
  let window = {
    let (buf_id, buf) = buffers.first_key_value().unwrap();
    Window::new(window_shape, Arc::downgrade(buf), tree_mut.local_options())
  };
  let window_id = window.id();
  let window_node = TreeNode::Window(window);
  tree_mut.bounded_insert(&tree_root_id, window_node);

  // Initialize cursor.
  let cursor_shape = IRect::new((0, 0), (1, 1));
  let cursor = Cursor::new(cursor_shape);
  let cursor_node = TreeNode::Cursor(cursor);
  tree_mut.bounded_insert(&window_id, cursor_node);

  tree
}

/// Create tree with 1 window and 1 buffer, the buffer is filled with strings list.
pub fn make_tree_from_lines(canvas_size: U16Size, lines: Vec<&str>) -> TreeArc {
  // UI Tree
  let tree = Tree::to_arc(Tree::new(canvas_size));
  // Buffers
  let buffers_manager = BuffersManager::to_arc(BuffersManager::new());

  let mut buffers = wlock!(buffers_manager);
  let buf_id = buffers.new_file_buffer(Path::new(&filename)).unwrap();

  let mut tree_mut = wlock!(tree);
  let tree_root_id = tree_mut.root_id();
  let window_shape = IRect::new(
    (0, 0),
    (canvas_size.width() as isize, canvas_size.height() as isize),
  );
  let window = {
    let (buf_id, buf) = buffers.first_key_value().unwrap();
    Window::new(window_shape, Arc::downgrade(buf), tree_mut.local_options())
  };
  let window_id = window.id();
  let window_node = TreeNode::Window(window);
  tree_mut.bounded_insert(&tree_root_id, window_node);

  // Initialize cursor.
  let cursor_shape = IRect::new((0, 0), (1, 1));
  let cursor = Cursor::new(cursor_shape);
  let cursor_node = TreeNode::Cursor(cursor);
  tree_mut.bounded_insert(&window_id, cursor_node);

  tree
}
