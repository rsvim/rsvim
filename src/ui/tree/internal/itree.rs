//! Internal tree structure that implements the widget tree.

#![allow(clippy::let_and_return)]

use geo::point;
use std::cmp::{max, min};
use std::collections::HashMap;
use std::fmt::Debug;
use std::{collections::VecDeque, iter::Iterator};
use tracing::debug;
// use tracing::debug;

use crate::cart::{IPos, IRect, ISize, U16Pos, U16Rect};
use crate::geo_point_as;
use crate::ui::tree::internal::inode::{Inode, InodeId, InodeValue};

#[derive(Debug, Default, Clone)]
pub struct Itree<T>
where
  T: InodeValue,
{
  // Root node ID.
  root_id: InodeId,
  // Nodes collection, maps from node ID to its node struct.
  nodes: HashMap<InodeId, Inode<T>>,
  // Maps from child ID to its parent ID.
  parent_ids: HashMap<InodeId, InodeId>,
  // Maps from parent ID to its children IDs, the children are sorted by zindex value from lower to higher.
  // For those children share the same zindex value, the one later inserted into the vector will be put in the back, thus it will be rendered later, i.e. it implicitly has a higher priority to show.
  children_ids: HashMap<InodeId, Vec<InodeId>>,
}

#[derive(Debug)]
/// The pre-order iterator of the tree.
///
/// For each node, it first visits the node itself, then visits all its children.
/// For all the children under the same parent, it visits from lower z-index to higher, thus the higher z-index ones will cover those lower ones.
/// This also follows the order when rendering the widget tree to terminal device.
pub struct ItreeIter<'a, T>
where
  T: InodeValue,
{
  tree: &'a Itree<T>,
  queue: VecDeque<&'a Inode<T>>,
}

impl<'a, T> Iterator for ItreeIter<'a, T>
where
  T: InodeValue,
{
  type Item = &'a Inode<T>;

  fn next(&mut self) -> Option<Self::Item> {
    if let Some(node) = self.queue.pop_front() {
      match self.tree.children_ids(node.id()) {
        Some(children_ids) => {
          for child_id in children_ids.iter() {
            match self.tree.node(*child_id) {
              Some(child) => {
                self.queue.push_back(child);
              }
              None => { /* Skip */ }
            }
          }
        }
        None => { /* Skip */ }
      }
      return Some(node);
    }
    None
  }
}

impl<'a, T> ItreeIter<'a, T>
where
  T: InodeValue,
{
  pub fn new(tree: &'a Itree<T>, start: Option<&'a Inode<T>>) -> Self {
    let mut queue = VecDeque::new();
    match start {
      Some(start) => queue.push_back(start),
      None => { /* Do nothing */ }
    }
    ItreeIter { tree, queue }
  }
}

#[derive(Debug)]
/// The mutable pre-order iterator of the tree.
pub struct ItreeIterMut<'a, T>
where
  T: InodeValue,
{
  tree: &'a mut Itree<T>,
  queue: VecDeque<&'a mut Inode<T>>,
}

impl<'a, T> Iterator for ItreeIterMut<'a, T>
where
  T: InodeValue,
{
  type Item = &'a mut Inode<T>;

  fn next(&mut self) -> Option<Self::Item> {
    if let Some(node) = self.queue.pop_front() {
      unsafe {
        let raw_tree = self.tree as *mut Itree<T>;
        match (*raw_tree).children_ids(node.id()) {
          Some(children_ids) => {
            for child_id in children_ids.iter() {
              match (*raw_tree).node_mut(*child_id) {
                Some(child) => {
                  self.queue.push_back(child);
                }
                None => { /* Skip */ }
              }
            }
          }
          None => { /* Skip */ }
        }
      } // unsafe
      return Some(node);
    }
    None
  }
}

impl<'a, T> ItreeIterMut<'a, T>
where
  T: InodeValue,
{
  pub fn new(tree: &'a mut Itree<T>, start: Option<&'a mut Inode<T>>) -> Self {
    let mut queue = VecDeque::new();
    match start {
      Some(start) => queue.push_back(start),
      None => { /* Do nothing */ }
    }
    ItreeIterMut { tree, queue }
  }
}

/// Convert (relative/logical) shape to actual shape, based on its parent's actual shape.
///
/// Note:
/// 1. If the widget doesn't have a parent, use the terminal shape as its parent's shape.
/// 2. If the relative/logical shape is outside of it's parent or the terminal, it will be
///    automatically bounded inside of it's parent or the terminal's shape.
fn convert_to_actual_shape(shape: IRect, parent_actual_shape: U16Rect) -> U16Rect {
  // debug!(
  //   "shape:{:?}, parent_actual_shape:{:?}",
  //   shape, parent_actual_shape
  // );
  let parent_actual_top_left_pos: U16Pos = parent_actual_shape.min().into();
  let parent_actual_top_left_ipos: IPos = geo_point_as!(parent_actual_top_left_pos, isize);
  let parent_actual_bottom_right_pos: U16Pos = parent_actual_shape.max().into();
  let parent_actual_bottom_right_ipos: IPos = geo_point_as!(parent_actual_bottom_right_pos, isize);

  let top_left_pos: IPos = shape.min().into();
  let bottom_right_pos: IPos = shape.max().into();

  let actual_top_left_ipos: IPos = top_left_pos + parent_actual_top_left_ipos;
  let actual_top_left_x = min(
    max(actual_top_left_ipos.x(), parent_actual_top_left_ipos.x()),
    parent_actual_bottom_right_ipos.x(),
  );
  let actual_top_left_y = min(
    max(actual_top_left_ipos.y(), parent_actual_top_left_ipos.y()),
    parent_actual_bottom_right_ipos.y(),
  );
  let actual_top_left_pos: U16Pos =
    point!(x: actual_top_left_x as u16, y: actual_top_left_y as u16);
  // debug!(
  //   "actual_top_left_ipos:{:?}, actual_top_left_pos:{:?}",
  //   actual_top_left_ipos, actual_top_left_pos
  // );

  let actual_bottom_right_ipos: IPos = bottom_right_pos + parent_actual_top_left_ipos;
  let actual_bottom_right_x = min(
    max(
      actual_bottom_right_ipos.x(),
      parent_actual_top_left_ipos.x(),
    ),
    parent_actual_bottom_right_ipos.x(),
  );
  let actual_bottom_right_y = min(
    max(
      actual_bottom_right_ipos.y(),
      parent_actual_top_left_ipos.y(),
    ),
    parent_actual_bottom_right_ipos.y(),
  );
  let actual_bottom_right_pos: U16Pos =
    point!(x: actual_bottom_right_x as u16, y: actual_bottom_right_y as u16);

  let actual_isize = ISize::new(
    (actual_bottom_right_pos.x() as isize) - (actual_top_left_pos.x() as isize),
    (actual_bottom_right_pos.y() as isize) - (actual_top_left_pos.y() as isize),
  );
  // debug!(
  //   "actual_isize:{:?}, actual_top_left_pos:{:?}",
  //   actual_isize, actual_top_left_pos
  // );
  let actual_shape = U16Rect::new(
    actual_top_left_pos,
    point!(x: actual_top_left_pos.x() + actual_isize.width() as u16, y: actual_top_left_pos.y() + actual_isize.height() as u16),
  );
  // debug!(
  //   "actual_isize:{:?}, actual_shape:{:?}",
  //   actual_isize, actual_shape
  // );

  actual_shape
}

