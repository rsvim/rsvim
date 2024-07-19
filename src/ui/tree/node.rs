//! Widget node in the tree.

use std::sync::{Arc, RwLock};

use crate::cart::{IPos, IRect, ISize, Size, URect, USize};
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

pub struct NodeAttribute {
  /// Relative and logical shape of a widget node, based on its parent (when it doesn't have a
  /// parent, the terminal is its parent).
  ///
  /// The coordinate system by default uses relative and logical shape, this is mostly for the
  /// convenience of calculation.
  shape: IRect,

  // Absolute and actual shape of a widget node.
  actual_shape: URect,

  /// The "z-index" arranges the display priority of the content stack when multiple children overlap
  /// on each other, a widget with higher z-index has higher priority to be displayed.
  ///
  /// Note: The z-index only works for the children under the same parent. For a child widget, it
  /// always covers/overrides its parent display. To change the visibility priority between
  /// children and parent, you need to change the relationship between them.
  /// For example, now we have two children under the same parent: A and B. A has 100 z-index, B
  /// has 10 z-index. Now B has a child: C, with z-index 1000. Even the z-index 1000 > 100 > 10, A
  /// still covers C, because it's a sibling of B.
  zindex: usize,
}
