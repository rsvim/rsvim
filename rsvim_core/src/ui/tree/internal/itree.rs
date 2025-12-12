//! Internal tree structure that implements the widget tree.

use crate::prelude::*;
use crate::ui::tree::TreeNodeId;
use crate::ui::tree::internal::Inodeable;
use crate::ui::tree::internal::inode::next_node_id;
use crate::ui::tree::internal::shapes;
use itertools::Itertools;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::intrinsics::sub_with_overflow;
use std::iter::Iterator;
use taffy::AvailableSpace;
use taffy::Layout;
use taffy::Style;
use taffy::TaffyResult;
use taffy::TaffyTree;

const INVALID_ROOT_ID: TreeNodeId = -1;

pub enum RelationshipSetShapePolicy {
  TRUNCATE,
  BOUND,
}

#[derive(Debug, Clone)]
pub struct Relationships {
  ta: TaffyTree,

  // Maps TreeNodeId <=> taffy::NodeId
  id2taid: FoldMap<TreeNodeId, taffy::NodeId>,
  taid2id: FoldMap<taffy::NodeId, TreeNodeId>,

  // Shapes
  shapes: FoldMap<TreeNodeId, IRect>,
  // Cached actual shapes
  cached_actual_shapes: RefCell<FoldMap<TreeNodeId, U16Rect>>,

  // Root id
  root_id: TreeNodeId,

  // For debugging
  #[cfg(debug_assertions)]
  root_id_changes: usize,
  #[cfg(debug_assertions)]
  names: FoldMap<TreeNodeId, &'static str>,
}

rc_refcell_ptr!(Relationships);

impl Relationships {
  pub fn new() -> Self {
    Self {
      ta: TaffyTree::new(),
      id2taid: FoldMap::new(),
      taid2id: FoldMap::new(),
      shapes: RefCell::new(FoldMap::new()),
      cached_actual_shapes: RefCell::new(FoldMap::new()),
      root_id: INVALID_ROOT_ID,
      #[cfg(debug_assertions)]
      root_id_changes: 0,
      #[cfg(debug_assertions)]
      names: FoldMap::new(),
    }
  }

  #[allow(dead_code)]
  pub fn is_empty(&self) -> bool {
    self.id2taid.is_empty()
  }

  #[allow(dead_code)]
  pub fn len(&self) -> usize {
    self.id2taid.len()
  }

  #[cfg(not(test))]
  fn _internal_check(&self) {}

  #[cfg(test)]
  fn _internal_check(&self) {
    debug_assert_eq!(self.ta.total_node_count(), self.id2taid.len());
    debug_assert_eq!(self.ta.total_node_count(), self.taid2id.len());

    for (id, taid) in self.id2taid.iter() {
      debug_assert!(self.taid2id.contains_key(taid));
      debug_assert_eq!(*self.taid2id.get(taid).unwrap(), *id);
      if let Some(parent_taid) = self.ta.parent(*taid) {
        debug_assert!(self.taid2id.contains_key(&parent_taid));
      }
    }
    for (taid, nid) in self.taid2id.iter() {
      debug_assert!(self.id2taid.contains_key(nid));
      debug_assert_eq!(*self.id2taid.get(nid).unwrap(), *taid);
    }
  }

  fn _set_root_id_if_empty(&mut self, root_id: TreeNodeId) {
    if self.root_id == INVALID_ROOT_ID {
      self.root_id = root_id;
      if cfg!(debug_assertions) {
        self.root_id_changes += 1;
        debug_assert!(self.root_id_changes <= 1);
      }
    }
  }

  fn _set_name(&mut self, id: TreeNodeId, name: &'static str) {
    if cfg!(debug_assertions) {
      self.names.insert(id, name);
    }
  }

  pub fn new_leaf(
    &mut self,
    style: Style,
    name: &'static str,
  ) -> TaffyResult<TreeNodeId> {
    self._internal_check();
    let taid = self.ta.new_leaf(style)?;
    let id = next_node_id();
    self.id2taid.insert(id, taid);
    self.taid2id.insert(taid, id);
    self._set_root_id_if_empty(id);
    self._set_name(id, name);
    self._internal_check();
    Ok(id)
  }

