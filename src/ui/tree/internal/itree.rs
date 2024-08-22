//! Internal tree structure that implements the widget tree.

use geo::point;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::{collections::VecDeque, iter::Iterator};
use tracing::debug;

use crate::cart::{IPos, IRect, U16Rect};
use crate::ui::tree::internal::shapes;
use crate::ui::tree::internal::{InodeId, Inodeable};

#[derive(Debug, Default, Clone)]
pub struct Itree<T>
where
  T: Inodeable,
{
  // Root node ID.
  root_id: InodeId,
  // Nodes collection, maps from node ID to its node struct.
  nodes: HashMap<InodeId, T>,
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
  T: Inodeable,
{
  tree: &'a Itree<T>,
  queue: VecDeque<InodeId>,
}

impl<'a, T> Iterator for ItreeIter<'a, T>
where
  T: Inodeable,
{
  type Item = &'a T;

  fn next(&mut self) -> Option<Self::Item> {
    if let Some(id) = self.queue.pop_front() {
      match self.tree.children_ids(&id) {
        Some(children_ids) => {
          for child_id in children_ids.iter() {
            if self.tree.node(child_id).is_some() {
              self.queue.push_back(*child_id);
            }
          }
        }
        None => { /* Skip */ }
      }
      return self.tree.node(&id);
    }
    None
  }
}

impl<'a, T> ItreeIter<'a, T>
where
  T: Inodeable,
{
  pub fn new(tree: &'a Itree<T>, start_node_id: Option<InodeId>) -> Self {
    let mut queue = VecDeque::new();
    match start_node_id {
      Some(id) => queue.push_back(id),
      None => { /* Do nothing */ }
    }
    ItreeIter { tree, queue }
  }
}

#[derive(Debug)]
/// The mutable pre-order iterator of the tree.
pub struct ItreeIterMut<'a, T>
where
  T: Inodeable,
{
  tree: NonNull<Itree<T>>,
  queue: VecDeque<InodeId>,
  phantom: PhantomData<&'a mut Itree<T>>,
}

impl<'a, T> Iterator for ItreeIterMut<'a, T>
where
  T: Inodeable,
{
  type Item = &'a mut T;

  fn next(&mut self) -> Option<Self::Item> {
    if let Some(id) = self.queue.pop_front() {
      unsafe {
        // Fix `self.tree` mutable references.
        match self.tree.as_ref().children_ids(&id) {
          Some(children_ids) => {
            for child_id in children_ids.iter() {
              if self.tree.as_ref().node(child_id).is_some() {
                self.queue.push_back(*child_id);
              }
            }
          }
          None => { /* Skip */ }
        }
        return self.tree.as_mut().node_mut(&id);
      }
    }
    None
  }
}

impl<'a, T> ItreeIterMut<'a, T>
where
  T: Inodeable,
{
  pub fn new(tree: &'a mut Itree<T>, start_node_id: Option<InodeId>) -> Self {
    let mut queue = VecDeque::new();
    match start_node_id {
      Some(id) => queue.push_back(id),
      None => { /* Do nothing */ }
    }
    ItreeIterMut {
      tree: NonNull::new(tree as *mut Itree<T>).unwrap(),
      queue,
      phantom: PhantomData,
    }
  }
}

impl<T> Itree<T>
where
  T: Inodeable,
{
  pub fn new(root_node: T) -> Self {
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

  pub fn parent_id(&self, id: &InodeId) -> Option<&InodeId> {
    self.parent_ids.get(id)
  }

  pub fn children_ids(&self, id: &InodeId) -> Option<&Vec<InodeId>> {
    self.children_ids.get(id)
  }

  pub fn node(&self, id: &InodeId) -> Option<&T> {
    self.nodes.get(id)
  }

  pub fn node_mut(&mut self, id: &InodeId) -> Option<&mut T> {
    self.nodes.get_mut(id)
  }

  /// Get the iterator.
  ///
  /// By default, it iterates in pre-order iterator which starts from the root.
  /// For the children under the same node, it visits from lower z-index to higher.
  pub fn iter(&self) -> ItreeIter<T> {
    ItreeIter::new(self, Some(self.root_id))
  }

  /// Get the iterator that returns mutable reference.
  pub fn iter_mut(&mut self) -> ItreeIterMut<T> {
    ItreeIterMut::new(self, Some(self.root_id))
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
    type ChildAndParentPair<'a, T> = (&'a mut T, InodeId, usize, U16Rect);

    // Avoid the multiple mutable references on `self.nodes.get_mut` when updating all descendants attributes.
    let mut raw_nodes = NonNull::new(&mut self.nodes as *mut HashMap<InodeId, T>).unwrap();

    // debug!("before create que");
    let mut que: VecDeque<ChildAndParentPair<T>> = VecDeque::new();
    let pnode = raw_nodes.as_ref().get(&start_parent_id).unwrap();
    let pnode_id = pnode.id();
    let pnode_depth = *pnode.depth();
    let pnode_actual_shape = *pnode.actual_shape();
    que.push_back((
      raw_nodes.as_mut().get_mut(&start_id).unwrap(),
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
      let cnode_actual_shape = shapes::make_actual_shape(cnode_shape, pnode_actual_shape);

      debug!("update attr, cnode id/depth/actual_shape:{:?}/{:?}/{:?}, pnode id/depth/actual_shape:{:?}/{:?}/{:?}", cnode_id, cnode_depth, cnode_actual_shape, pnode_id, pnode_depth, pnode_actual_shape);

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
            match raw_nodes.as_mut().get_mut(dnode_id) {
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
  /// # Returns
  ///
  /// 1. `None` if the `child_node` doesn't exist.
  /// 2. The previous node on the same `child_node` ID, i.e. the inserted key.
  ///
  /// # Panics
  ///
  /// If `parent_id` doesn't exist.
  pub fn insert(&mut self, parent_id: &InodeId, mut child_node: T) -> Option<T> {
    debug!(
      "parent_id:{:?}, node_ids:{:?}, children_ids:{:?}",
      parent_id,
      self.node_ids(),
      self.children_ids
    );

    // Panics if `parent_id` not exists.
    assert!(self.nodes.contains_key(parent_id));
    assert!(
      self.children_ids.contains_key(parent_id),
      "children_ids {:?} doesn't contains parent_id {:?}",
      self.children_ids,
      parent_id
    );

    // Insert node.
    let child_id = child_node.id();
    let child_zindex = *child_node.zindex();

    // Maps the inserted child => its parent.
    self.children_ids.insert(child_id, vec![]);

    // Maps its parent => the inserted child.
    //
    // It inserts child ID to the `children_ids` vector of the parent, sorted by the Z-index.
    // For the children that have the same z-index value, it inserts at the end of those children.
    self.parent_ids.insert(child_id, *parent_id);
    let higher_zindex_pos: Vec<usize> = self
      .children_ids
      .get(parent_id)
      .unwrap()
      .iter()
      .enumerate()
      .filter(|(_index, cid)| match self.nodes.get(cid) {
        Some(cnode) => *cnode.zindex() > child_zindex,
        None => false,
      })
      .map(|(index, _cid)| index)
      .collect();
    match higher_zindex_pos.first() {
      Some(insert_pos) => {
        self
          .children_ids
          .get_mut(parent_id)
          .unwrap()
          .insert(*insert_pos, child_id);
      }
      None => {
        self.children_ids.get_mut(parent_id).unwrap().push(child_id);
      }
    }

    // Update the inserted child attributes:
    // 1. Depth.
    // 2. Actual shape.
    let parent_node = self.nodes.get(parent_id).unwrap();
    let parent_depth = *parent_node.depth();
    let parent_actual_shape = *parent_node.actual_shape();
    *child_node.depth_mut() = parent_depth + 1;
    *child_node.actual_shape_mut() =
      shapes::make_actual_shape(*child_node.shape(), parent_actual_shape);

    // Update all the descendants attributes under the `child_id` node.
    unsafe {
      // Fix mutable references on `self.update_descendant_attributes`.
      let mut raw_self = NonNull::new(self as *mut Itree<T>).unwrap();

      match raw_self.as_ref().children_ids.get(&child_id) {
        Some(descendant_ids) => {
          for dnode_id in descendant_ids.iter() {
            raw_self
              .as_mut()
              .update_descendant_attributes(*dnode_id, child_id);
          }
        }
        None => { /* Skip */ }
      }
    } // unsafe

    // Maps inserted child ID => the struct itself.
    self.nodes.insert(child_id, child_node)
  }

  /// Insert a node to the tree.
  ///
  /// It works similar to [`insert`](Itree::insert) method, except it limits the inserted node
  /// boundary based the parent's actual shape. This affects two aspects:
  ///
  /// 1. For size, if the inserted `child_node` is larger than the parent actual shape. The size
  ///    will be truncated to fit its parent. The bottom-right part will be removed, while the
  ///    top-left part will be keeped.
  /// 2. For position, if the inserted `child_node` hits the boundary of its parent. It simply
  ///    stops at its parent boundary.
  ///
  /// # Returns
  ///
  /// 1. `None` if the `child_node` doesn't exist.
  /// 2. The previous node on the same `child_node` ID, i.e. the inserted key.
  ///
  /// # Panics
  ///
  /// If `parent_id` doesn't exist.
  pub fn bounded_insert(&mut self, parent_id: &InodeId, mut child_node: T) -> Option<T> {
    // Panics if `parent_id` not exists.
    assert!(self.nodes.contains_key(parent_id));

    let parent_node = self.nodes.get(parent_id).unwrap();
    let parent_actual_shape = parent_node.actual_shape();

    // Bound child shape
    *child_node.shape_mut() = shapes::bound_shape(*child_node.shape(), *parent_actual_shape);

    self.insert(parent_id, child_node)
  }

  /// Remove a node by its ID.
  ///
  /// This operation breaks the connection between the removed node and its parent.
  ///
  /// But the relationships between the removed node and its descendants still remains in the tree,
  /// thus once you insert it back in the same tree, its descendants are still connected with the removed node.
  ///
  /// # Returns
  ///
  /// 1. `None` if node `id` doesn't exist.
  /// 2. The removed node on the node `id`.
  ///
  /// # Panics
  ///
  /// If the node `id` is the root node id since root node cannot be removed.
  pub fn remove(&mut self, id: InodeId) -> Option<T> {
    // Cannot remove root node.
    assert!(id != self.root_id);

    // Remove child from nodes collection.
    match self.nodes.remove(&id) {
      Some(removed) => {
        // Remove child `id` => parent ID mapping.
        match self.parent_ids.remove(&id) {
          Some(parent_id) => {
            // Remove the child `id` from the children vector of its parent.
            match self.children_ids.get_mut(&parent_id) {
              Some(children_ids) => {
                let filtered_indexes = children_ids
                  .iter()
                  .enumerate()
                  .filter(|(_index, cid)| **cid == id)
                  .map(|(index, _cid)| index)
                  .collect::<Vec<usize>>();
                if !filtered_indexes.is_empty() {
                  let removed_index = filtered_indexes[0];
                  children_ids.remove(removed_index);
                }
              }
              None => { /* Skip */ }
            }
          }
          None => { /* Skip */ }
        }
        Some(removed)
      }
      None => None,
    }
  }

  /// Move node by `(x, y)`.
  ///
  /// * The node moves left when `x < 0`.
  /// * The node moves right when `x > 0`.
  /// * The node moves up when `y < 0`.
  /// * The node moves down when `y > 0`.
  ///
  /// Note: This operation also updates all descendants attributes such as
  /// [`insert`](Itree::insert) method.
  ///
  /// # Returns
  ///
  /// 1. The new shape after movement if successfully.
  /// 2. `None` if the node `id` doesn't exist.
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
          // Fix mutable references on `self.update_descendant_attributes`.
          let mut raw_self = NonNull::new(self as *mut Itree<T>).unwrap();
          raw_self
            .as_mut()
            .update_descendant_attributes(id, *self.parent_ids.get(&id).unwrap());
        }

        Some(next_shape)
      }
      None => None,
    }
  }

  /// Bounded move node by `(x, y)`.
  ///
  /// It works similar to [`move_by`](Itree::move_by), but when a node hits the actual boundary of
  /// its parent, it simply stops moving.
  ///
  /// Note: This operation also updates all descendants attributes such as
  /// [`insert`](Itree::insert) method.
  ///
  /// # Returns
  ///
  /// 1. The new shape after movement if successfully.
  /// 2. `None` if the node `id` doesn't exist.
  pub fn bounded_move_by(&mut self, id: InodeId, x: isize, y: isize) -> Option<IRect> {
    match self.parent_ids.get(&id) {
      Some(parent_id) => {
        unsafe {
          // Fix mutable borrow on `self.base.node_mut`.
          let mut raw_nodes = NonNull::new(&mut self.nodes as *mut HashMap<InodeId, T>).unwrap();

          match raw_nodes.as_ref().get(parent_id) {
            Some(parent_node) => {
              match raw_nodes.as_mut().get_mut(&id) {
                Some(node) => {
                  let current_shape = *node.shape();
                  let current_top_left_pos: IPos = current_shape.min().into();
                  let expected_top_left_pos: IPos =
                    point!(x: current_top_left_pos.x() + x, y: current_top_left_pos.y() + y);
                  let expected_shape = IRect::new(
                    expected_top_left_pos,
                    point!(x: expected_top_left_pos.x() + current_shape.width(), y: expected_top_left_pos.y() + current_shape.height()),
                  );

                  let parent_actual_shape = *parent_node.actual_shape();
                  let final_shape = shapes::bound_shape(expected_shape, parent_actual_shape);
                  let final_top_left_pos: IPos = final_shape.min().into();

                  // Real movement
                  let final_x = final_top_left_pos.x() - current_top_left_pos.x();
                  let final_y = final_top_left_pos.y() - current_top_left_pos.y();
                  self.move_by(id, final_x, final_y)
                }
                None => None,
              }
            }
            None => None,
          }
        }
      }
      None => None,
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::inode_generate_impl;
  use std::sync::Once;
  use tracing::info;

  use crate::cart::{IRect, U16Rect};
  use crate::test::log::init as test_log_init;
  use crate::ui::tree::internal::{InodeBase, Inodeable};

  static INIT: Once = Once::new();

  use super::*;

  #[derive(Copy, Clone, Debug)]
  struct TestValue {
    value: i32,
    base: InodeBase,
  }

  impl TestValue {
    pub fn new(value: i32, shape: IRect) -> Self {
      TestValue {
        value,
        base: InodeBase::new(shape),
      }
    }
  }

  inode_generate_impl!(TestValue, base);

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
        assert_eq!($node.value, $expect);
        break;
      }
    };
  }

  #[test]
  fn new() {
    INIT.call_once(test_log_init);

    let s1 = IRect::new((0, 0), (1, 1));
    let n1 = TestValue::new(1, s1);
    let nid1 = n1.id();
    let tree = Itree::new(n1);

    assert_eq!(tree.len(), 1);
    assert_eq!(tree.root_id(), nid1);
    assert!(tree.parent_id(&nid1).is_none());
    assert!(tree.children_ids(&nid1).is_some());
    assert!(tree.children_ids(&nid1).unwrap().is_empty());

    for node in tree.iter() {
      assert_node_id_eq!(node, nid1);
    }
  }

  #[test]
  fn insert1() {
    INIT.call_once(test_log_init);

    let s1 = IRect::new((0, 0), (1, 1));
    let n1 = TestValue::new(1, s1);
    let nid1 = n1.id();

    let s2 = IRect::new((0, 0), (1, 1));
    let n2 = TestValue::new(2, s2);
    let nid2 = n2.id();

    let s3 = IRect::new((0, 0), (1, 1));
    let n3 = TestValue::new(3, s3);
    let nid3 = n3.id();

    let s4 = IRect::new((0, 0), (1, 1));
    let n4 = TestValue::new(4, s4);
    let nid4 = n4.id();

    let s5 = IRect::new((0, 0), (1, 1));
    let n5 = TestValue::new(5, s5);
    let nid5 = n5.id();

    let s6 = IRect::new((0, 0), (1, 1));
    let n6 = TestValue::new(6, s6);
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
    tree.insert(&nid1, n2);
    tree.insert(&nid1, n3);
    tree.insert(&nid2, n4);
    tree.insert(&nid2, n5);
    tree.insert(&nid3, n6);

    assert!(tree.root_id() == nid1);
    let n1 = tree.node(&nid1).unwrap();
    let n2 = tree.node(&nid2).unwrap();
    let n3 = tree.node(&nid3).unwrap();
    let n4 = tree.node(&nid4).unwrap();
    let n5 = tree.node(&nid5).unwrap();
    let n6 = tree.node(&nid6).unwrap();
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

    assert_eq!(tree.children_ids(&nid1).unwrap().len(), 2);
    assert_eq!(tree.children_ids(&nid2).unwrap().len(), 2);
    assert_eq!(tree.children_ids(&nid3).unwrap().len(), 1);
    assert_eq!(tree.children_ids(&nid4).unwrap().len(), 0);
    assert_eq!(tree.children_ids(&nid5).unwrap().len(), 0);
    assert_eq!(tree.children_ids(&nid6).unwrap().len(), 0);

    let contains_child = |parent_id: InodeId, child_id: InodeId| -> bool {
      match tree.children_ids(&parent_id) {
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

    let s1 = IRect::new((0, 0), (20, 20));
    let n1 = TestValue::new(1, s1);
    let nid1 = n1.id();

    let s2 = IRect::new((0, 0), (15, 15));
    let n2 = TestValue::new(2, s2);
    let nid2 = n2.id();

    let s3 = IRect::new((10, 10), (18, 19));
    let n3 = TestValue::new(3, s3);
    let nid3 = n3.id();

    let s4 = IRect::new((3, 5), (20, 14));
    let n4 = TestValue::new(4, s4);
    let nid4 = n4.id();

    let s5 = IRect::new((-3, -5), (10, 20));
    let n5 = TestValue::new(5, s5);
    let nid5 = n5.id();

    let s6 = IRect::new((3, 6), (6, 10));
    let n6 = TestValue::new(6, s6);
    let nid6 = n6.id();

    let s7 = IRect::new((3, 6), (15, 25));
    let n7 = TestValue::new(7, s7);
    let nid7 = n7.id();

    let s8 = IRect::new((-1, -2), (2, 1));
    let n8 = TestValue::new(8, s8);
    let nid8 = n8.id();

    let s9 = IRect::new((5, 6), (9, 8));
    let n9 = TestValue::new(9, s9);
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
    tree.insert(&nid1, n2);
    tree.insert(&nid1, n3);
    tree.insert(&nid2, n4);
    tree.insert(&nid2, n5);
    tree.insert(&nid3, n6);
    tree.insert(&nid5, n7);
    tree.insert(&nid7, n8);
    tree.insert(&nid7, n9);

    assert!(tree.root_id() == nid1);
    let n1 = tree.node(&nid1).unwrap();
    let n2 = tree.node(&nid2).unwrap();
    let n3 = tree.node(&nid3).unwrap();
    let n4 = tree.node(&nid4).unwrap();
    let n5 = tree.node(&nid5).unwrap();
    let n6 = tree.node(&nid6).unwrap();
    let n7 = tree.node(&nid7).unwrap();
    let n8 = tree.node(&nid8).unwrap();
    let n9 = tree.node(&nid9).unwrap();
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

    assert_eq!(tree.children_ids(&nid1).unwrap().len(), 2);
    assert_eq!(tree.children_ids(&nid2).unwrap().len(), 2);
    assert_eq!(tree.children_ids(&nid3).unwrap().len(), 1);
    assert_eq!(tree.children_ids(&nid4).unwrap().len(), 0);
    assert_eq!(tree.children_ids(&nid5).unwrap().len(), 1);
    assert_eq!(tree.children_ids(&nid6).unwrap().len(), 0);
    assert_eq!(tree.children_ids(&nid7).unwrap().len(), 2);
    assert_eq!(tree.children_ids(&nid8).unwrap().len(), 0);
    assert_eq!(tree.children_ids(&nid9).unwrap().len(), 0);

    let contains_child = |parent_id: InodeId, child_id: InodeId| -> bool {
      let result = match tree.children_ids(&parent_id) {
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
        tree.children_ids(&parent_id),
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

    let s1 = IRect::new((0, 0), (20, 20));
    let us1 = U16Rect::new((0, 0), (20, 20));
    let n1 = TestValue::new(1, s1);
    let nid1 = n1.id();

    let s2 = IRect::new((0, 0), (15, 15));
    let us2 = U16Rect::new((0, 0), (15, 15));
    let n2 = TestValue::new(2, s2);
    let nid2 = n2.id();

    let s3 = IRect::new((10, 10), (18, 19));
    let us3 = U16Rect::new((10, 10), (18, 19));
    let n3 = TestValue::new(3, s3);
    let nid3 = n3.id();

    let s4 = IRect::new((3, 5), (20, 14));
    let us4 = U16Rect::new((3, 5), (15, 14));
    let n4 = TestValue::new(4, s4);
    let nid4 = n4.id();

    let s5 = IRect::new((-3, -5), (10, 20));
    let us5 = U16Rect::new((0, 0), (10, 15));
    let n5 = TestValue::new(5, s5);
    let nid5 = n5.id();

    let s6 = IRect::new((3, 6), (6, 10));
    let us6 = U16Rect::new((13, 16), (16, 19));
    let n6 = TestValue::new(6, s6);
    let nid6 = n6.id();

    let s7 = IRect::new((3, 6), (15, 25));
    let us7 = U16Rect::new((3, 6), (10, 15));
    let n7 = TestValue::new(7, s7);
    let nid7 = n7.id();

    let s8 = IRect::new((-1, -2), (2, 1));
    let us8 = U16Rect::new((3, 6), (5, 7));
    let n8 = TestValue::new(8, s8);
    let nid8 = n8.id();

    let s9 = IRect::new((5, 6), (9, 8));
    let us9 = U16Rect::new((8, 12), (10, 14));
    let n9 = TestValue::new(9, s9);
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
    tree.insert(&nid1, n2);
    tree.insert(&nid1, n3);
    tree.insert(&nid2, n4);
    tree.insert(&nid2, n5);
    tree.insert(&nid3, n6);
    tree.insert(&nid5, n7);
    tree.insert(&nid7, n8);
    tree.insert(&nid7, n9);

    assert!(tree.root_id() == nid1);
    let n1 = tree.node(&nid1).unwrap();
    let n2 = tree.node(&nid2).unwrap();
    let n3 = tree.node(&nid3).unwrap();
    let n4 = tree.node(&nid4).unwrap();
    let n5 = tree.node(&nid5).unwrap();
    let n6 = tree.node(&nid6).unwrap();
    let n7 = tree.node(&nid7).unwrap();
    let n8 = tree.node(&nid8).unwrap();
    let n9 = tree.node(&nid9).unwrap();
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

    let s1 = IRect::new((0, 0), (20, 20));
    let us1 = U16Rect::new((0, 0), (20, 20));
    let n1 = TestValue::new(1, s1);
    let nid1 = n1.id();

    let s2 = IRect::new((0, 0), (20, 20));
    let us2 = U16Rect::new((0, 0), (20, 20));
    let n2 = TestValue::new(2, s2);
    let nid2 = n2.id();

    let s3 = IRect::new((-2, -2), (-1, 0));
    let us3 = U16Rect::new((0, 0), (0, 0));
    let n3 = TestValue::new(3, s3);
    let nid3 = n3.id();

    let s4 = IRect::new((3, 5), (20, 20));
    let us4 = U16Rect::new((3, 5), (20, 20));
    let n4 = TestValue::new(4, s4);
    let nid4 = n4.id();

    let s5 = IRect::new((-3, -5), (15, 20));
    let us5 = U16Rect::new((3, 5), (18, 20));
    let n5 = TestValue::new(5, s5);
    let nid5 = n5.id();

    let s6 = IRect::new((8, 13), (18, 25));
    let us6 = U16Rect::new((11, 18), (18, 20));
    let n6 = TestValue::new(6, s6);
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
    tree.insert(&nid1, n2);
    tree.insert(&nid1, n3);
    tree.insert(&nid2, n4);
    tree.insert(&nid4, n5);
    tree.insert(&nid5, n6);

    assert!(tree.root_id() == nid1);
    let n1 = tree.node(&nid1).unwrap();
    let n2 = tree.node(&nid2).unwrap();
    let n3 = tree.node(&nid3).unwrap();
    let n4 = tree.node(&nid4).unwrap();
    let n5 = tree.node(&nid5).unwrap();
    let n6 = tree.node(&nid6).unwrap();
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
    let node_values: Vec<i32> = [1, 2, 3, 4, 5].to_vec();
    let nodes: Vec<TestValue> = node_values
      .iter()
      .map(|value| TestValue::new(*value, shape))
      .collect();
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
      tree.insert(&nodes_ids[0], node.clone());
    }

    assert!(tree.root_id() == nodes_ids[0]);
    assert!(tree.children_ids(&nodes_ids[0]).unwrap().len() == 4);
    assert!(!tree.children_ids(&nodes_ids[0]).unwrap().is_empty());
    for nid in nodes_ids.iter().skip(1) {
      assert!(tree.children_ids(nid).unwrap().is_empty());
    }

    for (i, nid) in nodes_ids.iter().enumerate() {
      let node = tree.node(nid).unwrap();
      let expect = node_values[i];
      assert_node_value_eq!(node, expect);
    }

    let first1 = tree.children_ids(&nodes_ids[0]).unwrap().first();
    assert!(first1.is_some());
    assert_eq!(*first1.unwrap(), nodes_ids[1]);

    let last1 = tree.children_ids(&nodes_ids[0]).unwrap().last();
    assert!(last1.is_some());
    assert_eq!(*last1.unwrap(), nodes_ids[4]);

    for nid in nodes_ids.iter().skip(1) {
      let first = tree.children_ids(nid).unwrap().first();
      let last = tree.children_ids(nid).unwrap().last();
      assert!(first.is_none());
      assert!(last.is_none());
    }
  }

  fn make_tree(n: usize) -> (Vec<InodeId>, Itree<TestValue>) {
    let mut value = 1;
    let mut node_ids: Vec<InodeId> = vec![];

    let s = IRect::new((0, 0), (10, 10));
    let root = TestValue::new(value, s);
    let root_id = root.id();
    node_ids.push(root_id);
    value += 1;

    let mut tree = Itree::new(root);
    for _ in 1..n {
      let node = TestValue::new(value, s);
      let node_id = node.id();
      value += 1;
      tree.insert(&root_id, node);
      node_ids.push(node_id);
    }

    (node_ids, tree)
  }

  #[test]
  fn remove1() {
    INIT.call_once(test_log_init);

    let (node_ids, mut tree) = make_tree(5);
    let remove2 = tree.remove(node_ids[2]);
    let remove4 = tree.remove(node_ids[4]);

    assert!(remove2.is_some());
    let remove2 = &remove2.unwrap();
    assert_node_value_eq!(remove2, 3);
    assert!(!tree
      .children_ids(&tree.root_id())
      .unwrap()
      .contains(&remove2.id()));
    assert!(remove4.is_some());
    let remove4 = &remove4.unwrap();
    assert_node_value_eq!(remove4, 5);
    assert!(!tree
      .children_ids(&tree.root_id())
      .unwrap()
      .contains(&remove4.id()));

    let remove1 = tree.remove(node_ids[1]);
    let remove3 = tree.remove(node_ids[3]);

    // 1,2,(3),4,(5)
    assert!(remove1.is_some());
    let remove1 = &remove1.unwrap();
    assert_node_value_eq!(remove1, 2);
    assert!(!tree
      .children_ids(&tree.root_id())
      .unwrap()
      .contains(&remove1.id()));
    assert!(remove3.is_some());
    let remove3 = &remove3.unwrap();
    assert_node_value_eq!(remove3, 4);
    assert!(!tree
      .children_ids(&tree.root_id())
      .unwrap()
      .contains(&remove3.id()));
  }

  #[test]
  #[should_panic]
  fn remove2() {
    INIT.call_once(test_log_init);

    let (node_ids, mut tree) = make_tree(5);
    tree.remove(node_ids[0]);
  }

  #[test]
  fn get1() {
    INIT.call_once(test_log_init);

    let s1 = IRect::new((0, 0), (20, 20));
    let n1 = TestValue::new(1, s1);
    let nid1 = n1.id();

    let s2 = IRect::new((0, 0), (15, 15));
    let n2 = TestValue::new(2, s2);
    let nid2 = n2.id();

    let s3 = IRect::new((10, 10), (18, 19));
    let n3 = TestValue::new(3, s3);
    let nid3 = n3.id();

    let s4 = IRect::new((3, 5), (20, 14));
    let n4 = TestValue::new(4, s4);
    let nid4 = n4.id();

    let s5 = IRect::new((-3, -5), (10, 20));
    let n5 = TestValue::new(5, s5);
    let nid5 = n5.id();

    let s6 = IRect::new((3, 6), (6, 10));
    let n6 = TestValue::new(6, s6);
    let nid6 = n6.id();

    let s7 = IRect::new((3, 6), (15, 25));
    let n7 = TestValue::new(7, s7);
    let nid7 = n7.id();

    let s8 = IRect::new((-1, -2), (2, 1));
    let n8 = TestValue::new(8, s8);
    let nid8 = n8.id();

    let s9 = IRect::new((5, 6), (9, 8));
    let n9 = TestValue::new(9, s9);
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
    tree.insert(&nid1, n2);
    tree.insert(&nid1, n3);
    tree.insert(&nid2, n4);
    tree.insert(&nid2, n5);
    tree.insert(&nid3, n6);
    tree.insert(&nid5, n7);
    tree.insert(&nid7, n8);
    tree.insert(&nid7, n9);

    assert!(nid1 == tree.root_id());
    let n1 = tree.node(&nid1).unwrap();
    let n2 = tree.node(&nid2).unwrap();
    let n3 = tree.node(&nid3).unwrap();
    let n4 = tree.node(&nid4).unwrap();
    let n5 = tree.node(&nid5).unwrap();
    let n6 = tree.node(&nid6).unwrap();
    let n7 = tree.node(&nid7).unwrap();
    let n8 = tree.node(&nid8).unwrap();
    let n9 = tree.node(&nid9).unwrap();
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

    let s1 = IRect::new((0, 0), (20, 20));
    let us1 = U16Rect::new((0, 0), (20, 20));
    let n1 = TestValue::new(1, s1);
    let nid1 = n1.id();

    let s2 = IRect::new((0, 0), (20, 20));
    let us2 = U16Rect::new((0, 0), (20, 20));
    let n2 = TestValue::new(2, s2);
    let nid2 = n2.id();

    let s3 = IRect::new((-2, -2), (-1, 0));
    let us3 = U16Rect::new((0, 0), (0, 0));
    let n3 = TestValue::new(3, s3);
    let nid3 = n3.id();

    let s4 = IRect::new((3, 5), (20, 20));
    let us4 = U16Rect::new((3, 5), (20, 20));
    let n4 = TestValue::new(4, s4);
    let nid4 = n4.id();

    let s5 = IRect::new((-3, -5), (15, 20));
    let us5 = U16Rect::new((3, 5), (18, 20));
    let n5 = TestValue::new(5, s5);
    let nid5 = n5.id();

    let s6 = IRect::new((8, 13), (18, 25));
    let us6 = U16Rect::new((11, 18), (18, 20));
    let n6 = TestValue::new(6, s6);
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
    tree.insert(&nid1, n2);
    tree.insert(&nid1, n3);
    tree.insert(&nid2, n4);
    tree.insert(&nid4, n5);
    tree.insert(&nid5, n6);

    let n1 = tree.node(&nid1).unwrap();
    let n2 = tree.node(&nid2).unwrap();
    let n3 = tree.node(&nid3).unwrap();
    let n4 = tree.node(&nid4).unwrap();
    let n5 = tree.node(&nid5).unwrap();
    let n6 = tree.node(&nid6).unwrap();
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

    let s1 = IRect::new((0, 0), (20, 20));
    let n1 = TestValue::new(1, s1);
    let nid1 = n1.id();

    let s2 = IRect::new((0, 0), (20, 20));
    let n2 = TestValue::new(2, s2);
    let nid2 = n2.id();

    let s3 = IRect::new((0, 0), (1, 1));
    let n3 = TestValue::new(3, s3);
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
    tree.insert(&nid1, n2);
    tree.insert(&nid2, n3);

    let n1 = tree.node(&nid1).unwrap();
    let n2 = tree.node(&nid2).unwrap();
    let n3 = tree.node(&nid3).unwrap();
    print_node!(n1, "n1");
    print_node!(n2, "n2");
    print_node!(n3, "n3");

    // n3 Move: (x, y)
    let moves: Vec<(isize, isize)> = vec![
      (-10, -4),
      (2, -7),
      (1, 90),
      (-70, 41),
      (23, -4),
      (49, -121),
      (8, 3),
      (-10, -7),
      (6, 8),
    ];
    let expects: Vec<IRect> = vec![
      IRect::new((-10, -4), (-9, -3)),
      IRect::new((-8, -11), (-7, -10)),
      IRect::new((-7, 79), (-6, 80)),
      IRect::new((-77, 120), (-76, 121)),
      IRect::new((-54, 116), (-53, 117)),
      IRect::new((-5, -5), (-4, -4)),
      IRect::new((3, -2), (4, -1)),
      IRect::new((-7, -9), (-6, -8)),
      IRect::new((-1, -1), (0, 0)),
    ];

    for (i, m) in moves.iter().enumerate() {
      let x = m.0;
      let y = m.1;
      tree.move_by(nid3, x, y);
      let actual = *tree.node(&nid3).unwrap().shape();
      let expect = expects[i];
      info!("i:{:?}, actual:{:?}, expect:{:?}", i, actual, expect);
      assert!(actual == expect);
    }
  }

  #[test]
  fn bounded_move_by1() {
    INIT.call_once(test_log_init);

    let s1 = IRect::new((0, 0), (20, 20));
    let n1 = TestValue::new(1, s1);
    let nid1 = n1.id();

    let s2 = IRect::new((0, 0), (20, 20));
    let n2 = TestValue::new(2, s2);
    let nid2 = n2.id();

    let s3 = IRect::new((0, 0), (1, 1));
    let n3 = TestValue::new(3, s3);
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
    tree.insert(&nid1, n2);
    tree.insert(&nid2, n3);

    let n1 = tree.node(&nid1).unwrap();
    let n2 = tree.node(&nid2).unwrap();
    let n3 = tree.node(&nid3).unwrap();
    print_node!(n1, "n1");
    print_node!(n2, "n2");
    print_node!(n3, "n3");

    // n3 bounded move: (x, y)
    let moves: Vec<(isize, isize)> = vec![
      (-10, -4),
      (2, -7),
      (1, 90),
      (-70, 41),
      (23, -4),
      (49, -121),
      (8, 3),
      (-10, -7),
      (6, 8),
    ];
    let expects: Vec<IRect> = vec![
      IRect::new((0, 0), (1, 1)),
      IRect::new((2, 0), (3, 1)),
      IRect::new((3, 19), (4, 20)),
      IRect::new((0, 19), (1, 20)),
      IRect::new((19, 15), (20, 16)),
      IRect::new((19, 0), (20, 1)),
      IRect::new((19, 3), (20, 4)),
      IRect::new((9, 0), (10, 1)),
      IRect::new((15, 8), (16, 9)),
    ];

    for (i, m) in moves.iter().enumerate() {
      let x = m.0;
      let y = m.1;
      tree.bounded_move_by(nid3, x, y);
      let actual = *tree.node(&nid3).unwrap().shape();
      let expect = expects[i];
      info!("i:{:?}, actual:{:?}, expect:{:?}", i, actual, expect);
      assert!(actual == expect);
    }
  }
}
