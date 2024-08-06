//! The widget tree that manages all the widget components.

#![allow(dead_code)]

use parking_lot::Mutex;
use std::collections::BTreeSet;
use std::sync::{Arc, Weak};

use crate::cart::IRect;
use crate::ui::term::TerminalWk;
use crate::ui::tree::internal::inode::{Inode, InodeId};
use crate::ui::tree::internal::itree::{Itree, ItreeIterateOrder, ItreeIterator};
use crate::ui::widget::RootContainer;
use crate::ui::widget::{Widget, WidgetId, WidgetValue};

pub mod internal;

/// The widget tree.
///
/// The widget tree manages all UI components and rendering on the terminal, i.e. the whole
/// terminal is the root widget node, everything inside is the children nodes, and can recursively
/// go down.
///
/// Each widget node inside the tree can contain 0 or more children nodes.
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
///    default relative to its parent top-left corner, and the size is by default logically
///    infinite. While rendering to the terminal device, we need to calculate its absolute position
///    and actual size.
///
///    There're two kinds of positions:
///    * Relative: Based on it's parent's position.
///    * Absolute: Based on the terminal device.
///
///    There're two kinds of sizes:
///    * Logical: An infinite size on the imaginary canvas.
///    * Actual: An actual size bounded by it's parent's actual shape, if it doesn't have a parent,
///      bounded by the terminal device's actual shape.
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
///    overlap on each other, a widget with higher z-index has higher priority to be displayed. For
///    those widgets have the same z-index, the later inserted one will cover the previous inserted
///    ones.
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

  // Internal tree.
  base: Itree<WidgetValue>,

  // A collection of all VIM window container IDs
  // ([`WindowContainer`](crate::ui::widget::container::window::WindowContainer)).
  window_container_ids: BTreeSet<WidgetId>,
}

pub type TreeArc = Arc<Mutex<Tree>>;
pub type TreeWk = Weak<Mutex<Tree>>;
pub type TreeNode = Inode<WidgetValue>;
pub type TreeNodeId = InodeId;
pub type TreeIterator<'a> = ItreeIterator<'a, WidgetValue>;
pub type TreeIterateOrder = ItreeIterateOrder;

impl Tree {
  /// Make a widget tree.
  ///
  /// Note: The root node is created along with the tree.
  pub fn new(terminal: TerminalWk) -> Self {
    let terminal_size = terminal.upgrade().unwrap().lock().size();
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
      terminal,
      base: Itree::new(root_node),
      window_container_ids: BTreeSet::new(),
    }
  }

  pub fn to_arc(tree: Tree) -> TreeArc {
    Arc::new(Mutex::new(tree))
  }

  pub fn len(&self) -> usize {
    self.base.len()
  }

  /// Whether the tree is empty.
  pub fn is_empty(&self) -> bool {
    self.base.is_empty()
  }

  // Node {

  pub fn root_id(&self) -> TreeNodeId {
    self.base.root_id()
  }

  pub fn parent_id(&self, id: TreeNodeId) -> Option<&TreeNodeId> {
    self.base.parent_id(id)
  }

  pub fn children_ids(&self, id: TreeNodeId) -> Option<&Vec<TreeNodeId>> {
    self.base.children_ids(id)
  }

  pub fn node(&self, id: TreeNodeId) -> Option<&TreeNode> {
    self.base.node(id)
  }

  pub fn node_mut(&mut self, id: TreeNodeId) -> Option<&mut TreeNode> {
    self.base.node_mut(id)
  }

  pub fn iter(&self) -> TreeIterator {
    self.base.iter()
  }

  pub fn ordered_iter(&self, order: TreeIterateOrder) -> TreeIterator {
    self.base.ordered_iter(order)
  }

  pub fn insert(&mut self, parent_id: TreeNodeId, child_node: TreeNode) -> Option<&TreeNode> {
    match child_node.value() {
      WidgetValue::WindowContainer(w) => {
        let child_id = w.id();
        self.window_container_ids.insert(child_id);
      }
      _ => { /* Skip */ }
    }
    self.base.insert(parent_id, child_node)
  }

  pub fn remove(&mut self, id: TreeNodeId) -> Option<TreeNode> {
    self.base.remove(id)
  }

  // Node }

  pub fn window_container_ids(&self) -> &BTreeSet<WidgetId> {
    &self.window_container_ids
  }

  // Draw {

  /// Draw the widget tree to terminal device.
  pub fn draw(&mut self) {
    for node in self.base.iter() {
      let mut node_lock = node.lock();
      let actual_shape = *node_lock.actual_shape();
      node_lock
        .value_mut()
        .draw(actual_shape, self.terminal.clone());
    }
  }

  // Draw }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::cart::U16Size;
  use crate::test::log::init as test_log_init;
  use crate::ui::term::Terminal;
  use std::sync::Once;

  static INIT: Once = Once::new();

  #[test]
  fn new() {
    INIT.call_once(|| test_log_init());

    let sz = U16Size::new(18, 10);
    let t = Terminal::new(sz);
    let t = Terminal::to_arc(t);
    let tree = Tree::new(Arc::downgrade(&t));
    assert!(tree.window_container_ids().is_empty());
    assert!(tree.is_empty());
    assert!(tree.len() == 1);
  }
}