  pub fn compute_layout(
    &mut self,
    id: TreeNodeId,
    available_size: taffy::Size<AvailableSpace>,
  ) -> TaffyResult<()> {
    self._internal_check();
    let taid = self.id2taid.get(&id).unwrap();
    let result = self.ta.compute_layout(*taid, available_size);
    self.clear_cached_actual_shapes(id);
    self._internal_check();
    result
  }

  pub fn layout(&self, id: TreeNodeId) -> TaffyResult<&Layout> {
    self._internal_check();
    let taid = self.id2taid.get(&id).unwrap();
    self.ta.layout(*taid)
  }

  pub fn style(&self, id: TreeNodeId) -> TaffyResult<&Style> {
    self._internal_check();
    let taid = self.id2taid.get(&id).unwrap();
    self.ta.style(*taid)
  }

  pub fn set_style(&mut self, id: TreeNodeId, style: Style) -> TaffyResult<()> {
    self._internal_check();
    let taid = self.id2taid.get(&id).unwrap();
    self.ta.set_style(*taid, style)
  }

  pub fn shape(&self, id: TreeNodeId) -> Option<&IRect> {
    self.shapes.get(&id)
  }

  #[inline]
  /// Set shape for a node. Since a node is always bounded by its parent, thus
  /// its real shape can be different with the "expecting" shape.
  ///
  /// Returns the "real" shape after adjustment.
  ///
  /// There are two policies when calculating the "adjusted" shape:
  /// - Truncate: Just cut all the parts that are out of its parent. For
  ///   example a node shape is `((-5, -10), (5, 9))`, and its parent size is
  ///   `(7, 8)`. This node's truncated shape is `((0, 0), (5, 8))`: its
  ///   left-top corner must be at least `(0, 0)`, and its bottom-right corner
  ///   is at most `(7, 8)`.
  /// - Bound: Keep as much as we can, first try to set at most the same size
  ///   as its parent, then move inside its parent thus avoid cutting any parts
  ///   that is out of its parent. For example a node shape is
  ///   `((-1, -2), (5, 6))`, and its parent size is `(6, 6)`. This node's
  ///   bounded shape is `((0, 0), (6, 6))`: First its original width is 6
  ///   which doesn't need to be truncated, but its original height is 8 so
  ///   need to be truncated into 6, it becomes `((-1, -2), (5, 4))`. Then move
  ///   it into parent to avoid more truncating, so its becomes
  ///   `((0, 0), (6, 6))`.
  pub fn set_shape(
    &mut self,
    id: TreeNodeId,
    shape: IRect,
    policy: RelationshipSetShapePolicy,
  ) -> TaffyResult<IRect> {
    let result = match self.parent(id) {
      Some(parent_id) => {
        let parent_actual_shape = self.actual_shape(parent_id)?;
        let result = match policy {
          RelationshipSetShapePolicy::TRUNCATE => {
            shapes::truncate_shape(&shape, &parent_actual_shape.size())
          }
          RelationshipSetShapePolicy::BOUND => {
            shapes::bound_shape(&shape, &parent_actual_shape.size())
          }
        };
        result
      }
      None => {
        debug_assert_eq!(shape.min().x, 0);
        debug_assert_eq!(shape.min().y, 0);
        debug_assert!(shape.max().x >= shape.min().x);
        debug_assert!(shape.max().y >= shape.min().y);
        shape
      }
    };
    self.shapes.insert(id, result);
    Ok(result)
  }

  #[inline]
  pub fn actual_shape(&self, id: TreeNodeId) -> Option<U16Rect> {
    self._internal_check();

    match self.parent(id) {
      None => {
        let shape = self.shape(id)?;
        Some(rect_as!(shape, u16))
      }
      Some(parent_id) => {
        let maybe_cached = self.cached_actual_shapes.borrow().get(&id).copied();
        match maybe_cached {
          Some(cached) => Some(cached),
          None => {
            // Non-root node truncated by its parent's shape.
            let shape = self.shape(id)?;
            let parent_actual_shape = self.actual_shape(parent_id)?;
            let actual_shape = shapes::convert_relative_to_absolute(
              &shape,
              &parent_actual_shape,
            );
            self
              .cached_actual_shapes
              .borrow_mut()
              .insert(id, actual_shape);
            Some(actual_shape)
          }
        }
      }
    }
  }