impl<T> Itree<T>
where
  T: InodeValue,
{
  pub fn new(root_node: Inode<T>) -> Self {
    let root_id = root_node.id();
    let mut nodes = HashMap::new();
    nodes.insert(root_id, root_node);
    let mut children_ids: HashMap<InodeId, Vec<InodeId>> = HashMap::new();
    children_ids.insert(root_id, vec![]);
    Itree {
      root_id,
      nodes,
      parent_ids: HashMap::new(),
      children_ids,
    }
  }

  pub fn len(&self) -> usize {
    self.nodes.len()
  }

  pub fn is_empty(&self) -> bool {
    self.nodes.len() <= 1
  }

  pub fn root_id(&self) -> InodeId {
    self.root_id
  }

  pub fn node_ids(&self) -> Vec<InodeId> {
    self.nodes.keys().copied().collect()
  }

  pub fn parent_id(&self, id: InodeId) -> Option<&InodeId> {
    self.parent_ids.get(&id)
  }

  pub fn children_ids(&self, id: InodeId) -> Option<&Vec<InodeId>> {
    self.children_ids.get(&id)
  }

  pub fn node(&self, id: InodeId) -> Option<&Inode<T>> {
    self.nodes.get(&id)
  }

  pub fn node_mut(&mut self, id: InodeId) -> Option<&mut Inode<T>> {
    self.nodes.get_mut(&id)
  }

  /// Get the iterator.
  ///
  /// By default, it iterates in pre-order iterator which starts from the root.
  /// For the children under the same node, it visits from lower z-index to higher.
  pub fn iter(&self) -> ItreeIter<T> {
    ItreeIter::new(self, Some(self.nodes.get(&self.root_id).unwrap()))
  }

  /// Get the iterator that returns mutable reference.
  pub fn iter_mut(&mut self) -> ItreeIterMut<T> {
    unsafe {
      let raw_nodes = &mut self.nodes as *mut HashMap<InodeId, Inode<T>>;
      ItreeIterMut::new(self, Some((*raw_nodes).get_mut(&self.root_id).unwrap()))
    }
  }

  /// Update the `start_id` node attributes, and all the descendants attributes of this node.
  ///
  /// Below attributes will be update:
  ///
  /// 1. [`depth`](Inode::depth()): The child depth should be always the parent depth + 1.
  /// 2. [`actual_shape`](Inode::actual_shape()): The child actual shape should be always be clipped by parent's boundaries.
  unsafe fn update_descendant_attributes(&mut self, start_id: InodeId, start_parent_id: InodeId) {
    // Create the queue of parent-child ID pairs, to iterate all descendants under the child node.

    // Tuple of (child, parent id, parent depth, parent actual shape)
    type ChildAndParentPair<'a, T> = (&'a mut Inode<T>, InodeId, usize, U16Rect);

    // Avoid the multiple mutable references on `self.nodes.get_mut` when updating all descendants attributes.
    let raw_nodes = &mut self.nodes as *mut HashMap<InodeId, Inode<T>>;

    // debug!("before create que");
    let mut que: VecDeque<ChildAndParentPair<T>> = VecDeque::new();
    let pnode = (*raw_nodes).get(&start_parent_id).unwrap();
    let pnode_id = pnode.id();
    let pnode_depth = *pnode.depth();
    let pnode_actual_shape = *pnode.actual_shape();
    que.push_back((
      (*raw_nodes).get_mut(&start_id).unwrap(),
      pnode_id,
      pnode_depth,
      pnode_actual_shape,
    ));
    // debug!("after create que");

    // Iterate all descendants, and update their attributes.
    while let Some(child_and_parent) = que.pop_front() {
      let cnode = child_and_parent.0;
      let pnode_id = child_and_parent.1;
      let pnode_depth = child_and_parent.2;
      let pnode_actual_shape = child_and_parent.3;

      // debug!("before update cnode attr: {:?}", cnode);
      let cnode_id = cnode.id();
      let cnode_depth = pnode_depth + 1;
      let cnode_shape = *cnode.shape();
      let cnode_actual_shape = convert_to_actual_shape(cnode_shape, pnode_actual_shape);

      debug!("update attr, cnode:{:?}, depth:{:?}, actual shape:{:?}, pnode:{:?}, depth:{:?}, actual shape:{:?}", cnode_id, cnode_depth, cnode_actual_shape, pnode_id, pnode_depth, pnode_actual_shape);
      *cnode.depth_mut() = cnode_depth;
      *cnode.actual_shape_mut() = cnode_actual_shape;
      // debug!("after update cnode attr: {:?}", cnode_id);

      // debug!(
      //   "before push descendant_ids, cnode_id:{:?}, children_ids: {:?}",
      //   cnode_id, self.children_ids
      // );
      match self.children_ids.get(&cnode_id) {
        Some(descendant_ids) => {
          for dnode_id in descendant_ids.iter() {
            // debug!("before push dnode: {:?}", dnode_id);
            match (*raw_nodes).get_mut(dnode_id) {
              Some(dnode) => {
                que.push_back((dnode, cnode_id, cnode_depth, cnode_actual_shape));
              }
              None => { /* Skip */ }
            }
            // debug!("after push dnode: {:?}", dnode_id);
          }
        }
        None => { /* Skip */ }
      }
    }
  }

  /// Insert a node to the tree, i.e. push it to the children vector of the parent.
  ///
  /// This operation builds the connection between the parent and the inserted child.
  ///
  /// It also sorts the children vector after inserted by the z-index value,
  /// and updates both the inserted child's attributes and all its descendants attributes.
  ///
  /// Below node attributes need to update:
  ///
  /// 1. [`depth`](Inode::depth()): The child depth should be always the parent depth + 1.
  /// 2. [`actual_shape`](Inode::actual_shape()): The child actual shape should be always be clipped by parent's boundaries.
  ///
  /// Fails if:
  ///
  /// 1. The `parent_id` doesn't exist.
  pub fn insert(&mut self, parent_id: InodeId, child_node: Inode<T>) -> Option<&Inode<T>> {
    // Returns `None` if `parent_id` not exists.
    self.nodes.get(&parent_id)?;

    debug!(
      "parent_id:{:?}, node_ids:{:?}, children_ids:{:?}",
      parent_id,
      self.node_ids(),
      self.children_ids
    );
    assert!(
      self.children_ids.contains_key(&parent_id),
      "children_ids {:?} doesn't contains parent_id {:?}",
      self.children_ids,
      parent_id
    );

    // Insert node.
    let child_id = child_node.id();
    let child_zindex = *child_node.zindex();
    self.nodes.insert(child_id, child_node);
    self.children_ids.insert(child_id, vec![]);

    // Map child ID => parent ID.
    self.parent_ids.insert(child_id, parent_id);
    // Map parent ID => children IDs.
    // It inserts child ID to the `children_ids` vector of the parent, sorted by the z-index.
    // For the children that have the same z-index value, it inserts at the end of those children.
    // debug!("before get higher zindex pos");
    let higher_zindex_pos: Vec<usize> = self
      .children_ids
      .get(&parent_id)
      .unwrap()
      .iter()
      .enumerate()
      .filter(|(_index, cid)| match self.nodes.get(cid) {
        Some(cnode) => *cnode.zindex() > child_zindex,
        None => false,
      })
      .map(|(index, _cid)| index)
      .collect();
    // debug!("after get higher zindex pos");
    match higher_zindex_pos.first() {
      Some(insert_pos) => {
        self
          .children_ids
          .get_mut(&parent_id)
          .unwrap()
          .insert(*insert_pos, child_id);
      }
      None => {
        self
          .children_ids
          .get_mut(&parent_id)
          .unwrap()
          .push(child_id);
      }
    }

    // Update all the descendants attributes under the `child_id` node.
    unsafe {
      self.update_descendant_attributes(child_id, parent_id);
    } // unsafe

    // Return the inserted child
    self.nodes.get(&child_id)
  }

  /// Remove a node by its ID.
  ///
  /// This operation breaks the connection between the removed node and its parent.
  ///
  /// But the relationships between the removed node and its descendants still remains in the tree,
  /// thus once you insert it back in the same tree, its descendants are still connected with the removed node.
  ///
  /// Fails if:
  /// 1. The removed node doesn't exist.
  /// 2. The removed node is the root node.
  pub fn remove(&mut self, id: InodeId) -> Option<Inode<T>> {
    // Cannot remove root node.
    if id == self.root_id {
      return None;
    }
    // Remove child from nodes collection.
    match self.nodes.remove(&id) {
      Some(removed) => {
        // Remove child `id` => parent ID mapping.
        self.parent_ids.remove(&id);
        Some(removed)
      }
      None => None,
    }
  }

  /// Move node by (x, y).
  /// When x < 0, the node moves up. When x > 0, the node moves down.
  /// When y < 0, the node moves left. When y > 0, the node moves right.
  ///
  /// Fails if the node doesn't exist.
  ///
  /// Returns the previous shape if move successfully.
  pub fn move_by(&mut self, id: InodeId, x: isize, y: isize) -> Option<IRect> {
    match self.nodes.get_mut(&id) {
      Some(node) => {
        let current_shape = *node.shape();
        let current_top_left_pos: IPos = current_shape.min().into();
        let next_top_left_pos: IPos =
          point!(x: current_top_left_pos.x() + x, y: current_top_left_pos.y() + y);
        let next_shape = IRect::new(
          next_top_left_pos,
          point!(x: next_top_left_pos.x() + current_shape.width(), y: next_top_left_pos.y() + current_shape.height()),
        );
        *node.shape_mut() = next_shape;

        // Update all the descendants attributes under the `id` node.
        unsafe {
          self.update_descendant_attributes(id, *self.parent_ids.get(&id).unwrap());
        }

        Some(current_shape)
      }
      None => None,
    }
  }
}

