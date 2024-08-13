//! The widget tree that manages all the widget components.

#![allow(dead_code)]

use geo::point;
use parking_lot::Mutex;
use std::collections::BTreeSet;
use std::sync::{Arc, Weak};
use tracing::debug;

use crate::cart::{IPos, IRect, U16Size};
use crate::ui::term::TerminalArc;
use crate::ui::tree::internal::inode::{Inode, InodeId};
use crate::ui::tree::internal::itree::{Itree, ItreeIter, ItreeIterMut};
use crate::ui::widget::RootContainer;
use crate::ui::widget::{Widget, WidgetId, WidgetValue};

pub mod internal;

#[derive(Debug, Clone)]
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
  // Internal tree.
  base: Itree<WidgetValue>,

  // A collection of all VIM window container
  // ([`WindowContainer`](crate::ui::widget::container::window::WindowContainer)) IDs.
  window_container_ids: BTreeSet<WidgetId>,

  // The cursor ID.
  cursor_id: Option<WidgetId>,
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
      window_container_ids: BTreeSet::new(),
      cursor_id: None,
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

  pub fn iter(&self) -> TreeIter {
    self.base.iter()
  }

  pub fn iter_mut(&mut self) -> TreeIterMut {
    self.base.iter_mut()
  }

  pub fn insert(&mut self, parent_id: TreeNodeId, child_node: TreeNode) -> Option<&TreeNode> {
    match child_node.value() {
      WidgetValue::WindowContainer(w) => {
        let widget_id = w.id();
        self.window_container_ids.insert(widget_id);
      }
      WidgetValue::Cursor(w) => {
        let widget_id = w.id();
        self.cursor_id = Some(widget_id);
      }
      _ => { /* Skip */ }
    }
    self.base.insert(parent_id, child_node)
  }

  pub fn remove(&mut self, id: TreeNodeId) -> Option<TreeNode> {
    if self.cursor_id == Some(id) {
      self.cursor_id = None;
    }
    if self.window_container_ids.contains(&id) {
      self.window_container_ids.remove(&id);
    }
    self.base.remove(id)
  }

  /// Bounded move by `(x, y)` (or `(columns, rows)`). When a widget hits the actual boundary of its
  /// parent, it simply stops moving.
  ///
  /// Note: This operation moves all the descendants together with the node.
  ///
  /// Fails if the widget doesn't exist.
  ///
  /// Returns the shape after movement.
  pub fn bounded_move_by(&mut self, id: InodeId, x: isize, y: isize) -> Option<IRect> {
    // Fix mutable borrow on `self.base.node_mut`.
    unsafe {
      let raw_base = &mut self.base as *mut Itree<WidgetValue>;

      match (*raw_base).parent_id(id) {
        Some(parent_id) => match (*raw_base).node(*parent_id) {
          Some(parent_node) => {
            match (*raw_base).node_mut(id) {
              Some(node) => {
                let parent_shape = *parent_node.shape();
                let parent_bottom_right_pos: IPos = parent_shape.max().into();

                let current_shape = *node.shape();
                let current_top_left_pos: IPos = current_shape.min().into();
                let next_top_left_pos: IPos =
                  point!(x: current_top_left_pos.x() + x, y: current_top_left_pos.y() + y);
                let expected_shape = IRect::new(
                  next_top_left_pos,
                  point!(x: next_top_left_pos.x() + current_shape.width(), y: next_top_left_pos.y() + current_shape.height()),
                );
                // Detect whether the expected shape is out of its parent's boundary.
                let expected_top_left_pos: IPos = expected_shape.min().into();
                let expected_bottom_right_pos: IPos = expected_shape.max().into();
                let final_x = if expected_top_left_pos.x() < 0 {
                  let x_diff = num_traits::sign::abs(expected_top_left_pos.x());
                  x + x_diff
                } else if expected_bottom_right_pos.x() >= parent_bottom_right_pos.x() {
                  let x_diff = num_traits::sign::abs_sub(
                    expected_top_left_pos.x(),
                    parent_bottom_right_pos.x(),
                  );
                  x - x_diff
                } else {
                  x
                };
                let final_y = if expected_top_left_pos.y() < 0 {
                  let y_diff = num_traits::sign::abs(expected_top_left_pos.y());
                  y + y_diff
                } else if expected_bottom_right_pos.y() >= parent_bottom_right_pos.y() {
                  let y_diff = num_traits::sign::abs_sub(
                    expected_top_left_pos.y(),
                    parent_bottom_right_pos.y(),
                  );
                  y - y_diff
                } else {
                  y
                };
                (*raw_base).move_by(id, final_x, final_y)
              }
              None => None,
            }
          }
          None => None,
        },
        None => None,
      }
    }
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

  pub fn window_container_ids(&self) -> &BTreeSet<WidgetId> {
    &self.window_container_ids
  }

  pub fn cursor_id(&self) -> Option<WidgetId> {
    self.cursor_id
  }

  // Draw {

  /// Draw the widget tree to terminal device.
  pub fn draw(&mut self, terminal: TerminalArc) {
    for node in self.base.iter_mut() {
      debug!("draw node:{:?}", node);
      let actual_shape = *node.actual_shape();
      node.value_mut().draw(actual_shape, terminal.clone());
    }
  }

  // Draw }
}

#[cfg(test)]
mod tests {
  use rand::prelude::*;
  use std::sync::Once;
  use tracing::info;

