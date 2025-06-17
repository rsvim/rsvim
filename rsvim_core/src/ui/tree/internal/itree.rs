//! Internal tree structure that implements the widget tree.

use crate::prelude::*;
use crate::ui::tree::internal::shapes;
use crate::ui::tree::internal::{Inodeable, TreeNodeId};

use geo::point;
use std::fmt::Debug;
use std::{collections::VecDeque, iter::Iterator};
// use tracing::trace;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
struct Relationships {
  // Root id.
  root_id: TreeNodeId,

  // Maps node id => its parent node id.
  parent_id: HashMap<TreeNodeId, TreeNodeId>,

  // Maps node id => all its children node ids.
  children_ids: HashMap<TreeNodeId, Vec<TreeNodeId>>,
}

impl Relationships {
  pub fn new(root_id: TreeNodeId) -> Self {
    let mut children_ids: HashMap<TreeNodeId, Vec<TreeNodeId>> = HashMap::new();
    children_ids.insert(root_id, Vec::new());

    Self {
      root_id,
      parent_id: HashMap::new(),
      children_ids,
    }
  }

  pub fn parent_id(&self, id: TreeNodeId) -> Option<TreeNodeId> {
    self.parent_id.get(&id).cloned()
  }

  pub fn children_ids(&self, id: TreeNodeId) -> Vec<TreeNodeId> {
    match self.children_ids.get(&id) {
      Some(children_ids) => children_ids.to_vec(),
      None => Vec::new(),
    }
  }

  #[allow(dead_code)]
  pub fn is_empty(&self) -> bool {
    self.children_ids.is_empty()
  }

  #[allow(dead_code)]
  pub fn len(&self) -> usize {
    self.children_ids.len()
  }

  #[cfg(not(test))]
  fn _internal_check(&self) {}

  #[cfg(test)]
  fn _internal_check(&self) {
    let mut que: VecDeque<TreeNodeId> = VecDeque::new();
    que.push_back(self.root_id);

    while let Some(id) = que.pop_front() {
      let children_ids = self.children_ids(id);
      for c in children_ids {
        let p = self.parent_id.get(&c).cloned();
        debug_assert!(p.is_some());
        debug_assert_eq!(p.unwrap(), id);
      }
      match self.parent_id.get(&id).cloned() {
        Some(parent) => {
          debug_assert_eq!(
            self
              .children_ids(parent)
              .iter()
              .cloned()
              .filter(|c| *c == id)
              .count(),
            1
          );
        }
        None => debug_assert_eq!(id, self.root_id),
      }
    }
  }

  pub fn root_id(&self) -> TreeNodeId {
    self.root_id
  }

  pub fn contains_id(&self, id: TreeNodeId) -> bool {
    self._internal_check();
    self.children_ids.contains_key(&id)
  }

  pub fn add_child<T>(
    &mut self,
    parent_id: TreeNodeId,
    child_id: TreeNodeId,
    child_zindex: usize,
    nodes: &HashMap<TreeNodeId, T>,
  ) where
    T: Inodeable,
  {
    debug_assert!(!self.contains_id(child_id));
    self._internal_check();

    // Initialize children_ids vector.
    self.children_ids.insert(child_id, Vec::new());

    // Binds connection from child => parent.
    self.parent_id.insert(child_id, parent_id);

    // Binds connection from parent => child.
    //
    // NOTE: It inserts child to the `children_ids` vector which belongs to the parent, and the
    // children are sorted by their Z-index value from lower to higher (UI widget node with higher
    // Z-index has a higher priority to show on the final TUI, but the order is reversed when
    // rendering). For those children that share the same Z-index value, it inserts at the end of
    // those children.
    let higher_zindex_pos: Vec<usize> = self
      .children_ids
      .get(&parent_id)
      .unwrap()
      .iter()
      .enumerate()
      .filter(|(_index, cid)| match nodes.get(cid) {
        Some(cnode) => cnode.zindex() > child_zindex,
        None => false,
      })
      .map(|(index, _cid)| index)
      .collect();
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

    self._internal_check();
  }

