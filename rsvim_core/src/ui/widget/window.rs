//! Window.

use crate::buf::BufferWk;
use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::tree::*;
use crate::ui::viewport::{CursorViewport, CursorViewportArc, Viewport, ViewportArc, Viewportable};
use crate::ui::widget::Widgetable;
use crate::ui::widget::cursor::Cursor;
use crate::ui::widget::window::content::WindowContent;
use crate::ui::widget::window::root::WindowRootContainer;
use crate::{inode_enum_dispatcher, inode_itree_impl, widget_enum_dispatcher};

// Re-export
pub use opt::*;

use std::sync::Arc;
// use tracing::trace;

pub mod content;
pub mod opt;
pub mod root;

#[derive(Debug, Clone)]
/// The value holder for each window widget.
pub enum WindowNode {
  WindowRootContainer(WindowRootContainer),
  WindowContent(WindowContent),
  Cursor(Cursor),
}

inode_enum_dispatcher!(WindowNode, WindowRootContainer, WindowContent, Cursor);
widget_enum_dispatcher!(WindowNode, WindowRootContainer, WindowContent, Cursor);

#[derive(Debug, Clone)]
/// The Vim window, it manages all descendant widget nodes, i.e. all widgets in the
/// [`crate::ui::widget::window`] module.
pub struct Window {
  base: Itree<WindowNode>,
  options: WindowLocalOptions,

  content_id: TreeNodeId,
  cursor_id: Option<TreeNodeId>,

  buffer: BufferWk,
  viewport: ViewportArc,
  cursor_viewport: CursorViewportArc,
}

impl Window {
  pub fn new(opts: &WindowLocalOptions, shape: IRect, buffer: BufferWk) -> Self {
    let window_root = WindowRootContainer::new(shape);
    let window_root_id = window_root.id();
    let window_root_node = WindowNode::WindowRootContainer(window_root);
    let window_root_actual_shape = window_root.actual_shape();

    let mut base = Itree::new(window_root_node);

    let (viewport, cursor_viewport) = {
      let buffer = buffer.upgrade().unwrap();
      let buffer = lock!(buffer);
      let viewport = Viewport::view(opts, buffer.text(), window_root_actual_shape, 0, 0);
      let cursor_viewport = CursorViewport::from_top_left(&viewport, buffer.text());
      (viewport, cursor_viewport)
    };
    let viewport = Viewport::to_arc(viewport);
    let cursor_viewport = CursorViewport::to_arc(cursor_viewport);

    let window_content = WindowContent::new(shape, buffer.clone(), Arc::downgrade(&viewport));
    let window_content_id = window_content.id();
    let window_content_node = WindowNode::WindowContent(window_content);

    base.bounded_insert(window_root_id, window_content_node);

    Window {
      base,
      options: *opts,
      content_id: window_content_id,
      cursor_id: None,
      buffer,
      viewport,
      cursor_viewport,
    }
  }
}

inode_itree_impl!(Window, base);

impl Widgetable for Window {
  fn draw(&self, canvas: &mut Canvas) {
    for node in self.base.iter() {
      // trace!("Draw window:{:?}", node);
      node.draw(canvas);
    }
  }
}

impl Viewportable for Window {
  /// Get window local options.
  fn options(&self) -> &WindowLocalOptions {
    &self.options
  }

  /// Set window local options.
  fn set_options(&mut self, options: &WindowLocalOptions) {
    self.options = *options;
  }

  /// Get viewport.
  fn viewport(&self) -> ViewportArc {
    self.viewport.clone()
  }

  /// Set viewport.
  fn set_viewport(&mut self, viewport: ViewportArc) {
    self.viewport = viewport.clone();
    if let Some(WindowNode::WindowContent(content)) = self.base.node_mut(self.content_id) {
      content.set_viewport(Arc::downgrade(&viewport));
    }
  }

  /// Get cursor viewport.
  fn cursor_viewport(&self) -> CursorViewportArc {
    self.cursor_viewport.clone()
  }

  /// Set cursor viewport.
  fn set_cursor_viewport(&mut self, cursor_viewport: CursorViewportArc) {
    self.cursor_viewport = cursor_viewport;
  }
}

// Viewport {
impl Window {
  /// Get buffer.
  pub fn buffer(&self) -> BufferWk {
    self.buffer.clone()
  }

  /// Get window content widget.
  pub fn window_content(&self) -> &WindowContent {
    match self.base.node(self.content_id) {
      Some(WindowNode::WindowContent(w)) => w,
      _ => unreachable!(),
    }
  }
}
// Viewport }

#[allow(unused_imports)]
#[cfg(test)]
mod tests {
  use super::*;

  use compact_str::ToCompactString;
  use ropey::{Rope, RopeBuilder};
  use std::collections::BTreeMap;
  use std::fs::File;
  use std::io::{BufReader, BufWriter};
  use std::sync::Arc;
  use std::sync::Once;
  use tracing::info;

  use crate::buf::opt::{BufferLocalOptions, BufferLocalOptionsBuilder};
  use crate::buf::{Buffer, BufferArc};
  use crate::prelude::*;
  use crate::test::buf::{make_buffer_from_lines, make_empty_buffer};
  use crate::test::log::init as test_log_init;
  use crate::ui::tree::Tree;

  fn make_window_from_size(
    terminal_size: U16Size,
    buffer: BufferArc,
    window_options: &WindowLocalOptions,
  ) -> Window {
    let mut tree = Tree::new(terminal_size);
    tree.set_global_local_options(window_options);
    let window_shape = IRect::new(
      (0, 0),
      (
        terminal_size.width() as isize,
        terminal_size.height() as isize,
      ),
    );
    Window::new(
      tree.global_local_options(),
      window_shape,
      Arc::downgrade(&buffer),
    )
  }

  fn do_test_draw(actual: &Canvas, expect: &[&str]) {
    let actual = actual
      .frame()
      .raw_symbols()
      .iter()
      .map(|cs| cs.join(""))
      .collect::<Vec<_>>();
    info!("actual:{}", actual.len());
    for a in actual.iter() {
      info!("{:?}", a);
    }
    info!("expect:{}", expect.len());
    for e in expect.iter() {
      info!("{:?}", e);
    }

    assert_eq!(actual.len(), expect.len());
    for i in 0..actual.len() {
      let e = &expect[i];
      let a = &actual[i];
      info!("i-{}, actual[{}]:{:?}, expect[{}]:{:?}", i, i, a, i, e);
      assert_eq!(e.len(), a.len());
      assert_eq!(e, a);
    }
  }

  #[test]
  fn draw_after_init1() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "Hello, RSV",
      "This is a ",
      "But still ",
      "  1. When ",
      "  2. When ",
      "     * The",
      "     * The",
      "          ",
      "          ",
      "          ",
    ];

    let window_local_options = WindowLocalOptionsBuilder::default()
      .wrap(false)
      .build()
      .unwrap();
    let window = make_window_from_size(terminal_size, buf.clone(), &window_local_options);
    let mut actual = Canvas::new(terminal_size);
    window.draw(&mut actual);
    do_test_draw(&actual, &expect);
  }
}
