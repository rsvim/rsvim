//! Widget node in the tree.

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::cart::{IRect, U16Rect};
use crate::ui::term::TerminalWk;
use crate::ui::widget::cursor::Cursor;
use crate::ui::widget::root::RootWidget;
use crate::ui::widget::window::Window;
use crate::ui::widget::Widget;

pub type NodeId = usize;

/// Widget node in the tree.
#[derive(Debug)]
pub enum Node {
  RootWidgetNode(RootWidget),
  CursorNode(Cursor),
  WindowNode(Window),
}

pub type NodePtr = Rc<RefCell<Node>>;
pub type NodeWk = Weak<RefCell<Node>>;

pub fn make_node_ptr(n: Node) -> NodePtr {
  Rc::new(RefCell::new(n))
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

  fn draw(&mut self, actual_shape: &U16Rect, terminal: TerminalWk) {
    match self {
      Self::RootWidgetNode(node) => node.draw(actual_shape, terminal.clone()),
      Self::CursorNode(node) => node.draw(actual_shape, terminal.clone()),
      Self::WindowNode(node) => node.draw(actual_shape, terminal.clone()),
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
