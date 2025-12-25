//! Internal tree context.

use crate::prelude::*;
use crate::ui::tree::TreeNodeId;
use crate::ui::tree::internal::shapes;
use itertools::Itertools;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::iter::Iterator;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering;
use taffy::AvailableSpace;
use taffy::Layout;
use taffy::Style;
use taffy::TaffyResult;
use taffy::TaffyTree;
use taffy::prelude::TaffyMaxContent;

pub const INVALID_ROOT_ID: TreeNodeId = -1;
pub const DEFAULT_ZINDEX: usize = 0;
pub const DEFAULT_TRUNCATE_POLICY: TruncatePolicy = TruncatePolicy::BRUTAL;

/// Next unique UI widget ID.
///
/// NOTE: Start from 100001, so be different from buffer ID.
pub fn next_node_id() -> TreeNodeId {
  static VALUE: AtomicI32 = AtomicI32::new(100001);
  VALUE.fetch_add(1, Ordering::Relaxed)
}

#[derive(Debug, Clone)]
pub struct Ta {
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

  pub fn is_empty(&self) -> bool {
    self._internal_check();
    self.id2taid.is_empty()
  }

  pub fn len(&self) -> usize {
    self._internal_check();
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// When insert a node into a tree under a parent node, we will need to adjust
/// its logical shape and calculate its actual shape. This is because TaffyTree
/// can calculate larger layout result, which doesn't fit into terminal actual
/// shape. We have to truncate a node shape by its parent.
///
/// There are two policies when truncating a shape:
///
/// ## Brutal
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
/// ## Reserved
///
/// Reserve child shape as much as we can:
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
  BRUTAL,
  RESERVED,
}

#[derive(Debug, Clone)]
pub struct TreeContext {
  ta: Ta,

  // Properties
  shapes: FoldMap<TreeNodeId, IRect>,
  actual_shapes: FoldMap<TreeNodeId, U16Rect>,
  zindexes: FoldMap<TreeNodeId, usize>,
  truncate_policies: FoldMap<TreeNodeId, TruncatePolicy>,

  // Root
  root: TreeNodeId,

  // For debugging
  #[cfg(debug_assertions)]
  root_changes: usize,
  #[cfg(debug_assertions)]
  names: FoldMap<TreeNodeId, &'static str>,
}

rc_refcell_ptr!(TreeContext);

impl TreeContext {
  pub fn new() -> Self {
    Self {
      ta: Ta::new(),
      shapes: FoldMap::new(),
      actual_shapes: FoldMap::new(),
      zindexes: FoldMap::new(),
      truncate_policies: FoldMap::new(),
      root: INVALID_ROOT_ID,
      #[cfg(debug_assertions)]
      root_changes: 0,
      #[cfg(debug_assertions)]
      names: FoldMap::new(),
    }
  }

  pub fn is_empty(&self) -> bool {
    self._internal_check();
    self.ta.is_empty()
  }

  pub fn len(&self) -> usize {
    self._internal_check();
    self.ta.len()
  }

  pub fn contains(&self, id: TreeNodeId) -> bool {
    self.ta.contains(id)
  }

  fn _set_root(&mut self, id: TreeNodeId) {
    debug_assert_eq!(self.root, INVALID_ROOT_ID);
    debug_assert_ne!(id, INVALID_ROOT_ID);
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

  /// The first created node will be the root node.
  pub fn root(&self) -> TreeNodeId {
    self.root
  }

  pub fn parent(&self, id: TreeNodeId) -> Option<TreeNodeId> {
    self.ta.parent(id)
  }

  pub fn children(&self, id: TreeNodeId) -> TaffyResult<Vec<TreeNodeId>> {
    self.ta.children(id)
  }

  pub fn shape(&self, id: TreeNodeId) -> Option<&IRect> {
    self.shapes.get(&id)
  }

  pub fn actual_shape(&self, id: TreeNodeId) -> Option<&U16Rect> {
    self.actual_shapes.get(&id)
  }

  pub fn zindex(&self, id: TreeNodeId) -> Option<usize> {
    self.zindexes.get(&id).copied()
  }

  pub fn set_zindex(&mut self, id: TreeNodeId, value: usize) -> Option<usize> {
    match self.zindexes.get_mut(&id) {
      Some(v) => {
        let result = *v;
        *v = value;
        Some(result)
      }
      None => None,
    }
  }

  pub fn truncate_policy(&self, id: TreeNodeId) -> Option<TruncatePolicy> {
    self.truncate_policies.get(&id).copied()
  }

  pub fn set_truncate_policy(
    &mut self,
    id: TreeNodeId,
    value: TruncatePolicy,
  ) -> Option<TruncatePolicy> {
    match self.truncate_policies.get_mut(&id) {
      Some(v) => {
        let result = *v;
        *v = value;
        Some(result)
      }
      None => None,
    }
  }

  pub fn disabled(&self, id: TreeNodeId) -> TaffyResult<bool> {
    self.ta.style(id).map(|s| s.display == taffy::Display::None)
  }

  pub fn enabled(&self, id: TreeNodeId) -> TaffyResult<bool> {
    self.disabled(id).map(|v| !v)
  }

  pub fn style(&self, id: TreeNodeId) -> TaffyResult<&Style> {
    self.ta.style(id)
  }

  pub fn set_style(&mut self, id: TreeNodeId, style: Style) -> TaffyResult<()> {
    self.ta.set_style(id, style)?;
    let parent_id = self.relation.parent(id).unwrap();
    let attr = self.relation.attribute(id).unwrap();
    let enabled = attr.enabled;
    let zindex = attr.zindex;
    if enabled {
      // If this node is enabled, changing its style will affect other sibling
      // nodes with the same Z-index.
      self._update_shapes(parent_id)
    } else {
      Ok(())
    }
  }
}

impl TreeContext {
  /// Update shape/actual_shape for a node and all its children and
  /// descendants.
  fn _update_shapes(&mut self) -> TaffyResult<()> {
    if self.root != INVALID_ROOT_ID {
      self
        .ta
        .compute_layout(self.root, taffy::Size::MAX_CONTENT)?;

      let mut q: VecDeque<TreeNodeId> = VecDeque::new();
      q.push_back(self.root);

      // Iterate all descendants, and update their shape/actual_shape.
      while let Some(id) = q.pop_front() {
        let layout = self.ta.layout(id)?.clone();
        let policy = self.truncate_policies.get(&id).copied().unwrap();
        let shape = rect_from_layout!(layout);
        let shape = self._truncate_shape(id, &shape, policy);
        let actual_shape = self._calculate_actual_shape(id, &shape);
        self.shapes.insert(id, shape);
        self.actual_shapes.insert(id, actual_shape);

        if let Ok(ta_children_ids) = self.ta.children(id) {
          for ta_child in ta_children_ids {
            q.push_back(ta_child);
          }
        }
      }
    }

    Ok(())
  }

