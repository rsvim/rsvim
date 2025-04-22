//! The widget tree that manages all the widget components.

#![allow(dead_code)]

use crate::prelude::*;
use crate::ui::canvas::{Canvas, CanvasArc};
use crate::ui::widget::Widgetable;
use crate::ui::widget::cursor::Cursor;
use crate::ui::widget::root::RootContainer;
use crate::ui::widget::window::{
  Window, WindowGlobalOptions, WindowGlobalOptionsBuilder, WindowLocalOptions,
  WindowLocalOptionsBuilder,
};
use crate::{arc_impl, lock};

// Re-export
pub use crate::ui::tree::internal::{
  InodeBase, Inodeable, Itree, ItreeIter, /*ItreeIterMut,*/
  TreeNodeId,
};

use paste::paste;
use std::collections::BTreeSet;
// use tracing::trace;

pub mod internal;

#[derive(Debug, Clone)]
/// The value holder for each widget.
pub enum TreeNode {
  RootContainer(RootContainer),
  Window(Window),
  Cursor(Cursor),
}

macro_rules! tree_node_getter {
  ($self_name:ident,$method_name:ident) => {
    match $self_name {
      TreeNode::RootContainer(n) => n.$method_name(),
      TreeNode::Window(n) => n.$method_name(),
      TreeNode::Cursor(n) => n.$method_name(),
    }
  };
}

macro_rules! tree_node_setter {
  ($self_name:ident,$method_name:ident,$method_arg:ident) => {
    match $self_name {
      TreeNode::RootContainer(n) => n.$method_name($method_arg),
      TreeNode::Window(n) => n.$method_name($method_arg),
      TreeNode::Cursor(n) => n.$method_name($method_arg),
    }
  };
}

impl TreeNode {
  pub fn id(&self) -> TreeNodeId {
    match self {
      TreeNode::RootContainer(n) => n.id(),
      TreeNode::Window(n) => n.id(),
      TreeNode::Cursor(n) => n.id(),
    }
  }
}

impl Inodeable for TreeNode {
  fn id(&self) -> TreeNodeId {
    tree_node_getter!(self, id)
  }

  fn depth(&self) -> usize {
    tree_node_getter!(self, depth)
  }

  fn set_depth(&mut self, depth: usize) {
    tree_node_setter!(self, set_depth, depth)
  }

  fn zindex(&self) -> usize {
    tree_node_getter!(self, zindex)
  }

  fn set_zindex(&mut self, zindex: usize) {
    tree_node_setter!(self, set_zindex, zindex)
  }

  fn shape(&self) -> &IRect {
    tree_node_getter!(self, shape)
  }

  fn set_shape(&mut self, shape: &IRect) {
    tree_node_setter!(self, set_shape, shape)
  }

  fn actual_shape(&self) -> &U16Rect {
    tree_node_getter!(self, actual_shape)
  }

  fn set_actual_shape(&mut self, actual_shape: &U16Rect) {
    tree_node_setter!(self, set_actual_shape, actual_shape)
  }

  fn enabled(&self) -> bool {
    tree_node_getter!(self, enabled)
  }

  fn set_enabled(&mut self, enabled: bool) {
    tree_node_setter!(self, set_enabled, enabled)
  }

  fn visible(&self) -> bool {
    tree_node_getter!(self, visible)
  }

  fn set_visible(&mut self, visible: bool) {
    tree_node_setter!(self, set_visible, visible)
  }
}

impl Widgetable for TreeNode {
  /// Draw widget on the canvas.
  fn draw(&self, canvas: &mut Canvas) {
    match self {
      TreeNode::RootContainer(w) => w.draw(canvas),
      TreeNode::Window(w) => w.draw(canvas),
      TreeNode::Cursor(w) => w.draw(canvas),
    }
  }
}

