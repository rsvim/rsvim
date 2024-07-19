//! Widget tree that manages all the widget components.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use geo::point;

use crate::cart::{IPos, IRect, ISize, Size, URect, USize};
use crate::geo_size_as;
use crate::ui::tree::edge::Edge;
use crate::ui::tree::node::{NodeId, NodePtr};

pub mod edge;
pub mod node;

/// Widget tree.
/// A widget tree contains only 1 root node, each node can have 0 or multiple nodes.
pub struct Tree {
  // A collection of all nodes, maps from node ID to node struct.
  nodes: BTreeMap<NodeId, NodePtr>,

  // A collection of all edges.
  edges: BTreeSet<Edge>,

  // Root node ID.
  root_id: Option<NodeId>,

  // Maps "parent ID" => its "children IDs".
  //
  // Note: A parent can have multiple children.
  children_ids: BTreeMap<NodeId, HashSet<NodeId>>,

  // Maps "child ID" => its "parent ID".
  parent_ids: BTreeMap<NodeId, NodeId>,

  // Maps node "ID" => its "relative and logical shape", i.e. relative position and logical shape
  // on its parent widget (when doesn't have a parent, the terminal is its parent).
  //
  // The coordinate system by default uses relative and logical shape, this is mostly for the
  // convenience of calculation.
  //
  // Note: A widget is always a rectangle.
  shapes: HashMap<NodeId, IRect>,

  // Maps node "ID" => its "absolute and actual shape", i.e. actual position and size on a terminal.
  //
  // Every time after a node's shape changes, i.e. its position moves or its shape resizes,
  // the tree will calculate the updated its actual shape (and all its children's actual shapes),
  // and cache all the results.
  // Thus when drawing the nodes to the terminal, we only need to get the cached results, instead
  // of real-time calculation (which involves too much duplicated calculation).
  //
  // This is based on the fact that for a widget's actual shape, we read more while modify less.
  // And mostly the user will only modify the leaf node widget, because it's on the top of a widget
  // tree, which gets the attention of user's eyes.
  //
  // Note: A widget is always a rectangle.
  actual_shapes: HashMap<NodeId, URect>,

  // Maps node "ID" => its "zindex".
  zindexes: HashMap<NodeId, usize>,
}

impl Tree {
  pub fn new() -> Tree {
    Tree {
      nodes: BTreeMap::new(),
      edges: BTreeSet::new(),
      root_id: None,
      children_ids: BTreeMap::new(),
      parent_ids: BTreeMap::new(),
      shapes: HashMap::new(),
      actual_shapes: HashMap::new(),
      zindexes: HashMap::new(),
    }
  }

  // Node {

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
    self.edges.insert(Edge::new(parent_id, id));
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

  // Node }

  // Edge {

  /// Get edge by the `from` node ID and the `to` node ID.
  ///
  /// Returns the edge if exists, returns `None` if not.
  pub fn get_edge(&self, from: NodeId, to: NodeId) -> Option<&Edge> {
    self.edges.get(&Edge { from, to })
  }

  // Edge }

  // Parent-Children Relationship {

  pub fn get_children(&self, parent_id: NodeId) -> Option<&HashSet<NodeId>> {
    self.children_ids.get(&parent_id)
  }

  pub fn get_parent(&self, child_id: NodeId) -> Option<&NodeId> {
    self.parent_ids.get(&child_id)
  }

  // Parent-Children Relationship }

  // Shape {

  pub fn get_shape(&self, id: NodeId) -> Option<&IRect> {
    self.shapes.get(&id)
  }

  pub fn get_shape_mut(&mut self, id: NodeId) -> Option<&mut IRect> {
    self.shapes.get_mut(&id)
  }

  pub fn set_shape(&mut self, id: NodeId, shape: IRect) -> Option<IRect> {
    self.shapes.insert(id, shape)
  }

  pub fn get_pos(&self, id: NodeId) -> Option<IPos> {
    match self.get_shape(id) {
      Some(shape) => Some(point!(x: shape.min().x, y: shape.min().y)),
      None => None,
    }
  }

  pub fn set_pos(&mut self, id: NodeId, pos: IPos) -> Option<IPos> {
    match self.get_shape_mut(id) {
      Some(shape) => {
        let old_pos = point!(x:shape.min().x, y:shape.min().y);
        *shape = IRect::new(
          pos,
          point!(x:pos.x() + shape.width(), y: pos.y() + shape.height() ),
        );
        Some(old_pos)
      }
      None => None,
    }
  }

  pub fn get_size(&self, id: NodeId) -> Option<USize> {
    match self.get_shape(id) {
      Some(shape) => {
        let isz = ISize::from(*shape);
        let usz = geo_size_as!(isz, usize);
        Some(usz)
      }
      None => None,
    }
  }

  pub fn set_size(&mut self, id: NodeId, sz: USize) -> Option<USize> {
    match self.get_shape_mut(id) {
      Some(shape) => {
        let old_isz = ISize::from(*shape);
        let old_usz = geo_size_as!(old_isz, usize);
        let pos = point!(x: shape.min().x, y: shape.min().y);
        *shape = IRect::new(
          pos,
          pos + point!(x: sz.width() as isize, y: sz.height() as isize),
        );
        Some(old_usz)
      }
      None => None,
    }
  }

  pub fn get_zindex(&self, id: NodeId) -> Option<&usize> {
    self.zindexes.get(&id)
  }

  pub fn get_zindex_mut(&mut self, id: NodeId) -> Option<&mut usize> {
    self.zindexes.get_mut(&id)
  }

  pub fn set_zindex(&mut self, id: NodeId, zindex: usize) -> Option<usize> {
    self.zindexes.insert(id, zindex)
  }

  // Shape }
}
