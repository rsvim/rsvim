//! Vim window.

use crate::buf::BufferWk;
use crate::lock;
use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::tree::*;
use crate::ui::viewport::ViewportOptions;
use crate::ui::viewport::{CursorViewport, CursorViewportArc, Viewport, ViewportArc};
use crate::ui::widget::Widgetable;
use crate::ui::widget::window::content::WindowContent;
use crate::ui::widget::window::root::WindowRootContainer;

// Re-export
pub use crate::ui::widget::window::opt::*;

use std::sync::Arc;
// use tracing::trace;

pub mod content;
pub mod opt;
pub mod root;

#[derive(Debug, Clone)]
/// The Vim window, it manages all descendant widget nodes, i.e. all widgets in the
/// [`crate::ui::widget::window`] module.
pub struct Window {
  base: Itree<WindowNode>,

  // The Window content widget ID.
  content_id: TreeNodeId,

  // Buffer.
  buffer: BufferWk,

  // Local window options.
  // By default these options will inherit from global options of UI.
  options: WindowLocalOptions,

  // Viewport.
  viewport: ViewportArc,

  // Cursor viewport.
  cursor_viewport: CursorViewportArc,
}

impl Window {
  pub fn new(shape: IRect, buffer: BufferWk, local_options: &WindowLocalOptions) -> Self {
    let options = *local_options;

    let window_root = WindowRootContainer::new(shape);
    let window_root_id = window_root.id();
    let window_root_node = WindowNode::WindowRootContainer(window_root);
    let window_root_actual_shape = *window_root_node.actual_shape();

    let (viewport, cursor_viewport) = {
      let buffer = buffer.upgrade().unwrap();
      let buffer = lock!(buffer);
      let viewport_opts = ViewportOptions::from(local_options);
      let viewport = Viewport::view(
        &viewport_opts,
        buffer.text(),
        &window_root_actual_shape,
        0,
        0,
      );
      let cursor_viewport = CursorViewport::from_top_left(&viewport, buffer.text());
      (viewport, cursor_viewport)
    };
    let viewport = Viewport::to_arc(viewport);
    let cursor_viewport = CursorViewport::to_arc(cursor_viewport);

    let mut base = Itree::new(window_root_node);

    let window_content = WindowContent::new(shape, buffer.clone(), Arc::downgrade(&viewport));
    let window_content_id = window_content.id();
    let window_content_node = WindowNode::WindowContent(window_content);

    base.bounded_insert(window_root_id, window_content_node);

    Window {
      base,
      content_id: window_content_id,
      buffer,
      options,
      viewport,
      cursor_viewport,
    }
  }
}

impl Inodeable for Window {
  fn id(&self) -> TreeNodeId {
    self.base.root_id()
  }

  fn depth(&self) -> usize {
    self.base.node(self.base.root_id()).unwrap().depth()
  }

  fn set_depth(&mut self, depth: usize) {
    self
      .base
      .node_mut(self.base.root_id())
      .unwrap()
      .set_depth(depth);
  }

  fn zindex(&self) -> usize {
    self.base.node(self.base.root_id()).unwrap().zindex()
  }

  fn set_zindex(&mut self, zindex: usize) {
    self
      .base
      .node_mut(self.base.root_id())
      .unwrap()
      .set_zindex(zindex);
  }

  fn shape(&self) -> &IRect {
    self.base.node(self.base.root_id()).unwrap().shape()
  }

  fn set_shape(&mut self, shape: &IRect) {
    self
      .base
      .node_mut(self.base.root_id())
      .unwrap()
      .set_shape(shape);
  }

  fn actual_shape(&self) -> &U16Rect {
    self.base.node(self.base.root_id()).unwrap().actual_shape()
  }

  fn set_actual_shape(&mut self, actual_shape: &U16Rect) {
    self
      .base
      .node_mut(self.base.root_id())
      .unwrap()
      .set_actual_shape(actual_shape);
  }

  fn enabled(&self) -> bool {
    self.base.node(self.base.root_id()).unwrap().enabled()
  }

  fn set_enabled(&mut self, enabled: bool) {
    self
      .base
      .node_mut(self.base.root_id())
      .unwrap()
      .set_enabled(enabled);
  }

