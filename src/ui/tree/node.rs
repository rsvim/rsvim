//! Widget node in the tree.

use std::sync::{Arc, RwLock, Weak};

use crate::cart::{IRect, U16Rect};
use crate::ui::widget::cursor::Cursor;
use crate::ui::widget::root::RootWidget;
use crate::ui::widget::window::Window;
use crate::ui::widget::Widget;

pub type NodeId = usize;

/// Widget node in the tree.
pub enum Node {
  RootWidgetNode(RootWidget),
  CursorNode(Cursor),
  WindowNode(Window),
}

pub type NodePtr = Arc<RwLock<Node>>;
pub type NodeWk = Weak<RwLock<Node>>;

pub fn make_node_ptr(n: Node) -> Arc<RwLock<Node>> {
  Arc::new(RwLock::new(n))
}

impl PartialOrd for Node {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    self.id().partial_cmp(&other.id())
  }
}

impl PartialEq for Node {
  fn eq(&self, other: &Self) -> bool {
    self.id().eq(&other.id())
  }
}

impl Widget for Node {
  fn id(&self) -> NodeId {
    match self {
      Self::RootWidgetNode(node) => node.id(),
      Self::CursorNode(node) => node.id(),
      Self::WindowNode(node) => node.id(),
    }
  }

  fn draw(&mut self) {
    match self {
      Self::RootWidgetNode(node) => node.draw(),
      Self::CursorNode(node) => node.draw(),
      Self::WindowNode(node) => node.draw(),
    }
  }
}

#[derive(Copy, Clone)]
pub struct NodeAttribute {
  /// Relative and logical shape of a widget node.
  pub shape: IRect,

  /// Absolute and actual shape of a widget node.
  pub actual_shape: U16Rect,

  pub zindex: usize,
  pub visible: bool,
  pub enabled: bool,
}

impl NodeAttribute {
  pub fn new(
    shape: IRect,
    actual_shape: U16Rect,
    zindex: usize,
    visible: bool,
    enabled: bool,
  ) -> Self {
    NodeAttribute {
      shape,
      actual_shape,
      zindex,
      visible,
      enabled,
    }
  }

  pub fn default(shape: IRect, actual_shape: U16Rect) -> Self {
    NodeAttribute {
      shape,
      actual_shape,
      zindex: 0_usize,
      visible: true,
      enabled: true,
    }
  }
}