#[cfg(test)]
mod tests {
  use rand::prelude::*;
  use std::cmp::min;
  use std::sync::Once;
  use tracing::info;

  use crate::cart::{IRect, U16Rect};
  use crate::test::log::init as test_log_init;
  use crate::ui::tree::internal::inode::InodeValue;
  use crate::uuid;

  static INIT: Once = Once::new();

  use super::*;

  #[test]
  fn convert_to_actual_shapes1() {
    INIT.call_once(test_log_init);

    let inputs: Vec<IRect> = vec![
      IRect::new((0, 0), (3, 5)),
      IRect::new((0, 0), (1, 5)),
      IRect::new((0, 0), (3, 7)),
      IRect::new((0, 0), (0, 0)),
      IRect::new((0, 0), (5, 4)),
    ];
    for t in inputs.iter() {
      for p in 0..10 {
        for q in 0..10 {
          let input_actual_parent_shape = U16Rect::new((0, 0), (p as u16, q as u16));
          let expect = U16Rect::new((0, 0), (min(t.max().x, p) as u16, min(t.max().y, q) as u16));
          let actual = convert_to_actual_shape(*t, input_actual_parent_shape);
          info!("expect:{:?}, actual:{:?}", expect, actual);
          assert_eq!(actual, expect);
        }
      }
    }
  }

