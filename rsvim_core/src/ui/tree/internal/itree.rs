//! Internal tree structure that implements the widget tree.

use crate::prelude::*;
use crate::ui::tree::TreeNodeId;
use crate::ui::tree::internal::Inodeable;
use crate::ui::tree::internal::inode::DEFAULT_ENABLED;
use crate::ui::tree::internal::inode::DEFAULT_ZINDEX;
use crate::ui::tree::internal::inode::next_node_id;
use crate::ui::tree::internal::shapes::*;
use itertools::Itertools;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::iter::Iterator;
use taffy::AvailableSpace;
use taffy::Layout;
use taffy::Style;
use taffy::TaffyResult;
use taffy::TaffyTree;
use taffy::prelude::FromLength;
use taffy::prelude::TaffyMaxContent;

pub const INVALID_ROOT_ID: TreeNodeId = -1;

#[derive(Debug, Clone)]
struct Ta {
  ta: TaffyTree,
  // Maps TreeNodeId <==> taffy::NodeId.
  id2taid: FoldMap<TreeNodeId, taffy::NodeId>,
  taid2id: FoldMap<taffy::NodeId, TreeNodeId>,
}

impl Ta {
  pub fn new() -> Self {
    Self {
      ta: TaffyTree::new(),
      id2taid: FoldMap::new(),
      taid2id: FoldMap::new(),
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
      if let Ok(children_taids) = self.ta.children(*taid) {
        for child_taid in children_taids {
          debug_assert!(self.taid2id.contains_key(&child_taid));
        }
      }
    }
    for (taid, nid) in self.taid2id.iter() {
      debug_assert!(self.id2taid.contains_key(nid));
      debug_assert_eq!(*self.id2taid.get(nid).unwrap(), *taid);
    }
  }