  fn visible(&self) -> bool {
    self.base.node(self.base.root_id()).unwrap().visible()
  }

  fn set_visible(&mut self, visible: bool) {
    self
      .base
      .node_mut(self.base.root_id())
      .unwrap()
      .set_visible(visible);
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
    self.options = *options;
  }

  pub fn wrap(&self) -> bool {
    self.options.wrap()
  }

  pub fn set_wrap(&mut self, value: bool) {
    self.options.set_wrap(value);
  }

  pub fn line_break(&self) -> bool {
    self.options.line_break()
  }

  pub fn set_line_break(&mut self, value: bool) {
    self.options.set_line_break(value);
  }

  pub fn scroll_off(&self) -> u16 {
    self.options.scroll_off()
  }

  pub fn set_scroll_off(&mut self, value: u16) {
    self.options.set_scroll_off(value);
  }
}
// Options }

// Viewport {
impl Window {
  /// Get viewport.
  pub fn viewport(&self) -> ViewportArc {
    self.viewport.clone()
  }

  /// Set viewport.
  pub fn set_viewport(&mut self, viewport: ViewportArc) {
    self.viewport = viewport.clone();
    if let Some(WindowNode::WindowContent(content)) = self.base.node_mut(self.content_id) {
      content.set_viewport(Arc::downgrade(&viewport));
    }
  }

  /// Get cursor viewport.
  pub fn cursor_viewport(&self) -> CursorViewportArc {
    self.cursor_viewport.clone()
  }

  /// Set cursor viewport.
  pub fn set_cursor_viewport(&mut self, cursor_viewport: CursorViewportArc) {
    self.cursor_viewport = cursor_viewport;
  }

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

#[derive(Debug, Clone)]
/// The value holder for each window widget.
pub enum WindowNode {
  WindowRootContainer(WindowRootContainer),
  WindowContent(WindowContent),
}

macro_rules! window_node_getter {
  ($self_name:ident,$method_name:ident) => {
    match $self_name {
      WindowNode::WindowRootContainer(n) => n.$method_name(),
      WindowNode::WindowContent(n) => n.$method_name(),
    }
  };
}

macro_rules! window_node_setter {
  ($self_name:ident,$method_name:ident,$method_arg:ident) => {
    match $self_name {
      WindowNode::WindowRootContainer(n) => n.$method_name($method_arg),
      WindowNode::WindowContent(n) => n.$method_name($method_arg),
    }
  };
}

impl Inodeable for WindowNode {
  fn id(&self) -> TreeNodeId {
    window_node_getter!(self, id)
  }

  fn depth(&self) -> usize {
    window_node_getter!(self, depth)
  }

  fn set_depth(&mut self, depth: usize) {
    window_node_setter!(self, set_depth, depth)
  }

  fn zindex(&self) -> usize {
    window_node_getter!(self, zindex)
  }

  fn set_zindex(&mut self, zindex: usize) {
    window_node_setter!(self, set_zindex, zindex)
  }

  fn shape(&self) -> &IRect {
    window_node_getter!(self, shape)
  }

  fn set_shape(&mut self, shape: &IRect) {
    window_node_setter!(self, set_shape, shape)
  }

  fn actual_shape(&self) -> &U16Rect {
    window_node_getter!(self, actual_shape)
  }

  fn set_actual_shape(&mut self, actual_shape: &U16Rect) {
    window_node_setter!(self, set_actual_shape, actual_shape)
  }

  fn enabled(&self) -> bool {
    window_node_getter!(self, enabled)
  }

  fn set_enabled(&mut self, enabled: bool) {
    window_node_setter!(self, set_enabled, enabled)
  }

  fn visible(&self) -> bool {
    window_node_getter!(self, visible)
  }

  fn set_visible(&mut self, visible: bool) {
    window_node_setter!(self, set_visible, visible)
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

  use crate::buf::{Buffer, BufferArc, BufferLocalOptions, BufferLocalOptionsBuilder};
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
      window_shape,
      Arc::downgrade(&buffer),
      tree.global_local_options(),
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
      terminal_size.height(),
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
