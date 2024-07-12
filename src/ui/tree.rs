//! Widget Tree that manages all the widget components.

use std::{
  collections::{BTreeMap, BTreeSet, HashSet},
  os::unix::process::parent_id,
};

use crate::ui::widget::Widget;

pub type NodeId = usize;

pub enum Node {}

pub struct NodeBase {}

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
  nodes: BTreeMap<NodeId, Node>,

  // A collection of all edges.
  edges: HashSet<Edge>,

  // Maps from parent ID to its children IDs.
  // Note: A parent can have multiple children.
  children_ids: BTreeMap<NodeId, Vec<NodeId>>,

  // Maps from child ID to its parent ID.
  parent_ids: BTreeMap<NodeId, NodeId>,
}

impl Tree {
  pub fn new() -> Tree {
    Tree {
      nodes: BTreeMap::new(),
      edges: HashSet::new(),
      children_ids: BTreeMap::new(),
      parent_ids: BTreeMap::new(),
    }
  }

  pub fn get_node(&self, id: NodeId) -> Option<&Node> {
    self.nodes.get(&id)
  }

  pub fn get_edge(&self, from: NodeId, to: NodeId) -> Option<&Edge> {
    self.edges.get(&Edge { from, to })
  }

  pub fn get_children(&self, parent_id: NodeId) -> Option<&Vec<NodeId>> {
    self.children_ids.get(&parent_id)
  }

  pub fn get_parent(&self, child_id: NodeId) -> Option<&NodeId> {
    self.parent_ids.get(&child_id)
  }
}