  /// Clear the cached actual_shapes since the provided id. All its
  /// descendants actual_shape will be cleared as well.
  fn clear_cached_actual_shapes(&mut self, id: TreeNodeId) {
    let mut q: VecDeque<TreeNodeId> = VecDeque::new();
    q.push_back(id);
    while let Some(parent_id) = q.pop_front() {
      self.cached_actual_shapes.borrow_mut().remove(&parent_id);
      if let Ok(children_ids) = self.children(parent_id) {
        for child_id in children_ids.iter() {
          q.push_back(*child_id);
        }
      }
    }
  }

  #[inline]
  /// Whether the node is visible, e.g. its actual_shape size is zero.
  pub fn visible(&self, id: TreeNodeId) -> TaffyResult<bool> {
    let actual_shape = self.actual_shape(id)?;
    Ok(!actual_shape.size().is_zero())
  }

  #[inline]
  pub fn invisible(&self, id: TreeNodeId) -> TaffyResult<bool> {
    self.visible(id).map(|v| !v)
  }

  #[inline]
  /// Whether the node is detached, e.g. it doesn't have a parent and it is not
  /// the root node. A root node is always attached even it has no parent.
  pub fn detached(&self, id: TreeNodeId) -> bool {
    !self.attached(id)
  }

  #[inline]
  pub fn attached(&self, id: TreeNodeId) -> bool {
    id == self.root_id || self.parent(id).is_some()
  }

  #[inline]
  /// The node is visible and its size > 0, e.g. both height and width > 0.
  pub fn enabled(&self, id: TreeNodeId) -> TaffyResult<bool> {
    self._internal_check();
    let visible = self.visible(id)?;
    let attached = self.attached(id);
    Ok(visible && attached)
  }

  #[inline]
  pub fn disabled(&self, id: TreeNodeId) -> TaffyResult<bool> {
    self.enabled(id).map(|v| !v)
  }

  pub fn parent(&self, id: TreeNodeId) -> Option<TreeNodeId> {
    self._internal_check();
    let taid = self.id2taid.get(&id)?;
    let parent_taid = self.ta.parent(*taid)?;
    self.taid2id.get(&parent_taid).copied()
  }

  pub fn children(&self, id: TreeNodeId) -> TaffyResult<Vec<TreeNodeId>> {
    self._internal_check();
    let taid = self.id2taid.get(&id).unwrap();
    let children_taids = self.ta.children(*taid)?;
    Ok(
      children_taids
        .iter()
        .map(|i| *self.taid2id.get(i).unwrap())
        .collect_vec(),
    )
  }

  pub fn add_child(
    &mut self,
    parent_id: TreeNodeId,
    child_id: TreeNodeId,
  ) -> TaffyResult<()> {
    self._internal_check();
    let parent_taid = self.id2taid.get(&parent_id).unwrap();
    let child_taid = self.id2taid.get(&child_id).unwrap();
    self.ta.add_child(*parent_taid, *child_taid)
  }

  pub fn remove_child(
    &mut self,
    parent_id: TreeNodeId,
    child_id: TreeNodeId,
  ) -> TaffyResult<TreeNodeId> {
    self._internal_check();
    let parent_taid = self.id2taid.get(&parent_id).unwrap();
    let child_taid = self.id2taid.get(&child_id).unwrap();
    let removed_taid = self.ta.remove_child(*parent_taid, *child_taid)?;
    debug_assert_eq!(removed_taid, *child_taid);
    let removed_id = *self.taid2id.get(&removed_taid).unwrap();
    debug_assert_eq!(removed_id, child_id);
    Ok(removed_id)
  }

  pub fn new_with_parent(
    &mut self,
    style: Style,
    parent_id: TreeNodeId,
    name: &'static str,
  ) -> TaffyResult<TreeNodeId> {
    let id = self.new_leaf(style, name)?;
    self.add_child(parent_id, id)?;
    Ok(id)
  }

