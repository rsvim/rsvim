//! The VIM window.

use compact_str::CompactString;
use tracing::debug;

use crate::cart::{IRect, U16Pos, U16Rect};
use crate::inode_value_generate_impl;
use crate::ui::canvas::Canvas;
use crate::ui::tree::internal::inode::{Inode, InodeId, InodeValue};
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
  base: Itree<WindowValue>,

  // The Window content widget ID.
  content_id: InodeId,
}

impl Window {
  pub fn new(shape: IRect) -> Self {
    let window_root = WindowRootContainer::new();
    let window_root_id = window_root.id();
    let window_root_node = Inode::new(WindowValue::WindowRootContainer(window_root), shape);

    let mut base = Itree::new(window_root_node);

    let window_content = WindowContent::new();
    let window_content_id = window_content.id();
    let window_content_node = Inode::new(WindowValue::WindowContent(window_content), shape);

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
      node.value_mut().draw(actual_shape, canvas);
    }
  }
}

impl Window {
  pub fn lines(&self) -> &Vec<CompactString> {
    if let WindowValue::WindowContent(c) = self.base.node(&self.content_id).unwrap().value() {
      c.lines()
    } else {
      unreachable!()
    }
  }

  pub fn lines_mut(&mut self) -> &mut Vec<CompactString> {
    if let WindowValue::WindowContent(c) = self.base.node_mut(&self.content_id).unwrap().value_mut()
    {
      c.lines_mut()
    } else {
      unreachable!()
    }
  }

  pub fn line(&self, index: usize) -> &CompactString {
    if let WindowValue::WindowContent(c) = self.base.node(&self.content_id).unwrap().value() {
      c.line(index)
    } else {
      unreachable!()
    }
  }

  pub fn line_mut(&mut self, index: usize) -> &mut CompactString {
    if let WindowValue::WindowContent(c) = self.base.node_mut(&self.content_id).unwrap().value_mut()
    {
      c.line_mut(index)
    } else {
      unreachable!()
    }
  }

  pub fn line_wrap(&self) -> bool {
    if let WindowValue::WindowContent(c) = self.base.node(&self.content_id).unwrap().value() {
      c.line_wrap()
    } else {
      unreachable!()
    }
  }

  pub fn set_line_wrap(&mut self, line_wrap: bool) -> bool {
    if let WindowValue::WindowContent(c) = self.base.node_mut(&self.content_id).unwrap().value_mut()
    {
      c.set_line_wrap(line_wrap)
    } else {
      unreachable!()
    }
  }

  pub fn word_wrap(&self) -> bool {
    if let WindowValue::WindowContent(c) = self.base.node(&self.content_id).unwrap().value() {
      c.word_wrap()
    } else {
      unreachable!()
    }
  }

  pub fn set_word_wrap(&mut self, word_wrap: bool) -> bool {
    if let WindowValue::WindowContent(c) = self.base.node_mut(&self.content_id).unwrap().value_mut()
    {
      c.set_word_wrap(word_wrap)
    } else {
      unreachable!()
    }
  }
}

#[derive(Debug, Clone)]
/// The value holder for each window widget.
pub enum WindowValue {
  WindowContent(WindowContent),
  WindowRootContainer(WindowRootContainer),
}

impl InodeValue for WindowValue {
  fn id(&self) -> InodeId {
    match self {
      WindowValue::WindowContent(w) => w.id(),
      WindowValue::WindowRootContainer(w) => w.id(),
    }
  }
}

impl Widget for WindowValue {
  /// Get widget ID.
  fn id(&self) -> WidgetId {
    match self {
      WindowValue::WindowRootContainer(w) => w.id(),
      WindowValue::WindowContent(w) => w.id(),
    }
  }

  /// Draw widget with (already calculated) actual shape, on the canvas.
  fn draw(&mut self, actual_shape: U16Rect, canvas: &mut Canvas) {
    match self {
      WindowValue::WindowRootContainer(w) => w.draw(actual_shape, canvas),
      WindowValue::WindowContent(w) => w.draw(actual_shape, canvas),
    }
  }
}
