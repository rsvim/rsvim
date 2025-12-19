//! The node structure of the internal tree.

use crate::flags_impl;
use crate::prelude::*;
use crate::ui::tree::TreeNodeId;
use std::fmt::Debug;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering;

pub trait Inodeable: Sized + Clone + Debug {
  fn id(&self) -> TreeNodeId;

  fn shape(&self) -> &IRect;

  fn set_shape(&mut self, shape: IRect);

  fn actual_shape(&self) -> &U16Rect;

  fn set_actual_shape(&mut self, actual_shape: U16Rect);

  fn zindex(&self) -> usize;

  fn set_zindex(&mut self, value: usize);

  fn enabled(&self) -> bool;

  fn set_enabled(&mut self, value: bool);
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

      fn set_shape(&mut self, shape: IRect) {
        self.$base_name.set_shape(shape);
      }

      fn actual_shape(&self) -> &U16Rect {
        self.$base_name.actual_shape()
      }

      fn set_actual_shape(&mut self, actual_shape: U16Rect) {
        self.$base_name.set_actual_shape(actual_shape)
      }

      fn zindex(&self) -> usize {
        self.$base_name.zindex()
      }

      fn set_zindex(&mut self, value: usize) {
        self.$base_name.set_zindex(value);
      }

      fn enabled(&self) -> bool {
        self.$base_name.enabled()
      }

      fn set_enabled(&mut self, value: bool) {
        self.$base_name.set_enabled(value);
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

      fn set_shape(&mut self, shape: IRect) {
        self
          .$base_name
          .node_mut(self.$base_name.root_id())
          .unwrap()
          .set_shape(shape);
      }

      fn actual_shape(&self) -> &U16Rect {
        self
          .$base_name
          .node(self.$base_name.root_id())
          .unwrap()
          .actual_shape()
      }

      fn set_actual_shape(&mut self, actual_shape: U16Rect) {
        self
          .$base_name
          .node_mut(self.$base_name.root_id())
          .unwrap()
          .set_actual_shape(actual_shape);
      }

      fn zindex(&self) -> usize {
        self
          .$base_name
          .node(self.$base_name.root_id())
          .unwrap()
          .zindex()
      }

      fn set_zindex(&mut self, value: usize) {
        self
          .$base_name
          .node_mut(self.$base_name.root_id())
          .unwrap()
          .set_zindex(value);
      }

      fn enabled(&self) -> bool {
        self
          .$base_name
          .node(self.$base_name.root_id())
          .unwrap()
          .enabled()
      }

      fn set_enabled(&mut self, value: bool) {
        self
          .$base_name
          .node_mut(self.$base_name.root_id())
          .unwrap()
          .set_enabled(value);
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

      fn set_shape(&mut self, shape: IRect) {
        match self {
          $(
            $enum::$variant(e) => e.set_shape(shape),
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

      fn set_actual_shape(&mut self, actual_shape: U16Rect) {
        match self {
          $(
            $enum::$variant(e) => e.set_actual_shape(actual_shape),
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

      fn set_zindex(&mut self, value: usize) {
        match self {
          $(
            $enum::$variant(e) => e.set_zindex(value),
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

      fn set_enabled(&mut self, value: bool) {
        match self {
          $(
            $enum::$variant(e) => e.set_enabled(value),
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

flags_impl!(Flags, u8, ENABLED);

pub const DEFAULT_ZINDEX: usize = 0;
pub const DEFAULT_ENABLED: bool = true;

// enabled=true
const FLAGS: Flags = Flags::all();

#[derive(Debug, Clone, Copy)]
/// The internal tree node, it's both a container for the widgets and common attributes.
pub struct InodeBase {
  id: TreeNodeId,
  shape: IRect,
  actual_shape: U16Rect,
  zindex: usize,
  // enabled
  flags: Flags,
}

impl InodeBase {
  pub fn new(shape: IRect) -> Self {
    let actual_shape = rect_as!(shape, u16);
    InodeBase {
      id: next_node_id(),
      shape,
      actual_shape,
      zindex: DEFAULT_ZINDEX,
      flags: FLAGS,
    }
  }

  pub fn id(&self) -> TreeNodeId {
    self.id
  }

  pub fn shape(&self) -> &IRect {
    &self.shape
  }

  pub fn set_shape(&mut self, shape: IRect) {
    self.shape = shape;
  }

  pub fn actual_shape(&self) -> &U16Rect {
    &self.actual_shape
  }

  pub fn set_actual_shape(&mut self, actual_shape: U16Rect) {
    self.actual_shape = actual_shape;
  }

  pub fn zindex(&self) -> usize {
    self.zindex
  }

  pub fn set_zindex(&mut self, zindex: usize) {
    self.zindex = zindex;
  }

  pub fn enabled(&self) -> bool {
    self.flags.contains(Flags::ENABLED)
  }

  pub fn set_enabled(&mut self, value: bool) {
    self.flags.set(Flags::ENABLED, value);
  }
}
