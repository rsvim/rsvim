//! Widget edge that connects two nodes in the tree.

use crate::ui::tree::node::NodeId;

/// Widget edge that connects two nodes in the tree.
#[derive(Hash, Copy, Clone, PartialEq, Eq, Default)]
pub struct Edge {
  pub from: NodeId,
  pub to: NodeId,
}

impl Edge {
  pub fn new(from: NodeId, to: NodeId) -> Self {
    Edge { from, to }
  }

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
