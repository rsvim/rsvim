//! Internal tree node.

use crate::prelude::*;
use crate::ui::tree::IrelationshipRc;
use crate::ui::tree::TreeNodeId;
use taffy::TaffyResult;

pub trait Inodeable: Sized + Clone + std::fmt::Debug {
  fn id(&self) -> TreeNodeId;

  fn relationship(&self) -> IrelationshipRc;

  fn shape(&self) -> IRect;

  fn actual_shape(&self) -> U16Rect;

  fn visible(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct InodeBase {
  relationship: IrelationshipRc,
  id: TreeNodeId,
}

impl InodeBase {
  pub fn new(relationship: IrelationshipRc, id: TreeNodeId) -> Self {
    Self { id, relationship }
  }
}

impl Inodeable for InodeBase {
  fn id(&self) -> TreeNodeId {
    self.id
  }

  fn relationship(&self) -> IrelationshipRc {
    self.relationship.clone()
  }

  fn shape(&self) -> IRect {
    self.relationship.borrow().shape(self.id).unwrap()
  }

  fn actual_shape(&self) -> U16Rect {
    self.relationship.borrow().actual_shape(self.id).unwrap()
  }

  fn visible(&self) -> bool {
    self.relationship.borrow().visible(self.id).unwrap()
  }
}

#[macro_export]
macro_rules! inode_impl {
  ($name:tt) => {
    impl Inodeable for $name {
      fn id(&self) -> TreeNodeId {
        self.base.id()
      }

      fn relationship(&self) -> IrelationshipRc {
        self.base.relationship()
      }

      fn shape(&self) -> TaffyResult<IRect> {
        self.base.shape()
      }

      fn actual_shape(&self) -> TaffyResult<U16Rect> {
        self.base.actual_shape()
      }

      fn visible(&self) -> TaffyResult<bool> {
        self.base.visible()
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

      fn relationship(&self) -> IrelationshipRc {
        match self {
          $(
            $enum::$variant(e) => e.relationship(),
          )*
        }
      }

      fn shape(&self) -> TaffyResult<IRect> {
        match self {
          $(
            $enum::$variant(e) => e.shape(),
          )*
        }
      }

      fn actual_shape(&self) -> TaffyResult<U16Rect> {
        match self {
          $(
            $enum::$variant(e) => e.actual_shape(),
          )*
        }
      }

      fn visible(&self) -> TaffyResult<bool> {
        match self {
          $(
            $enum::$variant(e) => e.visible(),
          )*
        }
      }
    }
  };
}
