//! Widget Tree that manages all the widget components.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use crate::ui::widget::Widget;

pub type NodeId = usize;

pub enum Node {}

impl PartialOrd for Node {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    None
  }
}

impl PartialEq for Node {
  fn eq(&self, other: &Self) -> bool {
    false
  }
}

impl Widget for Node {}

#[derive(Hash, Copy, Clone, PartialEq, Eq, Default)]
pub struct Edge {
  from: NodeId,
  to: NodeId,
}

pub struct Tree {
  // A collection of all nodes, maps from node ID to node struct.
  nodes: BTreeMap<NodeId, NodePtr>,

  // A collection of all edges.
  edges: HashSet<Edge>,

  // Root node ID.
  root_id: Option<NodeId>,

  // Maps from parent ID to its children IDs.
  // Note: A parent can have multiple children.
  children_ids: BTreeMap<NodeId, HashSet<NodeId>>,

  // Maps from child ID to its parent ID.
  parent_ids: BTreeMap<NodeId, NodeId>,
}

type NodePtr = Arc<RwLock<Node>>;

impl Tree {
  pub fn new() -> Tree {
    Tree {
      nodes: BTreeMap::new(),
      edges: HashSet::new(),
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

  pub fn get_edge(&self, from: NodeId, to: NodeId) -> Option<&Edge> {
    self.edges.get(&Edge { from, to })
  }

  pub fn get_root(&self) -> Option<NodeId> {
    self.root_id
  }

  pub fn set_root(&mut self, root_id: Option<NodeId>) -> Option<NodeId> {
    let old_root = self.root_id;
    self.root_id = root_id;
    old_root
  }

  pub fn get_children(&self, parent_id: NodeId) -> Option<&Vec<NodeId>> {
    self.children_ids.get(&parent_id)
  }

  pub fn get_parent(&self, child_id: NodeId) -> Option<&NodeId> {
    self.parent_ids.get(&child_id)
  }
}
