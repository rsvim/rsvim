//! Widget tree that manages all the widget components.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use geo::point;

use crate::cart::{IPos, IRect, ISize, Size, URect, USize};
use crate::geo_size_as;
use crate::ui::tree::edge::Edge;
use crate::ui::tree::node::{NodeId, NodePtr};

pub mod edge;
pub mod node;

/// The widget tree.
///
/// A widget tree contains only 1 root node, each node can have 0 or multiple nodes. It manages all
/// UI components and rendering on the terminal, i.e. the whole terminal is the root widget node,
/// everything inside is the children nodes, and can recursively go down.
///
/// Here we have several terms:
/// * Parent: The parent node.
/// * Child: The child node.
/// * Ancestor: Either the parent, or the parent of some ancestor of the node.
/// * Descendant: Either the child, or the child of some descendant of the node.
/// * Sibling: Other children nodes under the same parent.
///
/// The widget tree ensures:
///
/// 1. Parent owns all its children.
///
///    * Children will be destroyed when their parent is.
///    * Coordinate system are relative to their parent's top-left corner, while the absolute
///      coordinates are based on the terminal's top-left corner.
///    * Children are displayed inside their parent's geometric shape, clipped by boundaries. While
///      the size of each node can be logically infinite on the imaginary canvas.
///    * The `visible` and `enabled` attributes of a child are implicitly inherited from it's
///      parent, unless they're explicitly been set.
///
/// 2. Children have higher priority than their parent to display and process input events.
///
///    * Children are always displayed on top of their parent, and has higher priority to process
///      a user's input event when the event occurs within the shape of the child. The event will
///      fallback to their parent if the child doesn't process it.
///    * For children that shade each other, the one with higher z-index has higher priority to
///      display and process the input events.
///
/// The widget shape ensures:
///
/// 1. A shape can be relative/logical or absolute/actual.
///
///    A widget shape is always a rectangle, the position is by default relative to its parent, and
///    the size is by default logically infinite. While rendering to the terminal device, we need
///    to calculate its absolute position and actual size.
///
/// 2. Calculate absolute/actual shape with "copy-on-write" policy.
///
///    Based on the fact that a widget's shape is often read and rarely modified, we use a
///    "copy-on-write" policy to avoid too many duplicated calculations. i.e. we always calculates
///    a widget's absolute position and actual size right after it's shape is been changed, and
///    also caches the result. Thus we simply get the cached results when need.
///
/// The widget has some attributes:
///
/// 1. A widget can be visible or invisible.
///
///    When it's visible, it handles user's input events, processes them and updates the UI
///    contents. When it's invisible, it's just like not existed, so it doesn't handle or process
///    any input events, the UI hides.
///
/// 2. A widget can be enabled or disabled.
///
///    When it's enabled, it handles input events, processes them and updates the UI contents. When
///    it's disabled, it's just like been fronzen, so it doesn't handle or process any input
///    events, the UI keeps still and never changes.
///
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
  actual_shapes: HashMap<NodeId, URect>,

  // Maps node "ID" => its "zindex".
  //
  /// The "z-index" arranges the display priority of the content stack when multiple children overlap
  /// on each other, a widget with higher z-index has higher priority to be displayed.
  ///
  /// Note: The z-index only works for the children under the same parent. For a child widget, it
  /// always covers/overrides its parent display. To change the visibility priority between
  /// children and parent, you need to change the relationship between them.
  /// For example, now we have two children under the same parent: A and B. A has 100 z-index, B
  /// has 10 z-index. Now B has a child: C, with z-index 1000. Even the z-index 1000 > 100 > 10, A
  /// still covers C, because it's a sibling of B.
  zindexes: HashMap<NodeId, usize>,

  // Maps node "ID" => its "visible".
  visibles: HashMap<NodeId, bool>,

  // Maps node "ID" => its "visible".
  enables: HashMap<NodeId, bool>,
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
      visibles: HashMap::new(),
      enables: HashMap::new(),
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

  // Attributes {

  pub fn get_visible(&self, id: NodeId) -> Option<&bool> {
    self.visibles.get(&id)
  }

  pub fn get_visible_mut(&mut self, id: NodeId) -> Option<&mut bool> {
    self.visibles.get_mut(&id)
  }

  pub fn set_visible(&mut self, id: NodeId, visible: bool) -> Option<bool> {
    self.visibles.insert(id, visible)
  }

  pub fn get_enabled(&self, id: NodeId) -> Option<&bool> {
    self.enables.get(&id)
  }

  pub fn get_enabled_mut(&mut self, id: NodeId) -> Option<&mut bool> {
    self.enables.get_mut(&id)
  }

  pub fn set_enabled(&mut self, id: NodeId, enabled: bool) -> Option<bool> {
    self.enables.insert(id, enabled)
  }

  // Attributes }
}