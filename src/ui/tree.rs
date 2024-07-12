//! Widget Tree that manages all the widget components.

use std::collections::BTreeMap;

pub type NodeId = usize;

enum Node {}

#[derive(Copy, Clone, Default, Hash, PartialEq, Eq)]
struct Edge {
  /// From Node ID
  from: NodeId,
  /// To Node ID
  to: NodeId,
}

struct Tree {
  nodes: Node,
  edges: Vec<Edge>,
  from_to_map: BTreeMap<NodeId, NodeId>,
}
