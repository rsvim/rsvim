//! Vim window.

use crate::buf::BufferWk;
use crate::cart::{IRect, U16Rect};
use crate::envar;
use crate::ui::canvas::Canvas;
use crate::ui::tree::internal::{InodeId, Inodeable, Itree};
use crate::ui::widget::window::content::WindowContent;
use crate::ui::widget::window::root::WindowRootContainer;
use crate::ui::widget::Widgetable;
use crate::wlock;

// Re-export
pub use crate::ui::widget::window::opt::{
  ViewportOptions, WindowLocalOptions, WindowOptionsBuilder,
};
pub use crate::ui::widget::window::viewport::{
  LineViewport, LineViewportRow, Viewport, ViewportArc,
};

use std::convert::From;
use std::sync::Arc;
// use tracing::trace;

pub mod content;
pub mod opt;
pub mod root;
pub mod viewport;

#[allow(dead_code)]
#[derive(Debug, Clone)]
/// The Vim window, it manages all descendant widget nodes, i.e. all widgets in the
/// [`crate::ui::widget::window`] module.
pub struct Window {
  base: Itree<WindowNode>,

  // The Window content widget ID.
  content_id: InodeId,

  // Buffer.
  buffer: BufferWk,

  // Local window options.
  // By default these options will inherit from global options of UI.
  options: WindowLocalOptions,

  // Viewport.
  viewport: ViewportArc,
}

impl Window {
  pub fn new(shape: IRect, buffer: BufferWk, local_options: &WindowLocalOptions) -> Self {
    let options = local_options.clone();

    let window_root = WindowRootContainer::new(shape);
    let window_root_id = window_root.id();
    let window_root_node = WindowNode::WindowRootContainer(window_root);
    let window_root_actual_shape = *window_root_node.actual_shape();

    let viewport_options = ViewportOptions {
      wrap: options.wrap(),
      line_break: options.line_break(),
    };
    let viewport = Viewport::new(&viewport_options, buffer.clone(), &window_root_actual_shape);
    let viewport = Viewport::to_arc(viewport);

    let mut base = Itree::new(window_root_node);

    let window_content = WindowContent::new(shape, buffer.clone(), Arc::downgrade(&viewport));
    let window_content_id = window_content.id();
    let window_content_node = WindowNode::WindowContent(window_content);

    base.bounded_insert(&window_root_id, window_content_node);

    Window {
      base,
      content_id: window_content_id,
      buffer,
      options,
      viewport,
    }
  }
}

impl Inodeable for Window {
  fn id(&self) -> InodeId {
    self.base.root_id()
  }

  fn depth(&self) -> &usize {
    self.base.node(&self.base.root_id()).unwrap().depth()
  }

  fn depth_mut(&mut self) -> &mut usize {
    self
      .base
      .node_mut(&self.base.root_id())
      .unwrap()
      .depth_mut()
  }

  fn zindex(&self) -> &usize {
    self.base.node(&self.base.root_id()).unwrap().zindex()
  }

  fn zindex_mut(&mut self) -> &mut usize {
    self
      .base
      .node_mut(&self.base.root_id())
      .unwrap()
      .zindex_mut()
  }

  fn shape(&self) -> &IRect {
    self.base.node(&self.base.root_id()).unwrap().shape()
  }

  fn shape_mut(&mut self) -> &mut IRect {
    self
      .base
      .node_mut(&self.base.root_id())
      .unwrap()
      .shape_mut()
  }

  fn actual_shape(&self) -> &U16Rect {
    self.base.node(&self.base.root_id()).unwrap().actual_shape()
  }

  fn actual_shape_mut(&mut self) -> &mut U16Rect {
    self
      .base
      .node_mut(&self.base.root_id())
      .unwrap()
      .actual_shape_mut()
  }

  fn enabled(&self) -> &bool {
    self.base.node(&self.base.root_id()).unwrap().enabled()
  }

  fn enabled_mut(&mut self) -> &mut bool {
    self
      .base
      .node_mut(&self.base.root_id())
      .unwrap()
      .enabled_mut()
  }

  fn visible(&self) -> &bool {
    self.base.node(&self.base.root_id()).unwrap().visible()
  }

  fn visible_mut(&mut self) -> &mut bool {
    self
      .base
      .node_mut(&self.base.root_id())
      .unwrap()
      .visible_mut()
  }
}

impl Widgetable for Window {
  fn draw(&self, canvas: &mut Canvas) {
    for node in self.base.iter() {
      // trace!("Draw window:{:?}", node);
      node.draw(canvas);
    }
  }
}

