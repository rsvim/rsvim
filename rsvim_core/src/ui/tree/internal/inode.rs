//! Internal tree node.

use crate::prelude::*;
use crate::ui::tree::ItreeRc;
use crate::ui::tree::TreeNodeId;

pub trait Inodeable: Sized + Clone + std::fmt::Debug {
  fn id(&self) -> TreeNodeId;

  fn lotree(&self) -> ItreeRc;

  fn shape(&self) -> IRect;

  fn actual_shape(&self) -> U16Rect;

  fn visible(&self) -> bool;

  fn layout(&self) -> taffy::Layout;

  fn style(&self) -> taffy::Style;
}

#[derive(Debug, Clone)]
pub struct InodeBase {
  lotree: ItreeRc,
  id: TreeNodeId,
}

impl InodeBase {
  pub fn new(lotree: ItreeRc, id: TreeNodeId) -> Self {
    Self { lotree, id }
  }
}

impl Inodeable for InodeBase {
  fn id(&self) -> TreeNodeId {
    self.id
  }

  fn lotree(&self) -> ItreeRc {
    self.lotree.clone()
  }

  fn shape(&self) -> IRect {
    self.lotree.borrow().shape(self.id).unwrap()
  }

  fn actual_shape(&self) -> U16Rect {
    self.lotree.borrow().actual_shape(self.id).unwrap()
  }

  fn visible(&self) -> bool {
    self.lotree.borrow().visible(self.id).unwrap()
  }

  fn layout(&self) -> taffy::Layout {
    self.lotree.borrow().layout(self.id).unwrap().clone()
  }

  fn style(&self) -> taffy::Style {
    self.lotree.borrow().style(self.id).unwrap().clone()
  }
}

#[macro_export]
macro_rules! inode_impl {
  ($name:tt) => {
    impl Inodeable for $name {
      fn id(&self) -> TreeNodeId {
        self.base.id()
      }

      fn lotree(&self) -> ItreeRc {
        self.base.lotree()
      }

      fn shape(&self) -> IRect {
        self.base.shape()
      }

      fn actual_shape(&self) -> U16Rect {
        self.base.actual_shape()
      }

      fn visible(&self) -> bool {
        self.base.visible()
      }

      fn layout(&self) -> taffy::Layout {
        self.base.layout()
      }

      fn style(&self) -> taffy::Style {
        self.base.style()
      }
    }
  };
}

#[macro_export]
macro_rules! inode_dispatcher {
  ($enum:ident, $($variant:tt),*) => {
    impl Inodeable for $enum {
      fn id(&self) -> TreeNodeId {
        match self {
          $(
            $enum::$variant(e) => e.id(),
          )*
        }
      }

      fn lotree(&self) -> ItreeRc {
        match self {
          $(
            $enum::$variant(e) => e.lotree(),
          )*
        }
      }

      fn shape(&self) -> IRect {
        match self {
          $(
            $enum::$variant(e) => e.shape(),
          )*
        }
      }

      fn actual_shape(&self) -> U16Rect {
        match self {
          $(
            $enum::$variant(e) => e.actual_shape(),
          )*
        }
      }

      fn visible(&self) -> bool {
        match self {
          $(
            $enum::$variant(e) => e.visible(),
          )*
        }
      }

      fn layout(&self) -> taffy::Layout {
        match self {
          $(
            $enum::$variant(e) => e.layout(),
          )*
        }
      }

      fn style(&self) -> taffy::Style {
        match self {
          $(
            $enum::$variant(e) => e.style(),
          )*
        }
      }
    }
  };
}
