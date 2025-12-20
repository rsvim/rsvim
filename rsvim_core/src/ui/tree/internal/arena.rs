//! Internal tree arena (ok I don't know how to name it).

use crate::prelude::*;
use crate::ui::tree::TreeNodeId;
use crate::ui::tree::internal::inode::next_node_id;
use itertools::Itertools;
use std::fmt::Debug;
use std::iter::Iterator;
use taffy::AvailableSpace;
use taffy::Layout;
use taffy::Style;
use taffy::TaffyResult;
use taffy::TaffyTree;

#[derive(Debug, Clone)]
pub struct TaTree {
  ta: TaffyTree,
  // Maps TreeNodeId <==> taffy::NodeId.
  id2taid: FoldMap<TreeNodeId, taffy::NodeId>,
  taid2id: FoldMap<taffy::NodeId, TreeNodeId>,
}

impl TaTree {
  pub fn new() -> Self {
    Self {
      ta: TaffyTree::new(),
      id2taid: FoldMap::new(),
      taid2id: FoldMap::new(),
    }
  }

  pub fn is_empty(&self) -> bool {
    self.id2taid.is_empty()
  }

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

pub const INVALID_ROOT_ID: TreeNodeId = -1;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Attributes {
  pub shape: IRect,
  pub actual_shape: U16Rect,
  pub zindex: usize,
  pub enabled: bool,
  pub truncate_policy: TruncatePolicy,
}

#[derive(Debug, Clone)]
// Maintains all nodes relationship of the tree.
//
// NOTE: TaffyTree itself can also maintain parent/child relationship, but it
// has several limitations when we calculate the layout:
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
pub struct Relation {
  parent: FoldMap<TreeNodeId, TreeNodeId>,
  children: FoldMap<TreeNodeId, Vec<TreeNodeId>>,
  attributes: FoldMap<TreeNodeId, Attributes>,
  root: TreeNodeId,

  #[cfg(debug_assertions)]
  root_changes: usize,
  #[cfg(debug_assertions)]
  names: FoldMap<TreeNodeId, &'static str>,
}

impl Relation {
  pub fn new() -> Self {
    Self {
      parent: FoldMap::new(),
      children: FoldMap::new(),
      children_zindexes: FoldMap::new(),
      root: INVALID_ROOT_ID,
      root_changes: 0,
      names: FoldMap::new(),
    }
  }

  pub fn is_empty(&self) -> bool {
    self.children.is_empty()
  }

  #[allow(dead_code)]
  pub fn len(&self) -> usize {
    self.children.len()
  }

  #[cfg(not(test))]
  fn _internal_check(&self) {}

  #[cfg(test)]
  fn _internal_check(&self) {
    if self.root != INVALID_ROOT_ID {
      debug_assert!(!self.children.is_empty());
      let mut q: VecDeque<TreeNodeId> = VecDeque::new();
      q.push_back(self.root);
      while let Some(id) = q.pop_front() {
        if let Some(parent_id) = self.parent.get(&id) {
          debug_assert!(self.children.contains_key(&parent_id));
          debug_assert!(
            self
              .children
              .get(&parent_id)
              .unwrap()
              .iter()
              .any(|i| *i == id)
          );
          debug_assert!(self.children_zindexes.contains_key(&parent_id));
        }
        if let Some(children_ids) = self.children.get(&id) {
          for c in children_ids {
            debug_assert!(self.parent.contains_key(c));
            debug_assert_eq!(*self.parent.get(c).unwrap(), id);
          }

          for c in children_ids.iter() {
            q.push_back(*c);
          }
        }
      }
    } else {
      debug_assert!(self.children.is_empty());
      debug_assert!(self.parent.is_empty());
      debug_assert!(self.children_zindexes.is_empty());
    }
  }

  /// The first created node will be the root node.
  pub fn root_id(&self) -> TreeNodeId {
    self.root
  }

  fn _set_root(&mut self, id: TreeNodeId) {
    debug_assert_eq!(self.root, INVALID_ROOT_ID);
    debug_assert_eq!(self.root_changes, 0);
    self.root = id;
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
    self.children.contains_key(&id)
  }

  pub fn contains_edge(
    &self,
    parent_id: TreeNodeId,
    child_id: TreeNodeId,
  ) -> bool {
    self.parent.get(&child_id).copied() == Some(parent_id)
      && self
        .children
        .get(&parent_id)
        .map(|children| children.iter().any(|c| *c == child_id))
        .unwrap_or(false)
  }

  pub fn parent(&self, id: TreeNodeId) -> Option<TreeNodeId> {
    self.parent.get(&id).copied()
  }

  pub fn children(&self, id: TreeNodeId) -> Option<Vec<TreeNodeId>> {
    self.children.get(&id).cloned()
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
    debug_assert!(self.children.is_empty());
    debug_assert!(self.parent.is_empty());
    debug_assert_eq!(self.root, INVALID_ROOT_ID);
    self.children.insert(id, vec![]);
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
    debug_assert!(self.children.contains_key(&parent_id));
    debug_assert!(!self.children.contains_key(&id));
    debug_assert!(!self.parent.contains_key(&id));
    self.children.get_mut(&parent_id).unwrap().push(id);
    self.parent.insert(id, parent_id);
    self._set_name(id, name);
  }

  pub fn remove_child(&mut self, parent_id: TreeNodeId, id: TreeNodeId) {
    self._internal_check();
    debug_assert_ne!(id, self.root);
    debug_assert!(self.children.contains_key(&parent_id));
    debug_assert!(
      self
        .children
        .get(&parent_id)
        .unwrap()
        .iter()
        .any(|i| *i == id)
    );
    debug_assert!(self.children.contains_key(&id));
    debug_assert!(self.parent.contains_key(&id));
    debug_assert_eq!(*self.parent.get(&id).unwrap(), parent_id);

    let child_pos = self
      .children
      .get(&parent_id)
      .unwrap()
      .iter()
      .find_position(|i| **i == id)
      .unwrap()
      .0;
    self.children.get_mut(&parent_id).unwrap().remove(child_pos);
    self.parent.remove(&id);
  }
}

impl Default for Relation {
  fn default() -> Self {
    Self::new()
  }
}