#[derive(Debug, Clone)]
/// The widget tree.
///
/// The widget tree manages all UI components and rendering on the canvas, each widget is a tree
/// node on the widget tree, everything inside is the node's children. While the terminal itself is
/// the root widget node.
///
/// # Terms
///
/// * Parent: The parent node.
/// * Child: The child node.
/// * Ancestor: Either the parent, or the parent of some ancestor of the node.
/// * Descendant: Either the child, or the child of some descendant of the node.
/// * Sibling: Other children nodes under the same parent.
///
/// # Guarantees
///
/// ## Ownership
///
/// Parent owns all its children.
///
/// * Children will be destroyed when their parent is.
/// * Coordinate system are relative to their parent's top-left corner, while the absolute
///   coordinates are based on the terminal's top-left corner.
/// * Children are displayed inside their parent's geometric shape, clipped by boundaries. While
///   the size of each node can be logically infinite on the imaginary canvas.
/// * The `visible` and `enabled` attributes of a child are implicitly inherited from it's
///   parent, unless they're explicitly been set.
///
/// ## Priority
///
/// Children have higher priority than their parent to both display and process input events.
///
/// * Children are always displayed on top of their parent, and has higher priority to process
///   a user's input event when the event occurs within the shape of the child. The event will
///   fallback to their parent if the child doesn't process it.
/// * For children that shade each other, the one with higher z-index has higher priority to
///   display and process the input events.
///
/// # Attributes
///
/// ## Shape (position and size)
///
/// A shape can be relative/logical or absolute/actual, and always rectangle. The position is by
/// default relative to its parent top-left corner, and the size is by default logically
/// infinite. While rendering to the terminal device, we need to calculate its absolute position
/// and actual size.
///
/// There're two kinds of positions:
/// * Relative: Based on it's parent's position.
/// * Absolute: Based on the terminal device.
///
/// There're two kinds of sizes:
/// * Logical: An infinite size on the imaginary canvas.
/// * Actual: An actual size bounded by it's parent's actual shape, if it doesn't have a parent,
///   bounded by the terminal device's actual shape.
///
/// The shape boundary uses top-left open, bottom-right closed interval. For example the
/// terminal shape is `((0,0), (10,10))`, the top-left position `(0,0)` is inclusive, i.e.
/// inside the shape, the bottom-right position `(10,10)` is exclusive, i.e. outside the shape.
/// The width and height of the shape is both `10`.
///
/// The absolute/actual shape is calculated with a "copy-on-write" policy. Based on the fact
/// that a widget's shape is often read and rarely modified, thus the "copy-on-write" policy to
/// avoid too many duplicated calculations. i.e. we always calculates a widget's absolute
/// position and actual size right after it's shape is been changed, and also caches the result.
/// Thus we simply get the cached results when need.
///
/// ## Z-index
///
/// The z-index arranges the display priority of the content stack when multiple children
/// overlap on each other, a widget with higher z-index has higher priority to be displayed. For
/// those widgets have the same z-index, the later inserted one will cover the previous inserted
/// ones.
///
/// The z-index only works for the children under the same parent. For a child widget, it always
/// covers/overrides its parent display.
/// To change the visibility priority between children and parent, you need to change the
/// relationship between them.
///
/// For example, now we have two children under the same parent: A and B. A has 100 z-index, B
/// has 10 z-index. Now B has a child: C, with z-index 1000. Even the z-index 1000 > 100 > 10, A
/// still covers C, because it's a sibling of B.
///
/// ## Visible and enabled
///
/// A widget can be visible or invisible. When it's visible, it handles user's input events,
/// processes them and updates the UI contents. When it's invisible, it's just like not existed,
/// so it doesn't handle or process any input events, the UI hides.
///
/// A widget can be enabled or disabled. When it's enabled, it handles input events, processes
/// them and updates the UI contents. When it's disabled, it's just like been fronzen, so it
/// doesn't handle or process any input events, the UI keeps still and never changes.
///
pub struct Tree {
  // Internal implementation.
  base: Itree<TreeNode>,

  // Cursor and window state {

  // [`cursor`](crate::ui::widget::cursor::Cursor) node ID.
  cursor_id: Option<TreeNodeId>,

  // All [`Window`](crate::ui::widget::Window) node IDs.
  window_ids: BTreeSet<TreeNodeId>,
  // Cursor and window state }

  // Global options for windows.
  global_options: WindowGlobalOptions,

