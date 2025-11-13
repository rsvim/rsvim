//! The node structure of the internal tree.

use crate::flags_impl;
use crate::prelude::*;
use crate::ui::tree::TaffyTreeWk;
use std::fmt::Debug;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering;
use taffy::Style;
use taffy::TaffyResult;
use taffy::TaffyTree;

pub type LayoutNodeId = taffy::NodeId;
pub type TreeNodeId = i32;

/// Whole TUI is a tree structure, each node on the tree is a UI widget (e.g.
/// rectangle), and renders itself onto the terminal.
///
/// We use [taffy] library to maintain the relationships between each parent
/// and children nodes, and also the layout algorithms. Each node holds a weak
/// pointer to [TaffyTree], when the layout is changed, we just call taffy's
/// API to help us update node relationships and update layout, then render all
/// the nodes with newest layout. Here are some examples about layout changes:
///
/// All APIs with `layout_` prefix in this trait, are just wrappers on
/// [TaffyTree], except the `layout_id` API.
pub trait Inodeable: Sized + Clone + Debug {
  fn id(&self) -> TreeNodeId;

  fn layout_id(&self) -> LayoutNodeId;

  /// Get [TaffyTree] weak pointer.
  fn layout_tree(&self) -> TaffyTreeWk;
}

/// Generate getter/setter for `Inode`.
#[macro_export]
macro_rules! inode_impl {
  ($name:ty,$base:ident) => {
    impl Inodeable for $name {
      fn id(&self) -> TreeNodeId {
        self.$base.id()
      }

      fn layout_id(&self) -> LayoutNodeId {
        self.$base.layout_id()
      }

      fn layout_tree(&self) -> TaffyTreeWk {
        self.$base.layout_tree()
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

      fn layout_id(&self) -> LayoutNodeId {
        match self {
          $(
            $enum::$variant(e) => e.layout_id(),
          )*
        }
      }

      fn layout_tree(&self) -> TaffyTreeWk {
        match self {
          $(
            $enum::$variant(e) => e.layout_tree(),
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
  layout_id: LayoutNodeId,
  layout_tree: TaffyTreeWk,
}

impl InodeBase {
  pub fn new(
    layout_tree: TaffyTreeWk,
    parent_layout_id: LayoutNodeId,
    style: Style,
  ) -> TaffyResult<Self> {
    match layout.new_leaf(style) {
      Ok(layout_id) => {
        layout.add_child(parent_layout_id, layout_id).unwrap();
        Ok(InodeBase {
          id: next_node_id(),
          layout_id,
          flags: FLAGS,
        })
      }
      Err(e) => Err(e),
    }
  }

  pub fn id(&self) -> TreeNodeId {
    self.id
  }

  pub fn layout_node_id(&self) -> LayoutNodeId {
    self.layout_id
  }

  pub fn enabled(&self) -> bool {
    self.flags.contains(Flags::ENABLED)
  }

  pub fn set_enabled(&mut self, value: bool) {
    self.flags.set(Flags::ENABLED, value);
  }
}
