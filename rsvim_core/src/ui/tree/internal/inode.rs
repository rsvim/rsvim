//! The node structure of the internal tree.

use crate::ui::tree::TaffyTreeWk;
use std::fmt::Debug;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering;
use taffy::Layout;
use taffy::Style;
use taffy::TaffyResult;

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

  fn layout_tree(&self) -> TaffyTreeWk;

  fn style(&self) -> &Style;

  fn set_style(&mut self, style: Style);

  fn layout(&self) -> &Option<Layout>;

  fn set_layout(&mut self, layout: Option<Layout>);
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

      fn style(&self) -> &Style {
        self.$base.style()
      }

      fn set_style(&mut self, style: Style) {
        self.$base.set_style(style);
      }

      fn layout(&self) -> &Option<Layout> {
        self.$base.layout()
      }

      fn set_layout(&mut self, layout: Option<Layout>) {
        self.$base.set_layout(layout);
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

      fn style(&self) -> &Style {
        match self {
          $(
            $enum::$variant(e) => e.style(),
          )*
        }
      }

      fn set_style(&mut self, style: Style) {
        match self {
          $(
            $enum::$variant(e) => e.set_style(style),
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
  layout_id: LayoutNodeId,
  layout_tree: TaffyTreeWk,
  style: Style,
  layout: Layout,
}

impl InodeBase {
  pub fn new(layout_tree: TaffyTreeWk, style: Style) -> TaffyResult<Self> {
    let layout_tree1 = layout_tree.clone();
    let layout_tree = layout_tree.upgrade().unwrap();
    let mut layout_tree = layout_tree.borrow_mut();
    match layout_tree.new_leaf(style.clone()) {
      Ok(layout_id) => Ok(InodeBase {
        id: next_node_id(),
        layout_id,
        layout_tree: layout_tree1,
        style,
      }),
      Err(e) => Err(e),
    }
  }

  pub fn id(&self) -> TreeNodeId {
    self.id
  }

  pub fn layout_id(&self) -> LayoutNodeId {
    self.layout_id
  }

  pub fn style(&self) -> &Style {
    &self.style
  }

  pub fn set_style(&mut self, style: Style) {
    self.style = style;
  }
}
