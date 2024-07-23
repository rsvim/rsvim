//! Widget tree that manages all the widget components.

use std::collections::VecDeque;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::sync::{Arc, RwLock, Weak};

use geo::point;

use crate::cart::{conversion, IRect, U16Rect, U16Size};
use crate::geo_rect_as;
use crate::ui::term::TerminalWk;
use crate::ui::tree::edge::Edge;
use crate::ui::tree::node::{NodeAttribute, NodeId, NodePtr};

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
/// A widget also has several attributes:
///
/// 1. Shape, i.e. position and size.
///
///    A shape can be relative/logical or absolute/actual, and always rectangle. The position is by
///    default relative to its parent, and the size is by default logically infinite. While
///    rendering to the terminal device, we need to calculate its absolute position and actual
///    size.
///
///    The absolute/actual shape is calculated with a "copy-on-write" policy. Based on the fact
///    that a widget's shape is often read and rarely modified, thus the "copy-on-write" policy to
///    avoid too many duplicated calculations. i.e. we always calculates a widget's absolute
///    position and actual size right after it's shape is been changed, and also caches the result.
///    Thus we simply get the cached results when need.
///
/// 2. Z-index.
///
///    The z-index arranges the display priority of the content stack when multiple children
///    overlap on each other, a widget with higher z-index has higher priority to be displayed.
///
///    The z-index only works for the children under the same parent. For a child widget, it always
///    covers/overrides its parent display.
///    To change the visibility priority between children and parent, you need to change the
///    relationship between them.
///    For example, now we have two children under the same parent: A and B. A has 100 z-index, B
///    has 10 z-index. Now B has a child: C, with z-index 1000. Even the z-index 1000 > 100 > 10, A
///    still covers C, because it's a sibling of B.
///
/// 3. Visible and enabled.
///
///    A widget can be visible or invisible. When it's visible, it handles user's input events,
///    processes them and updates the UI contents. When it's invisible, it's just like not existed,
///    so it doesn't handle or process any input events, the UI hides.
///
///    A widget can be enabled or disabled. When it's enabled, it handles input events, processes
///    them and updates the UI contents. When it's disabled, it's just like been fronzen, so it
///    doesn't handle or process any input events, the UI keeps still and never changes.
///
pub struct Tree {
  // Terminal reference.
  terminal: TerminalWk,

  // A collection of all nodes, maps from node ID to node struct.
  nodes: BTreeMap<NodeId, NodePtr>,

  // A collection of all edges.
  edges: BTreeSet<Edge>,

  // Maps node "ID" => its attributes.
  attributes: HashMap<NodeId, NodeAttribute>,

  // Root node ID.
  root_id: Option<NodeId>,

  // Maps "parent ID" => its "children IDs".
  //
  // Note: A parent can have multiple children.
  children_ids: BTreeMap<NodeId, HashSet<NodeId>>,

  // Maps "child ID" => its "parent ID".
  parent_ids: BTreeMap<NodeId, NodeId>,
}

pub type TreePtr = Arc<RwLock<Tree>>;
pub type TreeWk = Weak<RwLock<Tree>>;

pub fn make_tree_ptr(t: Tree) -> Arc<RwLock<Tree>> {
  Arc::new(RwLock::new(t))
}

impl Tree {
  pub fn new(terminal: TerminalWk) -> Tree {
    Tree {
      terminal: terminal.clone(),
      nodes: BTreeMap::new(),
      edges: BTreeSet::new(),
      root_id: None,
      children_ids: BTreeMap::new(),
      parent_ids: BTreeMap::new(),
      attributes: HashMap::new(),
    }
  }

  // Node {

  /// Get node by its ID.
  ///
  /// Returns the node if exists, returns `None` if not.
  pub fn get_node(&self, id: NodeId) -> Option<NodePtr> {
    self.nodes.get(&id).cloned()
  }

  /// Get the root node ID.
  ///
  /// Returns the root node ID if exists, returns `None` if not.
  pub fn get_root_node(&self) -> Option<NodeId> {
    self.root_id
  }

  /// Insert root node, with ID, size.
  ///
  /// Returns the inserted node if succeeded, returns `None` if failed.
  ///
  /// # Panics
  ///
  /// Panics if there's already a root node.
  pub fn insert_root_node(
    &mut self,
    id: NodeId,
    node: NodePtr,
    terminal_size: U16Size,
  ) -> Option<NodePtr> {
    assert!(self.root_id.is_none());
    self.root_id = Some(id);
    let result = self.nodes.insert(id, node.clone());
    let actual_shape = U16Rect::new((0, 0), (terminal_size.width(), terminal_size.height()));
    let shape = geo_rect_as!(actual_shape, isize);
    self
      .attributes
      .insert(id, NodeAttribute::default(shape, actual_shape));
    result
  }

