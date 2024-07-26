//! Layout is a logical container that manages all its nested children widgets, and arranges their
//! layout and shapes.
//!
//! Layout is a special tree node, it's also a tree-structure when implemented in the widget
//! [tree](crate::ui::tree::Tree).

use crate::ui::tree::edge::Edge;
use crate::ui::tree::node::{make_node_ptr, Node, NodeAttribute, NodeId, NodePtr};
use crate::uuid;
use geo::point;
use std::cmp::{max, min};
use std::collections::VecDeque;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::cart::U16Rect;
use crate::ui::term::TerminalWk;
use crate::ui::widget::Widget;

pub mod root;

// Re-export
pub use crate::ui::layout::root::RootLayout;

/// Layout widget is a special widget that has no specific shape or content, but works as a logical
/// container for nested children widgets, and arrange their layout.
///
/// The layout has to be both a node inside the widget [tree](crate::ui::tree::Tree) and a sub-tree
/// structure (to implement the layout/shape management for all its nested children).
pub trait Layout: Widget {
  fn id(&self) -> NodeId;

  fn draw(&mut self, _actual_shape: &U16Rect, _terminal: TerminalWk) {
    // Do nothing.
  }
}

/// Common behaviors for all layout struct implementations.
pub struct LayoutBase {
  // Terminal reference.
  terminal: TerminalWk,

  // Node ID.
  id: NodeId,

  // Node {

  // A collection of all nodes, maps from node ID to node struct.
  nodes: BTreeMap<NodeId, NodePtr>,

  // Maps node "ID" => its attributes.
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

impl LayoutBase {
  /// Make a layout base.
  pub fn new(terminal: TerminalWk) -> Self {
    LayoutBase {
      id: uuid::next(),
      terminal,
      nodes: BTreeMap::new(),
      edges: BTreeSet::new(),
      children_ids: HashMap::new(),
      parent_ids: HashMap::new(),
      attributes: HashMap::new(),
    }
  }

  /// Whether the tree is empty.
  pub fn is_empty(&self) -> bool {
    self.root_id.is_none()
      && self.nodes.is_empty()
      && self.edges.is_empty()
      && self.children_ids.is_empty()
      && self.parent_ids.is_empty()
      && self.attributes.is_empty()
      && self.window_ids.is_empty()
  }
}
