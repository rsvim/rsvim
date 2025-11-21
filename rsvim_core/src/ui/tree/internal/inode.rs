//! The node structure of the internal tree.

use crate::flags_impl;
use crate::prelude::*;
use crate::ui::tree::IrelationshipRc;
use crate::ui::tree::TreeNodeId;

pub trait Inodeable: Sized + Clone + std::fmt::Debug {
  fn id(&self) -> TreeNodeId;

  fn relationship(&self) -> IrelationshipRc;

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

      fn relationship(&self) -> IrelationshipRc {
        self.$base_name.relationship()
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

flags_impl!(Flags, u8, ENABLED, VISIBLE);

// enabled=true
// visible=true
const FLAGS: Flags = Flags::all();

#[derive(Debug, Clone)]
/// The internal tree node, it's both a container for the widgets and common attributes.
pub struct InodeBase {
  relationship: IrelationshipRc,
  id: TreeNodeId,
  actual_shape: U16Rect,
  // enabled
  // visible
  flags: Flags,
}

impl InodeBase {
  pub fn new(
    relationship: IrelationshipRc,
    id: TreeNodeId,
    actual_shape: U16Rect,
  ) -> Self {
    InodeBase {
      relationship,
      id,
      actual_shape,
      flags: FLAGS,
    }
  }

  pub fn id(&self) -> TreeNodeId {
    self.id
  }

  pub fn relationship(&self) -> IrelationshipRc {
    self.relationship.clone()
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
