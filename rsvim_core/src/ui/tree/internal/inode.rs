//! Internal tree node.

use crate::prelude::*;
use crate::ui::tree::ItreeWk;
use crate::ui::tree::TreeNodeId;

pub trait Inodeable: Sized + Clone + std::fmt::Debug {
  fn id(&self) -> TreeNodeId;

  fn lotree(&self) -> ItreeWk;

  fn shape(&self) -> IRect;

  fn actual_shape(&self) -> U16Rect;

  /// Whether the node is visible, e.g. its actual_shape size is zero.
  fn visible(&self) -> bool;
  fn invisible(&self) -> bool;

  /// Whether the node is attached to a parent node.
  /// NOTE: The root node is always been considered as attached as well.
  fn attached(&self) -> bool;
  fn detached(&self) -> bool;

  /// The node is visible and its size > 0, e.g. both height and width > 0.
  fn enabled(&self) -> bool;
  fn disabled(&self) -> bool;
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

  fn invisible(&self) -> bool {
    self
      .lotree
      .upgrade()
      .unwrap()
      .borrow()
      .invisible(self.id)
      .unwrap()
  }

  fn attached(&self) -> bool {
    self.lotree.upgrade().unwrap().borrow().attached(self.id)
  }

  fn detached(&self) -> bool {
    self.lotree.upgrade().unwrap().borrow().detached(self.id)
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

  fn disabled(&self) -> bool {
    self
      .lotree
      .upgrade()
      .unwrap()
      .borrow()
      .disabled(self.id)
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

      fn invisible(&self) -> bool {
        self.base.invisible()
      }

      fn attached(&self) -> bool {
        self.base.attached()
      }

      fn detached(&self) -> bool {
        self.base.detached()
      }

      fn enabled(&self) -> bool {
        self.base.enabled()
      }

      fn disabled(&self) -> bool {
        self.base.disabled()
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

      fn invisible(&self) -> bool {
        match self {
          $(
            $enum::$variant(e) => e.invisible(),
          )*
        }
      }

      fn attached(&self) -> bool {
        match self {
          $(
            $enum::$variant(e) => e.attached(),
          )*
        }
      }

      fn detached(&self) -> bool {
        match self {
          $(
            $enum::$variant(e) => e.detached(),
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

      fn disabled(&self) -> bool {
        match self {
          $(
            $enum::$variant(e) => e.disabled(),
          )*
        }
      }
    }
  };
}
