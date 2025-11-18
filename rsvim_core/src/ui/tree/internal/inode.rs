//! The tree node of internal tree.

use crate::flags_impl;
use crate::prelude::*;
use crate::ui::tree::TaffyTreeWk;
use std::fmt::Debug;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering;

pub type LayoutNodeId = taffy::NodeId;
pub type TreeNodeId = i32;

pub trait Inodeable: Sized + Clone + Debug {
  fn id(&self) -> TreeNodeId;

  fn loid(&self) -> LayoutNodeId;

  fn lotree(&self) -> TaffyTreeWk;
}

/// Generate getter/setter for `Inode`.
#[macro_export]
macro_rules! inode_impl {
  ($struct_name:ty,$base_name:ident) => {
    impl Inodeable for $struct_name {
      fn id(&self) -> TreeNodeId {
        self.$base_name.id()
      }

      fn loid(&self) -> LayoutNodeId {
        self.$base_name.loid()
      }

      fn lotree(&self) -> TaffyTreeWk {
        self.$base_name.lotree()
      }
    }
  };
}

/// Generate enum dispatcher for `Inode`.
#[macro_export]
macro_rules! inode_enum_dispatcher {
  ($enum:ident, $($variant:tt),*) => {
    impl Inodeable for $enum {
      fn id(&self) -> TreeNodeId {
        match self {
          $(
            $enum::$variant(e) => e.id(),
          )*
        }
      }

      fn loid(&self) -> LayoutNodeId {
        match self {
          $(
            $enum::$variant(e) => e.loid(),
          )*
        }
      }

      fn lotree(&self) -> TaffyTreeWk {
        match self {
          $(
            $enum::$variant(e) => e.lotree(),
          )*
        }
      }
    }
  }
}

/// Next unique UI widget ID.
///
/// NOTE: Start from 100001, so be different from buffer ID.
pub fn next_node_id() -> TreeNodeId {
  static VALUE: AtomicI32 = AtomicI32::new(100001);
  VALUE.fetch_add(1, Ordering::Relaxed)
}

#[derive(Debug, Clone, Copy)]
/// The internal tree node, it's both a container for the widgets and common attributes.
pub struct InodeBase {
  id: TreeNodeId,
  loid: LayoutNodeId,
  lotree: TaffyTreeWk,
}

impl InodeBase {
  pub fn new(lotree: TaffyTreeWk) -> Self {
    let actual_shape = rect_as!(shape, u16);
    InodeBase {
      id: next_node_id(),
      depth: 0,
      shape,
      actual_shape,
      zindex: 0,
      flags: FLAGS,
    }
  }

  pub fn id(&self) -> TreeNodeId {
    self.id
  }

  pub fn depth(&self) -> usize {
    self.depth
  }

  pub fn set_depth(&mut self, depth: usize) {
    self.depth = depth;
  }

  pub fn zindex(&self) -> usize {
    self.zindex
  }

  pub fn set_zindex(&mut self, zindex: usize) {
    self.zindex = zindex;
  }

  pub fn shape(&self) -> &IRect {
    &self.shape
  }

  pub fn set_shape(&mut self, shape: &IRect) {
    self.shape = *shape;
  }

  pub fn actual_shape(&self) -> &U16Rect {
    &self.actual_shape
  }

  pub fn set_actual_shape(&mut self, actual_shape: &U16Rect) {
    self.actual_shape = *actual_shape;
  }

  pub fn enabled(&self) -> bool {
    self.flags.contains(Flags::ENABLED)
  }

  pub fn set_enabled(&mut self, value: bool) {
    self.flags.set(Flags::ENABLED, value);
  }

  pub fn visible(&self) -> bool {
    self.flags.contains(Flags::VISIBLE)
  }

  pub fn set_visible(&mut self, value: bool) {
    self.flags.set(Flags::VISIBLE, value);
  }
}
