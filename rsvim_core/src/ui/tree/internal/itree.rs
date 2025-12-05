//! Internal tree.

use crate::prelude::*;
use crate::ui::tree::Tree;
use crate::ui::tree::TreeNode;
use crate::ui::tree::TreeNodeId;
use crate::ui::tree::internal::shapes;
use compact_str::CompactString;
use compact_str::ToCompactString;
use itertools::Itertools;
use std::cell::RefCell;
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

pub const INVALID_ROOT_ID: TreeNodeId = -1;

/// Next unique UI widget ID.
///
/// NOTE: Start from 100001, so be different from buffer ID.
pub fn next_node_id() -> TreeNodeId {
  static VALUE: AtomicI32 = AtomicI32::new(100001);
  VALUE.fetch_add(1, Ordering::Relaxed)
}

#[derive(Clone)]
pub struct Itree {
  lo: TaffyTree,
  nid2loid: FoldMap<TreeNodeId, taffy::NodeId>,
  loid2nid: FoldMap<taffy::NodeId, TreeNodeId>,

  // Cached shapes for each node.
  cached_actual_shapes: RefCell<FoldMap<TreeNodeId, U16Rect>>,

  // Root ID
  root_nid: TreeNodeId,

  // Node names for debugging.
  names: FoldMap<TreeNodeId, CompactString>,
}

rc_refcell_ptr!(Itree);

impl Itree {
  pub fn new() -> Self {
    let mut lo = TaffyTree::new();
    lo.disable_rounding();
    Self {
      lo,
      nid2loid: FoldMap::new(),
      loid2nid: FoldMap::new(),
      cached_actual_shapes: RefCell::new(FoldMap::new()),
      root_nid: INVALID_ROOT_ID,
      names: FoldMap::new(),
    }
  }

  pub fn len(&self) -> usize {
    self._internal_check();
    self.nid2loid.len()
  }

  pub fn is_empty(&self) -> bool {
    self._internal_check();
    self.nid2loid.is_empty()
  }

  pub fn set_root_id(&mut self, root_id: TreeNodeId) {
    self.root_nid = root_id;
  }

  #[cfg(not(test))]
  fn _internal_check(&self) {}

  #[cfg(test)]
  fn _internal_check(&self) {
    debug_assert_eq!(self.lo.total_node_count(), self.nid2loid.len());
    debug_assert_eq!(self.lo.total_node_count(), self.loid2nid.len());
    for (nid, loid) in self.nid2loid.iter() {
      debug_assert!(self.loid2nid.contains_key(loid));
      debug_assert_eq!(*self.loid2nid.get(loid).unwrap(), *nid);
      if let Some(parent_loid) = self.lo.parent(*loid) {
        debug_assert!(self.loid2nid.contains_key(&parent_loid));
      }
    }
    for (loid, nid) in self.loid2nid.iter() {
      debug_assert!(self.nid2loid.contains_key(nid));
      debug_assert_eq!(*self.nid2loid.get(nid).unwrap(), *loid);
    }
  }

  pub fn new_leaf(
    &mut self,
    style: Style,
    name: &str,
  ) -> TaffyResult<TreeNodeId> {
    self._internal_check();
    let loid = self.lo.new_leaf(style)?;
    let nid = next_node_id();
    self.nid2loid.insert(nid, loid);
    self.loid2nid.insert(loid, nid);
    self.names.insert(nid, name.to_compact_string());
    self._internal_check();
    Ok(nid)
  }

  pub fn compute_layout(
    &mut self,
    id: TreeNodeId,
    available_size: taffy::Size<AvailableSpace>,
  ) -> TaffyResult<()> {
    self._internal_check();
    let loid = self.nid2loid.get(&id).unwrap();
    let result = self.lo.compute_layout(*loid, available_size);
    self.clear_cached_actual_shapes(id);
    self._internal_check();
    result
  }

  pub fn layout(&self, id: TreeNodeId) -> TaffyResult<&Layout> {
    self._internal_check();
    let loid = self.nid2loid.get(&id).unwrap();
    self.lo.layout(*loid)
  }

  #[inline]
  pub fn shape(&self, id: TreeNodeId) -> TaffyResult<IRect> {
    let layout = self.layout(id)?;
    let shape = rect_from_layout!(layout);
    let shape = rect_as!(shape, isize);

    match self.parent(id) {
      Some(parent_id) => {
        let parent_actual_shape = self.actual_shape(parent_id)?;
        let bounded_shape = shapes::bound_shape(&shape, &parent_actual_shape);
        Ok(bounded_shape)
      }
      None => {
        let min_x = num_traits::clamp_min(shape.min().x, 0);
        let min_y = num_traits::clamp_min(shape.min().y, 0);
        let max_x = num_traits::clamp_min(shape.max().x, min_x);
        let max_y = num_traits::clamp_min(shape.max().y, min_y);
        let bounded_shape = rect!(min_x, min_y, max_x, max_y);
        Ok(bounded_shape)
      }
    }
  }

  pub fn style(&self, id: TreeNodeId) -> TaffyResult<&Style> {
    self._internal_check();
    let loid = self.nid2loid.get(&id).unwrap();
    self.lo.style(*loid)
  }

  pub fn set_style(&mut self, id: TreeNodeId, style: Style) -> TaffyResult<()> {
    self._internal_check();
    let loid = self.nid2loid.get(&id).unwrap();
    self.lo.set_style(*loid, style)
  }

