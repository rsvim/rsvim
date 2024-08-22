//! The VIM window.

use std::collections::VecDeque;

use compact_str::CompactString;
use tracing::debug;

use crate::buf::BufferWk;
use crate::cart::{IRect, U16Pos, U16Rect};
use crate::inode_value_generate_impl;
use crate::ui::canvas::Canvas;
use crate::ui::tree::internal::inode::{InodeId, InodeValue};
use crate::ui::tree::internal::itree::{Itree, ItreeIter, ItreeIterMut};
use crate::ui::widget::window::content::WindowContent;
use crate::ui::widget::window::root::WindowRootContainer;
use crate::ui::widget::{Widget, WidgetId};

pub mod content;
pub mod root;

#[derive(Debug, Clone)]
/// The VIM window, it manages all descendant widget nodes, i.e. all widgets in the
/// [`crate::ui::widget::window`] module.
pub struct Window {
  base: Itree<WindowNode>,

  // The Window content widget ID.
  content_id: InodeId,
}

impl Window {
  pub fn new(shape: IRect, buffer: BufferWk) -> Self {
    let window_root = WindowRootContainer::new(shape);
    let window_root_id = window_root.id();
    let window_root_node = WindowNode::WindowRootContainer(window_root);

    let mut base = Itree::new(window_root_node);

    let window_content = WindowContent::new(shape, buffer);
    let window_content_id = window_content.id();
    let window_content_node = WindowNode::WindowContent(window_content);

    base.bounded_insert(&window_root_id, window_content_node);

    Window {
      base,
      content_id: window_content_id,
    }
  }
}

impl InodeValue for Window {
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

impl Widget for Window {
  fn id(&self) -> WidgetId {
    self.base.root_id()
  }

  fn draw(&mut self, _actual_shape: U16Rect, canvas: &mut Canvas) {
    // Do nothing.
    for node in self.base.iter_mut() {
      debug!("draw node:{:?}", node);
      let actual_shape = *node.actual_shape();
      node.draw(actual_shape, canvas);
    }
  }
}

impl Window {
  pub fn lines(&self) -> &VecDeque<CompactString> {
    if let WindowNode::WindowContent(c) = self.base.node(&self.content_id).unwrap() {
      c.lines()
    } else {
      unreachable!()
    }
  }

  pub fn line(&self, index: usize) -> &CompactString {
    if let WindowNode::WindowContent(c) = self.base.node(&self.content_id).unwrap() {
      c.line(index)
    } else {
      unreachable!()
    }
  }

  pub fn set_line(&mut self, index: usize, line: CompactString) -> &mut CompactString {
    if let WindowNode::WindowContent(c) = self.base.node_mut(&self.content_id).unwrap() {
      c.set_line(index, line)
    } else {
      unreachable!()
    }
  }

  pub fn line_wrap(&self) -> bool {
    if let WindowNode::WindowContent(c) = self.base.node(&self.content_id).unwrap().value() {
      c.line_wrap()
    } else {
      unreachable!()
    }
  }

  pub fn set_line_wrap(&mut self, line_wrap: bool) -> bool {
    if let WindowNode::WindowContent(c) = self.base.node_mut(&self.content_id).unwrap().value_mut()
    {
      c.set_line_wrap(line_wrap)
    } else {
      unreachable!()
    }
  }

  pub fn word_wrap(&self) -> bool {
    if let WindowNode::WindowContent(c) = self.base.node(&self.content_id).unwrap().value() {
      c.word_wrap()
    } else {
      unreachable!()
    }
  }

  pub fn set_word_wrap(&mut self, word_wrap: bool) -> bool {
    if let WindowNode::WindowContent(c) = self.base.node_mut(&self.content_id).unwrap().value_mut()
    {
      c.set_word_wrap(word_wrap)
    } else {
      unreachable!()
    }
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

impl InodeValue for WindowNode {
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

impl Widget for WindowNode {
  /// Get widget ID.
  fn id(&self) -> WidgetId {
    window_node_generate_dispatch!(self, id)
  }

  /// Draw widget with (already calculated) actual shape, on the canvas.
  fn draw(&mut self, actual_shape: U16Rect, canvas: &mut Canvas) {
    match self {
      WindowNode::WindowRootContainer(w) => w.draw(actual_shape, canvas),
      WindowNode::WindowContent(w) => w.draw(actual_shape, canvas),
    }
  }
}
