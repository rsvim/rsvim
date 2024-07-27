//! The internal tree-structure of a widget tree, it implements the core logic of nodes, edges,
//! parents and children relationship, etc.

use geo::point;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

use crate::ui::tree::edge::Edge;
use crate::ui::tree::node::{make_node_ptr, Node, NodeId, NodePtr};
use crate::ui::widget::layout::root::RootLayout;
use crate::ui::widget::Widget;
use crate::uuid;

/// The internal tree-structure of a widget tree.
///
/// Note: The tree itself is also a node, or say, the root node is created along with the tree.
pub struct TreeBase {
  // Root {

  // Root node ID.
  root_node_id: NodeId,

  // Root node.
  root_node: NodePtr,

  // Root }

  // Node {

  // A collection of all nodes (except the root node).
  // Maps from node ID to node struct.
  nodes: BTreeMap<NodeId, NodePtr>,

  // Node }

  // Edge {

  // Maps "parent ID" => its "children IDs".
  children_ids: HashMap<NodeId, HashSet<NodeId>>,

  // Maps "child ID" => its "parent ID".
  parent_ids: HashMap<NodeId, NodeId>,
  // Edge }
}

#[derive(Copy, Clone)]
struct ParentChildPair {
  pub parent_id: NodeId,
  pub child_id: NodeId,
}

impl TreeBase {
  /// Make a widget tree.
  ///
  /// Note: The root node is created along with the tree.
  pub fn new() -> Self {
    let root_node = RootLayout::new();
    let root_node_id = root_node.id();
    TreeBase {
      root_node_id,
      root_node: make_node_ptr(Node::RootLayout(root_node)),
      nodes: BTreeMap::new(),
      children_ids: HashMap::new(),
      parent_ids: HashMap::new(),
    }
  }

  /// Whether the tree is empty (except the root node).
  pub fn is_empty(&self) -> bool {
    self.nodes.is_empty() && self.children_ids.is_empty() && self.parent_ids.is_empty()
  }

  /// Get root node ID.
  pub fn root_node_id(&self) -> &NodeId {
    &self.root_node_id
  }

  /// Get all nodes.
  pub fn nodes(&self) -> &BTreeMap<NodeId, NodePtr> {
    &self.nodes
  }

  /// Get all "parent" ID => "children" ID mappings.
  pub fn childrens(&self) -> &HashMap<NodeId, HashSet<NodeId>> {
    &self.children_ids
  }

  /// Get all "child" ID => "parent" ID mappings.
  pub fn parents(&self) -> &HashMap<NodeId, NodeId> {
    &self.parent_ids
  }

  /// Get a node by its ID.
  pub fn get(&self, id: NodeId) -> Option<&NodePtr> {
    self.nodes.get(&id)
  }

  /// Whether contains a node by its ID.
  pub fn contains(&self, id: NodeId) -> bool {
    self.nodes.contains_key(&id) && self.children_ids.contains_key(&id)
  }

  /// Insert a node by its ID and parent's ID.
  /// This operation also binds the connection between the node and its parent.
  ///
  /// Returns inserted node if succeed, returns `None` if failed.
  ///
  /// Fails if:
  /// 1. The node already exists.
  /// 2. The parent node doesn't exist.
  pub fn insert(&mut self, id: NodeId, node: NodePtr, parent_id: NodeId) -> Option<NodePtr> {
    // Fails
    if self.contains(&id)
      || !self.contains(parent_id)
      || !self.children_ids.contains_key(&parent_id)
    {
      return None;
    }

    // Maps from parent ID to its child ID.
    self.children_ids.get_mut(&parent_id).unwrap().insert(id);
    // Maps from the child ID to its parent ID.
    self.parent_ids.insert(id, parent_id);
    // Initialize its children collection.
    self.children_ids.insert(id, HashMap::new());
    // Insert ID and node.
    self.nodes.insert(id, node)
  }

  /// Remove a node by its ID.
  ///
  /// Returns the removed node and the nested sub-tree if succeed, returns `None` if failed.
  ///
  /// Fails if:
  /// 1. The node doesn't exist.
  pub fn remove(&mut self, id: NodeId) -> Option<TreeBase> {
    // Fails
    if !self.contains(&id) || !self.parents.contains_key(&id) {
      return None;
    }

    let parent_id = self.get_parent(id).unwrap();
    self.children_ids.get_mut(parent_id).unwrap().remove(&id);

    let subtree_root_node_id = id;
    let subtree_root_node = self.nodes.get(&subtree_root_node_id).unwrap();

    let mut subtree = TreeBase {
      root_node_id: subtree_root_node_id,
      root_node: subtree_root_node,
      nodes: BTreeMap::new(),
      children_ids: HashMap::new(),
      parent_ids: HashMap::new(),
    };

    let mut q: VecDeque<ParentChildPair> = self
      .get_children(subtree_root_node_id)
      .unwrap()
      .iter()
      .map(|child_id| ParentChildPair {
        parent_id: subtree_root_node_id,
        child_id,
      })
      .collect();
    while let Some(parent_child_pair) = q.pop_front() {
      let pid = parent_child_pair.parent_id;
      let eid = parent_child_pair.child_id;
      let e = self.nodes.remove(&eid).unwrap();
      subtree.insert(eid, e, pid);
      self.parent_ids.remove(&eid);
      if let Some(e_children) = self.children_ids.get(&eid) {
        for cid in e_children.iter() {
          q.push_back(ParentChildPair {
            parent_id: eid,
            child_id: cid,
          });
        }
      }
    }

    Some(subtree)
  }

  /// Get all children IDs by the parent's ID.
  pub fn get_children(&self, id: NodeId) -> Option<&HashSet<NodeId>> {
    self.children_ids.get(&id)
  }

  /// Get parent ID by the child ID.
  pub fn get_parent(&self, id: NodeId) -> Option<&NodeId> {
    self.parent_ids.get(&id)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::cart::{IPos, IRect, ISize, Size, U16Pos, U16Rect, U16Size};
  use crate::geo_size_as;
  use crate::test::log::init as test_log_init;
  use crate::ui::term::{make_terminal_ptr, Terminal};
  use crate::ui::widget::{Cursor, RootLayout, Widget, Window};
  use std::sync::Once;
  use tracing::info;

  static INIT: Once = Once::new();

  #[test]
  fn tree_new() {
    INIT.call_once(|| {
      test_log_init();
    });
  }
}