  #[inline]
  /// Actual location/size in limited terminal device. The top-left location
  /// can never be negative.
  ///
  /// A node's shape is always truncated by its parent shape.
  /// Unless the node itself is the root node and doesn't have a parent, in
  /// such case, the root node logical shape does not need to be truncated.
  pub fn actual_shape(&self, id: TreeNodeId) -> TaffyResult<U16Rect> {
    self._internal_check();

    match self.parent(id) {
      None => {
        let shape = self.shape(id)?;
        Ok(rect_as!(shape, u16))
      }
      Some(parent_id) => {
        let maybe_cached = self.cached_actual_shapes.borrow().get(&id).copied();
        match maybe_cached {
          Some(cached) => Ok(cached),
          None => {
            // Non-root node truncated by its parent's shape.
            let shape = self.shape(id)?;
            let parent_actual_shape = self.actual_shape(parent_id)?;
            let actual_shape =
              shapes::convert_to_actual_shape(&shape, &parent_actual_shape);
            self
              .cached_actual_shapes
              .borrow_mut()
              .insert(id, actual_shape);
            Ok(actual_shape)
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
    id == self.root_nid || self.parent(id).is_some()
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
    let loid = self.nid2loid.get(&id)?;
    let parent_loid = self.lo.parent(*loid)?;
    self.loid2nid.get(&parent_loid).copied()
  }

  pub fn children(&self, id: TreeNodeId) -> TaffyResult<Vec<TreeNodeId>> {
    self._internal_check();
    let loid = self.nid2loid.get(&id).unwrap();
    let children_loids = self.lo.children(*loid)?;
    Ok(
      children_loids
        .iter()
        .map(|i| *self.loid2nid.get(i).unwrap())
        .collect_vec(),
    )
  }

  pub fn add_child(
    &mut self,
    parent_id: TreeNodeId,
    child_id: TreeNodeId,
  ) -> TaffyResult<()> {
    self._internal_check();
    let parent_loid = self.nid2loid.get(&parent_id).unwrap();
    let child_loid = self.nid2loid.get(&child_id).unwrap();
    self.lo.add_child(*parent_loid, *child_loid)
  }

  pub fn remove_child(
    &mut self,
    parent_id: TreeNodeId,
    child_id: TreeNodeId,
  ) -> TaffyResult<TreeNodeId> {
    self._internal_check();
    let parent_loid = self.nid2loid.get(&parent_id).unwrap();
    let child_loid = self.nid2loid.get(&child_id).unwrap();
    let removed_loid = self.lo.remove_child(*parent_loid, *child_loid)?;
    debug_assert_eq!(removed_loid, *child_loid);
    let removed_id = *self.loid2nid.get(&removed_loid).unwrap();
    debug_assert_eq!(removed_id, child_id);
    Ok(removed_id)
  }

  pub fn new_with_parent(
    &mut self,
    style: Style,
    parent_id: TreeNodeId,
    _name: &str,
  ) -> TaffyResult<TreeNodeId> {
    let id = self.new_leaf(style, _name)?;
    self.add_child(parent_id, id)?;
    Ok(id)
  }

  pub fn new_with_children(
    &mut self,
    style: Style,
    children: &[TreeNodeId],
    name: &str,
  ) -> TaffyResult<TreeNodeId> {
    self._internal_check();
    let children_loids = children
      .iter()
      .map(|i| *self.nid2loid.get(i).unwrap())
      .collect_vec();
    let loid = self.lo.new_with_children(style, &children_loids)?;
    let id = next_node_id();
    self.nid2loid.insert(id, loid);
    self.loid2nid.insert(loid, id);
    self.names.insert(id, name.to_compact_string());
    self._internal_check();
    Ok(id)
  }
}

impl Debug for Itree {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if self.root_nid == INVALID_ROOT_ID {
      f.write_str("Itree:Empty")
    } else {
      f.write_str("Itree:\n")?;
      let mut q: VecDeque<TreeNodeId> = VecDeque::new();
      q.push_back(self.root_nid);
      while let Some(id) = q.pop_front() {
        let layout = match self.layout(id) {
          Ok(layout) => {
            format!("{}({}):{:#?}\n", self.names.get(&id).unwrap(), id, layout)
          }
          Err(e) => {
            format!("{}({}):{:?}\n", self.names.get(&id).unwrap(), id, e)
          }
        };
        f.write_str(layout.as_str())?;
        if let Ok(children) = self.children(id) {
          for c in children {
            q.push_back(c);
          }
        }
      }
      Ok(())
    }
  }
}

impl Default for Itree {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug)]
/// The level-order iterator of the tree, start from tree root.
pub struct TreeIter<'a> {
  tree: &'a Tree,
  que: VecDeque<TreeNodeId>,
}

impl<'a> Iterator for TreeIter<'a> {
  type Item = &'a TreeNode;

  fn next(&mut self) -> Option<Self::Item> {
    if let Some(id) = self.que.pop_front() {
      if let Ok(children_ids) = self.tree.children_ids(id) {
        for child_id in children_ids {
          if self.tree.node(child_id).is_some() {
            self.que.push_back(child_id);
          }
        }
      }
      self.tree.node(id)
    } else {
      None
    }
  }
}

impl<'a> TreeIter<'a> {
  pub fn new(tree: &'a Tree, start_node_id: Option<TreeNodeId>) -> Self {
    let mut que = VecDeque::new();
    if let Some(id) = start_node_id {
      que.push_back(id);
    }
    Self { tree, que }
  }
}
