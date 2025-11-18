//! The tree node of internal tree.

use crate::ui::tree::TaffyTreeWk;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering;
use taffy::Style;
use taffy::TaffyResult;
use taffy::prelude::TaffyMaxContent;

pub type LayoutNodeId = taffy::NodeId;
pub type TreeNodeId = i32;

pub trait Inodeable: Sized + Clone + std::fmt::Debug {
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

#[derive(Debug, Clone)]
/// The internal tree node, it's both a container for the widgets and common attributes.
pub struct InodeBase {
  id: TreeNodeId,
  loid: LayoutNodeId,
  lotree: TaffyTreeWk,
}

impl InodeBase {
  pub fn new(lotree: TaffyTreeWk, style: Style) -> TaffyResult<Self> {
    let lo = lotree.upgrade().unwrap();
    let mut lo = lo.borrow_mut();
    let loid = lo.new_leaf(style)?;
    lo.compute_layout(loid, taffy::Size::MAX_CONTENT)?;
    Ok(Self {
      id: next_node_id(),
      loid,
      lotree,
    })
  }

  pub fn id(&self) -> TreeNodeId {
    self.id
  }

  pub fn loid(&self) -> LayoutNodeId {
    self.loid
  }

  pub fn lotree(&self) -> TaffyTreeWk {
    self.lotree.clone()
  }
}
