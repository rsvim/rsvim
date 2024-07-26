//! The base of a widget tree that manages nodes, edges, parents and children relationship, etc.

use geo::point;
use std::collections::VecDeque;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::ui::tree::edge::Edge;
use crate::ui::tree::node::{make_node_ptr, NodeAttribute, NodeId, NodePtr};
use crate::ui::widget::layout::root::RootLayout;
use crate::ui::widget::Widget;
use crate::uuid;

use super::Node;

/// The base of a widget tree.
///
/// Note: The tree itself is also a node, i.e. the root node exists along with the tree.
pub struct TreeBase {
  // Root {

  // The root node is a dummy node, and is always created along with the tree.
  // All the other node related collections don't contain the root node.
  //
  // Root node id.
  root_node_id: NodeId,

  // Root node.
  root_node: NodePtr,

  // Root }

  // Node {

  // A collection of all nodes, maps from node ID to node struct.
  nodes: BTreeMap<NodeId, NodePtr>,

  // A collection of all node attributes, maps from node ID to node attribute.
  attributes: HashMap<NodeId, NodeAttribute>,

  // Node }

  // Edge {

  // A collection of all edges.
  edges: BTreeSet<Edge>,

  // Maps "parent ID" => its "children IDs".
  children_ids: HashMap<NodeId, HashSet<NodeId>>,

  // Maps "child ID" => its "parent ID".
  parent_ids: HashMap<NodeId, NodeId>,
  // Edge }
}

pub struct RemovedSubTree {}

impl TreeBase {
  pub fn new() -> Self {
    let root_node = RootLayout::new();
    let root_node_id = root_node.id();
    TreeBase {
      root_node_id,
      root_node: make_node_ptr(Node::RootLayout(root_node)),
      nodes: BTreeMap::new(),
      attributes: HashMap::new(),
      edges: BTreeSet::new(),
      children_ids: HashMap::new(),
      parent_ids: HashMap::new(),
    }
  }
}