  use crate::cart::{IRect, U16Pos, U16Rect, U16Size};
  use crate::test::log::init as test_log_init;
  use crate::ui::tree::internal::inode::InodeValue;
  use crate::uuid;

  use super::*;

  static INIT: Once = Once::new();

  #[test]
  fn new() {
    INIT.call_once(test_log_init);

    let terminal_size = U16Size::new(18, 10);
    let tree = Tree::new(terminal_size);
    assert!(tree.window_container_ids().is_empty());
    assert!(tree.is_empty());
    assert!(tree.len() == 1);
  }

  #[derive(Copy, Clone, Debug, Default)]
  struct TestValue {
    id: InodeId,
    pub value: usize,
  }

  impl TestValue {
    pub fn new(value: usize) -> Self {
      TestValue {
        id: uuid::next(),
        value,
      }
    }
    pub fn value(&self) -> usize {
      self.value
    }
  }

  impl InodeValue for TestValue {
    fn id(&self) -> InodeId {
      self.id
    }
  }

  type TestNode = Inode<TestValue>;

  macro_rules! print_node {
    ($node: ident, $name: expr) => {
      loop {
        info!("{}: {:?}", $name, $node.clone());
        break;
      }
    };
  }

  #[test]
  fn bounded_move_by1() {
    INIT.call_once(test_log_init);

    let v1 = TestValue::new(1);
    let s1 = IRect::new((0, 0), (20, 20));
    let us1 = U16Rect::new((0, 0), (20, 20));
    let n1 = TestNode::new(v1, s1);
    let nid1 = n1.id();

    let v2 = TestValue::new(2);
    let s2 = IRect::new((0, 0), (20, 20));
    let us2 = U16Rect::new((0, 0), (20, 20));
    let n2 = TestNode::new(v2, s2);
    let nid2 = n2.id();

    let v3 = TestValue::new(3);
    let s3 = IRect::new((0, 0), (1, 1));
    let us3 = U16Rect::new((0, 0), (1, 1));
    let n3 = TestNode::new(v3, s3);
    let nid3 = n3.id();

    /*
     * The tree looks like:
     * ```
     *           n1
     *         /
     *        n2
     *       /
     *      n3
     * ```
     */
    let mut tree = Tree::new(n1);
    tree.insert(nid1, n2);
    tree.insert(nid2, n3);

    let n1 = tree.node(nid1).unwrap();
    let n2 = tree.node(nid2).unwrap();
    let n3 = tree.node(nid3).unwrap();
    print_node!(n1, "n1");
    print_node!(n2, "n2");
    print_node!(n3, "n3");

    let mut rng = rand::thread_rng();
    let count = 1000_usize;

    // Move: (x, y)
    let n3_moves = (0..count)
      .collect::<Vec<_>>()
      .iter()
      .map(|_i| (rng.gen_range(-1000..1000), rng.gen_range(-1000..1000)))
      .collect::<Vec<(isize, isize)>>();

    for m in n3_moves.iter() {
      let x = m.0;
      let y = m.1;
      let old_shape = *tree.node(nid3).unwrap().shape();
      let old_top_left_pos: IPos = old_shape.min().into();
      let old_bottom_right_pos: IPos = old_shape.max().into();
      let old_actual_shape = *tree.node(nid3).unwrap().actual_shape();
      let old_top_left_actual_pos: U16Pos = old_actual_shape.min().into();
      let old_bottom_right_actual_pos: U16Pos = old_actual_shape.max().into();
      tree.bounded_move_by(nid3, x, y);
      let new_shape = *tree.node(nid3).unwrap().shape();
      let new_top_left_pos: IPos = new_shape.min().into();
      let new_bottom_right_pos: IPos = new_shape.max().into();
      let new_actual_shape = *tree.node(nid3).unwrap().actual_shape();
      let new_top_left_actual_pos: U16Pos = new_actual_shape.min().into();
      let new_bottom_right_actual_pos: U16Pos = new_actual_shape.max().into();
      assert!(old_top_left_pos.x() + x == new_top_left_pos.x());
      assert!(old_top_left_pos.y() + y == new_top_left_pos.y());
      assert!(old_bottom_right_pos.x() + x == new_bottom_right_pos.x());
      assert!(old_bottom_right_pos.y() + y == new_bottom_right_pos.y());
      assert_eq!(new_shape.height(), old_shape.height());
      assert_eq!(new_shape.width(), old_shape.width());
      let parent_actual_shape = *tree.node(nid2).unwrap().actual_shape();
      let parent_top_left_actual_pos: U16Pos = parent_actual_shape.min().into();
      let parent_bottom_right_actual_pos: U16Pos = parent_actual_shape.max().into();
      assert!(old_top_left_actual_pos.x() >= parent_top_left_actual_pos.x());
      assert!(old_top_left_actual_pos.y() >= parent_top_left_actual_pos.y());
      assert!(old_bottom_right_actual_pos.x() <= parent_bottom_right_actual_pos.x());
      assert!(old_bottom_right_actual_pos.y() <= parent_bottom_right_actual_pos.y());
      assert!(new_top_left_actual_pos.x() >= parent_top_left_actual_pos.x());
      assert!(new_top_left_actual_pos.y() >= parent_top_left_actual_pos.y());
      assert!(new_bottom_right_actual_pos.x() <= parent_bottom_right_actual_pos.x());
      assert!(new_bottom_right_actual_pos.y() <= parent_bottom_right_actual_pos.y());
    }
  }
}
