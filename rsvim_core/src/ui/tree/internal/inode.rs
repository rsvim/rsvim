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
/// We use [taffy] library to maintain the node relationship
/// (e.g. parent and children) and layout calculation, each node will hold a
/// weak pointer of [TaffyTree](taffy::TaffyTree), when a pair of parent-child
/// relationship is changed, a node position/size is changed, a node is
/// created/removed, we are actually just calling taffy's API to help us
/// complete the work, and calculate the newest layout, and render all the
/// nodes to terminal with newest layout.
///
/// All APIs of this trait, with `layout_` prefix, are actually just wrappers
/// on TaffyTree APIs, except the `layout_id` API.
pub trait Inodeable: Sized + Clone + Debug {
  fn id(&self) -> TreeNodeId;

  fn layout_id(&self) -> LayoutNodeId;

  fn layout(&self) -> TaffyTreeWk;

  /// [TaffyTree::add_child]
  ///
  /// Add this node as a child to parent node.
  fn layout_add(&mut self, parent_layout_id: LayoutNodeId);

  /// [TaffyTree::insert_child_at_index]
  ///
  /// Add this node as a child to parent node, but insert at provide
  /// `child_index`.
  fn layout_insert_at_index(
    &mut self,
    parent_layout_id: LayoutNodeId,
    child_index: usize,
  );

  /// [TaffyTree::remove_child]
  fn layout_remove(&mut self, parent_layout_id: LayoutNodeId);

  /// [TaffyTree::remove_child]
  fn layout_remove(&mut self, parent_layout_id: LayoutNodeId);

  /// Get parent layout ID.
  ///
  /// It returns parent layout ID if this node is attached to a parent node,
  /// unless this node is the root node. Otherwise it returns `None` to
  /// indicates this node is detached or it is the root node itself.
  fn layout_parent(&self) -> Option<LayoutNodeId>;

  /// Insert this node to its parent layout ID.
  fn attach(&mut self, parent_layout_id: LayoutNodeId);

  /// Remove this node from its parent layout ID.
  fn detach(&mut self, parent_layout_id: LayoutNodeId);
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

      fn enabled(&self) -> bool {
        self.$base.enabled()
      }

      fn set_enabled(&mut self, enabled: bool) {
        self.$base.set_enabled(enabled);
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

// enabled=true
const FLAGS: Flags = Flags::all();

#[derive(Debug, Clone, Copy)]
/// The internal tree node, it's both a container for the widgets and common attributes.
pub struct InodeBase {
  id: TreeNodeId,
  layout_id: LayoutNodeId,
  // enabled
  flags: Flags,
}

impl InodeBase {
  pub fn new(
    layout: &mut TaffyTree,
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
