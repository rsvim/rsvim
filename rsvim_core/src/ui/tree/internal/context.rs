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
use taffy::prelude::FromLength;
use taffy::prelude::TaffyMaxContent;

pub const INVALID_ROOT_ID: TreeNodeId = -1;
pub const DEFAULT_ZINDEX: usize = 0;
pub const DEFAULT_ENABLED: bool = true;

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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Common attribute of a node.
pub struct Attribute {
  pub shape: IRect,
  pub actual_shape: U16Rect,
  pub zindex: usize,
  pub enabled: bool,
  pub truncate_policy: TruncatePolicy,
}

#[derive(Debug, Clone)]
// Maintains all nodes relationship of the tree, and all common attributes.
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
  attributes: FoldMap<TreeNodeId, Attribute>,

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
      attributes: FoldMap::new(),
      root: INVALID_ROOT_ID,
      #[cfg(debug_assertions)]
      root_changes: 0,
      #[cfg(debug_assertions)]
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
    }
  }

  /// The first created node will be the root node.
  pub fn root(&self) -> TreeNodeId {
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

  pub fn attribute(&self, id: TreeNodeId) -> Option<&Attribute> {
    self.attributes.get(&id)
  }

  pub fn set_attribute(&mut self, id: TreeNodeId, attribute: Attribute) {
    self.attributes.insert(id, attribute);
  }

  pub fn remove_attribute(&mut self, id: TreeNodeId) {
    self.attributes.remove(&id);
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

#[derive(Debug, Clone)]
pub struct TreeContext {
  ta: Ta,
  relation: Relation,
}

rc_refcell_ptr!(TreeContext);

impl TreeContext {
  pub fn new() -> Self {
    Self {
      ta: Ta::new(),
      relation: Relation::new(),
    }
  }

  pub fn is_empty(&self) -> bool {
    self._internal_check();
    self.relation.is_empty()
  }

  pub fn len(&self) -> usize {
    self._internal_check();
    self.relation.len()
  }

  pub fn contains(&self, id: TreeNodeId) -> bool {
    self.relation.contains(id)
  }

  pub fn root(&self) -> TreeNodeId {
    self.relation.root()
  }

  pub fn parent(&self, id: TreeNodeId) -> Option<TreeNodeId> {
    self.relation.parent(id)
  }

  pub fn children(&self, id: TreeNodeId) -> Option<Vec<TreeNodeId>> {
    self.relation.children(id)
  }

  pub fn attribute(&self, id: TreeNodeId) -> Option<&Attribute> {
    self.relation.attribute(id)
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
      self._refresh_ta_children_by_zindex(parent_id, zindex)?;
      self._update_shapes_for(parent_id)
    } else {
      Ok(())
    }
  }
}

impl TreeContext {
  fn _internal_check(&self) {
    debug_assert_eq!(self.relation.is_empty(), self.ta.is_empty());
    debug_assert_eq!(self.relation.len(), self.ta.len());

    if cfg!(test) && self.relation.len() > 0 {
      debug_assert_ne!(self.relation.root(), INVALID_ROOT_ID);
      let mut q: VecDeque<TreeNodeId> = VecDeque::new();
      q.push_back(self.relation.root());
      while let Some(id) = q.pop_front() {
        if let Ok(ta_children_ids) = self.ta.children(id) {
          let mut ta_zindex: Option<usize> = None;
          for ta_child in ta_children_ids {
            debug_assert!(self.relation.contains(ta_child));
            debug_assert!(self.relation.attribute(ta_child).is_some());
            if ta_zindex.is_none() {
              ta_zindex =
                Some(self.relation.attribute(ta_child).unwrap().zindex);
            } else {
              debug_assert_eq!(
                ta_zindex,
                Some(self.relation.attribute(ta_child).unwrap().zindex)
              );
            }
            q.push_back(ta_child);
          }
        }
      }
    }
  }

  /// Update shape/actual_shape for a node and all its children and
  /// descendants.
  fn _update_shapes_for(&mut self, start_id: TreeNodeId) -> TaffyResult<()> {
    let mut q: VecDeque<TreeNodeId> = VecDeque::new();
    q.push_back(start_id);

    // Iterate all descendants, and update their shape/actual_shape.
    while let Some(id) = q.pop_front() {
      let layout = self.ta.layout(id)?.clone();
      let policy = self.relation.attribute(id).unwrap().truncate_policy;
      let shape = rect_from_layout!(layout);
      let shape = self._adjust_shape(id, &shape, policy);
      let actual_shape = self._calculate_actual_shape(id, &shape);
      let mut attr = *self.relation.attribute(id).unwrap();
      attr.shape = shape;
      attr.actual_shape = actual_shape;
      self.relation.set_attribute(id, attr);

      if let Ok(ta_children_ids) = self.ta.children(id) {
        for ta_child in ta_children_ids {
          q.push_back(ta_child);
        }
      }
    }

    Ok(())
  }

  /// Update shape/actual_shape for all children and their descendants under a
  /// parent, except 1 child.
  fn _update_shapes_for_children_except(
    &mut self,
    parent_id: TreeNodeId,
    except_child_id: TreeNodeId,
  ) -> TaffyResult<()> {
    let ta_children_ids = self.ta.children(parent_id);
    if let Ok(ta_children_ids) = ta_children_ids {
      for ta_child in ta_children_ids {
        // We don't have to update `except_id` again because we had just done
        // it.
        if ta_child != except_child_id {
          self._update_shapes_for(ta_child)?;
        }
      }
    }
    Ok(())
  }

  /// Adjust the shape of a node, by its expected shape and the policy it
  /// follows.
  fn _adjust_shape(
    &self,
    id: TreeNodeId,
    shape: &IRect,
    policy: TruncatePolicy,
  ) -> IRect {
    match self.relation.parent(id) {
      Some(parent_id) => {
        let parent_actual_shape =
          self.relation.attribute(parent_id).unwrap().actual_shape;
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
    match self.relation.parent(id) {
      Some(parent_id) => {
        let parent_actual_shape =
          self.relation.attribute(parent_id).unwrap().actual_shape;
        shapes::convert_relative_to_absolute(&shape, &parent_actual_shape)
      }
      None => {
        let shape = shapes::clamp_shape(shape);
        rect_as!(shape, u16)
      }
    }
  }

  // Compute layout and update attribute of all nodes, after any changes
  // happened, such as:
  // - A node changed its style.
  // - A new node is added.
  // - A node is removed.
  fn _compute_layout(&mut self) {}

  // Detect whether TaffyTree currently is on the Z-index layer, clear and
  // re-insert all the children nodes that are in the same layer of
  // current `zindex`.
  pub fn _refresh_ta_children_by_zindex(
    &mut self,
    parent_id: TreeNodeId,
    target_zindex: usize,
  ) -> TaffyResult<()> {
    let children_ids = self.ta.children(parent_id);
    let has_children = children_ids
      .as_ref()
      .map(|children| !children.is_empty())
      .unwrap_or(false);
    let children_zindex = if has_children {
      self
        .relation
        .attribute(children_ids.unwrap()[0])
        .unwrap()
        .zindex
    } else {
      DEFAULT_ZINDEX
    };

    if !has_children || children_zindex != target_zindex {
      // Clear all children nodes under this parent.
      self.ta.set_children(parent_id, &[])?;

      // Re-inesrt all children nodes equals to the `zindex` to this parent.
      if let Some(children) = self.relation.children(parent_id) {
        for c in children {
          debug_assert!(self.relation.contains(c));
          let zindex = self.relation.attribute(c).unwrap().zindex;
          if zindex == target_zindex {
            self.ta.add_child(parent_id, c)?;
          }
        }
      }
    }
    Ok(())
  }

  /// Create a root node, which is the first node in the tree.
  /// Returns the root node ID.
  pub fn add_root(
    &mut self,
    actual_shape: U16Rect,
    style: Style,
    name: &'static str,
  ) -> TaffyResult<TreeNodeId> {
    self._internal_check();
    debug_assert!(self.relation.is_empty());

    let id = self.ta.new_leaf(style)?;
    self.relation.add_root(id, name);
    let shape = rect_as!(actual_shape, isize);
    let shape = shapes::clamp_shape(&shape);
    self.relation.set_attribute(
      id,
      Attribute {
        shape,
        actual_shape,
        zindex: DEFAULT_ZINDEX,
        enabled: DEFAULT_ENABLED,
        truncate_policy: TruncatePolicy::BRUTAL,
      },
    );

    let (id, shape) = {
      let id = self.ta.new_leaf(style)?;
      self.ta.compute_layout(
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
      let layout = self.ta.layout(id)?;
      let shape = rect_from_layout!(layout);
      let shape = shapes::clamp_shape(&shape);
      (id, shape)
    };

    self.relation.add_root(id, name);
    self.relation.set_attribute(
      id,
      Attribute {
        shape,
        actual_shape,
        zindex: DEFAULT_ZINDEX,
        enabled: DEFAULT_ENABLED,
        truncate_policy: TruncatePolicy::BRUTAL,
      },
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
    enabled: bool,
    truncate_policy: TruncatePolicy,
    name: &'static str,
  ) -> TaffyResult<TreeNodeId> {
    self._internal_check();
    debug_assert!(self.relation.contains(parent_id));

    let (id, shape) = {
      if enabled {
        self._refresh_ta_children_by_zindex(parent_id, zindex)?;
        let id = self.ta.new_with_parent(style, parent_id)?;

        self.ta.compute_layout(
          self.relation.root(),
          taffy::Size {
            width: taffy::AvailableSpace::from_length(
              actual_shape.size().width(),
            ),
            height: taffy::AvailableSpace::from_length(
              actual_shape.size().height(),
            ),
          },
        )?;
        let layout = self.ta.layout(id)?;
        (id, rect_from_layout!(layout))
      } else {
        // Where the child node is disabled, we simply mock it with parent's
        // shape.
        (
          self.ta.new_leaf(style)?,
          self.relation.attribute(parent_id).unwrap().shape,
        )
      }
    };

    let shape = self._adjust_shape(id, &shape, truncate_policy);
    let actual_shape = self._calculate_actual_shape(id, &shape);

    self.relation.add_child(parent_id, id, name);
    self.relation.set_attribute(
      id,
      Attribute {
        shape,
        actual_shape,
        zindex,
        enabled,
        truncate_policy,
      },
    );

    // After this new child node is created, it may also affected the other
    // children nodes under the same parent with the same Z-index, because the
    // layout is been changed.
    // Thus we have to update both shape and actual_shape for all the children
    // nodes under the parent, except this newly created child node because we
    // just had done it.
    self._update_shapes_for_children_except(parent_id, id)?;

    Ok(id)
  }

  /// Remove a child node.
  /// Returns the removed node.
  ///
  /// NOTE: Never remove the root node.
  pub fn remove_child(&mut self, id: TreeNodeId) -> TaffyResult<()> {
    self._internal_check();
    debug_assert_ne!(id, self.relation.root());
    debug_assert!(self.relation.contains(id));
    debug_assert!(self.relation.parent(id).is_some());
    let parent_id = self.relation.parent(id).unwrap();
    let attr = self.attribute(id).unwrap();
    let enabled = attr.enabled;
    let zindex = attr.zindex;
    self.ta.remove_child(parent_id, id);
    self.relation.remove_child(parent_id, id);
    self.relation.remove_attribute(id);

    // After this node is removed, if it is enabled, it can affect other
    // sibling nodes with the same Z-index.
    if enabled {
      self._refresh_ta_children_by_zindex(parent_id, zindex)?;
      self._update_shapes_for(parent_id)
    } else {
      Ok(())
    }
  }
}
