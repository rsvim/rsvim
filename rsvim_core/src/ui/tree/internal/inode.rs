//! The node structure of the internal tree.

use crate::prelude::*;
use crate::struct_id_impl;
use crate::ui::tree::internal::context::TreeContextWk;
use crate::ui::tree::internal::context::TruncatePolicy;
use std::fmt::Debug;

// pub type TreeNodeId = i32;

struct_id_impl!(TreeNodeId, i32);

pub trait Inodify: Sized + Clone + Debug {
  fn id(&self) -> TreeNodeId;

  fn shape(&self) -> IRect;

  fn actual_shape(&self) -> U16Rect;

  fn zindex(&self) -> usize;

  fn enabled(&self) -> bool;

  fn truncate_policy(&self) -> TruncatePolicy;
}

/// Generate getter/setter for `Inodify`.
#[macro_export]
macro_rules! inodify_impl {
  ($name:ty) => {
    impl Inodify for $name {
      fn id(&self) -> TreeNodeId {
        self.__node.id()
      }

      fn shape(&self) -> IRect {
        self.__node.shape()
      }

      fn actual_shape(&self) -> U16Rect {
        self.__node.actual_shape()
      }

      fn zindex(&self) -> usize {
        self.__node.zindex()
      }

      fn enabled(&self) -> bool {
        self.__node.enabled()
      }

      fn truncate_policy(&self) -> TruncatePolicy {
        self.__node.truncate_policy()
      }
    }
  };
}

/// Generate enum dispatcher for `Inodify`.
#[macro_export]
macro_rules! inodify_enum_impl {
  ($enum:ident, $($variant:tt),*) => {
    impl Inodify for $enum {
      fn id(&self) -> TreeNodeId {
        match self {
          $(
            $enum::$variant(e) => e.id(),
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

#[derive(Debug, Clone)]
pub struct InodeBase {
  id: TreeNodeId,
  ctx: TreeContextWk,
}

impl InodeBase {
  pub fn new(id: TreeNodeId, ctx: TreeContextWk) -> Self {
    Self { id, ctx }
  }

  pub fn context(&self) -> TreeContextWk {
    self.ctx.clone()
  }
}

impl Inodify for InodeBase {
  fn id(&self) -> TreeNodeId {
    self.id
  }

  fn shape(&self) -> IRect {
    self
      .ctx
      .upgrade()
      .unwrap()
      .borrow()
      .shape(self.id)
      .copied()
      .unwrap()
  }

  fn actual_shape(&self) -> U16Rect {
    self
      .ctx
      .upgrade()
      .unwrap()
      .borrow()
      .actual_shape(self.id)
      .copied()
      .unwrap()
  }

  fn zindex(&self) -> usize {
    self
      .ctx
      .upgrade()
      .unwrap()
      .borrow()
      .zindex(self.id)
      .unwrap()
  }

  fn enabled(&self) -> bool {
    self
      .ctx
      .upgrade()
      .unwrap()
      .borrow()
      .enabled(self.id)
      .unwrap()
  }

  fn truncate_policy(&self) -> TruncatePolicy {
    self
      .ctx
      .upgrade()
      .unwrap()
      .borrow()
      .truncate_policy(self.id)
      .unwrap()
  }
}
