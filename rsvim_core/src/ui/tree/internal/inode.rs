//! The node structure of the internal tree.

use crate::flags_impl;
use crate::prelude::*;
use std::fmt::Debug;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering;
use taffy::Style;
use taffy::TaffyResult;
use taffy::TaffyTree;

pub type LayoutNodeId = taffy::NodeId;
pub type TreeNodeId = i32;

/// This trait maintains the UI tree relationship and layout information
/// among all the nodes. The whole TUI is a tree structure, and each node on
/// the tree is a rectangle, and finally renders itself onto the terminal.
///
/// Since we're using [taffy](taffy) crate to maintain the node relationship
/// (e.g. parent and children) and layout calculation, each node will hold a
/// weak pointer of [TaffyTree](taffy::TaffyTree), when a pair of parent-child 
/// relationship is changed, a node position/size is changed, a node is
/// created/removed, we are actually just calling taffy's API to help us
/// complete the work, and calculate the newest layout, and render all the
/// nodes to terminal with newest layout.
///
/// All APIs of this trait, with `layout_` prefix are actually the TaffyTree
/// APIs, except `layout_id` API.
pub trait Inodeable: Sized + Clone + Debug {
  fn id(&self) -> TreeNodeId;

  fn layout_id(&self) -> LayoutNodeId;

  fn layout_add_child()

  /// Whether this node is attached to a parent node.
  ///
  /// By default a node is attached to a parent node when it is created, and
  /// all nodes are in the UI tree with the same root node. But you can detach
  /// it from its parent node to:
  /// 1. Hide/disable this node temporarily. This is useful to switch the UI
  ///    widgets.
  /// 2. Move this node from a parent node to another, e.g. first detach it
  ///    from a parent, then attach it to another parent.
  /// 3. Remove this node permanently.
  ///
  /// NOTE: This method is equivalent to `parent_layout_id().is_some()`,
  /// because has a parent layout ID means it is attached to a parent node.
  fn is_attached(&self) -> bool {
    self.parent_layout_id().is_some()
  }

  /// Get parent layout ID.
  ///
  /// It returns parent layout ID if this node is attached to a parent node,
  /// unless this node is the root node. Otherwise it returns `None` to
  /// indicates this node is detached or it is the root node itself.
  fn parent_layout_id(&self) -> Option<LayoutNodeId>;

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
