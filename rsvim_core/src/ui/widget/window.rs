//! Vim window.

use crate::buf::BufferWk;
use crate::cart::{IRect, U16Pos, U16Rect};
use crate::defaults;
use crate::ui::canvas::{Canvas, Cell};
use crate::ui::tree::internal::{InodeId, Inodeable, Itree};
use crate::ui::tree::Tree;
use crate::ui::util::ptr::SafeTreeRef;
use crate::ui::widget::window::content::WindowContent;
use crate::ui::widget::window::root::WindowRootContainer;
use crate::ui::widget::Widgetable;

// Re-export
pub use crate::ui::widget::window::opt::{WindowLocalOptions, WindowOptionsBuilder};
pub use crate::ui::widget::window::viewport::{
  LineViewport, LineViewportRow, Viewport, ViewportOptions,
};

use crossterm::style::{Attributes, Color};
use geo::point;
use regex::Regex;
use ropey::RopeSlice;
use std::collections::{BTreeSet, VecDeque};
use std::convert::From;
use std::ptr::NonNull;
use std::time::Duration;
use tracing::{debug, error};

pub mod content;
pub mod opt;
pub mod root;
pub mod viewport;

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

  // Tree ref.
  tree_ref: SafeTreeRef,

  // Viewport.
  viewport: Viewport,
}

impl Window {
  pub fn new(shape: IRect, buffer: BufferWk, tree: &mut Tree) -> Self {
    let options = tree.local_options().clone();

    let window_root = WindowRootContainer::new(shape);
    let window_root_id = window_root.id();
    let window_root_node = WindowNode::WindowRootContainer(window_root);

    let mut base = Itree::new(window_root_node);
    let root_id = base.root_id();

    let window_content = WindowContent::new(shape, buffer.clone(), tree);
    let window_content_id = window_content.id();
    let window_content_node = WindowNode::WindowContent(window_content);

    base.bounded_insert(&window_root_id, window_content_node);

    let viewport_options = ViewportOptions {
      wrap: options.wrap(),
      line_break: options.line_break(),
    };
    let viewport_actual_shape = base.node(&window_content_id).unwrap().actual_shape();
    let viewport = Viewport::new(&viewport_options, buffer.clone(), viewport_actual_shape);

    Window {
      base,
      content_id: window_content_id,
      buffer,
      options,
      tree_ref: SafeTreeRef::new(tree, root_id),
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
  fn draw(&mut self, canvas: &mut Canvas) {
    for node in self.base.iter_mut() {
      debug!("draw node:{:?}", node);
      node.draw(canvas);
    }
  }
}

// Options {
impl Window {
  pub fn options(&self) -> &WindowLocalOptions {
    &self.options
  }

  pub fn set_options(&mut self, options: &WindowLocalOptions) {
    self.options = options.clone();
    self.update_window_content_options();
  }

  pub fn wrap(&self) -> bool {
    self.options.wrap()
  }

  pub fn set_wrap(&mut self, value: bool) {
    self.options.set_wrap(value);
    self.update_window_content_options();
  }

  pub fn line_break(&self) -> bool {
    self.options.line_break()
  }

  pub fn set_line_break(&mut self, value: bool) {
    self.options.set_line_break(value);
    self.update_window_content_options();
  }

  fn update_window_content_options(&mut self) {
    match self.base.node_mut(&self.content_id).unwrap() {
      WindowNode::WindowContent(content) => content.set_options(&self.options),
      _ => unreachable!("Cannot find window_content node"),
    }
  }
}
// Options }

impl Window {
  pub fn buffer(&self) -> BufferWk {
    self.buffer.clone()
  }
}

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
  fn draw(&mut self, canvas: &mut Canvas) {
    match self {
      WindowNode::WindowRootContainer(w) => w.draw(canvas),
      WindowNode::WindowContent(w) => w.draw(canvas),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use compact_str::ToCompactString;
  use ropey::{Rope, RopeBuilder};
  use std::fs::File;
  use std::io::{BufReader, BufWriter};
  use std::sync::Arc;
  use std::sync::Once;
  use tracing::info;

  use crate::buf::{Buffer, BufferArc};
  use crate::cart::U16Size;
  #[allow(dead_code)]
  use crate::test::log::init as test_log_init;

  #[allow(dead_code)]
  static INIT: Once = Once::new();
}
