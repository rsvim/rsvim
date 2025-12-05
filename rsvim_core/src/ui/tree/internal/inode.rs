//! Internal tree node.

use crate::prelude::*;
use crate::ui::tree::ItreeWk;
use crate::ui::tree::TreeNodeId;

pub trait Inodeable: Sized + Clone + std::fmt::Debug {
  fn id(&self) -> TreeNodeId;

  fn lotree(&self) -> ItreeWk;

  fn shape(&self) -> IRect;

  fn actual_shape(&self) -> U16Rect;

  /// The node is visible, e.g. its style is not `display: none`.
  fn visible(&self) -> bool;

  /// The node is visible and its size > 0, e.g. both height and width > 0.
  fn enabled(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct InodeBase {
  lotree: ItreeWk,
  id: TreeNodeId,
}

impl InodeBase {
  pub fn new(lotree: ItreeWk, id: TreeNodeId) -> Self {
    Self { lotree, id }
  }
}

impl Inodeable for InodeBase {
  fn id(&self) -> TreeNodeId {
    self.id
  }

  fn lotree(&self) -> ItreeWk {
    self.lotree.clone()
  }

  fn shape(&self) -> IRect {
    self
      .lotree
      .upgrade()
      .unwrap()
      .borrow()
      .shape(self.id())
      .unwrap()
  }

  fn actual_shape(&self) -> U16Rect {
    self
      .lotree
      .upgrade()
      .unwrap()
      .borrow()
      .actual_shape(self.id)
      .unwrap()
  }

  fn visible(&self) -> bool {
    self
      .lotree
      .upgrade()
      .unwrap()
      .borrow()
      .visible(self.id)
      .unwrap()
  }

  fn enabled(&self) -> bool {
    self
      .lotree
      .upgrade()
      .unwrap()
      .borrow()
      .enabled(self.id)
      .unwrap()
  }
}

#[macro_export]
macro_rules! inode_impl {
  ($name:tt) => {
    impl Inodeable for $name {
      fn id(&self) -> TreeNodeId {
        self.base.id()
      }

      fn lotree(&self) -> ItreeWk {
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

      fn enabled(&self) -> bool {
        self.base.enabled()
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

      fn lotree(&self) -> ItreeWk {
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

      fn enabled(&self) -> bool {
        match self {
          $(
            $enum::$variant(e) => e.enabled(),
          )*
        }
      }
    }
  };
}