  pub fn remove_child(&mut self, child_id: TreeNodeId) -> bool {
    self._internal_check();

    let result = match self.parent_id.remove(&child_id) {
      Some(removed_parent) => match self.children_ids.get_mut(&removed_parent) {
        Some(to_be_removed_children) => {
          let to_be_removed_child = to_be_removed_children
            .iter()
            .enumerate()
            .filter(|(_idx, c)| **c == child_id)
            .map(|(idx, c)| (idx, *c))
            .collect::<Vec<(usize, TreeNodeId)>>();
          if !to_be_removed_child.is_empty() {
            debug_assert_eq!(to_be_removed_child.len(), 1);
            let to_be_removed = to_be_removed_child[0];
            to_be_removed_children.remove(to_be_removed.0);

            // If `to_be_removed` has a empty `children` vector, remove it to workaround the `len`
            // api.
            let children_of_to_be_removed_exists = self.children_ids.contains_key(&to_be_removed.1);
            let children_of_to_be_removed_is_empty = self
              .children_ids
              .get(&to_be_removed.1)
              .is_none_or(|children| children.is_empty());
            if children_of_to_be_removed_exists && children_of_to_be_removed_is_empty {
              self.children_ids.remove(&to_be_removed.1);
            }

            true
          } else {
            false
          }
        }
        None => false,
      },
      None => false,
    };

    self._internal_check();
    result
  }
}

#[derive(Debug, Clone)]
pub struct Itree<T>
where
  T: Inodeable,
{
  // Nodes collection, maps from node ID to its node struct.
  nodes: HashMap<TreeNodeId, T>,

  // Maps parent and children edges. The parent edge weight is negative, children edges are
  // positive. The edge weight of each child is increased with the order when they are inserted,
  // i.e. the first child has the lowest edge weight, the last child has the highest edge weight.
  //
  // NOTE: The children (under the same parent) are rendered with the order of their Z-index value
  // from lower to higher, for those children share the same Z-index, the child how owns the lower
  // edge weight will be rendered first.
  relationships: Rc<RefCell<Relationships>>,
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
  que: VecDeque<TreeNodeId>,
}

impl<'a, T> Iterator for ItreeIter<'a, T>
where
  T: Inodeable,
{
  type Item = &'a T;

  fn next(&mut self) -> Option<Self::Item> {
    if let Some(id) = self.que.pop_front() {
      for child_id in self.tree.children_ids(id) {
        if self.tree.node(child_id).is_some() {
          self.que.push_back(child_id);
        }
      }
      self.tree.node(id)
    } else {
      None
    }
  }
}

impl<'a, T> ItreeIter<'a, T>
where
  T: Inodeable,
{
  pub fn new(tree: &'a Itree<T>, start_node_id: Option<TreeNodeId>) -> Self {
    let mut que = VecDeque::new();
    if let Some(id) = start_node_id {
      que.push_back(id);
    }
    Self { tree, que }
  }
}

// Attributes {
impl<T> Itree<T>
where
  T: Inodeable,
{
  pub fn new(root_node: T) -> Self {
    let root_id = root_node.id();
    let mut nodes = HashMap::new();
    nodes.insert(root_id, root_node);
    let relationships = Relationships::new(root_id);
    Itree {
      nodes,
      relationships: Rc::new(RefCell::new(relationships)),
    }
  }

  #[cfg(not(test))]
  fn _internal_check(&self) {}

  #[cfg(test)]
  fn _internal_check(&self) {
    debug_assert!(!self.nodes.is_empty());
    debug_assert!(!self.relationships.borrow().is_empty());
    debug_assert_eq!(self.relationships.borrow().len(), self.nodes.len());

    let root_id = self.relationships.borrow().root_id();
    let mut que: VecDeque<TreeNodeId> = VecDeque::new();
    que.push_back(root_id);

    while let Some(id) = que.pop_front() {
      let parent = self.relationships.borrow().parent_id(id);
      if id == root_id {
        debug_assert!(parent.is_none());
      } else {
        debug_assert!(parent.is_some());
        let parents_children = self.relationships.borrow().children_ids(parent.unwrap());
        for c in parents_children {
          let child_parent = self.relationships.borrow().parent_id(c);
          debug_assert!(child_parent.is_some());
          debug_assert_eq!(child_parent.unwrap(), parent.unwrap());
        }
      }

      let children_ids = self.relationships.borrow().children_ids(id);
      debug_assert_eq!(
        children_ids.len(),
        children_ids
          .iter()
          .cloned()
          .collect::<HashSet<TreeNodeId>>()
          .len()
      );
      for c in children_ids {
        let child_parent = self.relationships.borrow().parent_id(c);
        debug_assert!(child_parent.is_some());
        debug_assert_eq!(child_parent.unwrap(), id);
      }
    }
  }

  pub fn len(&self) -> usize {
    self.nodes.len()
  }

  pub fn is_empty(&self) -> bool {
    self.nodes.len() <= 1
  }

  pub fn root_id(&self) -> TreeNodeId {
    self.relationships.borrow().root_id()
  }

  pub fn node_ids(&self) -> Vec<TreeNodeId> {
    self.nodes.keys().copied().collect()
  }

  pub fn parent_id(&self, id: TreeNodeId) -> Option<TreeNodeId> {
    self.relationships.borrow().parent_id(id)
  }

  pub fn children_ids(&self, id: TreeNodeId) -> Vec<TreeNodeId> {
    self.relationships.borrow().children_ids(id)
  }

  pub fn node(&self, id: TreeNodeId) -> Option<&T> {
    self.nodes.get(&id)
  }

  pub fn node_mut(&mut self, id: TreeNodeId) -> Option<&mut T> {
    self.nodes.get_mut(&id)
  }

  /// Get the iterator.
  ///
  /// By default, it iterates in pre-order iterator which starts from the root.
  /// For the children under the same node, it visits from lower z-index to higher.
  pub fn iter(&self) -> ItreeIter<T> {
    ItreeIter::new(self, Some(self.relationships.borrow().root_id()))
  }
}
// Attributes }