  pub fn new_with_children(
    &mut self,
    style: Style,
    children: &[TreeNodeId],
    name: &'static str,
  ) -> TaffyResult<TreeNodeId> {
    self._internal_check();
    let children_taids = children
      .iter()
      .map(|i| *self.id2taid.get(i).unwrap())
      .collect_vec();
    let taid = self.ta.new_with_children(style, &children_taids)?;
    let id = next_node_id();
    self.id2taid.insert(id, taid);
    self.taid2id.insert(taid, id);
    self._set_root_id_if_empty(id);
    self._set_name(id, name);
    self._internal_check();
    Ok(id)
  }
}

impl Default for Relationships {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug, Clone)]
pub struct Itree<T>
where
  T: Inodeable,
{
  // Nodes collection, maps from node ID to its node struct.
  nodes: FoldMap<TreeNodeId, T>,

  // Maps parent and children edges. The parent edge weight is negative,
  // children edges are positive. The edge weight of each child is increased
  // with the order when they are inserted, i.e. the first child has the lowest
  // edge weight, the last child has the highest edge weight.
  relationships: RefCell<Relationships>,
}

#[derive(Debug)]
/// Iterate all the tree nodes in pre-order.
///
/// For each node, it first visits the node itself, then visits all its
/// children. This also follows the order when rendering the widget tree to
/// terminal device.
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

impl<T> Itree<T>
where
  T: Inodeable,
{
  pub fn new() -> Self {
    Itree {
      nodes: FoldMap::new(),
      relationships: RefCell::new(Relationships::new()),
    }
  }

  #[cfg(not(test))]
  fn _internal_check(&self) {}

  #[cfg(test)]
  fn _internal_check(&self) {
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
          .collect::<FoldSet<TreeNodeId>>()
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
  pub fn iter(&self) -> ItreeIter<'_, T> {
    ItreeIter::new(self, Some(self.relationships.borrow().root_id()))
  }
}

impl<T> Default for Itree<T>
where
  T: Inodeable,
{
  fn default() -> Self {
    Self::new()
  }
}

