//! The widget tree that manages all the widget components.

#![allow(dead_code)]

use crate::cart::{IRect, U16Rect, U16Size};
use crate::glovar;
use crate::ui::canvas::{Canvas, CanvasArc};
use crate::ui::tree::internal::{InodeId, Inodeable, Itree, ItreeIter, ItreeIterMut};
use crate::ui::widget::{Cursor, RootContainer, Widgetable, Window};

// Re-export
pub use crate::ui::tree::opt::{GlobalOptions, WindowGlobalOptions, WindowGlobalOptionsBuilder};

use parking_lot::RwLock;
use regex::Regex;
use std::collections::BTreeSet;
use std::sync::{Arc, Weak};
use tracing::debug;

pub mod internal;
pub mod opt;

#[derive(Debug, Clone)]
/// The value holder for each widget.
pub enum TreeNode {
  RootContainer(RootContainer),
  Window(Window),
  Cursor(Cursor),
}

macro_rules! tree_node_generate_dispatch {
  ($self_name:ident,$method_name:ident) => {
    match $self_name {
      TreeNode::RootContainer(n) => n.$method_name(),
      TreeNode::Window(n) => n.$method_name(),
      TreeNode::Cursor(n) => n.$method_name(),
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
  fn id(&self) -> InodeId {
    tree_node_generate_dispatch!(self, id)
  }

  fn depth(&self) -> &usize {
    tree_node_generate_dispatch!(self, depth)
  }

  fn depth_mut(&mut self) -> &mut usize {
    tree_node_generate_dispatch!(self, depth_mut)
  }

  fn zindex(&self) -> &usize {
    tree_node_generate_dispatch!(self, zindex)
  }

  fn zindex_mut(&mut self) -> &mut usize {
    tree_node_generate_dispatch!(self, zindex_mut)
  }

  fn shape(&self) -> &IRect {
    tree_node_generate_dispatch!(self, shape)
  }

  fn shape_mut(&mut self) -> &mut IRect {
    tree_node_generate_dispatch!(self, shape_mut)
  }

  fn actual_shape(&self) -> &U16Rect {
    tree_node_generate_dispatch!(self, actual_shape)
  }

  fn actual_shape_mut(&mut self) -> &mut U16Rect {
    tree_node_generate_dispatch!(self, actual_shape_mut)
  }

  fn enabled(&self) -> &bool {
    tree_node_generate_dispatch!(self, enabled)
  }

  fn enabled_mut(&mut self) -> &mut bool {
    tree_node_generate_dispatch!(self, enabled_mut)
  }

  fn visible(&self) -> &bool {
    tree_node_generate_dispatch!(self, visible)
  }

  fn visible_mut(&mut self) -> &mut bool {
    tree_node_generate_dispatch!(self, visible_mut)
  }
}

impl Widgetable for TreeNode {
  /// Draw widget on the canvas.
  fn draw(&mut self, canvas: &mut Canvas) {
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
  windows_ids: BTreeSet<TreeNodeId>,
  // Cursor and window state }

  // Global options for UI.
  options: GlobalOptions,
}

pub type TreeArc = Arc<RwLock<Tree>>;
pub type TreeWk = Weak<RwLock<Tree>>;
pub type TreeNodeId = InodeId;
pub type TreeIter<'a> = ItreeIter<'a, TreeNode>;
pub type TreeIterMut<'a> = ItreeIterMut<'a, TreeNode>;

impl Tree {
  /// Make a widget tree.
  ///
  /// NOTE: The root node is created along with the tree.
  pub fn new(terminal_size: U16Size) -> Self {
    let shape = IRect::new(
      (0, 0),
      (
        terminal_size.width() as isize,
        terminal_size.height() as isize,
      ),
    );
    let root_container = RootContainer::new(shape);
    let root_node = TreeNode::RootContainer(root_container);
    Tree {
      base: Itree::new(root_node),
      cursor_id: None,
      windows_ids: BTreeSet::new(),
      options: GlobalOptions::default(),
    }
  }

  /// Convert `Tree` struct to `Arc<RwLock<_>>` pointer.
  pub fn to_arc(tree: Tree) -> TreeArc {
    Arc::new(RwLock::new(tree))
  }

  /// Nodes count, include the root node.
  pub fn len(&self) -> usize {
    self.base.len()
  }

  /// Whether the tree is empty.
  pub fn is_empty(&self) -> bool {
    self.base.is_empty()
  }

  // Node {

  /// Root node ID.
  pub fn root_id(&self) -> TreeNodeId {
    self.base.root_id()
  }

  /// All node IDs collection.
  pub fn node_ids(&self) -> Vec<TreeNodeId> {
    self.base.node_ids()
  }

  /// Get the parent ID by a node `id`.
  pub fn parent_id(&self, id: &TreeNodeId) -> Option<&TreeNodeId> {
    self.base.parent_id(id)
  }

  /// Get the children IDs by a node `id`.
  pub fn children_ids(&self, id: &TreeNodeId) -> Option<&Vec<TreeNodeId>> {
    self.base.children_ids(id)
  }

  /// Get the node struct by its `id`.
  pub fn node(&self, id: &TreeNodeId) -> Option<&TreeNode> {
    self.base.node(id)
  }

  /// Get mutable node struct by its `id`.
  pub fn node_mut(&mut self, id: &TreeNodeId) -> Option<&mut TreeNode> {
    self.base.node_mut(id)
  }

  /// See [`Itree::iter`].
  pub fn iter(&self) -> TreeIter {
    self.base.iter()
  }

  /// See [`Itree::iter_mut`].
  pub fn iter_mut(&mut self) -> TreeIterMut {
    self.base.iter_mut()
  }

  fn insert_widget_ids(&mut self, node: &TreeNode) {
    match node {
      TreeNode::Cursor(n) => {
        self.cursor_id = Some(n.id());
      }
      TreeNode::Window(n) => {
        self.windows_ids.insert(n.id());
      }
      _ => { /* Skip */ }
    }
  }

  fn remove_window_widget_ids(&mut self, id: &TreeNodeId) {
    self.windows_ids.remove(id);
  }

  /// See [`Itree::insert`].
  pub fn insert(&mut self, parent_id: &TreeNodeId, child_node: TreeNode) -> Option<TreeNode> {
    self.insert_widget_ids(&child_node);
    self.base.insert(parent_id, child_node)
  }

  /// See [`Itree::bounded_insert`].
  pub fn bounded_insert(
    &mut self,
    parent_id: &TreeNodeId,
    child_node: TreeNode,
  ) -> Option<TreeNode> {
    self.insert_widget_ids(&child_node);
    self.base.bounded_insert(parent_id, child_node)
  }

  /// See [`Itree::remove`].
  pub fn remove(&mut self, id: TreeNodeId) -> Option<TreeNode> {
    self.remove_window_widget_ids(&id);
    self.base.remove(id)
  }

  /// See [`Itree::bounded_move_by`].
  pub fn bounded_move_by(&mut self, id: InodeId, x: isize, y: isize) -> Option<IRect> {
    self.base.bounded_move_by(id, x, y)
  }

  /// Bounded move by Y-axis (or `rows`). This is simply a wrapper method on
  /// [`bounded_move_by`](Tree::bounded_move_by).
  pub fn bounded_move_y_by(&mut self, id: InodeId, rows: isize) -> Option<IRect> {
    self.bounded_move_by(id, 0, rows)
  }

  /// Bounded move up by Y-axis (or `rows`). This is simply a wrapper method on
  /// [`bounded_move_by`](Tree::bounded_move_by).
  pub fn bounded_move_up_by(&mut self, id: InodeId, rows: usize) -> Option<IRect> {
    self.bounded_move_by(id, 0, -(rows as isize))
  }

  /// Bounded move down by Y-axis (or `rows`). This is simply a wrapper method on
  /// [`bounded_move_by`](Tree::bounded_move_by).
  pub fn bounded_move_down_by(&mut self, id: InodeId, rows: usize) -> Option<IRect> {
    self.bounded_move_by(id, 0, rows as isize)
  }

  /// Bounded move by X-axis (or `columns`). This is simply a wrapper method on
  /// [`bounded_move_by`](Tree::bounded_move_by).
  pub fn bounded_move_x_by(&mut self, id: InodeId, cols: isize) -> Option<IRect> {
    self.bounded_move_by(id, cols, 0)
  }

  /// Bounded move left by X-axis (or `columns`). This is simply a wrapper method on
  /// [`bounded_move_by`](Tree::bounded_move_by).
  pub fn bounded_move_left_by(&mut self, id: InodeId, cols: usize) -> Option<IRect> {
    self.bounded_move_by(id, -(cols as isize), 0)
  }

  /// Bounded move right by X-axis (or `columns`). This is simply a wrapper method on
  /// [`bounded_move_by`](Tree::bounded_move_by).
  pub fn bounded_move_right_by(&mut self, id: InodeId, cols: usize) -> Option<IRect> {
    self.bounded_move_by(id, cols as isize, 0)
  }

  // Node }

  // Cursor and Window {

  /// Get current cursor node ID.
  pub fn cursor_id(&self) -> Option<TreeNodeId> {
    self.cursor_id
  }

  /// Set current cursor node ID.
  pub fn set_cursor_id(&mut self, cursor_id: Option<TreeNodeId>) {
    self.cursor_id = cursor_id;
  }

  /// Get current window node ID. A window is current because the cursor is inside it.
  pub fn current_window_id(&self) -> Option<TreeNodeId> {
    if let Some(cursor_id) = self.cursor_id {
      let mut id = cursor_id;
      while let Some(parent_id) = self.parent_id(&id) {
        if let Some(TreeNode::Window(_w)) = self.node(parent_id) {
          return Some(*parent_id);
        }
        id = *parent_id;
      }
    }

    None
  }

  // Cursor and Window }

  // Global options {

  pub fn global_options(&self) -> &GlobalOptions {
    &self.options
  }

  pub fn global_options_mut(&mut self) -> &mut GlobalOptions {
    &mut self.options
  }

  pub fn wrap(&self) -> bool {
    self.options.window_local_options.wrap()
  }

  pub fn set_wrap(&mut self, value: bool) {
    self.options.window_local_options.set_wrap(value);
  }

  pub fn line_break(&self) -> bool {
    self.options.window_local_options.line_break()
  }

  pub fn set_line_break(&mut self, value: bool) {
    self.options.window_local_options.set_line_break(value);
  }

  pub fn breat_at(&self) -> &String {
    self.options.window_global_options.break_at()
  }

  pub fn set_break_at(&mut self, value: &str) {
    self.options.window_global_options.set_break_at(value);
  }

  pub fn break_at_regex(&self) -> &Regex {
    self.options.window_global_options.break_at_regex()
  }

  // Global options }

  // Draw {

  /// Draw the widget tree to canvas.
  pub fn draw(&mut self, canvas: CanvasArc) {
    let mut canvas = canvas.try_write_for(glovar::MUTEX_TIMEOUT()).unwrap();
    for node in self.base.iter_mut() {
      debug!("draw node:{:?}", node);
      node.draw(&mut canvas);
    }
  }

  // Draw }
}

#[cfg(test)]
mod tests {
  use crate::cart::U16Size;
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