// Insert/Remove {
impl<T> Itree<T>
where
  T: Inodeable,
{
  /// Update the `start_id` node attributes, and all the descendants attributes of this node.
  ///
  /// Below attributes will be update:
  ///
  /// 1. [`depth`](Inode::depth()): The child depth should always be the parent's depth + 1.
  /// 2. [`actual_shape`](Inode::actual_shape()): The child actual shape should be always clipped
  ///    by parent's boundaries.
  fn update_descendant_attributes(&mut self, start_id: TreeNodeId, start_parent_id: TreeNodeId) {
    // Create the queue of parent-child ID pairs, to iterate all descendants under the child node.

    // Tuple of (child_id, parent_id, parent_depth, parent_actual_shape)
    type ChildAndParent = (TreeNodeId, TreeNodeId, usize, U16Rect);

    // trace!("before create que");
    let mut que: VecDeque<ChildAndParent> = VecDeque::new();
    let pnode = self.nodes.get_mut(&start_parent_id).unwrap();
    let pnode_id = pnode.id();
    let pnode_depth = pnode.depth();
    let pnode_actual_shape = *pnode.actual_shape();
    que.push_back((start_id, pnode_id, pnode_depth, pnode_actual_shape));
    // trace!("after create que");

    // Iterate all descendants, and update their attributes.
    while let Some(child_and_parent) = que.pop_front() {
      let cnode_id = child_and_parent.0;
      let _pnode_id = child_and_parent.1;
      let pnode_depth = child_and_parent.2;
      let pnode_actual_shape = child_and_parent.3;

      // trace!("before update cnode attr: {:?}", cnode);
      let cnode_ref = self.nodes.get_mut(&cnode_id).unwrap();
      let cnode_depth = pnode_depth + 1;
      let cnode_shape = *cnode_ref.shape();
      let cnode_actual_shape = shapes::make_actual_shape(&cnode_shape, &pnode_actual_shape);

      // trace!("update attr, cnode id/depth/actual_shape:{:?}/{:?}/{:?}, pnode id/depth/actual_shape:{:?}/{:?}/{:?}", cnode_id, cnode_depth, cnode_actual_shape, pnode_id, pnode_depth, pnode_actual_shape);

      // let cnode_ref = self.nodes.get_mut(&cnode_id).unwrap();
      cnode_ref.set_depth(cnode_depth);
      cnode_ref.set_actual_shape(&cnode_actual_shape);

      // raw_nodes
      //   .as_mut()
      //   .get_mut(&cnode_id)
      //   .unwrap()
      //   .set_depth(cnode_depth);
      // raw_nodes
      //   .as_mut()
      //   .get_mut(&cnode_id)
      //   .unwrap()
      //   .set_actual_shape(&cnode_actual_shape);

      for dnode_id in self.children_ids(cnode_id).iter() {
        if self.nodes.contains_key(dnode_id) {
          que.push_back((*dnode_id, cnode_id, cnode_depth, cnode_actual_shape));
        }
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
  /// 1. [`depth`](Inodeable::depth()): The child depth should be always the parent depth + 1.
  /// 2. [`actual_shape`](Inodeable::actual_shape()): The child actual shape should be always be clipped by parent's boundaries.
  ///
  /// # Returns
  ///
  /// 1. `None` if the `child_node` doesn't exist.
  /// 2. The previous node on the same `child_node` ID, i.e. the inserted key.
  ///
  /// # Panics
  ///
  /// If `parent_id` doesn't exist.
  pub fn insert(&mut self, parent_id: TreeNodeId, mut child_node: T) -> Option<T> {
    self._internal_check();
    debug_assert!(self.nodes.contains_key(&parent_id));
    debug_assert!(self.relationships.borrow().contains_id(parent_id));

    // Child node.
    let child_id = child_node.id();
    let child_zindex = child_node.zindex();

    debug_assert!(!self.relationships.borrow().contains_id(child_id));

    // Update attributes for both the newly inserted child, and all its descendants (if the child
    // itself is also a sub-tree in current relationship).
    //
    // NOTE: This is useful when we want to move some widgets and all its children nodes to another
    // place. We don't need to remove all the nodes (which could be slow), but only need to move
    // the root of the tree.
    //
    // The attributes to be updated:
    // 1. Depth.
    // 2. Actual shape.
    let parent_node = self.nodes.get(&parent_id).unwrap();
    let parent_depth = parent_node.depth();
    let parent_actual_shape = *parent_node.actual_shape();
    child_node.set_depth(parent_depth + 1);
    child_node.set_actual_shape(&shapes::make_actual_shape(
      child_node.shape(),
      &parent_actual_shape,
    ));

    // Insert node into collection.
    let result = self.nodes.insert(child_id, child_node);
    // Create edge between child and its parent.
    self
      .relationships
      .borrow_mut()
      .add_child(parent_id, child_id, child_zindex, &self.nodes);

    // Update all the descendants attributes under the `child_id` node.
    for dnode_id in self.children_ids(child_id).iter() {
      self.update_descendant_attributes(*dnode_id, child_id);
    }

    self._internal_check();
    result
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
  pub fn bounded_insert(&mut self, parent_id: TreeNodeId, mut child_node: T) -> Option<T> {
    // Panics if `parent_id` not exists.
    debug_assert!(self.nodes.contains_key(&parent_id));

    let parent_node = self.nodes.get(&parent_id).unwrap();
    let parent_actual_shape = parent_node.actual_shape();

    // Bound child shape
    child_node.set_shape(&shapes::bound_shape(
      child_node.shape(),
      parent_actual_shape,
    ));

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
  pub fn remove(&mut self, id: TreeNodeId) -> Option<T> {
    // Cannot remove root node.
    debug_assert_ne!(id, self.relationships.borrow().root_id());
    self._internal_check();

    // Remove child node from collection.
    let result = match self.nodes.remove(&id) {
      Some(removed) => {
        // Remove node/edge relationship.
        debug_assert!(self.relationships.borrow().contains_id(id));
        // Remove edges between `id` and its parent.
        let relation_removed = self.relationships.borrow_mut().remove_child(id);
        debug_assert!(relation_removed);
        Some(removed)
      }
      None => {
        debug_assert!(!self.relationships.borrow().contains_id(id));
        None
      }
    };

    self._internal_check();
    result
  }
}
// Insert/Remove }

// Movement {

// /// Describe the relative position of a node and its parent node, based on the actual shape (after
// /// truncated).
// ///
// /// There're several kinds of use cases:
// ///
// /// 1. No-edge contact (inside): The node is completely inside its parent without any edges
// ///    contacted, which looks like:
// ///
// ///    ```text
// ///    -----------------
// ///    |               |
// ///    |    --------   |
// ///    |    |//////|   |
// ///    |    |//////|   |
// ///    |    --------   |
// ///    |               |
// ///    -----------------
// ///    ```
// ///
// /// 2. Single-edge contact: The node is in contact with its parent on only 1 edge, which looks
// ///    like:
// ///
// ///    ```text
// ///    -----------------
// ///    |               |
// ///    |        -------|
// ///    |        |//////|
// ///    |        |//////|
// ///    |        -------|
// ///    |               |
// ///    -----------------
// ///    ```
// ///
// /// 3. Double-edges contact: The node is in contact on 2 edges, which looks like:
// ///
// ///    ```text
// ///    -----------------
// ///    |        |//////|
// ///    |        |//////|
// ///    |        -------|
// ///    |               |
// ///    |               |
// ///    |               |
// ///    -----------------
// ///    ```
// ///
// /// 4. Triple-edges contact: The node is in contact on 3 edges, which looks like:
// ///
// ///    ```text
// ///    -----------------
// ///    |  |////////////|
// ///    |  |////////////|
// ///    |  |////////////|
// ///    |  |////////////|
// ///    |  |////////////|
// ///    |  |////////////|
// ///    -----------------
// ///    ```
// ///
// /// 5. All-edges contact (overlapping): The node is in contact on 4 edges, i.e. the node is exactly
// ///    the same with (or even bigger than, and truncated by) its parent, which looks like:
// ///
// ///    ```text
// ///    -----------------
// ///    |///////////////|
// ///    |///////////////|
// ///    |///////////////|
// ///    |///////////////|
// ///    |///////////////|
// ///    |///////////////|
// ///    -----------------
// ///    ```
// ///
// pub enum InodeRelativePosition {
//   /// 0-edge
//   Inside,
//   /// 1-edge
//   Top,
//   Bottom,
//   Left,
//   Right,
//   // 2-edges
//   TopLeft,
//   TopRight,
//   BottomLeft,
//   BottomRight,
//   // 3-edges
//   TopBottomLeft,
//   TopBottomRight,
//   LeftRightTop,
//   LeftRightBottom,
//   // All-edges
//   Overlapping,
// }

impl<T> Itree<T>
where
  T: Inodeable,
{
  /// Move node by distance `(x, y)`, the `x`/`y` is the motion distances.
  ///
  /// * The node moves left when `x < 0`.
  /// * The node moves right when `x > 0`.
  /// * The node moves up when `y < 0`.
  /// * The node moves down when `y > 0`.
  ///
  /// NOTE:
  /// 1. The position is relatively based on the node parent.
  /// 2. This operation also updates the shape/position of all descendant nodes, similar to
  ///    [`insert`](Itree::insert) method.
  ///
  /// # Returns
  ///
  /// 1. The new shape after movement if successfully.
  /// 2. `None` if the node `id` doesn't exist.
  pub fn move_by(&mut self, id: TreeNodeId, x: isize, y: isize) -> Option<IRect> {
    match self.nodes.get_mut(&id) {
      Some(node) => {
        let current_shape = *node.shape();
        let current_top_left_pos: IPos = current_shape.min().into();
        self.move_to(
          id,
          current_top_left_pos.x() + x,
          current_top_left_pos.y() + y,
        )
      }
      None => None,
    }
  }

  /// Bounded move node by distance `(x, y)`, the `x`/`y` is the motion distances.
  ///
  /// It works similar to [`move_by`](Itree::move_by), but when a node hits the actual boundary of
  /// its parent, it simply stops moving.
  ///
  /// NOTE:
  /// 1. The position is relatively based on the node parent.
  /// 2. This operation also updates the shape/position of all descendant nodes, similar to
  ///    [`insert`](Itree::insert) method.
  ///
  /// # Returns
  ///
  /// 1. The new shape after movement if successfully.
  /// 2. `None` if the node `id` doesn't exist.
  pub fn bounded_move_by(&mut self, id: TreeNodeId, x: isize, y: isize) -> Option<IRect> {
    match self.parent_id(id) {
      Some(parent_id) => {
        let maybe_parent_actual_shape: Option<U16Rect> = self
          .nodes
          .get(&parent_id)
          .map(|parent_node| *parent_node.actual_shape());

        match maybe_parent_actual_shape {
          Some(parent_actual_shape) => {
            match self.nodes.get_mut(&id) {
              Some(node) => {
                let current_shape = *node.shape();
                let current_top_left_pos: IPos = current_shape.min().into();
                let expected_top_left_pos: IPos =
                  point!(x: current_top_left_pos.x() + x, y: current_top_left_pos.y() + y);
                let expected_shape = IRect::new(
                  expected_top_left_pos,
                  point!(x: expected_top_left_pos.x() + current_shape.width(), y: expected_top_left_pos.y() + current_shape.height()),
                );

                let final_shape = shapes::bound_shape(&expected_shape, &parent_actual_shape);
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
      None => None,
    }
  }

  /// Move node to position `(x, y)`, the `(x, y)` is the new position.
  ///
  /// NOTE:
  /// 1. The position is relatively based on the node parent. The `(x, y)` is based on the left-top
  ///    anchor, i.e. the left-top anchor position is `(0, 0)`.
  /// 2. This operation also updates the shape/position of all descendant nodes, similar to
  ///    [`insert`](Itree::insert) method.
  ///
  /// # Returns
  ///
  /// 1. The new shape after movement if successfully.
  /// 2. `None` if the node `id` doesn't exist.
  pub fn move_to(&mut self, id: TreeNodeId, x: isize, y: isize) -> Option<IRect> {
    match self.nodes.get_mut(&id) {
      Some(node) => {
        let current_shape = *node.shape();
        let next_top_left_pos: IPos = point!(x: x, y: y);
        let next_shape = IRect::new(
          next_top_left_pos,
          point!(x: next_top_left_pos.x() + current_shape.width(), y: next_top_left_pos.y() + current_shape.height()),
        );
        node.set_shape(&next_shape);

        // Update all the descendants attributes under the `id` node.
        self.update_descendant_attributes(id, self.parent_id(id).unwrap());

        Some(next_shape)
      }
      None => None,
    }
  }

  /// Bounded move node to position `(x, y)`, the `(x, y)` is the new position.
  ///
  /// It works similar to [`move_by`](Itree::move_by), but when a node hits the actual boundary of
  /// its parent, it simply stops moving.
  ///
  /// NOTE:
  /// 1. The position is relatively based on the node parent. The `(x, y)` is based on the left-top
  ///    anchor, i.e. the left-top anchor position is `(0, 0)`.
  /// 2. This operation also updates the shape/position of all descendant nodes, similar to
  ///    [`insert`](Itree::insert) method.
  ///
  /// # Returns
  ///
  /// 1. The new shape after movement if successfully.
  /// 2. `None` if the node `id` doesn't exist.
  pub fn bounded_move_to(&mut self, id: TreeNodeId, x: isize, y: isize) -> Option<IRect> {
    match self.parent_id(id) {
      Some(parent_id) => {
        let maybe_parent_actual_shape: Option<U16Rect> = self
          .nodes
          .get(&parent_id)
          .map(|parent_node| *parent_node.actual_shape());

        match maybe_parent_actual_shape {
          Some(parent_actual_shape) => match self.nodes.get_mut(&id) {
            Some(node) => {
              let current_shape = *node.shape();
              let expected_top_left_pos: IPos = point!(x: x, y: y);
              let expected_shape = IRect::new(
                expected_top_left_pos,
                point!(x: expected_top_left_pos.x() + current_shape.width(), y: expected_top_left_pos.y() + current_shape.height()),
              );

              let final_shape = shapes::bound_shape(&expected_shape, &parent_actual_shape);
              let final_top_left_pos: IPos = final_shape.min().into();

              self.move_to(id, final_top_left_pos.x(), final_top_left_pos.y())
            }
            None => None,
          },
          None => None,
        }
      }
      None => None,
    }
  }
}
// Movement }

#[cfg(test)]
mod tests {
  use crate::inode_impl;
  use tracing::info;

  use crate::prelude::*;
  use crate::test::log::init as test_log_init;
  use crate::ui::tree::internal::{InodeBase, Inodeable};

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

  inode_impl!(TestValue, base);

  macro_rules! print_node {
    ($node: ident, $name: expr) => {
      info!("{}: {:?}", $name, $node.clone());
    };
  }

  macro_rules! assert_parent_child_depth {
    ($parent: ident, $child: ident) => {
      assert_eq!($parent.depth() + 1, $child.depth());
    };
  }

  macro_rules! assert_node_actual_shape_eq {
    ($node: ident, $expect: expr, $index: expr) => {
      assert_eq!(*$node.actual_shape(), $expect, "index:{:?}", $index,);
    };
  }

  macro_rules! assert_node_value_eq {
    ($node: ident, $expect: expr) => {
      assert_eq!($node.value, $expect);
    };
  }

  #[test]
  fn new() {
    // test_log_init();

    let s1 = IRect::new((0, 0), (1, 1));
    let n1 = TestValue::new(1, s1);
    let nid1 = n1.id();
    let tree = Itree::new(n1);

    assert_eq!(tree.len(), 1);
    assert_eq!(tree.root_id(), nid1);
    assert!(tree.parent_id(nid1).is_none());
    assert!(tree.children_ids(nid1).is_empty());
  }

  #[test]
  fn insert1() {
    // test_log_init();

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

    assert!(nid1 < nid2);
    assert!(nid2 < nid3);
    assert!(nid3 < nid4);
    assert!(nid4 < nid5);
    assert!(nid5 < nid6);

    assert_parent_child_depth!(n1, n2);
    assert_parent_child_depth!(n1, n3);
    assert_parent_child_depth!(n2, n4);
    assert_parent_child_depth!(n2, n5);
    assert_parent_child_depth!(n2, n6);
    assert_parent_child_depth!(n3, n6);

    assert_eq!(tree.children_ids(nid1).len(), 2);
    assert_eq!(tree.children_ids(nid2).len(), 2);
    assert_eq!(tree.children_ids(nid3).len(), 1);
    assert_eq!(tree.children_ids(nid4).len(), 0);
    assert_eq!(tree.children_ids(nid5).len(), 0);
    assert_eq!(tree.children_ids(nid6).len(), 0);

    let contains_child = |parent_id: TreeNodeId, child_id: TreeNodeId| -> bool {
      tree
        .children_ids(parent_id)
        .iter()
        .filter(|cid| **cid == child_id)
        .collect::<Vec<_>>()
        .len()
        == 1
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
    // test_log_init();

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

    assert!(nid1 < nid2);
    assert!(nid2 < nid3);
    assert!(nid3 < nid4);
    assert!(nid4 < nid5);
    assert!(nid5 < nid6);
    assert!(nid6 < nid7);
    assert!(nid7 < nid8);
    assert!(nid8 < nid9);

    assert_parent_child_depth!(n1, n2);
    assert_parent_child_depth!(n1, n3);
    assert_parent_child_depth!(n2, n4);
    assert_parent_child_depth!(n2, n5);
    assert_parent_child_depth!(n2, n6);
    assert_parent_child_depth!(n3, n6);
    assert_parent_child_depth!(n5, n7);
    assert_parent_child_depth!(n7, n8);
    assert_parent_child_depth!(n7, n9);

    assert_eq!(tree.children_ids(nid1).len(), 2);
    assert_eq!(tree.children_ids(nid2).len(), 2);
    assert_eq!(tree.children_ids(nid3).len(), 1);
    assert_eq!(tree.children_ids(nid4).len(), 0);
    assert_eq!(tree.children_ids(nid5).len(), 1);
    assert_eq!(tree.children_ids(nid6).len(), 0);
    assert_eq!(tree.children_ids(nid7).len(), 2);
    assert_eq!(tree.children_ids(nid8).len(), 0);
    assert_eq!(tree.children_ids(nid9).len(), 0);

    let contains_child = |parent_id: TreeNodeId, child_id: TreeNodeId| -> bool {
      let result = tree
        .children_ids(parent_id)
        .iter()
        .filter(|cid| **cid == child_id)
        .collect::<Vec<_>>()
        .len()
        == 1;
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
    // test_log_init();

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
    // test_log_init();

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
    // test_log_init();

    let shape = IRect::new((0, 0), (10, 10));
    let node_values: Vec<i32> = [1, 2, 3, 4, 5].to_vec();
    let nodes: Vec<TestValue> = node_values
      .iter()
      .map(|value| TestValue::new(*value, shape))
      .collect();
    let nodes_ids: Vec<TreeNodeId> = nodes.iter().map(|n| n.id()).collect();

    /*
     * The tree looks like:
     * ```
     *             n1
     *         /        \
     *       n2, n3, n4, n5
     * ```
     */
    let mut tree = Itree::new(nodes[0]);
    for node in nodes.iter().skip(1) {
      tree.insert(nodes_ids[0], *node);
    }

    assert!(tree.root_id() == nodes_ids[0]);
    assert!(tree.children_ids(nodes_ids[0]).len() == 4);
    assert!(!tree.children_ids(nodes_ids[0]).is_empty());
    for nid in nodes_ids.iter().skip(1) {
      assert!(tree.children_ids(*nid).is_empty());
    }

    for (i, nid) in nodes_ids.iter().enumerate() {
      let node = tree.node(*nid).unwrap();
      let expect = node_values[i];
      assert_node_value_eq!(node, expect);
    }

    let first1 = tree.children_ids(nodes_ids[0]).first().cloned();
    assert!(first1.is_some());
    assert_eq!(first1.unwrap(), nodes_ids[1]);

    let last1 = tree.children_ids(nodes_ids[0]).last().cloned();
    assert!(last1.is_some());
    assert_eq!(last1.unwrap(), nodes_ids[4]);

    for nid in nodes_ids.iter().skip(1) {
      let first = tree.children_ids(*nid).first().cloned();
      let last = tree.children_ids(*nid).last().cloned();
      assert!(first.is_none());
      assert!(last.is_none());
    }
  }

  fn make_tree(n: usize) -> (Vec<TreeNodeId>, Itree<TestValue>) {
    let mut value = 1;
    let mut node_ids: Vec<TreeNodeId> = vec![];

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
      tree.insert(root_id, node);
      node_ids.push(node_id);
    }

    (node_ids, tree)
  }

  #[test]
  fn remove1() {
    // test_log_init();

    let (node_ids, mut tree) = make_tree(5);
    let remove2 = tree.remove(node_ids[2]);
    let remove4 = tree.remove(node_ids[4]);

    assert!(remove2.is_some());
    let remove2 = &remove2.unwrap();
    assert_node_value_eq!(remove2, 3);
    assert!(!tree.children_ids(tree.root_id()).contains(&remove2.id()));
    assert!(remove4.is_some());
    let remove4 = &remove4.unwrap();
    assert_node_value_eq!(remove4, 5);
    assert!(!tree.children_ids(tree.root_id()).contains(&remove4.id()));

    let remove1 = tree.remove(node_ids[1]);
    let remove3 = tree.remove(node_ids[3]);

    // 1,2,(3),4,(5)
    assert!(remove1.is_some());
    let remove1 = &remove1.unwrap();
    assert_node_value_eq!(remove1, 2);
    assert!(!tree.children_ids(tree.root_id()).contains(&remove1.id()));
    assert!(remove3.is_some());
    let remove3 = &remove3.unwrap();
    assert_node_value_eq!(remove3, 4);
    assert!(!tree.children_ids(tree.root_id()).contains(&remove3.id()));
  }

  #[test]
  #[should_panic]
  fn remove2() {
    // test_log_init();

    let (node_ids, mut tree) = make_tree(5);
    tree.remove(node_ids[0]);
  }

  #[test]
  fn get1() {
    // test_log_init();

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
    // test_log_init();

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
    // test_log_init();

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
    tree.insert(nid1, n2);
    tree.insert(nid2, n3);

    let n1 = tree.node(nid1).unwrap();
    let n2 = tree.node(nid2).unwrap();
    let n3 = tree.node(nid3).unwrap();
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
      let actual = *tree.node(nid3).unwrap().shape();
      let expect = expects[i];
      info!("i:{:?}, actual:{:?}, expect:{:?}", i, actual, expect);
      assert!(actual == expect);
    }
  }

  #[test]
  fn bounded_move_by1() {
    test_log_init();

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
    tree.insert(nid1, n2);
    tree.insert(nid2, n3);

    let n1 = tree.node(nid1).unwrap();
    let n2 = tree.node(nid2).unwrap();
    let n3 = tree.node(nid3).unwrap();
    print_node!(n1, "n1");
    print_node!(n2, "n2");
    print_node!(n3, "n3");

    // n3 bounded move by: (x, y)
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
      let actual = *tree.node(nid3).unwrap().shape();
      let expect = expects[i];
      info!("i:{:?}, actual:{:?}, expect:{:?}", i, actual, expect);
      assert!(actual == expect);
    }
  }

  #[test]
  fn move_to1() {
    test_log_init();

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
    tree.insert(nid1, n2);
    tree.insert(nid2, n3);

    let n1 = tree.node(nid1).unwrap();
    let n2 = tree.node(nid2).unwrap();
    let n3 = tree.node(nid3).unwrap();
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
      IRect::new((2, -7), (3, -6)),
      IRect::new((1, 90), (2, 91)),
      IRect::new((-70, 41), (-69, 42)),
      IRect::new((23, -4), (24, -3)),
      IRect::new((49, -121), (50, -120)),
      IRect::new((8, 3), (9, 4)),
      IRect::new((-10, -7), (-9, -6)),
      IRect::new((6, 8), (7, 9)),
    ];

    for (i, m) in moves.iter().enumerate() {
      let x = m.0;
      let y = m.1;
      tree.move_to(nid3, x, y);
      let actual = *tree.node(nid3).unwrap().shape();
      let expect = expects[i];
      info!("i:{:?}, actual:{:?}, expect:{:?}", i, actual, expect);
      assert!(actual == expect);
    }
  }

  #[test]
  fn bounded_move_to1() {
    test_log_init();

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
    tree.insert(nid1, n2);
    tree.insert(nid2, n3);

    let n1 = tree.node(nid1).unwrap();
    let n2 = tree.node(nid2).unwrap();
    let n3 = tree.node(nid3).unwrap();
    print_node!(n1, "n1");
    print_node!(n2, "n2");
    print_node!(n3, "n3");

    // n3 bounded move to: (x, y)
    let moves: Vec<(isize, isize)> = vec![
      (-10, -4),
      (2, -7),
      (1, 90),
      (-70, 41),
      (23, -4),
      (49, -121),
      (8, 3),
      (5, 6),
      (6, 8),
    ];
    let expects: Vec<IRect> = vec![
      IRect::new((0, 0), (1, 1)),
      IRect::new((2, 0), (3, 1)),
      IRect::new((1, 19), (2, 20)),
      IRect::new((0, 19), (1, 20)),
      IRect::new((19, 0), (20, 1)),
      IRect::new((19, 0), (20, 1)),
      IRect::new((8, 3), (9, 4)),
      IRect::new((5, 6), (6, 7)),
      IRect::new((6, 8), (7, 9)),
    ];

    for (i, m) in moves.iter().enumerate() {
      let x = m.0;
      let y = m.1;
      tree.bounded_move_to(nid3, x, y);
      let actual = *tree.node(nid3).unwrap().shape();
      let expect = expects[i];
      info!("i:{:?}, actual:{:?}, expect:{:?}", i, actual, expect);
      assert!(actual == expect);
    }
  }
}