  /// Insert node, with ID, parent's ID, shape.
  /// This operation also binds the connection between the inserted node and its parent.
  ///
  /// Returns the inserted node if succeeded, returns `None` if failed.
  pub fn insert_node(
    &mut self,
    id: NodeId,
    node: NodePtr,
    parent_id: NodeId,
    shape: IRect,
  ) -> Option<NodePtr> {
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
    let actual_shape = match self.attributes.get(&parent_id) {
      Some(parent_attribute) => conversion::to_actual_shape(shape, parent_attribute.actual_shape),
      None => {
        let terminal_size = self.terminal.upgrade().unwrap().read().unwrap().size();
        let terminal_actual_shape =
          U16Rect::new((0, 0), (terminal_size.width(), terminal_size.height()));
        conversion::to_actual_shape(shape, terminal_actual_shape)
      }
    };
    self
      .attributes
      .insert(id, NodeAttribute::default(shape, actual_shape));
    self.nodes.insert(id, node.clone())
  }

  /// Remove node by its ID.
  ///
  /// Returns the removed node if it exists, returns `None` if not.
  /// Returns `None` if the node is root node.
  ///
  /// This operation also removes the connection between the node and its parent (if any).
  /// This operation doesn't removes the connection between the node and its children (if any).
  pub fn remove_node(&mut self, id: NodeId) -> Option<NodePtr> {
    if self.root_id == Some(id) {
      return None;
    }
    if !self.parent_ids.contains_key(&id) {
      return None;
    }

    let parent_id = self.parent_ids.remove(&id).unwrap();
    assert!(self.children_ids.contains_key(&parent_id));

    let child_removed = self.children_ids.get_mut(&parent_id).unwrap().remove(&id);
    assert!(child_removed);

    let attribute_removed = self.attributes.remove(&id);
    assert!(attribute_removed.is_some());

    let edge_removed = self.edges.remove(&Edge::new(parent_id, id));
    assert!(edge_removed);

    assert!(self.nodes.contains_key(&id));
    self.nodes.remove(&id)
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

  // Attribute {

  pub fn get_shape(&self, id: NodeId) -> Option<&IRect> {
    match self.attributes.get(&id) {
      Some(attr) => Some(&attr.shape),
      None => None,
    }
  }

  pub fn set_shape(&mut self, id: NodeId, shape: IRect) -> Option<IRect> {
    match self.attributes.get_mut(&id) {
      Some(attr) => {
        let old_shape = attr.shape;
        attr.shape = shape;
        // Update the actual shape of `id`, and all its descendant nodes.
        self.calculate_actual_shape(id);
        Some(old_shape)
      }
      None => None,
    }
  }

  fn calculate_actual_shape(&mut self, start_id: NodeId) {
    let mut que: VecDeque<NodeId> = VecDeque::new();
    que.push_back(start_id);

    while let Some(id) = que.pop_front() {
      let shape = self.attributes.get(&id).unwrap().shape;
      let actual_shape = match self.parent_ids.get_mut(&id) {
        Some(parent_id) => {
          let parent_actual_shape = self.attributes.get(parent_id).unwrap().actual_shape;
          conversion::to_actual_shape(shape, parent_actual_shape)
        }
        None => {
          let terminal_size = self.terminal.upgrade().unwrap().read().unwrap().size();
          let terminal_actual_shape: U16Rect =
            U16Rect::new((0, 0), (terminal_size.width(), terminal_size.height()));
          conversion::to_actual_shape(shape, terminal_actual_shape)
        }
      };
      self.attributes.get_mut(&id).unwrap().actual_shape = actual_shape;

      // Add all children of `id` to the queue.
      match self.children_ids.get(&id) {
        Some(children_ids) => {
          for child_id in children_ids.iter() {
            que.push_back(*child_id);
          }
        }
        None => {
          // Do nothing
        }
      }
    }
  }

  pub fn get_actual_shape(&self, id: NodeId) -> Option<&U16Rect> {
    match self.attributes.get(&id) {
      Some(attr) => Some(&attr.actual_shape),
      None => None,
    }
  }

  // Attribute }
}