  #[test]
  fn convert_to_actual_shapes2() {
    INIT.call_once(test_log_init);

    let inputs: Vec<(IRect, U16Rect)> = vec![
      (IRect::new((0, 0), (3, 5)), U16Rect::new((0, 0), (7, 8))),
      (IRect::new((-3, 1), (1, 5)), U16Rect::new((3, 2), (9, 8))),
      (IRect::new((3, 9), (6, 10)), U16Rect::new((1, 1), (2, 2))),
      (IRect::new((0, 0), (0, 0)), U16Rect::new((0, 0), (0, 0))),
      (IRect::new((5, 3), (6, 4)), U16Rect::new((0, 0), (5, 3))),
    ];
    let expects: Vec<U16Rect> = vec![
      U16Rect::new((0, 0), (3, 5)),
      U16Rect::new((3, 3), (4, 7)),
      U16Rect::new((2, 2), (2, 2)),
      U16Rect::new((0, 0), (0, 0)),
      U16Rect::new((5, 3), (5, 3)),
    ];
    for (i, p) in inputs.iter().enumerate() {
      let actual = convert_to_actual_shape(p.0, p.1);
      let expect = expects[i];
      info!(
        "i:{:?}, input:{:?}, actual:{:?}, expect:{:?}",
        i, p, actual, expect
      );
      assert_eq!(actual, expect);
    }
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

  macro_rules! assert_node_id_eq {
    ($node: ident, $id: ident) => {
      loop {
        assert!($node.id() == $id);
        break;
      }
    };
  }

  macro_rules! print_node {
    ($node: ident, $name: expr) => {
      loop {
        info!("{}: {:?}", $name, $node.clone());
        break;
      }
    };
  }

  macro_rules! assert_parent_child_nodes_depth {
    ($parent: ident, $child: ident) => {
      loop {
        assert_eq!(*$parent.depth() + 1, *$child.depth());
        break;
      }
    };
  }

  macro_rules! assert_node_actual_shape_eq {
    ($node: ident, $expect: expr, $index: expr) => {
      loop {
        assert_eq!(*$node.actual_shape(), $expect, "index:{:?}", $index,);
        break;
      }
    };
  }

  macro_rules! assert_node_value_eq {
    ($node: ident, $expect: expr) => {
      loop {
        assert_eq!($node.value().value, $expect);
        break;
      }
    };
  }

  #[test]
  fn new() {
    INIT.call_once(test_log_init);

    let v1 = TestValue::new(1);
    let s1 = IRect::new((0, 0), (1, 1));
    let n1 = TestNode::new(v1, s1);
    let nid1 = n1.id();
    let tree = Itree::new(n1);

    assert_eq!(tree.len(), 1);
    assert_eq!(tree.root_id(), nid1);
    assert!(tree.parent_id(nid1).is_none());
    assert!(tree.children_ids(nid1).is_some());
    assert!(tree.children_ids(nid1).unwrap().is_empty());

    for node in tree.iter() {
      assert_node_id_eq!(node, nid1);
    }
  }

  #[test]
  fn insert1() {
    INIT.call_once(test_log_init);

    let v1 = TestValue::new(1);
    let s1 = IRect::new((0, 0), (1, 1));
    let n1 = TestNode::new(v1, s1);
    let nid1 = n1.id();

    let v2 = TestValue::new(2);
    let s2 = IRect::new((0, 0), (1, 1));
    let n2 = TestNode::new(v2, s2);
    let nid2 = n2.id();

    let v3 = TestValue::new(3);
    let s3 = IRect::new((0, 0), (1, 1));
    let n3 = TestNode::new(v3, s3);
    let nid3 = n3.id();

    let v4 = TestValue::new(4);
    let s4 = IRect::new((0, 0), (1, 1));
    let n4 = TestNode::new(v4, s4);
    let nid4 = n4.id();

    let v5 = TestValue::new(5);
    let s5 = IRect::new((0, 0), (1, 1));
    let n5 = TestNode::new(v5, s5);
    let nid5 = n5.id();

    let v6 = TestValue::new(6);
    let s6 = IRect::new((0, 0), (1, 1));
    let n6 = TestNode::new(v6, s6);
    let nid6 = n6.id();

    /*
     * The tree looks like:
     * ```
     *           n1
     *         /   \
     *        n2   n3
     *      /  \     \
     *     n4  n5    n6
     * ```
     */
    let mut tree = Itree::new(n1);
    tree.insert(nid1, n2);
    tree.insert(nid1, n3);
    tree.insert(nid2, n4);
    tree.insert(nid2, n5);
    tree.insert(nid3, n6);

    assert!(tree.root_id() == nid1);
    let n1 = tree.node(nid1).unwrap();
    let n2 = tree.node(nid2).unwrap();
    let n3 = tree.node(nid3).unwrap();
    let n4 = tree.node(nid4).unwrap();
    let n5 = tree.node(nid5).unwrap();
    let n6 = tree.node(nid6).unwrap();
    print_node!(n1, "n1");
    print_node!(n2, "n2");
    print_node!(n3, "n3");
    print_node!(n4, "n4");
    print_node!(n5, "n5");
    print_node!(n6, "n6");

    assert_eq!(nid1 + 1, nid2);
    assert_eq!(nid2 + 1, nid3);
    assert_eq!(nid3 + 1, nid4);
    assert_eq!(nid4 + 1, nid5);
    assert_eq!(nid5 + 1, nid6);

    assert_parent_child_nodes_depth!(n1, n2);
    assert_parent_child_nodes_depth!(n1, n3);
    assert_parent_child_nodes_depth!(n2, n4);
    assert_parent_child_nodes_depth!(n2, n5);
    assert_parent_child_nodes_depth!(n2, n6);
    assert_parent_child_nodes_depth!(n3, n6);

    assert_eq!(tree.children_ids(nid1).unwrap().len(), 2);
    assert_eq!(tree.children_ids(nid2).unwrap().len(), 2);
    assert_eq!(tree.children_ids(nid3).unwrap().len(), 1);
    assert_eq!(tree.children_ids(nid4).unwrap().len(), 0);
    assert_eq!(tree.children_ids(nid5).unwrap().len(), 0);
    assert_eq!(tree.children_ids(nid6).unwrap().len(), 0);

    let contains_child = |parent_id: InodeId, child_id: InodeId| -> bool {
      match tree.children_ids(parent_id) {
        Some(children_ids) => {
          children_ids
            .iter()
            .filter(|cid| **cid == child_id)
            .collect::<Vec<_>>()
            .len()
            == 1
        }
        None => false,
      }
    };

    assert!(contains_child(nid1, nid2));
    assert!(contains_child(nid1, nid3));
    assert!(!contains_child(nid1, nid4));
    assert!(!contains_child(nid1, nid5));
    assert!(!contains_child(nid1, nid6));

    assert!(contains_child(nid2, nid4));
    assert!(contains_child(nid2, nid5));
    assert!(!contains_child(nid2, nid6));

    assert!(contains_child(nid3, nid6));
    assert!(!contains_child(nid3, nid4));
    assert!(!contains_child(nid3, nid5));
  }

  #[test]
  fn insert2() {
    INIT.call_once(test_log_init);

    let v1 = TestValue::new(1);
    let s1 = IRect::new((0, 0), (20, 20));
    let n1 = TestNode::new(v1, s1);
    let nid1 = n1.id();

    let v2 = TestValue::new(2);
    let s2 = IRect::new((0, 0), (15, 15));
    let n2 = TestNode::new(v2, s2);
    let nid2 = n2.id();

    let v3 = TestValue::new(3);
    let s3 = IRect::new((10, 10), (18, 19));
    let n3 = TestNode::new(v3, s3);
    let nid3 = n3.id();

    let v4 = TestValue::new(4);
    let s4 = IRect::new((3, 5), (20, 14));
    let n4 = TestNode::new(v4, s4);
    let nid4 = n4.id();

    let v5 = TestValue::new(5);
    let s5 = IRect::new((-3, -5), (10, 20));
    let n5 = TestNode::new(v5, s5);
    let nid5 = n5.id();

    let v6 = TestValue::new(6);
    let s6 = IRect::new((3, 6), (6, 10));
    let n6 = TestNode::new(v6, s6);
    let nid6 = n6.id();

    let v7 = TestValue::new(7);
    let s7 = IRect::new((3, 6), (15, 25));
    let n7 = TestNode::new(v7, s7);
    let nid7 = n7.id();

    let v8 = TestValue::new(8);
    let s8 = IRect::new((-1, -2), (2, 1));
    let n8 = TestNode::new(v8, s8);
    let nid8 = n8.id();

    let v9 = TestValue::new(9);
    let s9 = IRect::new((5, 6), (9, 8));
    let n9 = TestNode::new(v9, s9);
    let nid9 = n9.id();

    /*
     * The tree looks like:
     * ```
     *           n1
     *         /   \
     *        n2   n3
     *      /  \     \
     *     n4  n5    n6
     *           \
     *            n7
     *           / \
     *         n8   n9
     * ```
     */
    let mut tree = Itree::new(n1);
    tree.insert(nid1, n2);
    tree.insert(nid1, n3);
    tree.insert(nid2, n4);
    tree.insert(nid2, n5);
    tree.insert(nid3, n6);
    tree.insert(nid5, n7);
    tree.insert(nid7, n8);
    tree.insert(nid7, n9);

    assert!(tree.root_id() == nid1);
    let n1 = tree.node(nid1).unwrap();
    let n2 = tree.node(nid2).unwrap();
    let n3 = tree.node(nid3).unwrap();
    let n4 = tree.node(nid4).unwrap();
    let n5 = tree.node(nid5).unwrap();
    let n6 = tree.node(nid6).unwrap();
    let n7 = tree.node(nid7).unwrap();
    let n8 = tree.node(nid8).unwrap();
    let n9 = tree.node(nid9).unwrap();
    print_node!(n1, "n1");
    print_node!(n2, "n2");
    print_node!(n3, "n3");
    print_node!(n4, "n4");
    print_node!(n5, "n5");
    print_node!(n6, "n6");
    print_node!(n7, "n7");
    print_node!(n8, "n8");
    print_node!(n9, "n9");

    assert_eq!(nid1 + 1, nid2);
    assert_eq!(nid2 + 1, nid3);
    assert_eq!(nid3 + 1, nid4);
    assert_eq!(nid4 + 1, nid5);
    assert_eq!(nid5 + 1, nid6);
    assert_eq!(nid6 + 1, nid7);
    assert_eq!(nid7 + 1, nid8);
    assert_eq!(nid8 + 1, nid9);

    assert_parent_child_nodes_depth!(n1, n2);
    assert_parent_child_nodes_depth!(n1, n3);
    assert_parent_child_nodes_depth!(n2, n4);
    assert_parent_child_nodes_depth!(n2, n5);
    assert_parent_child_nodes_depth!(n2, n6);
    assert_parent_child_nodes_depth!(n3, n6);
    assert_parent_child_nodes_depth!(n5, n7);
    assert_parent_child_nodes_depth!(n7, n8);
    assert_parent_child_nodes_depth!(n7, n9);

    assert_eq!(tree.children_ids(nid1).unwrap().len(), 2);
    assert_eq!(tree.children_ids(nid2).unwrap().len(), 2);
    assert_eq!(tree.children_ids(nid3).unwrap().len(), 1);
    assert_eq!(tree.children_ids(nid4).unwrap().len(), 0);
    assert_eq!(tree.children_ids(nid5).unwrap().len(), 1);
    assert_eq!(tree.children_ids(nid6).unwrap().len(), 0);
    assert_eq!(tree.children_ids(nid7).unwrap().len(), 2);
    assert_eq!(tree.children_ids(nid8).unwrap().len(), 0);
    assert_eq!(tree.children_ids(nid9).unwrap().len(), 0);

    let contains_child = |parent_id: InodeId, child_id: InodeId| -> bool {
      let result = match tree.children_ids(parent_id) {
        Some(children_ids) => {
          children_ids
            .iter()
            .filter(|cid| **cid == child_id)
            .collect::<Vec<_>>()
            .len()
            == 1
        }
        None => false,
      };
      info!(
        "parent: {:?}, child: {:?}, children_ids: {:?}, contains: {:?}",
        parent_id,
        child_id,
        tree.children_ids(parent_id),
        result
      );
      result
    };

    assert!(contains_child(nid1, nid2));
    assert!(contains_child(nid1, nid3));
    assert!(!contains_child(nid1, nid4));
    assert!(!contains_child(nid1, nid5));
    assert!(!contains_child(nid1, nid7));

    assert!(contains_child(nid2, nid4));
    assert!(contains_child(nid2, nid5));
    assert!(!contains_child(nid2, nid7));

    assert!(contains_child(nid3, nid6));
    assert!(!contains_child(nid3, nid7));
    assert!(!contains_child(nid3, nid4));
    assert!(!contains_child(nid3, nid5));

    assert!(contains_child(nid5, nid7));
    assert!(contains_child(nid7, nid8));
    assert!(contains_child(nid7, nid9));
  }

  #[test]
  fn shape1() {
    INIT.call_once(test_log_init);

    let v1 = TestValue::new(1);
    let s1 = IRect::new((0, 0), (20, 20));
    let us1 = U16Rect::new((0, 0), (20, 20));
    let n1 = TestNode::new(v1, s1);
    let nid1 = n1.id();

    let v2 = TestValue::new(2);
    let s2 = IRect::new((0, 0), (15, 15));
    let us2 = U16Rect::new((0, 0), (15, 15));
    let n2 = TestNode::new(v2, s2);
    let nid2 = n2.id();

    let v3 = TestValue::new(3);
    let s3 = IRect::new((10, 10), (18, 19));
    let us3 = U16Rect::new((10, 10), (18, 19));
    let n3 = TestNode::new(v3, s3);
    let nid3 = n3.id();

    let v4 = TestValue::new(4);
    let s4 = IRect::new((3, 5), (20, 14));
    let us4 = U16Rect::new((3, 5), (15, 14));
    let n4 = TestNode::new(v4, s4);
    let nid4 = n4.id();

    let v5 = TestValue::new(5);
    let s5 = IRect::new((-3, -5), (10, 20));
    let us5 = U16Rect::new((0, 0), (10, 15));
    let n5 = TestNode::new(v5, s5);
    let nid5 = n5.id();

    let v6 = TestValue::new(6);
    let s6 = IRect::new((3, 6), (6, 10));
    let us6 = U16Rect::new((13, 16), (16, 19));
    let n6 = TestNode::new(v6, s6);
    let nid6 = n6.id();

    let v7 = TestValue::new(7);
    let s7 = IRect::new((3, 6), (15, 25));
    let us7 = U16Rect::new((3, 6), (10, 15));
    let n7 = TestNode::new(v7, s7);
    let nid7 = n7.id();

    let v8 = TestValue::new(8);
    let s8 = IRect::new((-1, -2), (2, 1));
    let us8 = U16Rect::new((3, 6), (5, 7));
    let n8 = TestNode::new(v8, s8);
    let nid8 = n8.id();

    let v9 = TestValue::new(9);
    let s9 = IRect::new((5, 6), (9, 8));
    let us9 = U16Rect::new((8, 12), (10, 14));
    let n9 = TestNode::new(v9, s9);
    let nid9 = n9.id();

    /*
     * The tree looks like:
     * ```
     *           n1
     *         /   \
     *        n2   n3
     *      /  \     \
     *     n4  n5    n6
     *           \
     *            n7
     *           / \
     *         n8   n9
     * ```
     */
    let mut tree = Itree::new(n1);
    tree.insert(nid1, n2);
    tree.insert(nid1, n3);
    tree.insert(nid2, n4);
    tree.insert(nid2, n5);
    tree.insert(nid3, n6);
    tree.insert(nid5, n7);
    tree.insert(nid7, n8);
    tree.insert(nid7, n9);

    assert!(tree.root_id() == nid1);
    let n1 = tree.node(nid1).unwrap();
    let n2 = tree.node(nid2).unwrap();
    let n3 = tree.node(nid3).unwrap();
    let n4 = tree.node(nid4).unwrap();
    let n5 = tree.node(nid5).unwrap();
    let n6 = tree.node(nid6).unwrap();
    let n7 = tree.node(nid7).unwrap();
    let n8 = tree.node(nid8).unwrap();
    let n9 = tree.node(nid9).unwrap();
    print_node!(n1, "n1");
    print_node!(n2, "n2");
    print_node!(n3, "n3");
    print_node!(n4, "n4");
    print_node!(n5, "n5");
    print_node!(n6, "n6");
    print_node!(n7, "n7");
    print_node!(n8, "n8");
    print_node!(n9, "n9");

    let expects = [us1, us2, us3, us4, us5, us6, us7, us8, us9];
    let nodes = [n1, n2, n3, n4, n5, n6, n7, n8, n9];
    for i in 0..9 {
      let expect = expects[i];
      let node = nodes[i];
      assert_node_actual_shape_eq!(node, expect, i);
    }
  }

  #[test]
  fn shape2() {
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
    let s3 = IRect::new((-2, -2), (-1, 0));
    let us3 = U16Rect::new((0, 0), (0, 0));
    let n3 = TestNode::new(v3, s3);
    let nid3 = n3.id();

    let v4 = TestValue::new(4);
    let s4 = IRect::new((3, 5), (20, 20));
    let us4 = U16Rect::new((3, 5), (20, 20));
    let n4 = TestNode::new(v4, s4);
    let nid4 = n4.id();

    let v5 = TestValue::new(5);
    let s5 = IRect::new((-3, -5), (15, 20));
    let us5 = U16Rect::new((3, 5), (18, 20));
    let n5 = TestNode::new(v5, s5);
    let nid5 = n5.id();

    let v6 = TestValue::new(6);
    let s6 = IRect::new((8, 13), (18, 25));
    let us6 = U16Rect::new((11, 18), (18, 20));
    let n6 = TestNode::new(v6, s6);
    let nid6 = n6.id();

    /*
     * The tree looks like:
     * ```
     *           n1
     *         /   \
     *        n2   n3
     *         \
     *         n4
     *        /
     *       n5
     *      /
     *     n6
     * ```
     */
    let mut tree = Itree::new(n1);
    tree.insert(nid1, n2);
    tree.insert(nid1, n3);
    tree.insert(nid2, n4);
    tree.insert(nid4, n5);
    tree.insert(nid5, n6);

    assert!(tree.root_id() == nid1);
    let n1 = tree.node(nid1).unwrap();
    let n2 = tree.node(nid2).unwrap();
    let n3 = tree.node(nid3).unwrap();
    let n4 = tree.node(nid4).unwrap();
    let n5 = tree.node(nid5).unwrap();
    let n6 = tree.node(nid6).unwrap();
    print_node!(n1, "n1");
    print_node!(n2, "n2");
    print_node!(n3, "n3");
    print_node!(n4, "n4");
    print_node!(n5, "n5");
    print_node!(n6, "n6");

    let expects = [us1, us2, us3, us4, us5, us6];
    let nodes = [n1, n2, n3, n4, n5, n6];
    for i in 0..6 {
      let expect = expects[i];
      let node = &nodes[i];
      assert_node_actual_shape_eq!(node, expect, i);
    }
  }

  #[test]
  fn push1() {
    INIT.call_once(test_log_init);

    let shape = IRect::new((0, 0), (10, 10));
    let node_values: Vec<usize> = [1, 2, 3, 4, 5].to_vec();
    let nodes: Vec<TestNode> = node_values
      .iter()
      .map(|value| TestValue::new(*value))
      .map(|tv| TestNode::new(tv, shape))
      .collect::<Vec<TestNode>>();
    let nodes_ids: Vec<InodeId> = nodes.iter().map(|n| n.id()).collect();

    /*
     * The tree looks like:
     * ```
     *             n1
     *         /        \
     *       n2, n3, n4, n5
     * ```
     */
    let mut tree = Itree::new(nodes[0].clone());
    for node in nodes.iter().skip(1) {
      tree.insert(nodes_ids[0], node.clone());
    }

    assert!(tree.root_id() == nodes_ids[0]);
    assert!(tree.children_ids(nodes_ids[0]).unwrap().len() == 4);
    assert!(!tree.children_ids(nodes_ids[0]).unwrap().is_empty());
    for nid in nodes_ids.iter().skip(1) {
      assert!(tree.children_ids(*nid).unwrap().is_empty());
    }

    for (i, nid) in nodes_ids.iter().enumerate() {
      let node = tree.node(*nid).unwrap();
      let expect = node_values[i];
      assert_node_value_eq!(node, expect);
    }

    let first1 = tree.children_ids(nodes_ids[0]).unwrap().first();
    assert!(first1.is_some());
    assert_eq!(*first1.unwrap(), nodes_ids[1]);

    let last1 = tree.children_ids(nodes_ids[0]).unwrap().last();
    assert!(last1.is_some());
    assert_eq!(*last1.unwrap(), nodes_ids[4]);

    for nid in nodes_ids.iter().skip(1) {
      let first = tree.children_ids(*nid).unwrap().first();
      let last = tree.children_ids(*nid).unwrap().last();
      assert!(first.is_none());
      assert!(last.is_none());
    }
  }

  fn make_tree(n: usize) -> (Vec<InodeId>, Itree<TestValue>) {
    let mut value = 1;
    let mut node_ids: Vec<InodeId> = vec![];

    let v = TestValue::new(value);
    value += 1;
    let s = IRect::new((0, 0), (10, 10));
    let root = TestNode::new(v, s);
    let root_id = root.id();
    node_ids.push(root_id);

    let mut tree = Itree::new(root);
    for _ in 1..n {
      let v = TestValue::new(value);
      value += 1;
      let node = TestNode::new(v, s);
      let node_id = node.id();
      tree.insert(root_id, node);
      node_ids.push(node_id);
    }

    (node_ids, tree)
  }

  #[test]
  fn remove1() {
    INIT.call_once(test_log_init);

    let (node_ids, mut tree) = make_tree(5);
    let remove0 = tree.remove(node_ids[0]);
    let remove2 = tree.remove(node_ids[2]);
    let remove4 = tree.remove(node_ids[4]);

    assert!(remove0.is_none());
    assert!(remove2.is_some());
    let remove2 = &remove2.unwrap();
    assert_node_value_eq!(remove2, 3);
    assert!(remove4.is_some());
    let remove4 = &remove4.unwrap();
    assert_node_value_eq!(remove4, 5);

    let remove1 = tree.remove(node_ids[1]);
    let remove3 = tree.remove(node_ids[3]);

    // 1,2,(3),4,(5)
    assert!(remove1.is_some());
    let remove1 = &remove1.unwrap();
    assert_node_value_eq!(remove1, 2);
    assert!(remove3.is_some());
    let remove3 = &remove3.unwrap();
    assert_node_value_eq!(remove3, 4);
  }

  #[test]
  fn get1() {
    INIT.call_once(test_log_init);

    let v1 = TestValue::new(1);
    let s1 = IRect::new((0, 0), (20, 20));
    let n1 = TestNode::new(v1, s1);
    let nid1 = n1.id();

    let v2 = TestValue::new(2);
    let s2 = IRect::new((0, 0), (15, 15));
    let n2 = TestNode::new(v2, s2);
    let nid2 = n2.id();

    let v3 = TestValue::new(3);
    let s3 = IRect::new((10, 10), (18, 19));
    let n3 = TestNode::new(v3, s3);
    let nid3 = n3.id();

    let v4 = TestValue::new(4);
    let s4 = IRect::new((3, 5), (20, 14));
    let n4 = TestNode::new(v4, s4);
    let nid4 = n4.id();

    let v5 = TestValue::new(5);
    let s5 = IRect::new((-3, -5), (10, 20));
    let n5 = TestNode::new(v5, s5);
    let nid5 = n5.id();

    let v6 = TestValue::new(6);
    let s6 = IRect::new((3, 6), (6, 10));
    let n6 = TestNode::new(v6, s6);
    let nid6 = n6.id();

    let v7 = TestValue::new(7);
    let s7 = IRect::new((3, 6), (15, 25));
    let n7 = TestNode::new(v7, s7);
    let nid7 = n7.id();

    let v8 = TestValue::new(8);
    let s8 = IRect::new((-1, -2), (2, 1));
    let n8 = TestNode::new(v8, s8);
    let nid8 = n8.id();

    let v9 = TestValue::new(9);
    let s9 = IRect::new((5, 6), (9, 8));
    let n9 = TestNode::new(v9, s9);
    let nid9 = n9.id();

    /*
     * The tree looks like:
     * ```
     *           n1
     *         /   \
     *        n2   n3
     *      /  \     \
     *     n4  n5    n6
     *           \
     *            n7
     *           / \
     *         n8   n9
     * ```
     */
    let mut tree = Itree::new(n1);
    tree.insert(nid1, n2);
    tree.insert(nid1, n3);
    tree.insert(nid2, n4);
    tree.insert(nid2, n5);
    tree.insert(nid3, n6);
    tree.insert(nid5, n7);
    tree.insert(nid7, n8);
    tree.insert(nid7, n9);

    assert!(nid1 == tree.root_id());
    let n1 = tree.node(nid1).unwrap();
    let n2 = tree.node(nid2).unwrap();
    let n3 = tree.node(nid3).unwrap();
    let n4 = tree.node(nid4).unwrap();
    let n5 = tree.node(nid5).unwrap();
    let n6 = tree.node(nid6).unwrap();
    let n7 = tree.node(nid7).unwrap();
    let n8 = tree.node(nid8).unwrap();
    let n9 = tree.node(nid9).unwrap();
    print_node!(n1, "n1");
    print_node!(n2, "n2");
    print_node!(n3, "n3");
    print_node!(n4, "n4");
    print_node!(n5, "n5");
    print_node!(n6, "n6");
    print_node!(n7, "n7");
    print_node!(n8, "n8");
    print_node!(n9, "n9");
  }

  #[test]
  fn get2() {
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
    let s3 = IRect::new((-2, -2), (-1, 0));
    let us3 = U16Rect::new((0, 0), (0, 0));
    let n3 = TestNode::new(v3, s3);
    let nid3 = n3.id();

    let v4 = TestValue::new(4);
    let s4 = IRect::new((3, 5), (20, 20));
    let us4 = U16Rect::new((3, 5), (20, 20));
    let n4 = TestNode::new(v4, s4);
    let nid4 = n4.id();

    let v5 = TestValue::new(5);
    let s5 = IRect::new((-3, -5), (15, 20));
    let us5 = U16Rect::new((3, 5), (18, 20));
    let n5 = TestNode::new(v5, s5);
    let nid5 = n5.id();

    let v6 = TestValue::new(6);
    let s6 = IRect::new((8, 13), (18, 25));
    let us6 = U16Rect::new((11, 18), (18, 20));
    let n6 = TestNode::new(v6, s6);
    let nid6 = n6.id();

    /*
     * The tree looks like:
     * ```
     *           n1
     *         /   \
     *        n2   n3
     *         \
     *         n4
     *        /
     *       n5
     *      /
     *     n6
     * ```
     */
    let mut tree = Itree::new(n1);
    tree.insert(nid1, n2);
    tree.insert(nid1, n3);
    tree.insert(nid2, n4);
    tree.insert(nid4, n5);
    tree.insert(nid5, n6);

    let n1 = tree.node(nid1).unwrap();
    let n2 = tree.node(nid2).unwrap();
    let n3 = tree.node(nid3).unwrap();
    let n4 = tree.node(nid4).unwrap();
    let n5 = tree.node(nid5).unwrap();
    let n6 = tree.node(nid6).unwrap();
    print_node!(n1, "n1");
    print_node!(n2, "n2");
    print_node!(n3, "n3");
    print_node!(n4, "n4");
    print_node!(n5, "n5");
    print_node!(n6, "n6");

    let expects = [us1, us2, us3, us4, us5, us6];
    let nodes = [n1, n2, n3, n4, n5, n6];
    for i in 0..6 {
      let expect = expects[i];
      let node = &nodes[i];
      assert_node_actual_shape_eq!(node, expect, i);
    }
  }

  #[test]
  fn move_by1() {
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
    let mut tree = Itree::new(n1);
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
      .map(|_i| (rng.next_u32() as isize, rng.next_u32() as isize))
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
      tree.move_by(nid3, x, y);
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
