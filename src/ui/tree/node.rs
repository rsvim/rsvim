//! Widget node in the tree.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::sync::{Arc, RwLock, Weak};
use tracing::debug;

use crate::cart::{shapes, IRect, U16Rect};
use crate::ui::term::TerminalWk;
use crate::ui::widget::cursor::Cursor;
use crate::ui::widget::layout::root::RootLayout;
use crate::ui::widget::window::Window;
use crate::ui::widget::Widget;

pub type NodeId = usize;

/// Widget node in the tree.
#[derive(Debug)]
pub enum Node {
  RootLayout(RootLayout),
  Cursor(Cursor),
  Window(Window),
}

pub type NodePtr = Arc<RwLock<Node>>;
pub type NodeWk = Weak<RwLock<Node>>;

pub fn make_node_ptr(n: Node) -> NodePtr {
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

impl Widget for Node {
  fn id(&self) -> NodeId {
    match self {
      Self::RootLayout(node) => node.id(),
      Self::Cursor(node) => node.id(),
      Self::Window(node) => node.id(),
    }
  }

  fn draw(&mut self, actual_shape: &U16Rect, terminal: TerminalWk) {
    match self {
      Self::RootLayout(node) => node.draw(actual_shape, terminal.clone()),
      Self::Cursor(node) => node.draw(actual_shape, terminal.clone()),
      Self::Window(node) => node.draw(actual_shape, terminal.clone()),
    }
  }
}

#[derive(Copy, Clone)]
pub struct NodeAttribute {
  /// Relative and logical shape of a widget node.
  pub shape: IRect,

  /// Absolute and actual shape of a widget node.
  pub actual_shape: U16Rect,

  pub zindex: usize,
  pub visible: bool,
  pub enabled: bool,
}

impl NodeAttribute {
  pub fn new(
    shape: IRect,
    actual_shape: U16Rect,
    zindex: usize,
    visible: bool,
    enabled: bool,
  ) -> Self {
    NodeAttribute {
      shape,
      actual_shape,
      zindex,
      visible,
      enabled,
    }
  }

  pub fn default(shape: IRect, actual_shape: U16Rect) -> Self {
    NodeAttribute {
      shape,
      actual_shape,
      zindex: 0_usize,
      visible: true,
      enabled: true,
    }
  }
}

pub struct NodesCollection {
  // A collection of all nodes, maps from node ID to node struct.
  nodes: BTreeMap<NodeId, NodePtr>,

  // A collection of all node attributes, maps from node ID to node attribute.
  attributes: HashMap<NodeId, NodeAttribute>,
}

pub struct RemovedSubTree {
  // A collection of all the removed nodes, maps from node ID to node struct.
  nodes: BTreeMap<NodeId, NodePtr>,

  // A collection of all the removed node attributes, maps from node ID to node attribute.
  attributes: HashMap<NodeId, NodeAttribute>,
}

impl NodesCollection {
  /// Make a nodes collection.
  pub fn new() -> Self {
    NodesCollection {
      nodes: BTreeMap::new(),
      attributes: HashMap::new(),
    }
  }

  /// Whether the collection has no nodes.
  pub fn is_empty(&self) -> bool {
    self.nodes.is_empty() && self.attributes.is_empty()
  }

  /// Whether the collection contains a node.
  pub fn contains(&self, id: NodeId) -> bool {
    self.nodes.contains_key(&id) && self.attributes.contains_key(&id)
  }

  /// Get the internal nodes tree-map.
  pub fn nodes(&self) -> &BTreeMap<NodeId, NodePtr> {
    &self.nodes
  }

  /// Get the internal attributes tree-map.
  pub fn attributes(&self) -> &HashMap<NodeId, NodeAttribute> {
    &self.attributes
  }

  /// Get a node by its ID.
  pub fn get_node(&self, id: NodeId) -> Option<&NodePtr> {
    self.nodes.get(&id)
  }

  /// Get a node attribute by its ID.
  pub fn get_attribute(&self, id: NodeId) -> Option<&NodeAttribute> {
    self.attributes.get(&id)
  }

  /// Insert a node.
  /// This operation also binds the connection between the inserted node and its parent.
  ///
  /// Returns the inserted node if succeeded, returns `None` if failed.
  ///
  /// Fails if:
  /// 1. The parent (`parent_id`) not exists.
  /// 2. The node (`id`) already exists.
  pub fn insert(
    &mut self,
    id: NodeId,
    node: NodePtr,
    parent_id: NodeId,
    shape: IRect,
  ) -> Option<NodePtr> {
    // Fails if the node already exists, or parent node not exists.
    if self.contains(id) || !self.contains(parent_id) {
      return None;
    }

    let parent_actual_shape = self.attributes.get(&parent_id).unwrap().actual_shape;
    let actual_shape = shapes::convert_to_actual_shape(shape, parent_actual_shape);
    debug!("Calculated actual shape:{:?}", actual_shape);
    self
      .attributes
      .insert(id, NodeAttribute::default(shape, actual_shape));

    self.nodes.insert(id, node.clone())
  }

  /// Remove a node.
  /// This operation also breaks the connection between the removed node and its parent, all
  /// descendants (if any) under the removed node are removed as well.
  ///
  /// Returns the removed node (actually a sub-tree structure if there're any descendants) if
  /// succeeded, returns `None` if failed.
  pub fn remove(&mut self, id: NodeId) -> Option<(NodePtr, NodeAttribute)> {}
}

impl RemovedSubTree {
  pub fn new() -> Self {
    RemovedSubTree {
      nodes: BTreeMap::new(),
      attributes: HashMap::new(),
    }
  }
}
