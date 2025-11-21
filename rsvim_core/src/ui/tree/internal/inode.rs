//! The node structure of the internal tree.

use crate::prelude::*;
use crate::ui::tree::IrelationshipRc;
use crate::ui::tree::TreeNodeId;

pub trait Inodeable: Sized + Clone + std::fmt::Debug {
  fn id(&self) -> TreeNodeId;

  fn visible(&self) -> bool;
}

/// Generate getter/setter for `Inode`.
#[macro_export]
macro_rules! inode_impl {
  ($struct_name:ty,$base_name:ident) => {
    impl Inodeable for $struct_name {
      fn id(&self) -> TreeNodeId {
        self.$base_name.id()
      }

      fn relationship(&self) -> IrelationshipRc {
        self.$base_name.relationship()
      }

      fn shape(&self) -> IRect {
        self.$base_name.actual_shape()
      }

      fn actual_shape(&self) -> U16Rect {
        self.$base_name.actual_shape()
      }

      fn visible(&self) -> bool {
        self.$base_name.visible()
      }
    }
  };
}

/// Generate enum dispatcher for `Inode`.
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

      fn shape(&self) -> IRect {
        match self {
          $(
            $enum::$variant(e) => e.actual_shape(),
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

    }
  }
}

#[derive(Debug, Clone)]
/// The internal tree node, it's both a container for the widgets and common attributes.
pub struct InodeBase {
  relationship: IrelationshipRc,
  id: TreeNodeId,
}

impl InodeBase {
  pub fn new(relationship: IrelationshipRc, id: TreeNodeId) -> Self {
    InodeBase { relationship, id }
  }

  pub fn id(&self) -> TreeNodeId {
    self.id
  }

  pub fn relationship(&self) -> IrelationshipRc {
    self.relationship.clone()
  }

  pub fn actual_shape(&self) -> U16Rect {
    let rel = self.relationship.borrow();
    let layout = rel.layout(self.id).unwrap();
    u16rect_from_layout!(layout)
  }

  pub fn visible(&self) -> bool {
    let rel = self.relationship.borrow();
    rel.style(self.id).unwrap().display == taffy::Display::None
  }
}
