//! The node structure of the internal tree.

use crate::prelude::*;
use crate::ui::tree::TreeNodeId;
use crate::ui::tree::internal::arena::TruncatePolicy;
use std::fmt::Debug;

pub trait Inodeable: Sized + Clone + Debug {
  fn id(&self) -> TreeNodeId;

  fn shape(&self) -> &IRect;

  fn actual_shape(&self) -> &U16Rect;

  fn zindex(&self) -> usize;

  fn enabled(&self) -> bool;

  fn truncate_policy(&self) -> TruncatePolicy;
}

/// Generate getter/setter for `Inode`.
#[macro_export]
macro_rules! inode_impl {
  ($struct_name:ty,$base_name:ident) => {
    impl Inodeable for $struct_name {
      fn id(&self) -> TreeNodeId {
        self.$base_name.id()
      }

      fn shape(&self) -> &IRect {
        self.$base_name.shape()
      }

      fn actual_shape(&self) -> &U16Rect {
        self.$base_name.actual_shape()
      }

      fn zindex(&self) -> usize {
        self.$base_name.zindex()
      }

      fn enabled(&self) -> bool {
        self.$base_name.enabled()
      }

      fn truncate_policy(&self) -> TruncatePolicy {
        self.$base_name.truncate_policy()
      }
    }
  };
}

/// Generate getter/setter for `Inode` with `Itree` base.
#[macro_export]
macro_rules! inode_itree_impl {
  ($struct_name:ty,$base_name:ident) => {
    impl Inodeable for $struct_name {
      fn id(&self) -> TreeNodeId {
        self.$base_name.root_id()
      }

      fn shape(&self) -> &IRect {
        self
          .$base_name
          .node(self.$base_name.root_id())
          .unwrap()
          .shape()
      }

      fn actual_shape(&self) -> &U16Rect {
        self
          .$base_name
          .node(self.$base_name.root_id())
          .unwrap()
          .actual_shape()
      }

      fn zindex(&self) -> usize {
        self
          .$base_name
          .node(self.$base_name.root_id())
          .unwrap()
          .zindex()
      }

      fn enabled(&self) -> bool {
        self
          .$base_name
          .node(self.$base_name.root_id())
          .unwrap()
          .enabled()
      }

      fn truncate_policy(&self) -> TruncatePolicy {
        self
          .$base_name
          .node_mut(self.$base_name.root_id())
          .unwrap()
          .truncate_policy()
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

      fn shape(&self) -> &IRect {
        match self {
          $(
            $enum::$variant(e) => e.shape(),
          )*
        }
      }


      fn actual_shape(&self) -> &U16Rect {
        match self {
          $(
            $enum::$variant(e) => e.actual_shape(),
          )*
        }
      }

      fn zindex(&self) -> usize {
        match self {
          $(
            $enum::$variant(e) => e.zindex(),
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

      fn truncate_policy(&self) -> TruncatePolicy {
        match self {
          $(
            $enum::$variant(e) => e.truncate_policy(),
          )*
        }
      }
    }
  }
}
