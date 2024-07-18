//! Widget tree that manages all the widget components.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::hash::Hash;
use std::sync::{Arc, RwLock};

use crate::cart::{IPos, IRect, USize};
use crate::ui::widget::cursor::Cursor;
use crate::ui::widget::root::RootWidget;
use crate::ui::widget::window::Window;
use crate::ui::widget::Widget;

pub mod edge;
pub mod node;

pub type NodeId = usize;

pub enum Node {
  RootWidgetNode(RootWidget),
  CursorNode(Cursor),
  WindowNode(Window),
}

pub type NodePtr = Arc<RwLock<Node>>;

pub fn make_node_ptr(n: Node) -> Arc<RwLock<Node>> {
  Arc::new(RwLock::new(n))
}

impl PartialOrd for Node {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    self.id().partial_cmp(&other.id())
  }
}

impl PartialEq for Node {
  fn eq(&self, other: &Self) -> bool {
    self.id().eq(&other.id())
  }
}

macro_rules! define_widget_node_getter {
  ($getter_name:ident,$return_type_name:ty) => {
    fn $getter_name(&self) -> $return_type_name {
      match self {
        Self::RootWidgetNode(node) => node.$getter_name(),
        Self::CursorNode(node) => node.$getter_name(),
        Self::WindowNode(node) => node.$getter_name(),
      }
    }
  };
}

macro_rules! define_widget_node_setter {
  ($setter_name:ident,$value_type_name:ty) => {
    fn $setter_name(&mut self, value: $value_type_name) {
      match self {
        Self::RootWidgetNode(node) => node.$setter_name(value),
        Self::CursorNode(node) => node.$setter_name(value),
        Self::WindowNode(node) => node.$setter_name(value),
      }
    }
  };
}

impl Widget for Node {
  define_widget_node_getter!(id, NodeId);

  define_widget_node_getter!(rect, IRect);
  define_widget_node_setter!(set_rect, IRect);
  define_widget_node_getter!(pos, IPos);
  define_widget_node_setter!(set_pos, IPos);
  define_widget_node_getter!(size, USize);
  define_widget_node_setter!(set_size, USize);
  define_widget_node_getter!(zindex, usize);
  define_widget_node_setter!(set_zindex, usize);
  define_widget_node_getter!(visible, bool);
  define_widget_node_setter!(set_visible, bool);
  define_widget_node_getter!(enabled, bool);
  define_widget_node_setter!(set_enabled, bool);

  fn draw(&mut self) {
    match self {
      Self::RootWidgetNode(node) => node.draw(),
      Self::CursorNode(node) => node.draw(),
      Self::WindowNode(node) => node.draw(),
    }
  }
}

#[derive(Hash, Copy, Clone, PartialEq, Eq, Default)]
pub struct Edge {
  from: NodeId,
  to: NodeId,
}

impl Edge {
  pub fn hash_str(&self) -> String {
    let width = std::cmp::max(
      std::mem::size_of_val(&self.from),
      std::mem::size_of_val(&self.to),
    );
    format!("{:0<width$}{:0<width$}", self.from, self.to, width = width)
  }
}

impl PartialOrd for Edge {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    let h1 = self.hash_str();
    let h2 = other.hash_str();
    h1.partial_cmp(&h2)
  }
}

impl Ord for Edge {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    let h1 = self.hash_str();
    let h2 = other.hash_str();
    h1.cmp(&h2)
  }
}

/// Widget tree.
/// A widget tree contains only 1 root node, each node can have 0 or multiple nodes.
pub struct Tree {
  // A collection of all nodes, maps from node ID to node struct.
  nodes: BTreeMap<NodeId, NodePtr>,

  // A collection of all edges.
  edges: BTreeSet<Edge>,

  // Root node ID.
  root_id: Option<NodeId>,

  // Maps from parent ID to its children IDs.
  // Note: A parent can have multiple children.
  children_ids: BTreeMap<NodeId, HashSet<NodeId>>,

  // Maps from child ID to its parent ID.
  parent_ids: BTreeMap<NodeId, NodeId>,
}

impl Tree {
  pub fn new() -> Tree {
    Tree {
      nodes: BTreeMap::new(),
      edges: BTreeSet::new(),
      root_id: None,
      children_ids: BTreeMap::new(),
      parent_ids: BTreeMap::new(),
    }
  }

  /// Get node by its ID.
  ///
  /// Returns the node if exists, returns `None` if not.
  pub fn get_node(&self, id: NodeId) -> Option<NodePtr> {
    match self.nodes.get(&id) {
      Some(node) => Some(node.clone()),
      None => None,
    }
  }

  /// Get the root node ID.
  ///
  /// Returns the root node ID if exists, returns `None` if not.
  pub fn get_root_node(&self) -> Option<NodeId> {
    self.root_id
  }

  /// Insert root node with its ID.
  ///
  /// Returns the inserted node if succeeded, returns `None` if failed.
  ///
  /// # Panics
  ///
  /// Panics if there's already a root node.
  pub fn insert_root_node(&mut self, id: NodeId, node: NodePtr) -> Option<NodePtr> {
    assert!(self.root_id.is_none());
    self.root_id = Some(id);
    self.nodes.insert(id, node.clone())
  }

  /// Insert node with both its and its parent's ID.
  /// This operation also binds the connection between the inserted node and its parent.
  ///
  /// Returns the inserted node if succeeded, returns `None` if failed.
  pub fn insert_node(&mut self, id: NodeId, node: NodePtr, parent_id: NodeId) -> Option<NodePtr> {
    match self.children_ids.get_mut(&parent_id) {
      Some(children) => {
        children.insert(id);
      }
      None => {
        let mut init_ids = HashSet::new();
        init_ids.insert(id);
        self.children_ids.insert(parent_id, init_ids);
      }
    }
    self.parent_ids.insert(id, parent_id);
    self.edges.insert(Edge {
      from: parent_id,
      to: id,
    });
    self.nodes.insert(id, node.clone())
  }

  /// Remove node by its ID.
  ///
  /// Returns the removed node if it exists, returns `None` if not.
  ///
  /// This operation also removes the connection between the node and its parent (if any).
  /// This operation doesn't removes the connection between the node and its children (if any).
  pub fn remove_node(&mut self, id: NodeId) -> Option<NodePtr> {
    match self.nodes.remove(&id) {
      Some(node) => {
        if self.parent_ids.contains_key(&id) {
          assert!(self.root_id != Some(id));
          let parent_id = self.parent_ids.remove(&id).unwrap();
          assert!(self.children_ids.contains_key(&parent_id));
          let removed = self.children_ids.get_mut(&parent_id).unwrap().remove(&id);
          assert!(removed);
        } else {
          assert!(self.root_id == Some(id));
          self.root_id = None;
        }
        Some(node)
      }
      None => {
        assert!(!self.parent_ids.contains_key(&id) && self.root_id != Some(id));
        None
      }
    }
  }

  /// Get edge by the `from` node ID and the `to` node ID.
  ///
  /// Returns the edge if exists, returns `None` if not.
  pub fn get_edge(&self, from: NodeId, to: NodeId) -> Option<&Edge> {
    self.edges.get(&Edge { from, to })
  }

  pub fn get_children(&self, parent_id: NodeId) -> Option<&HashSet<NodeId>> {
    self.children_ids.get(&parent_id)
  }

  pub fn get_parent(&self, child_id: NodeId) -> Option<&NodeId> {
    self.parent_ids.get(&child_id)
  }
}