  // Global-local options for windows.
  global_local_options: WindowLocalOptions,
}

arc_impl!(Tree);

// pub type TreeIter<'a> = ItreeIter<'a, TreeNode>;
// pub type TreeIterMut<'a> = ItreeIterMut<'a, TreeNode>;

// Node {
impl Tree {
  /// Make a widget tree.
  ///
  /// NOTE: The root node is created along with the tree.
  pub fn new(canvas_size: U16Size) -> Self {
    let shape = IRect::new(
      (0, 0),
      (canvas_size.width() as isize, canvas_size.height() as isize),
    );
    let root_container = RootContainer::new(shape);
    let root_node = TreeNode::RootContainer(root_container);
    Tree {
      base: Itree::new(root_node),
      cursor_id: None,
      window_ids: BTreeSet::new(),
      global_options: WindowGlobalOptionsBuilder::default().build().unwrap(),
      global_local_options: WindowLocalOptionsBuilder::default().build().unwrap(),
    }
  }

  /// Nodes count, include the root node.
  pub fn len(&self) -> usize {
    self.base.len()
  }

  /// Whether the tree is empty.
  pub fn is_empty(&self) -> bool {
    self.base.is_empty()
  }

  /// Root node ID.
  pub fn root_id(&self) -> TreeNodeId {
    self.base.root_id()
  }

  /// All node IDs collection.
  pub fn node_ids(&self) -> Vec<TreeNodeId> {
    self.base.node_ids()
  }

  /// Get the parent ID by a node `id`.
  pub fn parent_id(&self, id: TreeNodeId) -> Option<TreeNodeId> {
    self.base.parent_id(id)
  }

  /// Get the children IDs by a node `id`.
  pub fn children_ids(&self, id: TreeNodeId) -> Vec<TreeNodeId> {
    self.base.children_ids(id)
  }

  /// Get the node struct by its `id`.
  pub fn node(&self, id: TreeNodeId) -> Option<&TreeNode> {
    self.base.node(id)
  }

  /// Get mutable node struct by its `id`.
  pub fn node_mut(&mut self, id: TreeNodeId) -> Option<&mut TreeNode> {
    self.base.node_mut(id)
  }

  // /// See [`Itree::iter`].
  // pub fn iter(&self) -> TreeIter {
  //   self.base.iter()
  // }
  //
  // /// See [`Itree::iter_mut`].
  // pub fn iter_mut(&mut self) -> TreeIterMut {
  //   self.base.iter_mut()
  // }

  /// Get current cursor node ID.
  pub fn cursor_id(&self) -> Option<TreeNodeId> {
    self.cursor_id
  }

  /// Set current cursor node ID.
  pub fn set_cursor_id(&mut self, cursor_id: Option<TreeNodeId>) {
    self.cursor_id = cursor_id;
  }

  /// Get current window node ID.
  /// NOTE: A window is called the current window because it has cursor inside it.
  pub fn current_window_id(&self) -> Option<TreeNodeId> {
    if let Some(cursor_id) = self.cursor_id {
      let mut id = cursor_id;
      while let Some(parent_id) = self.parent_id(id) {
        if let Some(TreeNode::Window(_w)) = self.node(parent_id) {
          return Some(parent_id);
        }
        id = parent_id;
      }
    }

    None
  }

  /// Get all the window widget IDs.
  pub fn window_ids(&self) -> &BTreeSet<TreeNodeId> {
    &self.window_ids
  }
}
// Node {

// Insert/Remove {
impl Tree {
  // This method handles some special requirements when insert a widget node:
  //
  // 1. When insert a cursor widget, it's parent widget must be a window widget.
  // 2. Maintain the cursor widget ID and window widget IDs when insert.
  fn insert_guard(&mut self, node: &TreeNode, parent_id: TreeNodeId) {
    match node {
      TreeNode::Cursor(cursor) => {
        // Ensure the parent node is a window widget.
        let parent_node = self.node(parent_id).unwrap();
        match parent_node {
          TreeNode::Window(_) => { /* Skip */ }
          _ => unreachable!("Cursor widget must insert under the window widget parent"),
        }
        self.cursor_id = Some(cursor.id());
      }
      TreeNode::Window(window) => {
        self.window_ids.insert(window.id());
      }
      _ => { /* Skip */ }
    }
  }

