//! The node structure of the internal tree.

use crate::geo_rect_as;
use crate::prelude::*;
use bitflags::bitflags;
use std::fmt::Debug;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering;

pub type TreeNodeId = i32;

pub trait Inodeable: Sized + Clone + Debug {
  fn id(&self) -> TreeNodeId;

  fn depth(&self) -> usize;

  fn set_depth(&mut self, depth: usize);

  fn zindex(&self) -> usize;

  fn set_zindex(&mut self, zindex: usize);

  fn shape(&self) -> &IRect;

  fn set_shape(&mut self, shape: &IRect);

  fn actual_shape(&self) -> &U16Rect;

  fn set_actual_shape(&mut self, actual_shape: &U16Rect);

  fn enabled(&self) -> bool;

  fn set_enabled(&mut self, enabled: bool);

  fn visible(&self) -> bool;

  fn set_visible(&mut self, visible: bool);
}

/// Generate getter/setter for `Inode`.
#[macro_export]
macro_rules! inode_impl {
  ($struct_name:ty,$base_name:ident) => {
    impl Inodeable for $struct_name {
      fn id(&self) -> TreeNodeId {
        self.$base_name.id()
      }

      fn depth(&self) -> usize {
        self.$base_name.depth()
      }

      fn set_depth(&mut self, depth: usize) {
        self.$base_name.set_depth(depth);
      }

      fn zindex(&self) -> usize {
        self.$base_name.zindex()
      }

      fn set_zindex(&mut self, zindex: usize) {
        self.$base_name.set_zindex(zindex);
      }

      fn shape(&self) -> &IRect {
        self.$base_name.shape()
      }

      fn set_shape(&mut self, shape: &IRect) {
        self.$base_name.set_shape(shape);
      }

      fn actual_shape(&self) -> &U16Rect {
        self.$base_name.actual_shape()
      }

      fn set_actual_shape(&mut self, actual_shape: &U16Rect) {
        self.$base_name.set_actual_shape(actual_shape)
      }

      fn enabled(&self) -> bool {
        self.$base_name.enabled()
      }

      fn set_enabled(&mut self, enabled: bool) {
        self.$base_name.set_enabled(enabled);
      }

      fn visible(&self) -> bool {
        self.$base_name.visible()
      }

      fn set_visible(&mut self, visible: bool) {
        self.$base_name.set_visible(visible);
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

      fn depth(&self) -> usize {
        self
          .$base_name
          .node(self.$base_name.root_id())
          .unwrap()
          .depth()
      }

      fn set_depth(&mut self, depth: usize) {
        self
          .$base_name
          .node_mut(self.$base_name.root_id())
          .unwrap()
          .set_depth(depth);
      }

      fn zindex(&self) -> usize {
        self
          .$base_name
          .node(self.$base_name.root_id())
          .unwrap()
          .zindex()
      }

      fn set_zindex(&mut self, zindex: usize) {
        self
          .$base_name
          .node_mut(self.$base_name.root_id())
          .unwrap()
          .set_zindex(zindex);
      }

      fn shape(&self) -> &IRect {
        self
          .$base_name
          .node(self.$base_name.root_id())
          .unwrap()
          .shape()
      }

      fn set_shape(&mut self, shape: &IRect) {
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

      fn set_actual_shape(&mut self, actual_shape: &U16Rect) {
        self
          .$base_name
          .node_mut(self.$base_name.root_id())
          .unwrap()
          .set_actual_shape(actual_shape);
      }

      fn enabled(&self) -> bool {
        self
          .$base_name
          .node(self.$base_name.root_id())
          .unwrap()
          .enabled()
      }

      fn set_enabled(&mut self, enabled: bool) {
        self
          .$base_name
          .node_mut(self.$base_name.root_id())
          .unwrap()
          .set_enabled(enabled);
      }

      fn visible(&self) -> bool {
        self
          .$base_name
          .node(self.$base_name.root_id())
          .unwrap()
          .visible()
      }

      fn set_visible(&mut self, visible: bool) {
        self
          .$base_name
          .node_mut(self.$base_name.root_id())
          .unwrap()
          .set_visible(visible);
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

      fn depth(&self) -> usize {
        match self {
          $(
            $enum::$variant(e) => e.depth(),
          )*
        }
      }

      fn set_depth(&mut self, depth: usize) {
        match self {
          $(
            $enum::$variant(e) => e.set_depth(depth),
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

      fn set_zindex(&mut self, zindex: usize) {
        match self {
          $(
            $enum::$variant(e) => e.set_zindex(zindex),
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

      fn set_shape(&mut self, shape: &IRect) {
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

      fn set_actual_shape(&mut self, actual_shape: &U16Rect) {
        match self {
          $(
            $enum::$variant(e) => e.set_actual_shape(actual_shape),
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

      fn set_enabled(&mut self, enabled: bool) {
        match self {
          $(
            $enum::$variant(e) => e.set_enabled(enabled),
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

      fn set_visible(&mut self, visible: bool) {
        match self {
          $(
            $enum::$variant(e) => e.set_visible(visible),
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

#[allow(dead_code)]
const ENABLED: bool = true;
#[allow(dead_code)]
const VISIBLE: bool = true;

bitflags! {
  #[derive(Copy, Clone)]
  struct BaseFlags: u8 {
    const ENABLED = 1;
    const VISIBLE = 1 << 1;
  }
}

impl Debug for BaseFlags {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Flags")
      .field("bits", &format!("{:b}", self.bits()))
      .finish()
  }
}

// enabled=true
// visible=true
const BASE_FLAGS: BaseFlags = BaseFlags::all();

#[derive(Debug, Clone, Copy)]
/// The internal tree node, it's both a container for the widgets and common attributes.
pub struct InodeBase {
  id: TreeNodeId,
  depth: usize,
  shape: IRect,
  actual_shape: U16Rect,
  zindex: usize,

  // enabled
  // visible
  flags: BaseFlags,
}

impl InodeBase {
  pub fn new(shape: IRect) -> Self {
    let actual_shape = geo_rect_as!(shape, u16);
    InodeBase {
      id: next_node_id(),
      depth: 0,
      shape,
      actual_shape,
      zindex: 0,
      flags: BASE_FLAGS,
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
    self.flags.contains(BaseFlags::ENABLED)
  }

  pub fn set_enabled(&mut self, enabled: bool) {
    if enabled {
      self.flags.insert(BaseFlags::ENABLED);
    } else {
      self.flags.remove(BaseFlags::ENABLED);
    }
  }

  pub fn visible(&self) -> bool {
    self.flags.contains(BaseFlags::VISIBLE)
  }

  pub fn set_visible(&mut self, visible: bool) {
    if visible {
      self.flags.insert(BaseFlags::VISIBLE);
    } else {
      self.flags.remove(BaseFlags::VISIBLE);
    }
  }
}