  pub fn new_leaf(&mut self, style: Style) -> TaffyResult<TreeNodeId> {
    self._internal_check();
    let taid = self.ta.new_leaf(style)?;
    let id = next_node_id();
    self.id2taid.insert(id, taid);
    self.taid2id.insert(taid, id);
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

  pub fn contains(&self, id: TreeNodeId) -> bool {
    self.id2taid.contains_key(&id)
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
    let result = self.ta.add_child(*parent_taid, *child_taid)?;
    Ok(result)
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

  pub fn set_children(
    &mut self,
    parent_id: TreeNodeId,
    children: &[TreeNodeId],
  ) -> TaffyResult<()> {
    self._internal_check();
    let parent_taid = self.id2taid.get(&parent_id).unwrap();
    let children_taids = children
      .iter()
      .map(|i| *self.id2taid.get(i).unwrap())
      .collect_vec();
    self.ta.set_children(*parent_taid, &children_taids)
  }

  pub fn new_with_parent(
    &mut self,
    style: Style,
    parent_id: TreeNodeId,
  ) -> TaffyResult<TreeNodeId> {
    let id = self.new_leaf(style)?;
    self.add_child(parent_id, id)?;
    Ok(id)
  }

  pub fn new_with_children(
    &mut self,
    style: Style,
    children: &[TreeNodeId],
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
    self._internal_check();
    Ok(id)
  }
}

#[derive(Debug, Clone)]
pub struct Relation {
  // Maps parent and children IDs.
  //
  // NOTE: TaffyTree itself can also maintain parent/child relationship, but it
  // has several limitations when calculating the layout:
  // 1. It doesn't support hidden/invisble, i.e. when specifying `{display:
  //    None}` for some children nodes, but TaffyTree still calculates these
  //    non-display nodes.
  // 2. It doesn't support Z-index, i.e. we will have to manually remove/insert
  //    some children nodes on TaffyTree for different Z-index.
  // These issues will force us to maintain parent/child relationship by
  // ourself, instead of directly relying on TaffyTree's internal parent/child
  // relationship.
  // For each time, we can only calculate layout for those visible nodes or the
  // nodes that are in same Z-index. For other hidden nodes or the nodes with
  // different Z-index, we need to manually remove them from the parent, thus
  // to make sure the layout calculation is correct.
  parent_ids: FoldMap<TreeNodeId, TreeNodeId>,
  children_ids: FoldMap<TreeNodeId, Vec<TreeNodeId>>,

  // Maps A parent ==> their children's Z-index value in the
  // TaffyTree.
  // NOTE: When a parent has multiple children that have different Z-index
  // values, TaffyTree cannot calculate correct layout for different Z-index
  // children. When calculating a layout for all children nodes with `A`
  // Z-index value, we will have to remove all other children nodes that are
  // not `A` Z-index value.
  children_zindexes: FoldMap<TreeNodeId, usize>,

  root_id: TreeNodeId,

  #[cfg(debug_assertions)]
  root_changes: usize,
  #[cfg(debug_assertions)]
  names: FoldMap<TreeNodeId, &'static str>,
}

impl Relation {
  pub fn new() -> Self {
    Self {
      parent_ids: FoldMap::new(),
      children_ids: FoldMap::new(),
      children_zindexes: FoldMap::new(),
      root_id: INVALID_ROOT_ID,
      root_changes: 0,
      names: FoldMap::new(),
    }
  }

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
    if self.root_id != INVALID_ROOT_ID {
      debug_assert!(!self.children_ids.is_empty());
      let mut q: VecDeque<TreeNodeId> = VecDeque::new();
      q.push_back(self.root_id);
      while let Some(id) = q.pop_front() {
        if let Some(parent_id) = self.parent_ids.get(&id) {
          debug_assert!(self.children_ids.contains_key(&parent_id));
          debug_assert!(
            self
              .children_ids
              .get(&parent_id)
              .unwrap()
              .iter()
              .any(|i| *i == id)
          );
          debug_assert!(self.children_zindexes.contains_key(&parent_id));
        }
        if let Some(children_ids) = self.children_ids.get(&id) {
          for c in children_ids {
            debug_assert!(self.parent_ids.contains_key(c));
            debug_assert_eq!(*self.parent_ids.get(c).unwrap(), id);
          }

          for c in children_ids.iter() {
            q.push_back(*c);
          }
        }
      }
    } else {
      debug_assert!(self.children_ids.is_empty());
      debug_assert!(self.parent_ids.is_empty());
      debug_assert!(self.children_zindexes.is_empty());
    }
  }

  /// The first created node will be the root node.
  pub fn root_id(&self) -> TreeNodeId {
    self.root_id
  }

  fn _set_root(&mut self, id: TreeNodeId) {
    debug_assert_eq!(self.root_id, INVALID_ROOT_ID);
    debug_assert_eq!(self.root_changes, 0);
    self.root_id = id;
    if cfg!(debug_assertions) {
      self.root_changes += 1;
      debug_assert!(self.root_changes <= 1);
    }
  }

  fn _set_name(&mut self, id: TreeNodeId, name: &'static str) {
    if cfg!(debug_assertions) {
      self.names.insert(id, name);
    }
  }

  fn _unset_name(&mut self, id: TreeNodeId) {
    if cfg!(debug_assertions) {
      debug_assert!(self.names.contains_key(&id));
      self.names.remove(&id);
    }
  }

  // pub fn shape(&self, id: TreeNodeId) -> Option<IRect> {
  //   self.shapes.borrow().get(&id).copied()
  // }
  //
  // #[inline]
  // /// Set shape for a node. Since a node is always bounded by its parent, thus
  // /// its real shape can be different with the "expecting" shape.
  // ///
  // /// Returns the "real" shape after adjustment.
  // ///
  // /// There are two policies when calculating the "adjusted" shape:
  // /// - Truncate: Just cut all the parts that are out of its parent. For
  // ///   example a node shape is `((-5, -10), (5, 9))`, and its parent size is
  // ///   `(7, 8)`. This node's truncated shape is `((0, 0), (5, 8))`: its
  // ///   left-top corner must be at least `(0, 0)`, and its bottom-right corner
  // ///   is at most `(7, 8)`.
  // /// - Bound: Keep as much as we can, first try to set at most the same size
  // ///   as its parent, then move inside its parent thus avoid cutting any parts
  // ///   that is out of its parent. For example a node shape is
  // ///   `((-1, -2), (5, 6))`, and its parent size is `(6, 6)`. This node's
  // ///   bounded shape is `((0, 0), (6, 6))`: First its original width is 6
  // ///   which doesn't need to be truncated, but its original height is 8 so
  // ///   need to be truncated into 6, it becomes `((-1, -2), (5, 4))`. Then move
  // ///   it into parent to avoid more truncating, so its becomes
  // ///   `((0, 0), (6, 6))`.
  // pub fn set_shape(
  //   &mut self,
  //   id: TreeNodeId,
  //   shape: IRect,
  //   policy: RelationshipSetShapePolicy,
  // ) -> Option<IRect> {
  //   let result = match self.parent(id) {
  //     Some(parent_id) => {
  //       let parent_actual_shape = self.actual_shape(parent_id)?;
  //       let result = match policy {
  //         RelationshipSetShapePolicy::TRUNCATE => {
  //           truncate_shape(&shape, &parent_actual_shape.size())
  //         }
  //         RelationshipSetShapePolicy::BOUND => {
  //           bound_shape(&shape, &parent_actual_shape.size())
  //         }
  //       };
  //       result
  //     }
  //     None => {
  //       let min_x = num_traits::clamp_min(shape.min().x, 0);
  //       let min_y = num_traits::clamp_min(shape.min().y, 0);
  //       let max_x = num_traits::clamp_min(shape.max().x, min_x);
  //       let max_y = num_traits::clamp_min(shape.max().y, min_y);
  //       rect!(min_x, min_y, max_x, max_y)
  //     }
  //   };
  //   self.shapes.borrow_mut().insert(id, result);
  //   Some(result)
  // }
  //
  // #[inline]
  // pub fn actual_shape(&self, id: TreeNodeId) -> Option<U16Rect> {
  //   match self.parent(id) {
  //     None => {
  //       let shape = self.shape(id)?;
  //       Some(rect_as!(shape, u16))
  //     }
  //     Some(parent_id) => {
  //       let maybe_cached = self.cached_actual_shapes.borrow().get(&id).copied();
  //       match maybe_cached {
  //         Some(cached) => Some(cached),
  //         None => {
  //           // Non-root node truncated by its parent's shape.
  //           let shape = self.shape(id)?;
  //           let parent_actual_shape = self.actual_shape(parent_id)?;
  //           let actual_shape =
  //             convert_relative_to_absolute(&shape, &parent_actual_shape);
  //           self
  //             .cached_actual_shapes
  //             .borrow_mut()
  //             .insert(id, actual_shape);
  //           Some(actual_shape)
  //         }
  //       }
  //     }
  //   }
  // }
  //
  // #[inline]
  // /// Clear the cached actual_shapes since the provided id. All its
  // /// descendants actual_shape will be cleared as well.
  // fn clear_cached_actual_shapes(&mut self, id: TreeNodeId) {
  //   let mut q: VecDeque<TreeNodeId> = VecDeque::new();
  //   q.push_back(id);
  //   while let Some(parent_id) = q.pop_front() {
  //     self.cached_actual_shapes.borrow_mut().remove(&parent_id);
  //     if let Ok(children_ids) = self.children(parent_id) {
  //       for child_id in children_ids.iter() {
  //         q.push_back(*child_id);
  //       }
  //     }
  //   }
  // }
  //
  // #[inline]
  // /// Whether the node is visible, e.g. its actual_shape size is zero.
  // pub fn visible(&self, id: TreeNodeId) -> Option<bool> {
  //   let actual_shape = self.actual_shape(id)?;
  //   Some(!actual_shape.size().is_zero())
  // }
  //
  // #[inline]
  // pub fn invisible(&self, id: TreeNodeId) -> Option<bool> {
  //   self.visible(id).map(|v| !v)
  // }

  pub fn contains(&self, id: TreeNodeId) -> bool {
    self.children_ids.contains_key(&id)
  }

  pub fn contains_edge(
    &self,
    parent_id: TreeNodeId,
    child_id: TreeNodeId,
  ) -> bool {
    self.parent_ids.get(&child_id).copied() == Some(parent_id)
      && self
        .children_ids
        .get(&parent_id)
        .map(|children| children.iter().any(|c| *c == child_id))
        .unwrap_or(false)
  }

  pub fn parent(&self, id: TreeNodeId) -> Option<TreeNodeId> {
    self.parent_ids.get(&id).copied()
  }

  pub fn children(&self, id: TreeNodeId) -> Option<Vec<TreeNodeId>> {
    self.children_ids.get(&id).cloned()
  }

  pub fn children_zindex(&self, id: TreeNodeId) -> Option<usize> {
    self._internal_check();
    self.children_zindexes.get(&id).copied()
  }

  pub fn set_children_zindex(
    &mut self,
    id: TreeNodeId,
    value: usize,
  ) -> Option<usize> {
    self._internal_check();
    self.children_zindexes.insert(id, value)
  }

  /// Add the first node, which is the root node.
  pub fn add_root(&mut self, id: TreeNodeId, name: &'static str) {
    self._internal_check();
    debug_assert!(self.children_ids.is_empty());
    debug_assert!(self.parent_ids.is_empty());
    debug_assert_eq!(self.root_id, INVALID_ROOT_ID);
    self.children_ids.insert(id, vec![]);
    self._set_root(id);
    self._set_name(id, name);
  }

  /// Add the a new node ID, which is the child node of a parent node.
  ///
  /// NOTE: The parent ID must already exists, the child node ID must not
  /// exist.
  pub fn add_child(
    &mut self,
    parent_id: TreeNodeId,
    id: TreeNodeId,
    name: &'static str,
  ) {
    self._internal_check();
    debug_assert!(self.children_ids.contains_key(&parent_id));
    debug_assert!(!self.children_ids.contains_key(&id));
    debug_assert!(!self.parent_ids.contains_key(&id));
    self.children_ids.get_mut(&parent_id).unwrap().push(id);
    self.parent_ids.insert(id, parent_id);
    self._set_name(id, name);
  }

  pub fn remove_child(&mut self, parent_id: TreeNodeId, id: TreeNodeId) {
    self._internal_check();
    debug_assert_ne!(id, self.root_id);
    debug_assert!(self.children_ids.contains_key(&parent_id));
    debug_assert!(
      self
        .children_ids
        .get(&parent_id)
        .unwrap()
        .iter()
        .any(|i| *i == id)
    );
    debug_assert!(self.children_ids.contains_key(&id));
    debug_assert!(self.parent_ids.contains_key(&id));
    debug_assert_eq!(*self.parent_ids.get(&id).unwrap(), parent_id);

    let child_pos = self
      .children_ids
      .get(&parent_id)
      .unwrap()
      .iter()
      .find_position(|i| **i == id)
      .unwrap()
      .0;
    self
      .children_ids
      .get_mut(&parent_id)
      .unwrap()
      .remove(child_pos);
    self.parent_ids.remove(&id);
  }
}

impl Default for Relation {
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
  relation: Relation,
  ta: RefCell<Ta>,
}

impl<T> Itree<T>
where
  T: Inodeable,
{
  pub fn new() -> Self {
    Itree {
      nodes: FoldMap::new(),
      ta: RefCell::new(Ta::new()),
      relation: Relation::new(),
    }
  }

  fn _internal_check(&self) {
    debug_assert_eq!(self.relation.len(), self.nodes.len());
    debug_assert_eq!(self.ta.borrow().len(), self.nodes.len());
  }

  pub fn len(&self) -> usize {
    self.nodes.len()
  }

  pub fn is_empty(&self) -> bool {
    self.nodes.len() <= 1
  }

  pub fn root_id(&self) -> TreeNodeId {
    self.relation.root_id()
  }

  pub fn node_ids(&self) -> Vec<TreeNodeId> {
    self.nodes.keys().copied().collect()
  }

  pub fn parent_id(&self, id: TreeNodeId) -> Option<TreeNodeId> {
    self.relation.parent(id)
  }

  pub fn children_ids(&self, id: TreeNodeId) -> Vec<TreeNodeId> {
    self.relation.children(id).unwrap_or_default()
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
    ItreeIter::new(self, Some(self.relation.root_id()))
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

#[derive(Debug)]
/// When insert a node into a tree under a parent node, we will need to adjust
/// its logical shape and calculate its actual shape. This is because TaffyTree
/// can calculate larger layout result, which doesn't fit into terminal actual
/// shape. We have to truncate a node shape by its parent.
///
/// There are two policies when truncating a shape:
///
/// ## Neglect
/// Directly cut off the excess parts that are outside, for example:
///
/// ```
/// Original:
///
/// (-6,-3)    (4,-3)
///    +---------+
///    |C        |
///    |  (0,0)  |      (13,0)
///    |     +---+--------+
///    |     |   |       P|
///    +-----+---+        |
///  (-6,2)  | (4,2)      |
///          |            |
///          |            |
///          +------------+
///       (0,6)         (13,6)
///
/// Truncated:
///
///       (0,0) (4,0)      (13,0)
///          +---+--------+
///          |C  |       P|
///          +---+        |
///     (0,2)|  (4,2)     |
///          |            |
///          |            |
///          +------------+
///       (0,6)         (13,6)
/// ```
///
/// The shape of child C is `((-6, -3), (4, 2))`, its parent P size is
/// `(13, 6)`. C's truncated shape is `((0, 0), (4, 2))`.
///
/// ## Preserve
///
/// Preserve child shape as much as we can:
/// 1. Try to set the child size to be close to the size of its parent.
/// 2. Move it inside its parent to avoid been cut off, but if there's
///    still some parts outside, cut them off then.
/// For example:
///
/// ```
/// Original:
///
/// (-6,-3)    (4,-3)
///    +---------+
///    |C        |
///    |  (0,0)  |      (13,0)
///    |     +---+--------+
///    |     |   |       P|
///    +-----+---+        |
///  (-6,2)  | (4,2)      |
///          |            |
///          |            |
///          +------------+
///       (0,6)         (13,6)
///
/// Truncated:
///
///       (0,0)     (10,0)
///          +---+-----+--+ <-- (13,0)
///          |C        | P|
///          |         |  |
///          |         |  |
///          |   (10,5)|  |
///     (0,5)|---------+  |
///          +------------+
///       (0,6)         (13,6)
/// ```
///
/// The original C and P in the example is still the same, but C's size is
/// smaller than P, thus in 1st step we don't need to cut off. Then we can try
/// to move C inside P (with minimal movement), so the bounded shape of C
/// becomes `((0, 0), (10, 5))`.
pub enum TruncatePolicy {
  NEGLECT,
  PRESERVE,
}

// Insert/Remove {
impl<T> Itree<T>
where
  T: Inodeable,
{
  #[inline]
  fn _update_shapes_impl(&mut self, start_id: TreeNodeId) -> TaffyResult<()> {
    let mut q: VecDeque<TreeNodeId> = VecDeque::new();
    q.push_back(start_id);

    // Iterate all descendants, and update their shape/actual_shape.
    while let Some(id) = q.pop_front() {
      let layout = self.ta.borrow().layout(id)?.clone();
      let policy = self.node(id).unwrap().truncate_policy();
      let shape = rect_from_layout!(layout);
      let shape = self.calculate_shape(id, &shape, policy);
      let actual_shape = self.calculate_actual_shape(id, &shape);
      self.node_mut(id).unwrap().set_shape(shape);
      self.node_mut(id).unwrap().set_actual_shape(actual_shape);

      if let Ok(ta_children_ids) = self.ta.borrow().children(id) {
        for ta_child in ta_children_ids {
          q.push_back(ta_child);
        }
      }
    }

    Ok(())
  }

  fn clamp_shape(shape: &IRect) -> IRect {
    let min_x = num_traits::clamp_min(shape.min().x, 0);
    let min_y = num_traits::clamp_min(shape.min().y, 0);
    let max_x = num_traits::clamp_min(shape.max().x, min_x);
    let max_y = num_traits::clamp_min(shape.max().y, min_y);
    rect!(min_x, min_y, max_x, max_y)
  }

  fn calculate_shape(
    &self,
    id: TreeNodeId,
    shape: &IRect,
    policy: TruncatePolicy,
  ) -> IRect {
    match self.parent_id(id) {
      Some(parent_id) => {
        let parent_actual_shape = self.node(parent_id).unwrap().actual_shape();
        match policy {
          TruncatePolicy::NEGLECT => {
            truncate_shape(&shape, &parent_actual_shape.size())
          }
          TruncatePolicy::PRESERVE => {
            bound_shape(&shape, &parent_actual_shape.size())
          }
        }
      }
      None => Self::clamp_shape(shape),
    }
  }

  pub fn calculate_actual_shape(
    &self,
    id: TreeNodeId,
    shape: &IRect,
  ) -> U16Rect {
    match self.parent_id(id) {
      Some(parent_id) => {
        let parent_actual_shape = self.node(parent_id).unwrap().actual_shape();
        convert_relative_to_absolute(&shape, &parent_actual_shape)
      }
      None => {
        rect_as!(shape, u16)
      }
    }
  }

  /// Create a root node, which is the first node in the tree.
  /// Returns the root node ID.
  pub fn add_root<F>(
    &mut self,
    actual_shape: U16Rect,
    style: Style,
    constructor: F,
    name: &'static str,
  ) -> TaffyResult<TreeNodeId>
  where
    F: FnOnce(
      /* id */ TreeNodeId,
      /* shape */ IRect,
      /* actual_shape */ U16Rect,
    ) -> T,
  {
    self._internal_check();
    debug_assert!(self.nodes.is_empty());

    let (id, shape) = {
      let mut ta = self.ta.borrow_mut();
      let id = ta.new_leaf(style)?;
      ta.compute_layout(
        id,
        taffy::Size {
          width: taffy::AvailableSpace::from_length(
            actual_shape.size().width(),
          ),
          height: taffy::AvailableSpace::from_length(
            actual_shape.size().height(),
          ),
        },
      )?;
      let layout = ta.layout(id)?;
      let shape = rect_from_layout!(layout);
      let shape = Self::clamp_shape(&shape);
      (id, shape)
    };

    self.relation.add_root(id, name);
    self.relation.set_children_zindex(id, DEFAULT_ZINDEX);

    let mut node = constructor(id, shape, actual_shape);
    node.set_zindex(DEFAULT_ZINDEX);
    node.set_enabled(DEFAULT_ENABLED);
    node.set_shape(shape);
    node.set_actual_shape(actual_shape);
    self.nodes.insert(id, node);
    Ok(id)
  }

  /// Create a new child node in the tree, and insert it under a parent node.
  /// Returns the child node ID.
  pub fn add_child<F>(
    &mut self,
    parent_id: TreeNodeId,
    style: Style,
    zindex: usize,
    enabled: bool,
    policy: TruncatePolicy,
    constructor: F,
    name: &'static str,
  ) -> TaffyResult<TreeNodeId>
  where
    F: FnOnce(
      /* id */ TreeNodeId,
      /* shape */ IRect,
      /* actual_shape */ U16Rect,
    ) -> T,
  {
    self._internal_check();
    debug_assert!(self.nodes.contains_key(&parent_id));

    let (id, shape) = {
      let mut ta = self.ta.borrow_mut();
      if enabled {
        // Detect whether TaffyTree currently is on the Z-index layer, clear and
        // re-insert all the children nodes that are in the same layer of
        // current `zindex`.
        let children_zindex = self.relation.children_zindex(parent_id);
        if children_zindex.is_none() || children_zindex.unwrap() != zindex {
          // Clear all children nodes under this parent.
          ta.set_children(parent_id, &[]);

          // Re-inesrt all children nodes equals to the `zindex` to this parent.
          for child in self.children_ids(parent_id) {
            debug_assert!(self.node(child).is_some());
            let child_zindex = self.node(child).unwrap().zindex();
            if child_zindex == zindex {
              ta.add_child(parent_id, child);
            }
          }
          self.relation.set_children_zindex(parent_id, zindex);
        }

        let id = ta.new_with_parent(style, parent_id)?;
        ta.compute_layout(self.relation.root_id(), taffy::Size::MAX_CONTENT)?;
        let layout = ta.layout(id)?;
        (id, rect_from_layout!(layout))
      } else {
        // Where the child node is disabled, we simply mock it with parent's
        // shape.
        (ta.new_leaf(style)?, *self.node(parent_id).unwrap().shape())
      }
    };

    self.relation.add_child(parent_id, id, name);

    let shape = self.calculate_shape(id, &shape, policy);
    let actual_shape = self.calculate_actual_shape(id, &shape);
    let mut node = constructor(id, shape, actual_shape);
    node.set_zindex(zindex);
    node.set_enabled(enabled);
    node.set_shape(shape);
    node.set_actual_shape(actual_shape);
    self.nodes.insert(id, node);

    // After this new child node is created, it may also affected the other
    // children nodes under the same parent with the same Z-index, because the
    // layout is been changed.
    // Thus we have to update both shape and actual_shape for all the children
    // nodes under the parent, except this newly created child node because we
    // just had done it.
    let ta_children_ids = self.ta.borrow().children(parent_id);
    if let Ok(ta_children_ids) = ta_children_ids {
      for ta_child in ta_children_ids {
        // We don't have to update `id` again because we had just done it.
        if ta_child != id {
          self._update_shapes_impl(ta_child)?;
        }
      }
    }

    Ok(id)
  }

  /// Same with [`add_child`](Itree::add_child) method, with default values for
  /// below parameters:
  /// - zindex: 0
  /// - enabled: true
  /// - policy: Truncate
  ///
  /// NOTE: For cursor widget node, you should always use the bound policy to
  /// ensure it is inside its parent and avoid been cut off.
  pub fn add_child_with_defaults<F>(
    &mut self,
    parent_id: TreeNodeId,
    style: Style,
    constructor: F,
    name: &'static str,
  ) -> TaffyResult<TreeNodeId>
  where
    F: FnOnce(
      /* id */ TreeNodeId,
      /* shape */ IRect,
      /* actual_shape */ U16Rect,
    ) -> T,
  {
    self.add_child(
      parent_id,
      style,
      DEFAULT_ZINDEX,
      DEFAULT_ENABLED,
      TruncatePolicy::NEGLECT,
      constructor,
      name,
    )
  }

  /// Remove a child node, returns the removed node.
  pub fn remove_child(&mut self, id: TreeNodeId) -> Option<T> {
    // Cannot remove root node.
    debug_assert_ne!(id, self.relation.root_id());
    self._internal_check();

    // Remove child node from collection.
    let result = match self.nodes.remove(&id) {
      Some(removed) => {
        // Remove node/edge relationship.
        debug_assert!(self.relation.contains_id(id));
        // Remove edges between `id` and its parent.
        let relation_removed = self.relation.remove_child(id);
        debug_assert!(relation_removed);
        Some(removed)
      }
      None => {
        debug_assert!(!self.relation.contains_id(id));
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
                  bound_shape(&expected_shape, &parent_actual_size);
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
        self._update_shapes_impl(id, self.parent_id(id).unwrap());

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
                bound_shape(&expected_shape, &parent_actual_size);
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