  /// Truncate the shape of a node, by its expected shape and the policy it
  /// follows.
  fn _truncate_shape(
    &self,
    id: TreeNodeId,
    shape: &IRect,
    policy: TruncatePolicy,
  ) -> IRect {
    match self.ta.parent(id) {
      Some(parent_id) => {
        let parent_actual_shape =
          self.actual_shapes.get(&parent_id).copied().unwrap();
        match policy {
          TruncatePolicy::BRUTAL => {
            shapes::truncate_shape(&shape, &parent_actual_shape.size())
          }
          TruncatePolicy::RESERVED => {
            shapes::bound_shape(&shape, &parent_actual_shape.size())
          }
        }
      }
      None => shapes::clamp_shape(shape),
    }
  }

  /// Calculate the actual_shape of a node, by its adjusted shape and its
  /// parent's actual_shape.
  pub fn _calculate_actual_shape(
    &self,
    id: TreeNodeId,
    shape: &IRect,
  ) -> U16Rect {
    match self.ta.parent(id) {
      Some(parent_id) => {
        let parent_actual_shape =
          self.actual_shapes.get(&parent_id).copied().unwrap();
        shapes::convert_relative_to_absolute(&shape, &parent_actual_shape)
      }
      None => {
        let shape = shapes::clamp_shape(shape);
        rect_as!(shape, u16)
      }
    }
  }

  /// Create a root node, which is the first node in the tree.
  /// Returns the root node ID.
  pub fn add_root(
    &mut self,
    style: Style,
    name: &'static str,
  ) -> TaffyResult<TreeNodeId> {
    debug_assert!(self.ta.is_empty());

    let height = style.size.height.value() as isize;
    let width = style.size.width.value() as isize;

    let id = self.ta.new_leaf(style)?;

    self._set_root(id);
    self._set_name(id, name);
    self.zindexes.insert(id, DEFAULT_ZINDEX);
    self.truncate_policies.insert(id, DEFAULT_TRUNCATE_POLICY);

    self._update_shapes()?;

    debug_assert_eq!(height, self.shapes.get(&id).unwrap().height());
    debug_assert_eq!(width, self.shapes.get(&id).unwrap().width());
    debug_assert_eq!(
      height,
      self.actual_shapes.get(&id).unwrap().height() as isize
    );
    debug_assert_eq!(
      width,
      self.actual_shapes.get(&id).unwrap().width() as isize
    );

    Ok(id)
  }

  /// Create a new child node in the tree, and insert it under a parent node.
  /// Returns the child node ID.
  pub fn add_child(
    &mut self,
    parent_id: TreeNodeId,
    style: Style,
    zindex: usize,
    truncate_policy: TruncatePolicy,
    name: &'static str,
  ) -> TaffyResult<TreeNodeId> {
    debug_assert!(self.ta.contains(parent_id));

    let id = self.ta.new_with_parent(style, parent_id)?;

    self._set_name(id, name);
    self.zindexes.insert(id, zindex);
    self.truncate_policies.insert(id, truncate_policy);

    self._update_shapes()?;

    Ok(id)
  }

  /// Remove a child node.
  /// Returns the removed node.
  ///
  /// NOTE: Never remove the root node.
  pub fn remove_child(&mut self, id: TreeNodeId) -> TaffyResult<()> {
    debug_assert_ne!(id, self.root);
    debug_assert!(self.ta.contains(id));
    debug_assert!(self.ta.parent(id).is_some());

    let parent_id = self.ta.parent(id).unwrap();
    let removed_id = self.ta.remove_child(parent_id, id)?;
    debug_assert_eq!(removed_id, id);

    // Even we removed this child, we just remove its edge (e.g. parent-child
    // relationship) inside TaffyTree, but we don't really purge its properties
    // such as zindex, truncate_policy, etc.

    self._update_shapes()?;

    Ok(())
  }
}
