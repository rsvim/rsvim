//! Internal tree structure that implements the widget tree.

use crate::prelude::*;
use crate::ui::tree::internal::Inodeable;
use crate::ui::tree::internal::TreeNodeId;
use crate::ui::tree::internal::shapes;
use geo::point;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::iter::Iterator;

#[derive(Debug, Clone)]
pub struct Relationships {
  // Root id.
  root_id: TreeNodeId,

  // Maps node id => its parent node id.
  parent_id: FoldMap<TreeNodeId, TreeNodeId>,

  // Maps node id => all its children node ids.
  children_ids: FoldMap<TreeNodeId, Vec<TreeNodeId>>,
}

impl Relationships {
  pub fn new(root_id: TreeNodeId) -> Self {
    let mut children_ids: FoldMap<TreeNodeId, Vec<TreeNodeId>> = FoldMap::new();
    children_ids.insert(root_id, Vec::new());

    Self {
      root_id,
      parent_id: FoldMap::new(),
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
      Some(removed_parent) => {
        match self.children_ids.get_mut(&removed_parent) {
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
              let children_of_to_be_removed_exists =
                self.children_ids.contains_key(&to_be_removed.1);
              let children_of_to_be_removed_is_empty = self
                .children_ids
                .get(&to_be_removed.1)
                .is_none_or(|children| children.is_empty());
              if children_of_to_be_removed_exists
                && children_of_to_be_removed_is_empty
              {
                self.children_ids.remove(&to_be_removed.1);
              }

              true
            } else {
              false
            }
          }
          None => false,
        }
      }
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
  relationships: RefCell<Relationships>,
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
    let relationships = RefCell::new(Relationships::new(root_id));
    Itree {
      nodes,
      relationships,
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
        let parents_children =
          self.relationships.borrow().children_ids(parent.unwrap());
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
  pub fn iter(&self) -> ItreeIter<'_, T> {
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
  fn update_descendant_attributes(
    &mut self,
    start_id: TreeNodeId,
    start_parent_id: TreeNodeId,
  ) {
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
      let cnode_actual_shape =
        shapes::make_actual_shape(&cnode_shape, &pnode_actual_shape);

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
  pub fn insert(
    &mut self,
    parent_id: TreeNodeId,
    mut child_node: T,
  ) -> Option<T> {
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
    self.relationships.borrow_mut().add_child(
      parent_id,
      child_id,
      child_zindex,
      &self.nodes,
    );

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
  pub fn bounded_insert(
    &mut self,
    parent_id: TreeNodeId,
    mut child_node: T,
  ) -> Option<T> {
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
  pub fn move_by(
    &mut self,
    id: TreeNodeId,
    x: isize,
    y: isize,
  ) -> Option<IRect> {
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
  pub fn bounded_move_by(
    &mut self,
    id: TreeNodeId,
    x: isize,
    y: isize,
  ) -> Option<IRect> {
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
                let expected_top_left_pos: IPos = point!(x: current_top_left_pos.x() + x, y: current_top_left_pos.y() + y);
                let expected_shape = IRect::new(
                  expected_top_left_pos,
                  point!(x: expected_top_left_pos.x() + current_shape.width(), y: expected_top_left_pos.y() + current_shape.height()),
                );

                let final_shape =
                  shapes::bound_shape(&expected_shape, &parent_actual_shape);
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
  pub fn move_to(
    &mut self,
    id: TreeNodeId,
    x: isize,
    y: isize,
  ) -> Option<IRect> {
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
  pub fn bounded_move_to(
    &mut self,
    id: TreeNodeId,
    x: isize,
    y: isize,
  ) -> Option<IRect> {
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

              let final_shape =
                shapes::bound_shape(&expected_shape, &parent_actual_shape);
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
