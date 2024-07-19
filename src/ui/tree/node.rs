//! Widget node in the tree.

use std::sync::{Arc, RwLock};

use crate::cart::{IPos, IRect, USize};
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

macro_rules! define_widget_node_getter {
  ($getter_name:ident,$return_type_name:ty) => {
    fn $getter_name(&self) -> $return_type_name {
      match self {
        Self::RootWidgetNode(node) => node.$getter_name(),
        Self::CursorNode(node) => node.$getter_name(),
        Self::WindowNode(node) => node.$getter_name(),
      }
    }
  };
}

macro_rules! define_widget_node_setter {
  ($setter_name:ident,$value_type_name:ty) => {
    fn $setter_name(&mut self, value: $value_type_name) {
      match self {
        Self::RootWidgetNode(node) => node.$setter_name(value),
        Self::CursorNode(node) => node.$setter_name(value),
        Self::WindowNode(node) => node.$setter_name(value),
      }
    }
  };
}

impl Widget for Node {
  define_widget_node_getter!(id, NodeId);

  fn draw(&mut self) {
    match self {
      Self::RootWidgetNode(node) => node.draw(),
      Self::CursorNode(node) => node.draw(),
      Self::WindowNode(node) => node.draw(),
    }
  }
}
