//! Internal tree node.

use crate::prelude::*;
use crate::ui::tree::IrelationshipRc;
use crate::ui::tree::TreeNodeId;
use taffy::TaffyResult;

pub trait Inodeable: Sized + Clone + std::fmt::Debug {
  fn id(&self) -> TreeNodeId;

  fn relationship(&self) -> IrelationshipRc;

  fn shape(&self) -> TaffyResult<IRect>;

  fn actual_shape(&self) -> TaffyResult<U16Rect>;

  fn visible(&self) -> bool;
}

#[macro_export]
macro_rules! inode_impl {
  ($name:tt) => {
    impl Inodeable for $name {
      fn id(&self) -> TreeNodeId {
        self.id
      }

      fn relationship(&self) -> IrelationshipRc {
        self.base.clone()
      }

      fn relationship(&self) -> IrelationshipRc {
        self.base.clone()
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
    }
  };
}
