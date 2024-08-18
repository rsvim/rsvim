//! The widget tree that manages all the widget components.

#![allow(dead_code)]

use parking_lot::Mutex;
use std::sync::{Arc, Weak};
use std::time::Duration;
use tracing::debug;

use crate::cart::{IRect, U16Size};
use crate::glovar;
use crate::ui::canvas::CanvasArc;
use crate::ui::tree::internal::inode::{Inode, InodeId};
use crate::ui::tree::internal::itree::{Itree, ItreeIter, ItreeIterMut};
use crate::ui::widget::RootContainer;
use crate::ui::widget::{Widget, WidgetValue};

pub mod internal;

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
  base: Itree<WidgetValue>,
}

pub type TreeArc = Arc<Mutex<Tree>>;
pub type TreeWk = Weak<Mutex<Tree>>;
pub type TreeNode = Inode<WidgetValue>;
pub type TreeNodeId = InodeId;
pub type TreeIter<'a> = ItreeIter<'a, WidgetValue>;
pub type TreeIterMut<'a> = ItreeIterMut<'a, WidgetValue>;

impl Tree {
  /// Make a widget tree.
  ///
  /// Note: The root node is created along with the tree.
  pub fn new(terminal_size: U16Size) -> Self {
    let shape = IRect::new(
      (0, 0),
      (
        terminal_size.width() as isize,
        terminal_size.height() as isize,
      ),
    );
    let root_container = RootContainer::new();
    let root_node = TreeNode::new(WidgetValue::RootContainer(root_container), shape);
    Tree {
      base: Itree::new(root_node),
    }
  }

  /// Convert `Tree` struct to `Arc<Mutex<_>>` pointer.
  pub fn to_arc(tree: Tree) -> TreeArc {
    Arc::new(Mutex::new(tree))
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

  /// See [`Itree::insert`].
  pub fn insert(&mut self, parent_id: &TreeNodeId, child_node: TreeNode) -> Option<TreeNode> {
    self.base.insert(parent_id, child_node)
  }

  /// See [`Itree::bounded_insert`].
  pub fn bounded_insert(
    &mut self,
    parent_id: &TreeNodeId,
    child_node: TreeNode,
  ) -> Option<TreeNode> {
    self.base.bounded_insert(parent_id, child_node)
  }

  /// See [`Itree::remove`].
  pub fn remove(&mut self, id: TreeNodeId) -> Option<TreeNode> {
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

  // Draw {

  /// Draw the widget tree to canvas.
  pub fn draw(&mut self, canvas: CanvasArc) {
    let mut canvas = canvas
      .try_lock_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
      .unwrap();
    for node in self.base.iter_mut() {
      debug!("draw node:{:?}", node);
      let actual_shape = *node.actual_shape();
      node.value_mut().draw(actual_shape, &mut canvas);
    }
  }

  // Draw }
}

#[cfg(test)]
mod tests {
  use std::sync::Once;

  use crate::cart::U16Size;
  use crate::test::log::init as test_log_init;

  use super::*;

  static INIT: Once = Once::new();

  #[test]
  fn new() {
    INIT.call_once(test_log_init);

    let terminal_size = U16Size::new(18, 10);
    let tree = Tree::new(terminal_size);
    assert!(tree.is_empty());
    assert!(tree.len() == 1);
  }
}