// Options {
impl Window {
  /// Get window local options.
  pub fn options(&self) -> &WindowLocalOptions {
    &self.options
  }

  /// Set window local options.
  pub fn set_options(&mut self, options: &WindowLocalOptions) {
    self.options = options.clone();
    let viewport_options = ViewportOptions::from(&self.options);
    wlock!(self.viewport).set_options(&viewport_options);
  }

  pub fn wrap(&self) -> bool {
    self.options.wrap()
  }

  pub fn set_wrap(&mut self, value: bool) {
    self.options.set_wrap(value);
    let viewport_options = ViewportOptions::from(&self.options);
    wlock!(self.viewport).set_options(&viewport_options);
  }

  pub fn line_break(&self) -> bool {
    self.options.line_break()
  }

  pub fn set_line_break(&mut self, value: bool) {
    self.options.set_line_break(value);
    let viewport_options = ViewportOptions::from(&self.options);
    wlock!(self.viewport).set_options(&viewport_options);
  }

  /// Get viewport.
  pub fn viewport(&self) -> ViewportArc {
    self.viewport.clone()
  }

  /// Get buffer.
  pub fn buffer(&self) -> BufferWk {
    self.buffer.clone()
  }
}
// Options }

#[derive(Debug, Clone)]
/// The value holder for each window widget.
pub enum WindowNode {
  WindowRootContainer(WindowRootContainer),
  WindowContent(WindowContent),
}

macro_rules! window_node_generate_dispatch {
  ($self_name:ident,$method_name:ident) => {
    match $self_name {
      WindowNode::WindowRootContainer(n) => n.$method_name(),
      WindowNode::WindowContent(n) => n.$method_name(),
    }
  };
}

impl Inodeable for WindowNode {
  fn id(&self) -> InodeId {
    window_node_generate_dispatch!(self, id)
  }

  fn depth(&self) -> &usize {
    window_node_generate_dispatch!(self, depth)
  }

  fn depth_mut(&mut self) -> &mut usize {
    window_node_generate_dispatch!(self, depth_mut)
  }

  fn zindex(&self) -> &usize {
    window_node_generate_dispatch!(self, zindex)
  }

  fn zindex_mut(&mut self) -> &mut usize {
    window_node_generate_dispatch!(self, zindex_mut)
  }

  fn shape(&self) -> &IRect {
    window_node_generate_dispatch!(self, shape)
  }

  fn shape_mut(&mut self) -> &mut IRect {
    window_node_generate_dispatch!(self, shape_mut)
  }

  fn actual_shape(&self) -> &U16Rect {
    window_node_generate_dispatch!(self, actual_shape)
  }

  fn actual_shape_mut(&mut self) -> &mut U16Rect {
    window_node_generate_dispatch!(self, actual_shape_mut)
  }

  fn enabled(&self) -> &bool {
    window_node_generate_dispatch!(self, enabled)
  }

  fn enabled_mut(&mut self) -> &mut bool {
    window_node_generate_dispatch!(self, enabled_mut)
  }

  fn visible(&self) -> &bool {
    window_node_generate_dispatch!(self, visible)
  }

  fn visible_mut(&mut self) -> &mut bool {
    window_node_generate_dispatch!(self, visible_mut)
  }
}

impl Widgetable for WindowNode {
  /// Draw widget on the canvas.
  fn draw(&self, canvas: &mut Canvas) {
    match self {
      WindowNode::WindowRootContainer(w) => w.draw(canvas),
      WindowNode::WindowContent(w) => w.draw(canvas),
    }
  }
}

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

  use crate::buf::{Buffer, BufferArc};
  use crate::cart::U16Size;
  use crate::test::buf::{make_buffer_from_lines, make_empty_buffer};
  #[allow(dead_code)]
  use crate::test::log::init as test_log_init;
  use crate::ui::tree::Tree;

  fn make_window_from_size(
    size: U16Size,
    buffer: BufferArc,
    window_options: &WindowLocalOptions,
  ) -> Window {
    let mut tree = Tree::new(size);
    tree.set_local_options(window_options);
    let window_shape = IRect::new((0, 0), (size.width() as isize, size.height() as isize));
    Window::new(window_shape, Arc::downgrade(&buffer), tree.local_options())
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

    let buffer = make_buffer_from_lines(vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ]);
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

    let terminal_size = U16Size::new(10, 10);
    let window_local_options = WindowLocalOptions::builder().wrap(false).build();
    let window = make_window_from_size(terminal_size, buffer.clone(), &window_local_options);
    let mut actual = Canvas::new(terminal_size);
    window.draw(&mut actual);
    do_test_draw(&actual, &expect);
  }
}