  // This method handles some special requirements when remove a widget node:
  //
  // 1. When insert a cursor widget, it's parent widget must be a window widget.
  // 2. Maintain the cursor widget ID and window widget IDs when remove.
  fn remove_guard(&mut self, id: TreeNodeId) {
    // If the removed ID is cursor ID, remove it.
    if self.cursor_id == Some(id) {
      self.cursor_id = None;
    }
    self.window_ids.remove(&id);
  }

  /// See [`Itree::insert`].
  pub fn insert(&mut self, parent_id: TreeNodeId, child_node: TreeNode) -> Option<TreeNode> {
    self.insert_guard(&child_node, parent_id);
    self.base.insert(parent_id, child_node)
  }

  /// See [`Itree::bounded_insert`].
  pub fn bounded_insert(
    &mut self,
    parent_id: TreeNodeId,
    child_node: TreeNode,
  ) -> Option<TreeNode> {
    self.insert_guard(&child_node, parent_id);
    self.base.bounded_insert(parent_id, child_node)
  }

  /// See [`Itree::remove`].
  pub fn remove(&mut self, id: TreeNodeId) -> Option<TreeNode> {
    self.remove_guard(id);
    self.base.remove(id)
  }
}
// Insert/Remove }

// Movement {
impl Tree {
  /// Bounded move by x(columns) and y(rows). This is simply a wrapper method on
  /// [`Itree::bounded_move_by`].
  pub fn bounded_move_by(&mut self, id: TreeNodeId, x: isize, y: isize) -> Option<IRect> {
    self.base.bounded_move_by(id, x, y)
  }

  /// Move by x(columns) and y(rows). This is simply a wrapper method on [`Itree::move_by`].
  pub fn move_by(&mut self, id: TreeNodeId, x: isize, y: isize) -> Option<IRect> {
    self.base.move_by(id, x, y)
  }

  /// Bounded move to position x(columns) and y(rows). This is simply a wrapper method on
  /// [`Itree::bounded_move_to`].
  pub fn bounded_move_to(&mut self, id: TreeNodeId, x: isize, y: isize) -> Option<IRect> {
    self.base.bounded_move_to(id, x, y)
  }

  /// Move to position x(columns) and y(rows). This is simply a wrapper method on
  /// [`Itree::move_to`].
  pub fn move_to(&mut self, id: TreeNodeId, x: isize, y: isize) -> Option<IRect> {
    self.base.move_to(id, x, y)
  }
}
// Movement }

// Global options {
impl Tree {
  pub fn global_options(&self) -> &WindowGlobalOptions {
    &self.global_options
  }

  pub fn global_options_mut(&mut self) -> &mut WindowGlobalOptions {
    &mut self.global_options
  }

  pub fn set_global_options(&mut self, options: &WindowGlobalOptions) {
    self.global_options = *options;
  }

  pub fn global_local_options(&self) -> &WindowLocalOptions {
    &self.global_local_options
  }

  pub fn global_local_options_mut(&mut self) -> &mut WindowLocalOptions {
    &mut self.global_local_options
  }

  pub fn set_global_local_options(&mut self, options: &WindowLocalOptions) {
    self.global_local_options = *options;
  }
}
// Global options }

// Draw {
impl Tree {
  /// Draw the widget tree to canvas.
  pub fn draw(&self, canvas: CanvasArc) {
    let mut canvas = lock!(canvas);
    for node in self.base.iter() {
      // trace!("Draw tree:{:?}", node);
      node.draw(&mut canvas);
    }
  }
}
// Draw }

#[cfg(test)]
mod tests {
  use crate::prelude::*;
  // use crate::test::log::init as test_log_init;

  use super::*;

  #[test]
  fn new() {
    // test_log_init();

    let terminal_size = U16Size::new(18, 10);
    let tree = Tree::new(terminal_size);
    assert!(tree.is_empty());
    assert!(tree.len() == 1);
  }
}
