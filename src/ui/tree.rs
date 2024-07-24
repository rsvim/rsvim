//! Widget tree that manages all the widget components.

use std::collections::VecDeque;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::sync::{Arc, RwLock, Weak};

use geo::point;

use crate::cart::{conversion, IPos, IRect, ISize, U16Pos, U16Rect, U16Size};
use crate::geo_rect_as;
use crate::ui::term::TerminalWk;
use crate::ui::widget::Widget;

// Re-export
pub use crate::ui::tree::edge::Edge;
pub use crate::ui::tree::node::{make_node_ptr, Node, NodeAttribute, NodeId, NodePtr};

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
/// A widget has several attributes:
///
/// 1. Shape, i.e. position and size.
///
///    A shape can be relative/logical or absolute/actual, and always rectangle. The position is by
///    default relative to its parent, and the size is by default logically infinite. While
///    rendering to the terminal device, we need to calculate its absolute position and actual
///    size.
///
///    The shape boundary uses top-left open, bottom-right closed interval. For example the
///    terminal shape is `((0,0), (10,10))`, the top-left position `(0,0)` is inclusive, i.e.
///    inside the shape, the bottom-right position `(10,10)` is exclusive, i.e. outside the shape.
///    The width and height of the shape is both `10`.
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
///
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

  // A collection of all VIM window widget nodes.
  window_ids: BTreeSet<NodeId>,

  // Maps "parent ID" => its "children IDs".
  //
  // Note: A parent can have multiple children.
  children_ids: HashMap<NodeId, HashSet<NodeId>>,

  // Maps "child ID" => its "parent ID".
  parent_ids: HashMap<NodeId, NodeId>,
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
      window_ids: BTreeSet::new(),
      children_ids: HashMap::new(),
      parent_ids: HashMap::new(),
      attributes: HashMap::new(),
    }
  }

  // Node {

  /// Get the collection of all nodes.
  pub fn get_nodes(&self) -> &BTreeMap<NodeId, NodePtr> {
    &self.nodes
  }

  /// Get node by its ID.
  ///
  /// Returns the node if exists, returns `None` if not.
  pub fn get_node(&self, id: NodeId) -> Option<NodePtr> {
    self.nodes.get(&id).cloned()
  }

  /// Get the root node ID.
  ///
  /// Returns the root node ID if exists, returns `None` if not.
  pub fn get_root_id(&self) -> Option<NodeId> {
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

    // If `node` is a window widget, add it into the `window_ids` collection.
    if let Node::WindowNode(window_node) = &*node.read().unwrap() {
      self.window_ids.insert(window_node.id());
    }

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
    if !self.nodes.contains_key(&id) {
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

    let removed_node = self.nodes.remove(&id).unwrap();

    let removed_window = self.window_ids.remove(&id);
    match &*removed_node.read().unwrap() {
      Node::WindowNode(_) => assert!(removed_window),
      _ => assert!(!removed_window),
    }

    Some(removed_node)
  }

  // Node }

  // Edge {

  /// Get the collection of all edges.
  pub fn get_edges(&self) -> &BTreeSet<Edge> {
    &self.edges
  }

  /// Get edge by the `from` node ID and the `to` node ID.
  ///
  /// Returns the edge if exists, returns `None` if not.
  pub fn get_edge(&self, from: NodeId, to: NodeId) -> Option<&Edge> {
    self.edges.get(&Edge { from, to })
  }

  // Edge }

  // Parent-Children Relationship {

  /// Get the collection of all "parent" => "children" IDs mapping.
  pub fn get_children_ids(&self) -> &HashMap<NodeId, HashSet<NodeId>> {
    &self.children_ids
  }

  /// Get the collection of all "child" => "parent" ID mapping.
  pub fn get_parent_ids(&self) -> &HashMap<NodeId, NodeId> {
    &self.parent_ids
  }

  /// Get the children IDs by the `parent` ID.
  pub fn get_children(&self, parent_id: NodeId) -> Option<&HashSet<NodeId>> {
    self.children_ids.get(&parent_id)
  }

  /// Get the parent ID by the `child` ID.
  pub fn get_parent(&self, child_id: NodeId) -> Option<&NodeId> {
    self.parent_ids.get(&child_id)
  }

  // Parent-Children Relationship }

  // Window Nodes {

  /// Get the collection of all window widget nodes.
  pub fn get_window_ids(&self) -> &BTreeSet<NodeId> {
    &self.window_ids
  }

  // Window Nodes }

  // Attribute {

  /// Get the collection of all node attributes.
  pub fn get_attributes(&self) -> &HashMap<NodeId, NodeAttribute> {
    &self.attributes
  }

  /// Get shape of a node.
  pub fn get_shape(&self, id: NodeId) -> Option<&IRect> {
    match self.attributes.get(&id) {
      Some(attr) => Some(&attr.shape),
      None => None,
    }
  }

  /// Set shape of a node.
  ///
  /// Note: This triggers the calculation of its actual shape, and all its descendants actual
  /// shapes.
  pub fn set_shape(&mut self, id: NodeId, shape: IRect) -> Option<IRect> {
    match self.attributes.get_mut(&id) {
      Some(attr) => {
        let old_shape = attr.shape;
        attr.shape = shape;
        // Update the actual shape of `id`, and all its descendant nodes.
        self.update_actual_shape(id);
        Some(old_shape)
      }
      None => None,
    }
  }

  /// Internal implementation of [`set_shape`](Tree::set_shape()).
  ///
  /// It updates the node's (`start_id`) actual shape, and all the descendants actual shapes.
  fn update_actual_shape(&mut self, start_id: NodeId) {
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

  /// Get the position of a node.
  pub fn get_pos(&self, id: NodeId) -> Option<IPos> {
    self.attributes.get(&id).map(|attr| attr.shape.min().into())
  }

  /// Set the position of a node.
  pub fn set_pos(&mut self, id: NodeId, pos: IPos) -> Option<IPos> {
    match self.attributes.get_mut(&id) {
      Some(attr) => {
        let old_shape = attr.shape;
        let new_shape = IRect::new(
          pos,
          point!(x: pos.x() + old_shape.width(), y: pos.y() + old_shape.height()),
        );
        self.set_shape(id, new_shape);
        Some(old_shape.min().into())
      }
      None => None,
    }
  }

  /// Get the size of a node.
  pub fn get_size(&self, id: NodeId) -> Option<ISize> {
    self.attributes.get(&id).map(|attr| ISize::from(attr.shape))
  }

  /// Set the size of a node.
  pub fn set_size(&mut self, id: NodeId, size: ISize) -> Option<ISize> {
    match self.attributes.get_mut(&id) {
      Some(attr) => {
        let old_shape = attr.shape;
        let old_pos: IPos = old_shape.min().into();
        let new_shape = IRect::new(
          old_pos,
          point!(x: old_pos.x() + size.width(), y: old_pos.y() + size.height()),
        );
        self.set_shape(id, new_shape);
        Some(ISize::from(old_shape))
      }
      None => None,
    }
  }

  /// Get the actual shape of a node.
  pub fn get_actual_shape(&self, id: NodeId) -> Option<&U16Rect> {
    match self.attributes.get(&id) {
      Some(attr) => Some(&attr.actual_shape),
      None => None,
    }
  }

  /// Get the actual position of a node.
  pub fn get_actual_pos(&self, id: NodeId) -> Option<U16Pos> {
    self
      .attributes
      .get(&id)
      .map(|attr| attr.actual_shape.min().into())
  }

  /// Get the actual size of a node.
  pub fn get_actual_size(&self, id: NodeId) -> Option<U16Size> {
    self
      .attributes
      .get(&id)
      .map(|attr| U16Size::from(attr.actual_shape))
  }

  // Attribute }

  // Draw {

  /// Draw the widget tree to terminal device.
  pub fn draw(&mut self) {}

  // Draw }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::cart::{IPos, IRect, ISize, Size, U16Pos, U16Rect, U16Size};
  use crate::geo_size_as;
  use crate::ui::term::{make_terminal_ptr, Terminal};
  use crate::ui::widget::{Cursor, RootWidget, Widget, Window};
  use tracing::info;

  #[test]
  fn tree_new() {
    let terminal = Terminal::new(U16Size::new(10, 10));
    let terminal = make_terminal_ptr(terminal);

    let tree = Tree::new(Arc::downgrade(&terminal));
    assert!(tree.get_nodes().is_empty());
    assert!(tree.get_edges().is_empty());
    assert!(tree.get_children_ids().is_empty());
    assert!(tree.get_parent_ids().is_empty());
    assert!(tree.get_root_id().is_none());
    assert!(tree.get_window_ids().is_empty());
    assert!(tree.get_attributes().is_empty());
  }

  #[test]
  fn tree_insert() {
    let terminal = Terminal::new(U16Size::new(10, 10));
    let terminal = make_terminal_ptr(terminal);

    let mut tree = Tree::new(Arc::downgrade(&terminal));

    let n1 = RootWidget::new();
    let n1 = make_node_ptr(Node::RootWidgetNode(n1));

    let n2 = Window::default();
    let n2 = make_node_ptr(Node::WindowNode(n2));

    let n3 = Window::default();
    let n3 = make_node_ptr(Node::WindowNode(n3));

    let n4 = Cursor::default();
    let n4 = make_node_ptr(Node::CursorNode(n4));

    tree.insert_root_node(
      n1.read().unwrap().id(),
      n1.clone(),
      terminal.read().unwrap().size(),
    );
    tree.insert_node(
      n2.read().unwrap().id(),
      n2.clone(),
      n1.read().unwrap().id(),
      IRect::new((0, 0), (10, 10)),
    );
    tree.insert_node(
      n3.read().unwrap().id(),
      n3.clone(),
      n1.read().unwrap().id(),
      IRect::new((0, 0), (10, 10)),
    );
    tree.insert_node(
      n4.read().unwrap().id(),
      n4.clone(),
      n2.read().unwrap().id(),
      IRect::new((0, 0), (1, 1)),
    );

    // println!("ui::tree::tree_insert get_nodes:{:?}", tree.get_nodes());
    assert!(tree.get_nodes().len() == 4);
    assert!(tree.get_edges().len() == 3);
    assert!(tree.get_children_ids().len() == 2);
    assert!(tree.get_parent_ids().len() == 3);
    assert!(tree.get_root_id().unwrap() == n1.read().unwrap().id());
    assert!(tree.get_window_ids().len() == 2);
    assert!(tree.get_attributes().len() == 4);

    let node_ids: Vec<NodeId> = [n1, n2, n3, n4]
      .iter()
      .map(|node| node.read().unwrap().id())
      .collect();
    for i in node_ids.iter() {
      let actual = tree.get_node(*i);
      assert!(actual.is_some());
      assert!(actual.unwrap().read().unwrap().id() == *i);
    }
    let edges: Vec<Edge> = vec![
      Edge::new(node_ids[0], node_ids[1]),
      Edge::new(node_ids[0], node_ids[2]),
      Edge::new(node_ids[1], node_ids[3]),
    ];
    for e in edges.iter() {
      let actual = tree.get_edge(e.from, e.to);
      assert!(actual.is_some());
      assert!(actual.unwrap().from == e.from);
      assert!(actual.unwrap().to == e.to);
    }
    let children_ids: Vec<(NodeId, HashSet<NodeId>)> = vec![
      (
        node_ids[0],
        [node_ids[1], node_ids[2]].iter().cloned().collect(),
      ),
      (node_ids[1], [node_ids[3]].iter().cloned().collect()),
      (node_ids[2], [].iter().cloned().collect()),
      (node_ids[3], [].iter().cloned().collect()),
    ];
    for c in children_ids.iter() {
      let actual = tree.get_children(c.0);
      if c.1.is_empty() {
        assert!(actual.is_none());
      } else {
        assert!(*actual.unwrap() == c.1);
      }
    }
    let parent_ids: Vec<(NodeId, NodeId)> = vec![
      (node_ids[1], node_ids[0]),
      (node_ids[2], node_ids[0]),
      (node_ids[3], node_ids[1]),
    ];
    for p in parent_ids.iter() {
      let actual = tree.get_parent(p.0);
      assert!(actual.is_some());
      assert!(*actual.unwrap() == p.1);
    }
    let window_ids: BTreeSet<NodeId> = [node_ids[1], node_ids[2]].iter().cloned().collect();
    assert!(window_ids == *tree.get_window_ids());
  }

  #[test]
  fn tree_shape1() {
    let terminal = Terminal::new(U16Size::new(10, 10));
    let terminal_size = terminal.size();
    let terminal = make_terminal_ptr(terminal);

    let mut tree = Tree::new(Arc::downgrade(&terminal));

    let n1 = RootWidget::new();
    let n1_id = n1.id();
    let n1 = make_node_ptr(Node::RootWidgetNode(n1));

    let n2 = Window::default();
    let n2_id = n2.id();
    let n2 = make_node_ptr(Node::WindowNode(n2));

    let n3 = Window::default();
    let n3_id = n3.id();
    let n3 = make_node_ptr(Node::WindowNode(n3));

    let n4 = Cursor::default();
    let n4_id = n4.id();
    let n4 = make_node_ptr(Node::CursorNode(n4));

    tree.insert_root_node(n1.read().unwrap().id(), n1.clone(), terminal_size);
    let shape1 = tree.get_shape(n1_id);
    let pos1 = tree.get_pos(n1_id);
    let size1 = tree.get_size(n1_id);
    let actual_shape1 = tree.get_actual_shape(n1_id);
    let actual_pos1 = tree.get_actual_pos(n1_id);
    let actual_size1 = tree.get_actual_size(n1_id);
    assert!(
      *shape1.unwrap()
        == IRect::new(
          (0, 0),
          (
            terminal_size.width() as isize,
            terminal_size.height() as isize
          )
        )
    );
    assert!(pos1.unwrap() == point!(x:0, y:0));
    assert!(
      size1.unwrap()
        == ISize::new(
          terminal_size.width() as isize,
          terminal_size.height() as isize
        )
    );
    assert!(
      *actual_shape1.unwrap()
        == U16Rect::new((0, 0), (terminal_size.width(), terminal_size.height()))
    );
    assert!(actual_pos1.unwrap() == U16Pos::new(0_u16, 0_u16));
    assert!(actual_size1.unwrap() == terminal_size);

    tree.insert_node(
      n2.read().unwrap().id(),
      n2.clone(),
      n1.read().unwrap().id(),
      IRect::new((0, 0), (3, 5)),
    );
    let shape2 = tree.get_shape(n2_id);
    let pos2 = tree.get_pos(n2_id);
    let size2 = tree.get_size(n2_id);
    let actual_shape2 = tree.get_actual_shape(n2_id);
    let actual_pos2 = tree.get_actual_pos(n2_id);
    let actual_size2 = tree.get_actual_size(n2_id);
    assert!(*shape2.unwrap() == IRect::new((0, 0), (3, 5)));
    assert!(pos2.unwrap() == point!(x:0, y:0));
    assert!(size2.unwrap() == ISize::new(3, 5));
    assert!(*actual_shape2.unwrap() == U16Rect::new((0, 0), (3_u16, 5_u16)));
    assert!(actual_pos2.unwrap() == U16Pos::new(0_u16, 0_u16));
    assert!(actual_size2.unwrap() == U16Size::new(3_u16, 5_u16));

    tree.insert_node(
      n3.read().unwrap().id(),
      n3.clone(),
      n1.read().unwrap().id(),
      IRect::new((3, 5), (9, 10)),
    );
    let shape3 = tree.get_shape(n3_id);
    let pos3 = tree.get_pos(n3_id);
    let size3 = tree.get_size(n3_id);
    let actual_shape3 = tree.get_actual_shape(n3_id);
    let actual_pos3 = tree.get_actual_pos(n3_id);
    let actual_size3 = tree.get_actual_size(n3_id);
    assert!(*shape3.unwrap() == IRect::new((3, 5), (9, 10)));
    assert!(pos3.unwrap() == point!(x:3, y:5));
    assert!(size3.unwrap() == ISize::new(6, 5));
    assert!(*actual_shape3.unwrap() == U16Rect::new((3, 5), (9_u16, 10_u16)));
    assert!(actual_pos3.unwrap() == U16Pos::new(3_u16, 5_u16));
    assert!(actual_size3.unwrap() == U16Size::new(6_u16, 5_u16));

    tree.insert_node(
      n4.read().unwrap().id(),
      n4.clone(),
      n2.read().unwrap().id(),
      IRect::new((0, 0), (1, 1)),
    );
    let shape4 = tree.get_shape(n4_id);
    let pos4 = tree.get_pos(n4_id);
    let size4 = tree.get_size(n4_id);
    let actual_shape4 = tree.get_actual_shape(n4_id);
    let actual_pos4 = tree.get_actual_pos(n4_id);
    let actual_size4 = tree.get_actual_size(n4_id);
    assert!(*shape4.unwrap() == IRect::new((0, 0), (1, 1)));
    assert!(pos4.unwrap() == point!(x:0, y:0));
    assert!(size4.unwrap() == ISize::new(1, 1));
    assert!(*actual_shape4.unwrap() == U16Rect::new((0, 0), (1_u16, 1_u16)));
    assert!(actual_pos4.unwrap() == U16Pos::new(0_u16, 0_u16));
    assert!(actual_size4.unwrap() == U16Size::new(1_u16, 1_u16));

    let expects: Vec<(IRect, IPos, ISize, U16Rect, U16Pos, U16Size)> = vec![
      (
        IRect::new(
          (0, 0),
          (
            terminal_size.width() as isize,
            terminal_size.height() as isize,
          ),
        ),
        point!(x:0, y:0),
        geo_size_as!(terminal_size, isize),
        U16Rect::new((0, 0), (terminal_size.width(), terminal_size.height())),
        point!(x: 0_u16, y: 0_u16),
        terminal_size,
      ),
      (
        IRect::new((0, 0), (3, 5)),
        point!(x:0, y:0),
        ISize::new(3, 5),
        U16Rect::new((0, 0), (3_u16, 5_u16)),
        point!(x: 0_u16, y: 0_u16),
        U16Size::new(3_u16, 5_u16),
      ),
      (
        IRect::new((3, 5), (9, 10)),
        point!(x:3, y:5),
        ISize::new(6, 5),
        U16Rect::new((3, 5), (9_u16, 10_u16)),
        point!(x: 3_u16, y: 5_u16),
        U16Size::new(6_u16, 5_u16),
      ),
      (
        IRect::new((0, 0), (1, 1)),
        point!(x:0, y:0),
        ISize::new(1, 1),
        U16Rect::new((0, 0), (1_u16, 1_u16)),
        point!(x: 0_u16, y: 0_u16),
        U16Size::new(1_u16, 1_u16),
      ),
    ];

    let node_ids = [n1_id, n2_id, n3_id, n4_id];
    for (i, id) in node_ids.iter().enumerate() {
      let shape = tree.get_shape(*id);
      let pos = tree.get_pos(*id);
      let size = tree.get_size(*id);
      let actual_shape = tree.get_actual_shape(*id);
      let actual_pos = tree.get_actual_pos(*id);
      let actual_size = tree.get_actual_size(*id);
      let expect = expects[i];
      assert!(*shape.unwrap() == expect.0);
      assert!(pos.unwrap() == expect.1);
      assert!(size.unwrap() == expect.2);
      assert!(*actual_shape.unwrap() == expect.3);
      assert!(actual_pos.unwrap() == expect.4);
      // println!(
      //   "ui::tree::tree_shape1 i:{:?}, actual_size:{:?}, expect:{:?}",
      //   i, actual_size, expect
      // );
      assert!(actual_size.unwrap() == expect.5);
    }
  }

  #[test]
  fn tree_shape2() {
    let terminal_size = U16Size::new(50, 50);
    let terminal = Terminal::new(terminal_size);
    let terminal = make_terminal_ptr(terminal);

    let mut tree = Tree::new(Arc::downgrade(&terminal));

    let n1 = RootWidget::new();
    let nid1 = n1.id();
    let n1 = make_node_ptr(Node::RootWidgetNode(n1));

    let n2 = Window::default();
    let nid2 = n2.id();
    let n2 = make_node_ptr(Node::WindowNode(n2));

    let n3 = Window::default();
    let nid3 = n3.id();
    let n3 = make_node_ptr(Node::WindowNode(n3));

    let n4 = Window::default();
    let nid4 = n4.id();
    let n4 = make_node_ptr(Node::WindowNode(n4));

    let n5 = Cursor::default();
    let nid5 = n5.id();
    let n5 = make_node_ptr(Node::CursorNode(n5));

    tree.insert_root_node(nid1, n1.clone(), terminal_size);
    let shape1 = tree.get_shape(nid1);
    let pos1 = tree.get_pos(nid1);
    let size1 = tree.get_size(nid1);
    let actual_shape1 = tree.get_actual_shape(nid1);
    let actual_pos1 = tree.get_actual_pos(nid1);
    let actual_size1 = tree.get_actual_size(nid1);
    let expect1: (IRect, IPos, ISize, U16Rect, U16Pos, U16Size) = (
      IRect::new(
        (0, 0),
        (
          terminal_size.width() as isize,
          terminal_size.height() as isize,
        ),
      ),
      point!(x:0,y:0),
      ISize::new(
        terminal_size.width() as isize,
        terminal_size.height() as isize,
      ),
      U16Rect::new((0, 0), (terminal_size.width(), terminal_size.height())),
      U16Pos::new(0_u16, 0_u16),
      terminal_size,
    );
    info!(
      "expect-1:{:?}, shape/pos/size-1:{:?}/{:?}/{:?}, actual shape/pos/size-1:{:?}/{:?}/{:?}",
      expect1, shape1, pos1, size1, actual_shape1, actual_pos1, actual_size1,
    );
    assert!(*shape1.unwrap() == expect1.0);
    assert!(pos1.unwrap() == expect1.1);
    assert!(size1.unwrap() == expect1.2);
    assert!(*actual_shape1.unwrap() == expect1.3);
    assert!(actual_pos1.unwrap() == expect1.4);
    assert!(actual_size1.unwrap() == expect1.5);

    tree.insert_node(
      nid2,
      n2.clone(),
      nid1,
      IRect::new(
        (0, 0),
        (
          terminal_size.width() as isize,
          terminal_size.height() as isize,
        ),
      ),
    );
    let shape2 = tree.get_shape(nid2);
    let pos2 = tree.get_pos(nid2);
    let size2 = tree.get_size(nid2);
    let actual_shape2 = tree.get_actual_shape(nid2);
    let actual_pos2 = tree.get_actual_pos(nid2);
    let actual_size2 = tree.get_actual_size(nid2);
    let expect2: (IRect, IPos, ISize, U16Rect, U16Pos, U16Size) = (
      IRect::new(
        (0, 0),
        (
          terminal_size.width() as isize,
          terminal_size.height() as isize,
        ),
      ),
      point!(x:0, y:0),
      ISize::new(
        terminal_size.width() as isize,
        terminal_size.height() as isize,
      ),
      U16Rect::new((0, 0), (terminal_size.width(), terminal_size.height())),
      U16Pos::new(0_u16, 0_u16),
      terminal_size,
    );

    assert!(*shape2.unwrap() == expect2.0);
    assert!(pos2.unwrap() == expect2.1);
    assert!(size2.unwrap() == expect2.2);
    assert!(*actual_shape2.unwrap() == expect2.3);
    assert!(actual_pos2.unwrap() == expect2.4);
    assert!(actual_size2.unwrap() == expect2.5);

    tree.insert_node(nid3, n3.clone(), nid2, IRect::new((-2, -3), (4, 2)));
    let shape3 = tree.get_shape(nid3);
    let pos3 = tree.get_pos(nid3);
    let size3 = tree.get_size(nid3);
    let actual_shape3 = tree.get_actual_shape(nid3);
    let actual_pos3 = tree.get_actual_pos(nid3);
    let actual_size3 = tree.get_actual_size(nid3);
    let expect3: (IRect, IPos, ISize, U16Rect, U16Pos, U16Size) = (
      IRect::new((-2, -3), (4, 2)),
      point!(x:-2, y:-3),
      ISize::new(6, 5),
      U16Rect::new((0, 0), (4_u16, 2_u16)),
      U16Pos::new(0_u16, 0_u16),
      U16Size::new(4_u16, 2_u16),
    );
    assert!(*shape3.unwrap() == expect3.0);
    assert!(pos3.unwrap() == expect3.1);
    assert!(size3.unwrap() == expect3.2);
    assert!(*actual_shape3.unwrap() == expect3.3);
    assert!(actual_pos3.unwrap() == expect3.4);
    assert!(actual_size3.unwrap() == expect3.5);

    tree.insert_node(nid4, n4.clone(), nid2, IRect::new((5, 10), (15, 18)));
    let shape4 = tree.get_shape(nid4);
    let pos4 = tree.get_pos(nid4);
    let size4 = tree.get_size(nid4);
    let actual_shape4 = tree.get_actual_shape(nid4);
    let actual_pos4 = tree.get_actual_pos(nid4);
    let actual_size4 = tree.get_actual_size(nid4);
    let expect4: (IRect, IPos, ISize, U16Rect, U16Pos, U16Size) = (
      IRect::new((5, 10), (15, 18)),
      point!(x:5, y:10),
      ISize::new(10, 8),
      U16Rect::new((5, 10), (15_u16, 18_u16)),
      U16Pos::new(5_u16, 10_u16),
      U16Size::new(10_u16, 8_u16),
    );
    assert!(*shape4.unwrap() == expect4.0);
    assert!(pos4.unwrap() == expect4.1);
    assert!(size4.unwrap() == expect4.2);
    assert!(*actual_shape4.unwrap() == expect4.3);
    assert!(actual_pos4.unwrap() == expect4.4);
    assert!(actual_size4.unwrap() == expect4.5);

    tree.insert_node(nid5, n5.clone(), nid4, IRect::new((7, 3), (8, 4)));
    let shape5 = tree.get_shape(nid5);
    let pos5 = tree.get_pos(nid5);
    let size5 = tree.get_size(nid5);
    let actual_shape5 = tree.get_actual_shape(nid5);
    let actual_pos5 = tree.get_actual_pos(nid5);
    let actual_size5 = tree.get_actual_size(nid5);
    let expect5: (IRect, IPos, ISize, U16Rect, U16Pos, U16Size) = (
      IRect::new((7, 3), (8, 4)),
      point!(x:7, y:3),
      ISize::new(1, 1),
      U16Rect::new((12, 13), (13_u16, 14_u16)),
      U16Pos::new(12_u16, 13_u16),
      U16Size::new(1_u16, 1_u16),
    );
    assert!(*shape5.unwrap() == expect5.0);
    assert!(pos5.unwrap() == expect5.1);
    assert!(size5.unwrap() == expect5.2);
    assert!(*actual_shape5.unwrap() == expect5.3);
    assert!(actual_pos5.unwrap() == expect5.4);
    assert!(actual_size5.unwrap() == expect5.5);

    let expects: Vec<(IRect, IPos, ISize, U16Rect, U16Pos, U16Size)> =
      vec![expect1, expect2, expect3, expect4, expect5];

    let node_ids = [nid1, nid2, nid3, nid4];
    for (i, id) in node_ids.iter().enumerate() {
      let shape = tree.get_shape(*id);
      let pos = tree.get_pos(*id);
      let size = tree.get_size(*id);
      let actual_shape = tree.get_actual_shape(*id);
      let actual_pos = tree.get_actual_pos(*id);
      let actual_size = tree.get_actual_size(*id);
      let expect = expects[i];
      assert!(*shape.unwrap() == expect.0);
      assert!(pos.unwrap() == expect.1);
      assert!(size.unwrap() == expect.2);
      assert!(*actual_shape.unwrap() == expect.3);
      assert!(actual_pos.unwrap() == expect.4);
      // println!(
      //   "ui::tree::tree_shape1 i:{:?}, actual_size:{:?}, expect:{:?}",
      //   i, actual_size, expect
      // );
      assert!(actual_size.unwrap() == expect.5);
    }
  }
}
