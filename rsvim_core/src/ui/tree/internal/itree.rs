//! Internal tree structure that implements the widget tree.

use crate::prelude::*;
use crate::ui::tree::Inodeable;
use crate::ui::tree::LayoutNodeId;
use crate::ui::tree::TaffyTreeRc;
use crate::ui::tree::TreeNodeId;
use crate::ui::tree::internal::shapes;
use itertools::Itertools;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::iter::Iterator;
use taffy::TaffyTree;
use taffy::prelude::TaffyMaxContent;

#[derive(Debug, Clone)]
pub struct Irelationship {
  lotree: TaffyTree,
  nid2loid: FoldMap<TreeNodeId, LayoutNodeId>,
  loid2nid: FoldMap<LayoutNodeId, TreeNodeId>,
}

arc_mutex_ptr!(Irelationship);

#[derive(Debug, Clone)]
pub struct Itree<T>
where
  T: Inodeable,
{
  // Layout tree
  lotree: TaffyTreeRc,
  // Tree nodes
  nodes: FoldMap<TreeNodeId, T>,

  // Root node
  root_id: TreeNodeId,
  root_loid: LayoutNodeId,

  // Maps between node ID and layout node ID
  nid2loid: FoldMap<TreeNodeId, LayoutNodeId>,
  loid2nid: FoldMap<LayoutNodeId, TreeNodeId>,
}

#[derive(Debug)]
/// The level-order iterator of the tree, start from tree root.
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
  pub fn new(lotree: TaffyTreeRc, root_node: T) -> Self {
    let root_id = root_node.id();
    let root_loid = root_node.loid();

    let mut nodes = FoldMap::new();
    nodes.insert(root_id, root_node);

    let mut nid2loid = FoldMap::new();
    let mut loid2nid = FoldMap::new();
    nid2loid.insert(root_id, root_loid);
    loid2nid.insert(root_loid, root_id);

    Itree {
      lotree,
      nodes,
      root_id,
      root_loid,
      nid2loid,
      loid2nid,
    }
  }

  #[cfg(not(test))]
  fn _internal_check(&self) {}

  #[cfg(test)]
  fn _internal_check(&self) {
    debug_assert!(!self.nodes.is_empty());
    debug_assert!(self.lotree.borrow().total_node_count() != 0);
    debug_assert_eq!(self.nid2loid.len(), self.nodes.len());
    debug_assert_eq!(self.loid2nid.len(), self.nodes.len());
  }

  pub fn root_id(&self) -> TreeNodeId {
    self.root_id
  }

  pub fn root_loid(&self) -> LayoutNodeId {
    *self.nid2loid.get(&self.root_id).unwrap()
  }

  pub fn parent_id(&self, id: TreeNodeId) -> Option<TreeNodeId> {
    let loid = self.nid2loid.get(&id)?;
    let parent_loid = self.lotree.borrow().parent(*loid)?;
    self.loid2nid.get(&parent_loid).copied()
  }

  pub fn children_ids(&self, id: TreeNodeId) -> Vec<TreeNodeId> {
    if let Some(loid) = self.nid2loid.get(&id) {
      if let Ok(children_loids) = self.lotree.borrow().children(*loid) {
        return children_loids
          .iter()
          .filter(|i| self.loid2nid.contains_key(i))
          .map(|i| *self.loid2nid.get(i).unwrap())
          .collect_vec();
      }
    }
    vec![]
  }

  pub fn node(&self, id: TreeNodeId) -> Option<&T> {
    self.nodes.get(&id)
  }

  pub fn node_mut(&mut self, id: TreeNodeId) -> Option<&mut T> {
    self.nodes.get_mut(&id)
  }

  /// Get the level-order iterator on this tree, starts from root node.
  pub fn iter(&self) -> ItreeIter<'_, T> {
    ItreeIter::new(self, Some(self.root_id))
  }

  /// Get layout tree.
  pub fn lotree(&self) -> TaffyTreeRc {
    self.lotree.clone()
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
    debug_assert!(self.nid2loid.contains_key(&parent_id));

    let child_id = child_node.id();
    let child_loid = child_node.loid();
    let parent_loid = self.nid2loid.get(&parent_id).unwrap();

    let child_actual_shape = {
      let mut lo = self.lotree.borrow_mut();
      lo.add_child(*parent_loid, child_loid).unwrap();
      lo.compute_layout(*parent_loid, taffy::Size::MAX_CONTENT)
        .unwrap();
      let child_layout = lo.layout(child_loid).unwrap();
      let child_pos = point!(child_layout.location.x, child_layout.location.y);
      let child_pos = point_as!(child_pos, u16);
      let child_size = size!(child_layout.size.width, child_layout.size.height);
      let child_size = size_as!(child_size, u16);
      rect!(
        child_pos.x(),
        child_pos.y(),
        child_pos.x() + child_size.width(),
        child_pos.y() + child_size.height()
      )
    };
    child_node.set_actual_shape(&child_actual_shape);

    // Insert node into collection.
    let result = self.nodes.insert(child_id, child_node);
    self.nid2loid.insert(child_id, child_loid);
    self.loid2nid.insert(child_loid, child_id);

    self._internal_check();
    result
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
    debug_assert_ne!(id, self.root_id);
    self._internal_check();

    // Remove child node from collection.
    let result = match self.nodes.remove(&id) {
      Some(removed) => {
        let mut lo = self.lotree.borrow_mut();
        let loid = self.nid2loid.get(&id).unwrap();
        match lo.parent(*loid) {
          Some(parent_loid) => {
            lo.remove_child(parent_loid, *loid);
            Some(removed)
          }
          None => None,
        }
      }
      None => None,
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
                let expected_top_left_pos: IPos = point!(
                  current_top_left_pos.x() + x,
                  current_top_left_pos.y() + y
                );
                let expected_shape = rect!(
                  expected_top_left_pos.x(),
                  expected_top_left_pos.y(),
                  expected_top_left_pos.x() + current_shape.width(),
                  expected_top_left_pos.y() + current_shape.height()
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
        let next_top_left_pos: IPos = point!(x, y);
        let next_shape = rect!(
          next_top_left_pos.x(),
          next_top_left_pos.y(),
          next_top_left_pos.x() + current_shape.width(),
          next_top_left_pos.y() + current_shape.height()
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
              let expected_top_left_pos: IPos = point!(x, y);
              let expected_shape = rect!(
                expected_top_left_pos.x(),
                expected_top_left_pos.y(),
                expected_top_left_pos.x() + current_shape.width(),
                expected_top_left_pos.y() + current_shape.height()
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