// Insert/Remove {
impl<T> Itree<T>
where
  T: Inodeable,
{
  /// Update the `start_id` node attributes, and all the descendants attributes of this node.
  ///
  /// Below attributes will be update:
  ///
  /// 1. [`actual_shape`](Inode::actual_shape()): The child actual shape should be always clipped
  ///    by parent's boundaries.
  fn update_descendant_attributes(
    &mut self,
    start_id: TreeNodeId,
    start_parent_id: TreeNodeId,
  ) {
    // Create the queue of parent-child ID pairs, to iterate all descendants under the child node.

    // Tuple of (child_id, parent_id, parent_actual_shape)
    type ChildAndParent = (TreeNodeId, TreeNodeId, U16Rect);

    // trace!("before create que");
    let mut que: VecDeque<ChildAndParent> = VecDeque::new();
    let pnode = self.nodes.get_mut(&start_parent_id).unwrap();
    let pnode_id = pnode.id();
    let pnode_actual_shape = *pnode.actual_shape();
    que.push_back((start_id, pnode_id, pnode_actual_shape));
    // trace!("after create que");

    // Iterate all descendants, and update their attributes.
    while let Some(child_and_parent) = que.pop_front() {
      let cnode_id = child_and_parent.0;
      let _pnode_id = child_and_parent.1;
      let pnode_actual_shape = child_and_parent.2;

      // trace!("before update cnode attr: {:?}", cnode);
      let cnode_ref = self.nodes.get_mut(&cnode_id).unwrap();
      let cnode_shape = *cnode_ref.shape();
      let cnode_actual_shape =
        shapes::convert_relative_to_absolute(&cnode_shape, &pnode_actual_shape);

      trace!(
        "update attr, cnode id/actual_shape:{:?}/{:?}, pnode id/actual_shape:{:?}/{:?}",
        cnode_id, cnode_actual_shape, pnode_id, pnode_actual_shape
      );

      // let cnode_ref = self.nodes.get_mut(&cnode_id).unwrap();
      cnode_ref.set_actual_shape(&cnode_actual_shape);

      for dnode_id in self.children_ids(cnode_id).iter() {
        if self.nodes.contains_key(dnode_id) {
          que.push_back((*dnode_id, cnode_id, cnode_actual_shape));
        }
      }
    }
  }

  /// Insert root node, without a parent node.
  pub fn insert_root(&mut self, mut child_node: T) {
    self._internal_check();

    // Child node.
    let child_id = child_node.id();

    debug_assert!(self.nodes.is_empty());
    debug_assert!(self.relationships.borrow().is_empty());

    // Update attributes for both the newly inserted child, and all its
    // descendants (if the child itself is also a sub-tree in current
    // relationship).
    //
    // NOTE: This is useful when we want to move some widgets and all its
    // children nodes to another place. We don't need to remove all the nodes
    // (which could be slow), but only need to move the root of the tree.
    //
    // The attributes to be updated:
    // 1. Actual shape.
    let shape = *child_node.shape();
    let actual_shape = rect_as!(shape, u16);
    child_node.set_actual_shape(&actual_shape);

    // Insert node into collection.
    self.nodes.insert(child_id, child_node);
    // Create first edge for root node.
    self.relationships.borrow_mut().add_root(child_id);

    self._internal_check();
  }

  /// Insert a node to the tree, with a parent node.
  ///
  /// This operation builds the connection between the parent and the inserted
  /// child. Also updates both the inserted child's attributes and all its
  /// descendants attributes.
  ///
  /// Below node attributes need to update:
  /// 1. [`actual_shape`](Inodeable::actual_shape()): The child actual shape
  ///    should be always be clipped by parent's boundaries.
  ///
  /// # Returns
  /// 1. `None` if the `child_node` doesn't exist.
  /// 2. The previous node on the same `child_node` ID, i.e. the inserted key.
  ///
  /// # Panics
  /// 1. If `parent_id` doesn't exist.
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

    debug_assert!(!self.relationships.borrow().contains_id(child_id));

    // Update attributes for both the newly inserted child, and all its
    // descendants (if the child itself is also a sub-tree in current
    // relationship).
    //
    // NOTE: This is useful when we want to move some widgets and all its
    // children nodes to another place. We don't need to remove all the nodes
    // (which could be slow), but only need to move the root of the tree.
    //
    // The attributes to be updated:
    // 1. Actual shape.
    let parent_node = self.nodes.get(&parent_id).unwrap();
    let parent_actual_shape = *parent_node.actual_shape();
    child_node.set_actual_shape(&shapes::convert_relative_to_absolute(
      child_node.shape(),
      &parent_actual_shape,
    ));

    // Insert node into collection.
    let result = self.nodes.insert(child_id, child_node);
    // Create edge between child and its parent.
    self
      .relationships
      .borrow_mut()
      .add_child(parent_id, child_id);

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
  /// 1. `None` if the `child_node` doesn't exist.
  /// 2. The previous node on the same `child_node` ID, i.e. the inserted key.
  ///
  /// # Panics
  /// 1. If `parent_id` doesn't exist.
  pub fn bounded_insert(
    &mut self,
    parent_id: TreeNodeId,
    mut child_node: T,
  ) -> Option<T> {
    // Panics if `parent_id` not exists.
    debug_assert!(self.nodes.contains_key(&parent_id));

    let parent_node = self.nodes.get(&parent_id).unwrap();
    let parent_actual_size = parent_node.actual_shape().size();

    // Bound child shape
    child_node.set_shape(&shapes::bound_shape(
      child_node.shape(),
      &parent_actual_size,
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
  /// 1. `None` if node `id` doesn't exist.
  /// 2. The removed node on the node `id`.
  ///
  /// # Panics
  /// If the node `id` is root node and root node cannot be removed.
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
        let maybe_parent_actual_size: Option<U16Size> = self
          .nodes
          .get(&parent_id)
          .map(|parent_node| parent_node.actual_shape().size());

        match maybe_parent_actual_size {
          Some(parent_actual_size) => {
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
                  shapes::bound_shape(&expected_shape, &parent_actual_size);
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
        let maybe_parent_actual_size: Option<U16Size> = self
          .nodes
          .get(&parent_id)
          .map(|parent_node| parent_node.actual_shape().size());

        match maybe_parent_actual_size {
          Some(parent_actual_size) => match self.nodes.get_mut(&id) {
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
                shapes::bound_shape(&expected_shape, &parent_actual_size);
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
